#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::needless_borrow)]

use crate::media_core::{
    compare_summary, create_dit_report_with_progress, dit_csv, dit_mhl, dit_txt,
    parse_ignore_patterns, pass_fail, ChecksumMethod, CompareMode, DitMetadata, DitReport,
    FileStatus, ProgressUpdate,
};
use anyhow::{Context, Result};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::ptr;

pub const SEDER_COMPARE_PATH_SIZE: u32 = 0;
pub const SEDER_COMPARE_PATH_SIZE_MODIFIED: u32 = 1;
pub const SEDER_COMPARE_PATH_SIZE_CHECKSUM: u32 = 2;

pub const SEDER_ROW_MATCHING: c_int = 0;
pub const SEDER_ROW_CHANGED: c_int = 1;
pub const SEDER_ROW_ONLY_IN_A: c_int = 2;
pub const SEDER_ROW_ONLY_IN_B: c_int = 3;
pub const SEDER_ROW_FOLDER_ONLY_IN_A: c_int = 4;
pub const SEDER_ROW_FOLDER_ONLY_IN_B: c_int = 5;

#[repr(C)]
pub struct SederDitRequest {
    pub source_path: *const c_char,
    pub destination_path: *const c_char,
    pub project_name: *const c_char,
    pub shoot_date: *const c_char,
    pub card_name: *const c_char,
    pub camera_id: *const c_char,
    pub ignore_patterns: *const c_char,
    pub compare_mode: u32,
    pub ignore_hidden_system: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SederDitSummary {
    pub only_a: u64,
    pub only_b: u64,
    pub changed: u64,
    pub matching: u64,
    pub total_files: u64,
    pub total_folders: u64,
    pub total_size: u64,
    pub pass: u8,
    pub mhl_available: u8,
    pub compare_mode: u32,
}

pub type SederProgressCallback = Option<
    extern "C" fn(
        phase: *const c_char,
        processed_files: u64,
        processed_bytes: u64,
        status: *const c_char,
        user_data: *mut c_void,
    ),
>;

struct FfiRow {
    status: c_int,
    relative_path: CString,
    size_a: Option<u64>,
    size_b: Option<u64>,
    checksum_a: Option<CString>,
    checksum_b: Option<CString>,
    is_folder: bool,
}

pub struct SederDitReportHandle {
    report: DitReport,
    rows: Vec<FfiRow>,
    summary: SederDitSummary,
}

fn compare_mode_from_raw(value: u32) -> Result<CompareMode> {
    match value {
        SEDER_COMPARE_PATH_SIZE => Ok(CompareMode::PathSize),
        SEDER_COMPARE_PATH_SIZE_MODIFIED => Ok(CompareMode::PathSizeModified),
        SEDER_COMPARE_PATH_SIZE_CHECKSUM => Ok(CompareMode::PathSizeChecksum),
        _ => anyhow::bail!("Unknown compare mode: {value}"),
    }
}

fn compare_mode_to_raw(value: CompareMode) -> u32 {
    match value {
        CompareMode::PathSize => SEDER_COMPARE_PATH_SIZE,
        CompareMode::PathSizeModified => SEDER_COMPARE_PATH_SIZE_MODIFIED,
        CompareMode::PathSizeChecksum => SEDER_COMPARE_PATH_SIZE_CHECKSUM,
    }
}

fn string_from_ptr(ptr: *const c_char, name: &str) -> Result<String> {
    if ptr.is_null() {
        anyhow::bail!("{name} is required");
    }
    let value = unsafe { CStr::from_ptr(ptr) };
    Ok(value
        .to_str()
        .with_context(|| format!("{name} must be valid UTF-8"))?
        .to_string())
}

fn optional_string_from_ptr(ptr: *const c_char) -> Result<String> {
    if ptr.is_null() {
        return Ok(String::new());
    }
    Ok(unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .context("String argument must be valid UTF-8")?
        .to_string())
}

fn cstring(value: impl AsRef<str>) -> CString {
    CString::new(value.as_ref().replace('\0', " ")).expect("interior NULs are stripped")
}

fn string_into_raw(value: impl AsRef<str>) -> *mut c_char {
    cstring(value).into_raw()
}

fn set_error(error_out: *mut *mut c_char, message: impl AsRef<str>) {
    if !error_out.is_null() {
        unsafe {
            *error_out = string_into_raw(message);
        }
    }
}

fn status_to_raw(status: &FileStatus) -> c_int {
    match status {
        FileStatus::Matching => SEDER_ROW_MATCHING,
        FileStatus::Changed => SEDER_ROW_CHANGED,
        FileStatus::OnlyInA => SEDER_ROW_ONLY_IN_A,
        FileStatus::OnlyInB => SEDER_ROW_ONLY_IN_B,
    }
}

fn build_rows(report: &DitReport) -> Vec<FfiRow> {
    let mut rows = report
        .comparison
        .rows
        .iter()
        .map(|row| FfiRow {
            status: status_to_raw(&row.status),
            relative_path: cstring(&row.relative_path),
            size_a: row.size_a,
            size_b: row.size_b,
            checksum_a: row.checksum_a.as_ref().map(cstring),
            checksum_b: row.checksum_b.as_ref().map(cstring),
            is_folder: false,
        })
        .collect::<Vec<_>>();
    rows.extend(
        report
            .comparison
            .folders_only_in_a
            .iter()
            .map(|path| FfiRow {
                status: SEDER_ROW_FOLDER_ONLY_IN_A,
                relative_path: cstring(path),
                size_a: None,
                size_b: None,
                checksum_a: None,
                checksum_b: None,
                is_folder: true,
            }),
    );
    rows.extend(
        report
            .comparison
            .folders_only_in_b
            .iter()
            .map(|path| FfiRow {
                status: SEDER_ROW_FOLDER_ONLY_IN_B,
                relative_path: cstring(path),
                size_a: None,
                size_b: None,
                checksum_a: None,
                checksum_b: None,
                is_folder: true,
            }),
    );
    rows
}

fn build_summary(report: &DitReport) -> SederDitSummary {
    let (only_a, only_b, changed, matching) = compare_summary(&report.comparison);
    SederDitSummary {
        only_a: only_a as u64,
        only_b: only_b as u64,
        changed: changed as u64,
        matching: matching as u64,
        total_files: report.comparison.total_files as u64,
        total_folders: report.comparison.total_folders as u64,
        total_size: report.comparison.total_size,
        pass: (pass_fail(&report.comparison) == "PASS") as u8,
        mhl_available: (report.compare_mode == CompareMode::PathSizeChecksum) as u8,
        compare_mode: compare_mode_to_raw(report.compare_mode),
    }
}

fn progress_bridge(
    update: ProgressUpdate,
    callback: SederProgressCallback,
    user_data: *mut c_void,
) {
    if let Some(callback) = callback {
        let phase = cstring(update.phase);
        let status = cstring(update.status);
        callback(
            phase.as_ptr(),
            update.processed_files,
            update.processed_bytes,
            status.as_ptr(),
            user_data,
        );
    }
}

fn compare_impl(
    request: &SederDitRequest,
    callback: SederProgressCallback,
    user_data: *mut c_void,
) -> Result<*mut SederDitReportHandle> {
    let source_path = string_from_ptr(request.source_path, "source_path")?;
    let destination_path = string_from_ptr(request.destination_path, "destination_path")?;
    let project_name = optional_string_from_ptr(request.project_name)?;
    let shoot_date = optional_string_from_ptr(request.shoot_date)?;
    let card_name = optional_string_from_ptr(request.card_name)?;
    let camera_id = optional_string_from_ptr(request.camera_id)?;
    let ignore_patterns =
        parse_ignore_patterns(&optional_string_from_ptr(request.ignore_patterns)?);
    let mode = compare_mode_from_raw(request.compare_mode)?;
    let metadata = DitMetadata {
        project_name,
        shoot_date,
        card_name,
        camera_id,
        source_path: source_path.clone(),
        destination_path: destination_path.clone(),
        checksum_method: ChecksumMethod::Blake3,
    };

    let mut progress = |update| progress_bridge(update, callback, user_data);
    let report = create_dit_report_with_progress(
        Path::new(&source_path),
        Path::new(&destination_path),
        metadata,
        mode,
        request.ignore_hidden_system != 0,
        ignore_patterns,
        &mut progress,
    )?;
    let rows = build_rows(&report);
    let summary = build_summary(&report);
    Ok(Box::into_raw(Box::new(SederDitReportHandle {
        report,
        rows,
        summary,
    })))
}

#[no_mangle]
pub extern "C" fn seder_dit_compare(
    request: *const SederDitRequest,
    callback: SederProgressCallback,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut SederDitReportHandle {
    if !error_out.is_null() {
        unsafe {
            *error_out = ptr::null_mut();
        }
    }
    if request.is_null() {
        set_error(error_out, "Request is required");
        return ptr::null_mut();
    }
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        compare_impl(&*request, callback, user_data)
    }));
    match result {
        Ok(Ok(handle)) => handle,
        Ok(Err(err)) => {
            set_error(error_out, err.to_string());
            ptr::null_mut()
        }
        Err(_) => {
            set_error(error_out, "DIT comparison panicked");
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_free(handle: *mut SederDitReportHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[no_mangle]
pub extern "C" fn seder_string_free(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            drop(CString::from_raw(value));
        }
    }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_row_count(handle: *const SederDitReportHandle) -> u64 {
    if handle.is_null() {
        return 0;
    }
    unsafe { (*handle).rows.len() as u64 }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_row_status(
    handle: *const SederDitReportHandle,
    row: u64,
) -> c_int {
    if handle.is_null() {
        return -1;
    }
    unsafe {
        (&(*handle).rows)
            .get(row as usize)
            .map(|row| row.status)
            .unwrap_or(-1)
    }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_row_is_folder(
    handle: *const SederDitReportHandle,
    row: u64,
) -> u8 {
    if handle.is_null() {
        return 0;
    }
    unsafe {
        (&(*handle).rows)
            .get(row as usize)
            .map(|row| row.is_folder as u8)
            .unwrap_or(0)
    }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_row_path(
    handle: *const SederDitReportHandle,
    row: u64,
) -> *const c_char {
    if handle.is_null() {
        return ptr::null();
    }
    unsafe {
        (&(*handle).rows)
            .get(row as usize)
            .map(|row| row.relative_path.as_ptr())
            .unwrap_or(ptr::null())
    }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_row_size_a(
    handle: *const SederDitReportHandle,
    row: u64,
    value_out: *mut u64,
) -> u8 {
    row_size(handle, row, value_out, true)
}

#[no_mangle]
pub extern "C" fn seder_dit_report_row_size_b(
    handle: *const SederDitReportHandle,
    row: u64,
    value_out: *mut u64,
) -> u8 {
    row_size(handle, row, value_out, false)
}

fn row_size(handle: *const SederDitReportHandle, row: u64, value_out: *mut u64, left: bool) -> u8 {
    if handle.is_null() || value_out.is_null() {
        return 0;
    }
    let value = unsafe {
        (&(*handle).rows)
            .get(row as usize)
            .and_then(|row| if left { row.size_a } else { row.size_b })
    };
    if let Some(value) = value {
        unsafe {
            *value_out = value;
        }
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_row_checksum_a(
    handle: *const SederDitReportHandle,
    row: u64,
) -> *const c_char {
    row_checksum(handle, row, true)
}

#[no_mangle]
pub extern "C" fn seder_dit_report_row_checksum_b(
    handle: *const SederDitReportHandle,
    row: u64,
) -> *const c_char {
    row_checksum(handle, row, false)
}

fn row_checksum(handle: *const SederDitReportHandle, row: u64, left: bool) -> *const c_char {
    if handle.is_null() {
        return ptr::null();
    }
    unsafe {
        (&(*handle).rows)
            .get(row as usize)
            .and_then(|row| {
                if left {
                    row.checksum_a.as_ref()
                } else {
                    row.checksum_b.as_ref()
                }
            })
            .map(|value| value.as_ptr())
            .unwrap_or(ptr::null())
    }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_summary(
    handle: *const SederDitReportHandle,
    summary_out: *mut SederDitSummary,
) -> u8 {
    if handle.is_null() || summary_out.is_null() {
        return 0;
    }
    unsafe {
        *summary_out = (*handle).summary;
    }
    1
}

#[no_mangle]
pub extern "C" fn seder_dit_report_mhl_available(handle: *const SederDitReportHandle) -> u8 {
    if handle.is_null() {
        return 0;
    }
    unsafe { (*handle).summary.mhl_available }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_export_txt(handle: *const SederDitReportHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }
    unsafe { string_into_raw(dit_txt(&(*handle).report)) }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_export_csv(handle: *const SederDitReportHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }
    unsafe { string_into_raw(dit_csv(&(*handle).report)) }
}

#[no_mangle]
pub extern "C" fn seder_dit_report_export_mhl(handle: *const SederDitReportHandle) -> *mut c_char {
    if handle.is_null() || seder_dit_report_mhl_available(handle) == 0 {
        return ptr::null_mut();
    }
    unsafe { string_into_raw(dit_mhl(&(*handle).report)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn returns_error_for_missing_request() {
        let mut error = ptr::null_mut();
        let handle = seder_dit_compare(ptr::null(), None, ptr::null_mut(), &mut error);

        assert!(handle.is_null());
        assert!(!error.is_null());
        unsafe {
            assert_eq!(
                CStr::from_ptr(error).to_str().unwrap(),
                "Request is required"
            );
        }
        seder_string_free(error);
    }

    #[test]
    fn compares_and_exports_through_ffi() {
        let a = tempdir().unwrap();
        let b = tempdir().unwrap();
        write(&a.path().join("A001/clip.mov"), "abcd");
        write(&b.path().join("A001/clip.mov"), "abcd");

        let source = cstring(a.path().display().to_string());
        let destination = cstring(b.path().display().to_string());
        let project = cstring("Project");
        let shoot_date = cstring("2026-04-29");
        let card = cstring("A001");
        let camera = cstring("A");
        let ignore = cstring(".DS_Store\nThumbs.db");
        let request = SederDitRequest {
            source_path: source.as_ptr(),
            destination_path: destination.as_ptr(),
            project_name: project.as_ptr(),
            shoot_date: shoot_date.as_ptr(),
            card_name: card.as_ptr(),
            camera_id: camera.as_ptr(),
            ignore_patterns: ignore.as_ptr(),
            compare_mode: SEDER_COMPARE_PATH_SIZE_CHECKSUM,
            ignore_hidden_system: 1,
        };
        let mut error = ptr::null_mut();
        let handle = seder_dit_compare(&request, None, ptr::null_mut(), &mut error);

        assert!(!handle.is_null());
        assert!(error.is_null());
        assert_eq!(seder_dit_report_row_count(handle), 1);

        let mut summary = SederDitSummary::default();
        assert_eq!(seder_dit_report_summary(handle, &mut summary), 1);
        assert_eq!(summary.matching, 1);
        assert_eq!(summary.pass, 1);
        assert_eq!(summary.mhl_available, 1);

        let txt = seder_dit_report_export_txt(handle);
        assert!(!txt.is_null());
        unsafe {
            assert!(CStr::from_ptr(txt)
                .to_str()
                .unwrap()
                .contains("Compare mode: Path + Size + Checksum"));
        }
        seder_string_free(txt);
        seder_dit_report_free(handle);
    }
}

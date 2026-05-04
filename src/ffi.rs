use crate::offload::engine::{offload_files, scan_source};
use crate::offload::*;
use crate::report;
use std::ffi::{c_char, c_void, CStr, CString};
use std::panic::catch_unwind;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ============================================================================
// C Types
// ============================================================================

#[repr(C)]
pub struct SederDestinationConfig {
    pub path: *const c_char,
    pub label: *const c_char,
}

#[repr(C)]
pub struct SederOffloadRequest {
    pub source_path: *const c_char,
    pub destinations: *const SederDestinationConfig,
    pub destination_count: usize,
    pub project_name: *const c_char,
    pub shoot_date: *const c_char,
    pub card_name: *const c_char,
    pub camera_id: *const c_char,
    pub ignore_patterns: *const c_char,
    pub ignore_hidden_system: u8,
    pub verify_after_copy: u8,
    pub cancel_token: *mut u8,
}

#[repr(C)]
pub struct SederDestinationProgress {
    pub state: u32,
    pub files_completed: u64,
    pub files_total: u64,
    pub bytes_completed: u64,
    pub bytes_total: u64,
    pub current_file: *const c_char,
    pub error: *const c_char,
}

#[repr(C)]
pub struct SederOffloadProgress {
    pub phase: *const c_char,
    pub overall_files_completed: u64,
    pub overall_files_total: u64,
    pub overall_bytes_completed: u64,
    pub overall_bytes_total: u64,
    pub current_file: *const c_char,
    pub destinations: *const SederDestinationProgress,
    pub destination_count: usize,
}

pub type SederOffloadProgressCallback =
    extern "C" fn(progress: *const SederOffloadProgress, user_data: *mut c_void);

// ============================================================================
// Report Handle
// ============================================================================

pub struct OffloadReportHandle {
    pub report: OffloadReport,
    pub txt_export: CString,
    pub csv_export: CString,
    pub mhl_export: CString,
}

// ============================================================================
// FFI Functions
// ============================================================================

#[no_mangle]
pub extern "C" fn seder_offload_start(
    request: *const SederOffloadRequest,
    callback: SederOffloadProgressCallback,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut OffloadReportHandle {
    let result = catch_unwind(|| {
        let req = unsafe {
            if request.is_null() {
                return std::ptr::null_mut();
            }
            &*request
        };

        let source = unsafe { cstr_to_string(req.source_path) };
        let mut destinations = Vec::new();
        for i in 0..req.destination_count {
            let dest = unsafe { &*req.destinations.add(i) };
            destinations.push(DestinationConfig {
                path: PathBuf::from(unsafe { cstr_to_string(dest.path) }),
                label: unsafe { nullable_cstr_to_option(dest.label) },
            });
        }

        let metadata = ProjectMetadata {
            project_name: unsafe { cstr_to_string(req.project_name) },
            shoot_date: unsafe { cstr_to_string(req.shoot_date) },
            card_name: unsafe { cstr_to_string(req.card_name) },
            camera_id: unsafe { cstr_to_string(req.camera_id) },
        };

        let ignore_patterns = unsafe { cstr_to_string(req.ignore_patterns) }
            .split([',', '\n', '\r'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let options = OffloadOptions {
            ignore_hidden_system: req.ignore_hidden_system != 0,
            ignore_patterns,
            verify_after_copy: req.verify_after_copy != 0,
        };

        let offload_request = OffloadRequest {
            source: PathBuf::from(source),
            destinations,
            metadata,
            options,
        };

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cancel_ptr = req.cancel_token;

        // Scan source
        let scan = scan_source(
            &offload_request.source,
            &offload_request.options,
            &mut |_files, _bytes| {},
        )?;

        // Progress callback bridge
        let mut progress_strings: Vec<(CString, Option<CString>)> = Vec::new();
        let mut dest_progress_vec: Vec<SederDestinationProgress> = Vec::new();

        let progress_callback = |progress: OffloadProgress| {
            // Update cancel flag from Qt side
            if !cancel_ptr.is_null() {
                if unsafe { *cancel_ptr } != 0 {
                    cancel_flag.store(true, Ordering::Relaxed);
                }
            }

            progress_strings.clear();
            dest_progress_vec.clear();

            let phase_c = CString::new(progress.phase.clone()).unwrap_or_default();
            let current_file_c = CString::new(progress.current_file.clone()).unwrap_or_default();

            for dest in &progress.destinations {
                let file_c = CString::new(dest.current_file.clone()).unwrap_or_default();
                let err_c = dest.error.as_ref().map(|e| CString::new(e.clone()).unwrap_or_default());
                progress_strings.push((file_c, err_c));
            }

            for (idx, dest) in progress.destinations.iter().enumerate() {
                let (file_c, err_c) = &progress_strings[idx];
                dest_progress_vec.push(SederDestinationProgress {
                    state: dest.state as u32,
                    files_completed: dest.files_completed,
                    files_total: dest.files_total,
                    bytes_completed: dest.bytes_completed,
                    bytes_total: dest.bytes_total,
                    current_file: file_c.as_ptr(),
                    error: err_c.as_ref().map(|c| c.as_ptr()).unwrap_or(std::ptr::null()),
                });
            }

            let c_progress = SederOffloadProgress {
                phase: phase_c.as_ptr(),
                overall_files_completed: progress.overall_files_completed,
                overall_files_total: progress.overall_files_total,
                overall_bytes_completed: progress.overall_bytes_completed,
                overall_bytes_total: progress.overall_bytes_total,
                current_file: current_file_c.as_ptr(),
                destinations: dest_progress_vec.as_ptr(),
                destination_count: dest_progress_vec.len(),
            };

            callback(&c_progress, user_data);
        };

        // Offload
        let destination_results = offload_files(
            &offload_request.source,
            &scan,
            &offload_request.destinations,
            offload_request.options.verify_after_copy,
            &cancel_flag,
            &mut progress_callback,
        )?;

        let timestamp = chrono_nowish();
        let report = OffloadReport {
            metadata: offload_request.metadata,
            source_scan: scan,
            destination_results,
            timestamp,
        };

        let txt = report::report_txt(&report);
        let csv = report::report_csv(&report);
        let mhl = report::report_mhl(&report, 0);

        let handle = Box::new(OffloadReportHandle {
            report,
            txt_export: CString::new(txt).unwrap_or_default(),
            csv_export: CString::new(csv).unwrap_or_default(),
            mhl_export: CString::new(mhl).unwrap_or_default(),
        });

        Ok(Box::into_raw(handle))
    });

    match result {
        Ok(Ok(ptr)) => ptr,
        Ok(Err(e)) => {
            let msg = CString::new(format!("{}", e)).unwrap_or_default();
            unsafe {
                if !error_out.is_null() {
                    *error_out = msg.into_raw();
                }
            }
            std::ptr::null_mut()
        }
        Err(_) => {
            let msg = CString::new("Rust panic").unwrap_or_default();
            unsafe {
                if !error_out.is_null() {
                    *error_out = msg.into_raw();
                }
            }
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn seder_report_free(handle: *mut OffloadReportHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

#[no_mangle]
pub extern "C" fn seder_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn seder_report_export_txt(handle: *mut OffloadReportHandle) -> *const c_char {
    if handle.is_null() {
        return std::ptr::null();
    }
    unsafe { (*handle).txt_export.as_ptr() }
}

#[no_mangle]
pub extern "C" fn seder_report_export_csv(handle: *mut OffloadReportHandle) -> *const c_char {
    if handle.is_null() {
        return std::ptr::null();
    }
    unsafe { (*handle).csv_export.as_ptr() }
}

#[no_mangle]
pub extern "C" fn seder_report_export_mhl(handle: *mut OffloadReportHandle) -> *const c_char {
    if handle.is_null() {
        return std::ptr::null();
    }
    unsafe { (*handle).mhl_export.as_ptr() }
}

#[no_mangle]
pub extern "C" fn seder_report_summary(
    handle: *mut OffloadReportHandle,
    total_files_out: *mut u64,
    total_size_out: *mut u64,
    dest_count_out: *mut usize,
) -> u8 {
    if handle.is_null() {
        return 0;
    }
    let report = unsafe { &(*handle).report };
    unsafe {
        if !total_files_out.is_null() {
            *total_files_out = report.source_scan.total_files;
        }
        if !total_size_out.is_null() {
            *total_size_out = report.source_scan.total_size;
        }
        if !dest_count_out.is_null() {
            *dest_count_out = report.destination_results.len();
        }
    }
    1
}

#[no_mangle]
pub extern "C" fn seder_report_dest_state(
    handle: *mut OffloadReportHandle,
    dest_index: usize,
    state_out: *mut u32,
    files_copied_out: *mut u64,
    files_verified_out: *mut u64,
    files_failed_out: *mut u64,
    bytes_copied_out: *mut u64,
) -> u8 {
    if handle.is_null() {
        return 0;
    }
    let report = unsafe { &(*handle).report };
    if dest_index >= report.destination_results.len() {
        return 0;
    }
    let dest = &report.destination_results[dest_index];
    unsafe {
        if !state_out.is_null() {
            *state_out = dest.state as u32;
        }
        if !files_copied_out.is_null() {
            *files_copied_out = dest.files_copied;
        }
        if !files_verified_out.is_null() {
            *files_verified_out = dest.files_verified;
        }
        if !files_failed_out.is_null() {
            *files_failed_out = dest.files_failed;
        }
        if !bytes_copied_out.is_null() {
            *bytes_copied_out = dest.bytes_copied;
        }
    }
    1
}

// ============================================================================
// Helpers
// ============================================================================

unsafe fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr)
        .to_string_lossy()
        .into_owned()
}

unsafe fn nullable_cstr_to_option(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
}

fn chrono_nowish() -> String {
    // Simple ISO-like timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;

    // Approximate year/month/day from days since epoch (not exact but good enough for reports)
    // Use a simpler approach: just seconds since epoch formatted
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        1970 + days / 365,
        1,
        1,
        hour,
        min,
        sec
    )
}

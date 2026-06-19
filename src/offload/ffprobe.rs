//! ffprobe-based clip metadata extraction.
//!
//! This module shells out to `ffprobe` (from the FFmpeg suite). It is a
//! soft dependency: we discover the binary at runtime and fail closed
//! if it is missing. Nothing in the offload pipeline branches on this
//! information; the metadata is purely additive, surfaced through an
//! optional JSON sidecar report.

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Hard ceiling for an ffprobe invocation, in case a file probes hang
/// (broken codec, network read stall). Five seconds is generous for
/// `ffprobe -of json` against a single file.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipMetadata {
    /// Primary video codec, e.g. "prores", "h264", "dnxhd". Empty if no
    /// video stream (e.g. WAV audio file).
    pub video_codec: String,
    pub width: u32,
    pub height: u32,
    /// Frame rate numerator / denominator from r_frame_rate.
    pub fps_num: u32,
    pub fps_den: u32,
    /// Total duration in seconds (best-effort: video stream first, then
    /// container, then 0.0).
    pub duration_seconds: f64,
    /// Primary audio codec, empty if no audio stream.
    pub audio_codec: String,
    pub audio_channels: u32,
    pub audio_sample_rate: u32,
    /// SMPTE timecode if present in any stream tags, otherwise None.
    pub timecode: Option<String>,
    /// Color space tag from the video stream, otherwise empty.
    pub color_space: String,
}

impl ClipMetadata {
    /// Pretty `frame_rate` like "24000/1001" → "23.976".
    pub fn fps_display(&self) -> String {
        if self.fps_den == 0 {
            return String::new();
        }
        let fps = self.fps_num as f64 / self.fps_den as f64;
        if (fps - fps.round()).abs() < 0.001 {
            format!("{:.0}", fps)
        } else {
            format!("{:.3}", fps)
        }
    }
}

/// Find the `ffprobe` binary, falling back to common install locations
/// when PATH is incomplete (e.g. macOS GUI launches inherit a stunted PATH).
pub fn discover() -> Option<PathBuf> {
    discover_binary("ffprobe")
}

/// Sibling of `discover()` for `ffmpeg` itself. Used by future proxy
/// generation; exposed here so the Qt UI can show a unified "ffmpeg
/// suite available" badge.
pub fn discover_ffmpeg() -> Option<PathBuf> {
    discover_binary("ffmpeg")
}

fn discover_binary(name: &str) -> Option<PathBuf> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = candidate_for(&dir, name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    for fallback in fallback_dirs() {
        let candidate = candidate_for(&fallback, name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn candidate_for(dir: &Path, name: &str) -> PathBuf {
    #[cfg(windows)]
    {
        dir.join(format!("{}.exe", name))
    }
    #[cfg(not(windows))]
    {
        dir.join(name)
    }
}

fn fallback_dirs() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        vec![
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/opt/local/bin"),
        ]
    }
    #[cfg(target_os = "linux")]
    {
        vec![
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/snap/bin"),
        ]
    }
    #[cfg(target_os = "windows")]
    {
        vec![
            PathBuf::from("C:\\Program Files\\ffmpeg\\bin"),
            PathBuf::from("C:\\ffmpeg\\bin"),
        ]
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

/// Run ffprobe on a single file and parse the result. Returns Err when
/// ffprobe fails to start, exits non-zero, or emits unparseable JSON.
/// A hung ffprobe is bounded by `PROBE_TIMEOUT` — the child process is
/// killed and the call returns `Err` so a corrupt file can't freeze the
/// whole scan.
pub fn probe(media: &Path, ffprobe: &Path) -> anyhow::Result<ClipMetadata> {
    let output = run_with_timeout(
        Command::new(ffprobe),
        &[
            "-v",
            "error",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
        ],
        media,
        PROBE_TIMEOUT,
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("ffprobe emitted non-UTF8 output: {}", e))?;

    parse_ffprobe_json(&stdout)
}

/// Run `cmd` with the given args + `media` argument, enforcing `timeout`.
/// On expiry the child is killed and the function returns an error.
///
/// The child is shared with a waiter thread via `Arc<Mutex<Option<Child>>>`
/// so the timeout killer can reach it after the waiter has taken it out of
/// the mutex (and vice versa). stdout/stderr are drained in their own
/// threads so a chatty child cannot deadlock on a full pipe buffer.
fn run_with_timeout(
    mut cmd: Command,
    args: &[&str],
    media: &Path,
    timeout: Duration,
) -> anyhow::Result<std::process::Output> {
    cmd.args(args).arg(media);
    let (child, out_rx, err_rx) = spawn_piped(cmd)?;
    let child_slot: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(Some(child)));
    let status = wait_with_timeout(child_slot, timeout)?;
    // Drain stdout/stderr. These threads are guaranteed to send exactly
    // once each and we don't time them out — once the process has exited
    // any pending bytes will arrive quickly.
    let stdout = out_rx
        .recv()
        .map_err(|_| anyhow::anyhow!("stdout drain thread failed"))?;
    let stderr = err_rx
        .recv()
        .map_err(|_| anyhow::anyhow!("stderr drain thread failed"))?;
    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
}

/// Child plus the receiver halves for its captured stdout/stderr. The
/// two receivers each yield exactly once, when the corresponding drain
/// thread observes the OS pipe close.
type PipedChild = (Child, mpsc::Receiver<Vec<u8>>, mpsc::Receiver<Vec<u8>>);

/// Spawn a `Command` with piped stdout/stderr, returning the child and
/// the two receiver halves for the drain threads. The drain threads run
/// for the lifetime of the spawned process; their receivers yield the
/// captured bytes once the OS pipe closes.
fn spawn_piped(mut cmd: Command) -> anyhow::Result<PipedChild> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Spawn: {}", e))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("stdout was not piped"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("stderr was not piped"))?;
    let (out_tx, out_rx) = mpsc::sync_channel::<Vec<u8>>(1);
    let (err_tx, err_rx) = mpsc::sync_channel::<Vec<u8>>(1);
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        let _ = out_tx.send(buf);
    });
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        let _ = err_tx.send(buf);
    });
    Ok((child, out_rx, err_rx))
}

/// Wait for a spawned child (held inside `child_slot`) with the given
/// `timeout`. On expiry the child is killed via its OS process id; on
/// success the final `ExitStatus` is returned. Split out from
/// `run_with_timeout` so tests can drive it without spinning up extra
/// drain threads.
///
/// Implementation note: the child is moved OUT of the mutex into the
/// waiter thread before `wait()` is called — otherwise the timeout
/// killer would deadlock against the waiter for the same mutex.
fn wait_with_timeout(
    child_slot: Arc<Mutex<Option<Child>>>,
    timeout: Duration,
) -> anyhow::Result<std::process::ExitStatus> {
    // Capture the pid first; the killer needs it (the child itself will
    // be moved into the waiter thread below).
    let pid = {
        let guard = child_slot.lock().unwrap_or_else(|p| p.into_inner());
        match guard.as_ref() {
            Some(c) => c.id(),
            None => anyhow::bail!("child already taken"),
        }
    };

    let (status_tx, status_rx) = mpsc::sync_channel::<std::io::Result<std::process::ExitStatus>>(1);
    {
        let slot = Arc::clone(&child_slot);
        thread::spawn(move || {
            let mut child = {
                let mut guard = slot.lock().unwrap_or_else(|p| p.into_inner());
                match guard.take() {
                    Some(c) => c,
                    None => return, // killed by the timeout path
                }
            };
            let status = child.wait();
            let _ = status_tx.send(status);
        });
    }

    match status_rx.recv_timeout(timeout) {
        Ok(s) => s.map_err(|e| anyhow::anyhow!("Wait: {}", e)),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Kill via OS handle — the waiter thread still owns the
            // Child, so we can't go through `child.kill()`.
            kill_process(pid);
            anyhow::bail!("process timed out after {:?}", timeout);
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            anyhow::bail!("waiter disconnected unexpectedly");
        }
    }
}

/// Send SIGKILL (Unix) or call TerminateProcess (Windows) for the given
/// process id. Best-effort: errors are silently ignored because the
/// timeout path has no useful recovery if the kill itself fails.
fn kill_process(pid: u32) {
    #[cfg(unix)]
    {
        // Safety: kill(2) is async-signal-safe; passing an invalid pid
        // returns ESRCH which we ignore.
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGKILL);
        }
    }
    #[cfg(windows)]
    {
        // Safety: OpenProcess / TerminateProcess / CloseHandle are
        // documented to be safe to call with a valid process id and
        // NULL handle returns are well-defined.
        extern "system" {
            fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut std::ffi::c_void;
            fn TerminateProcess(hProcess: *mut std::ffi::c_void, uExitCode: u32) -> i32;
            fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
        }
        const PROCESS_TERMINATE: u32 = 0x0001;
        unsafe {
            let h = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if !h.is_null() {
                TerminateProcess(h, 1);
                CloseHandle(h);
            }
        }
    }
}

/// Parse the JSON produced by `ffprobe -of json -show_format -show_streams`.
/// Exposed (pub) for testing — production callers should use `probe()`.
pub fn parse_ffprobe_json(json: &str) -> anyhow::Result<ClipMetadata> {
    let root: serde_json::Value =
        serde_json::from_str(json).map_err(|e| anyhow::anyhow!("ffprobe JSON: {}", e))?;

    let streams = root
        .get("streams")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();

    let video = streams
        .iter()
        .find(|s| s.get("codec_type").and_then(|v| v.as_str()) == Some("video"));
    let audio = streams
        .iter()
        .find(|s| s.get("codec_type").and_then(|v| v.as_str()) == Some("audio"));

    let video_codec = video
        .and_then(|s| s.get("codec_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let width = video
        .and_then(|s| s.get("width"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let height = video
        .and_then(|s| s.get("height"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let (fps_num, fps_den) = video
        .and_then(|s| s.get("r_frame_rate"))
        .and_then(|v| v.as_str())
        .and_then(parse_rational)
        .unwrap_or((0, 1));
    let color_space = video
        .and_then(|s| s.get("color_space"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let audio_codec = audio
        .and_then(|s| s.get("codec_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let audio_channels = audio
        .and_then(|s| s.get("channels"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let audio_sample_rate = audio
        .and_then(|s| s.get("sample_rate"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    // Duration: prefer the format-level duration, fall back to the
    // first stream's duration.
    let duration_seconds = root
        .get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| {
            video
                .and_then(|s| s.get("duration"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    // SMPTE timecode lives in stream tags or format tags.
    let timecode = streams
        .iter()
        .find_map(|s| {
            s.get("tags")
                .and_then(|t| t.get("timecode"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            root.get("format")
                .and_then(|f| f.get("tags"))
                .and_then(|t| t.get("timecode"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

    let _ = PROBE_TIMEOUT; // reserved for future use when we add a watchdog
    Ok(ClipMetadata {
        video_codec,
        width,
        height,
        fps_num,
        fps_den,
        duration_seconds,
        audio_codec,
        audio_channels,
        audio_sample_rate,
        timecode,
        color_space,
    })
}

fn parse_rational(s: &str) -> Option<(u32, u32)> {
    let (num, den) = s.split_once('/')?;
    Some((num.parse().ok()?, den.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PRORES_JSON: &str = r#"
{
  "streams": [
    {
      "codec_type": "video",
      "codec_name": "prores",
      "width": 1920,
      "height": 1080,
      "r_frame_rate": "24000/1001",
      "duration": "10.500000",
      "color_space": "bt709",
      "tags": { "timecode": "01:00:00:00" }
    },
    {
      "codec_type": "audio",
      "codec_name": "pcm_s16le",
      "channels": 2,
      "sample_rate": "48000"
    }
  ],
  "format": {
    "duration": "10.500000"
  }
}
"#;

    const SAMPLE_AUDIO_ONLY_JSON: &str = r#"
{
  "streams": [
    {
      "codec_type": "audio",
      "codec_name": "pcm_s24le",
      "channels": 6,
      "sample_rate": "48000"
    }
  ],
  "format": { "duration": "120.0" }
}
"#;

    #[test]
    fn parses_video_with_audio_and_timecode() {
        let m = parse_ffprobe_json(SAMPLE_PRORES_JSON).unwrap();
        assert_eq!(m.video_codec, "prores");
        assert_eq!(m.width, 1920);
        assert_eq!(m.height, 1080);
        assert_eq!(m.fps_num, 24000);
        assert_eq!(m.fps_den, 1001);
        assert_eq!(m.audio_codec, "pcm_s16le");
        assert_eq!(m.audio_channels, 2);
        assert_eq!(m.audio_sample_rate, 48000);
        assert_eq!(m.timecode.as_deref(), Some("01:00:00:00"));
        assert_eq!(m.color_space, "bt709");
        assert!((m.duration_seconds - 10.5).abs() < 1e-6);
    }

    #[test]
    fn fps_display_recognises_drop_frame() {
        let m = parse_ffprobe_json(SAMPLE_PRORES_JSON).unwrap();
        assert_eq!(m.fps_display(), "23.976");
    }

    #[test]
    fn fps_display_recognises_integer_rates() {
        let m = ClipMetadata {
            video_codec: "h264".into(),
            width: 1920,
            height: 1080,
            fps_num: 24,
            fps_den: 1,
            duration_seconds: 0.0,
            audio_codec: String::new(),
            audio_channels: 0,
            audio_sample_rate: 0,
            timecode: None,
            color_space: String::new(),
        };
        assert_eq!(m.fps_display(), "24");
    }

    #[test]
    fn parses_audio_only_file() {
        let m = parse_ffprobe_json(SAMPLE_AUDIO_ONLY_JSON).unwrap();
        assert_eq!(m.video_codec, "");
        assert_eq!(m.width, 0);
        assert_eq!(m.height, 0);
        assert_eq!(m.audio_codec, "pcm_s24le");
        assert_eq!(m.audio_channels, 6);
        assert!((m.duration_seconds - 120.0).abs() < 1e-6);
        assert!(m.timecode.is_none());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_ffprobe_json("not json").is_err());
    }

    #[test]
    fn handles_missing_streams_gracefully() {
        let m = parse_ffprobe_json(r#"{"format": {"duration": "5.0"}}"#).unwrap();
        assert_eq!(m.video_codec, "");
        assert_eq!(m.audio_codec, "");
        assert!((m.duration_seconds - 5.0).abs() < 1e-6);
    }

    #[test]
    fn parse_rational_basic() {
        assert_eq!(super::parse_rational("24000/1001"), Some((24000, 1001)));
        assert_eq!(super::parse_rational("24/1"), Some((24, 1)));
        assert_eq!(super::parse_rational("not-a-rational"), None);
        assert_eq!(super::parse_rational("/1"), None);
    }

    #[test]
    fn discover_does_not_panic_when_path_is_unset() {
        // We can't assert presence/absence portably, but we can assert
        // the discoverer doesn't crash regardless of environment.
        let _ = discover();
        let _ = discover_ffmpeg();
    }

    /// Drives `wait_with_timeout` (the testable inner half) with a
    /// guaranteed-hung process so we can assert that the timeout fires
    /// and the child is killed. We use `ping -n 30 127.0.0.1` on Windows
    /// (builtin-free) and `sleep 30` on Unix.
    #[test]
    fn wait_with_timeout_kills_hung_process() {
        let (child, _out_rx, _err_rx) = spawn_piped({
            #[cfg(windows)]
            {
                let mut c = Command::new("ping");
                c.args(["-n", "30", "127.0.0.1"]);
                c
            }
            #[cfg(not(windows))]
            {
                let mut c = Command::new("sleep");
                c.arg("30");
                c
            }
        })
        .expect("spawn_piped should succeed for ping/sleep");
        let slot: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(Some(child)));

        let start = std::time::Instant::now();
        let result = wait_with_timeout(slot, Duration::from_millis(250));
        let elapsed = start.elapsed();
        assert!(result.is_err(), "expected timeout, got Ok: {:?}", result);
        // Should be killed well before the 30s the child would otherwise
        // take. Allow generous slack for slow CI.
        assert!(
            elapsed < Duration::from_secs(5),
            "timeout took too long: {:?}",
            elapsed
        );
    }

    /// A fast-completing command should succeed and return its output.
    #[cfg(windows)]
    #[test]
    fn run_with_timeout_returns_output_for_fast_command() {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "echo hello"]);
        let out = run_with_timeout(cmd, &[], Path::new(""), Duration::from_secs(2))
            .expect("fast command should complete");
        assert!(out.status.success());
        let s = String::from_utf8_lossy(&out.stdout);
        assert!(s.contains("hello"));
    }
}

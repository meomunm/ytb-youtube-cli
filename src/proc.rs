//! Spawning external tools.

use std::path::Path;
use std::process::Command;

use crate::err::AppError;

/// Is `tool` an executable on `PATH`?
///
/// Used for lazy dependency checks: `ytb list` works without `mpv`, but
/// `ytb play` needs it. We only look it up right before we'd use it.
pub fn have(tool: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(tool)))
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(p) {
        Ok(m) => m.is_file() && (m.permissions().mode() & 0o111) != 0,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}

/// Run `bin args...` inheriting this process's stdio, returning its exit code.
///
/// Inheriting stdio is essential: it hands the TTY to `mpv` so keyboard
/// controls (space, `q`, `>`) work, and lets `yt-dlp` draw its progress bar.
pub fn run_inherit(bin: &str, args: &[String]) -> Result<i32, AppError> {
    let status = Command::new(bin)
        .args(args)
        .status()
        .map_err(|e| AppError::Io(format!("không chạy được '{bin}': {e}")))?;
    Ok(status.code().unwrap_or(1))
}

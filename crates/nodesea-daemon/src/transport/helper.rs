use std::path::PathBuf;

use super::TransportError;

const DEFAULT_BUFFER_SIZE: usize = 4096;
const MAX_BUFFER_SIZE: usize = 1 << 20; // 1 MiB
const BUFFER_GROWTH_STEP: usize = 1024; // 1 KiB

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(super) fn current_user_endpoint_path() -> Result<PathBuf, TransportError> {
    #[cfg(target_os = "macos")]
    {
        Ok(current_user_home()?.join("Library/Application Support/NodeSea/socket"))
    }

    #[cfg(target_os = "linux")]
    {
        Ok(current_user_runtime_dir()?.join("nodesea/socket"))
    }
}

#[cfg(target_os = "macos")]
fn current_user_home() -> Result<PathBuf, TransportError> {
    let uid = unsafe { libc::geteuid() };
    current_user_home_for_uid(uid)
}

#[cfg(target_os = "macos")]
fn current_user_home_for_uid(uid: libc::uid_t) -> Result<PathBuf, TransportError> {
    use std::{
        ffi::{CStr, OsStr},
        os::unix::ffi::OsStrExt,
    };

    // Resolve the home directory from the effective UID instead of trusting
    // HOME, which may be absent or caller-controlled.
    let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
    let mut buf_len = DEFAULT_BUFFER_SIZE;
    let mut buf = vec![0u8; buf_len];

    loop {
        let mut result = std::ptr::null_mut();

        // SAFETY: `pwd`, `buf`, and `result` are valid writable pointers for
        // the duration of the call. `buf` remains alive until `pw_dir` has
        // been converted into an owned `PathBuf` below.
        let rc = unsafe {
            libc::getpwuid_r(uid, &mut pwd, buf.as_mut_ptr().cast(), buf_len, &mut result)
        };

        match rc {
            0 if !result.is_null() => {
                let home = unsafe {
                    if pwd.pw_dir.is_null() {
                        return Err(TransportError::MissingHomeDirectory);
                    }
                    CStr::from_ptr(pwd.pw_dir)
                };
                return Ok(PathBuf::from(OsStr::from_bytes(home.to_bytes())));
            }
            0 => return Err(TransportError::MissingHomeDirectory),
            libc::ERANGE if buf_len < MAX_BUFFER_SIZE => {
                buf_len = (buf_len + BUFFER_GROWTH_STEP).min(MAX_BUFFER_SIZE);
                buf.resize(buf_len, 0);
            }
            libc::ERANGE => {
                tracing::error!(
                    "Failed to get passwd entry for uid {}: buffer size exceeded maximum of {} bytes",
                    uid,
                    MAX_BUFFER_SIZE
                );
                return Err(TransportError::MissingHomeDirectory);
            }
            error => {
                tracing::error!(
                    "Failed to get passwd entry for uid {}: {}",
                    uid,
                    std::io::Error::from_raw_os_error(error)
                );
                return Err(TransportError::MissingHomeDirectory);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn current_user_runtime_dir() -> Result<PathBuf, TransportError> {
    current_user_runtime_dir_from(std::env::var_os("XDG_RUNTIME_DIR"))
}

#[cfg(target_os = "linux")]
fn current_user_runtime_dir_from(
    runtime_dir: Option<std::ffi::OsString>,
) -> Result<PathBuf, TransportError> {
    runtime_dir
        .map(PathBuf::from)
        .ok_or(TransportError::MissingRuntimeDirectory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn current_user_home_returns_absolute_path() {
        let home = current_user_home().expect("current user should have a home directory");

        assert!(home.is_absolute());
        assert!(!home.as_os_str().is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn current_user_home_rejects_unknown_uid() {
        let result = current_user_home_for_uid(libc::uid_t::MAX);

        assert!(matches!(result, Err(TransportError::MissingHomeDirectory)));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn current_user_runtime_dir_rejects_missing_environment() {
        let result = current_user_runtime_dir_from(None);

        assert!(matches!(
            result,
            Err(TransportError::MissingRuntimeDirectory)
        ));
    }
}

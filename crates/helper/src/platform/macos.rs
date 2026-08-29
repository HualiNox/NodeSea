//! macOS path resolution.

use std::{
    ffi::{CStr, OsStr},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

use super::HelperError;

const DEFAULT_BUFFER_SIZE: usize = 4096;
const MAX_BUFFER_SIZE: usize = 1 << 20;
const BUFFER_GROWTH_STEP: usize = 1024;

/// Returns the home directory for the effective user.
pub fn current_user_home() -> Result<PathBuf, HelperError> {
    let uid = unsafe { libc::geteuid() };
    current_user_home_for_uid(uid)
}

/// Returns the default NodeSea Unix socket path for the current process.
pub fn default_socket_path() -> Result<PathBuf, HelperError> {
    // Keep this policy in the shared crate: daemon binding and CLI dialing
    // must never silently choose different socket locations.
    if unsafe { libc::geteuid() } == 0 {
        Ok(PathBuf::from("/var/run/nodesea/socket"))
    } else {
        Ok(current_user_home()?.join("Library/Application Support/NodeSea/socket"))
    }
}

fn current_user_home_for_uid(uid: libc::uid_t) -> Result<PathBuf, HelperError> {
    let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
    let mut buf_len = DEFAULT_BUFFER_SIZE;
    let mut buf = vec![0u8; buf_len];

    loop {
        let mut result = std::ptr::null_mut();

        // SAFETY: all pointers are valid for the duration of the call and the
        // backing buffer remains alive while pw_dir is converted below.
        let rc = unsafe {
            libc::getpwuid_r(uid, &mut pwd, buf.as_mut_ptr().cast(), buf_len, &mut result)
        };

        match rc {
            0 if !result.is_null() => {
                let home = unsafe {
                    if pwd.pw_dir.is_null() {
                        return Err(HelperError::MissingHomeDirectory);
                    }
                    CStr::from_ptr(pwd.pw_dir)
                };
                return Ok(PathBuf::from(OsStr::from_bytes(home.to_bytes())));
            }
            0 => return Err(HelperError::MissingHomeDirectory),
            libc::ERANGE if buf_len < MAX_BUFFER_SIZE => {
                buf_len = (buf_len + BUFFER_GROWTH_STEP).min(MAX_BUFFER_SIZE);
                buf.resize(buf_len, 0);
            }
            libc::ERANGE => {
                tracing::error!(
                    uid,
                    max_buffer_size = MAX_BUFFER_SIZE,
                    "passwd entry buffer exceeded maximum size"
                );
                return Err(HelperError::MissingHomeDirectory);
            }
            error => {
                tracing::error!(uid, error, "failed to resolve passwd entry");
                return Err(HelperError::MissingHomeDirectory);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_user_home_returns_absolute_path() {
        let home = current_user_home().expect("current user should have a home directory");
        assert!(home.is_absolute());
        assert!(!home.as_os_str().is_empty());
    }

    #[test]
    fn current_user_home_rejects_unknown_uid() {
        let result = current_user_home_for_uid(libc::uid_t::MAX);
        assert!(matches!(result, Err(HelperError::MissingHomeDirectory)));
    }
}

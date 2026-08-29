//! Linux path resolution.

use std::{ffi::OsString, path::PathBuf};

use super::HelperError;

/// Returns the default NodeSea Unix socket path for the current process.
pub fn default_socket_path() -> Result<PathBuf, HelperError> {
    if unsafe { libc::geteuid() } == 0 {
        Ok(PathBuf::from("/run/nodesea/socket"))
    } else {
        Ok(current_user_runtime_dir()?.join("nodesea/socket"))
    }
}

fn current_user_runtime_dir() -> Result<PathBuf, HelperError> {
    current_user_runtime_dir_from(std::env::var_os("XDG_RUNTIME_DIR"))
}

fn current_user_runtime_dir_from(value: Option<OsString>) -> Result<PathBuf, HelperError> {
    value
        .map(PathBuf::from)
        .ok_or(HelperError::MissingRuntimeDirectory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_socket_path_uses_linux_runtime_directory() {
        if unsafe { libc::geteuid() } == 0 {
            assert_eq!(
                default_socket_path().unwrap(),
                PathBuf::from("/run/nodesea/socket")
            );
        } else {
            let path = default_socket_path().expect("runtime directory should be configured");
            assert!(path.is_absolute());
            assert!(path.ends_with("nodesea/socket"));
        }
    }

    #[test]
    fn runtime_directory_rejects_missing_value() {
        let result = current_user_runtime_dir_from(None);
        assert!(matches!(result, Err(HelperError::MissingRuntimeDirectory)));
    }

    #[test]
    fn runtime_directory_accepts_configured_value() {
        let result = current_user_runtime_dir_from(Some("/tmp/runtime".into()))
            .expect("configured runtime directory should be returned");
        assert_eq!(result, PathBuf::from("/tmp/runtime"));
    }
}

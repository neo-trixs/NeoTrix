#![allow(dead_code)]
//! macOS Seatbelt sandbox integration.
//!
//! Uses `sandbox_init` from libSystem via FFI to enable Apple's
//! mandatory access control (MAC) sandbox on the current process.
//! Once enabled, restrictions are irreversible for the process lifetime.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

// FFI declarations for Apple's sandbox framework.
// Linked automatically via libSystem (always available on macOS).
extern "C" {
    fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> c_int;
    fn sandbox_free_error(errorbuf: *mut c_char);
}

const SANDBOX_NAMED: u64 = 0x0001;

/// Check whether the Seatbelt sandbox API is available.
///
/// On macOS, `sandbox_init` is always available from libSystem.
/// Returns `true` on macOS, `false` on other platforms (compile-time).
#[cfg(target_os = "macos")]
pub fn seatbelt_available() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn seatbelt_available() -> bool {
    false
}

/// Enable the Apple Seatbelt sandbox with the given SBPL profile string.
///
/// The profile is passed as a named profile (inline SBPL).
/// On failure, returns an error string. On success, restrictions are
/// permanent for the process lifetime — no escape is possible.
///
/// # Arguments
/// * `sbpl_profile` — Sandbox Profile Language (SBPL) string defining
///   the sandbox rules. See Apple's `sandbox(7)` man page for syntax.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seatbelt_available_platform() {
        #[cfg(target_os = "macos")]
        assert!(seatbelt_available());
        #[cfg(not(target_os = "macos"))]
        assert!(!seatbelt_available());
    }

    #[test]
    fn test_enable_seatbelt_empty_profile_fails_non_macos() {
        let result = enable_seatbelt("");
        #[cfg(not(target_os = "macos"))]
        assert!(result.is_err());
        #[cfg(target_os = "macos")]
        {
            // On macOS, an empty SBPL profile is syntactically valid
            // but will fail at sandbox_init with "invalid profile"
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_enable_seatbelt_nul_byte_rejected() {
        let result = enable_seatbelt("bad\0profile");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("NUL"));
    }
}

pub fn enable_seatbelt(sbpl_profile: &str) -> Result<(), String> {
    let profile_c = CString::new(sbpl_profile)
        .map_err(|_| "SBPL profile contains interior NUL byte".to_string())?;

    let mut error_buf: *mut c_char = std::ptr::null_mut();

    let ret = unsafe {
        sandbox_init(
            profile_c.as_ptr(),
            SANDBOX_NAMED,
            &mut error_buf as *mut *mut c_char,
        )
    };

    if ret == 0 {
        Ok(())
    } else {
        let err_msg = if !error_buf.is_null() {
            let c_str = unsafe { std::ffi::CStr::from_ptr(error_buf) };
            let msg = c_str.to_string_lossy().into_owned();
            unsafe {
                sandbox_free_error(error_buf);
            }
            msg
        } else {
            format!("sandbox_init returned error code {}", ret)
        };
        Err(format!("Seatbelt init failed: {}", err_msg))
    }
}

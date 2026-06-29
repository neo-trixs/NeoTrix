#![allow(dead_code)]
//! Linux Landlock LSM integration.
//!
//! Landlock (since Linux 5.13) provides unprivileged filesystem sandboxing.
//! Uses raw syscalls (444-446) to create rulesets, add access rules, and
//! restrict the current process.

/// Supported Landlock access rule types.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Debug)]
pub enum LandlockRule {
    /// Allow reading a specific path and its contents.
    PathRead(String),
    /// Allow writing a specific path and its contents.
    PathWrite(String),
    /// Allow binding to a network address (Landlock ABI v3+).
    NetworkBind(String),
    /// Allow connecting to a network address (Landlock ABI v3+).
    NetworkConnect(String),
}

// Syscall numbers for Landlock (Linux ≥5.13)
const LANDLOCK_CREATE_RULESET: i64 = 444;
const LANDLOCK_ADD_RULE: i64 = 445;
const LANDLOCK_RESTRICT_SELF: i64 = 446;

// Access rights bit flags (fs access)
const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;
const LANDLOCK_ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
const LANDLOCK_ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
const LANDLOCK_ACCESS_FS_MAKE_CHAR: u64 = 1 << 6;
const LANDLOCK_ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
const LANDLOCK_ACCESS_FS_MAKE_REG: u64 = 1 << 8;
const LANDLOCK_ACCESS_FS_MAKE_SOCK: u64 = 1 << 9;
const LANDLOCK_ACCESS_FS_MAKE_FIFO: u64 = 1 << 10;
const LANDLOCK_ACCESS_FS_MAKE_BLOCK: u64 = 1 << 11;
const LANDLOCK_ACCESS_FS_MAKE_SYM: u64 = 1 << 12;

/// Landlock ruleset attribute (struct landlock_ruleset_attr).
#[repr(C)]
struct LandlockRulesetAttr {
    handled_access_fs: u64,
    handled_access_net: u64,
}

/// Landlock path-beneath attribute (struct landlock_path_beneath_attr).
#[repr(C)]
struct LandlockPathBeneathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

const LANDLOCK_RULE_PATH_BENEATH: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_landlock_available_non_linux() {
        #[cfg(not(target_os = "linux"))]
        assert!(!landlock_available());
        #[cfg(target_os = "linux")]
        {
            // On Linux, depends on kernel version — just verify no panic
            let _ = landlock_available();
        }
    }

    #[test]
    fn test_enable_landlock_empty_rules_errors_non_linux() {
        let result = enable_landlock(&[]);
        #[cfg(not(target_os = "linux"))]
        assert!(result.is_err());
    }

    #[test]
    fn test_landlock_rule_construction() {
        let r = LandlockRule::PathRead("/tmp".to_string());
        let w = LandlockRule::PathWrite("/var/tmp".to_string());
        let nb = LandlockRule::NetworkBind("0.0.0.0:8080".to_string());
        let nc = LandlockRule::NetworkConnect("1.2.3.4:443".to_string());
        // Just verify Debug formatting doesn't panic
        let _ = format!("{:?}", r);
        let _ = format!("{:?}", w);
        let _ = format!("{:?}", nb);
        let _ = format!("{:?}", nc);
    }

    #[test]
    fn test_enable_landlock_nonexistent_path_non_linux() {
        let rules = vec![LandlockRule::PathRead("/nonexistent_path_xyz".to_string())];
        let result = enable_landlock(&rules);
        #[cfg(not(target_os = "linux"))]
        assert!(result.is_err());
    }
}

/// Check whether Landlock is supported by the running kernel.
///
/// Attempts to create a minimal ruleset. If the syscall returns EOPNOTSUPP
/// or ENOSYS, Landlock is not available (kernel <5.13 or LSM disabled).
pub fn landlock_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        let attr = LandlockRulesetAttr {
            handled_access_fs: 0,
            handled_access_net: 0,
        };
        let fd = unsafe {
            syscall_3(
                LANDLOCK_CREATE_RULESET,
                &attr as *const _ as i64,
                std::mem::size_of::<LandlockRulesetAttr>() as i64,
                0,
            )
        };
        if fd < 0 {
            return false;
        }
        unsafe {
            libc_close(fd as i32);
        }
        true
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Enable Landlock restrictions for the current process.
///
/// # Arguments
/// * `rules` — List of path-based access rules. All allowed paths must be
///   explicitly listed; anything not listed is denied.
///
/// # Errors
/// Returns an error if Landlock is not available, a rule path doesn't exist,
/// or the syscall fails.
pub fn enable_landlock(rules: &[LandlockRule]) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        // Collect all access rights that we will ever grant
        let mut handled_access_fs: u64 = 0;
        let mut read_access = LANDLOCK_ACCESS_FS_READ_FILE | LANDLOCK_ACCESS_FS_READ_DIR;
        let mut write_access = LANDLOCK_ACCESS_FS_WRITE_FILE
            | LANDLOCK_ACCESS_FS_REMOVE_DIR
            | LANDLOCK_ACCESS_FS_REMOVE_FILE
            | LANDLOCK_ACCESS_FS_MAKE_CHAR
            | LANDLOCK_ACCESS_FS_MAKE_DIR
            | LANDLOCK_ACCESS_FS_MAKE_REG
            | LANDLOCK_ACCESS_FS_MAKE_SOCK
            | LANDLOCK_ACCESS_FS_MAKE_FIFO
            | LANDLOCK_ACCESS_FS_MAKE_BLOCK
            | LANDLOCK_ACCESS_FS_MAKE_SYM;

        for rule in rules {
            match rule {
                LandlockRule::PathRead(_) => {
                    handled_access_fs |= read_access;
                }
                LandlockRule::PathWrite(_) => {
                    handled_access_fs |= read_access | write_access;
                }
                LandlockRule::NetworkBind(_) | LandlockRule::NetworkConnect(_) => {
                    // net access requires ABI v3+; skip for now
                }
            }
        }

        // Create ruleset
        let attr = LandlockRulesetAttr {
            handled_access_fs,
            handled_access_net: 0,
        };
        let ruleset_fd = unsafe {
            syscall_3(
                LANDLOCK_CREATE_RULESET,
                &attr as *const _ as i64,
                std::mem::size_of::<LandlockRulesetAttr>() as i64,
                0,
            )
        };
        if ruleset_fd < 0 {
            return Err(format!(
                "landlock_create_ruleset failed: errno={}",
                ruleset_fd.abs()
            ));
        }

        // Add each rule
        for rule in rules {
            let (path, allowed_access) = match rule {
                LandlockRule::PathRead(p) => (p, read_access),
                LandlockRule::PathWrite(p) => (p, read_access | write_access),
                LandlockRule::NetworkBind(_) | LandlockRule::NetworkConnect(_) => continue,
            };
            let cpath =
                CString::new(path.as_str()).map_err(|_| format!("path contains NUL: {}", path))?;
            let parent_fd = unsafe { libc_open(cpath.as_ptr(), 0o100000) }; // O_PATH | O_CLOEXEC
            if parent_fd < 0 {
                unsafe {
                    libc_close(ruleset_fd as i32);
                }
                return Err(format!("cannot open path '{}' for Landlock rule", path));
            }
            let path_attr = LandlockPathBeneathAttr {
                allowed_access,
                parent_fd,
            };
            let ret = unsafe {
                syscall_4(
                    LANDLOCK_ADD_RULE,
                    ruleset_fd,
                    LANDLOCK_RULE_PATH_BENEATH as i64,
                    &path_attr as *const _ as i64,
                    0,
                )
            };
            unsafe {
                libc_close(parent_fd);
            }
            if ret < 0 {
                unsafe {
                    libc_close(ruleset_fd as i32);
                }
                return Err(format!(
                    "landlock_add_rule failed for '{}': errno={}",
                    path,
                    ret.abs()
                ));
            }
        }

        // Restrict self
        let ret = unsafe { syscall_2(LANDLOCK_RESTRICT_SELF, ruleset_fd, 0) };
        unsafe {
            libc_close(ruleset_fd as i32);
        }
        if ret < 0 {
            return Err(format!(
                "landlock_restrict_self failed: errno={}",
                ret.abs()
            ));
        }

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = rules;
        Err("Landlock is only available on Linux".to_string())
    }
}

// ── Raw syscall wrappers (no libc dependency) ──

#[cfg(target_os = "linux")]
unsafe fn syscall_3(n: i64, a1: i64, a2: i64, a3: i64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        lateout("rcx") _,
        lateout("r11") _,
        out("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[cfg(target_os = "linux")]
unsafe fn syscall_4(n: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        lateout("rcx") _,
        lateout("r11") _,
        out("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[cfg(target_os = "linux")]
unsafe fn syscall_2(n: i64, a1: i64, a2: i64) -> i64 {
    syscall_3(n, a1, a2, 0)
}

// Minimal C library stubs for open/close (already available via libc in std's linkage)
#[cfg(target_os = "linux")]
extern "C" {
    fn open(
        pathname: *const std::os::raw::c_char,
        flags: std::os::raw::c_int,
        ...
    ) -> std::os::raw::c_int;
    fn close(fd: std::os::raw::c_int) -> std::os::raw::c_int;
}

#[cfg(target_os = "linux")]
unsafe fn libc_open(
    path: *const std::os::raw::c_char,
    flags: std::os::raw::c_int,
) -> std::os::raw::c_int {
    open(path, flags)
}

#[cfg(target_os = "linux")]
unsafe fn libc_close(fd: std::os::raw::c_int) -> std::os::raw::c_int {
    close(fd)
}

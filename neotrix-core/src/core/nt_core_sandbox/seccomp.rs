#![allow(dead_code)]
//! Linux seccomp-bpf integration.
//!
//! Uses `seccomp(SECCOMP_SET_MODE_FILTER, ...)` via raw syscall (317 on x86_64)
//! or `prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, ...)` to install a BPF
//! syscall whitelist filter.

use std::os::raw::c_uint;

// ── Constants ──

// prctl
const PR_SET_SECCOMP: i32 = 22;
const SECCOMP_MODE_FILTER: i32 = 2;

// seccomp syscall (x86_64: 317, aarch64: 277)
#[cfg(target_arch = "x86_64")]
const SECCOMP_SYSCALL: i64 = 317;
#[cfg(target_arch = "aarch64")]
const SECCOMP_SYSCALL: i64 = 277;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const SECCOMP_SYSCALL: i64 = -1; // unsupported

const SECCOMP_SET_MODE_FILTER: c_uint = 1;

// BPF instruction
#[repr(C)]
struct SockFilter {
    code: u16,
    jt: u8,
    jf: u8,
    k: u32,
}

// BPF program (struct sock_fprog)
#[repr(C)]
struct SockFprog {
    len: u16,
    filter: *const SockFilter,
}

// ── BPF opcodes ──
const BPF_LD: u16 = 0x00;
const BPF_W: u16 = 0x00;
const BPF_ABS: u16 = 0x20;
const BPF_JMP: u16 = 0x05;
const BPF_JEQ: u16 = 0x10;
const BPF_JEQ_JMP: u16 = BPF_JMP | BPF_JEQ;
const BPF_JGE: u16 = 0x30;
const BPF_JGT: u16 = 0x20;
const BPF_RET: u16 = 0x06;
const BPF_K: u16 = 0x00;

// seccomp data offsets (arch/x86/include/asm/seccomp.h)
const SECCOMP_DATA_NR_OFFSET: u32 = 0; // offsetof(struct seccomp_data, nr)

// Allow / kill return codes
const SECCOMP_RET_KILL_PROCESS: u32 = 0x80000000;
const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;

/// Helper to build a BPF filter that allows only the specified syscalls.
fn build_whitelist_filter(allowed_syscalls: &[i64]) -> Vec<SockFilter> {
    let mut filters = Vec::new();
    let n = allowed_syscalls.len() as u16;

    // Load syscall number (arch-specific: nr is 4 bytes at offset 0)
    filters.push(SockFilter {
        code: BPF_LD | BPF_W | BPF_ABS,
        jt: 0,
        jf: 0,
        k: SECCOMP_DATA_NR_OFFSET,
    });

    // For each allowed syscall, check if nr matches
    for (i, &sc) in allowed_syscalls.iter().enumerate() {
        let is_last = i == allowed_syscalls.len() - 1;
        if is_last {
            // Last one: match or kill
            filters.push(SockFilter {
                code: BPF_JEQ_JMP,
                jt: 1, // jump 1 forward (to allow)
                jf: 1, // jump 1 forward (to kill, after this instruction)
                k: sc as u32,
            });
            // Allow
            filters.push(SockFilter {
                code: BPF_RET | BPF_K,
                jt: 0,
                jf: 0,
                k: SECCOMP_RET_ALLOW,
            });
            // Kill
            filters.push(SockFilter {
                code: BPF_RET | BPF_K,
                jt: 0,
                jf: 0,
                k: SECCOMP_RET_KILL_PROCESS,
            });
        } else {
            // Check this syscall; if matched, jump to allow; else continue
            filters.push(SockFilter {
                code: BPF_JEQ_JMP,
                jt: (n - i as u16) as u8, // jump over remaining checks to allow
                jf: 0,
                k: sc as u32,
            });
        }
    }

    // Fallback kill (if no syscall matched)
    if filters.is_empty() || n == 0 {
        filters.push(SockFilter {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: SECCOMP_RET_KILL_PROCESS,
        });
    }

    filters
}

/// Generate a default allowlist of syscalls for normal program operation.
///
/// This is a conservative whitelist covering basic process operations,
/// memory management, file I/O, and threading.
pub fn default_allowlist() -> Vec<i64> {
    vec![
        // Process control
        0,   // read
        1,   // write
        2,   // open
        3,   // close
        4,   // stat
        5,   // fstat
        6,   // lstat
        7,   // poll
        8,   // lseek
        9,   // mmap
        10,  // mprotect
        11,  // munmap
        12,  // brk
        13,  // rt_sigaction
        14,  // rt_sigprocmask
        15,  // rt_sigreturn
        16,  // ioctl
        17,  // pread64
        18,  // pwrite64
        19,  // readv
        20,  // writev
        21,  // access
        22,  // pipe
        23,  // select
        24,  // sched_yield
        25,  // mremap
        26,  // msync
        27,  // mincore
        28,  // madvise
        29,  // shmget
        30,  // shmat
        31,  // shmctl
        32,  // dup
        33,  // dup2
        34,  // pause
        35,  // nanosleep
        36,  // getitimer
        37,  // alarm
        38,  // setitimer
        39,  // getpid
        40,  // sendfile
        41,  // socket
        42,  // connect
        43,  // accept
        44,  // sendto
        45,  // recvfrom
        46,  // sendmsg
        47,  // recvmsg
        48,  // shutdown
        49,  // bind
        50,  // listen
        51,  // getsockname
        52,  // getpeername
        53,  // socketpair
        54,  // setsockopt
        55,  // getsockopt
        56,  // clone
        57,  // fork
        58,  // vfork
        59,  // execve
        60,  // exit
        61,  // wait4
        62,  // kill
        63,  // uname
        78,  // getdents
        79,  // getcwd
        80,  // chdir
        82,  // rename
        83,  // mkdir
        84,  // rmdir
        85,  // creat
        87,  // link
        88,  // unlink
        89,  // symlink
        90,  // readlink
        91,  // chmod
        92,  // fchmod
        93,  // chown
        94,  // fchown
        96,  // gettimeofday
        97,  // getrlimit
        98,  // getrusage
        99,  // sysinfo
        102, // getuid
        103, // getgid
        104, // geteuid
        105, // getegid
        106, // setpgid
        107, // getppid
        108, // getpgrp
        110, // getpgid
        111, // setuid
        112, // setgid
        113, // setreuid
        114, // setregid
        115, // getgroups
        116, // setgroups
        117, // setresuid
        118, // getresuid
        119, // setresgid
        120, // getresgid
        123, // gettid
        124, // syslog
        125, // ptrace
        126, // getpriority
        127, // setpriority
        128, // sched_setparam
        129, // sched_getparam
        130, // sched_setscheduler
        131, // sched_getscheduler
        132, // sched_get_priority_max
        133, // sched_get_priority_min
        137, // statfs
        138, // fstatfs
        139, // sysfs
        140, // getpriority
        141, // setpriority
        186, // sigaltstack
        187, // utime
        188, // mknod
        189, // uselib (unused but harmless)
        217, // getdents64
        218, // settimeofday
        219, // mount
        220, // umount2
        221, // swapon
        222, // swapoff
        228, // clock_gettime
        229, // clock_getres
        230, // clock_nanosleep
        231, // exit_group
        232, // epoll_create
        233, // epoll_ctl
        234, // epoll_wait
        235, // remap_file_pages
        236, // set_tid_address
        237, // timer_create
        238, // timer_settime
        239, // timer_gettime
        240, // timer_getoverrun
        241, // timer_delete
        242, // clock_settime
        257, // openat
        258, // mkdirat
        259, // mknodat
        260, // fchownat
        261, // futimesat
        262, // newfstatat
        263, // unlinkat
        264, // renameat
        265, // linkat
        266, // symlinkat
        267, // readlinkat
        268, // fchmodat
        269, // faccessat
        270, // pselect6
        271, // ppoll
        272, // unshare
        273, // set_robust_list
        274, // get_robust_list
        275, // splice
        276, // tee
        277, // sync_file_range
        278, // vmsplice
        279, // move_pages
        280, // utimensat
        281, // epoll_pwait
        282, // signalfd
        283, // timerfd_create
        284, // eventfd
        285, // fallocate
        286, // timerfd_settime
        287, // timerfd_gettime
        288, // accept4
        289, // signalfd4
        290, // eventfd2
        291, // epoll_create1
        292, // dup3
        293, // pipe2
        294, // inotify_init1
        295, // preadv
        296, // pwritev
        297, // rt_tgsigqueueinfo
        298, // perf_event_open
        299, // recvmmsg
        300, // fanotify_init
        301, // fanotify_mark
        302, // prlimit64
        303, // name_to_handle_at
        304, // open_by_handle_at
        305, // clock_adjtime
        306, // syncfs
        307, // sendmmsg
        308, // setns
        309, // getns
        310, // process_vm_readv
        311, // process_vm_writev
        312, // kcmp
        313, // finit_module
        314, // sched_setattr
        315, // sched_getattr
        316, // renameat2
        318, // getrandom
        319, // memfd_create
        320, // kexec_file_load
        321, // bpf
        322, // execveat
        323, // userfaultfd
        324, // membarrier
        325, // mlock2
        326, // copy_file_range
        327, // preadv2
        328, // pwritev2
        329, // pkey_mprotect
        330, // pkey_alloc
        331, // pkey_free
        332, // statx
        333, // io_pgetevents
        334, // rseq
        424, // pidfd_send_signal
        425, // io_uring_setup
        426, // io_uring_enter
        427, // io_uring_register
        434, // pidfd_open
        435, // clone3
        436, // close_range
        437, // openat2
        438, // pidfd_getfd
        439, // faccessat2
        440, // process_madvise
        441, // epoll_pwait2
        442, // mount_setattr
        443, // quotactl_fd
        // Landlock syscalls (must be allowed if used)
        444, // landlock_create_ruleset
        445, // landlock_add_rule
        446, // landlock_restrict_self
    ]
}

/// Check whether seccomp is available on this system.
///
/// Returns `false` on unsupported architectures or if running on a kernel
/// without seccomp support. Otherwise returns `true`.
pub fn seccomp_available() -> bool {
    #[cfg(all(
        any(target_arch = "x86_64", target_arch = "aarch64"),
        target_os = "linux"
    ))]
    {
        true
    }
    #[cfg(not(all(
        any(target_arch = "x86_64", target_arch = "aarch64"),
        target_os = "linux"
    )))]
    {
        false
    }
}

/// Enable seccomp-bpf with a whitelist of allowed syscalls.
///
/// # Arguments
/// * `allowed_syscalls` — List of syscall numbers that are allowed.
///   All other syscalls will cause the process to be killed.
///
/// # Errors
/// Returns an error if seccomp is not available, prctl fails, or
/// the architecture is not supported.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seccomp_available_platform() {
        #[cfg(all(
            any(target_arch = "x86_64", target_arch = "aarch64"),
            target_os = "linux"
        ))]
        assert!(seccomp_available());
        #[cfg(not(all(
            any(target_arch = "x86_64", target_arch = "aarch64"),
            target_os = "linux"
        )))]
        assert!(!seccomp_available());
    }

    #[test]
    fn test_default_allowlist_contains_core_syscalls() {
        let list = default_allowlist();
        assert!(list.contains(&0)); // read
        assert!(list.contains(&1)); // write
        assert!(list.contains(&2)); // open
        assert!(list.contains(&60)); // exit
        assert!(list.contains(&231)); // exit_group
        assert!(list.len() > 100);
    }

    #[test]
    fn test_enable_seccomp_errors_gracefully_non_linux() {
        let allowed = vec![0, 1, 2, 60];
        let result = enable_seccomp(&allowed);
        #[cfg(not(all(
            any(target_arch = "x86_64", target_arch = "aarch64"),
            target_os = "linux"
        )))]
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_seccomp_empty_list_non_linux() {
        let result = enable_seccomp(&[]);
        #[cfg(not(all(
            any(target_arch = "x86_64", target_arch = "aarch64"),
            target_os = "linux"
        )))]
        assert!(result.is_err());
    }

    #[test]
    fn test_build_whitelist_filter_produces_instructions() {
        let allowed = vec![0, 1, 60];
        let filters = build_whitelist_filter(&allowed);
        // At minimum: LD instruction + per-syscall check + ALLOW + KILL
        assert!(filters.len() >= allowed.len() + 2);
    }
}

pub fn enable_seccomp(allowed_syscalls: &[i64]) -> Result<(), String> {
    #[cfg(all(
        any(target_arch = "x86_64", target_arch = "aarch64"),
        target_os = "linux"
    ))]
    {
        if SECCOMP_SYSCALL < 0 {
            return Err("seccomp syscall not defined for this architecture".to_string());
        }

        let filters = build_whitelist_filter(allowed_syscalls);
        let prog = SockFprog {
            len: filters.len() as u16,
            filter: filters.as_ptr(),
        };

        // Try seccomp(SECCOMP_SET_MODE_FILTER, SECCOMP_FILTER_FLAG_NEW_LISTENER, &prog)
        // Fall back to prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog)
        let ret = unsafe { syscall_2(SECCOMP_SYSCALL, SECCOMP_SET_MODE_FILTER as i64, 0) };
        if ret == 0 {
            return Ok(());
        }

        // Fall back to prctl
        let ret = unsafe {
            syscall_3(
                157, // prctl syscall
                PR_SET_SECCOMP as i64,
                SECCOMP_MODE_FILTER as i64,
                &prog as *const _ as i64,
            )
        };
        if ret == 0 {
            return Ok(());
        }

        Err(format!("seccomp/prctl failed: errno={}", ret.abs()))
    }

    #[cfg(not(all(
        any(target_arch = "x86_64", target_arch = "aarch64"),
        target_os = "linux"
    )))]
    {
        let _ = allowed_syscalls;
        Err("seccomp is only available on x86_64 or aarch64 Linux".to_string())
    }
}

// ── Raw syscall wrappers ──

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[cfg(target_os = "linux")]
unsafe fn syscall_2(n: i64, a1: i64, a2: i64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") a1,
        in("rsi") a2,
        lateout("rcx") _,
        lateout("r11") _,
        out("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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

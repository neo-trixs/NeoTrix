#[test]
fn test_shell_direct_echo() {
    // `!` 前缀直跑：echo 成功 + exit 0
    let (code, stdout, stderr) = super::run_shell_direct("echo hello-neotrix").expect("shell runs");
    assert_eq!(code, 0);
    assert_eq!(stdout, "hello-neotrix");
    assert!(stderr.is_empty());
}

#[test]
fn test_shell_direct_nonzero_exit() {
    // 非零退出码应透传
    let (code, stdout, stderr) = super::run_shell_direct("exit 3").expect("shell runs");
    assert_eq!(code, 3);
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

#[test]
fn test_shell_direct_stderr_captured() {
    // stderr 应被捕获而非丢弃
    let (code, _stdout, stderr) = super::run_shell_direct("echo boom >&2").expect("shell runs");
    assert_eq!(code, 0);
    assert_eq!(stderr, "boom");
}

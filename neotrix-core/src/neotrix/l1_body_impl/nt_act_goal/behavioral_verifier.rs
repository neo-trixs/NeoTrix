//! BehavioralVerifier — 行为验证引擎
//!
//! P4-02: 对自动生成的代码修改执行编译 + 测试 + 属性验证。
//! 作为 SafeCodeApplier 的升级层: 不仅验证编译通过,
//! 还验证相关测试和不变属性。
//!
//! 文献对齐 (2026):
//!   - SelfEvolve (arXiv, Apr 2026): TDD 先行的双验证策略
//!   - ReVeal (ICLR 2026): 多轮自验证 + 工具评估
//!   - 核心差异: 验证结果作为 CapabilityVector 的 RL 奖励信号

use std::time::{Instant, Duration};
use std::process::{Command, Output};

/// 带超时的子进程执行：cargo 可能因构建锁/网络挂起，必须在限时内 kill，
/// 否则后台 handler (持全局锁) 会被永久卡住。reader 线程捕获完整输出。
fn run_bounded<const N: usize>(args: [&str; N], timeout_secs: u64) -> Option<Output> {
    let child = std::sync::Arc::new(std::sync::Mutex::new(
        Command::new("cargo")
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .ok()?,
    ));
    let (tx, rx) = std::sync::mpsc::channel::<Output>();
    let c2 = child.clone();
    std::thread::spawn(move || {
        // 轮询退出；结束后读取输出并发送
        loop {
            let exited = {
                let mut g = c2.lock().unwrap_or_else(|e| e.into_inner());
                g.try_wait().ok().flatten().is_some()
            };
            if exited {
                break;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        let out = {
            let mut g = c2.lock().unwrap_or_else(|e| e.into_inner());
            let status = g.wait();
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(s) = g.stdout.as_mut() {
                let _ = std::io::Read::read_to_end(s, &mut stdout);
            }
            if let Some(e) = g.stderr.as_mut() {
                let _ = std::io::Read::read_to_end(e, &mut stderr);
            }
            status.map(|st| Output {
                status: st,
                stdout,
                stderr,
            })
        };
        if let Ok(output) = out {
            let _ = tx.send(output);
        }
    });
    match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(output) => Some(output),
        Err(_) => {
            // 超时: kill 子进程 (reader 线程轮询到退出后自行结束)
            if let Ok(mut g) = child.lock() {
                let _ = g.kill();
                let _ = g.wait();
            }
            None
        }
    }
}

/// 验证级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationLevel {
    CompileOnly,
    CompileAndTest,
    Full,
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub passed: bool,
    pub compile_ok: bool,
    pub tests_ok: bool,
    pub properties_ok: bool,
    pub compile_errors: Vec<String>,
    pub test_failures: Vec<String>,
    pub duration_ms: u64,
}

/// 行为验证器
#[derive(Debug, Clone)]
pub struct BehavioralVerifier;

impl BehavioralVerifier {
    /// 验证代码修改 — 执行编译 + 测试 + 属性检查
    pub fn verify(
        file: &str,
        old_content: &str,
        new_content: &str,
        level: VerificationLevel,
    ) -> VerificationResult {
        let start = Instant::now();

        // 1. 编译验证
        let compile_ok = Self::check_compile();
        let compile_errors = if !compile_ok {
            Self::capture_compile_errors()
        } else {
            vec![]
        };

        if !compile_ok && level == VerificationLevel::CompileOnly {
            return VerificationResult {
                passed: false,
                compile_ok: false,
                tests_ok: false,
                properties_ok: false,
                compile_errors,
                test_failures: vec![],
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        // 2. 测试验证
        let (tests_ok, test_failures) = if level as u8 >= VerificationLevel::CompileAndTest as u8 {
            Self::check_tests()
        } else {
            (true, vec![])
        };

        // 3. 属性验证 (简单的 diff 合理性检查)
        let properties_ok = Self::check_properties(file, old_content, new_content);

        let passed = compile_ok && tests_ok && properties_ok;
        VerificationResult {
            passed,
            compile_ok,
            tests_ok,
            properties_ok,
            compile_errors,
            test_failures,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// 快速编译检查
    fn check_compile() -> bool {
        run_bounded(["check", "--lib", "-p", "neotrix"], 180)
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// 捕获编译错误
    fn capture_compile_errors() -> Vec<String> {
        let output = run_bounded(["check", "--lib", "-p", "neotrix"], 180);
        match output {
            Some(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                stderr.lines()
                    .filter(|l| l.contains("error["))
                    .take(5)
                    .map(|l| l.to_string())
                    .collect()
            }
            None => vec!["无法运行 cargo check".into()],
        }
    }

    /// 快速测试检查
    fn check_tests() -> (bool, Vec<String>) {
        let output = run_bounded(["test", "--lib", "-p", "neotrix", "--", "--test-threads=1"], 300);
        match output {
            Some(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let passed = stdout.contains("test result: ok");
                let failures: Vec<String> = stdout.lines()
                    .filter(|l| l.contains("FAILED"))
                    .take(5)
                    .map(|l| l.to_string())
                    .collect();
                (passed, failures)
            }
            None => (false, vec!["无法运行 cargo test".into()]),
        }
    }

    /// 属性验证 — 确保修改不会破坏基本不变性
    fn check_properties(file: &str, _old: &str, new: &str) -> bool {
        if new.is_empty() {
            return false; // 空内容 = 破坏性修改
        }
        if new.len() < 10 {
            return false; // 内容太短 = 异常
        }
        if file.ends_with(".rs") && !new.contains("fn ") && !new.contains("use ")
            && !new.contains("struct ") && !new.contains("impl ")
            && !new.contains("mod ") && !new.contains("//")
        {
            // Rust 文件至少应该有基本的语法元素
            // 注释也是有效的
            return false;
        }
        true
    }

    /// 验证结果摘要
    pub fn summarize(result: &VerificationResult) -> String {
        let status = if result.passed { "✅ 通过" } else { "❌ 失败" };
        format!(
            "{} 编译={} 测试={} 属性={} ({})",
            status,
            if result.compile_ok { "✅" } else { "❌" },
            if result.tests_ok { "✅" } else { "❌" },
            if result.properties_ok { "✅" } else { "❌" },
            if result.passed { "可提交" } else { "需修复" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_empty_new_content_fails_properties() {
        let result = BehavioralVerifier::verify("test.rs", "old", "", VerificationLevel::CompileOnly);
        assert!(!result.passed);
        assert!(!result.properties_ok);
    }

    #[test]
    fn test_verify_too_short_content_fails() {
        let result = BehavioralVerifier::verify("test.rs", "old", "hi", VerificationLevel::CompileOnly);
        assert!(!result.properties_ok);
    }

    #[test]
    fn test_check_properties_valid_rust() {
        assert!(BehavioralVerifier::check_properties("test.rs", "old", "fn main() { println!(\"hello\"); }"));
    }

    #[test]
    fn test_summarize_passed() {
        let r = VerificationResult {
            passed: true,
            compile_ok: true,
            tests_ok: true,
            properties_ok: true,
            compile_errors: vec![],
            test_failures: vec![],
            duration_ms: 1500,
        };
        let s = BehavioralVerifier::summarize(&r);
        assert!(s.contains("✅"));
        assert!(s.contains("通过"));
    }

    #[test]
    fn test_summarize_failed() {
        let r = VerificationResult {
            passed: false,
            compile_ok: false,
            tests_ok: false,
            properties_ok: false,
            compile_errors: vec!["error[E0425]".into()],
            test_failures: vec!["test_foo FAILED".into()],
            duration_ms: 3000,
        };
        let s = BehavioralVerifier::summarize(&r);
        assert!(s.contains("❌"));
    }

    #[test]
    fn test_check_properties_struct_rust() {
        assert!(BehavioralVerifier::check_properties("mod.rs", "old", "struct Foo { x: i32 }"));
    }

    #[test]
    fn test_check_properties_no_keywords_fails() {
        assert!(!BehavioralVerifier::check_properties("mod.rs", "old", "just text without rust"));
    }

    #[test]
    fn test_empty_rust_file_no_basic_elements() {
        let content = "just some random text without rust keywords";
        let result = BehavioralVerifier::verify("test.rs", "old", content, VerificationLevel::CompileOnly);
        assert!(!result.properties_ok);
    }
}

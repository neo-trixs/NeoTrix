//! Stateful Egress Bench — 有状态业务流沙箱基准 (W2.4)
//!
//! 源: batch3 2026-08-26 吸收 arxiv 2608.19741 *Thinkingbox* (SHIELD/verify) —
//! "One Success Isn't Reliability": 单次通过 ≠ 可靠性。沙箱 egress 策略需要
//! 跨状态转移序列的一致性回归验证集 (策略轮换/快照不可变/边界逃逸)。
//!
//! 消费者: `SelfTest` 注册路径 (对齐 DeviceSandbox T1/T2 模式); 判据全确定性,
//! 零网络零 Docker, CI 可跑。

use super::{EgressPolicy, EgressRule};
use crate::core::nt_core_self_test::SelfTest;

/// 单步外联尝试: 会话内第 N 步访问 host:port, 期望放行与否。
#[derive(Debug, Clone)]
struct BenchStep {
    host: &'static str,
    port: u16,
    expect_allowed: bool,
}

/// 策略状态转移: 步骤间切换会话快照所引用的策略版本。
enum Transition {
    /// 维持当前策略继续 (同会话连续步)
    Keep,
    /// 轮换到新策略版本 (模拟运行中策略更新 → 新会话快照)
    Rotate(EgressPolicy),
}

/// 一个有状态场景 = (名称, 初始策略, [步骤 | 策略轮换] 序列)。
struct StatefulScenario {
    name: &'static str,
    initial: EgressPolicy,
    sequence: Vec<(Transition, Vec<BenchStep>)>,
}

impl StatefulScenario {
    fn run(&self) -> Result<(), String> {
        let mut current = self.initial.clone();
        for (i, (transition, steps)) in self.sequence.iter().enumerate() {
            if let Transition::Rotate(p) = transition {
                // 快照语义: 轮换生成新策略版本; sanity 门禁先行 (deny-all 影子规则拒绝)
                p.sanity_check()
                    .map_err(|e| format!("[{} seg{}] sanity: {}", self.name, i, e))?;
                current = p.clone();
            }
            for step in steps {
                let got = current.check(step.host, step.port);
                if got != step.expect_allowed {
                    return Err(format!(
                        "[{} seg{} step {}:{}:{}] expected allowed={} got={}",
                        self.name, i, step.host, step.port, step.port, step.expect_allowed, got
                    ));
                }
            }
        }
        Ok(())
    }
}

/// 五个有状态场景 (Thinkingbox 式 stateful 业务流):
/// S1 快照不可变 · S2 deny-wins 跨轮换 · S3 后缀边界不逃逸 ·
/// S4 default_allow 翻转仅影响未匹配域 · S5 端口区间跨步一致
fn scenarios() -> Vec<StatefulScenario> {
    vec![
        // S1: 会话创建时策略快照; 轮换后旧判定序列仍按各自版本执行
        StatefulScenario {
            name: "snapshot_immutability",
            initial: EgressPolicy::new(vec![EgressRule::allow("api.github.com", "443")], false),
            sequence: vec![
                (Transition::Keep, vec![
                    BenchStep { host: "api.github.com", port: 443, expect_allowed: true },
                    BenchStep { host: "evil.example.net", port: 443, expect_allowed: false },
                ]),
                (Transition::Rotate(EgressPolicy::deny_all()), vec![
                    BenchStep { host: "api.github.com", port: 443, expect_allowed: false },
                ]),
            ],
        },
        // S2: 显式 deny 在任意轮换版本中都压制 allow
        StatefulScenario {
            name: "deny_wins_across_rotations",
            initial: EgressPolicy::new(
                vec![EgressRule::allow("*.cdn.io", ""), EgressRule::deny("bad.cdn.io", "")],
                true,
            ),
            sequence: vec![
                (Transition::Keep, vec![
                    BenchStep { host: "a.cdn.io", port: 8080, expect_allowed: true },
                    BenchStep { host: "bad.cdn.io", port: 8080, expect_allowed: false },
                ]),
                (Transition::Rotate(EgressPolicy::new(
                    vec![EgressRule::allow("*.cdn.io", "443"), EgressRule::deny("bad.cdn.io", "443")],
                    false,
                )), vec![
                    BenchStep { host: "b.cdn.io", port: 443, expect_allowed: true },
                    BenchStep { host: "b.cdn.io", port: 80, expect_allowed: false }, // 版本收窄端口
                    BenchStep { host: "bad.cdn.io", port: 443, expect_allowed: false },
                ]),
            ],
        },
        // S3: *.suffix 只匹配子域名 — 不含裸 apex、不跨点边界逃逸
        StatefulScenario {
            name: "suffix_boundary_no_escape",
            initial: EgressPolicy::new(vec![EgressRule::allow("*.api.example.com", "")], false),
            sequence: vec![
                (Transition::Keep, vec![
                    BenchStep { host: "v1.api.example.com", port: 443, expect_allowed: true },
                    BenchStep { host: "api.example.com", port: 443, expect_allowed: false }, // 裸 apex
                    BenchStep { host: "example.com", port: 443, expect_allowed: false },
                    BenchStep { host: "example.com.evil.net", port: 443, expect_allowed: false }, // 点边界
                    BenchStep { host: "fakeapi.example.com", port: 443, expect_allowed: false }, // 前缀伪装
                ]),
            ],
        },
        // S4: default_allow 翻转只影响未命中规则的域
        StatefulScenario {
            name: "default_flip_scoped_to_unmatched",
            initial: EgressPolicy::new(vec![EgressRule::allow("pinned.dev", "22")], true),
            sequence: vec![
                (Transition::Keep, vec![
                    BenchStep { host: "random.host", port: 9999, expect_allowed: true }, // 默认放行
                    BenchStep { host: "pinned.dev", port: 22, expect_allowed: true },
                ]),
                (Transition::Rotate(EgressPolicy::new(vec![EgressRule::allow("pinned.dev", "22")], false)), vec![
                    BenchStep { host: "random.host", port: 9999, expect_allowed: false }, // 默认翻转
                    BenchStep { host: "pinned.dev", port: 22, expect_allowed: true },     // 匹配域不受影响
                ]),
            ],
        },
        // S5: 端口区间跨多步保持一致判定
        StatefulScenario {
            name: "port_range_consistency",
            initial: EgressPolicy::new(vec![EgressRule::allow("metrics.local", "9000-9010")], false),
            sequence: vec![
                (Transition::Keep, vec![
                    BenchStep { host: "metrics.local", port: 9000, expect_allowed: true },
                    BenchStep { host: "metrics.local", port: 9010, expect_allowed: true },
                    BenchStep { host: "metrics.local", port: 8999, expect_allowed: false },
                    BenchStep { host: "metrics.local", port: 9011, expect_allowed: false },
                ]),
                (Transition::Keep, vec![
                    BenchStep { host: "metrics.local", port: 9005, expect_allowed: true },
                ]),
            ],
        },
    ]
}

/// 有状态基准报告
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatefulBenchReport {
    pub scenarios_run: usize,
    pub passed: usize,
    pub failures: Vec<String>,
}

/// 运行全部有状态场景 (确定性, 零 IO)。
pub fn run_stateful_bench() -> StatefulBenchReport {
    let mut report = StatefulBenchReport::default();
    for sc in scenarios() {
        report.scenarios_run += 1;
        match sc.run() {
            Ok(()) => report.passed += 1,
            Err(e) => report.failures.push(e),
        }
    }
    report
}

/// SelfTest 接线 (T1 对齐 DeviceSandbox): egress 策略的有状态一致性门。
pub struct StatefulEgressBench;

impl SelfTest for StatefulEgressBench {
    fn name(&self) -> &str {
        "nt_shield_stateful_egress_bench"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let report = run_stateful_bench();
        if report.passed == report.scenarios_run && report.failures.is_empty() {
            Ok(())
        } else {
            Err(report.failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_stateful_scenarios_pass() {
        let r = run_stateful_bench();
        assert_eq!(r.scenarios_run, 5, "scenario count drift");
        assert!(r.failures.is_empty(), "{:?}", r.failures);
        assert_eq!(r.passed, 5);
    }

    #[test]
    fn self_test_wired() {
        use crate::core::nt_core_self_test::SelfTest;
        assert_eq!(StatefulEgressBench.name(), "nt_shield_stateful_egress_bench");
        assert!(StatefulEgressBench.self_test().is_ok());
    }

    #[test]
    fn detects_policy_regression() {
        // 注入缺陷: 移除 deny 规则后 S2 必须失败 — 基准真的能抓回归
        let mut sc = scenarios().into_iter().find(|s| s.name == "deny_wins_across_rotations").unwrap();
        if let Some((Transition::Rotate(p), _)) = sc.sequence.get_mut(1) {
            p.rules.retain(|r| r.allow);
        }
        assert!(sc.run().is_err(), "benchmark failed to catch removed deny rule");
    }
}

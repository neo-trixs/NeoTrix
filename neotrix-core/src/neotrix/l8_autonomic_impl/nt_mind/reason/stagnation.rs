use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ════════════════════════════════════════════════════════════════════
// step-budget 吸收 (opencode 插件 → NT-MIND 停滞节点能力网)
// 来源: .opencode/plugins/step-budget.js 七层动态步骤能力网 (L1-L7)。
// 吸收策略: R-P42 强化既有 StagnationDetector 节点, 不新建平行模块。
// 融入:
//   L1 环增益 Aβ 收敛检测   → observe_stage() 前后半产出率对比
//   L2 循环检测 (cyclic)    → 阶段名序列三连重复检测
//   L4 死胡同回溯 (dead-end)→ 循环且零产出 → DeadEnd 洞察 (回溯换路)
//   L3 A* 目标距离估计      → remaining_estimate() (h(n)=base×trend)
// 定位: 护栏 + 引导, 非硬上限 — 洞察返回给 pipeline 打印, 不阻断执行。
// ════════════════════════════════════════════════════════════════════

/// 阶段级检测洞察 (step-budget L1/L2/L4 吸收 + ReflexGrad 双进程路由 + VRR-Stop 信念过滤)
#[derive(Debug, Clone, PartialEq)]
pub enum StageInsight {
    /// 正常推进
    None,
    /// 阶段序列循环重复 (A→B→C→A), 建议切换路径
    Cyclic(String),
    /// 死胡同: 循环且期间零产出 → 建议回溯到最近产出点换路
    DeadEnd(String),
    /// 环增益衰减: 前半有产出, 后半近停滞 → 建议收敛
    Stalling(String),
    /// 慢进程重规划 (ReflexGrad): m 次连续低产出触发因果重规划。
    /// 快进程 (局部精炼) 已收敛到死区, 需要慢进程换策略 — 非局部微调, 是路线级重规划。
    Escalate(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continue_on_absorb() {
        let mut d = StagnationDetector::new();
        let sig = d.observe(true, true, 0, 0.5, true, false);
        assert_eq!(sig, StagnationSignal::Continue);
    }

    #[test]
    fn test_stop_after_minor_errors() {
        let mut d = StagnationDetector {
            zero_reward_pause: 100,
            error_only_pause: 100,
            pause_duration_secs: 0,
            ..Default::default()
        };
        for _ in 0..25 {
            let sig = d.observe(false, false, 0, 0.5, false, true);
            if let StagnationSignal::Stop(_) = sig {
                return;
            }
        }
        panic!("should have stopped after 20 minor-error cycles");
    }

    #[test]
    fn test_pause_after_pure_errors() {
        let mut d = StagnationDetector {
            pause_duration_secs: 0,
            ..Default::default()
        };
        for i in 0..10 {
            let sig = d.observe(false, false, 2, 0.0, false, false);
            if i >= 8 {
                if matches!(sig, StagnationSignal::Pause(_, _)) {
                    return;
                }
            }
        }
        panic!("should have paused after 8+ pure-error cycles");
    }

    #[test]
    fn test_stop_after_no_absorb() {
        let mut d = StagnationDetector {
            stop_threshold: 5,
            pause_duration_secs: 0,
            ..Default::default()
        };
        for _ in 0..6 {
            let sig = d.observe(false, false, 0, 0.0, false, false);
            if let StagnationSignal::Stop(_) = sig {
                return;
            }
        }
        panic!("should have stopped after 5 no-absorb cycles");
    }

    #[test]
    fn test_reset_clears_counters() {
        let mut d = StagnationDetector::new();
        for _ in 0..6 {
            d.observe(false, false, 0, 0.0, false, true);
        }
        d.reset();
        let sig = d.observe(true, true, 0, 0.5, true, false);
        assert_eq!(sig, StagnationSignal::Continue);
    }

    #[test]
    fn test_pause_check_via_signal() {
        let mut d = StagnationDetector {
            pause_duration_secs: 0,
            ..Default::default()
        };
        assert!(!d.is_paused());
        for _ in 0..8 {
            d.observe(false, false, 2, 0.0, false, false);
        }
        let sig = d.observe(false, false, 2, 0.0, false, false);
        assert!(matches!(sig, StagnationSignal::Pause(_, _)), "expected Pause, got {:?}", sig);
    }

    /// 端到端集成测试: StagnationDetector → SelfIteratingBrain 全链路
    /// 离线运行, 不依赖网络
    #[test]
    fn test_stagnation_integration_with_brain() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut brain = super::super::SelfIteratingBrain::new();
        brain.stagnation = StagnationDetector {
            pause_duration_secs: 0,
            ..Default::default()
        };

        // 1. 首次 SEAL loop 应正常通过
        let r1 = brain.run_seal_loop("test integration", None, None);
        assert!(r1.is_ok(), "first SEAL should succeed");

        // 2. 用短任务跑几次 — 模拟无信息循环
        for i in 0..2 {
            let r = brain.run_seal_loop(&format!("task_{}", i), None, None);
            assert!(r.is_ok(), "stagnation gate should return Ok, not Err at iter {}", i);
        }

        // 3. 验证 iteration 正常增长
        assert!(brain.iteration >= 2, "brain should have iterated >=2 times, got {}", brain.iteration);
    }

    #[test]
    fn test_stagnation_with_real_absorb_cancels_stall() {
        let mut d = StagnationDetector {
            pause_duration_secs: 0,
            ..Default::default()
        };

        for _ in 0..10 {
            d.observe(false, false, 0, 0.0, false, true);
        }

        // absorb 事件应重置 stagnation
        d.observe(true, true, 0, 0.5, true, false);
        let stats = d.stats();
        assert_eq!(stats.consecutive_no_absorb, 0,
            "absorb should reset no-absorb counter");
        assert_eq!(stats.consecutive_zero_reward, 0,
            "absorb should reset zero-reward counter");
        assert_eq!(stats.consecutive_minor_errors, 0,
            "absorb should reset minor-errors counter");
    }

    /// 验证 evolve 级别的停滞场景: 所有维度=纯错误, 最终触发 Stop
    #[test]
    fn test_evolve_level_stagnation_full_stop() {
        let mut d = StagnationDetector {
            stop_threshold: 5,
            error_only_pause: 100,
            zero_reward_pause: 100,
            pause_duration_secs: 0,
            ..Default::default()
        };

        // 模拟 evolve 中 frontier empty 场景: 无吸收 + 无抓取 + 无新来源
        for i in 0..10 {
            let sig = d.observe(false, false, 0, 0.0, false, false);
            if let StagnationSignal::Stop(_) = sig {
                assert!(i >= 4, "should stop after {}+ cycles, stopped at {}", d.stop_threshold, i);
                return;
            }
        }
        panic!("should have stopped after {} no-absorb cycles", d.stop_threshold);
    }

    // ═══ step-budget 吸收测试 (L1/L2/L4/L3) ═══

    /// L2 循环检测: 三连阶段序列重复 → Cyclic
    #[test]
    fn test_observe_stage_cyclic_detection() {
        let mut d = StagnationDetector::new();
        // 构造 A→B→C 三轮 (9 元素, 满足窗口门槛), 全部产出, 非死胡同
        for name in ["A", "B", "C", "A", "B", "C", "A", "B", "C"] {
            let insight = d.observe_stage(name, true);
            if matches!(insight, StageInsight::Cyclic(_)) {
                return; // 检测到循环即通过
            }
        }
        panic!("expected Cyclic insight for repeated A>B>C sequence");
    }

    /// L4 死胡同: 循环且两次循环间零产出 → DeadEnd (要求回溯换路)
    #[test]
    fn test_observe_stage_dead_end() {
        let mut d = StagnationDetector::new();
        // A→B→C 三轮, 全程零产出 → DeadEnd
        for (i, name) in ["A", "B", "C", "A", "B", "C", "A", "B", "C"].iter().enumerate() {
            let insight = d.observe_stage(name, false); // 全程零产出
            if matches!(insight, StageInsight::DeadEnd(_)) {
                assert!(i >= 8, "dead-end should fire at the repeated cycle, fired at {}", i);
                return;
            }
        }
        panic!("expected DeadEnd insight for zero-output repeated cycle");
    }

    /// L1 环增益: 前半有产出后半停滞 → Stalling
    #[test]
    fn test_observe_stage_stalling() {
        let mut d = StagnationDetector::new();
        // 前 6 步产出 (不同名, 避免三连重复), 后 6 步零产出
        for i in 0..6 {
            d.observe_stage(&format!("produce_{}", i), true);
        }
        for i in 0..6 {
            let insight = d.observe_stage(&format!("idle_{}", i), false);
            if matches!(insight, StageInsight::Stalling(_)) {
                return;
            }
        }
        panic!("expected Stalling insight for front-loaded production");
    }

    /// L3 目标距离: 产出率高 → 剩余充裕; 零产出 → 剩余收紧
    #[test]
    fn test_remaining_estimate_trend() {
        let mut d = StagnationDetector::new();
        // 全产出 → trend ≥ 1, 剩余 ≥ base
        for _ in 0..9 {
            d.observe_stage("s", true);
        }
        let (rem_high, trend_high) = d.remaining_estimate(30.0);
        assert!(trend_high >= 1.0, "all-produce trend should be >=1, got {}", trend_high);
        assert!(rem_high >= 30, "all-produce remaining should be >= base, got {}", rem_high);

        // 零产出 → trend ≈ 0, 剩余 ≈ 0
        let mut d2 = StagnationDetector::new();
        for _ in 0..9 {
            d2.observe_stage("s", false);
        }
        let (rem_low, trend_low) = d2.remaining_estimate(30.0);
        assert!(trend_low < 0.1, "zero-produce trend should be ~0, got {}", trend_low);
        assert!(rem_low <= 1, "zero-produce remaining should be ~0, got {}", rem_low);
    }

    /// 正常推进不误报: 全产出无重复 → 始终 None
    #[test]
    fn test_observe_stage_clean_progress_no_false_positive() {
        let mut d = StagnationDetector::new();
        for name in ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"] {
            let insight = d.observe_stage(name, true);
            assert_eq!(insight, StageInsight::None, "clean progress should be None, got {:?}", insight);
        }
    }

    // ═══ ReflexGrad / VRR-Stop / TIDE 吸收测试 ═══

    /// ReflexGrad 慢进程: 连续 5 次无产出 → Escalate (即使窗口不足)
    #[test]
    fn test_reflexgrad_slow_process_escalate() {
        let mut d = StagnationDetector::new();
        for i in 0..5 {
            let insight = d.observe_stage(&format!("idle_{}", i), false);
            if matches!(insight, StageInsight::Escalate(_)) {
                assert!(i >= 4, "escalate should fire at m=5, fired at {}", i);
                return;
            }
        }
        panic!("expected Escalate after 5 consecutive low-score stages");
    }

    /// VRR-Stop 信念过滤: 全产出 → 信念趋近 1; 全无产出 → 信念趋近 0
    #[test]
    fn test_vrr_stop_belief_filtering() {
        let mut d = StagnationDetector::new();
        for _ in 0..10 {
            d.observe_stage("s", true);
        }
        assert!(d.validity() > 0.9, "all-produce validity should be high, got {}", d.validity());

        let mut d2 = StagnationDetector::new();
        for _ in 0..10 {
            d2.observe_stage("s", false);
        }
        assert!(d2.validity() < 0.1, "zero-produce validity should be low, got {}", d2.validity());
    }

    /// TIDE Loop Ratio: 循环/停滞阶段占比
    #[test]
    fn test_tide_loop_ratio() {
        let mut d = StagnationDetector::new();
        // 5 次无产出触发 Escalate (计入 loop_stages), 然后 5 次产出
        for i in 0..5 {
            d.observe_stage(&format!("idle_{}", i), false);
        }
        for i in 0..5 {
            d.observe_stage(&format!("prod_{}", i), true);
        }
        let lr = d.loop_ratio();
        assert!(lr > 0.0 && lr < 1.0, "loop ratio should be in (0,1), got {}", lr);
        assert!(lr <= 0.5, "5 loop stages / 10 total should be <= 0.5, got {}", lr);
    }

    /// agent-loop-guard 四级升级: 连续命中升级
    #[test]
    fn test_escalation_levels() {
        let mut d = StagnationDetector::new();
        // 触发 6 次 Escalate (每次连续 5 无产出)
        for _ in 0..6 {
            for i in 0..5 {
                d.observe_stage(&format!("idle_{}", i), false);
            }
        }
        let (streak, level) = d.escalation_level();
        assert!(streak >= 6, "escalation streak should be >= 6, got {}", streak);
        assert_eq!(level, "ESCALATE", "6+ hits should be ESCALATE, got {}", level);
    }

    /// VRR-Stop 有界停止 (缺陷③): ESCALATE + 持续无产出 (信念<0.3) → abort;
    /// 升级达 ESCALATE 但产出正常 (信念高) → 不 abort (防误杀)。
    #[test]
    fn test_vrr_stop_bounded_abort() {
        let mut d = StagnationDetector::new();
        // 触发 6 轮 Escalate (每轮 5 次无产出) → streak>=6, validity 持续衰减
        for _ in 0..6 {
            for i in 0..5 {
                d.observe_stage(&format!("idle_{}", i), false);
            }
        }
        let (streak, lvl) = d.escalation_level();
        assert!(streak >= 6, "streak={} 应达 ESCALATE", streak);
        assert_eq!(lvl, "ESCALATE");
        assert!(d.validity() < 0.3, "validity={:.3} 应低于 0.3", d.validity());
        assert!(d.should_abort(), "ESCALATE+低信念应触发有界停止");

        // 对照: ESCALATE 但产出正常 (信念高) → 不 abort
        let mut d2 = StagnationDetector::new();
        for _ in 0..6 {
            for i in 0..5 {
                d2.observe_stage(&format!("idle_{}", i), false);
            }
        }
        for i in 0..10 {
            d2.observe_stage(&format!("prod_{}", i), true);
        }
        assert!(!d2.should_abort(), "高信念不应 abort");
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StagnationSignal {
    Continue,
    Pause(u64, String),
    Stop(String),
}

pub struct StagnationDetector {
    pub pause_threshold: u64,
    pub stop_threshold: u64,
    pub zero_reward_pause: u64,
    pub error_only_pause: u64,
    pub pause_duration_secs: u64,

    consecutive_no_absorb: u64,
    consecutive_pure_error: u64,
    consecutive_zero_reward: u64,
    consecutive_no_new_sources: u64,
    consecutive_minor_errors: u64,
    total_cycles: u64,
    last_reward: f64,
    pause_until: Option<Instant>,

    // ── step-budget 吸收: 阶段级窗口 (L1 环增益 + L2 循环检测) ──
    stage_window: VecDeque<String>, // 最近阶段名 (滑动窗口, 循环检测用)
    stage_produced: VecDeque<bool>, // 每阶段是否有产出 (环增益用)
    progress_events: u64,           // 有产出的阶段总数 (L3 目标距离用)

    // ── ReflexGrad 双进程路由 + VRR-Stop 信念过滤 (外部调研吸收) ──
    low_score_streak: u64,          // 连续低产出阶段数 (慢进程触发条件 m)
    escalation_streak: u64,         // 连续升级命中数 (agent-loop-guard 四级升级)
    committed_validity: f64,        // VRR-Stop 信念过滤: 验证投票的信念估计 (0..1)
    total_stages: u64,              // 总阶段数 (TIDE Loop Ratio 分母)
    loop_stages: u64,               // 循环/停滞阶段数 (TIDE Loop Ratio 分子)
}

impl Default for StagnationDetector {
    fn default() -> Self {
        Self {
            pause_threshold: 5,
            stop_threshold: 20,
            zero_reward_pause: 10,
            error_only_pause: 8,
            pause_duration_secs: 10,
            consecutive_no_absorb: 0,
            consecutive_pure_error: 0,
            consecutive_zero_reward: 0,
            consecutive_no_new_sources: 0,
            consecutive_minor_errors: 0,
            total_cycles: 0,
            last_reward: 0.0,
            pause_until: None,
            stage_window: VecDeque::new(),
            stage_produced: VecDeque::new(),
            progress_events: 0,
            low_score_streak: 0,
            escalation_streak: 0,
            committed_validity: 0.0,
            total_stages: 0,
            loop_stages: 0,
        }
    }
}

impl StagnationDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_paused(&self) -> bool {
        self.pause_until.is_some_and(|t| Instant::now() < t)
    }

    pub fn observe(
        &mut self,
        absorbed: bool,
        fetched: bool,
        error_count: usize,
        reward: f64,
        new_sources: bool,
        minor_errors: bool,
    ) -> StagnationSignal {
        self.total_cycles += 1;

        if self.is_paused() {
            return StagnationSignal::Continue;
        }

        if absorbed || fetched && !minor_errors {
            self.consecutive_no_absorb = 0;
            self.consecutive_pure_error = 0;
            self.consecutive_minor_errors = 0;
        } else {
            self.consecutive_no_absorb += 1;
        }

        if error_count > 0 && !fetched {
            self.consecutive_pure_error += 1;
        } else {
            self.consecutive_pure_error = 0;
        }

        if reward.abs() < 1e-6 && !absorbed {
            self.consecutive_zero_reward += 1;
        } else {
            self.consecutive_zero_reward = 0;
        }

        if !new_sources {
            self.consecutive_no_new_sources += 1;
        } else {
            self.consecutive_no_new_sources = 0;
        }

        if minor_errors && !absorbed {
            self.consecutive_minor_errors += 1;
        } else {
            self.consecutive_minor_errors = 0;
        }

        self.last_reward = reward;

        if self.consecutive_minor_errors >= self.stop_threshold {
            return StagnationSignal::Stop(format!(
                "连续 {} 次纯 minor errors, 无吸收 → 停止",
                self.consecutive_minor_errors
            ));
        }

        if self.consecutive_pure_error >= self.error_only_pause {
            self.pause_until = Some(Instant::now() + Duration::from_secs(self.pause_duration_secs));
            return StagnationSignal::Pause(
                self.pause_duration_secs,
                format!(
                    "连续 {} 次纯错误循环, 暂停 {}s",
                    self.consecutive_pure_error, self.pause_duration_secs
                ),
            );
        }

        if self.consecutive_zero_reward >= self.zero_reward_pause {
            self.pause_until = Some(Instant::now() + Duration::from_secs(self.pause_duration_secs));
            return StagnationSignal::Pause(
                self.pause_duration_secs,
                format!(
                    "连续 {} 次零奖励循环, 暂停 {}s",
                    self.consecutive_zero_reward, self.pause_duration_secs
                ),
            );
        }

        if self.consecutive_no_absorb >= self.stop_threshold {
            return StagnationSignal::Stop(format!(
                "连续 {} 次无吸收循环, frontier 可能枯竭",
                self.consecutive_no_absorb
            ));
        }

        StagnationSignal::Continue
    }

    pub fn reset(&mut self) {
        self.consecutive_no_absorb = 0;
        self.consecutive_pure_error = 0;
        self.consecutive_zero_reward = 0;
        self.consecutive_no_new_sources = 0;
        self.consecutive_minor_errors = 0;
        self.pause_until = None;
    }

    /// step-budget 吸收 (L1 环增益 + L2 循环 + L4 死胡同) +
    /// ReflexGrad 双进程路由 + VRR-Stop 信念过滤 + TIDE Loop Ratio。
    ///
    /// 每个阶段执行完后调用一次, 记录阶段名与是否产出, 并返回检测洞察。
    /// 检测优先级 (对齐 ReflexGrad 确定性优先级合并 plan≻gradient≻base):
    ///   1. DeadEnd (L4 死胡同, 最高优先 — 要求回溯换路)
    ///   2. Escalate (ReflexGrad 慢进程: m 次连续低分 → 因果重规划)
    ///   3. Cyclic (L2 循环)
    ///   4. Stalling (L1 环增益衰减)
    ///
    /// 设计为"护栏+引导": 返回洞察供调用方打印, 不阻断执行。
    pub fn observe_stage(&mut self, stage_name: &str, produced: bool) -> StageInsight {
        const WINDOW: usize = 12; // 与 step-budget windowSize*1.2 对齐
        const SLOW_PROCESS_M: u64 = 5; // ReflexGrad: m 次连续低分触发慢进程
        self.stage_window.push_back(stage_name.to_string());
        self.stage_produced.push_back(produced);
        if self.stage_window.len() > WINDOW {
            self.stage_window.pop_front();
            self.stage_produced.pop_front();
        }
        if produced {
            self.progress_events += 1;
        }
        self.total_stages += 1;

        // ── VRR-Stop 信念过滤: 验证投票 → 信念估计 (指数平滑) ──
        // 产出视为一次"通过验证"投票, 无产出视为"失败"投票。
        // committed_validity 是当前阶段真实有效性的信念估计。
        let vote = if produced { 1.0 } else { 0.0 };
        self.committed_validity = self.committed_validity * 0.7 + vote * 0.3;

        // ── ReflexGrad 慢进程触发: 连续低分 (无产出) 计数 ──
        if produced {
            self.low_score_streak = 0;
        } else {
            self.low_score_streak += 1;
        }

        // 样本门槛对齐 step-budget 原版 (calls.length >= 9):
        // 循环/死胡同检测需要足够窗口, 否则 between 切片 [i+3..len-3] 会越界或空切片误判。
        if self.stage_window.len() < 9 {
            // 窗口不足时仍可触发慢进程 (ReflexGrad 不依赖窗口, 只依赖连续低分)
            if self.low_score_streak >= SLOW_PROCESS_M {
                self.loop_stages += 1;
                self.escalation_streak += 1;
                return StageInsight::Escalate(format!(
                    "慢进程重规划: 连续 {} 次无产出, 快进程(局部精炼)已收敛到死区。请因果诊断根因并换路线, 而非继续局部微调。",
                    self.low_score_streak
                ));
            }
            return StageInsight::None;
        }

        let names: Vec<&str> = self.stage_window.iter().map(|s| s.as_str()).collect();

        // L2 循环检测: 三连序列重复 ≥2 次
        let last3: Vec<&str> = names[names.len() - 3..].to_vec();
        let last_key = last3.join(">");
        let limit = names.len() - 3;
        for i in 0..limit {
            // 跳过与 last 重叠的起始位 (对齐 step-budget 原版重叠区保护):
            // i 太靠近末尾时 between 切片 [i+3..len-3] 会越界。
            if i > names.len() - 6 && i < names.len() - 3 {
                continue;
            }
            if names[i..i + 3].join(">") == last_key {
                // 已发现历史重复 → L4 死胡同判定: 两个实例之间零产出
                let produced_vals: Vec<bool> = self.stage_produced.iter().copied().collect();
                let between = &produced_vals[i + 3..produced_vals.len() - 3];
                let has_output = between.iter().any(|&p| p);
                self.loop_stages += 1;
                if !has_output {
                    self.escalation_streak += 1;
                    return StageInsight::DeadEnd(format!(
                        "死胡同: 阶段序列 \"{}\" 重复且两次循环间零产出。请回溯到最近产出点换路, 勿原地重试。",
                        last_key
                    ));
                }
                return StageInsight::Cyclic(format!(
                    "阶段循环: 序列 \"{}\" 重复出现 (A→B→C→A)。建议切换路径。",
                    last_key
                ));
            }
        }

        // L1 环增益 Aβ: 前半有产出, 后半近乎停滞
        let vals: Vec<bool> = self.stage_produced.iter().copied().collect();
        let half = vals.len() / 2;
        let e1 = vals[..half].iter().filter(|&&p| p).count();
        let e2 = vals[half..].iter().filter(|&&p| p).count();
        if e1 >= 3 && e2 <= (e1 / 5).max(1) {
            let gain = e2 as f64 / e1 as f64;
            self.loop_stages += 1;
            return StageInsight::Stalling(format!(
                "环增益 Aβ≈{:.2}: 后半段产出 {}/{} 步, 前段 {}/{}。建议收敛到已完成部分或明确换策略。",
                gain, e2, vals.len() - half, e1, half
            ));
        }

        // ReflexGrad 慢进程: 窗口足够时, 连续低分仍触发重规划 (优先级高于 None)
        if self.low_score_streak >= SLOW_PROCESS_M {
            self.loop_stages += 1;
            self.escalation_streak += 1;
            return StageInsight::Escalate(format!(
                "慢进程重规划: 连续 {} 次无产出, 快进程(局部精炼)已收敛到死区。请因果诊断根因并换路线, 而非继续局部微调。",
                self.low_score_streak
            ));
        }

        StageInsight::None
    }

    /// TIDE Loop Ratio (LR): 循环/停滞阶段占总阶段比例。
    /// LR 高 → 递归失败主导; LR 低 → 行为适应良好。最小化 LR 是 TTI 的必要非充分条件。
    pub fn loop_ratio(&self) -> f64 {
        if self.total_stages == 0 {
            return 0.0;
        }
        self.loop_stages as f64 / self.total_stages as f64
    }

    /// agent-loop-guard 四级升级: 连续命中数 → 动作等级。
    /// 0-1: CONTINUE, 2-3: WARN, 4-5: STOP, 6+: ESCALATE。
    pub fn escalation_level(&self) -> (u64, &'static str) {
        match self.escalation_streak {
            0..=1 => (self.escalation_streak, "CONTINUE"),
            2..=3 => (self.escalation_streak, "WARN"),
            4..=5 => (self.escalation_streak, "STOP"),
            _ => (self.escalation_streak, "ESCALATE"),
        }
    }

    /// VRR-Stop 信念过滤: 当前 committed validity 估计。
    /// 用于"是否值得继续修复"决策 — 信念低于阈值时应停止而非盲目继续。
    pub fn validity(&self) -> f64 {
        self.committed_validity
    }

    /// VRR-Stop 有界停止 (缺陷③补齐): 升级达 ESCALATE 且信念持续低于阈值 →
    /// 应强制终止 pipeline 而非仅打印提醒 (T3: 输出必须影响行为)。
    /// 两个条件同时满足才触发, 避免误杀"偶发升级但产出正常"的 pipeline:
    ///   - escalation_streak >= 6 (ESCALATE 级)
    ///   - committed_validity < ABORT_VALIDITY_FLOOR (持续无产出, 证据不信任)
    ///
    /// ABORT_VALIDITY_FLOOR = 0.3: 连续 ~4 次无产出投票后衰减到达 (0.7^4 ≈ 0.24)。
    pub fn should_abort(&self) -> bool {
        const ABORT_VALIDITY_FLOOR: f64 = 0.3;
        self.escalation_streak >= 6 && self.committed_validity < ABORT_VALIDITY_FLOOR
    }

    /// step-budget 吸收 (L3 A* 目标距离估计 h(n) = base × trend)).
    /// 不数"已走多少步", 而估计"离出口还有几步":
    /// trend = 最近 1/3 段产出率 / 前 2/3 段产出率。trend≥1 → 充裕可深挖;
    /// trend≈0.3 → 收紧优先收敛; trend≈0 → 立即收敛/回溯。
    pub fn remaining_estimate(&self, base_allowance: f64) -> (u64, f64) {
        let n = self.stage_produced.len();
        if n < 6 {
            return (base_allowance as u64, 1.0);
        }
        let split = n / 3;
        let recent: Vec<bool> = self.stage_produced.iter().skip(n - split).copied().collect();
        let early: Vec<bool> = self.stage_produced.iter().take(n - split).copied().collect();
        let r1 = recent.iter().filter(|&&p| p).count() as f64 / recent.len().max(1) as f64;
        let r0 = early.iter().filter(|&&p| p).count() as f64 / early.len().max(1) as f64;
        let trend = if r0 > 0.0 { r1 / r0 } else if r1 > 0.0 { 1.5 } else { 0.0 };
        let remaining = (base_allowance * trend.min(1.5)).round() as u64;
        (remaining, (trend * 100.0).round() / 100.0)
    }

    pub fn stats(&self) -> StagnationStats {
        StagnationStats {
            total_cycles: self.total_cycles,
            consecutive_no_absorb: self.consecutive_no_absorb,
            consecutive_pure_error: self.consecutive_pure_error,
            consecutive_zero_reward: self.consecutive_zero_reward,
            consecutive_no_new_sources: self.consecutive_no_new_sources,
            consecutive_minor_errors: self.consecutive_minor_errors,
            paused: self.is_paused(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StagnationStats {
    pub total_cycles: u64,
    pub consecutive_no_absorb: u64,
    pub consecutive_pure_error: u64,
    pub consecutive_zero_reward: u64,
    pub consecutive_no_new_sources: u64,
    pub consecutive_minor_errors: u64,
    pub paused: bool,
}

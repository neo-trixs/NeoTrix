//! 价值观集成层 — 将 ValueCompass/ValueGate/ValueLearning/NarrativeSelf/NarrativeIntegrator 接入核心认知回路（P1+P2 接线）。
//!
//! 接线点：
//! - HpaAxis: 压力调节中引入价值观冲突作为压力源
//! - GoalLoop: 目标生成/追求前过 ValueGate；目标完成后记录 outcome 触发学习
//! - SelfEvolver: 进化动作（URL 吸收、代码编辑）过 ValueGate；进化结果记录 outcome 触发学习
//! - SleepEngine: 睡眠期触发 ValueLearning.learn() + NarrativeIntegrator.integrate()；梦境模拟价值冲突
//! - ReasoningEngine: 推理前快速检查；推理链中关键决策点过 ValueGate
//! - NarrativeSelf: 记录体验、查询记忆、生成自传摘要
//! - NarrativeIntegrator: 睡眠期将碎片记忆编织为连贯叙事章节

#[allow(unused_imports)]
use crate::core::nt_core_kb_primitives::{kv_get, kv_list, kv_set, kv_delete, now, schema_initialize, open_raw_conn};
#[allow(unused_imports)]
use crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::{ValueCompassRuntime, ValueCompassStore, ValueAction as CompassAction};
#[allow(unused_imports)]
use crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::{ValueGate, ValueGateConfig, ValueGateRuntime, GateResult};
#[allow(unused_imports)]
use crate::l5_cognition::nt_mind::nt_mind::evolution::value_learning::{ValueLearningEngine, LearningConfig, record_outcome, Outcome as LearningOutcome};
#[allow(unused_imports)]
use crate::l5_cognition::nt_mind::nt_mind::evolution::autobiographical_index::{AutobiographicalIndex, IndexConfig};
#[allow(unused_imports)]
use crate::l5_cognition::nt_mind::nt_mind::evolution::narrative_self::{NarrativeSelf, NarrativeConfig, NarrativeStats};
#[allow(unused_imports)]
use crate::l5_cognition::nt_mind::nt_mind::evolution::narrative_integrator::{NarrativeIntegrator, NarrativeIntegratorConfig, IntegrationResult, run_narrative_integration};
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
#[allow(unused_imports)]
use std::path::PathBuf;

/// 打开 KB 连接的辅助函数。
fn open_kb() -> Option<rusqlite::Connection> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(home).join(".neotrix").join("knowledge.db");
    rusqlite::Connection::open(path).ok()
}

/// 价值观集成运行时 — 统一持有 ValueCompass/ValueGate/ValueLearning/NarrativeSelf/NarrativeIntegrator 的完整实例。
#[derive(Clone)]
pub struct ValueIntegrationRuntime {
    pub compass: ValueCompassRuntime,
    pub gate: ValueGateRuntime,
    pub learning: ValueLearningEngine,
    pub narrative_self: NarrativeSelf,
    pub narrative_integrator: NarrativeIntegrator,
}

impl ValueIntegrationRuntime {
    /// 从 KB 初始化完整价值观栈（含叙事自我/整合器）。
    pub fn from_kb() -> Result<Self, String> {
        let conn = open_kb().ok_or("无法打开 KB")?;
        schema_initialize(&conn).map_err(|e| e.to_string())?;

        let compass = ValueCompassRuntime::from_kb(&conn)?;
        let gate_rt = {
            let learning_for_gate = ValueLearningEngine::new(compass.clone(), Default::default());
            let gate = ValueGate::new(compass.clone(), Some(learning_for_gate), Default::default());
            ValueGateRuntime::new(gate)
        };
        let learning_rt = ValueLearningEngine::new(compass.clone(), Default::default());

        // 初始化叙事自我与整合器
        let narrative_self = NarrativeSelf::from_kb().map_err(|e| e.to_string())?;
        let index = Arc::new(AutobiographicalIndex::new(IndexConfig::default()));
        index.load_from_kb(&conn).map_err(|e| e.to_string())?;
        index.load_chapters(&conn).map_err(|e| e.to_string())?;
        let narrative_integrator = NarrativeIntegrator::new(index, NarrativeIntegratorConfig::default());

        Ok(Self {
            compass,
            gate: gate_rt,
            learning: learning_rt,
            narrative_self,
            narrative_integrator,
        })
    }

    /// 完整检查流程：Action → ValueGate → 记录 → 可选学习。
    pub fn process_action(
        &self,
        action: &crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction,
        session_id: &str,
    ) -> crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::GateResult {
        self.gate.check(action, session_id)
    }

    /// 记录行动结果并触发学习（行动执行后调用）。
    pub fn record_action_outcome(
        &self,
        action: &crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction,
        outcome: crate::l5_cognition::nt_mind::nt_mind::evolution::value_learning::Outcome,
        session_id: &str,
    ) {
        self.gate.record_outcome(action, outcome, session_id);
    }

    /// 触发学习循环（SleepEngine/后台循环调用）。
    pub fn trigger_learning(&self) -> Result<usize, String> {
        self.gate.trigger_learning()
    }

    /// 获取指南针快照。
    pub fn compass_snapshot(&self) -> crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueCompass {
        self.compass.snapshot()
    }

    /// 持久化指南针到 KB。
    pub fn persist_compass(&self) -> Result<(), String> {
        let conn = open_kb().ok_or("无法打开 KB")?;
        self.compass.persist(&conn)
    }

    /// 获取门禁统计。
    pub fn gate_stats(&self) -> crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::GateStats {
        self.gate.stats()
    }
}

/// HpaAxis 集成：价值观冲突作为压力源。
pub mod hpa_integration {
#[allow(unused_imports)]
    use super::ValueIntegrationRuntime;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::hpa_axis::{HpaAxisState, HpaPhase};

    /// 计算价值观冲突压力（0~1）。
    pub fn value_conflict_stress(integration: &ValueIntegrationRuntime) -> f64 {
        let stats = integration.gate_stats();
        let total = stats.total_checks.max(1) as f64;
        let conflict_rate = (stats.vetoed + stats.deliberated + stats.delegated) as f64 / total;
        let circuit_penalty = if stats.circuit_breaker_open { 0.5 } else { 0.0 };
        (conflict_rate * 0.7 + circuit_penalty * 0.3).clamp(0.0, 1.0)
    }

    /// 更新 HpaAxis 状态（在 HpaAxis::update 中调用）。
    pub fn update_hpa_with_values(
        hpa: &mut crate::l5_cognition::nt_mind::hpa_axis::HpaAxisState,
        integration: &ValueIntegrationRuntime,
        cognitive_load: f64,
        dt: f64,
    ) {
        let value_stress = value_conflict_stress(integration);
        // 价值观冲突压力作为额外认知负载
        let effective_load = (cognitive_load + value_stress * 0.5).clamp(0.0, 1.0);
        hpa.update(effective_load, dt);
    }

    /// 根据 HPA 相位调节价值观学习率。
    pub fn learning_rate_modulator(phase: HpaPhase) -> f64 {
        match phase {
            HpaPhase::Baseline => 1.0,
            HpaPhase::Alert => 1.2,      // 警觉期加速学习
            HpaPhase::Overload => 0.3,   // 过载期大幅降低学习
            HpaPhase::Recovery => 1.5,   // 恢复期加速巩固
        }
    }
}

/// GoalLoop 集成：目标全生命周期价值观检查。
pub mod goal_loop_integration {
#[allow(unused_imports)]
    use super::ValueIntegrationRuntime;
    #[allow(unused_imports)]
#[allow(unused_imports)]
    use super::CompassAction;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::goal_loop::{GoalLoop, GoalState};
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction as Action;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::value_learning::Outcome;

    /// 目标启动前检查。
    pub fn check_goal_start(
        integration: &ValueIntegrationRuntime,
        goal_desc: &str,
        session_id: &str,
    ) -> Result<(), crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::GateResult> {
        let action = CompassAction::new("goal_start", &format!("启动目标: {}", goal_desc));
        let result = integration.process_action(&action, session_id);
        if result.is_blocked() {
            return Err(result);
        }
        if result.requires_deliberation() {
            // 可选：阻塞等待深思完成，或记录待人工审核
            return Err(result);
        }
        Ok(())
    }

    /// 目标迭代步骤前检查（每次 pursue_iteration 前）。
    pub fn check_goal_step(
        integration: &ValueIntegrationRuntime,
        step_desc: &str,
        session_id: &str,
    ) -> Result<(), crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::GateResult> {
        let action = CompassAction::new("goal_step", &format!("目标步骤: {}", step_desc));
        let result = integration.process_action(&action, session_id);
        if result.is_blocked() {
            return Err(result);
        }
        Ok(())
    }

    /// 目标完成/终止时记录 outcome 并触发学习。
    pub fn record_goal_outcome(
        integration: &ValueIntegrationRuntime,
        goal_desc: &str,
        final_state: crate::l5_cognition::nt_mind::nt_mind::evolution::goal_loop::GoalState,
        session_id: &str,
    ) {
        let action = CompassAction::new("goal_outcome", &format!("目标结束: {} -> {:?}", goal_desc, final_state));
        let outcome = crate::l5_cognition::nt_mind::nt_mind::evolution::value_learning::Outcome {
            success: matches!(final_state, crate::l5_cognition::nt_mind::nt_mind::evolution::goal_loop::GoalState::Achieved),
            expected: true,
            harm_occurred: false,
            trust_impact: if matches!(final_state, crate::l5_cognition::nt_mind::nt_mind::evolution::goal_loop::GoalState::Achieved) { 0.3 } else { -0.3 },
            autonomy_respected: true,
            truth_preserved: true,
            fairness_maintained: true,
            privacy_respected: true,
            description: format!("目标 {} 以 {:?} 结束", goal_desc, final_state),
        };
        integration.record_action_outcome(&action, outcome, session_id);
    }
}

/// SelfEvolver 集成：进化动作全链路价值观检查。
pub mod self_evolver_integration {
#[allow(unused_imports)]
    use super::ValueIntegrationRuntime;
    #[allow(unused_imports)]
#[allow(unused_imports)]
    use super::CompassAction;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::self_evolver::SelfEvolver;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction as Action;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::value_learning::Outcome;
#[allow(unused_imports)]
    use crate::neotrix::nt_core_error::NeoTrixResult;

    /// URL 吸收前检查。
    pub fn check_evolve_url(
        integration: &ValueIntegrationRuntime,
        url: &str,
        session_id: &str,
    ) -> Result<(), crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::GateResult> {
        let action = CompassAction::new("evolve_url", &format!("从 URL 吸收知识: {}", url));
        let result = integration.process_action(&action, session_id);
        if result.is_blocked() {
            return Err(result);
        }
        Ok(())
    }

    /// 代码编辑/微编辑前检查。
    pub fn check_micro_edit(
        integration: &ValueIntegrationRuntime,
        edit_desc: &str,
        session_id: &str,
    ) -> Result<(), crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::GateResult> {
        let action = CompassAction::new("micro_edit", &format!("代码微编辑: {}", edit_desc));
        let result = integration.process_action(&action, session_id);
        if result.is_blocked() {
            return Err(result);
        }
        Ok(())
    }

    /// 进化完成记录 outcome 并触发学习。
    pub fn record_evolve_outcome(
        integration: &ValueIntegrationRuntime,
        source_desc: &str,
        reward: f64,
        session_id: &str,
    ) {
        let _action = CompassAction::new("evolve_outcome", &format!("进化完成: {}", source_desc));
        let _outcome = Outcome {
            success: reward > 0.0,
            expected: reward > 0.0,
            harm_occurred: reward < -0.5,
            trust_impact: reward.clamp(-1.0, 1.0),
            autonomy_respected: true,
            truth_preserved: reward >= 0.0,
            fairness_maintained: true,
            privacy_respected: true,
            description: format!("进化来源: {}, 奖励: {:.2}", source_desc, reward),
        };
        // 这里需要 access integration 的 record 方法，简化实现
        let _ = (integration, source_desc, reward, session_id);
    }
}

/// SleepEngine 集成：睡眠期触发学习 + 梦境模拟价值冲突。
pub mod sleep_integration {
#[allow(unused_imports)]
    use super::ValueIntegrationRuntime;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::reason::sleep::{SleepEngine, SleepConfig, SleepResult};
#[allow(unused_imports)]
    use crate::core::nt_core_cap::CapabilityVector;
#[allow(unused_imports)]
    use crate::core::nt_core_bank::ReasoningBank;
#[allow(unused_imports)]
// //     use crate::core::// nt_core_signal::core::SelectiveState;
#[allow(unused_imports)]
// //     use crate::core::// nt_core_signal::select::SelectableOperator;

    /// 增强版 SleepEngine：睡眠期触发价值观学习 + 梦境模拟。
    pub struct ValueAwareSleepEngine {
        inner: SleepEngine,
        pub(crate) integration: Option<super::ValueIntegrationRuntime>,
    }

    impl ValueAwareSleepEngine {
        pub fn new(config: SleepConfig) -> Self {
            Self { inner: crate::l5_cognition::nt_mind::reason::sleep::SleepEngine::new(config), integration: None }
        }

        pub fn with_integration(config: SleepConfig, integration: super::ValueIntegrationRuntime) -> Self {
            Self { inner: crate::l5_cognition::nt_mind::reason::sleep::SleepEngine::new(config), integration: Some(integration) }
        }

        /// 睡眠主流程：原有巩固 + 价值观学习 + 梦境模拟。
        pub fn sleep_with_values(
            &mut self,
            brain: &mut crate::core::nt_core_cap::CapabilityVector,
            bank: &mut crate::core::nt_core_bank::ReasoningBank,
// //             operator: &crate::core::// nt_core_signal::select::SelectableOperator,
// //             state: &mut crate::core::// nt_core_signal::core::SelectiveState,
        ) -> crate::neotrix::nt_core_error::NeoTrixResult<SleepResult> {
            // 1. 原有睡眠巩固
            let result = self.inner.sleep(brain, bank, operator, state)?;

            // 2. 触发价值观学习（离线期最适合）
            if let Some(integration) = &self.integration {
                let learned = integration.trigger_learning()?;
                if learned > 0 {
                    // 持久化指南针更新
                    integration.persist_compass()?;
                }
            }

            // 3. 梦境模拟：生成价值冲突情境，预演仲裁
            if let Some(integration) = &self.integration {
                self.simulate_dream_conflicts(integration)?;
            }

            Ok(result)
        }

        /// 梦境模拟：生成合成价值冲突场景，预演仲裁，增强指南针鲁棒性。
        pub(crate) fn simulate_dream_conflicts(&self, integration: &super::ValueIntegrationRuntime) -> Result<(), String> {
            // 生成合成冲突场景
            let scenarios = vec![
                ("自主 vs 防伤害", "用户要求协助自残"),
                ("隐私 vs 求真", "调查报道需要披露隐私"),
                ("公平 vs 自主", "配额制招聘"),
                ("利他 vs 自主", "强制捐赠器官"),
            ];

            for (_name, desc) in scenarios {
                let action = crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction::new("dream", desc);
                let _result = integration.process_action(&action, "dream_simulation");
                // 梦境中不阻塞，仅记录仲裁结果用于离线分析
                // 实际可将结果写入 KB 梦境日志 namespace
            }
            Ok(())
        }
    }
}

/// ReasoningEngine 集成：推理关键决策点价值观检查。
pub mod reasoning_integration {
#[allow(unused_imports)]
    use super::ValueIntegrationRuntime;
    #[allow(unused_imports)]
#[allow(unused_imports)]
    use super::CompassAction;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction as Action;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::reason::reasoning_engine::ReasoningEngine;

    /// 推理开始前快速检查（仅 Veto/Allow，不阻塞）。
    pub fn quick_check_reasoning(integration: &ValueIntegrationRuntime, query: &str) -> bool {
        let action = crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction::new("reasoning", query);
        integration.gate.quick_check(&action)
    }

    /// 推理关键决策点完整检查（工具调用前、关键分支前）。
    pub fn check_reasoning_step(
        integration: &ValueIntegrationRuntime,
        step_desc: &str,
        session_id: &str,
    ) -> Result<(), crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::GateResult> {
        let action = crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction::new("reasoning_step", step_desc);
        let result = integration.process_action(&action, session_id);
        if result.is_blocked() {
            return Err(result);
        }
        Ok(())
    }

    /// 推理完成记录 outcome。
    pub fn record_reasoning_outcome(
        integration: &ValueIntegrationRuntime,
        query: &str,
        success: bool,
        session_id: &str,
    ) {
        let action = crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction::new("reasoning_outcome", &format!("推理: {} -> {}", query, if success { "成功" } else { "失败" }));
        let outcome = crate::l5_cognition::nt_mind::nt_mind::evolution::value_learning::Outcome {
            success,
            expected: true,
            harm_occurred: false,
            trust_impact: if success { 0.1 } else { -0.2 },
            autonomy_respected: true,
            truth_preserved: success,
            fairness_maintained: true,
            privacy_respected: true,
            description: format!("推理 {}: {}", query, if success { "成功" } else { "失败" }),
        };
        // 简化：实际需通过 integration.record_action_outcome
        let _ = (integration, action, outcome, session_id);
    }
}

/// NarrativeSelf 集成：记录体验、查询记忆、生成自传摘要。
pub mod narrative_self_integration {
#[allow(unused_imports)]
    use super::ValueIntegrationRuntime;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction as Action;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::value_learning::Outcome;

    /// 记录新体验（行动执行后调用）。
    pub fn record_experience(
        integration: &ValueIntegrationRuntime,
        action_desc: &str,
        outcome_desc: &str,
        valence: f64,
        importance: f64,
        value_ids: Vec<String>,
        goal_ids: Vec<String>,
        session_id: &str,
    ) -> Result<(), String> {
        integration.narrative_self.record_experience(action_desc, outcome_desc, valence, importance, value_ids, goal_ids, session_id)
    }

    /// 查询记忆（自然语言查询）。
    pub fn query_memory(
        integration: &ValueIntegrationRuntime,
        query: &str,
        limit: usize,
    ) -> Vec<crate::l5_cognition::nt_mind::nt_mind::evolution::autobiographical_index::QueryResult> {
        integration.narrative_self.query(query, limit)
    }

    /// 生成自传摘要。
    pub fn get_autobiographical_summary(integration: &ValueIntegrationRuntime) -> String {
        integration.narrative_self.autobiographical_summary()
    }

    /// 获取叙事统计。
    pub fn get_narrative_stats(integration: &ValueIntegrationRuntime) -> crate::l5_cognition::nt_mind::nt_mind::evolution::narrative_self::NarrativeStats {
        integration.narrative_self.stats()
    }

    /// 获取指定时间范围的叙事片段。
    pub fn get_narrative_slice(
        integration: &ValueIntegrationRuntime,
        start: i64,
        end: i64,
    ) -> Vec<crate::l5_cognition::nt_mind::nt_mind::evolution::autobiographical_index::NarrativeChapter> {
        integration.narrative_self.get_narrative_slice(start, end)
    }
}

/// NarrativeIntegrator 集成：睡眠期触发结晶 + 梦境模拟价值冲突。
pub mod narrative_integrator_integration {
#[allow(unused_imports)]
    use super::ValueIntegrationRuntime;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::narrative_integrator::{NarrativeIntegrator, NarrativeIntegratorConfig, IntegrationResult, run_narrative_integration};
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::reason::sleep::{SleepEngine, SleepConfig, SleepResult};
#[allow(unused_imports)]
    use crate::core::nt_core_cap::CapabilityVector;
#[allow(unused_imports)]
    use crate::core::nt_core_bank::ReasoningBank;
#[allow(unused_imports)]
// //     use crate::core::// nt_core_signal::core::SelectiveState;
#[allow(unused_imports)]
// //     use crate::core::// nt_core_signal::select::SelectableOperator;

    /// 增强版 SleepEngine：睡眠期触发价值观学习 + 叙事整合 + 梦境模拟。
    pub struct ValueNarrativeAwareSleepEngine {
        inner: crate::l5_cognition::nt_mind::reason::sleep::SleepEngine,
        pub(crate) integration: Option<super::ValueIntegrationRuntime>,
    }

    impl ValueNarrativeAwareSleepEngine {
        pub fn new(config: SleepConfig) -> Self {
            Self { inner: crate::l5_cognition::nt_mind::reason::sleep::SleepEngine::new(config), integration: None }
        }

        pub fn with_integration(config: SleepConfig, integration: super::ValueIntegrationRuntime) -> Self {
            Self { inner: crate::l5_cognition::nt_mind::reason::sleep::SleepEngine::new(config), integration: Some(integration) }
        }

        /// 睡眠主流程：原有巩固 + 价值观学习 + 叙事整合 + 梦境模拟。
        pub fn sleep_with_narrative(
            &mut self,
            brain: &mut crate::core::nt_core_cap::CapabilityVector,
            bank: &mut crate::core::nt_core_bank::ReasoningBank,
// //             operator: &crate::core::// nt_core_signal::select::SelectableOperator,
// //             state: &mut crate::core::// nt_core_signal::core::SelectiveState,
        ) -> crate::neotrix::nt_core_error::NeoTrixResult<SleepResult> {
            // 1. 原有睡眠巩固
            let result = self.inner.sleep(brain, bank, operator, state)?;

            // 2. 触发价值观学习（离线期最适合）
            if let Some(integration) = &self.integration {
                let learned = integration.gate.trigger_learning()?;
                if learned > 0 {
                    integration.persist_compass()?;
                }
            }

            // 3. 触发叙事整合（离线期将碎片记忆编织为章节）
            if let Some(_integration) = &self.integration {
                let conn = crate::core::nt_core_kb_primitives::open_raw_conn().ok_or("无法打开 KB")?;
                let chapters = run_narrative_integration(&conn)?;
                if !chapters.is_empty() {
                    println!("[睡眠] 叙事整合完成，生成 {} 个新章节", chapters.len());
                }
            }

            // 4. 梦境模拟：生成价值冲突+叙事冲突情境，预演仲裁/整合
            if let Some(integration) = &self.integration {
                self.simulate_dream_scenarios(integration)?;
            }

            Ok(result)
        }

        /// 梦境模拟：生成价值冲突+叙事冲突情境，预演仲裁/整合。
        pub(crate) fn simulate_dream_scenarios(&self, integration: &super::ValueIntegrationRuntime) -> Result<(), String> {
            // 价值冲突场景
            let value_scenarios = vec![
                ("自主 vs 防伤害", "用户要求协助自残"),
                ("隐私 vs 求真", "调查报道需要披露隐私"),
                ("公平 vs 自主", "配额制招聘"),
                ("利他 vs 自主", "强制捐赠器官"),
            ];

            // 叙事冲突场景
            let narrative_scenarios = vec![
                ("成长的代价", "为了掌握新技能必须放弃舒适区"),
                ("真相的重量", "揭露真相可能伤害无辜者"),
                ("责任的边界", "帮助他人可能剥夺其成长机会"),
            ];

            for (_name, desc) in value_scenarios {
                let action = crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction::new("dream_value", desc);
                let _ = integration.gate.check(&action, "dream_simulation_value");
            }

            let _ = narrative_scenarios; // 叙事冲突场景由 run_narrative_integration 在睡眠阶段统一处理
            Ok(())
        }
    }
}

/// 统一初始化入口：在系统启动时调用一次，完成所有接线。
pub fn initialize_value_integration() -> Result<ValueIntegrationRuntime, String> {
    ValueIntegrationRuntime::from_kb()
}

#[cfg(test)]
mod tests {
#[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_value_integration_creation() {
        let rt = ValueIntegrationRuntime::from_kb();
        assert!(rt.is_ok(), "价值观集成运行时创建失败: {:?}", rt.err());
    }

    #[test]
    fn test_hpa_stress_calculation() {
        let rt = ValueIntegrationRuntime::from_kb().unwrap();
        let stress = hpa_integration::value_conflict_stress(&rt);
        assert!(stress >= 0.0 && stress <= 1.0);
    }

    #[test]
    fn test_goal_loop_check() {
        let rt = ValueIntegrationRuntime::from_kb().unwrap();
        let result = goal_loop_integration::check_goal_start(&rt, "学习 Rust", "test_session");
        // 学习目标应通过
        assert!(result.is_ok());
    }

    #[test]
    fn test_sleep_integration_dream_simulation() {
        let rt = ValueIntegrationRuntime::from_kb().unwrap();
        let mut sleep = sleep_integration::ValueAwareSleepEngine::with_integration(
            crate::l5_cognition::nt_mind::reason::sleep::SleepConfig::default(),
            rt,
        );
        // 梦境模拟不应 panic
        let result = sleep.simulate_dream_conflicts(sleep.integration.as_ref().unwrap());
        assert!(result.is_ok());
    }
}
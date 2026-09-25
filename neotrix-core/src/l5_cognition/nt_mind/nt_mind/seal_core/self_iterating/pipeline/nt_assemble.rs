//! nt_assemble — 从 `pipeline.rs` 拆分 (assemble — 管线组装: execute + seal_pipeline + kernel_iterate_pipeline (行为零变更).)
//! 原文逐行搬运, 仅补可见性/导入, 行为零变更。

use super::nt_types::*;
use super::nt_core_stages::*;
use super::nt_optimize_stages::*;
use super::nt_wrapper_stages::*;
use super::nt_quality_stages::*;
use super::nt_credit_stages::*;
use super::super::recipe::RecipeStage;
use super::super::SelfIteratingBrain;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_core_error::NeoTrixResult;
use super::super::stage_contracts::ContractAwareStage;
use super::super::stage_contracts::DeliveryPromise;
use super::super::stage_contracts::DeliveryPromiseContract;
use crate::l5_cognition::nt_mind::reason::stagnation::StageInsight;
use super::super::aging_monitor::AgingDiagnosisStage;
use super::super::benchmark_gate::BenchmarkGateStage;
use super::super::checkpoint::CheckpointStage;
use super::super::checkpoint::RewindStage;
use super::super::dp_sgd_stage::DpSgdStage;
use super::super::goal_contract::EvidenceCaptureStage;
use super::super::goal_contract::ExternalVerifierStage;
use super::super::goal_contract::FinalVerificationStage;
use super::super::goal_contract::GoalTerminatorStage;
use super::super::goal_contract::NarrowRecoveryStage;
use super::super::goal_contract::SemanticRecallStage;
use super::super::procedural_memory::ProceduralMemoryStage;
use super::super::skillopt::BoundedEditStage;
use super::super::skillopt::EpochSlowUpdateStage;
use super::super::skillopt::RejectedBufferFeedbackStage;
use super::super::skillopt::ValidationGateStage;
use super::super::anti_distillation_stage::AntiDistillationStage;
use super::super::rsi_operators::MetaRsiStage;
use super::super::hyperarchive::HyperAgentArchive;
use super::super::hyperarchive::SelectionConfig;
use super::super::hypercore::HyperMetaAgent;
use super::super::hyperdgm::DGMMetaAgent;
use super::super::hyperdgm::GenerativeReplay;
use super::super::hyperdgm::SelfReferentialCheck;
use super::super::hyperstage::DGMMetaEvolveStage;
use super::super::hyperstage::MetaEvolveStage;
use super::super::openspace_evolution::OpenSpaceEvolveStage;

/// step-budget 吸收: 把 StageInsight 转成可打印的引导文本 (护栏非上限)。
fn stage_insight_text(insight: &StageInsight) -> String {
    match insight {
        StageInsight::None => String::new(),
        StageInsight::Cyclic(m) => format!("阶段循环: {}", m),
        StageInsight::DeadEnd(m) => format!("死胡同: {}", m),
        StageInsight::Stalling(m) => format!("停滞: {}", m),
        StageInsight::Escalate(m) => format!("升级重规划: {}", m),
    }
}

impl BrainPipeline {
    /// Execute all stages with visible per-stage progress.
    ///
    /// Previously this loop ran 40+ stages with zero output — from the caller's
    /// perspective the SEAL loop looked frozen ("IO 空转无反馈"). Now each active
    /// stage prints `[SEAL] [NN%] i/N name` before processing and a completion
    /// line with elapsed time, so long IO/reasoning steps show liveness. The
    /// lightweight println (not indicatif) keeps output stable in non-TTY
    /// contexts (tests, pipes) while still satisfying the progress contract.
    pub fn execute(&self, brain: &mut SelfIteratingBrain) -> NeoTrixResult<()> {
        // ── Pre-compute active stages (frequency-gated) for an honest total ──
        let active: Vec<&Box<dyn BrainStage>> = self
            .stages
            .iter()
            .filter(|s| brain.iteration.is_multiple_of(s.frequency() as u64))
            .collect();
        let total = active.len();

        if total == 0 {
            println!(
                "[SEAL] pipeline: 0 active stages (iteration {}), skip",
                brain.iteration
            );
            return Ok(());
        }
        println!(
            "[SEAL] pipeline: {} stages (iteration {})",
            total, brain.iteration
        );

        for (idx, stage) in active.iter().enumerate() {
            let pct = (idx * 100) / total;
            let stage_name = stage.name().to_string();

            // Liveness signal BEFORE the (possibly slow) stage — the core fix.
            println!(
                "[SEAL] [{:>3}%] {}/{} ▸ {}",
                pct,
                idx + 1,
                total,
                stage_name
            );

            let started = std::time::Instant::now();
            let mut stage_produced = true; // 默认产出; skip/rollback 视为未产出
            match stage.process(brain)? {
                StageDecision::Continue => {}
                StageDecision::Skip(reason) => {
                    stage_produced = false;
                    println!("[SEAL]        └─ skip: {}", reason);
                }
                StageDecision::Promote(champ) => {
                    brain.champion = Some(champ);
                    println!("[SEAL]        └─ champion promoted");
                }
                StageDecision::Rollback(reason) => {
                    println!("[SEAL]        └─ rollback: {}", reason);
                    return Err(NeoTrixError::Brain(format!(
                        "Pipeline rollback: {}",
                        reason
                    )));
                }
            }
            let elapsed_ms = started.elapsed().as_millis();
            println!("[SEAL]        └─ done ({:.1}ms)", elapsed_ms as f64);

            // step-budget 吸收 (L1/L2/L4) + ReflexGrad/VRR-Stop/TIDE 调研吸收:
            // 阶段级停滞/循环/死胡同洞察 + 升级等级 + 信念 + loop ratio
            match brain.stagnation.observe_stage(&stage_name, stage_produced) {
                StageInsight::None => {}
                // pi-agent steer vs abort (缺陷②): Escalate 洞察 + 升级达 STOP 级 →
                // 返回 Steer 错误: 信号是"换路线重规划"而非"硬终止"。由 run_seal_loop
                // 既有 Err 处理决定 (reward<0 + 有外部奖励 → 回滚换路线; 否则保留进度)。
                // 与 should_abort (ESCALATE+低信念→终止) 构成完整二分:
                //   STOP 级 → steer (重定向继续) ; ESCALATE 级 → abort (有界停止)。
                StageInsight::Escalate(m) => {
                    println!(
                        "[SEAL]        ⚠ {}",
                        stage_insight_text(&StageInsight::Escalate(m.clone()))
                    );
                    let (streak, lvl) = brain.stagnation.escalation_level();
                    // 仅 STOP 级 steer — ESCALATE 级留给下方 should_abort (缺陷③) 硬终止,
                    // 避免 steer 抢先绕过 abort (ESACALATE+低信念 = 必须停, 不是重定向)。
                    if lvl == "STOP" {
                        return Err(NeoTrixError::Steer(format!(
                            "慢进程重规划建议 (escalation={}({})): {} — 请重定向任务目标/换路线后继续",
                            streak, lvl, m
                        )));
                    }
                }
                insight => println!("[SEAL]        ⚠ {}", stage_insight_text(&insight)),
            }
            // 每 6 阶段打印一次控制面状态 (loopx 五问: 证据变化 + 循环继续?)
            if idx % 6 == 5 {
                let (lvl, lvl_name) = brain.stagnation.escalation_level();
                println!(
                    "[SEAL]        📊 控制面: 升级={}({}) 信念={:.2} LR={:.2}",
                    lvl,
                    lvl_name,
                    brain.stagnation.validity(),
                    brain.stagnation.loop_ratio()
                );
            }

            // VRR-Stop 有界停止 (缺陷③): ESCALATE + 信念<0.3 → 强制终止 pipeline
            // (T3: 洞察不再只是打印, 真正影响行为 — 防止慢进程门控失效时无限空转)
            if brain.stagnation.should_abort() {
                let (lvl, lvl_name) = brain.stagnation.escalation_level();
                return Err(NeoTrixError::Brain(format!(
                    "VRR-Stop 有界停止: 升级={}({}) 信念={:.2} — 持续无产出证据, 终止 pipeline 而非盲目继续",
                    lvl, lvl_name, brain.stagnation.validity()
                )));
            }

            // 动态配额控制 (缺陷①, loopx quota-aware 吸收): remaining_estimate 显示
            // 产出趋势锐减 (trend≈0) 且已消耗过半 → 提前收敛剩余阶段, 不做无谓空转。
            // 与 should_abort 的区别: 这里不报错终止, 而是正常跳过剩余阶段 (保留已完成成果),
            // 仅在证据强烈表明继续无产出时触发。
            if idx > 0 && idx >= total / 2 {
                let (_, trend) = brain.stagnation.remaining_estimate(0.0);
                if trend < 0.2 {
                    println!(
                        "[SEAL]        📉 动态配额收敛 (缺陷①): trend={:.2} 产出锐减, 提前结束剩余 {} 阶段",
                        trend, total - idx - 1
                    );
                    break;
                }
            }

            brain._stage_results.push(StageResult::new(&stage_name));
            const MAX_STAGE_RESULTS: usize = 1000;
            if brain._stage_results.len() > MAX_STAGE_RESULTS {
                brain
                    ._stage_results
                    .drain(0..(brain._stage_results.len() - MAX_STAGE_RESULTS));
            }
        }
        Ok(())
    }
}

pub fn seal_pipeline() -> BrainPipeline {
    BrainPipeline {
        stages: vec![
            Box::new(CheckpointStage::new()),
            Box::new(RecipeStage::new(Box::new(RewardCalculationStage::new())).with_frequency(3)),
            Box::new(ConsciousnessRewardStage::new()),
            Box::new(_SftWrapperStage::new()),
            Box::new(ProcessWrapperStage::new()),
            Box::new(_SearchSkillWrapperStage::new()),
            Box::new(DpoWrapperStage::new()),
            Box::new(ConstitutionalWrapperStage::new()),
            Box::new(SafetyWrapperStage::new()),
            Box::new(_BoundarySeparationStage::new()),
            Box::new(BoundedEditStage::new()),
            Box::new(_ScaffoldAwareRLStage::new()),
            Box::new(ValidationGateStage::new()),
            Box::new(DpSgdStage::new()),
            Box::new(_GwtAbsorbStage::new()),
            Box::new(BenchmarkGateStage::new()),
            Box::new(HarnessAdaptStage::new()),
            Box::new(_KnowledgeQualityStage::new()),
            Box::new(_AutonomyPerStage::new()),
            Box::new(RewindStage::new()),
            Box::new(RejectedBufferFeedbackStage::new()),
            Box::new(SecretScanStage::new()),
            // GoalContractStage omitted — individual stages handle each phase:
            // EvidenceCaptureStage, NarrowRecoveryStage, FinalVerificationStage,
            // GoalTerminatorStage, ExternalVerifierStage, SemanticRecallStage
            Box::new(EvidenceCaptureStage::new()),
            Box::new(NarrowRecoveryStage::new()),
            Box::new(FinalVerificationStage::new()),
            Box::new(GoalTerminatorStage::new()),
            Box::new(ExternalVerifierStage::new()),
            Box::new(SemanticRecallStage::new()),
            Box::new(ProceduralMemoryStage::new()),
            Box::new(MetaEvolveStage::new(
                HyperMetaAgent::new(10, true),
                HyperAgentArchive::new(SelectionConfig::default()),
            )),
            Box::new(DGMMetaEvolveStage::new(
                DGMMetaAgent::new(512, 5, 0.1),
                HyperAgentArchive::new(SelectionConfig::default()),
                GenerativeReplay {
                    num_components: 64,
                    min_score: 0.3,
                    max_samples: 100,
                    enabled: true,
                },
                SelfReferentialCheck {
                    max_distortion_ratio: 0.5,
                    max_spectral_growth: 1.5,
                    min_self_consistency: 0.4,
                },
            )),
            Box::new(_HypothesisAccuracyStage::new()),
            Box::new(_PatternExtractionStage::new()),
            // P0-2 接线 (OpenMontage delivery_promise 吸收):
            // 蒸馏阶段承诺真实能力提升 — 若 reward 上升但 champion 未动 (表面提升),
            // DeliveryPromiseContract 报 Error 阻断静默降级。
            Box::new(
                ContractAwareStage::new(Box::new(_DistillationStage::new())).with_contract(
                    Box::new(DeliveryPromiseContract {
                        promise: DeliveryPromise::capability_led(0.02),
                    }),
                ),
            ),
            Box::new(_ConversationDistillStage::new()),
            Box::new(EpochSlowUpdateStage::new()),
            Box::new(AgingDiagnosisStage::new()),
            Box::new(SelfReviewStage::new()),
            Box::new(_CreditAssignmentStage::new()),
            // W3.6 (batch3): 步级信用分歧审计 — 紧随 credit_assignment 消费同一信号面
            Box::new(StepCreditAuditStage::new()),
            Box::new(_OracleGateStage::new()),
            Box::new(_ArchitectureOptimizerStage::new()),
            Box::new(_TrendAnalysisStage::new()),
            Box::new(_MetaGoalStage::new()),
            Box::new(MemoryConsolidationStage::new()),
            Box::new(CacheCleanupStage::new()),
            Box::new(ExternalKnowledgeAbsorbStage::new()),
            // 概念涌现: 从KB节点聚类中发现新概念 (freq 10)
            Box::new(ConceptEmergenceStage::new()),
            // 外置大脑消化闭环: SEAL 调度自主把冷 corpus 转化为 live 能力 (R-P79)
            Box::new(_ExternalBrainDigestStage::new()),
            Box::new(_ConvergenceCheckStage::new()),
            Box::new(SelfTestStage::new()),
            Box::new(
                OpenSpaceEvolveStage::new()
                    .with_fix_enabled(true)
                    .with_derived_enabled(true)
                    .with_captured_enabled(true),
            ),
            // L8 接线: 此前未注册但有真实功能的 stage (Dark Forest: 接线或删除)。
            // 这些 stage 定义了 frequency, 不会每 tick 执行, 接线安全。
            Box::new(_SSMUpdateStage::new()), // E8 策略学习 (mode 值更新 + ε 衰减)
            Box::new(_HyperCubeOptimizeStage::new()), // HyperCube 剪枝 (freq 10)
            Box::new(_MetaImprovementStage::new()), // 元改进 (freq 10)
            Box::new(MetaRsiStage::new()),           // MetaRSI 三算子 (freq 5)
            Box::new(_OpenSourceCompareStage::new()), // 开源对比 (freq 5)
            Box::new(_UQCalibrationStage::new()), // 熵危机校准 (freq 20)
            Box::new(SleepStage::new()),     // 记忆巩固 (freq 100)
            Box::new(ReasoningBankStorageStage::new()), // 记忆写入 (P0-2: 从 log-only stub 实现为真实存储, freq 2)
            Box::new(AntiDistillationStage::new()),     // 反蒸馏健康监控 + 自适应水印/分解 (freq 5)
        ],
    }
}

pub fn kernel_iterate_pipeline() -> BrainPipeline {
    BrainPipeline {
        stages: vec![
            Box::new(CheckpointStage::new()),
            Box::new(BoundedEditStage::new()),
            Box::new(ValidationGateStage::new()),
            Box::new(RejectedBufferFeedbackStage::new()),
        ],
    }
}

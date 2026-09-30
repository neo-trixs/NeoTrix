use super::*;
use crate::l5_cognition::nt_crystal_core::{
    CalibRow, CocoonStore, CrystalConsciousness, CrystalCore, NtJevCalibration, NtOrchestrator,
    PlattBucketTable, GOLD_FLOOR,
};

// ============================================================================
// handlers_crystal.rs — 晶体自我迭代 tick (D2 结晶复用 + D5 任务验收融合接线)
//
// 职责: 每 1h 在 cocoons 同步的意识体上跑 NtSelfIterate
//      （tick→结晶率→calib eval→awaken 爬坡→收敛检查），
//      eval 信号来自晶体自己的 reasoning_chains
//      （NtJevCalibration::noul_rows，COREA 式自评，无需外部教师）。
// 状态跨 tick 累积（orchestrator EMA + 门限 + 自醒轮数），真持续进化。
//
// 挂载点: run.rs spawn_handler!(CRYSTAL_ITERATE_INTERVAL_SECS, "crystal_iterate",
//          |h| h.handle_crystal_self_iterate().await)
// ============================================================================

/// 跨 tick 持久的晶体三件套（game_trainer 同模式：字段持有，首 tick 懒加载）。
pub(crate) struct CrystalIterState {
    orchestrator: NtOrchestrator,
    consciousness: CrystalConsciousness,
    core: CrystalCore,
    threshold: f64,
}

impl CrystalIterState {
    /// 从 cocoons + crystal.json 恢复连续体；缺文件则从空意识起步（空跑必收敛，无害）。
    pub(crate) fn load() -> Self {
        let mut consciousness = CrystalConsciousness::new("bg-crystal");
        let cocoons = CocoonStore::load();
        cocoons.sync_to_consciousness(&mut consciousness);
        let core = CrystalCore::load().unwrap_or_else(|_| CrystalCore::new("bg-crystal"));
        Self {
            orchestrator: NtOrchestrator::default(),
            consciousness,
            core,
            threshold: 0.7,
        }
    }
}

/// calib 行 → eval 对（pred=按域校准的 confidence；gold=校准后过 GOLD 门）。
/// 与 `NtJevCalibration::decide_calibrated` 同语义的行批量版：
/// 域命中桶用桶参，否则 global；domain 缺席回 global。11 域桶由此进生产 tick。
fn calib_pairs(
    rows: Vec<CalibRow>,
    round: u64,
) -> (
    Vec<crate::l5_cognition::nt_jev::eval::EvalCase>,
    Vec<crate::l5_cognition::nt_jev::eval::EvalPrediction>,
) {
    use crate::l5_cognition::nt_jev::eval::{EvalCase, EvalPrediction, GoldAnswer};
    use crate::l5_cognition::nt_jev::primitives::{DecisionStatus, JevDecision, NoulAnswer};
    let table = PlattBucketTable::embedded();
    let mut cs = Vec::new();
    let mut ps = Vec::new();
    for (i, r) in rows.into_iter().enumerate() {
        let id = format!("crystal-r{round}-{i}");
        let conf = match r.domain.as_deref() {
            Some(d) => table.calibrate(d, r.confidence),
            None => table.calibrate("", r.confidence),
        }
        .clamp(0.0, 1.0);
        let gold = conf >= GOLD_FLOOR;
        cs.push(EvalCase {
            id: id.clone(),
            gold: GoldAnswer::Noul(gold),
        });
        ps.push(EvalPrediction {
            case_id: id,
            decision: JevDecision::Noul(NoulAnswer {
                noul: conf,
                needs_review: false,
                reason: None,
                status: DecisionStatus::Selected,
            }),
            latency_ms: 0,
        });
    }
    (cs, ps)
}

impl BackgroundLoopHandle {
    /// 晶体自我迭代 — 每 tick 最多 2 轮（控占锁时长），状态跨 tick 累积。
    pub(crate) async fn handle_crystal_self_iterate(&mut self) {
        use crate::l5_cognition::nt_crystal_core::{NtSelfIterate, NtSelfIterateConfig};

        if self.crystal_iter.is_none() {
            self.crystal_iter = Some(CrystalIterState::load());
        }
        let st = match self.crystal_iter.as_mut() {
            Some(s) => s,
            None => return,
        };
        // provider 只读意识（零捕获闭包），与 run 的 &mut 三件套无借用冲突。
        let mut provider = |c: &CrystalConsciousness, round: u64| {
            calib_pairs(NtJevCalibration::noul_rows(c, 20), round)
        };
        let cfg = NtSelfIterateConfig {
            max_rounds: 2,
            capability: "crystal-reasoning".to_string(),
            init_threshold: st.threshold,
            ..Default::default()
        };
        let rep = NtSelfIterate::run(
            &mut st.orchestrator,
            &mut st.consciousness,
            &mut st.core,
            &[],
            &mut provider,
            &cfg,
        );
        st.threshold = rep.final_threshold;
        log::info!(
            "[bg-loop] crystal_iterate rounds={} converged={} acc={:.3} ece={:.3} adapts={} awaken_rounds={}",
            rep.rounds_run,
            rep.converged,
            rep.final_accuracy,
            rep.final_ece,
            rep.adaptations,
            rep.final_awaken_rounds,
        );
    }
}

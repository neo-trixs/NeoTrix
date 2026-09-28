use super::super::*;

impl BackgroundLoopHandle {
    /// L10 超越层 T3 接线: 意识核心快照 ↔ 能力网共振 → 建议落盘 + goal 入队。
    /// 依赖文件缺失时静默跳过 (能力网未初始化是合法状态, 不视为错误)。
    pub(crate) async fn run_transcendent_tick(&mut self) {
        use crate::l5_cognition::traits::EvolutionHarnessApi;

        let Some(ref kb) = self.kb else { return };
        // 能力网注册表 (RegistryExport 格式, 与 handle_capability_auto_evolve 一致)
        let path = std::path::PathBuf::from(".neotrix/capability_registry.json");
        let json = match std::fs::read_to_string(&path) {
            Ok(j) => j,
            Err(_) => return, // 能力网未初始化 → 静默跳过
        };
        let (infos, _problems): (Vec<_>, Vec<_>) = <crate::l5_cognition::l1_facade::EvolutionHarness as EvolutionHarnessApi>::harness_infos_from_registry_export(&json);
        if infos.is_empty() {
            return;
        }
        let snapshot_json = serde_json::to_value(crate::l5_cognition::consciousness_core::status())
            .unwrap_or(serde_json::json!({}));
        let mut harness = <crate::l5_cognition::l1_facade::EvolutionHarness as EvolutionHarnessApi>::new_harness();
        let report = harness.harness_run_cycle(&snapshot_json, &infos);
        let persisted = harness.harness_persist_suggestions(kb, &report);
        // 高共振建议 → goal_loop (超越层建议真实驱动行为, 而非仅日志)
        let actionable = <crate::l5_cognition::l1_facade::EvolutionHarness as EvolutionHarnessApi>::harness_actionable_suggestions(&report, 0.7);
        let goal_count = actionable.len();
        if goal_count > 0 {
            if let Ok(mut brain) = self.brain.try_write() {
                for s in actionable.iter().take(3) {
                    self.goal_loop.enqueue_goal(
                        &mut brain,
                        &format!(
                            "[transcendent] strengthen {} (resonance={:.2}) — {}",
                            s.node_id, s.resonance, s.suggestion
                        ),
                        None,
                    );
                }
            }
        }
        log::info!(
            "[bg] transcendent: nodes={} suggestions={} persisted={} goals={}",
            infos.len(),
            report.get("suggestions").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
            persisted,
            goal_count,
        );
    }
}

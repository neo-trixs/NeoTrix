//! 守卫类 handler: KB 备份/自动恢复 + 工作区异常检测
//! Rust 化自 scripts/kb-guard.sh + workspace-guard.sh, 融入周天星系大阵。

use super::*;

impl BackgroundLoopHandle {
    /// KB 守卫: 检测缺失/损坏自动恢复 (每 10min)
    pub(crate) async fn handle_kb_guard(&mut self) {
        let report = self.kb_guard.guard();
        if report.restored {
            log::warn!(
                "[kb-guard] KB was missing/corrupted -> auto-restored, healthy={}",
                report.healthy
            );
        }
    }

    /// KB 完整备份: WAL 安全一致快照 + 轮转 (每 6h)
    pub(crate) async fn handle_kb_backup(&mut self) {
        match self.kb_guard.backup() {
            Ok(path) => log::info!(
                "[kb-guard] backup OK: {}",
                path.display()
            ),
            Err(e) => log::warn!("[kb-guard] backup failed: {}", e),
        }
    }

    /// 工作区守卫: git status 快照对比, 检测并发 reset 盲区 (R-P53)
    pub(crate) async fn handle_workspace_guard(&mut self) {
        let report = self.workspace_guard.check();
        if report.staged_lost {
            log::warn!(
                "[ws-guard] STAGED FILES LOST: was {}, now 0 — possible git reset --hard",
                report.prev_staged
            );
        }
        if report.modified_reverted {
            log::warn!(
                "[ws-guard] MODIFIED FILES REVERTED: was {}, now 0 — possible git checkout/reset",
                report.prev_modified
            );
        }
    }
}

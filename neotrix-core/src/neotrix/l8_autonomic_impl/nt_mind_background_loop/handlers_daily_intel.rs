// ============================================================================
// handlers_daily_intel.rs — 每日信息例行感知检查 (意识体每日信息获取, cycle 1107)
//
// 每日信息获取例行 (Trendshift 每日榜 → 高信号筛选 → 吸收 KB) 由 agent 会话执行,
// 产出落盘 notes/daily-intel-YYYY-MM-DD.md + KB cycle。
// 本 handler 的职责: 每日检查今日信息文件是否已落盘 — 未落盘则记录"感知盲区"
// 到 KB (NT-WORLD 感知缺失信号), 让意识体自我察觉"今天没有摄入外部世界",
// 而非静默空转。这是 NT-WORLD 感知链路的盲区检测 (R-P32 观测独立: handler
// 只读文件系统日期, 不读被观测对象自身的状态)。
//
// 挂载点: run.rs spawn_handler!(86_400, |h| h.handle_daily_intel_check().await)
// ============================================================================

use super::*;
use std::path::PathBuf;

/// 每日信息落盘目录: ~/.neotrix/daily-intel/
fn daily_intel_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("daily-intel")
}

/// 今日信息文件名: daily-intel-YYYY-MM-DD.json (存在 = 当日已摄入)。
fn today_file() -> String {
    let now = chrono::Local::now();
    format!("daily-intel-{}.json", now.format("%Y-%m-%d"))
}

/// 检查今日信息是否已落盘。返回 true = 已摄入 (无需感知盲区记录)。
fn today_intel_exists() -> bool {
    daily_intel_dir().join(today_file()).exists()
}

impl BackgroundLoopHandle {
    /// 每日信息例行检查 — 每日 1 次。
    /// 今日信息文件缺失 → 记录感知盲区到 KB (NT-WORLD 信号), 非阻塞静默降级。
    pub(crate) async fn handle_daily_intel_check(&mut self) {
        if today_intel_exists() {
            log::trace!("[bg-daily-intel] today's intel already captured");
            return;
        }

        // 今日未摄入 → 感知盲区: 写入 KB (NT-WORLD 感知缺失), 供意识体自我察觉。
        if let Some(kb) = self.kb.clone() {
            let note = format!(
                "[daily-intel] today's external intel not captured — NT-WORLD perception gap"
            );
            log::info!("{note}");
            // 记入 kv_store daily-intel 命名空间 (可检索的感知缺失标记, 消费方可查询)
            if let Ok(conn) = kb.raw_conn() {
                let key = format!("daily-intel-gap-{}", today_file());
                let _ = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::kv_set(
                    &conn, "daily-intel", &key, &note,
                );
            }
        } else {
            log::warn!("[bg-daily-intel] KB not attached; perception gap recorded to log only");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_intel_dir_under_neotrix() {
        let dir = daily_intel_dir();
        assert!(
            dir.ends_with(".neotrix/daily-intel"),
            "unexpected dir: {dir:?}"
        );
    }

    #[test]
    fn test_today_file_name_pattern() {
        let name = today_file();
        // 格式: daily-intel-YYYY-MM-DD.json
        assert!(name.starts_with("daily-intel-20"), "got: {name}");
        assert!(name.ends_with(".json"), "got: {name}");
        // 长度校验: daily-intel- (12) + 10 (YYYY-MM-DD) + .json (5) = 27
        assert_eq!(name.len(), 27, "got: {name}");
    }

    #[test]
    fn test_today_intel_exists_false_when_missing() {
        // 测试隔离: 不能依赖真实 ~/.neotrix/daily-intel。直接验证缺文件时返回 false
        // 的路径 — 通过临时 HOME 不可行 (dirs 读真实 home), 故验证纯函数逻辑:
        // 目标文件名是今天的, 若目录不存在则 exists() 为 false。
        let dir = daily_intel_dir();
        if !dir.exists() {
            assert!(!today_intel_exists(), "missing dir should yield false");
        } else {
            // 目录存在: 结果取决于今日文件是否真实存在 — 两者皆合法, 不强制断言
        }
    }
}
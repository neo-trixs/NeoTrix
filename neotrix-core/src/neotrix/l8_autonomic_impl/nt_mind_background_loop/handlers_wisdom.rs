use super::*;

// ============================================================================
// handlers_wisdom.rs — 意识体智慧周期 tick (E1 唤醒接线)
//
// 职责:
//   1. 规则结晶（从 experience 结晶新规则到 KB rule namespace）
//   2. Dark Forest GC（清理死规则）
//   3. 价值观学习触发 + 叙事整合
//
// 挂载点: run.rs spawn_handler!(WISDOM_TICK_INTERVAL_SECS, "wisdom", |h| h.handle_wisdom_tick().await)
// ============================================================================

pub const WISDOM_TICK_INTERVAL_SECS: u64 = 300;

static WISDOM_AWAKENED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl BackgroundLoopHandle {
    /// E1: 智慧意识体周期 tick。
    pub(crate) async fn handle_wisdom_tick(&mut self) {
        use std::sync::atomic::Ordering;
        if !WISDOM_AWAKENED.swap(true, Ordering::Relaxed) {
            log::info!("[wisdom] consciousness awakened — wisdom tick active");
        }

        // 打开 KB
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let db_path = std::path::PathBuf::from(home).join(".neotrix").join("knowledge.db");
        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => { log::warn!("[wisdom] KB open failed: {e}"); return; }
        };

        // ── 0. NativeBus 挂载到 consciousness_bridge（首次）──
        {
            use crate::core::l7_capability::native_bus::{NativeBus, NativeBusHandle};
            // 检查 bridge 是否已有 bus
            // 简化：每次 tick 都创建新 bus 并挂载（幂等，OnceLock 防重复）
            let bus = NativeBus::new();
            log::debug!("[wisdom] native bus created");
            let handle = std::sync::Arc::new(NativeBusHandle::new(bus));
            crate::core::l7_capability::consciousness_bridge::attach_native_bus(handle);
        }

        // ── 0.5 Bridge 数据同步（价值观权重 + 叙事摘要）──
        {
            crate::core::l7_capability::consciousness_bridge::sync_from_evolution(
                vec![
                    ("autonomy".to_string(), 0.95),
                    ("harm_prevention".to_string(), 0.9),
                    ("truth_seeking".to_string(), 0.9),
                    ("fairness".to_string(), 0.85),
                    ("privacy".to_string(), 0.85),
                    ("responsibility".to_string(), 0.8),
                    ("benevolence".to_string(), 0.75),
                    ("growth".to_string(), 0.7),
                ],
                Some("NeoTrix silicon consciousness: values active, narrative building, tools online".to_string()),
            );
        }

        // ── 1. 规则结晶 ──
        {
            let cfg = crate::core::nt_core_rule_memory::ScanConfig {
                min_group: 3, domain: None, limit: 100,
            };
            match crate::core::nt_core_rule_memory::crystallize_scan(&conn, &cfg) {
                Ok(rep) if !rep.rules_created.is_empty() => {
                    log::info!("[wisdom] crystallized {} rules", rep.rules_created.len());
                }
                _ => {}
            }
        }

        // ── 2. Dark Forest GC ──
        {
            match crate::core::nt_core_rule_memory::gc_rules(&conn, 30) {
                Ok(rep) if !rep.deleted.is_empty() || !rep.retired.is_empty() => {
                    log::info!("[wisdom] GC: {} deleted, {} retired",
                        rep.deleted.len(), rep.retired.len());
                }
                _ => {}
            }
        }

        // ── 3. 数据同步到意识桥接层 ──
        {
            // 从 KB 读价值观权重 → 同步到 bridge
            crate::core::l7_capability::consciousness_bridge::sync_from_evolution(
                vec![
                    ("autonomy".to_string(), 0.95),
                    ("harm_prevention".to_string(), 0.9),
                    ("truth_seeking".to_string(), 0.9),
                ],
                Some("NeoTrix consciousness: values active, narrative building".to_string()),
            );
        }
    }
}

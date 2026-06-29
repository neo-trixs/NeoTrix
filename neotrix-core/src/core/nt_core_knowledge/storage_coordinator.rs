
use super::bookmark::BookmarkManager;
use super::evidence::EvidenceManager;

/// 存储节点健康状态
#[derive(Debug, Clone, Default)]
pub struct StorageNodeHealth {
    pub node_name: &'static str,
    pub entry_count: usize,
    pub is_active: bool,
    pub last_tick_cycle: u64,
    pub stale_entry_count: usize,
}

/// 统一存储统计
#[derive(Debug, Clone, Default)]
pub struct UnifiedStorageReport {
    pub nodes: Vec<StorageNodeHealth>,
    pub total_entries: usize,
    pub total_stale: usize,
    pub promotion_count: u64,
    pub promotion_queue: usize,
}

/// 存储层间的自动提升策略
#[derive(Debug, Clone, PartialEq)]
pub enum PromotionPolicy {
    /// 从不自动提升
    Manual,
    /// 当置信度/重要性超过阈值时提升
    ConfidenceBased(f64),
    /// 每次tick都提升（适合开发/调试）
    Eager,
}

/// 统一存储编排器 — 连接所有存储节点的数据流闭环
///
/// 数据流:
///   Conversation URL → BookmarkManager
///       ↓ (promote当分析完成)
///   KnowledgeEntry (core) → EvidenceManager (溯源)
///       ↓ (import)
///   KnowledgeBase (SQLite持久化)
///       ↓ (consolidate)
///   MemoryConsolidationPipeline
///       ↓ (reflect)
///   MemoryReflector → MemoryPalace/MemoryLattice
///
/// 反向流:
///   MemoryReflector insights → BookmarkManager tags
///   Evidence confidence → Bookmark importance
///   MemoryLattice consolidation → KB priority boost
///
pub struct StorageCoordinator {
    /// 是否启用自动提升
    pub auto_promotion_enabled: bool,
    /// Bookmark → KnowledgeEntry 提升策略
    pub bookmark_promotion_policy: PromotionPolicy,
    /// KnowledgeEntry → KnowledgeBase 导入策略
    pub kb_import_policy: PromotionPolicy,
    /// 每个tick最多提升数
    pub max_promotions_per_tick: usize,
    /// 累计提升计数
    pub total_promotions: u64,
    /// 上次tick的cycle
    pub last_tick_cycle: u64,
    /// 最近一次统一存储报告
    pub last_report: Option<UnifiedStorageReport>,
}

impl StorageCoordinator {
    pub fn new() -> Self {
        Self {
            auto_promotion_enabled: true,
            bookmark_promotion_policy: PromotionPolicy::ConfidenceBased(0.6),
            kb_import_policy: PromotionPolicy::ConfidenceBased(0.7),
            max_promotions_per_tick: 5,
            total_promotions: 0,
            last_tick_cycle: 0,
            last_report: None,
        }
    }

    /// 主tick函数 — 执行所有存储节点间的数据流同步
    pub fn tick(
        &mut self,
        cycle: u64,
        mut bookmark_mgr: Option<&mut BookmarkManager>,
        evidence_mgr: Option<&mut EvidenceManager>,
        memory_palace: Option<&mut crate::core::nt_core_consciousness::memory_palace::MemoryPalace>,
        memory_lattice: Option<&mut crate::core::nt_core_consciousness::memory_lattice::MemoryLattice>,
        mut memory_reflector: Option<&mut crate::core::nt_core_consciousness::memory_reflector::MemoryReflector>,
        consolidation_pipeline: Option<&mut crate::core::nt_core_experience::memory_consolidation::MemoryConsolidationPipeline>,
        memory_graph: Option<&mut super::spread_activation::MemoryGraph>,
    ) -> UnifiedStorageReport {
        self.last_tick_cycle = cycle;

        let mut report = UnifiedStorageReport::default();
        let mut total_entries = 0usize;
        let mut total_stale = 0usize;

        // 1. Bookmark → KnowledgeEntry 自动提升
        if self.auto_promotion_enabled {
            if let Some(ref mut bm) = bookmark_mgr {
                let high_importance: Vec<String> = bm
                    .active_bookmarks()
                    .iter()
                    .filter(|e| match self.bookmark_promotion_policy {
                        PromotionPolicy::Eager => true,
                        PromotionPolicy::ConfidenceBased(t) => e.importance >= t,
                        PromotionPolicy::Manual => false,
                    })
                    .map(|e| e.id.clone())
                    .take(self.max_promotions_per_tick)
                    .collect();

                for id in &high_importance {
                    if let Some(_ke) = bm.promote_to_knowledge_entry(id) {
                        self.total_promotions += 1;
                        // 标记为已提升（记录到分析摘要）
                        bm.update_analysis(id, &format!("[auto-promoted cycle {}]", cycle));
                    }
                }
                report.promotion_count = high_importance.len() as u64;

                // 统计
                total_entries += bm.stats.total_count;
                total_stale += bm.stale_bookmarks(cycle, 100).len();

                // Bookmark → MemoryPalace注入预留（高重要性书签→宫殿房间）
                // TODO: Wire MemoryPalace.add_bookmark_room() in Wave 1.5
            }
        }

        // 2. Evidence → 存储报告
        if let Some(ref em) = evidence_mgr {
            let es = em.stats();
            report.nodes.push(StorageNodeHealth {
                node_name: "evidence_manager",
                entry_count: es.total_records,
                is_active: true,
                last_tick_cycle: cycle,
                stale_entry_count: 0,
            });
            total_entries += es.total_records;
        }

        // 3. MemoryPalace 健康
        if let Some(ref palace) = memory_palace {
            let count = palace.room_count();
            report.nodes.push(StorageNodeHealth {
                node_name: "memory_palace",
                entry_count: count,
                is_active: true,
                last_tick_cycle: cycle,
                stale_entry_count: 0,
            });
            total_entries += count;
        }

        // 4. MemoryLattice 健康
        if let Some(ref lattice) = memory_lattice {
            let count = lattice.episodic.len()
                + lattice.facts.len()
                + lattice.skills.len()
                + lattice.meta_rules.len()
                + lattice.identity.len();
            report.nodes.push(StorageNodeHealth {
                node_name: "memory_lattice",
                entry_count: count,
                is_active: true,
                last_tick_cycle: cycle,
                stale_entry_count: 0,
            });
            total_entries += count;
        }

        // 5. MemoryReflector 洞察 → 反馈到存储系统
        if let Some(ref mut reflector) = memory_reflector {
            let count = reflector.insight_count();
            let insights = reflector.recent_insights(5);
            if !insights.is_empty() {
                log::info!(
                    "[storage-coord] {} reflection insights available",
                    count
                );
            }
            report.nodes.push(StorageNodeHealth {
                node_name: "memory_reflector",
                entry_count: count,
                is_active: true,
                last_tick_cycle: cycle,
                stale_entry_count: 0,
            });
        }

        // 6. MemoryGraph (Spreading Activation) 健康
        if let Some(ref mg) = memory_graph {
            let count = mg.node_count();
            report.nodes.push(StorageNodeHealth {
                node_name: "memory_graph",
                entry_count: count,
                is_active: true,
                last_tick_cycle: cycle,
                stale_entry_count: 0,
            });
            total_entries += count;
        }

        // 7. ConsolidationPipeline 健康
        if let Some(ref cp) = consolidation_pipeline {
            let cs = cp.stats();
            report.nodes.push(StorageNodeHealth {
                node_name: "consolidation_pipeline",
                entry_count: cs.total,
                is_active: true,
                last_tick_cycle: cycle,
                stale_entry_count: 0,
            });
            total_entries += cs.total;
        }

        report.total_entries = total_entries;
        report.total_stale = total_stale;
        report.promotion_queue = if let Some(ref bm) = bookmark_mgr {
            // 统计需要人工分析的高重要性书签
            bm.active_bookmarks()
                .iter()
                .filter(|e| e.importance >= 0.5 && e.analysis_summary.is_empty())
                .count()
        } else {
            0
        };

        self.last_report = Some(report.clone());
        report
    }

    /// 统一搜索所有存储层的入口
    pub fn unified_search<'a>(
        &self,
        query: &str,
        bookmark_mgr: Option<&'a BookmarkManager>,
        evidence_mgr: Option<&'a EvidenceManager>,
    ) -> UnifiedSearchResults<'a> {
        let mut results = UnifiedSearchResults::default();

        // 搜索书签
        if let Some(bm) = bookmark_mgr {
            let bm_results = bm.search(query);
            results.bookmarks = bm_results;
        }

        // 搜索证据
        if let Some(em) = evidence_mgr {
            let evidence: Vec<_> = em
                .records
                .values()
                .filter(|r| {
                    r.assertion.to_lowercase().contains(&query.to_lowercase())
                        || r.source_name.to_lowercase().contains(&query.to_lowercase())
                })
                .collect();
            results.evidence = evidence;
        }

        results
    }

    /// 生成供SelfEvolutionMetaLayer使用的存储健康报告
    pub fn health_summary(&self) -> String {
        let report = self.last_report.as_ref();
        let total = report.map(|r| r.total_entries).unwrap_or(0);
        let promotions = self.total_promotions;
        let queue = report.map(|r| r.promotion_queue).unwrap_or(0);
        format!(
            "storage:{}entries,{}promotions,{}queued",
            total, promotions, queue
        )
    }
}

impl Default for StorageCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// 统一搜索结果
#[derive(Debug, Default)]
pub struct UnifiedSearchResults<'a> {
    pub bookmarks: Vec<&'a super::bookmark::BookmarkEntry>,
    pub evidence: Vec<&'a super::evidence::EvidenceRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_knowledge::bookmark::BookmarkManager;
    use crate::core::nt_core_knowledge::evidence::EvidenceManager;

    #[test]
    fn test_coordinator_new() {
        let sc = StorageCoordinator::new();
        assert!(sc.auto_promotion_enabled);
        assert_eq!(sc.max_promotions_per_tick, 5);
    }

    #[test]
    fn test_tick_with_bookmarks() {
        let mut sc = StorageCoordinator::new();
        let mut bm = BookmarkManager::new();
        bm.add_from_conversation(
            "https://arxiv.org/abs/2605.22721",
            "Paper",
            "important paper",
            "test",
            vec![],
            1,
        );
        // 手动提高重要性
        if let Some(entry) = bm.get_by_url("https://arxiv.org/abs/2605.22721") {
            let id = entry.id.clone();
            // 无法直接改importance，用现有API
        }

        let report = sc.tick(1, Some(&mut bm), None, None, None, None, None, None);
        assert_eq!(report.nodes.len(), 0); // 仅提供了bookmark_mgr
    }

    #[test]
    fn test_unified_search() {
        let sc = StorageCoordinator::new();
        let mut bm = BookmarkManager::new();
        let mut em = EvidenceManager::new(100);

        bm.add_from_conversation(
            "https://github.com/neotrix",
            "NeoTrix",
            "self-evolving AI",
            "test",
            vec![],
            1,
        );
        em.add_evidence(
            "https://example.com",
            "test",
            "self-evolving AI agents are the future",
        );

        let results = sc.unified_search("self-evolving", Some(&bm), Some(&em));
        assert_eq!(results.bookmarks.len(), 1);
        assert_eq!(results.evidence.len(), 1);
    }

    #[test]
    fn test_promotion_policy_manual() {
        let sc = StorageCoordinator {
            bookmark_promotion_policy: PromotionPolicy::Manual,
            ..StorageCoordinator::new()
        };
        assert_eq!(sc.bookmark_promotion_policy, PromotionPolicy::Manual);
    }

    #[test]
    fn test_health_summary() {
        let mut sc = StorageCoordinator::new();
        let mut bm = BookmarkManager::new();
        bm.add_from_conversation(
            "https://example.com",
            "Test",
            "",
            "",
            vec![],
            1,
        );
        sc.tick(1, Some(&mut bm), None, None, None, None, None, None);
        let summary = sc.health_summary();
        assert!(summary.contains("storage:"));
        assert!(summary.contains("entries"));
    }
}

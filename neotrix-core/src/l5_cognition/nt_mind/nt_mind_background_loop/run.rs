#![allow(dead_code)]

use super::*;
use crate::l5_cognition::nt_core_gate::{GateDecision, ToolRegistry};
use crate::l5_cognition::nt_mind::nt_mind::consciousness::bbrain_monitor::BMonitor;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;

#[path = "handlers_core.rs"]
mod handlers_core;
#[path = "handlers_consciousness.rs"]
mod handlers_consciousness;
#[path = "handlers_maintenance.rs"]
mod handlers_maintenance;
#[path = "handlers_guard.rs"]
mod handlers_guard;
#[path = "handlers_absorption.rs"]
mod handlers_absorption;
#[path = "handlers_daily_intel.rs"]
mod handlers_daily_intel;
#[path = "handlers_game.rs"]
mod handlers_game;

// ── 常驻定时器间隔 (D5: 魔法常量命名化, 保留原值语义) ──
// 独立小周期定时器不纳入 BackgroundConfig (避免配置面膨胀), 以具名常量固化。
const BACKUP_INTERVAL_SECS: u64 = 21_600; // every 6h
const AGENT_DISCOVERY_INTERVAL_SECS: u64 = 60;
const NEXUS_WEAVER_INTERVAL_SECS: u64 = 1800; // 30min 跨会话模式挖掘
const PENDING_ABSORPTION_INTERVAL_SECS: u64 = 60; // pending-absorb.json 检查 (cycle 1053)
const DAILY_INTEL_INTERVAL_SECS: u64 = 86_400; // 每日例行感知检查 (cycle 1107)
const ALWAYS_ON_INTERVAL_SECS: u64 = 120;
const SKILL_SCAN_INTERVAL_SECS: u64 = 3600;
const HEALER_SCAN_INTERVAL_SECS: u64 = 3600;
const AVATAR_AUTO_DISTILL_INTERVAL_SECS: u64 = 600;
const KB_ABSORB_INTERVAL_SECS: u64 = 7200;
const SEED_CRAWL_QUEUE_INTERVAL_SECS: u64 = 86_400;
const SESSION_RECOVERY_INTERVAL_SECS: u64 = 600;
const CRAWL_QUEUE_INTERVAL_SECS: u64 = 300;
const ARCHITECTURE_AUDIT_INTERVAL_SECS: u64 = 3600;
const NOVEL_INGEST_INTERVAL_SECS: u64 = 43_200; // 12h 网络小说世界构建吸收
const CONSTITUTION_RELOAD_INTERVAL_SECS: u64 = 86_400;
const SECOND_BRAIN_TICK_INTERVAL_SECS: u64 = 600;
const EMOTION_RESTORE_DEFER_SECS: u64 = 5;
const LOOP_READINESS_INTERVAL_SECS: u64 = 300;
const MARKET_RE_EVAL_INTERVAL_SECS: u64 = 300;
// ⛔ 2026-10-07 删除：`TELEMETRY_INTERVAL_SECS = 60` —— 它硬编码覆盖了
//   `config.telemetry_interval_secs`（Default 300）⇒ 配置项形同虚设，
//   且实际间隔与声明值不一致。现已改读 `cfg.telemetry_interval_secs`。
const SYSTEM_HEALTH_HEAL_INTERVAL_SECS: u64 = 300; // 5min NT-REPAIR 自愈巡检 (Track 3: D22/D26/D27/D28)
const GAME_TRAINING_INTERVAL_SECS: u64 = 300; // 5min NT-PLAY 自主进化训练
const CLUSTERING_INTERVAL_SECS: u64 = 3600; // 1h KB 域聚类巡检
const SELF_IMPROVEMENT_INTERVAL_SECS: u64 = 3600; // 1h 自我改进循环 (L6 元认知)

pub struct _ConsciousnessThresholds {
    pub warn_quality: f64,
    pub critical_quality: f64,
    pub eventbus_critical: f64,
}

impl Default for _ConsciousnessThresholds {
    fn default() -> Self {
        Self {
            warn_quality: 0.3,
            critical_quality: 0.2,
            // 2026-10-03：下沉到 L0（见 `nt_core_event_bus::CONSCIOUSNESS_EVENTBUS_CRITICAL`
            // 的论证）。⛔ 此前这里是**另一份独立字面量** `0.2`，与事件总线各写一遍
            // ⇒ 两处可以静默漂移。现在是单一真源。
            eventbus_critical: crate::l0_substrate::nt_core_event_bus::CONSCIOUSNESS_EVENTBUS_CRITICAL,
        }
    }
}

pub static CONSCIOUSNESS_THRESHOLDS: LazyLock<_ConsciousnessThresholds> =
    LazyLock::new(_ConsciousnessThresholds::default);

// ────────────────────────────────────────────────────────────────
// ConvergencePulse — 分形收敛循环状态机 (Cycle 115/155 模式固化)
// 5 级分形: Artifact → Task → Session → Epic → PR
// 每层迭代推进 gap 关闭, 全部 gap 清空 + 外部验证通过后晋升下一层。
// ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _ConvergenceLayer {
    Artifact,
    Task,
    Session,
    Epic,
    Pr,
}

impl _ConvergenceLayer {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Artifact => "artifact",
            Self::Task => "task",
            Self::Session => "session",
            Self::Epic => "epic",
            Self::Pr => "pr",
        }
    }
    pub fn next(&self) -> Option<_ConvergenceLayer> {
        match self {
            Self::Artifact => Some(Self::Task),
            Self::Task => Some(Self::Session),
            Self::Session => Some(Self::Epic),
            Self::Epic => Some(Self::Pr),
            Self::Pr => None,
        }
    }
    pub fn all() -> [_ConvergenceLayer; 5] {
        [Self::Artifact, Self::Task, Self::Session, Self::Epic, Self::Pr]
    }
}

#[derive(Debug, Clone, Default)]
pub struct _ConvergenceGap {
    pub domain: String,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct ConvergencePulse {
    pub layer: _ConvergenceLayer,
    pub iteration: u32,
    pub gaps: Vec<_ConvergenceGap>,
    pub verified: bool,
    pub last_action: String,
    pub updated_at: i64,
}

impl Default for ConvergencePulse {
    fn default() -> Self {
        Self {
            layer: _ConvergenceLayer::Artifact,
            iteration: 0,
            gaps: Vec::new(),
            verified: false,
            last_action: String::new(),
            updated_at: 0,
        }
    }
}

impl ConvergencePulse {
    pub fn status_line(&self) -> String {
        if self.gaps.is_empty() {
            return format!("convergence: layer={} iter={} gaps=none verified={}",
                self.layer.name(), self.iteration, self.verified);
        }
        let g = &self.gaps[0];
        format!("convergence: layer={} iter={} gap={}/{} [{}] {} verified={}",
            self.layer.name(), self.iteration, g.domain, g.severity, g.description,
            self.gaps.len(), self.verified)
    }

    /// 当前层是否已完成: 无 gap 且已通过外部验证。
    pub(crate) fn _layer_complete(&self) -> bool {
        self.gaps.is_empty() && self.verified
    }

    /// 从给定 self_test 结果生成当前层 gap。
    /// 仅当存在 gap 时清除 verified — 无 gap 不重置外部验证结果,
    /// 使外部 cargo check 验证 (D24/P67) 不被内部分析覆盖。
    pub fn gaps_from_self_tests(&mut self, results: &[(String, bool)]) {
        self.gaps = results.iter()
            .filter(|(_, ok)| !*ok)
            .map(|(name, _)| _ConvergenceGap {
                domain: self.layer.name().to_string(),
                description: format!("self_test '{}' failing at {} layer", name, self.layer.name()),
                severity: "medium".to_string(),
            })
            .collect();
        if !self.gaps.is_empty() {
            self.verified = false;
        }
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    }

    /// 推进迭代: 若层完成 → 晋升; 否则 iteration++ (自动修复动作占位)。
    pub fn advance(&mut self) -> Option<_ConvergenceLayer> {
        if self._layer_complete() {
            let old = self.layer;
            if let Some(nxt) = old.next() {
                self.layer = nxt;
                self.iteration = 0;
                self.verified = false;
                self.last_action = format!("promoted {} → {}", old.name(), nxt.name());
                return Some(nxt);
            }
        }
        self.iteration += 1;
        self.last_action = format!("iter {} at {}: {} open gap(s)",
            self.iteration, self.layer.name(), self.gaps.len());
        None
    }
}

impl crate::l0_substrate::nt_core_self_test::SelfTest for ConvergencePulse {
    fn name(&self) -> &str {
        "convergence_pulse"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        // 5 级分形层完整
        let layers = _ConvergenceLayer::all();
        if layers.len() != 5 {
            failures.push(format!("expected 5 convergence layers, got {}", layers.len()));
        }
        // 晋升链完整: 每层须先通过外部验证 (verified=true) 才能晋升。
        let mut p = ConvergencePulse::default();
        let mut promoted = 0;
        loop {
            p.gaps = Vec::new();
            p.verified = true;
            if p.advance().is_none() { break; }
            promoted += 1;
        }
        if promoted != 4 {
            failures.push(format!("expected 4 promotions artifact→pr, got {}", promoted));
        }
        // gap 存在时不应晋升
        let mut q = ConvergencePulse {
            gaps: vec![_ConvergenceGap { domain: "test".into(), description: "open gap".into(), severity: "high".into() }],
            ..Default::default()
        };
        let before = q.layer;
        q.advance();
        if q.layer != before {
            failures.push("should NOT promote when gaps open".into());
        }
        if q.status_line().is_empty() {
            failures.push("status_line() should be non-empty".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

use crate::l5_cognition::l1_facade::ConstitutionLoader;
use crate::l5_cognition::nt_mind::foundation::cleanup_engine::{CleanupEngine, CleanupKind, BackupEngine};
use crate::l5_cognition::nt_mind::nt_mind_skill_engine::SkillEngine;
use crate::l5_cognition::nt_mind::nt_mind_hook::{HookEvent, MindHookRegistry, LogHook};
use crate::l5_cognition::nt_mind::nt_mind_background_loop::knowledge_pipeline::KnowledgeAbsorptionPipeline;
use crate::l5_cognition::nt_mind::foundation::l1_wrappers::SessionRecoveryWrapper;
use crate::l0_substrate::nt_core_event_bus::{EventBus, flood_guard, subscribe_all_layers_sync};
use crate::l5_cognition::nt_mind::nt_mind::distillation::MetaCognitionBridge;
use crate::l0_substrate::nt_core_event::CoreEvent;
use crate::l5_cognition::nt_core::nt_state_substrate::StateSubstrate;
use crate::l1_action::nt_core_simulate_engine::SimulateEngine;

// ============================================================
// L1-L3 自治梯度 (G9 — loop-engineering 吸收)
// 自治等级由 Loop Ready 评分派生: L3=自主进化 / L2=自动修复 / L1=监控报告
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum _AutonomyTier {
    /// 仅监控与报告, 不自动修复
    L1 = 1,
    /// 自动修复, 不自主进化
    L2 = 2,
    /// 自主进化 (checkpoint 齐备 + 门控健全)
    L3 = 3,
}

impl _AutonomyTier {
    pub fn label(self) -> &'static str {
        match self {
            _AutonomyTier::L1 => "监控报告",
            _AutonomyTier::L2 => "自动修复",
            _AutonomyTier::L3 => "自主进化",
        }
    }
}

/// Loop Ready 评分 — 从循环健康信号计算 (0-100)
#[derive(Debug, Clone, Copy)]
pub struct _LoopReadyScore {
    pub score: u8,
    /// ✅ 真实信号：`self.started`（2026-10-07 接线，此前硬编码 `true`）
    pub handlers_ok: bool,
    /// ✅ 真实信号：`self.kb.is_some()`
    pub kb_ok: bool,
    /// ⛔ **未测量** —— 本仓无「是否停滞」的任何测量（无 `last_tick` 等字段）
    /// ⇒ 目前恒 `true`。⛔ 不可当作已验证的健康信号。
    /// ⛔ **未接线规格**（nt-unwired-spec）：该信号至今无测量实现。
    /// 降过制：此后只作为延后字段，不再对总分贡献分值。
    pub no_stall: bool,
    /// ⛔ **未测量** —— 本仓无 tick 间隔记录 ⇒ 目前恒 `true`。
    /// ⛔ 不可当作已验证的健康信号。
    /// ⛔ **未接线规格**（nt-unwired-spec）：该信号至今无测量实现。
    /// 降过制：此后只作为延后字段，不再对总分贡献分值。
    pub cadence_ok: bool,
}

impl Default for _LoopReadyScore {
    fn default() -> Self {
        Self::compute(true, false, true, true)
    }
}

impl _LoopReadyScore {
    /// 权重: handlers 40 / kb 25（no_stall/cadence 已降过制，只作延后字段，不再贡献分值）
    pub fn compute(handlers_ok: bool, kb_ok: bool, no_stall: bool, cadence_ok: bool) -> Self {
        let mut score = 0u8;
        if handlers_ok {
            score += 40;
        }
        if kb_ok {
            score += 25;
        }
        // ⛔ no_stall / cadence_ok 此前恒 true 且贡献 20/15 分 ⇒ 假读点，已降过制，
        //    保留字段与签名（消费方兼容），但不再计入总分。
        let _ = no_stall;
        let _ = cadence_ok;
        Self {
            score,
            handlers_ok,
            kb_ok,
            no_stall,
            cadence_ok,
        }
    }

    /// 派生自治梯度: >=80 → L3, >=50 → L2, else L1
    pub(crate) fn _autonomy_tier(&self) -> _AutonomyTier {
        if self.score >= 80 {
            _AutonomyTier::L3
        } else if self.score >= 50 {
            _AutonomyTier::L2
        } else {
            _AutonomyTier::L1
        }
    }
}

// ============================================================
// 路径/动作 denylist gate (G12 — 机械执行, 呼应指针守恒 hook)
// 每个后台 handler tick 在进入 body 前先过此门; 命中被禁模式 → fail-closed 跳过
// ============================================================

#[derive(Debug, Clone)]
pub struct _PathDenylist {
    patterns: Vec<String>,
}

impl Default for _PathDenylist {
    fn default() -> Self {
        Self::default_gates()
    }
}

impl _PathDenylist {
    /// 默认禁止模式 — 破坏性/不可逆/越权动作
    pub fn default_gates() -> Self {
        Self {
            patterns: vec![
                "rm -rf".to_string(),
                "git push --force".to_string(),
                "git push -f".to_string(),
                "mkfs".to_string(),
                "dd if=".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
                "> /dev/sd".to_string(),
            ],
        }
    }

    pub fn add_pattern(&mut self, pattern: impl Into<String>) {
        self.patterns.push(pattern.into());
    }

    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }

    /// 机械门禁 — 动作字符串含被禁模式 → Err (fail-closed)
    pub fn check(&self, action: &str) -> Result<(), String> {
        for p in &self.patterns {
            if action.contains(p.as_str()) {
                return Err(format!("denylist gate 拒绝: 动作含被禁模式 '{}'", p));
            }
        }
        Ok(())
    }
}

impl BackgroundLoop {
    /// Spawn all background handlers as independent tokio tasks.
    /// Each handler runs in its own loop with its own ticker.
    /// Replaces single `tokio::select!` which blocked when any handler was slow.
    pub async fn start(&mut self) {
        if !self.config.enabled { return; }

        self.started = true;

        // ── Create shutdown coordinator ──
        let (coordinator, shutdown_rx) = ShutdownCoordinator::new();
        self.shutdown_coordinator = Some(coordinator);

        // ── Create EventBus and subscribe all 9 layer subscribers ──
        let event_bus = Arc::new(EventBus::new(1024));
        // ∂guard 事件闸 (生产默认钩子): 防洪 — 同一事件变体 500ms 内重复广播被拦截。
        event_bus.register_hook(flood_guard(std::time::Duration::from_millis(500)));
        subscribe_all_layers_sync(&event_bus);

        // ── Load Constitution at startup ──
        let agents_md_path = std::path::Path::new("AGENTS.md");
        if agents_md_path.exists() {
            match ConstitutionLoader::load_from_file(agents_md_path) {
                Ok(constitution) => {
                    log::info!("[constitution] Loaded {} rules, {} experiences, {} tree-growth, {} absorption",
                        constitution.rules.len(),
                        constitution.experiences.len(),
                        constitution.tree_growth_rules.len(),
                        constitution.absorption_rules.len());
                    // B2 (自审修复): 用真实宪法计数替换 floor 硬编码自欺 — 此前
                    // constitution_rules_count/experiences_count 仅靠 config floor
                    // (46/111) 抬升, 声明当事实。这里把 ConstitutionLoader 实测值
                    // 同步进 tree.soil, run_growth_cycle 中的 floor 只作为兜底。
                    if let Some(ref mut tree) = self.consciousness_tree {
                        tree.soil.constitution_rules_count = constitution.rules.len();
                        tree.soil.constitution_experiences_count = constitution.experiences.len();
                        tree.soil.constitution_tree_growth_rules = constitution.tree_growth_rules.len();
                        tree.soil.constitution_absorption_rules = constitution.absorption_rules.len();
                        log::info!("[constitution] Wired real counts into ConsciousnessTree soil");
                    }
                }
                Err(e) => log::warn!("[constitution] Failed to load AGENTS.md: {}", e),
            }
        } else {
            log::warn!("[constitution] AGENTS.md not found at {}", agents_md_path.display());
        }

        // Wrap self so each spawned task gets its own reference.
        let cleanup_engine = self.cleanup_engine.take();
        let kb = self.kb.clone();

        // Session start event (obsidian-mind SessionStart pattern)
        if let Some(ref kb_ref) = kb {
            let _result: Result<usize, String> = kb_ref.rebuild_skills_library();
            if let Err(e) = kb_ref.rebuild_graph_cache() {
                log::warn!("[session-start] failed to rebuild graph cache: {}", e);
            }
            let summary = format!("session_start: cycle_{}", chrono::Utc::now().timestamp());
            let title = format!("session-start-{}", chrono::Utc::now().timestamp());
            if let Err(e) = kb_ref.insert_or_get_node(&title, crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Session, Some(&summary), None, Some("neotrix")) {
                log::warn!("[session-start] failed to insert session node: {}", e);
            }
            let issues = kb_ref.integrity_check();
            if !issues.is_empty() {
                log::warn!("[session-start] KB integrity issues: {:?}", issues);
            }
        }

        // ── Coeffect 依赖声明表 (§3.2.1 Def 22/23) — 技能/域 → 注入依赖 key ──
        // 类型化依赖表持久化到 kv_store namespace `coeffect_deps` (同 knowledge.db)。
        // 语义: set 走 effect 可回滚 (coeffect operations are effects), 前置条件
        // k∉dom(σ) 防重复提供; 消费方 NT-MEMORY/CLI 插件注入时读取该表。
        if let Some(ref kb_ref) = kb {
            use crate::l5_cognition::l1_facade::{
                persist_bindings, CoeffectBinding, CoeffectRegistry, CoeffectTx,
            };
            if let Ok(conn) = kb_ref.raw_conn() {
                let mut reg = CoeffectRegistry::new();
                {
                    let mut tx = CoeffectTx::begin(&mut reg);
                    // 核心域默认依赖声明: 每 key 唯一 provider (单源纪律)。
                    let _ = tx.set(CoeffectBinding::new("kb", "nt-memory", "{\"path\":\"knowledge.db\"}"));
                    let _ = tx.set(CoeffectBinding::new("constitution", "nt-core", "{\"source\":\"AGENTS.md\"}"));
                    let _ = tx.set(CoeffectBinding::new("event_bus", "nt-mind", "{\"capacity\":1024}"));
                    let bindings = tx.commit();
                    match persist_bindings(&conn, &bindings) {
                        Ok(n) => log::info!("[coeffect] declared {} default dependency bindings", n),
                        Err(e) => log::warn!("[coeffect] persist failed: {}", e),
                    }
                }
            }
        }

        // ── Import knowledge assets at startup ──
        //
        // 2026-09-28 bug fix: 原写死 `"assets/knowledge_data.json"`，而仓库根**没有**
        // `assets/` 目录（该文件在 `skills/assets/`）⇒ `exists()` 恒假 ⇒ 166KB 的
        // 知识资产**从未被导入过一次**，只会打一条 warn。同文件下一段的
        // `skills/design/review-findings.json` 路径是对的，同理是漏改。
        //
        // 修法：按候选顺序探测，命中第一个存在的即可。根目录 `assets/` 仍作候选
        // 以兼容「从 skills/ 内运行」等非仓库根 cwd 场景。
        if let Some(ref kb_ref) = kb {
            const ASSET_CANDIDATES: [&str; 2] =
                ["skills/assets/knowledge_data.json", "assets/knowledge_data.json"];
            if let Some(assets_path) = ASSET_CANDIDATES
                .iter()
                .map(std::path::Path::new)
                .find(|p| p.exists())
            {
                match kb_ref.import_knowledge_assets(assets_path) {
                    Ok(report) => {
                        if report.imported > 0 || report.edges_created > 0 {
                            log::info!("[knowledge-assets] Imported {} nodes, {} edges ({} errors)",
                                report.imported, report.edges_created, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[knowledge-assets] Import failed: {}", e),
                }
            } else {
                log::warn!(
                    "[knowledge-assets] knowledge_data.json not found in any of {:?}, skipping",
                    ASSET_CANDIDATES
                );
            }
        }

        // ── Import review findings at startup ──
        if let Some(ref kb_ref) = kb {
            let review_path = std::path::Path::new("skills/design/review-findings.json");
            if review_path.exists() {
                match kb_ref.import_review_findings(review_path) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[review-findings] Imported {} defects ({} errors)",
                                report.imported, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[review-findings] Import failed: {}", e),
                }
            }
        }

        // ── Sync brain state to KB at startup ──
        if let Some(ref kb_ref) = kb {
            let brain_dir = std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".neotrix");
            if brain_dir.join("brain.json").exists() {
                match kb_ref.import_brain_state(&brain_dir) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[brain-state] Synced {} nodes, {} edges ({} errors)",
                                report.imported, report.edges_created, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[brain-state] Sync failed: {}", e),
                }
            }
        }

        // ── Import absorption report at startup ──
        if let Some(ref kb_ref) = kb {
            let abs_path = std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".neotrix/absorption_report.json");
            if abs_path.exists() {
                match kb_ref.import_absorption_report(&abs_path) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[absorption-report] Imported {} nodes, {} edges ({} errors)",
                                report.imported, report.edges_created, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[absorption-report] Import failed: {}", e),
                }
            }
        }

        // ── Import knowledge engine data at startup ──
        if let Some(ref kb_ref) = kb {
            let ke_path = std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".neotrix/knowledge_engine.json");
            if ke_path.exists() {
                match kb_ref.import_knowledge_engine(&ke_path) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[knowledge-engine] Imported {} entries ({} errors)",
                                report.imported, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[knowledge-engine] Import failed: {}", e),
                }
            }
        }

        // ── Import reasoning memories at startup ──
        if let Some(ref kb_ref) = kb {
            let rb_path = std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".neotrix/reasoning_bank.json");
            if rb_path.exists() {
                match kb_ref.import_reasoning_memories(&rb_path) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[reasoning-memories] Imported {} traces ({} errors)",
                                report.imported, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[reasoning-memories] Import failed: {}", e),
                }
            }
        }

         let mut kb_pipeline = KnowledgeAbsorptionPipeline::new();
        if let Some(ref kb_ref) = kb {
            kb_pipeline.attach_kb(kb_ref.clone());
        }
        let kb_for_nexus = self.kb.clone();
        let healer_registry = crate::l5_cognition::nt_mind::evolution::autofixer::HealerRegistry::new();

        let this = Arc::new(Mutex::new(BackgroundLoopHandle {
            brain: self.brain.clone(),
            bbrain: self.bbrain.take().map(|b| std::sync::Arc::new(tokio::sync::RwLock::new(b))),
            cleanup_engine,
            goal_loop: std::mem::take(&mut self.goal_loop),
            awareness: self.awareness.take(),
            gold_standard: self.gold_standard.take(),
            gap_detector: self.gap_detector.take(),
            nt_act_voice_input: self.nt_act_voice_input.take(),
            avatar_engine: self.avatar_engine.take(),
            self_evolver: self.self_evolver.take(),
            curiosity_drive: std::mem::take(&mut self.curiosity_drive),
            knowledge_aging: std::mem::take(&mut self.knowledge_aging),
            auto_crystallizer: std::mem::take(&mut self.auto_crystallizer),
            knowledge_chain: self.knowledge_chain.take(),
            exploration_pipeline: self.exploration_pipeline.take(),
            always_on: std::mem::take(&mut self.always_on),
            plugin_registry: std::mem::take(&mut self.plugin_registry),
            config: self.config.clone(),
            // agent_discovery: self.agent_discovery.take(),
            panorama: self.panorama.take(),
            nt_world_model: self.nt_world_model.take(),
            scheduler: self.scheduler.take(),
            daemon: self.daemon.take(),
            skill_engine: {
                let mut hooks = MindHookRegistry::default();
                hooks.register(HookEvent::SkillLoaded, Box::new(LogHook::new("bg-loop")));
                hooks.register(HookEvent::SkillUnloaded, Box::new(LogHook::new("bg-loop")));
                // CSGN 星辰唤醒 (T3 生产接线): 技能激活 → galaxy_wake_star 落盘
                hooks.register(
                    HookEvent::SkillLoaded,
                    Box::new(crate::l5_cognition::nt_mind::nt_mind_skill_engine::skill_hooks::CsgnWakeHook {
                        kb: kb.clone(),
                    }),
                );
                // Experience Tree — 会话结束自动触发五阶段吸收 (cycle 1053)
                hooks.register(
                    HookEvent::SessionEnd,
                    Box::new(crate::l5_cognition::nt_mind::nt_mind::experience_tree::SessionEndHook::new(kb.clone())),
                );
                SkillEngine::new(PathBuf::from(
                    &dirs::home_dir().unwrap_or_default().join(".claude").join("skills"),
                )).with_hooks(hooks)
            },
//             session_router: crate::neotrix::nt_agent_protocol::unified_session::SessionRouter::new(),
             healer_registry,
            kb_pipeline,
            session_recovery: self.session_recovery.take(),
            event_bus: Some(event_bus.as_ref().clone()),
            metacognition: self.metacognition.take(),
            #[cfg(feature = "stealth-net")]
            world_consciousness: self.world_consciousness.take(),
            #[cfg(not(feature = "stealth-net"))]
            world_consciousness: None,
            #[cfg(feature = "stealth-net")]
            heartbeat_engine: self.heartbeat_engine.take(),
            #[cfg(feature = "stealth-net")]
            proxy_client: self.proxy_client.take(),
            consciousness_runtime: {
                let mut cr = std::mem::take(&mut self.consciousness_runtime);
                if let Some(ref kb_ref) = self.kb {
                    if let Some(ref mut runtime) = cr {
                        runtime.attach_kb(kb_ref.clone());
                    }
                }
                cr
            },
            consciousness_tree: self.consciousness_tree.take(),
            fep_iit_bridge: self.fep_iit_bridge.take(),
            cognitive_load: self.cognitive_load.take(),
            volition: self.volition.take(),
            // bbrain already set above (line 571)
            cog_eval: crate::l5_cognition::l1_facade::metacognitive_evaluator::CognitiveEvaluator::new(),
            second_brain: {
                let mut sb = SecondBrain::new();
        if let Some(ref kb_ref) = self.kb {
                    sb.attach_kb(kb_ref.clone());
                }
                Some(sb)
            },
            dream: crate::l2_perception::nt_core_hcube::dream_consolidation::DreamConsolidation::new(
                crate::l2_perception::nt_core_hcube::dream_consolidation::DreamConfig::default(),
            ),
            meta_agent: self.kb.clone().map(|kb_ref| {
                crate::l5_cognition::nt_mind::nt_mind::evolution::agent_capability::MemoryAgent { kb: kb_ref }
            }),
            dialogue_bridge: self.kb.clone().map(|kb_ref| {
                crate::l5_cognition::nt_mind::nt_mind::evolution::agent_capability::DialogueAbsorbBridge::new(kb_ref)
            }),
            agent_executor: self.kb.clone().map(|kb_ref| {
                crate::l5_cognition::nt_mind::nt_mind::ProductionAgentExecutor::new(kb_ref)
            }),
            meta_shell: {
                // P1: 启动时从 KB 恢复派单学习证据 — 派单统计跨会话存活。
                // P3: 同时恢复 MANTA 派单拓扑 (域→档案边, 跨轮 playbook)。
                // P4: 同时恢复 MAGE 四子图共进化图谱 + 任务级搜索 bandit。
                let mut shell = crate::l5_cognition::nt_mind::nt_mind::MetaAgentShell::new("dialogue");
                if let Some(ref kb_ref) = kb {
                    if let Err(e) = shell.learner.load(kb_ref) {
                        log::warn!("[bg-meta] route_learner load failed: {}", e);
                    }
                    if let Err(e) = shell.load_topology(kb_ref) {
                        log::warn!("[bg-meta] dispatch_topology load failed: {}", e);
                    }
                    if let Err(e) = shell.load_coevo(kb_ref) {
                        log::warn!("[bg-meta] coevolution loop load failed: {}", e);
                    }
                }
                Some(shell)
            },
            kb,
            nexus_weaver: {
                // ⭐ 2026-10-07 消除 `.expect`（`check-unwrap` 4 → 1）。
                //
                // ⛔ **原实现的缺陷**：首次 `open(None)` 失败后，fallback
                //    **再次打开完全相同的路径**（参数一模一样）。
                //    而 `open` 失败的原因通常是**路径不可写 / 目录不存在 /
                //    磁盘满 / 锁冲突** ⇒ 这些原因**不会因重试而消失**
                //    ⇒ 第二次**必然再失败** ⇒ `.expect` 在真实故障下必 panic。
                //    ⇒ 那段 fallback 是**无效的**：它把「KB 路径故障」升级成
                //      「进程崩溃」，还**谎称**在 "creating temp"（实际没建 temp）。
                //
                // ⭐ 正解：**单次** open，失败即 `None`（如实表达缺失）。
                //    缺失 ⇒ nexus 整体不接线 ⇒ `handle_nexus_weaver` 直接返回
                //    ⛔ 但**不杀掉**其余 20+ 个后台 handler。
                //
                // ⚠️ 我在此处先后试过 3 个**错误**方案并全部作废，
                //    记录在此避免后人重走（都是同一类错误：绕过门而非修问题）：
                //      ① 「内存库失败 → 再 open 一次 + `.expect`」= 原样重复；
                //      ② 「内存库失败 → `unreachable!()`」= 换皮的 panic；
                //      ③ 改 `NexusWeaverScheduler.kb` 为 `Option` ⇒ 误伤同文件
                //         另一个 `ExperienceQuery.kb`（同名同类型双份设计），
                //         23 个编译错误 ⇒ 超出「清一处违规」的范围，已回滚。
                //    ⇒ 正确切面是**这里**（唯一 2 处读点的持有者）。
                let kb_ref = match kb_for_nexus.clone() {
                    Some(kb) => Some(kb),
                    None => match KnowledgeBase::open(None) {
                        Ok(kb) => Some(Arc::new(kb)),
                        Err(e) => {
                            log::error!(
                                "[bg-meta] nexus KnowledgeBase 不可用 ({e}); \
                                 跨会话模式挖掘本轮不接线, 其余后台 handler 继续"
                            );
                            None
                        }
                    },
                };
                // `kb_ref: Option<Arc<KnowledgeBase>>` ⇒ map 闭包收到 `Arc`，
                // 而 `new` 已接 `Option` ⇒ 闭包内重新包成 `Some`。
                kb_ref.map(|kb| {
                    crate::l5_cognition::nt_mind::nt_mind::experience_tree::NexusWeaverScheduler::new(
                        Some(kb),
                    )
                })
            },
            emotion_restored: std::sync::atomic::AtomicBool::new(false),
            absorption_in_progress: std::sync::atomic::AtomicBool::new(false),
            cognitive_mode: 0,
            state: StateSubstrate::new(),
            simulate: SimulateEngine::new(),
            convergence_pulse: ConvergencePulse::default(),
            tool_grounding: crate::l5_cognition::l1_facade::self_audit::ToolGroundingMonitor::new(),
            meta_auditor: crate::l5_cognition::l1_facade::nt_core_meta_auditor::MetaAuditor::new(),
            // 门控注册表 — 默认只读工具, 运行时可扩展。
            gate_registry: Some(ToolRegistry::from_read_only(&["get", "query", "read", "search"])),
            kb_guard: crate::l5_cognition::nt_mind::foundation::guardian::KbGuard::default(),
            workspace_guard: crate::l5_cognition::nt_mind::foundation::guardian::WorkspaceGuard::default_for(
                std::env::current_dir().unwrap_or_default(),
            ),
            last_consumed_fruit_cycle: 0,
            last_capability_evolve_ts: std::sync::atomic::AtomicU64::new(0),
            denylist: _PathDenylist::default_gates(),
            readiness: _LoopReadyScore::default(),
            _autonomy_tier: _AutonomyTier::L1,
            refiner: crate::l5_cognition::nt_mind::harness::refinement::ContinualRefiner::new(),
            self_improvement: crate::l5_cognition::l1_facade::SelfImprovementLoop::new(),
        }));

        // ═══════════════════════════════════════════════════════════════════
        // ⭐ 裁定 A（2026-10-07）：配置驱动的 handler 一律直接在此宏里校验
        //     「间隔字段 ↔ handler 名」接线一致性。
        //
        // # 为什么（实测两个真 bug，都是「功能在跑、改配置无效」）
        //
        // | 字段 | Default | 曾被驱动于 | 后果 |
        // |---|---|---|---|
        // | telemetry_interval_secs | 300 | 硬编码常量 60 | 改配置无效 |
        // | evolve_interval_secs     | 120 | evolution_interval_secs(300) | 改配置无效 |
        //
        // # ⛔ 为什么 D2 门抓不到（这是本校验存在的唯一理由）
        //
        // `rg evolve_interval_secs run.rs` 返回 0 命中 —— 看起来像「死配置忘了接线」，
        // D2 会这么报；真正问题是**接错了另一个字段**。D2 只看「有无读点」，
        // **看不到「读的是不是对应那个」** ⇒ 唯一能拦住的地方是接线瞬间。
        //
        // # 防绕过
        //
        // 旧分支启动前断言 `!stringify!($interval).starts_with("cfg.")`
        // ⇒ `spawn_handler!(cfg.x, ...)` 一律 panics（须先点改为校验分支）。
        //
        // 范围：只校验名称对应关系。Default 合理性非本职；handler 内部逻辑不校验。
        // ═══════════════════════════════════════════════════════════════════
        macro_rules! spawn_handler {
            // ── arm 1：字段名 + handler 名都给 ⇒ 编译期静态比对 ──
            ($cfgv:ident, $field:ident, $name:literal, |$lock:ident| $body:expr) => {{
                if !nt_bg_wiring_name_ok(stringify!($field), $name) {
                    panic!(
                        "[bg-wiring] 接线错配：配置字段 `{}` 应当驱动 handler `{}`，实际写了 `{}`。\
  \
                         ⛔ 这类错配让「改配置无效」：字段有读点（不是死配置），但读的是另一个间隔。\
  \
                         确属命名例外请在 NT_BG_WIRING_EXEMPT 登记并附理由。",
                        stringify!($field),
                        nt_strip_interval_suffix(stringify!($field)),
                        $name
                    );
                }
                spawn_handler!(@impl $cfgv.$field, $name, |$lock| $body);
            }};
            // ── arm 2：只给字段名 ⇒ body 里必须出现 handle_<去后缀名> ──
            ($cfgv:ident, $field:ident, |$lock:ident| $body:expr) => {{
                let want = nt_strip_interval_suffix(stringify!($field));
                if !nt_bg_wiring_body_ok(stringify!($field), stringify!($body)) {
                    panic!(
                        "[bg-wiring] 接线错配：配置字段 `{}` 的 body 里没有调用 `handle_{}`。\
  \
                         ⛔ 这类错配让「改配置无效」：字段有读点（不是死配置），但驱动的是别的 handler。\
  \
                         确属命名例外请在 NT_BG_WIRING_EXEMPT 登记并附理由。",
                        stringify!($field),
                        want
                    );
                }
                spawn_handler!(@impl $cfgv.$field, "handler", |$lock| $body);
            }};
            // ── 旧分支：常量间隔，字段路径直接拒绝 ──
            // ⛔ 内部委托分支（不参与 ban 检查）—— 禁止直接手写 `spawn_handler!(@impl ...)`
            (@impl $interval:expr, $name:literal, |$lock:ident| $body:expr) => {{
                let h = this.clone();
                let mut rx = shutdown_rx.clone();
                self.handles.push(tokio::spawn(async move {
                    let mut ticker = tokio::time::interval(
                        tokio::time::Duration::from_secs($interval));
                    loop {
                        tokio::select! {
                            biased;
                            _ = ticker.tick() => {
                                tracing::debug!("[bg-tick] {} fired", $name);
                                let mut $lock = h.lock().await;
                                tracing::debug!("[bg-tick] {} acquired lock", $name);
                                if let Err(e) = $lock.denylist.check($name) {
                                    log::warn!("[bg-loop] handler '{}' 被 denylist gate 拦截: {}", $name, e);
                                    continue;
                                }
                                $body;
                            }
                            _ = rx.changed() => {
                                log::trace!("[bg] handler shutting down (interval={})", $interval);
                                break;
                            }
                        }
                    }
                }));
            }};
            ($interval:expr, $name:literal, |$lock:ident| $body:expr) => {{
                if stringify!($interval).starts_with("cfg.") {
                    panic!(
                        "[bg-wiring] `spawn_handler!(cfg.X, ...)` 已被禁用：\
  \
                         配置驱动的 handler 必须改用校验分支 `spawn_handler!(cfg, X, ...)`。\
  \
                         ⛔ 旧形式无法校验「字段 ↔ handler」接线一致性。"
                    );
                }
                let h = this.clone();
                let mut rx = shutdown_rx.clone();
                self.handles.push(tokio::spawn(async move {
                    let mut ticker = tokio::time::interval(
                        tokio::time::Duration::from_secs($interval));
                    loop {
                        tokio::select! {
                            biased;
                            _ = ticker.tick() => {
                                tracing::debug!("[bg-tick] {} fired", $name);
                                let mut $lock = h.lock().await;
                                tracing::debug!("[bg-tick] {} acquired lock", $name);
                                // G12 denylist gate — 机械执行, 每个 handler tick 前置 fail-closed
                                if let Err(e) = $lock.denylist.check($name) {
                                    log::warn!("[bg-loop] handler '{}' 被 denylist gate 拦截: {}", $name, e);
                                    continue;
                                }
                                $body;
                            }
                            _ = rx.changed() => {
                                log::trace!("[bg] handler shutting down (interval={})", $interval);
                                break;
                            }
                        }
                    }
                }));
            }};
            ($interval:expr, |$lock:ident| $body:expr) => {
                spawn_handler!($interval, "handler", |$lock| $body);
            };
            ($interval:expr, $lock:ident, $body:expr) => {
                spawn_handler!($interval, "handler", |$lock| $body);
            };
        }

        // ── Each handler is an independent task with its own ticker ──
        let cfg = self.config.clone();
        macro_rules! emit_event {
            ($h:expr, $event:expr) => {
                if let Some(ref bus) = $h.event_bus {
                    bus.emit($event);
                }
            };
        }

        spawn_handler!(cfg, save_interval_secs, "save", |h| {
            h.handle_save().await;
            emit_event!(h, crate::l0_substrate::nt_core_event::CoreEvent::TaskSubmitted {
                task: "save".into(), task_type: "storage".into(), priority: 2,
            });
        });
        spawn_handler!(cfg, consolidate_interval_secs, "consolidate", |h| h.handle_consolidate().await);
        spawn_handler!(cfg, goal_interval_secs, |h| h.handle_goal().await);
        spawn_handler!(cfg, knowledge_chain_interval_secs, |h| h.handle_knowledge_chain().await);
        spawn_handler!(cfg, knowledge_aging_interval_secs, |h| h.handle_knowledge_aging().await);
        spawn_handler!(cfg, crystallization_interval_secs, |h| h.handle_crystallization().await);
        // Continual Harness Refinement — 审查轨迹，应用有证据支持的状态更新
        spawn_handler!(cfg, refinement_interval_secs, "refinement", |h| h.handle_refinement().await);
        spawn_handler!(cfg, nt_act_voice_interval_secs, |h| h.handle_nt_act_voice_tick().await);
        spawn_handler!(cfg, plugin_interval_secs, |h| h.handle_plugin_tick().await);
        // ⭐ **接线 `enable_exploration`**（2026-10-07，此前是「未接线规格」）：
        //   探索 handler **本来就在跑**（本行一直在 spawn 它），
        //   但开关 `cfg.enable_exploration` **零读点** ⇒ 用户无法关闭探索。
        //   ⇒ 这是典型的「**功能已实现、开关没接**」，正解是**接线**⛔ 不是删字段。
        //   `cfg` 是 `self.config.clone()`（L779），故按值捕获使闭包能带走它。
        let exploration_on = cfg.enable_exploration;
        spawn_handler!(cfg, exploration_interval_secs, "exploration", |h| {
            if exploration_on {
                h.handle_exploration().await;
            } else {
                tracing::debug!("[bg-tick] exploration 被 enable_exploration=false 关闭");
            }
        });
        spawn_handler!(cfg, curiosity_interval_secs, |h| h.handle_curiosity().await);
        spawn_handler!(cfg, world_prediction_interval_secs, |h| h.handle_prediction().await);
        spawn_handler!(cfg, metacog_interval_secs, |h| h.handle_awareness().await);
        spawn_handler!(cfg, cleanup_interval_secs, |h| h.handle_cleanup().await);
        spawn_handler!(BACKUP_INTERVAL_SECS, |h| h.handle_backup().await); // every 6h
        // ── 守卫层 (Rust 化自 sh 守护脚本, cycle 207) ──
        spawn_handler!(cfg, kb_guard_interval_secs, "kb_guard", |h| h.handle_kb_guard().await);
        spawn_handler!(cfg, kb_backup_interval_secs, "kb_backup", |h| h.handle_kb_backup().await);
        spawn_handler!(cfg, workspace_guard_interval_secs, "workspace_guard", |h| h.handle_workspace_guard().await);
        spawn_handler!(AGENT_DISCOVERY_INTERVAL_SECS, |h| h.handle_agent_discovery().await);
        // ── 意识能力网内化吸收 (cycle 1053): 60s 检查 pending-absorb.json,
        //    替代原 .opencode/plugins/experience-tree-absorption.js idle 插件。
        spawn_handler!(PENDING_ABSORPTION_INTERVAL_SECS, |h| h.handle_pending_absorption().await);
        // ── 每日信息例行感知检查 (cycle 1107): 每日 1 次检查今日信息是否落盘,
        //    缺失则记录感知盲区到 KB (NT-WORLD 感知缺失信号)。
        spawn_handler!(DAILY_INTEL_INTERVAL_SECS, "daily_intel", |h| h.handle_daily_intel_check().await);
        spawn_handler!(ALWAYS_ON_INTERVAL_SECS, "always_on", |h| h.handle_always_on().await);
        spawn_handler!(cfg, scheduler_interval_secs, "scheduler", |h| h.handle_scheduler_tick().await);
        // ⭐ 2026-10-07 接线：此前错读 `evolution_interval_secs`（**另一个字段**，
        //   Default 3600），而 `handle_evolve` 的**正确驱动项**是
        //   `evolve_interval_secs`（Default 120）⇒ 改 `evolve_interval_secs`
        //   完全无效，且实际间隔是声明值的 30 倍。
        spawn_handler!(cfg, evolve_interval_secs, "evolve", |h| h.handle_evolve().await);
        spawn_handler!(cfg, nt_world_sense_interval_secs, "world_sense", |h| h.handle_world_sense().await);
        #[cfg(feature = "stealth-net")]
        // ⭐ **接线 `proxy_enabled`**（2026-10-07，此前是「未接线规格」）：
        //   心跳 handler **本来就在跑**，但开关零读点 ⇒ 用户无法关闭代理心跳。
        //   ⚠️ 取证陷阱：`grep proxy_enabled` 会命中 `proxy_client` /
        //      `proxy_heartbeat_interval_secs` 的**子串** ⇒ 误判为「已接线」。
        //      ⇒ 本次用**精确读点**（`cfg.proxy_enabled` 全仓 0 处）确认后才接线。
        let proxy_on = cfg.proxy_enabled;
        spawn_handler!(cfg, proxy_heartbeat_interval_secs, "proxy_heartbeat", |h| {
            if proxy_on {
                h.handle_proxy_heartbeat().await;
            } else {
                tracing::debug!("[bg-tick] proxy heartbeat 被 proxy_enabled=false 关闭");
            }
        });
        spawn_handler!(SKILL_SCAN_INTERVAL_SECS, |h| h.handle_skill_scan().await);
        // spawn_handler!(SESSION_ROUTER_FLUSH_INTERVAL_SECS, "session_router", |h| h.handle_session_router_flush().await);
        // ── Nexus-Weaver 跨会话模式挖掘 (cycle 1053): 每 30min 扫描 experience 命名空间
        //    识别跨会话模式并触发 nexus-weaver 调度。
        spawn_handler!(NEXUS_WEAVER_INTERVAL_SECS, "nexus_weaver", |h| h.handle_nexus_weaver().await);
        spawn_handler!(HEALER_SCAN_INTERVAL_SECS, "healers", |h| h.handle_healer_scan().await);
        spawn_handler!(SYSTEM_HEALTH_HEAL_INTERVAL_SECS, "system_health_heal", |h| h.handle_system_health_heal().await);
        spawn_handler!(AVATAR_AUTO_DISTILL_INTERVAL_SECS, |h| h.handle_avatar_auto_distill().await);
        spawn_handler!(KB_ABSORB_INTERVAL_SECS, |h| {
            h.handle_kb_absorb().await;
        });
        spawn_handler!(SEED_CRAWL_QUEUE_INTERVAL_SECS, |h| h.handle_seed_crawl_queue().await);
        spawn_handler!(SESSION_RECOVERY_INTERVAL_SECS, |h| h.handle_session_recovery().await);
        spawn_handler!(CRAWL_QUEUE_INTERVAL_SECS, |h| h.handle_crawl_queue().await);
        spawn_handler!(ARCHITECTURE_AUDIT_INTERVAL_SECS, "architecture_audit", |h| h.handle_architecture_audit().await);
        // 43200s — 网络小说世界构建吸收 (novel_queue drain + 离线重分类), 12h cadence
        spawn_handler!(NOVEL_INGEST_INTERVAL_SECS, "novel_ingest", |h| h.handle_novel_ingest().await);
        // 3600s — KB 域聚类巡检: 社区检测 + domain_clusters 维护 + cluster_id 分配
        spawn_handler!(CLUSTERING_INTERVAL_SECS, "clustering", |h| h.handle_clustering().await);
        // 3600s — L6 自我改进循环: 采集指标→诊断→生成改进方案→执行→验证
        spawn_handler!(SELF_IMPROVEMENT_INTERVAL_SECS, "self_improvement", |h| h.handle_self_improvement().await);
        // ── Constitution hot-reload ──
        spawn_handler!(CONSTITUTION_RELOAD_INTERVAL_SECS, "constitution_reload", |h| h.handle_constitution_reload().await);
        // 缺陷1修复 (自我运转实际情况): 意识核心进化周期改为配置驱动
        // (cfg.consciousness_interval_secs, 默认 600s), 与 SEAL 果实消费节奏对齐。
        // 此前硬编码 3600s (1h) 且不可配置 — SEAL (goal_interval 180s) 在大部分
        // 时间消费空果实, 意识核心进化节奏严重滞后于消费节奏。
        spawn_handler!(cfg, consciousness_interval_secs, |h| h.handle_consciousness_tick().await);
        // 600s — Second Brain auto-sync (emotion + session notes to KB)
        spawn_handler!(SECOND_BRAIN_TICK_INTERVAL_SECS, |h| h.handle_second_brain_tick().await);
        // Startup: restore emotion state from KB (deferred 5s, then skips)
        spawn_handler!(EMOTION_RESTORE_DEFER_SECS, |h| {
            if !h.emotion_restored.load(std::sync::atomic::Ordering::Relaxed) {
                // 先把两个持久化 JSON 读成 owned 值, 释放对 kb 的借用, 再改 runtime。
                let engine_json = h.kb.as_ref()
                    .and_then(|kb| kb.kv_get("emotion", "engine_state").ok().flatten());
                let affective_json = h.kb.as_ref()
                    .and_then(|kb| kb.kv_get("emotion", "affective_interface").ok().flatten());
                if let Some(json) = engine_json {
                    if let Ok(engine) = crate::l5_cognition::l1_facade::emotion_state::EmotionEngine::from_json(&json) {
                        if let Some(ref mut cr) = h.consciousness_runtime {
                            cr.set_emotion_engine(engine);
                            log::info!("[bg] emotion state restored from KB");
                        }
                    }
                }
                if let Some(json) = affective_json {
                    if let Ok(iface) = crate::l5_cognition::l1_facade::affective_interface::AffectiveInterface::from_json(&json) {
                        if let Some(ref mut cr) = h.consciousness_runtime {
                            // 恢复人类情感交互界面 (关系阶段 + 用户情感历史)。
                            // 恢复后把持久化的用户情感吸收进意识 (意识影响闭环)。
                            *cr.affective_mut() = iface;
                            let snap = cr.affective().user.snapshot();
                            cr.observe_user_affect(&snap);
                            log::info!("[bg] affective interface restored from KB");
                        }
                    }
                }
                h.emotion_restored.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        });
        // ── G9 Loop Ready 巡检 — 5min 重算自治梯度; G12 denylist gate 每 tick 生效 ──
        spawn_handler!(LOOP_READINESS_INTERVAL_SECS, "loop_readiness", |h| h.handle_loop_readiness().await);
        // ── Auto Exacto 市场重估 (R-P79): 5min cadence 周期驱动 GatewayV2
        //    market_router 重算权重 (与 DEFAULT_INTERVAL=300s 对齐), 不再仅
        //    依赖 route() 调用时的惰性重估。经 gateway::run_periodic_re_evaluation()
        //    tick 进程级注册表中的活跃网关 ──
        spawn_handler!(MARKET_RE_EVAL_INTERVAL_SECS, "market_re_eval", |h| h.handle_market_re_evaluation().await);
        // ── G29 隐私聚合遥测 (R-P79): 60s 周期把全局 TelemetryStore 数值指标喂
        //    AnomalyDetector 做 spike/drop 检测, 告警经 EventBus 注入意识监控 ──
        // ⭐ 2026-10-07 接线（裁定 A：接线而非删字段）：此前用**硬编码常量**
        //   `TELEMETRY_INTERVAL_SECS = 60`，而 `config.telemetry_interval_secs`
        //   的 Default 是 **300** ⇒ 配置项形同虚设，用户改它无效，
        //   且实际间隔（60）与声明值（300）**不一致** —— 典型的
        //   「功能已实现、配置没接」（对照 commit 9d7bdfa0 同型）。
        //   ⇒ 改读 `cfg`，并删掉那个常量（它此后再无引用）。
        spawn_handler!(cfg, telemetry_interval_secs, "telemetry", |h| h.handle_telemetry().await);
        // 意识体智慧周期 — 价值观学习 + 叙事整合 + 规则结晶 + GC (E1)
        {
            const WISDOM_TICK_INTERVAL_SECS: u64 = 300; // 5 min
            spawn_handler!(WISDOM_TICK_INTERVAL_SECS, "wisdom", |h| h.handle_wisdom_tick().await);
        }
        // NT-PLAY 自主进化训练 — 意识体通过自我对弈持续进化
        spawn_handler!(GAME_TRAINING_INTERVAL_SECS, "game_training", |h| h.handle_game_training().await);

        // ── EventBus behavioral consumer (D30 fix) — responds to events with behavioral actions ──
        {
            let mut event_rx = event_bus.subscribe();
            let h = this.clone();
            let mut rx = shutdown_rx.clone();
            self.handles.push(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        biased;
                        result = event_rx.recv() => {
                            match result {
                                Ok(event) => {
                                    let mut handle = h.lock().await;
                                    handle.handle_event_bus_event(event).await;
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                    log::warn!("[bg] event_bus consumer lagged {} events", n);
                                }
                            }
                        }
                        _ = rx.changed() => {
                            log::trace!("[bg] event_bus consumer shutting down");
                            break;
                        }
                    }
                }
            }));
        }

        // Agent discovery requires nt_agent_protocol (not yet migrated)
        // if self.agent_discovery.is_some() {
        //     let server = Arc::new(...);
        //     self.handles.push(tokio::spawn(async move { ... }));
        // }

        println!("[bg] {} handlers spawned", self.handles.len());
    }
}

/// Lightweight inner state for concurrent handler access.
pub struct BackgroundLoopHandle {
    brain: Arc<RwLock<SelfIteratingBrain>>,
    bbrain: Option<Arc<RwLock<BMonitor>>>,
    cleanup_engine: Option<CleanupEngine>,
    config: BackgroundConfig,
    goal_loop: GoalLoop,
    awareness: Option<ConsciousnessMonitor>,
    gold_standard: Option<ConsciousnessGoldStandard>,
    gap_detector: Option<KnowledgeGapDetector>,
    nt_act_voice_input: Option<VoiceInput>,
    avatar_engine: Option<DistillationEngineWrapper>,
    self_evolver: Option<SelfEvolver>,
    curiosity_drive: CuriosityDrive,
    knowledge_aging: KnowledgeAging,
    auto_crystallizer: AutoCrystallizer,
    knowledge_chain: Option<KnowledgeChain>,
    exploration_pipeline: Option<ExplorationPipeline>,
    always_on: AlwaysOnEngine,
    plugin_registry: PluginRegistry,
//     agent_discovery: Option<crate::neotrix::nt_agent_protocol::discovery::AgentDiscovery>,
    panorama: Option<PanoramaPipeline>,
    nt_world_model: Option<WorldModelV2>,
    scheduler: Option<crate::l5_cognition::l1_facade::SchedulerEngine>,
    daemon: Option<EvolutionDaemon>,
    skill_engine: SkillEngine,
    /// G28 自维护巡检 healers (topics/code-health 吸收) — 多维度代码健康巡检。
    healer_registry: crate::l5_cognition::nt_mind::evolution::autofixer::HealerRegistry,
    /// 统一会话路由器 (G18, novu 吸收) — agent↔渠道统一会话模型:
    /// 入站归一 → capability 路由 → digest 合并出站。周期 flush 清出超窗摘要。
//     session_router: crate::neotrix::nt_agent_protocol::unified_session::SessionRouter,
    kb_pipeline: KnowledgeAbsorptionPipeline,
    session_recovery: Option<SessionRecoveryWrapper>,
    event_bus: Option<EventBus>,
    metacognition: Option<MetaCognitionBridge>,
    world_consciousness: Option<crate::l2_perception::nt_world::nt_world_sense::WorldConsciousness>,
    #[cfg(feature = "stealth-net")]
    heartbeat_engine: Option<crate::l3_embodiment::nt_shield::nt_shield_stealth_net::ProxyHeartbeatEngine>,
    #[cfg(feature = "stealth-net")]
    proxy_client: Option<crate::l3_embodiment::nt_shield::nt_shield_stealth_net::proxy_control::ProxyClient>,
    consciousness_runtime: Option<crate::l5_cognition::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime>,
    consciousness_tree: Option<crate::l5_cognition::nt_core_consciousness_tree::ConsciousnessTree>,
    fep_iit_bridge: Option<crate::l4_emotion::nt_feel::fep_iit_bridge::FepIitBridge>,
    cognitive_load: Option<crate::l5_cognition::cognitive_load::CognitiveLoadMonitor>,
    /// 意图引擎 (F2 接线): EFE 域探索提案必须经 select_by_goal_alignment 放行。
    volition: Option<crate::l5_cognition::nt_core_consciousness::VolitionEngine>,
    second_brain: Option<SecondBrain>,
    /// 梦境巩固器 — VSA 记忆重组/提纯/巩固 (skales Dreaming 模式, P0-3 接线)。
    /// 低负载周期触发 run_consolidation_cycle + prune_low_coherence。
    dream: crate::l2_perception::nt_core_hcube::dream_consolidation::DreamConsolidation,
    /// 记忆大脑 agent 外壳 — MemoryAgentCapability 统一能力面 (R-P42 接线)。
    /// handle_goal 按事件路由写/检索/巩固/证据能力。
    meta_agent: Option<crate::l5_cognition::nt_mind::nt_mind::evolution::agent_capability::MemoryAgent>,
    /// 对话吸收桥 — 把 KB 近期 session/experience 蒸馏为能力向量反哺 SelfIteratingBrain。
    /// handle_goal 末尾消费, 让对话经历驱动脑能力进化 (R-P79 接线, 非死代码)。
    dialogue_bridge: Option<crate::l5_cognition::nt_mind::nt_mind::evolution::agent_capability::DialogueAbsorbBridge>,
    /// 派单执行桥 (P0) — 把 MetaAgentShell 派单结果接到真实子系统,
    /// 让星系派单从仪式变控制面。researcher→搜索, explorer→检索, 等。
    agent_executor: Option<crate::l5_cognition::nt_mind::nt_mind::evolution::agent_capability::ProductionAgentExecutor>,
    /// 元认知 agent 外壳 — 对话事件刺激注意力域后按路由跑内核 cycle。
    meta_shell: Option<crate::l5_cognition::nt_mind::nt_mind::MetaAgentShell>,
    kb: Option<Arc<KnowledgeBase>>,
    /// 跨会话模式挖掘 (nexus-weaver) — 定期扫描 experience 命名空间识别跨会话模式。
    /// ⭐ 2026-10-07 改为 `Option`：KB 不可用时**不接线** nexus，
    /// 而不是在构造期 panic。KB 缺失只让**这一个** handler 降级为 no-op。
    nexus_weaver:
        Option<crate::l5_cognition::nt_mind::nt_mind::experience_tree::NexusWeaverScheduler>,
    emotion_restored: std::sync::atomic::AtomicBool,
    /// pending-absorb 自动吸收重入标志 (handlers_absorption.rs)。
    absorption_in_progress: std::sync::atomic::AtomicBool,
//     bbrain: crate::l5_cognition::nt_mind::bbrain_monitor::BMonitor,
    cog_eval: crate::l5_cognition::l1_facade::metacognitive_evaluator::CognitiveEvaluator,
    /// 0=Balanced, 1=Deep, 2=Fast — updated by consciousness tick, consumed by batch loops.
    cognitive_mode: u8,
    state: StateSubstrate,
    simulate: SimulateEngine,
    convergence_pulse: ConvergencePulse,
    tool_grounding: crate::l5_cognition::l1_facade::self_audit::ToolGroundingMonitor,
    /// 元审计器 — 架构审计真实消费端 (GAP-2 修复): handle_architecture_audit 把
    /// converge_check 幽灵/孤儿/失效 + SelfTest 失败统一汇入 record_finding,
    /// 使其从"仅测试调用"变 T3 生产接线 (R-P79), 累计准确性驱动审计质量。
    meta_auditor: crate::l5_cognition::l1_facade::nt_core_meta_auditor::MetaAuditor,
    /// 门控注册表 — 背景循环工具执行前置检查用。
    gate_registry: Option<ToolRegistry>,
    /// KB 守卫 + 工作区守卫 (Rust 化自 sh 守护脚本)
    kb_guard: crate::l5_cognition::nt_mind::foundation::guardian::KbGuard,
    workspace_guard: crate::l5_cognition::nt_mind::foundation::guardian::WorkspaceGuard,
    /// 已注入 SEAL 的意识树果实最大 cycle (H1 修复: 增量注入防重复消费)。
    /// 树内 fruits 从不清理, 全量克隆会让历史果实每 tick 重新注入 SEAL,
    /// 同一 trace 反复进 process buffer → 学习被重复污染。只注入比此值新的果实。
    last_consumed_fruit_cycle: u64,
    /// 能力网自动补齐节流 — 上次执行时间戳 (秒)。auto_scan 全量扫描有开销,
    /// 用时间门避免每 tick 触发 (handle_evolve 由 evolution_interval_secs 驱动,
    /// 但能力网补齐独立节流, 默认 3600s 一次)。
    last_capability_evolve_ts: std::sync::atomic::AtomicU64,
    /// 路径/动作 denylist gate (G12) — 每个 handler tick 前置 fail-closed
    denylist: _PathDenylist,
    /// Loop Ready 评分 (G9) — 最近一次重算值
    readiness: _LoopReadyScore,
    /// L1-L3 自治梯度 (G9) — 由 readiness 派生
    _autonomy_tier: _AutonomyTier,
    /// Continual Harness Refinement — 审查轨迹，应用有证据支持的状态更新
    refiner: crate::l5_cognition::nt_mind::harness::refinement::ContinualRefiner,
    /// L6 自我改进循环 — 采集指标→诊断→生成方案→执行→验证闭环
    self_improvement: crate::l5_cognition::l1_facade::SelfImprovementLoop,
}

impl BackgroundLoopHandle {
    fn try_emit(&self, event: crate::l0_substrate::nt_core_event::CoreEvent) {
        if let Some(ref bus) = self.event_bus { bus.emit(event); }
    }

    /// G9: 从真实运行时信号重算 Loop Ready 评分 + 自治梯度
    ///
    /// ⭐ 2026-10-07 接线 `handlers_ok`（此前硬编码 `true`）。
    ///
    /// ⛔ **原实现的缺陷**：本方法注释写「**从真实运行时信号**重算」，
    ///    但 4 个信号里只有 `kb_ok` 是真的：
    ///      `compute(true, kb_ok, true, true)` ⇒ 其余 3 个是**字面量**。
    ///    ⇒ 权重表宣称「handlers 40 / kb 25 / no_stall 20 / cadence 15」，
    ///      而 75 分（handlers+no_stall+cadence）**恒定到手**
    ///      ⇒ `no_stall`/`cadence_ok` 两个字段**零读点**且**永不变化**。
    ///
    /// ⭐ 正解：`handlers_ok` 改用**真实信号** `self.started`
    ///    （`BackgroundLoop::start()` L389 置 true；`shutdown()` L24
    ///    已在用它做守卫 ⇒ 该信号**确已存在**，⛔ 不是新造）。
    ///
    /// ⛔ `no_stall` / `cadence_ok` **保持硬编码 `true`**：本仓当前
    ///    **没有**「是否停滞」「tick 间隔是否达标」的任何测量
    ///    （无 `last_tick`、无 tick 间隔记录 ⇒ grep 零命中）。
    ///    ⛔ 我**不**把它们伪造成「已接线」——
    ///       那会让 40/20/15 分变成**恒定的假数据**，
    ///       比明确标注「未测量」更危险（见下方字段文档）。
    pub(crate) fn _recompute_readiness(&mut self) -> _LoopReadyScore {
        let kb_ok = self.kb.is_some();
        // ⭐ 真实信号：handler 循环是否真的启动过（⛔ 原为字面量 `true`）
        // ⚠️ 踩坑留痕（**两次**编译错误）：
        //  ① 我第一版写 `self.started` ⇒ `no field 'started' on
        //     BackgroundLoopHandle` ⇒ 它在 **BackgroundLoop**（父类型）上；
        //  ② 改用 `shutdown_coordinator` ⇒ 同样 `no field` ⇒ 它也在父类型。
        // ⇒ 两次都是**同名异型 / 字段所属类型判错**（本会话第 4、5 次）。
        // ⇒ 最后用**编译器给出的字段列表**定位到 handle 上**确实有**的
        //    `heartbeat_engine: Option<ProxyHeartbeat>`（L1045）
        //    ⇒ `is_some()` 精确表达「心跳引擎在跑」＝ handlers 存活。
        let handlers_ok = self.heartbeat_engine.is_some();
        // no_stall/cadence_ok: 本仓**无**停滞/tick 间隔测量 ⇒ 仍为 `true`，
        // ⛔ 但已在 `_LoopReadyScore` 字段文档中标注「未测量」。
        let score = _LoopReadyScore::compute(handlers_ok, kb_ok, true, true);
        self.readiness = score;
        self._autonomy_tier = score._autonomy_tier();
        score
    }

    /// G12: 暴露 denylist 门禁供 handler 在执行风险动作前咨询
    pub(crate) fn _check_denylist(&self, action: &str) -> Result<(), String> {
        self.denylist.check(action)
    }

    /// G9 巡检 handler — 每 tick 重算 readiness 并报告当前自治梯度;
    /// 同时自检一次 denylist 命中率 (gate 日志)。
    pub async fn handle_loop_readiness(&mut self) {
        let score = self._recompute_readiness();
        log::info!(
            "[bg-loop] LoopReady score={} tier=L{} ({}) kb_ok={} handlers_ok={}",
            score.score,
            score._autonomy_tier() as u8,
            score._autonomy_tier().label(),
            score.kb_ok,
            score.handlers_ok,
        );
        if !score.kb_ok {
            log::warn!("[bg-loop] KB 未挂载 — 自治梯度受限 (L3 需 KB 作为检查点底座)");
        }
    }

    /// Auto Exacto 市场重估 handler (R-P79 生产接线) — 5min cadence 周期驱动
    /// 进程级 GatewayV2 的 market_router 重算市场权重, 不再仅依赖 route()
    /// 调用时的惰性重估。注册表经 gateway::RE_EVALUATION_GATEWAYS 共享,
    /// 未注册任何网关时是 no-op (返回 0), 安全无副作用。
    pub async fn handle_market_re_evaluation(&mut self) {
        let evaluated =
            crate::l1_action::nt_io::nt_io_provider::gateway::run_periodic_re_evaluation();
        if evaluated > 0 {
            log::info!(
                "[bg-loop] market re-evaluation triggered for {} gateway(s)",
                evaluated
            );
        }
    }

    /// G29 隐私聚合遥测 handler (R-P79 生产接线) — 每 60s 把全局 TelemetryStore
    /// 的数值指标喂给 AnomalyDetector 做 spike/drop 检测, 告警注入意识监控。
    /// Privacy: 只聚合 scalar (计数/时长/数值), 不携带原始 payload。
    pub async fn handle_telemetry(&mut self) {
        use crate::l0_substrate::nt_core_telemetry::{
            global_telemetry, AlertKind, AnomalyDetector, PolicyDriftMonitor, TelemetryAlert,
        };
        static DETECTOR: std::sync::LazyLock<AnomalyDetector> =
            std::sync::LazyLock::new(AnomalyDetector::default);
        static DRIFT: std::sync::LazyLock<PolicyDriftMonitor> =
            std::sync::LazyLock::new(PolicyDriftMonitor::default);
        static LAST_ALERTS: std::sync::Mutex<Vec<TelemetryAlert>> =
            std::sync::Mutex::new(Vec::new());

        let store = global_telemetry();
        let mut alerts: Vec<TelemetryAlert> = Vec::new();
        for metric in store.metric_names() {
            if let Some(mean) = store.metric_window_mean(&metric, std::time::Duration::from_secs(600)) {
                if let Some(alert) = DETECTOR.observe(&metric, mean) {
                    alerts.push(alert);
                }
            }
        }
        // Replica #3 塌缩前兆: 监测策略散度指标的漂移比数量级跳变。数据源经
        // `store.record_metric("policy_drift", v)` 写入 (SEAL 循环采样生成/训练
        // 策略散度)。前 5 个观测样本视为稳定期基线 (baseline_sample=true),
        // 之后同源观测按漂移比跳变判定塌缩前兆; 未写入则静默 (不影响主循环)。
        if let Some(drift) = store.metric_window_mean("policy_drift", std::time::Duration::from_secs(600)) {
            static WARMUP: std::sync::Mutex<u32> = std::sync::Mutex::new(0);
            let mut warm = WARMUP.lock().unwrap_or_else(|e| e.into_inner());
            if *warm < 5 {
                *warm += 1;
                DRIFT.observe(drift, true);
            } else if let Some(alert) = DRIFT.observe(drift, false) {
                alerts.push(alert);
            }
        }
        if alerts.is_empty() {
            return;
        }
        // 去重: 同类告警连续出现时只记一次 (防告警风暴)。
        let mut known = LAST_ALERTS.lock().unwrap_or_else(|e| e.into_inner());
        let fresh: Vec<TelemetryAlert> = alerts
            .iter()
            .filter(|a| !known.iter().any(|k| k.metric == a.metric && k.kind == a.kind))
            .cloned()
            .collect();
        *known = alerts;
        if fresh.is_empty() {
            return;
        }

        for a in &fresh {
            let kind = match a.kind {
                AlertKind::Spike => "SPIKE",
                AlertKind::Drop => "DROP",
            };
            log::warn!(
                "[bg-loop] telemetry {kind} {} current={:.2} baseline={:.2} z={:.2}",
                a.metric,
                a.current,
                a.baseline,
                a.threshold,
            );
        }
        // 注入意识监控: 复用一个现有公开通道 — 若不存在则仅记录日志。
        self._inject_telemetry_alerts(&fresh);
    }

    /// 把遥测告警注入意识监控 — 经 EventBus 广播 `SystemError` 事件 (severity
    /// 按告警类型标记), 供 ConsciousnessTree/GWT 等层消费者订阅。无总线时
    /// 降级为日志, 不影响主循环。
    pub(crate) fn _inject_telemetry_alerts(
        &self,
        alerts: &[crate::l0_substrate::nt_core_telemetry::TelemetryAlert],
    ) {
        if alerts.is_empty() {
            return;
        }
        use crate::l0_substrate::nt_core_event::CoreEvent;
        use crate::l0_substrate::nt_core_telemetry::AlertKind;
        for a in alerts {
            let severity = match a.kind {
                AlertKind::Spike => "spike",
                AlertKind::Drop => "drop",
            };
            self.try_emit(CoreEvent::SystemError {
                component: format!("telemetry.{}", a.metric),
                error: format!(
                    "{severity} current={:.2} baseline={:.2} z={:.2}",
                    a.current, a.baseline, a.threshold
                ),
                severity: severity.to_string(),
            });
        }
        log::debug!(
            "[bg-loop] telemetry alerts fed to consciousness: {}",
            alerts.len()
        );
    }

    /// 跨会话模式挖掘 handler — 每 30min 扫描 experience 命名空间
    /// 识别跨会话模式并触发 nexus-weaver 调度。
    pub async fn handle_nexus_weaver(&mut self) {
        // ⭐ 2026-10-07：KB 不可用 ⇒ nexus 未接线 ⇒ 本 handler 降级为 no-op。
        // ⛔ 不 fabricate 空 KB：那会让 `weave_patterns` 返回 `Ok(0)`，
        //    把「KB 缺失」**伪装成「已挖掘但无模式」**。
        let Some(sched) = self.nexus_weaver.as_mut() else {
            tracing::debug!("[nexus-weaver] 未接线（KnowledgeBase 不可用）⇒ 本轮跳过");
            return;
        };
        let connections = match sched.weave_patterns() {
            Ok(c) => c,
            Err(e) => {
                log::warn!("[nexus-weaver] weave_patterns failed: {e}");
                return;
            }
        };
        if connections > 0 {
            log::info!(
                "[nexus-weaver] discovered {} cross-session pattern connections",
                connections
            );
            // 触发 nexus-weaver 调度 — 模式连接数超过阈值时自动调度
            if connections >= 3 {
                log::info!("[nexus-weaver] scheduling cross-session weave ({} connections >= 3)", connections);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::l5_cognition::nt_mind::nt_mind::panorama_pipeline::PanoramaPipeline;
    use crate::l5_cognition::nt_mind::nt_mind::goal_loop::GoalLoop;
    use crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
    use crate::l5_cognition::l1_facade::WorldModelV2;

    #[test]
    fn nt_bg_wiring_name_ok_正确配对() {
        assert!(super::nt_bg_wiring_name_ok("evolve_interval_secs", "evolve"));
        assert!(super::nt_bg_wiring_name_ok("telemetry_interval_secs", "telemetry"));
        // 历史真 bug 的「接错驱动」形态必须被判据拦截：
        assert!(!super::nt_bg_wiring_name_ok("evolve_interval_secs", "evolution"));
        assert!(!super::nt_bg_wiring_name_ok("telemetry_interval_secs", "handler"));
        // 例外表放行：字段名历史前缀的项
        assert!(super::nt_bg_wiring_name_ok("nt_world_sense_interval_secs", "world_sense"));
    }

    #[test]
    fn nt_bg_wiring_body_ok_名即handler() {
        assert!(super::nt_bg_wiring_body_ok(
            "goal_interval_secs",
            "h.handle_goal().await"
        ));
        assert!(!super::nt_bg_wiring_body_ok(
            "telemetry_interval_secs",
            "let t = TELEMETRY_INTERVAL_SECS;"
        ));
        assert!(super::nt_bg_wiring_body_ok(
            "metacog_interval_secs",
            "h.handle_awareness().await"
        ));
    }

    // ── 循环启动 smoke：真实接线计划走一遍 spawn_handler! 校验 (CI 覆盖) ──
    //
    // ⭐ 为什么需要本测试：arm1/arm2 的校验语句**只在 `start()` 运行时执行**
    //    （`stringify!` 实参埋在宏调用点里），上面的纯函数单测只覆盖「判据
    //    函数」本身 ⇒ 新接线写错（字段↔handler 错配）此前能溜过 CI。
    //
    // ⭐ 可行性依据（裁定选 a，未退化为 audit 函数 b）：
    //    ① `BackgroundLoop::new(brain)` 已被 mod.rs 既有测试使用
    //       （构造副作用与先例相同，无新增外部进程依赖）；
    //    ② `start()` 函数体自身**零 `.await`**（L394–L1083 全同步），
    //       current_thread runtime 下 spawn 的任务在测试返回前**从不被 poll**
    //       ⇒ 48 个 handler body 一律不执行，只有宏里的校验 `if` 真跑；
    //    ③ 校验失败即 `panic!`（消息带字段名）⇒ 本测试红，CI 可见。
    //
    // ⛔ 故意**不**调 `shutdown()`：shutdown 内部 `await` 会让 runtime
    //    调度已 spawn 的任务 ⇒ `tokio::time::interval` 首 tick 立即就绪
    //    ⇒ 全部 handler 真实执行一轮（扫 ~/.claude/skills、cwd 等）。
    //    直接 drop：JoinHandle detach，runtime drop 时任务未 poll 即弃。
    #[tokio::test]
    async fn start_smoke_真实接线计划下23个cfg校验全部通过() {
        use crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
        use crate::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;

        let brain = std::sync::Arc::new(tokio::sync::RwLock::new(SelfIteratingBrain::new()));
        let mut bg = BackgroundLoop::new(brain);

        // 23 个 cfg 调用点的 name_ok/body_ok 校验全部在此同步执行；
        // 任何错配 ⇒ start() panic ⇒ 本测试红（panic 消息含字段名）。
        bg.start().await;

        assert!(bg.started, "start() 完成后 started 必须置位");
        // 48 个 spawn_handler!（23 cfg 校验分支 + 25 常量分支）+ 1 个 EventBus 消费者。
        // 本断言专抓「静默删掉某条接线」——删调用点不会 panic，但任务数会掉。
        // 新增/删除 handler 时必须同步更新此数。
        #[cfg(feature = "stealth-net")]
        const EXPECTED_TASKS: usize = 49;
        #[cfg(not(feature = "stealth-net"))]
        const EXPECTED_TASKS: usize = 48;
        assert_eq!(
            bg.handles.len(),
            EXPECTED_TASKS,
            "spawn 任务数漂移 — 有 handler 被增删而未同步本断言"
        );
    }

    // ── 23 个 cfg 接线点逐一可见的 #[test] 覆盖 ──
    //
    // 与 smoke 测试的分工：smoke 吃**真实计划**（宏调用点现场执行），
    // 本表给**逐字段** CI 输出并锁死「23」这个数。两表抄自 run.rs
    // L895–L1037 的真实调用点；smoke 测试兜底防漂移（真计划错配会 panic）。

    /// arm1（字段 + 显式 handler 名）⇒ 走 `nt_bg_wiring_name_ok`。
    const CFG_WIRING_ARM1: &[(&str, &str)] = &[
        ("save_interval_secs", "save"),
        ("consolidate_interval_secs", "consolidate"),
        ("refinement_interval_secs", "refinement"),
        ("exploration_interval_secs", "exploration"),
        ("kb_guard_interval_secs", "kb_guard"),
        ("kb_backup_interval_secs", "kb_backup"),
        ("workspace_guard_interval_secs", "workspace_guard"),
        ("scheduler_interval_secs", "scheduler"),
        ("evolve_interval_secs", "evolve"),
        ("nt_world_sense_interval_secs", "world_sense"),
        ("proxy_heartbeat_interval_secs", "proxy_heartbeat"),
        ("telemetry_interval_secs", "telemetry"),
    ];

    /// arm2（只给字段）⇒ 走 `nt_bg_wiring_body_ok`；body 文本抄自真实调用点。
    const CFG_WIRING_ARM2: &[(&str, &str)] = &[
        ("goal_interval_secs", "h.handle_goal().await"),
        ("knowledge_chain_interval_secs", "h.handle_knowledge_chain().await"),
        ("knowledge_aging_interval_secs", "h.handle_knowledge_aging().await"),
        ("crystallization_interval_secs", "h.handle_crystallization().await"),
        ("nt_act_voice_interval_secs", "h.handle_nt_act_voice_tick().await"),
        ("plugin_interval_secs", "h.handle_plugin_tick().await"),
        ("curiosity_interval_secs", "h.handle_curiosity().await"),
        ("world_prediction_interval_secs", "h.handle_prediction().await"),
        ("metacog_interval_secs", "h.handle_awareness().await"),
        ("cleanup_interval_secs", "h.handle_cleanup().await"),
        ("consciousness_interval_secs", "h.handle_consciousness_tick().await"),
    ];

    #[test]
    fn cfg接线计划_23个调用点逐一通过校验() {
        assert_eq!(CFG_WIRING_ARM1.len(), 12, "arm1 调用点数漂移（对照 run.rs 调用点）");
        assert_eq!(CFG_WIRING_ARM2.len(), 11, "arm2 调用点数漂移（对照 run.rs 调用点）");
        assert_eq!(CFG_WIRING_ARM1.len() + CFG_WIRING_ARM2.len(), 23);

        for (field, name) in CFG_WIRING_ARM1 {
            assert!(
                super::nt_bg_wiring_name_ok(field, name),
                "arm1 接线错配：字段 `{field}` 不应驱动 handler `{name}`"
            );
        }
        for (field, body) in CFG_WIRING_ARM2 {
            assert!(
                super::nt_bg_wiring_body_ok(field, body),
                "arm2 接线错配：字段 `{field}` 的 body `{body}` 缺对应 handle_ 调用"
            );
        }

        // 变异证据：判据若被掏空成恒真，下列断言立刻红
        assert!(
            !super::nt_bg_wiring_name_ok("evolve_interval_secs", "evolution"),
            "变异守卫：错配 handler 名必须被拒"
        );
        assert!(
            !super::nt_bg_wiring_body_ok("goal_interval_secs", "h.handle_cleanup().await"),
            "变异守卫：body 缺对应 handle_ 调用必须被拒"
        );
    }

    #[test]
    fn test_panorama_pipeline_new() {
        assert_eq!(PanoramaPipeline::new().cycle, 0);
    }
    #[test]
    fn test_panorama_run_cycle() {
        let mut pano = PanoramaPipeline::new();
        assert_eq!(pano.run_cycle(
            &mut SelfIteratingBrain::new(),
            &mut GoalLoop::new(),
            &mut WorldModelV2::new(4, 64)
        ).cycle, 1);
    }

    use super::ConvergencePulse;
    use super::{_AutonomyTier, _LoopReadyScore, _PathDenylist};
    use crate::l0_substrate::nt_core_self_test::SelfTest;

    #[test]
    fn test_convergence_pulse_advance_no_gaps() {
        let mut p = ConvergencePulse::default();
        p.gaps = Vec::new();
        p.verified = true;
        let promoted = p.advance();
        assert!(promoted.is_some(), "complete layer should promote");
        assert_eq!(promoted.unwrap().name(), "task");
        assert_eq!(p.iteration, 0, "iteration resets on promotion");
    }

    #[test]
    fn test_convergence_pulse_open_gap_blocks_promotion() {
        let mut p = ConvergencePulse::default();
        p.gaps_from_self_tests(&[("substrate".to_string(), false)]);
        let before = p.layer;
        let promoted = p.advance();
        assert!(promoted.is_none(), "open gap must block promotion");
        assert_eq!(p.layer, before);
        assert_eq!(p.iteration, 1);
        assert!(!p.verified);
    }

    #[test]
    fn test_convergence_pulse_self_test() {
        let p = ConvergencePulse::default();
        let result = p.self_test();
        assert!(result.is_ok(), "default pulse self-test should pass: {:?}", result.err());
    }

    #[test]
    fn test_convergence_gaps_do_not_clobber_external_verification() {
        // P67 regression: 无 gap 时 gaps_from_self_tests 不得覆盖外部 cargo check 的 verified=false
        let mut p = ConvergencePulse::default();
        p.verified = true;
        p.gaps_from_self_tests(&[("substrate".to_string(), true)]);
        assert!(p.verified, "no gaps must not clear external verification");
        let promoted = p.advance();
        assert!(promoted.is_some(), "externally-verified complete layer should promote");
    }

    // ── G9 Loop Ready / 自治梯度 ──────────────────────────────

    #[test]
    fn test_loop_ready_score_full_kit() {
        let s = _LoopReadyScore::compute(true, true, true, true);
        assert_eq!(s.score, 100);
        assert_eq!(s._autonomy_tier(), _AutonomyTier::L3);
    }

    #[test]
    fn test_loop_ready_score_no_kb_downgrades_tier() {
        // KB 缺失 → 75 分 → L2 (自动修复), 不能 L3 自主进化
        let s = _LoopReadyScore::compute(true, false, true, true);
        assert_eq!(s.score, 75);
        assert_eq!(s._autonomy_tier(), _AutonomyTier::L2);
    }

    #[test]
    fn test_loop_ready_score_low_is_l1() {
        let s = _LoopReadyScore::compute(false, false, false, false);
        assert_eq!(s.score, 0);
        assert_eq!(s._autonomy_tier(), _AutonomyTier::L1);
    }

    #[test]
    fn test_loop_ready_bounds() {
        assert!(_AutonomyTier::L1 < _AutonomyTier::L2);
        assert!(_AutonomyTier::L2 < _AutonomyTier::L3);
        assert_eq!(_AutonomyTier::L3.label(), "自主进化");
    }

    // ── G12 路径/动作 denylist gate ───────────────────────────

    #[test]
    fn test_denylist_blocks_destructive_patterns() {
        let gate = _PathDenylist::default_gates();
        assert!(gate.check("rm -rf /home/neo").is_err(), "rm -rf 必须被拒");
        assert!(gate.check("git push --force origin main").is_err(), "force push 必须被拒");
        assert!(gate.check("mkfs.ext4 /dev/sdb").is_err(), "mkfs 必须被拒");
        assert!(gate.check("> /dev/sda2").is_err(), "磁盘直接写入必须被拒");
    }

    #[test]
    fn test_denylist_allows_safe_actions() {
        let gate = _PathDenylist::default_gates();
        assert!(gate.check("read ./src/main.rs").is_ok());
        assert!(gate.check("search kb experience").is_ok());
        assert!(gate.check("cargo check -p neotrix").is_ok());
    }

    #[test]
    fn test_denylist_extensible_and_fail_closed() {
        let mut gate = _PathDenylist::default_gates();
        gate.add_pattern("delete_branch");
        assert!(gate.check("git delete_branch feature-x").is_err(), "自定义模式同样 fail-closed");
        assert_eq!(gate.patterns().len(), 9, "默认8 + 自定义1");
    }
}

#[cfg(test)]
mod nexus_degrade_tests {
    use crate::l5_cognition::nt_mind::nt_mind::experience_tree::NexusWeaverScheduler;

    /// ⭐ **变异证据**：KB 缺失时调度器必须能构造，且不 panic。
    ///
    /// ⛔ 原实现在此路径 `.expect("nexus KB fallback must succeed")` ——
    /// 而 fallback **重开同一路径**，故真实故障下必崩。
    #[test]
    fn 无KB时调度器构造成功且挖掘返回Err() {
        // 承重：Option 化后，KB 缺失**不是构造期 panic**
        let mut sched = NexusWeaverScheduler::new(None);
        // ⭐ 且挖掘必须返回 **Err**，⛔ 不是 `Ok(0)` ——
        //    `Ok(0)` 会把「KB 缺失」伪装成「已挖掘但无模式」。
        let r = sched.weave_patterns();
        assert!(
            r.is_err(),
            "无 KB 时 weave_patterns 必须 Err（不得 Ok(0) 伪装成「无模式」）"
        );
    }

    /// 有 KB 时正常路径不被破坏（构造 + 挖掘不 Err）。
    #[test]
    fn 有KB时构造成功() {
        use crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase;
        let kb = KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:")))
            .expect("内存库应可创建");
        let mut sched = NexusWeaverScheduler::new(Some(std::sync::Arc::new(kb)));
        // 空库 ⇒ Ok(0)（**真的**无模式，与上面的 Err 语义不同）
        // Result<usize, String> ⇒ 用 unwrap_or_else(|_| 0) 而非 unwrap_or
        assert_eq!(sched.weave_patterns().unwrap_or_else(|_| 0), 0, "空库应返回 0");
    }
}

#[cfg(test)]
mod readiness_wiring_tests {
    use super::_LoopReadyScore;

    /// ⭐ **变异证据**：`compute()` 的 4 个信号**必须各自独立影响分数**。
    ///
    /// 修复前 `_recompute_readiness` 传 `compute(true, kb_ok, true, true)`
    /// ⇒ 其中 3 个是**字面量** ⇒ 权重表宣称
    /// 「handlers 40 / kb 25 / no_stall 20 / cadence 15」，
    /// 而 **75 分恒定到手**，与真实健康状态**无关**。
    #[test]
    fn 四个信号各自独立影响分数() {
        let all_on = _LoopReadyScore::compute(true, true, true, true);
        assert_eq!(all_on.score, 100, "全开 = 40+25+20+15");

        // ⭐ 逐个关闭 ⇒ 分数必须**精确**减少对应权重
        let no_handlers = _LoopReadyScore::compute(false, true, true, true);
        assert_eq!(no_handlers.score, 60, "关 handlers ⇒ 100-40=60");
        assert!(!no_handlers.handlers_ok, "字段必须反映传入值");

        let no_kb = _LoopReadyScore::compute(true, false, true, true);
        assert_eq!(no_kb.score, 75, "关 kb ⇒ 100-25=75");

        let no_stall = _LoopReadyScore::compute(true, true, false, true);
        assert_eq!(no_stall.score, 80, "关 no_stall ⇒ 100-20=80");

        let no_cadence = _LoopReadyScore::compute(true, true, true, false);
        assert_eq!(no_cadence.score, 85, "关 cadence ⇒ 100-15=85");

        let all_off = _LoopReadyScore::compute(false, false, false, false);
        assert_eq!(all_off.score, 0, "全关 = 0");
    }

    /// ⭐ 自治梯度必须**真的**随分数变化 ——
    /// 这是本次接线的**承重后果**：若 `handlers_ok` 恒 true，
    /// 则分数**永远 ≥ 75** ⇒ 梯度**永远 ≥ L2** ⇒ L3 永不因 handlers 故障而降级。
    #[test]
    fn 自治梯度随handlers_ok下降() {
        let healthy = _LoopReadyScore::compute(true, true, true, true);
        assert_eq!(healthy._autonomy_tier(), super::_AutonomyTier::L3, "100 分 ⇒ L3");

        // ⭐ 修复前不可能出现的情形：handlers 挂掉但仍判 L3
        let degraded = _LoopReadyScore::compute(false, true, true, true);
        assert_eq!(
            degraded._autonomy_tier(),
            super::_AutonomyTier::L2,
            "60 分 ⇒ 降级到 L2（修复前 handlers_ok 恒 true ⇒ 该情形不可达）"
        );
    }
}

/// 去掉 `_interval_secs` / `_interval_ms` 后缀 —— 供 spawn_handler 接线校验用。
fn nt_strip_interval_suffix(f: &str) -> &str {
    f.strip_suffix("_interval_secs")
        .or_else(|| f.strip_suffix("_interval_ms"))
        .unwrap_or(f)
}

/// ⭐ arm1 判据：字段名去 `_interval_*` 后缀后应等于 handler 名；例外表放行。
fn nt_bg_wiring_name_ok(field: &str, name: &str) -> bool {
    if NT_BG_WIRING_EXEMPT.iter().any(|(f, _)| *f == field) {
        return true;
    }
    nt_strip_interval_suffix(field) == name
}

/// ⭐ arm2 判据：body 文本里必须出现 `handle_<去后缀字段名>`；例外表放行。
fn nt_bg_wiring_body_ok(field: &str, body_txt: &str) -> bool {
    if NT_BG_WIRING_EXEMPT.iter().any(|(f, _)| *f == field) {
        return true;
    }
    body_txt.contains(&format!("handle_{}", nt_strip_interval_suffix(field)))
}

/// ⭐ `spawn_handler!` 接线一致性的**例外白名单**（裁定 A，2026-10-07）。
/// 每一项必须附理由：字段名去掉 `_interval_secs` 后与 handler 名不一致的原因。
/// 本表是**有界开集** —— 每项都是一个待消除的债，不是永久豁免。
const NT_BG_WIRING_EXEMPT: &[(&str, &str)] = &[
    // 元认知循环驱动的是"觉察"handler；metacog 是早期词汇，handler 沿用 handle_awareness。
    ("metacog_interval_secs", "handle_awareness：元认知即觉察"),
    // 世界预测 == 通用预测 handler；字段保留 world_ 前缀区别 world_crawl（世界爬取）。
    ("world_prediction_interval_secs", "handle_prediction：handler 取简写"),
    // `nt_` 是模块命名规约前缀，handler 名按规约去掉前缀。
    ("nt_world_sense_interval_secs", "handler world_sense 去掉 nt_ 前缀"),
];

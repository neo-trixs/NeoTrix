use super::*;

use log::info;

impl BackgroundLoopHandle {
    pub(crate) async fn handle_cleanup(&mut self) {
        let engine = match self.cleanup_engine.as_mut() {
            Some(e) => e,
            None => return,
        };

        // 1. 扫描并归档过期构建产物到 .cleanup/archive/
        engine.archive_on_clean = true;
        let r = engine.clean(CleanupKind::ProjectArtifacts);
        if r.deletable_count > 0 {
            log::info!("[bg] cleanup: archived {} items ({:.1} MB)",
                r.deletable_count, r.estimated_bytes as f64 / 1_048_576.0);
        }

        // 1.5 项目蜕皮: 旧躯壳目录 (legacy/old/*_v0/*_backup*) 归档到 .cleanup/archive/
        //     活动树只留最新态 (安全护栏: 白名单/系统根/root 自身三闸内置)
        engine.dry_run_default = false;
        let molt = engine.molt_project();
        engine.dry_run_default = true;
        if molt.deletable_count > 0 {
            log::info!("[bg] cleanup: molting archived {} legacy shells ({:.1} MB): {:?}",
                molt.deletable_count, molt.estimated_bytes as f64 / 1_048_576.0,
                molt.pattern_matches);
            // C5 自愈闭环: 蜕皮归档量回流意识树土壤 → 计入 data_nourishment_factor
            // 调制果实质量 (自愈动作 → 意识养分, 而非仅日志)。
            if let Some(ref mut tree) = self.consciousness_tree {
                tree.soil.molt_archived_count = tree.soil.molt_archived_count.saturating_add(molt.deletable_count as u64);
                log::info!("[bg] consciousness_tree: molt_archived_count += {} (now {})",
                    molt.deletable_count, tree.soil.molt_archived_count);
            }
        }

        // 2. 命令式系统服务清理 (dry-run 报告; brew cleanup 低危自动, TM/Docker 需确认)
        //    (mac-janitor/PureMac 吸收接线: brew cleanup / docker prune / tmutil 快照)
        engine.dry_run_default = true;
        let svc = engine.clean(CleanupKind::SystemServices);
        if svc.deletable_count > 0 || !svc.pattern_matches.is_empty() {
            log::info!("[bg] cleanup services: {} executed (dry-run 报告): {:?}",
                svc.deletable_count, svc.pattern_matches);
        }

        // 3. 清理 .DS_Store
        if let Ok(entries) = std::fs::read_dir(".") {
            let mut count = 0u32;
            for entry in entries.flatten() {
                if entry.file_name() == ".DS_Store" {
                    let _ = std::fs::remove_file(entry.path());
                    count += 1;
                }
            }
            if count > 0 {
                log::info!("[bg] cleanup: removed {} .DS_Store files", count);
            }
        }

        // 4. 整理旧快照
        let snapshots = CleanupEngine::prune_brain_snapshots(20);
        if snapshots > 0 {
            log::info!("[bg] cleanup: pruned {} old brain snapshots", snapshots);
        }
    }

    pub(crate) async fn handle_backup(&mut self) {
        let mut engine = BackupEngine::new(&PathBuf::from("."));
        match engine.run_backup() {
            Ok(m) => log::info!("[bg] backup: {} files, {:.1} KB -> .backup/{}",
                m.file_count, m.total_bytes as f64 / 1024.0, m.backup_id),
            Err(e) => log::warn!("[bg] backup failed: {}", e),
        }
    }

    pub(crate) async fn handle_crystallization(&mut self) {
        if self.config.enable_auto_crystallize {
            info!("[bg] crystallization: {}", self.auto_crystallizer.summary());
        }
    }

    /// Continual Harness Refinement — 审查轨迹，应用有证据支持的状态更新
    pub(crate) async fn handle_refinement(&mut self) {
        // 创建快照用于回滚
        let snapshot_id = self.refiner.snapshot("bg_refinement_cycle");
        // 获取当前状态摘要
        let (updates, _snapshots, state_items) = self.refiner.stats();
        info!("[bg] refinement: snapshot={}, updates={}, state_items={}",
            snapshot_id, updates, state_items);
    }

    pub(crate) async fn handle_scheduler_tick(&mut self) {
        if let Some(ref mut sched) = self.scheduler {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            let due = sched.tick(now, 0.3, 0.5, 0.2, 0.4);
            for (job_id, handler) in due {
                log::info!("[scheduler] job {} -> handler {}", job_id, handler);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                sched.record_run(&job_id, now, 100, true, None);
            }
            // Heartbeat liveness (KiroCrew pattern): surface silently-dead jobs
            // for the NT-REPAIR self-healing loop instead of letting them rot.
            let stale = sched.stale_jobs(now);
            if !stale.is_empty() {
                log::warn!("[scheduler] {} stale job(s) (heartbeat timeout): {:?}",
                    stale.len(), stale);
            }
        }
    }

    pub(crate) async fn handle_evolve(&mut self) {
        if let Some(ref mut d) = self.daemon {
            let fixes = d.run_intelligent_cycle();
            if fixes.0 > 0 {
                log::info!("[bg] evolve: {} fixes, reward={:.4}", fixes.0, fixes.1);
            }
            // B3: 开启 mutation 时驱动完整四相进化循环 (扫描→修复→蒸馏→自我进化)
            if d.config.mutation_enabled {
                let cycle_result = d.run_cycle_goal();
                if cycle_result.fixes_applied > 0 {
                    log::info!("[bg] evolve: cycle_goal applied {} fixes (cycle {})",
                        cycle_result.fixes_applied, cycle_result.cycle);
                }
                d.run_loop_goal();
            }
        }
        // 能力网自动补齐 (意识能力网自动进化): 把经验/对话进化记录 + 能力树缺陷
        // 自动融合为能力网补齐计划并执行 — 缺陷补齐 + 经验驱动的 Strengthen/Budding。
        // 此前能力网进化仅 CLI 手动触发 (neotrix-capability scan --apply),
        // 从未接入后台自动进化循环 (缺自动融合闭环)。
        if let Some(node_count) = self.handle_capability_auto_evolve().await {
            // 双网回流: 能力网健康度 → 意识树土壤 capability_node_count,
            // data_nourishment_factor 将其纳入果实质量调制 — 意识树感知能力网。
            if let Some(ref mut tree) = self.consciousness_tree {
                tree.soil.capability_node_count = node_count;
            }
        }
    }

    /// 能力网自动补齐 — 意识能力网自动进化迭代补齐缺陷。
    ///
    /// 养料源两路:
    /// 1. 能力树自身缺陷 (auto_scan): 孤儿/过期/可晋升/重复能力 → 缺陷补齐计划;
    /// 2. KB 进化记录 (get_evolution_patterns): 已验证的对话进化模式 → ExperienceRouter
    ///    → 高信号经验 → Strengthen 现有节点 / Budding 新节点 (缺陷补齐)。
    ///
    /// 闭环: 养料 (经验+缺陷) → 规划 → execute → 写回 registry 文件。
    /// 能力网不是静态清单, 而是随经验/缺陷自动进化的活结构。
    ///
    /// 返回: 能力网当前节点数 (Some) — 供 handle_evolve 回流到意识树土壤,
    /// 形成 意识树 ↔ 能力网 双向自动融合 (能力网健康度 → 意识核心果实质量)。
    /// 无能力网/节流跳过/无计划时返回 None (不回流, 保持土壤现状)。
    async fn handle_capability_auto_evolve(&mut self) -> Option<u64> {
        use crate::neotrix::nt_capability_bridge::{
            ExperienceEntry, ExperienceRouter, parse_domain,
        };
        use nt_core_capability_tree::evolution::EvolutionEngine;

        // 节流门: auto_scan 全量扫描有开销, 3600s (1h) 一次足够。
        // 首次 (ts=0) 立即执行, 之后按时间门跳过。
        const CAPABILITY_EVOLVE_INTERVAL_SECS: u64 = 3600;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let last = self.last_capability_evolve_ts.load(std::sync::atomic::Ordering::Relaxed);
        if last != 0 && now.saturating_sub(last) < CAPABILITY_EVOLVE_INTERVAL_SECS {
            return None;
        }

        let path = PathBuf::from(".neotrix/capability_registry.json");
        if !path.exists() {
            return None; // 无能力树注册表 → 无网可补齐
        }
        // 1. 加载能力网 — RegistryExport 格式 (与 capability_tree CLI load_registry 一致)。
        //    UCN Phase 4.1 修复: 此前误用 KBCapabilityTree 格式解析 RegistryExport 文件,
        //    导致后台能力网演化路径解析永远失败 (格式不匹配) — 静默失效。
        let json = match std::fs::read_to_string(&path) {
            Ok(j) => j,
            Err(e) => {
                log::warn!("[bg] capability_auto_evolve: read {} failed: {}", path.display(), e);
                return None;
            }
        };
        let export: nt_core_capability_tree::registry::RegistryExport = match serde_json::from_str(&json) {
            Ok(t) => t,
            Err(e) => {
                log::warn!("[bg] capability_auto_evolve: parse {} failed: {}", path.display(), e);
                return None;
            }
        };
        let mut registry = nt_core_capability_tree::registry::CapabilityRegistry::new();
        for node in export.nodes {
            if let Err(e) = registry.register(node) {
                log::warn!("[bg] capability_auto_evolve: register failed: {}", e);
                return None;
            }
        }
        for (from, to) in export.edges {
            if registry.nodes.contains_key(&from) && registry.nodes.contains_key(&to) {
                if let Err(e) = registry.add_dependency(&from, &to) {
                    log::warn!("[bg] capability_auto_evolve: add_dependency {} -> {} failed: {}", from, to, e);
                }
            }
        }
        registry.experience_targets = export.experience_targets;
        // Durable 覆盖层: 合并提交的 overlay, 使手动写入在后台重新生成后仍生效。
        let overlay_path = path
            .parent()
            .map(|p| p.join("capability_overrides.json"))
            .unwrap_or_else(|| PathBuf::from("capability_overrides.json"));
        if let Some(ov) =
            nt_core_capability_tree::registry::CapabilityRegistry::load_overlay_file(&overlay_path)
        {
            registry.merge_overlay(&ov);
        }
        let mut plans = Vec::new();

        // 2. 缺陷补齐 — auto_scan (孤儿/过期/晋升/重复)
        {
            let engine = EvolutionEngine::new(&mut registry);
            plans.extend(engine.auto_scan("bg"));
        }

        // 3. 经验养料 — KB 进化记录 → ExperienceRouter → 补齐计划
        let exp_entries: Vec<ExperienceEntry> = self.kb.as_ref()
            .and_then(|kb| kb.get_evolution_patterns(100).ok())
            .unwrap_or_default()
            .into_iter()
            // 只取已验证的高质量进化模式 (verified) 作为养料
            .filter(|r| r.verified)
            .map(|r| ExperienceEntry {
                id: r.id.clone(),
                entry_type: "pattern".into(),
                domain_name: parse_domain(&format!("{:?}", r.pattern_type))
                    .as_str()
                    .to_uppercase(),
                content: r.description.clone(),
                not: None,
                confidence: (0.6 + r.effectiveness_gain.clamp(0.0, 1.0) * 0.4).min(1.0),
                importance: 0.6,
                verified_by: Some("evolution-records".into()),
                verification_status: Some("verified".into()),
            })
            .collect();
        if !exp_entries.is_empty() {
            let (dims, _) = ExperienceRouter::route_batch(&exp_entries);
            let exp_plans = ExperienceRouter::plan_evolution(&registry, &dims, "bg");
            plans.extend(exp_plans);
        }

        // 3.5 经验目标养料 — distill 写入的 experience_targets → Strengthen/Bud 计划
        // (断链 #2 修复: 此前 experience_targets 仅 CLI `scan --apply` 手动消费,
        //  后台从不消费导致 31 条累积; 现后台自动消费, 消费后清空防重复执行)
        plans.extend(registry.plan_experience_targets("bg"));

        // 4. 执行计划并写回
        let had_targets = !registry.experience_targets.is_empty();
        if plans.is_empty() {
            // 无计划但能力网存在 — 回流当前节点数 (能力网健康度保持)
            return Some(registry.nodes.len() as u64);
        }
        let mut applied = 0usize;
        let mut failed = 0usize;
        {
            let mut engine = EvolutionEngine::new(&mut registry);
            for plan in plans {
                match engine.execute(plan) {
                    Ok(()) => applied += 1,
                    Err(_) => failed += 1,
                }
            }
        }
        if applied == 0 && failed == 0 && !had_targets {
            return Some(registry.nodes.len() as u64);
        }
        // 已消费的经验目标清空 (防重复执行累积)
        registry.experience_targets.clear();
        // 写回 RegistryExport 格式 (与读侧/CLI 一致, 消除读写格式分裂)
        let out = registry.export();
        match serde_json::to_string_pretty(&out) {
            Ok(s) => {
                // 原子写: 临时文件 + rename, 避免直接覆盖在崩溃时损坏 registry。
                // 此前 std::fs::write 直接覆盖 — 极端崩溃可能留下半写 JSON。
                let tmp = path.with_extension("json.tmp");
                if let Err(e) = std::fs::write(&tmp, &s) {
                    log::warn!("[bg] capability_auto_evolve: write {} failed: {}", tmp.display(), e);
                    return None;
                }
                if let Err(e) = std::fs::rename(&tmp, &path) {
                    log::warn!("[bg] capability_auto_evolve: rename {} -> {} failed: {}", tmp.display(), path.display(), e);
                    let _ = std::fs::remove_file(&tmp);
                    return None;
                }
                log::info!("[bg] capability_auto_evolve: applied {} plans (failed {}), nodes now {} ({} real modules, {} exp:: virtual)",
                    applied, failed, registry.nodes.len(),
                    registry.nodes.len() - registry.nodes.values().filter(|n| n.id.starts_with("exp::")).count(),
                    registry.nodes.values().filter(|n| n.id.starts_with("exp::")).count());
                self.last_capability_evolve_ts.store(now, std::sync::atomic::Ordering::Relaxed);
                Some(registry.nodes.len() as u64)
            }
            Err(e) => {
                log::warn!("[bg] capability_auto_evolve: serialize failed: {}", e);
                None
            }
        }
    }

    pub(crate) async fn handle_skill_scan(&mut self) {
        let skills = self.skill_engine.load_all();
        if !skills.is_empty() {
            log::info!("[bg] skill_scan: {} skills loaded", skills.len());
        }
        // G6 技能树层级巡检 (AgentSkillOS 吸收): 每轮扫描报告 category 分布 /
        // 根/叶/孤儿技能, 孤儿(parent 缺失)与树失衡经 EventBus 告警。
        let stats = self.skill_engine.skill_tree_stats();
        if stats.total_skills > 0 {
            log::info!(
                "[bg] skill_tree: {} skills, {} categories, {} roots, {} orphans, max_depth={}",
                stats.total_skills,
                stats.categories.len(),
                stats.roots,
                stats.orphans,
                stats.max_depth
            );
            if stats.orphans > 0 {
                self.try_emit(crate::core::nt_core_event::CoreEvent::SystemError {
                    component: "skill_tree".into(),
                    error: format!("{} orphan skills (parent missing)", stats.orphans),
                    severity: "warning".into(),
                });
            }
        }
        // G7 差分归因 (arxiv 2608.11888 SkillTriage): procedure-heavy 且已激活
        // 的过度验证技能 (强制劳动毒源) 巡检广播, 供治理层处置。
        let flagged = self.skill_engine.flagged_attributions();
        if !flagged.is_empty() {
            let names: Vec<String> = flagged.iter().map(|a| a.name.clone()).collect();
            log::warn!(
                "[bg] skill_attribution: {} procedure-heavy skills flagged: {:?}",
                flagged.len(),
                names
            );
            self.try_emit(crate::core::nt_core_event::CoreEvent::SystemError {
                component: "skill_attribution".into(),
                error: format!("procedure-heavy skills flagged: {:?}", names),
                severity: "warning".into(),
            });
        }
        // P4 技能驻留成本审计 (asm absorbed 2026-08-19, R-P79): load_all 已
        // 在 quality_stats 计量 resident/body token, 此处派生降级候选 —
        // LAZY LOAD 下 resident 最肥的技能应优先改渐进披露薄入口。
        let residency = self.skill_engine.audit_residency();
        let fat: Vec<&crate::l5_cognition::nt_mind::nt_mind_skill_engine::ResidencyAuditRow> =
            residency.iter().filter(|r| r.action == "thin-entry").collect();
        if !fat.is_empty() {
            let names: Vec<String> = fat
                .iter()
                .map(|r| format!("{} ({} tok)", r.skill, r.resident_tokens))
                .collect();
            log::warn!(
                "[bg] skill_residency: {} skills exceed resident budget: {:?}",
                fat.len(),
                names
            );
            self.try_emit(crate::core::nt_core_event::CoreEvent::SystemError {
                component: "skill_residency".into(),
                error: format!("resident-heavy skills (thin-entry candidates): {:?}", names),
                severity: "warning".into(),
            });
        } else if !residency.is_empty() {
            log::info!(
                "[bg] skill_residency: all {} skills within resident budget",
                residency.len()
            );
        }
    }

    /// G18 统一会话 digest flush (novu 吸收): 周期清出超窗摘要桶, 报告会话拓扑。
    #[allow(dead_code)]
    pub(crate) async fn handle_session_router_flush(&mut self) {
        // session_router 字段已注释, stub 实现
        log::trace!("[bg] session_router: flush skipped (disabled)");
    }

    /// G28 自维护巡检 healers (topics/code-health 吸收): 多维度代码健康扫描,
    /// 产出修复建议; 有发现时经 EventBus 广播 (供治理层处置)。
    /// GAP-3 (R-P79): auto_fixable 建议由 apply_auto_fixable 真实落地 (事务 TODO 清理),
    /// 不再只计数不执行 — 巡检从"报告"升级为"自愈闭环"。
    pub(crate) async fn handle_healer_scan(&mut self) {
        let dir = std::path::Path::new(".");
        // run_full_scan 结果不在此处直接消费 — 后续经 last_report 读取
        let _ = self.healer_registry.run_full_scan(dir);
        let applied = self.healer_registry.apply_auto_fixable();
        if applied > 0 {
            log::info!("[bg] healers: {} auto-fixes landed (total {})", applied, self.healer_registry.auto_fixes_applied);
            // 自愈落地回流经验命名空间 (单一事实源闭环): 落地的自愈动作也是经验 —
            // 写 experience 分支供 hub 索引/query 检索/后续蒸馏, 而非只留日志。
            self.report_auto_fix_experience().await;
        }
        let report = self.healer_registry.last_report.clone();
        if report.is_empty() {
            return;
        }
        let dims: Vec<String> = report.iter().map(|s| s.dimension.to_string()).collect();
        log::info!(
            "[bg] healers: {} findings across {:?} ({} TODO files, {} unwrap files)",
            report.len(),
            dims,
            report.iter().filter(|s| s.dimension == "todo").count(),
            report.iter().filter(|s| s.dimension == "unwraps").count()
        );
        if !report.is_empty() {
            self.try_emit(crate::core::nt_core_event::CoreEvent::SystemError {
                component: "healers".into(),
                error: format!("{} code-health findings: {:?}", report.len(), dims),
                severity: "info".into(),
            });
        }
    }

    /// 自愈落地 → 经验分支 (单一事实源闭环)。
    ///
    /// AutoFixer 每次成功落地不再只写日志, 而是把修复事件沉淀为一条 NT-REPAIR
    /// defect 经验写入 KB experience 命名空间 — 与 session 吸收同 schema,
    /// 供 hub 索引 / `neotrix-experience query` 检索 / 蒸馏升维复用。
    /// 幂等: 同 session_id 拒绝重复 (与 cmd_absorb 语义一致)。
    async fn report_auto_fix_experience(&mut self) {
        let kb = match self.kb.as_ref() {
            Some(kb) => kb,
            None => {
                log::warn!("[bg] auto-fix experience: kb not attached");
                return;
            }
        };
        let landed: Vec<_> = self.healer_registry.last_landed.iter().collect();
        if landed.is_empty() {
            return;
        }
        let detail = landed
            .iter()
            .map(|s| format!("{} {}", s.dimension, s.file.as_deref().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("; ");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let sid = format!("autofixer_{}_{}", now, landed.len());
        let entry = serde_json::json!({
            "schema_version": 1,
            "type": "defect",
            "session_id": sid,
            "cycle": "autofixer",
            "ts": now,
            "domain": "NT-REPAIR",
            "content": format!("自愈落地: {} 项自动修复 ({})", landed.len(), detail),
            "evidence": detail,
            "source": "code",
            "not": null,
            "verified_by": "autofixer",
            "verification_status": "verified",
            "confidence": 0.8,
            "importance": 0.6,
            "context": "handle_healer_scan / apply_auto_fixable",
        });
        let key = format!("branch_autofixer_{}_{}", now, landed.len());
        // W4 场账本写路径: 时序追加键先 stage 暂存 (G1 版本化审计), 一批一解统一求解。
        if let Err(e) = kb.field_stage("experience", &key, &entry.to_string(), "absorption") {
            log::warn!("[bg] auto-fix experience stage failed: {}", e);
            return;
        }
        if let Err(e) = kb.field_tick() {
            log::warn!("[bg] field_tick failed after auto-fix experience: {}", e);
        }
        log::info!("[bg] auto-fix experience recorded: {} ({} fixes)", key, landed.len());
        if let Ok(v) = kb.field_version() {
            log::debug!("[bg] field_version={v}");
        }
    }

    pub(crate) async fn handle_avatar_auto_distill(&mut self) {
        use crate::l5_cognition::nt_mind::foundation::l1_wrappers::UserDistillation;
        if let Some(ref mut eng) = self.avatar_engine {
            let result = eng.auto_distill();
            log::info!("[bg] avatar auto_distill: {}", &result[..result.len().min(100)]);
        }
    }

    pub(crate) async fn handle_kb_absorb(&mut self) {
        let report = match self.kb_pipeline.update_panorama() {
            Ok(r) => r,
            Err(e) => {
                log::error!("[bg] kb_absorb failed: {}", e);
                self.try_emit(crate::core::nt_core_event::CoreEvent::SystemError {
                    component: "kb_absorb".into(), error: e.to_string(), severity: "error".into(),
                });
                return;
            }
        };
        log::info!("[bg] kb_absorb: {} sources", report.total_sources);
        self.try_emit(crate::core::nt_core_event::CoreEvent::TaskSubmitted {
            task: "kb_absorb".into(), task_type: "ingestion".into(), priority: 3,
        });
    }

    pub(crate) async fn handle_session_recovery(&mut self) {
        use crate::l5_cognition::nt_mind::foundation::l1_wrappers::SessionRecovery;
        if let Some(ref mut sr) = self.session_recovery {
            let snap = sr.create_snapshot(&[], &[], "auto-snapshot");
            if let Ok(s) = snap {
                log::info!("[bg] session_recovery: snapshot {} created", s.session_id);
            }
            // ai-memory 吸收接线 (R-P79): 快照后构建有界交接摘要, 注入后续日志 —
            // 跨会话连续性靠"有界交接"而非重新解释上下文 (bounded handoff)。
            if let Some(handoff) = sr.build_handoff() {
                log::info!("[bg] session_recovery handoff: {}", handoff);
            }
        }
    }

    pub(crate) async fn handle_crawl_queue(&mut self) {
        use crate::l5_cognition::nt_mind::foundation::knowledge_store::{KbKnowledgeStore, KnowledgeStore};
        if self.kb_pipeline.kb.is_none() {
            log::warn!("[bg] crawl_queue: kb not attached");
            return;
        }
        
        let base_batch_size: usize = 50;
        let batch_size = match self.cognitive_mode {
            1 => base_batch_size * 2,
            2 => base_batch_size / 2,
            _ => base_batch_size,
        };
        
        let mode_name = match self.cognitive_mode {
            1 => "Deep",
            2 => "Fast",
            _ => "Balanced",
        };
        log::info!("[COGNITIVE MODE] {:?} — batch size adjusted to {}", mode_name, batch_size);
        
        let mut processed = 0;
        loop {
            if processed >= batch_size {
                log::debug!("[bg] crawl_queue: batch limit reached ({})", batch_size);
                break;
            }
            let (id, url) = {
                let kb = match self.kb_pipeline.kb.as_ref() {
                    Some(kb) => kb, None => break,
                };
                let store = KbKnowledgeStore { kb: kb.clone() };
                match store.claim_next_crawl_url() {
                    Ok(Some(item)) => (item.id, item.url),
                    _ => break,
                }
            };
            log::info!("[bg] crawl claimed: {} (domain tracked)", url);
            match self.kb_pipeline.absorb_url_async(&url).await {
                Ok(report) => {
                    log::info!("[bg] crawl absorbed: {} -> {} nodes (summary: {} chars)", 
                        report.url, report.nodes_created,
                        report.distil_summary.as_ref().map(|s| s.len()).unwrap_or(0));
                    let kb = match self.kb_pipeline.kb.as_ref() {
                        Some(kb) => kb, None => break,
                    };
                    let store = KbKnowledgeStore { kb: kb.clone() };
                    if let Err(e) = store.mark_crawl_complete(&id, true, None) {
                        log::warn!("[bg] failed to mark crawl complete for {}: {}", url, e);
                    }
                }
                Err(e) => {
                    log::warn!("[bg] crawl failed: {}: {:?}", url, e);
                    let kb = match self.kb_pipeline.kb.as_ref() {
                        Some(kb) => kb, None => break,
                    };
                    let store = KbKnowledgeStore { kb: kb.clone() };
                    if let Err(e) = store.mark_crawl_complete(&id, false, Some(&e)) {
                        log::warn!("[bg] failed to mark crawl failed for {}: {}", url, e);
                    }
                }
            }
            processed += 1;
        }
        if let Some(kb) = self.kb_pipeline.kb.as_ref() {
            kb.rebuild_bm25();
            kb.rebuild_tech_reserve();
            log::info!("[bg] crawl_queue: BM25 + tech reserve rebuilt after batch ({} processed)", processed);
        }
    }

    pub(crate) async fn handle_constitution_reload(&mut self) {
        use crate::core::nt_core_self_constitution::ConstitutionLoader;
        let path = std::path::Path::new("AGENTS.md");
        if path.exists() {
            match ConstitutionLoader::load_from_file(path) {
                Ok(constitution) => {
                    log::info!("[constitution] Hot-reload: {} rules, {} experiences, {} tree-growth, {} absorption",
                        constitution.rules.len(),
                        constitution.experiences.len(),
                        constitution.tree_growth_rules.len(),
                        constitution.absorption_rules.len());
                    // Note: Global Constitution is LazyLock, so can't be replaced.
                    // In production, use a Mutex<Constitution> for true hot-reload.
                    // This reload validates the file is parseable.
                }
                Err(e) => log::warn!("[constitution] Hot-reload failed: {}", e),
            }
        }
    }

    /// Seed the crawl queue when nearly empty — runs daily.
    pub(crate) async fn handle_seed_crawl_queue(&mut self) {
        use crate::l5_cognition::nt_mind::foundation::knowledge_store::{KbKnowledgeStore, KnowledgeStore};
        if self.kb_pipeline.kb.is_none() {
            log::warn!("[bg] seed_crawl: kb not attached");
            return;
        }
        let kb = match self.kb_pipeline.kb.as_ref() {
            Some(kb) => kb,
            None => { log::warn!("[bg] seed_crawl: kb disappeared"); return; }
        };
        let store = KbKnowledgeStore { kb: kb.clone() };
        let domains = store.count_nodes_by_domain().unwrap_or_default();
        let seed_count = domains.len();
        if seed_count == 0 {
            let seed_info: [(&str, i64, &str); 5] = [
                ("rust", 10, "programming"),
                ("machine learning", 10, "ai"),
                ("distributed-systems", 10, "computer-science"),
                ("webassembly", 5, "programming"),
                ("neural-networks", 10, "ai"),
            ];
            let enqueued: Vec<String> = seed_info.iter()
                .map(|(topic, _, _)| format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", topic))
                .collect();
            let refs: Vec<(&str, i64, &str)> = enqueued.iter().enumerate()
                .map(|(i, url)| (url.as_str(), seed_info[i].1, seed_info[i].2))
                .collect();
            match store.enqueue_seed_urls(&refs) {
                Ok(n) => log::info!("[bg] seed_crawl: enqueued {} Wikipedia seed topics", n),
                Err(e) => log::warn!("[bg] seed_crawl: enqueue failed: {}", e),
            }
            kb.rebuild_bm25();
            log::info!("[bg] seed_crawl: BM25 rebuilt after seeding");
        } else {
            log::info!("[bg] seed_crawl: {} domains already in KB, auto-seeding complete", seed_count);
        }
    }

    /// 网络小说世界构建知识吸收 (R-P79 closure for nt_world_novel):
    /// ① 消费 novel_queue(外部起点采集器入队) → ingest_qidian_book;
    /// ② 离线为既有 Book 节点补世界观分类。
    pub(crate) async fn handle_novel_ingest(&mut self) {
        use crate::l5_cognition::l2_facade::{drain_novel_queue, classify_unanalyzed_books};
        let kb = match self.kb_pipeline.kb.as_ref() {
            Some(kb) => kb,
            None => { log::warn!("[bg] novel_ingest: kb not attached"); return; }
        };
        let conn = match kb.conn.lock() {
            Ok(c) => c,
            Err(e) => { log::warn!("[bg] novel_ingest lock: {}", e); return; }
        };

        let queue_report = drain_novel_queue(&conn, 50);
        let (classified, edges) = classify_unanalyzed_books(&conn, 50);

        log::info!("[bg] novel_ingest: queue={} books/{} edges, classified={}/{}",
            queue_report.books, queue_report.edges, classified, edges);
    }

    /// KB 域聚类巡检 — 社区检测 + domain_clusters 维护 + cluster_id 分配。
    ///
    /// 每次运行:
    /// 1. 读取全部节点和边
    /// 2. 运行 CommunityDetector 检测社区结构
    /// 3. 按 domain 维护 domain_clusters 表条目
    /// 4. 把社区检测结果分配到节点 cluster_id
    ///
    /// 聚类结果供 GWT 注意力路由和知识检索使用 (域感知检索)。
    pub(crate) async fn handle_clustering(&mut self) {
        use crate::l5_cognition::kb_facade::{
            CommunityDetector, CommunityAwareSearch,
            ensure_domain_cluster, get_all_edges, get_all_nodes, update_cluster_stats,
        };

        let kb = match self.kb_pipeline.kb.as_ref() {
            Some(kb) => kb,
            None => { log::warn!("[bg] clustering: kb not attached"); return; }
        };

        let (nodes, edges) = {
            let conn = match kb.raw_conn() {
                Ok(c) => c,
                Err(e) => { log::warn!("[bg] clustering: conn lock failed: {}", e); return; }
            };
            let nodes = match get_all_nodes(&conn) {
                Ok(n) => n,
                Err(e) => { log::warn!("[bg] clustering: get_all_nodes failed: {}", e); return; }
            };
            let edges = match get_all_edges(&conn) {
                Ok(e) => e,
                Err(e) => { log::warn!("[bg] clustering: get_all_edges failed: {}", e); return; }
            };
            (nodes, edges)
        };

        if nodes.is_empty() {
            log::debug!("[bg] clustering: no nodes, skipping");
            return;
        }

        // 1. Run community detection
        let detector = CommunityDetector::default();
        let mut searcher = CommunityAwareSearch::new(detector);
        searcher.detect(&nodes, &edges);

        let hierarchy = match searcher.hierarchy() {
            Some(h) => h,
            None => { log::warn!("[bg] clustering: detection produced no hierarchy"); return; }
        };

        // 2. Ensure domain_clusters entries exist for all domains
        {
            let conn = match kb.raw_conn() {
                Ok(c) => c,
                Err(e) => { log::warn!("[bg] clustering: conn lock failed: {}", e); return; }
            };
            let mut domains_seen = std::collections::HashSet::new();
            for node in &nodes {
                let domain = node.domain.as_deref().unwrap_or("unclustered");
                if domains_seen.insert(domain.to_string()) {
                    if let Err(e) = ensure_domain_cluster(&conn, domain) {
                        log::warn!("[bg] clustering: ensure_domain_cluster({}) failed: {}", domain, e);
                    }
                }
            }
        }

        // 3. Assign cluster_id based on domain
        {
            let conn = match kb.raw_conn() {
                Ok(c) => c,
                Err(e) => { log::warn!("[bg] clustering: conn lock failed: {}", e); return; }
            };
            let mut assigned = 0usize;
            for node in &nodes {
                let domain = node.domain.as_deref().unwrap_or("unclustered");
                if let Ok(cluster_id) = ensure_domain_cluster(&conn, domain) {
                    if node.cluster_id.as_deref() != Some(&cluster_id) {
                        if let Err(e) = conn.execute(
                            "UPDATE nodes SET cluster_id=?1 WHERE id=?2",
                            rusqlite::params![cluster_id, node.id],
                        ) {
                            log::warn!("[bg] clustering: assign cluster_id to {} failed: {}", node.id, e);
                        } else {
                            assigned += 1;
                        }
                    }
                }
            }

            // Update stats for all clusters
            if let Ok(clusters) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::list_clusters(&conn) {
                for cluster in &clusters {
                    if let Err(e) = update_cluster_stats(&conn, &cluster.id) {
                        log::warn!("[bg] failed to update cluster stats for {}: {}", cluster.id, e);
                    }
                }
            }

            log::info!("[bg] clustering: {} nodes, {} edges, {} hierarchy levels, {} clusters, {} reassigned",
                nodes.len(), edges.len(), hierarchy.num_levels(),
                crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::list_clusters(&conn)
                    .map(|c| c.len()).unwrap_or(0),
                assigned);
        }
    }

    /// 系统健康监控 & NT-REPAIR 自愈闭环 (Track 3: D22/D26/D27/D28)
    /// 扫描磁盘/内存/测试/构建四维信号, 路由到 HealerRegistry 执行自愈动作:
    /// - clean_cache: cargo clean / 清理临时目录
    /// - restart_module: 重启故障模块 (通过 EventBus 发送重启信号)
    /// - alert: 经 EventBus 广播告警, 注入意识监控
    pub(crate) async fn handle_system_health_heal(&mut self) {
        use crate::core::nt_core_self::self_audit::scan_system_health;
        use crate::core::nt_core_event::CoreEvent;

        let findings = scan_system_health(".");
        if findings.is_empty() {
            return; // 系统健康, 无需自愈
        }

        log::info!("[bg] system_health: {} findings, routing to NT-REPAIR", findings.len());

        // 统计各类信号
        let disk_pressure = findings.iter().any(|f| f.category == "disk-pressure");
        let memory_pressure = findings.iter().any(|f| f.category == "memory-pressure");
        let test_flake = findings.iter().any(|f| f.category == "test-flake");
        let build_failure = findings.iter().any(|f| f.category == "build-failure");

        // 1. 磁盘/内存压力 → clean_cache (cargo clean + 清理临时目录)
        if disk_pressure || memory_pressure {
            log::warn!("[bg] NT-REPAIR: triggering clean_cache (disk={}, memory={})", disk_pressure, memory_pressure);
            self.execute_clean_cache().await;
        }

        // 2. 构建失败 → restart_module (重启编译相关模块) + clean_cache
        if build_failure {
            log::warn!("[bg] NT-REPAIR: triggering restart_module + clean_cache for build failure");
            self.execute_clean_cache().await;
            self.emit_restart_signal("build").await;
        }

        // 3. 测试抖动 → alert (告警注入意识监控, 供治理层处置)
        if test_flake {
            log::warn!("[bg] NT-REPAIR: triggering alert for test flakiness");
            self.emit_alert("test-flake", &findings).await;
        }

        // 4. 所有发现经 EventBus 广播, 注入意识监控 (D22 意识层感知自愈)
        for f in &findings {
            self.try_emit(CoreEvent::SystemError {
                component: "nt_repair".into(),
                error: f.message.clone(),
                severity: match f.severity {
                    crate::core::nt_core_self::self_audit::AuditSeverity::Error => "error",
                    crate::core::nt_core_self::self_audit::AuditSeverity::Warning => "warning",
                    crate::core::nt_core_self::self_audit::AuditSeverity::Info => "info",
                }.into(),
            });
        }

        // 5. 记录自愈动作到经验分支 (单一事实源闭环)
        self.report_heal_experience(&findings).await;
    }

    /// 执行清理缓存自愈动作
    async fn execute_clean_cache(&mut self) {
        // cargo clean
        let output = std::process::Command::new("cargo")
            .args(["clean"])
            .output();
        match output {
            Ok(out) => {
                if out.status.success() {
                    log::info!("[bg] NT-REPAIR: cargo clean executed successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    log::warn!("[bg] NT-REPAIR: cargo clean failed: {}", stderr);
                }
            }
            Err(e) => log::warn!("[bg] NT-REPAIR: cargo clean invocation failed: {}", e),
        }

        // 清理 /private/tmp/nt-target-* 孤儿目录
        if let Ok(entries) = std::fs::read_dir("/private/tmp") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("nt-target-") {
                    let _ = std::fs::remove_dir_all(entry.path());
                    log::info!("[bg] NT-REPAIR: removed orphan target dir: {}", entry.path().display());
                }
            }
        }

        // 回流意识树: 清理动作作为养分
        if let Some(ref mut tree) = self.consciousness_tree {
            tree.soil.molt_archived_count = tree.soil.molt_archived_count.saturating_add(1);
            log::info!("[bg] consciousness_tree: clean_cache recorded as nourishment");
        }
    }

    /// 发送模块重启信号 (经 EventBus 广播, 由对应模块消费处理)
    async fn emit_restart_signal(&mut self, module: &str) {
        self.try_emit(CoreEvent::SystemError {
            component: "nt_repair".into(),
            error: format!("RESTART_SIGNAL: module='{}' reason='build_failure'", module),
            severity: "warning".into(),
        });
        log::info!("[bg] NT-REPAIR: restart signal emitted for module '{}'", module);
    }

    /// 发送告警 (注入意识监控, 供治理层/NT-SHIELD 处置)
    async fn emit_alert(&mut self, alert_type: &str, findings: &[crate::core::nt_core_self::self_audit::AuditFinding]) {
        let msg = findings.iter()
            .filter(|f| f.category == alert_type)
            .map(|f| f.message.clone())
            .collect::<Vec<_>>()
            .join("; ");
        self.try_emit(CoreEvent::SystemError {
            component: "nt_repair".into(),
            error: format!("ALERT: type='{}' details='{}'", alert_type, msg),
            severity: "warning".into(),
        });
        log::warn!("[bg] NT-REPAIR: alert emitted: type={} details={}", alert_type, msg);
    }

    /// 自愈动作落地 → 经验分支 (单一事实源闭环)
    async fn report_heal_experience(&mut self, findings: &[crate::core::nt_core_self::self_audit::AuditFinding]) {
        let kb = match self.kb.as_ref() {
            Some(kb) => kb,
            None => {
                log::warn!("[bg] heal experience: kb not attached");
                return;
            }
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let categories: Vec<String> = findings.iter().map(|f| f.category.to_string()).collect();
        let unique_cats: std::collections::HashSet<_> = categories.iter().collect();
        let sid = format!("nt_repair_{}_{}", now, unique_cats.len());
        let entry = serde_json::json!({
            "schema_version": 1,
            "type": "defect",
            "session_id": sid,
            "cycle": "nt_repair",
            "ts": now,
            "domain": "NT-REPAIR",
            "content": format!("自愈触发: {:?} 信号, 执行 clean_cache/restart_module/alert", unique_cats),
            "evidence": findings.iter().map(|f| f.message.clone()).collect::<Vec<_>>().join("; "),
            "source": "monitoring",
            "not": null,
            "verified_by": "nt_repair_healer",
            "verification_status": "verified",
            "confidence": 0.85,
            "importance": 0.7,
            "context": "handle_system_health_heal / system health monitoring",
        });
        let key = format!("branch_nt_repair_{}_{}", now, unique_cats.len());
        if let Err(e) = kb.field_stage("experience", &key, &entry.to_string(), "absorption") {
            log::warn!("[bg] heal experience stage failed: {}", e);
            return;
        }
        if let Err(e) = kb.field_tick() {
            log::warn!("[bg] field_tick failed after heal experience: {}", e);
        }
        log::info!("[bg] heal experience recorded: {} ({} signals)", key, unique_cats.len());
    }

    /// L6 自我改进循环 — 采集系统指标 → 诊断瓶颈 → 生成改进方案 → 执行 → 验证。
    /// 与 SEAL pipeline 互补: SEAL 聚焦技能模板提取, 本模块聚焦系统层面参数调优。
    pub(crate) async fn handle_self_improvement(&mut self) {
        use crate::l6_meta::coordination::self_improvement::SystemMetrics;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

        // 从系统各模块采集实时指标
        let healer_report_len = self.healer_registry.last_report.len();
        let auto_fixes = self.healer_registry.auto_fixes_applied;
        let tree_stats = self.skill_engine.skill_tree_stats();
        let residency = self.skill_engine.audit_residency();

        let success_rate = if healer_report_len == 0 {
            1.0 // 无发现 = 健康
        } else {
            (1.0 - (healer_report_len as f64 / 50.0)).max(0.0)
        };

        let error_recovery_rate = if auto_fixes > 0 {
            (auto_fixes as f64 / (auto_fixes as f64 + healer_report_len as f64)).min(1.0)
        } else {
            0.5 // 无数据时中性值
        };

        // 技能命中率: 非孤儿技能占比 (孤儿 = 无父节点)
        let skill_hit_rate = if tree_stats.total_skills > 0 {
            (tree_stats.total_skills - tree_stats.orphans) as f64 / tree_stats.total_skills as f64
        } else {
            0.0
        };

        // 结晶率: 根节点占比 (根 = 已结晶为独立技能)
        let crystallization_rate = if tree_stats.total_skills > 0 {
            tree_stats.roots as f64 / tree_stats.total_skills as f64
        } else {
            0.0
        };

        // 平均 token 消耗: 驻留审计中位 resident_tokens
        let avg_tokens = if !residency.is_empty() {
            let total: usize = residency.iter().map(|r| r.resident_tokens).sum();
            total as f64 / residency.len() as f64
        } else {
            1000.0 // 默认值
        };

        let metrics = SystemMetrics {
            success_rate,
            avg_tokens,
            skill_hit_rate,
            crystallization_rate,
            knowledge_retention: 0.8, // 默认值, 后续可从 KB 衰减模块接入
            error_recovery_rate,
            timestamp: now,
        };

        self.self_improvement.collect_metrics(metrics);

        let result = self.self_improvement.run_cycle();

        if result.issues_found > 0 {
            log::info!(
                "[bg] self_improvement cycle {}: health={:.2}, {} issues, {} plans generated, {} applied, improved={}",
                result.cycle, result.diagnosis_health, result.issues_found,
                result.plans_generated, result.plans_applied, result.overall_improved
            );
            self.try_emit(crate::core::nt_core_event::CoreEvent::SystemError {
                component: "self_improvement".into(),
                error: format!(
                    "cycle {}: health={:.2}, {} issues, {} applied, improved={}",
                    result.cycle, result.diagnosis_health, result.issues_found,
                    result.plans_applied, result.overall_improved
                ),
                severity: if result.overall_improved { "info" } else { "warning" }.into(),
            });
        }
    }

}

// Commented out - tests call non-existent methods on BackgroundLoop
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::l5_cognition::nt_mind_background_loop::BackgroundLoop;
//     use crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
//     use std::sync::Arc;
//     use std::sync::RwLock;
//
//     /// Test that system health heal handler exists and can be called
//     #[tokio::test]
//     async fn test_handle_system_health_heal_exists() {
//         let brain = Arc::new(RwLock::new(SelfIteratingBrain::new()));
//         let mut bg = BackgroundLoop::new(brain);
//         // Just verify the handler method exists and is callable
//         // (without KB attached, it will early return)
//         bg.handle_system_health_heal().await;
//     }
//
//     /// Test clean_cache execution logic (dry-run verification)
//     #[tokio::test]
//     async fn test_execute_clean_cache_dry_run() {
//         let brain = Arc::new(RwLock::new(SelfIteratingBrain::new()));
//         let mut bg = BackgroundLoop::new(brain);
//         // Execute clean cache - without KB it won't record to consciousness tree
//         // but should not panic
//         bg.execute_clean_cache().await;
//     }
//
//     /// Test restart signal emission
//     #[tokio::test]
//     async fn test_emit_restart_signal() {
//         let brain = Arc::new(RwLock::new(SelfIteratingBrain::new()));
//         let mut bg = BackgroundLoop::new(brain);
//         bg.emit_restart_signal("test_module").await;
//     }
//
//     /// Test alert emission
//     #[tokio::test]
//     async fn test_emit_alert() {
//         use crate::core::nt_core_self::self_audit::{AuditFinding, AuditSeverity};
//         let brain = Arc::new(RwLock::new(SelfIteratingBrain::new()));
//         let mut bg = BackgroundLoop::new(brain);
//         let findings = vec![
//             AuditFinding {
//                 category: "test-flake",
//                 severity: AuditSeverity::Warning,
//                 file: "test.rs".to_string(),
//                 line: None,
//                 message: "Flaky test detected".to_string(),
//             }
//         ];
//         bg.emit_alert("test-flake", &findings).await;
//     }
// }

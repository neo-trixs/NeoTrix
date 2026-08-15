use super::*;

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
            eprintln!("[bg] crystallization: {}", self.auto_crystallizer.summary());
        }
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
                let _ = registry.add_dependency(&from, &to);
            }
        }
        registry.experience_targets = export.experience_targets;
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
                    .to_uppercase()
                    .into(),
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
    }

    pub(crate) async fn handle_avatar_auto_distill(&mut self) {
        if let Some(ref mut eng) = self.avatar_engine {
            #[allow(clippy::mut_mutex_lock)]
            if let Ok(mut e) = eng.lock() {
                let _snapshot = e.auto_distill();
                log::info!("[bg] avatar auto_distill: edition={}, confidence={:.2}, msgs={}",
                    e.avatar.edition, e.avatar.confidence, e.avatar.total_messages_processed);
            }
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
        if let Some(ref mut sr) = self.session_recovery {
            let snap = sr.create_snapshot(&[], &[], "auto-snapshot");
            if let Ok(s) = snap {
                log::info!("[bg] session_recovery: snapshot {} created", s.session_id);
            }
        }
    }

    pub(crate) async fn handle_crawl_queue(&mut self) {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::{claim_next_crawl_url, mark_crawl_complete};
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
                let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
                match claim_next_crawl_url(&conn) {
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
                    let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = mark_crawl_complete(&conn, &id, true, None);
                }
                Err(e) => {
                    log::warn!("[bg] crawl failed: {}: {:?}", url, e);
                    let kb = match self.kb_pipeline.kb.as_ref() {
                        Some(kb) => kb, None => break,
                    };
                    let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = mark_crawl_complete(&conn, &id, false, Some(&e));
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
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::count_nodes_by_domain;
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::enqueue_seed_urls;
        if self.kb_pipeline.kb.is_none() {
            log::warn!("[bg] seed_crawl: kb not attached");
            return;
        }
        let kb = match self.kb_pipeline.kb.as_ref() {
            Some(kb) => kb,
            None => { log::warn!("[bg] seed_crawl: kb disappeared"); return; }
        };
        let conn = match kb.conn.lock() {
            Ok(c) => c,
            Err(e) => { log::warn!("[bg] seed_crawl lock: {}", e); return; }
        };
        let domains = count_nodes_by_domain(&conn).unwrap_or_default();
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
            match enqueue_seed_urls(&conn, &refs) {
                Ok(n) => log::info!("[bg] seed_crawl: enqueued {} Wikipedia seed topics", n),
                Err(e) => log::warn!("[bg] seed_crawl: enqueue failed: {}", e),
            }
            drop(conn);
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
        use crate::neotrix::l2_world_impl::nt_world_novel::{drain_novel_queue, classify_unanalyzed_books};
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

}

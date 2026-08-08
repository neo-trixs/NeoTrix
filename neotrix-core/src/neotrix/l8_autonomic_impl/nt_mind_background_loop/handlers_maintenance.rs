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

        // 2. 清理 .DS_Store
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

        // 3. 整理旧快照
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
    }

    pub(crate) async fn handle_skill_scan(&mut self) {
        let skills = self.skill_engine.load_all();
        if !skills.is_empty() {
            log::info!("[bg] skill_scan: {} skills loaded", skills.len());
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

use super::*;
use super::super::nt_mind::self_iterating::ReasoningBrain;
use super::super::nt_mind::memory::ReasoningBank;

impl BackgroundLoop {
    pub async fn start(&mut self) {
        if !self.config.enabled { return; }

        println!("[bg] background loop started");

        let mut always_on_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.always_on_interval_secs));
        let mut save_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.save_interval_secs));
        let mut consolidate_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.consolidate_interval_secs));
        let mut meta_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.metacog_interval_secs));
        let mut thinking_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.thinking_interval_secs));
        let mut goal_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.goal_interval_secs));
        let mut prediction_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.world_prediction_interval_secs));
        let mut panorama_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.panorama_interval_secs));
        let mut exploration_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.exploration_interval_secs));
        let mut curiosity_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.curiosity_interval_secs));
        let mut knowledge_chain_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.knowledge_chain_interval_secs));
        let mut aging_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.knowledge_aging_interval_secs));
        let mut crystallization_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.crystallization_interval_secs));
        let mut nt_act_voice_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.nt_act_voice_interval_secs));
        let mut awareness_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.metacog_interval_secs));
        let mut plugin_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.plugin_interval_secs));
        let mut discovery_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(60));

        #[cfg(feature = "stealth-net")]
        let mut heartbeat_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.proxy_heartbeat_interval_secs));
        #[cfg(not(feature = "stealth-net"))]
        let mut heartbeat_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(3600));

        #[cfg(feature = "stealth-net")]
        let mut nt_world_sense_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.nt_world_sense_interval_secs));
        #[cfg(not(feature = "stealth-net"))]
        let mut nt_world_sense_ticker = tokio::time::interval(
            tokio::time::Duration::from_secs(3600));

        #[cfg(feature = "stealth-net")]
        if let Some(ref nt_world_crawl) = self.tor_crawler {
            let c = nt_world_crawl.clone();
            tokio::spawn(async move { c.run().await });
        }

        // Spawn AgentServer if discovery is enabled
        if self.agent_discovery.is_some() {
            let server = std::sync::Arc::new(super::super::nt_agent_protocol::tcp_server::AgentServer::new(42070));
            let server_clone = server.clone();
            tokio::spawn(async move {
                match server_clone.start().await {
                    Ok(port) => log::info!("[bg] AgentServer listening on TCP :{}", port),
                    Err(e) => log::error!("[bg] AgentServer start failed: {}", e),
                }
            });
        }

        loop {
            tokio::select! {
                _ = always_on_ticker.tick() => self.handle_always_on().await,
                _ = save_ticker.tick() => self.handle_save().await,
                _ = consolidate_ticker.tick() => self.handle_consolidate().await,
                _ = meta_ticker.tick() => self.handle_meta().await,
                _ = thinking_ticker.tick() => self.handle_thinking().await,
                _ = goal_ticker.tick() => self.handle_goal().await,
                _ = prediction_ticker.tick() => self.handle_prediction().await,
                _ = panorama_ticker.tick() => self.handle_panorama().await,
                _ = exploration_ticker.tick() => self.handle_exploration().await,
                _ = curiosity_ticker.tick() => self.handle_curiosity().await,
                _ = knowledge_chain_ticker.tick() => self.handle_knowledge_chain().await,
                _ = aging_ticker.tick() => self.handle_knowledge_aging().await,
                _ = crystallization_ticker.tick() => self.handle_crystallization().await,
                _ = heartbeat_ticker.tick() => {
                    #[cfg(feature = "stealth-net")]
                    self.handle_proxy_heartbeat().await;
                },
                _ = nt_world_sense_ticker.tick() => {
                    #[cfg(feature = "stealth-net")]
                    self.handle_nt_world_sense_tick().await;
                },
                _ = awareness_ticker.tick() => self.handle_awareness().await,
                _ = nt_act_voice_ticker.tick() => self.handle_nt_act_voice_tick().await,
                _ = plugin_ticker.tick() => self.handle_plugin_tick().await,
                _ = discovery_ticker.tick() => self.handle_agent_discovery().await,
            }
        }
    }

    async fn handle_save(&self) {
        let b = self.brain.write().await;
        if let Err(e) = b.brain.save() {
            eprintln!("[bg] auto-save failed: {}", e);
        }
        let _ = self.goal_loop.save();
    }

    async fn handle_consolidate(&self) {
        let mut b = self.brain.write().await;
        let r = b.consolidate_memories();
        eprintln!("[bg] consolidated: {} merged, {} pruned, {} replayed",
            r.merged_count, r.pruned_count, r.replayed_count);
    }

    async fn handle_meta(&mut self) {
        let r = self.metacognition.run_full_cycle();
        eprintln!("[bg] meta cycle #{}", r.iteration);
    }

    async fn handle_thinking(&self) {
        let mut b = self.brain.write().await;
        b.iterate(super::super::nt_world_model::TaskType::General);
    }

    async fn handle_goal(&mut self) {
        let mut b = self.brain.write().await;
        self.goal_loop.pursue_all(&mut b, 1);
    }

    async fn handle_prediction(&mut self) {
        // Pre-action introspection before prediction cycle
        if let Some(ref mut ip) = self.introspector {
            let state = self.thinking.silicon.current_state();
            let action = format!("prediction_cycle_{}", self.thinking.silicon.iteration);
            let _r = ip.introspect(
                &action,
                crate::core::nt_core_self::reasoning_strategy::StrategyKind::Deliberate,
                crate::core::nt_core_self::attention_head::AttentionDomain::Planning,
                &state,
            );
        }

        // 1. PREDICT — run panorama pipeline
        let report_opt = if let Some(ref mut pano) = self.panorama {
            let mut brain = self.brain.write().await;
            if let Some(ref mut wm) = self.nt_world_model {
                let report = pano.run_cycle(&mut brain, &mut self.goal_loop, wm);
                eprintln!("[bg] prediction: cycle={}, anomaly={}", report.cycle, report.anomaly);
                Some(report)
            } else {
                None
            }
        } else {
            if let Some(ref wm) = self.nt_world_model {
                wm.predict_all(&[]);
            }
            None
        };

        // 2. OBSERVE — run awareness monitor after prediction
        if let Some(ref mut aw) = self.awareness {
            aw.observe();
            let level = aw.current.consciousness_level;
            let phi = aw.current.phi_current;
            let coherence = aw.current.coherence_current;
            let anomaly_flag = report_opt.as_ref().map(|r| r.anomaly).unwrap_or(false);
            eprintln!("[bg] awareness after prediction: consciousness={:.3}, phi={:.4}, coherence={:.3}, anomaly={}",
                level, phi, coherence, anomaly_flag);
        }

        // 3. REPORT — consolidated prediction summary
        if let Some(ref report) = report_opt {
            eprintln!("[bg] prediction report: cycle={}, hypercube={}, cortex={}, gwt={}, fe={:.3}, phi={:.3}, goals={}",
                report.cycle, report.hypercube_entries, report.cortex_traces,
                report.gwt_broadcasts, report.fe_energy, report.phi, report.goals_created);
        }

        #[cfg(feature = "stealth-net")]
        self.handle_stealth_rotation().await;
    }

    async fn handle_panorama(&self) {
        if let Some(ref pano) = self.panorama {
            eprintln!("[bg] panorama status: {}", pano.status());
        }
    }

    async fn handle_exploration(&mut self) {
        let sources_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".neotrix")
            .join("exploration_sources.txt");

        let urls: Vec<String> = match std::fs::read_to_string(&sources_path) {
            Ok(content) => content.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect(),
            Err(_) => Vec::new(),
        };

        if !urls.is_empty() {
            if let Some(ref mut evolver) = self.self_evolver {
                for url in &urls {
                    if !SelfEvolver::is_url(url) {
                        eprintln!("[bg] skipping non-URL: {}", url);
                        continue;
                    }
                    match evolver.evolve_from_url(url) {
                        Ok(reward) => eprintln!("[bg] exploration evolved {}: reward={:.3}", url, reward),
                        Err(e) => eprintln!("[bg] exploration failed {}: {}", url, e),
                    }
                }
            }

            // Clear processed URLs to avoid re-processing
            if let Err(e) = std::fs::write(&sources_path, "") {
                eprintln!("[bg] failed to clear exploration sources: {}", e);
            }
        }

        // Knowledge gap detection — scans what's missing vs known sources
        if let Some(ref mut gap_detector) = self.gap_detector {
            use crate::core::nt_core_meta::scanner::CodeScanner;
            use crate::core::nt_core_meta::weakness::WeaknessAnalyzer;
            let scanner = CodeScanner::new(".");
            let model = scanner.scan();
            let analyzer = WeaknessAnalyzer::new();
            let weaknesses = analyzer.analyze(&model);
            let report = gap_detector.detect_gaps(&model, &weaknesses.weaknesses);
            if report.high_priority_count > 0 {
                eprintln!("[bg] gap detection: {} gaps, {} high-priority, coherence={:.2}",
                    report.total_gaps, report.high_priority_count, report.coherence_score);
                for s in report.exploration_suggestions.iter().take(3) {
                    eprintln!("[bg]   gap suggestion: {}", s);
                }
            }
        }
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_stealth_rotation(&mut self) {
        if let Some(ref mut sm) = self.nt_shield_manager {
            let tags: Vec<String> = vec!["pool_0".to_string()];
            let _id = sm.get_identity(&tags);
            let stats = sm.stats();
            if stats.active_count < stats.total_identities {
                eprintln!("[bg] stealth: {}/{} active, success={:.2}, confidence={:.2}",
                    stats.active_count, stats.total_identities, stats.avg_success_rate, stats.avg_confidence);
            }
        }
    }

    async fn handle_awareness(&mut self) {
        if let Some(ref mut aw) = self.awareness {
            aw.observe();
            let level = aw.current.consciousness_level;
            let phi = aw.current.phi_current;
            let coherence = aw.current.coherence_current;
            eprintln!("[bg] awareness: consciousness={:.3}, phi={:.4}, coherence={:.3}",
                level, phi, coherence);
        }
    }

    async fn handle_always_on(&mut self) {
        use crate::neotrix::nt_mind_background_loop::always_on::AlwaysOnEngine;
        static ALWAYS_ON: std::sync::LazyLock<std::sync::Mutex<Option<AlwaysOnEngine>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(None));

        let report = {
            let mut guard = ALWAYS_ON.lock().unwrap();
            let engine = guard.get_or_insert_with(|| {
                let mut e = AlwaysOnEngine::load();
                e.enabled = true;
                e
            });
            if engine.enabled {
                match engine.full_cycle() {
                    Ok(report) => Some(report),
                    Err(_e) => {
                        let _ = engine.save();
                        None
                    }
                }
            } else {
                None
            }
        };

        if let Some(report) = report {
            if report.tasks_executed > 0 {
                eprintln!("[bg] always_on: scanned={}, executed={}, completed={}, took={}ms",
                    report.scan_count, report.tasks_executed, report.tasks_completed, report.duration_ms);
                let guard = ALWAYS_ON.lock().unwrap();
                if let Some(ref engine) = *guard {
                    let _ = engine.save();
                }
            }
        }
    }

    async fn handle_nt_act_voice_tick(&mut self) {
        if let Some(ref mut vi) = self.nt_act_voice_input {
            if !vi.is_active() {
                return;
            }
            if vi.is_continuous() {
                if let Some(text) = vi.poll_transcription() {
                    eprintln!("[nt_act_voice] transcribed: {}", text);
                    let cmd = crate::neotrix::nt_act_voice::VoiceCommand::parse(&text);
                    match cmd {
                        crate::neotrix::nt_act_voice::VoiceCommand::OpenSettings => {
                            eprintln!("[nt_act_voice] command: open settings");
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::ShowHelp => {
                            eprintln!("[nt_act_voice] command: show help");
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::RunCommand(c) => {
                            eprintln!("[nt_act_voice] command: run {}", c);
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::SwitchSession(s) => {
                            eprintln!("[nt_act_voice] command: switch to session {}", s);
                        }
                        _ => {}
                    }
                }
            } else if vi.check_wake_word() {
                eprintln!("[nt_act_voice] wake word detected");
            }
        }
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_nt_world_sense_tick(&mut self) {
        if let Some(ref mut wc) = self.world_consciousness {
            let events = wc.nt_world_sense.poll_all();
            if !events.is_empty() {
                log::info!("[nt_world_sense] {} new events", events.len());
            }
            wc.refresh_self_awareness();
            log::info!("[nt_world_sense] status: {}", wc.consciousness_status());
        }
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_proxy_heartbeat(&self) {
        if let Some(ref engine) = self.heartbeat_engine {
            let record = engine.tick().await;
            if record.success {
                eprintln!("[bg] proxy heartbeat #{}: proxy={}, geo={:?}, fp={}, dns={}",
                    record.tick,
                    record.proxy_url,
                    record.proxy_geo,
                    record.fingerprint_id,
                    record.dns_flushed,
                );
            } else {
                eprintln!("[bg] proxy heartbeat #{}: no proxy available (pool empty?)", record.tick);
            }
        }
        self.handle_proxy_auto_mode().await;
    }

    /// 根据 brain 上下文自动切换 proxy 模式
    #[cfg(feature = "stealth-net")]
    async fn handle_proxy_auto_mode(&self) {
        use super::super::nt_shield_stealth_net::proxy_control::DaemonMode;

        let client = match self.proxy_client {
            Some(ref c) => c,
            None => return,
        };
        if !client.is_reachable().await {
            return;
        }

        // 读取当前 daemon 模式
        let status_str = match client.status().await {
            Ok(s) => s,
            Err(_) => return,
        };
        let current = match serde_json::from_str::<serde_json::Value>(&status_str) {
            Ok(v) => DaemonMode::from_str(v["mode"].as_str().unwrap_or("geo")).unwrap_or(DaemonMode::Geo),
            Err(_) => return,
        };

        // 决定目标模式 (函数已 #[cfg(feature = "stealth-net")], 字段可用)
        let target = if self.tor_crawler.is_some() {
            DaemonMode::Tor
        } else if self.nt_shield_manager.as_ref().map_or(false, |sm| sm.stats().active_count > 0) {
            DaemonMode::Stealth
        } else {
            DaemonMode::Geo
        };

        if target != current {
            match client.set_mode(target).await {
                Ok(_) => eprintln!("[bg] proxy auto-mode: {} → {}", current.as_str(), target.as_str()),
                Err(e) => eprintln!("[bg] proxy auto-mode failed: {}", e),
            }
        }
    }

    async fn handle_plugin_tick(&self) {
        use crate::neotrix::nt_io_plugin::PluginEvent;
        self.plugin_registry.dispatch(&PluginEvent::BrainTick);
    }

    /// Periodic agent discovery listener — sweep for UDP broadcasts.
    async fn handle_agent_discovery(&mut self) {
        if let Some(ref mut discovery) = self.agent_discovery {
            if let Err(e) = discovery.listen() {
                log::warn!("[bg] agent discovery listen: {}", e);
            }
            if discovery.agent_count() > 0 {
                log::info!("[bg] known agents: {}", discovery.agent_count());
            }
        }
    }

    /// Curiosity drive: knowledge gaps → GWT attention → exploration queries
    /// Wired to negentropy: gap sparsity → negentropy proxy → curiosity calibration
    async fn handle_curiosity(&mut self) {
        use crate::neotrix::nt_mind::hypercube_bridge::HyperCubeBridge;
        let gap_reports = {
            let bridge = HyperCubeBridge::new();
            bridge.analyze_gaps()
        };

        self.curiosity_drive.ingest_gap_reports(&gap_reports);

        // Negentropy alignment: use gap sparsity as inverse negentropy proxy
        // sparsity ↑ → order ↓ → negentropy ↓ → curiosity ↑
        let n_total_proxy = if gap_reports.is_empty() {
            0.5
        } else {
            let avg_sparsity: f64 = gap_reports.iter().map(|r| r.sparsity_score).sum::<f64>() / gap_reports.len() as f64;
            (1.0 - avg_sparsity).clamp(0.0, 1.0)
        };
        self.curiosity_drive.calibrate_to_negentropy(n_total_proxy, 0.0);

        let queries = self.curiosity_drive.drain_queries();

        if !queries.is_empty() {
            eprintln!("[bg] curiosity: {} signals, {} queries generated: {:?}",
                self.curiosity_drive.signals.len(),
                queries.len(),
                &queries[..queries.len().min(3)],
            );
            if let Some(ref mut evolver) = self.self_evolver {
                for query_str in queries.iter().take(2) {
                    let q: &String = query_str;
                    let search_url = format!("https://en.wikipedia.org/wiki/{}", q.replace(' ', "_"));
                    match evolver.evolve_from_url(&search_url) {
                        Ok(reward) => eprintln!("[bg] curiosity evolved {}: reward={:.3}", q, reward),
                        Err(e) => eprintln!("[bg] curiosity failed {}: {}", q, e),
                    }
                }
            }
        } else {
            let level = self.curiosity_drive.curiosity_level;
            let signal_count = self.curiosity_drive.signals.len();
            if signal_count > 0 {
                eprintln!("[bg] curiosity: {:?}, {} signals, {} total gaps",
                    level, signal_count, self.curiosity_drive.total_gaps_detected);
            }
        }
    }

    /// Knowledge chain: discovery → mining → validation → absorption → storage
    async fn handle_knowledge_chain(&mut self) {
        if let Some(ref mut chain) = self.knowledge_chain {
            let kc: &mut KnowledgeChain = chain;
            if !kc.has_pending() {
                kc.init_default_discovery();
            }
            let mut brain = ReasoningBrain::new();
            let mut bank = ReasoningBank::new(100);
            match kc.run_chain(&mut brain, &mut bank) {
                Ok(result) => {
                    eprintln!("[bg] knowledge chain: discovered={}, mined={}, absorbed={}, reward={:.3}",
                        result.discovered, result.mined, result.absorbed, result.total_reward);
                    if result.absorbed > 0 && self.config.enable_auto_crystallize {
                        for d in &result.details {
                            let detail: &String = d;
                            if detail.starts_with("吸收阶段") {
                                let edits = vec![super::super::nt_mind::self_edit::MicroEdit::NormalizeVector];
                                self.auto_crystallizer.crystallize_from_absorption(
                                    &mut brain, &mut bank,
                                    "knowledge_chain", "chain_batch", "general",
                                    &edits, result.total_reward / result.absorbed as f64,
                                );
                            }
                        }
                    }
                }
                Err(e) => eprintln!("[bg] knowledge chain failed: {}", e),
            }
        }
    }

    /// Knowledge aging: score decay → stale detection → re-scan scheduling
    async fn handle_knowledge_aging(&mut self) {
        let report = self.knowledge_aging.run_aging_cycle();
        if report.stale_count > 0 || report.expired_count > 0 {
            eprintln!("[bg] knowledge aging: {} survived, {} stale, {} expired, avg_age={:.1}d",
                report.surviving_entries, report.stale_count,
                report.expired_count, report.avg_age_days);

            if !report.rescans_needed.is_empty() {
                eprintln!("[bg] aging: {} rescans needed", report.rescans_needed.len());
                if let Some(ref mut evolver) = self.self_evolver {
                    for url_str in report.rescans_needed.iter().take(3) {
                        if SelfEvolver::is_url(url_str) {
                            let ev: &mut SelfEvolver = evolver;
                            match ev.evolve_from_url(url_str) {
                                Ok(reward) => eprintln!("[bg] re-scan {}: reward={:.3}", url_str, reward),
                                Err(e) => eprintln!("[bg] re-scan failed {}: {}", url_str, e),
                            }
                        }
                    }
                }
            }
        }
    }

    /// Auto-crystallization: check SelfEvolver results → create SkillCrystals
    async fn handle_crystallization(&mut self) {
        if !self.config.enable_auto_crystallize {
            return;
        }
        let summary = self.auto_crystallizer.summary();
        eprintln!("[bg] crystallization: {}", summary);
    }
}

#[cfg(test)]
mod tests {
    use crate::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    use crate::neotrix::nt_world_model_v2::WorldModelV2;
    use crate::neotrix::nt_mind::goal_loop::GoalLoop;

    #[test]
    fn test_panorama_pipeline_new() {
        let pano = PanoramaPipeline::new();
        assert_eq!(pano.cycle, 0);
        assert_eq!(pano.total_anomalies, 0);
    }

    #[test]
    fn test_panorama_pipeline_status_nonempty() {
        let pano = PanoramaPipeline::new();
        let s = pano.status();
        assert!(!s.is_empty());
        assert!(s.contains("Panorama:"));
    }

    #[test]
    fn test_panorama_run_cycle_basic() {
        let mut pano = PanoramaPipeline::new();
        let mut brain = SelfIteratingBrain::new();
        let mut goal_loop = GoalLoop::new();
        let mut wm = WorldModelV2::new(4, 64);

        let report = pano.run_cycle(&mut brain, &mut goal_loop, &mut wm);
        assert_eq!(report.cycle, 1);
        assert!(report.hypercube_entries > 0);
    }

    #[test]
    fn test_panorama_multiple_cycles() {
        let mut pano = PanoramaPipeline::new();
        let mut brain = SelfIteratingBrain::new();
        let mut goal_loop = GoalLoop::new();
        let mut wm = WorldModelV2::new(4, 64);

        for i in 1..=3 {
            let report = pano.run_cycle(&mut brain, &mut goal_loop, &mut wm);
            assert_eq!(report.cycle, i);
        }
        assert_eq!(pano.cycle, 3);
        assert!(pano.status().contains("cycle=3"));
    }
}

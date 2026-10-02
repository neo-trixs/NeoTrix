use super::super::*;

impl BackgroundLoopHandle {
    /// EventBus behavioral consumer (D30) — responds to events with brain/KB actions, not just logs.
    pub(crate) async fn handle_event_bus_event(&mut self, event: CoreEvent) {
        // ── G3 写回闭环 (R-P79): 事件作为源项 ΔJ 入场 (版本链可审计),
        // EventBus 从"事实同步信道"降级为"观测仪器输入"。行为反应仍走下方 match。
        if let Some(ref kb) = self.kb {
            let kind = format!("{:?}", event);
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let key = format!("ev_{}", ts);
            let _ = kb.field_stage("field_events", &key, &kind, "event_bus");
            if let Ok(Some(r)) = kb.field_tick() {
                self.state.record_metric("field_version", r.version as f64);
            }
        }
        match &event {
            CoreEvent::SystemError {
                severity,
                component,
                error,
            } if severity == "critical" => {
                log::error!("[bg] event_bus: CRITICAL {}: {}", component, error);
                if let Ok(mut brain) = self.brain.try_write() {
                        self.goal_loop.enqueue_goal(
                            &mut brain,
                            &format!("event_bus_critical: {} - {}", component, error),
                            None,
                        );
                }
            }
            CoreEvent::GlobalHalt { reason, source } => {
                log::error!("[bg] event_bus: GLOBAL HALT {} from {}", reason, source);
                if let Some(ref kb) = self.kb {
                    // ⭐ 2026-10-02：原 `let _ =` 丢弃 ⇒ 落库失败无痕。⭐ 决定性证据：`:35` 已 `log::error!("[bg] event_bus: GLOBAL HALT …")` ⇒ 停机事件本身**已告警**；本写只是可事后查询的耐久副本，丢失不影响「停机被知道」这件事。
                    // ⭐ 范式抄**同目录** `nt_awareness.rs`（该文件 5 处已是 `if let Err(e) = kb.kv_set(…) { log::warn!(…) }`）
                    //    —— 本目录里 `nt_event_bus.rs` / `nt_audit.rs` 是仅剩的漏网，同一次修复应一起改。
                    if let Err(e) = kb.kv_set(
                        "event_bus",
                        "global_halt",
                        &serde_json::json!({
                            "reason": reason, "source": source,
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs(),
                        })
                        .to_string(),
                    ) { log::warn!("[bg] task_submitted 落库失败: {e}"); }
                }
                  // 2026-09-30: 原来只有 `if let Ok(mut brain) = self.brain.try_write()`，
                  // **无 else**。`GlobalHalt` 是全系统停机事件，其恢复动作就是这个
                  // `event_bus_recovery` 目标；`try_write()` 在 brain 锁被持有时返回 Err
                  // ⇒ 恢复目标不入队、无日志、事件处理继续。用户看到
                  // `GLOBAL HALT` 的 error 日志却等不到任何恢复，日志里也找不到
                  // 「恢复目标未入队」这一句。⇒ 补 else 分支显式告警。
                  if let Ok(mut brain) = self.brain.try_write() {
                      self.goal_loop.enqueue_goal(
                          &mut brain,
                          &format!("event_bus_recovery: {} - {}", source, reason),
                          None,
                      );
                  } else {
                      log::error!(
                          "[bg] GlobalHalt 恢复目标未入队（brain 锁被占）—— 需人工触发恢复: {source} - {reason}"
                      );
                  }
            }
            CoreEvent::ConsciousnessCritique { quality, .. }
                if *quality < CONSCIOUSNESS_THRESHOLDS.eventbus_critical =>
            {
                log::warn!("[bg] event_bus: consciousness CRITICAL ({:.3})", quality);
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set(
                        "event_bus",
                        "consciousness_critical",
                        &serde_json::json!({
                            "quality": quality,
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs(),
                        })
                        .to_string(),
                    );
                }
            }
            CoreEvent::TaskSubmitted {
                task,
                task_type,
                priority,
            } if *priority >= 3 => {
                log::info!(
                    "[bg] event_bus: high-priority task {} ({})",
                    task,
                    task_type
                );
                // R-P41: 高优先级任务入 KB，供后续 handler 消费，而非纯日志
                if let Some(ref kb) = self.kb {
                    // ⭐ 2026-10-02：原 `let _ =` 丢弃 ⇒ 落库失败无痕。⭐ `:96` 注释明写「R-P41: 高优先级任务入 KB，供后续 handler 消费，而非纯日志」⇒ 本写**就是投递通道**。
                    // ⭐ 范式抄**同目录** `nt_awareness.rs:73/91`（该文件已是 `if let Err(e) = kb.kv_set(…) { log::warn!(…) }`）
                    //    —— 本目录里 `nt_event_bus.rs` / `nt_audit.rs` 是仅剩的漏网，同一次修复应一起改。
                    if let Err(e) = kb.kv_set(
                        "event_bus",
                        "task_submitted",
                        &serde_json::json!({
                            "task": task, "task_type": task_type, "priority": priority,
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs(),
                        })
                        .to_string(),
                    ) { log::warn!("[bg] task_submitted 落库失败: {e}"); }
                }
            }
            _ => {
                log::trace!("[bg] event_bus: {:?}", event);
            }
        }
    }
}

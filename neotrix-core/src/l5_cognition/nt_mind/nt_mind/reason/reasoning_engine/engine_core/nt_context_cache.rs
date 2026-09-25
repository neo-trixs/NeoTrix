

use crate::l0_substrate::nt_core_hex::{FullReasoningState, ReasoningHexagram};
use crate::l0_substrate::nt_core_span::{ Span,
};
use crate::l5_cognition::l1_facade::{ SearchResult};
use crate::l5_cognition::nt_mind::nt_mind::reasoning_types::{ ReasoningType};
use crate::l1_action::nt_io::nt_io_provider::{estimate_tokens};
use super::nt_builders::{ReasoningEngine, MAX_KB_CACHE_ENTRIES, MAX_KB_INJECTION_TOKENS};

#[derive(serde::Serialize, serde::Deserialize)]
struct E8PersistedState {
    current_mode: u8,
    current_meta: u8,
    last_e8_attention_weights: Option<Vec<f64>>,
    #[serde(default)]
    last_e8_confidence: f64,
    trajectory_modes: Vec<(u8, u8)>,
    /// Serialized E8Policy RL state (mode_values, mode_counts, factor_energies, factor_control)
    #[serde(default)]
    e8_policy: Option<crate::l5_cognition::nt_core_policy::E8Policy>,
    /// PRM learning count
    #[serde(default)]
    prm_learning_count: u64,
    /// PRM score history (last 100)
    #[serde(default)]
    prm_score_history: Vec<f64>,
}

impl ReasoningEngine {
    pub(crate) fn _current_state_string(&self) -> String {
        let hex = self.current_state.mode;
        format!("{}: {}", hex.mode_name(), hex.mode_description())
    }


    pub(crate) fn _save_e8_state(&self, path: &std::path::Path) -> Result<(), String> {
        let json = self.e8_state_json()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
        }
        std::fs::write(path, &json).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    /// Serialize E8 persisted state to JSON string (KB TEXT 列存储)。
    pub fn e8_state_json(&self) -> Result<String, String> {
        let (e8_policy, prm_learning_count, prm_score_history) =
            self.prm.as_ref().map_or((None, 0u64, Vec::new()), |prm| {
                let history: Vec<f64> = prm.score_history.iter().rev().take(100).cloned().collect();
                (Some(prm.policy.clone()), prm.learning_count, history)
            });
        let state = E8PersistedState {
            current_mode: self.current_state.mode.0,
            current_meta: self.current_state.meta.0,
            last_e8_attention_weights: self.last_e8_attention_weights.clone(),
            last_e8_confidence: self.last_e8_confidence,
            trajectory_modes: self
                .state_trajectory
                .iter()
                .map(|s| (s.mode.0, s.meta.0))
                .collect(),
            e8_policy,
            prm_learning_count,
            prm_score_history,
        };
        serde_json::to_string_pretty(&state).map_err(|e| format!("serialize: {}", e))
    }

    pub(crate) fn _load_e8_state(&mut self, path: &std::path::Path) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        let json = std::fs::read_to_string(path).map_err(|e| format!("read: {}", e))?;
        self.load_e8_state_json(&json)
    }

    /// Deserialize E8 persisted state from JSON string (KB TEXT 列存储)。
    pub fn load_e8_state_json(&mut self, json: &str) -> Result<(), String> {
        let state: E8PersistedState =
            serde_json::from_str(json).map_err(|e| format!("deserialize: {}", e))?;
        self.current_state = FullReasoningState::new(
            ReasoningHexagram::new(state.current_mode.min(63)),
            crate::l5_cognition::nt_core_hex::MetaState::new(state.current_meta),
        );
        self.last_e8_attention_weights = state.last_e8_attention_weights;
        self.last_e8_confidence = state.last_e8_confidence;
        self.state_trajectory = state
            .trajectory_modes
            .into_iter()
            .map(|(mode, meta)| {
                FullReasoningState::new(
                    ReasoningHexagram::new(mode.min(63)),
                    crate::l5_cognition::nt_core_hex::MetaState::new(meta),
                )
            })
            .collect();
        if let Some(policy) = state.e8_policy {
            if let Some(ref mut prm) = self.prm {
                prm.policy = policy;
                prm.learning_count = state.prm_learning_count;
                prm.score_history = state.prm_score_history;
            }
        }
        Ok(())
    }

    /// 检索上下文 → 会话内缓存读写 (容量超限整体清空, 防膨胀)。
    pub(crate) fn kb_cache_get(&self, key: &str) -> Option<String> {
        self.kb_cache.lock().ok().and_then(|c| c.get(key).cloned())
    }
    pub(crate) fn kb_cache_put(&self, key: String, val: String) {
        if let Ok(mut cache) = self.kb_cache.lock() {
            if cache.len() >= MAX_KB_CACHE_ENTRIES {
                cache.clear();
            }
            cache.insert(key, val);
        }
    }

    /// P0-6: 会话内缓存 + 注入预算封顶的 KB 检索上下文。
    /// 命中 (query 前缀, 推理类型) 即直接复用, 避免同一 session 重复 kb.search;
    /// 未命中则检索并按相关度顺序截断到 ~MAX_KB_INJECTION_TOKENS。
    pub fn build_context(&self, query: &str, rtype: ReasoningType) -> String {
        let prefix: String = query.chars().take(64).collect();
        let key = format!("s:{}|{:?}", prefix, rtype);
        if let Some(cached) = self.kb_cache_get(&key) {
            return cached;
        }
        let ctx = self.build_context_uncached(query);
        self.kb_cache_put(key, ctx.clone());
        ctx
    }

    pub(crate) fn build_context_uncached(&self, query: &str) -> String {
        if let Some(ref kb) = self.kb {
            if let Ok(results) = kb.search(query, 3) {
                if !results.is_empty() {
                    // P2-E2: 相邻轮次去重 — 跳过上一轮已注入的 node (只注入增量)。
                    let mut injected: Vec<String> = Vec::new();
                    let last: std::collections::HashSet<String> = self
                        .last_kb_injected
                        .lock()
                        .map(|g| g.iter().cloned().collect())
                        .unwrap_or_default();
                    let mut filtered: Vec<&SearchResult> = results
                        .iter()
                        .filter(|r| !last.contains(&r.node.id))
                        .collect();
                    // 全部重复 → 至少保留最高相关度一条, 避免空注入
                    if filtered.is_empty() && !results.is_empty() {
                        filtered.push(&results[0]);
                    }
                    let mut ctx = format!("Past experiences relevant to \"{}\":\n", query);
                    let mut used = estimate_tokens(&ctx);
                    // results 已按相关度排序: 优先保留前面的高相关条目
                    for r in filtered {
                        let mut line = format!(
                            "- {}: {}\n",
                            r.node.title,
                            r.node.summary.as_deref().unwrap_or("(no summary)")
                        );
                        let line_tokens = estimate_tokens(&line);
                        if used + line_tokens > MAX_KB_INJECTION_TOKENS {
                            // 全行超限 → title/summary 各取头部再试; 仍超限则丢弃该行
                            let head_title = r.node.title.chars().take(64).collect::<String>();
                            let head_summary = r
                                .node
                                .summary
                                .as_deref()
                                .map(|s| s.chars().take(160).collect::<String>())
                                .unwrap_or_else(|| "(no summary)".to_string());
                            line = format!("- {}: {}\n", head_title, head_summary);
                            if used + estimate_tokens(&line) > MAX_KB_INJECTION_TOKENS {
                                break;
                            }
                        }
                        ctx.push_str(&line);
                        used += estimate_tokens(&line);
                        injected.push(r.node.id.clone());
                    }
                    *self
                        .last_kb_injected
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = injected;
                    return ctx;
                }
            }
        }
        format!("Reasoning task: {}", query)
    }

    pub fn build_artifact_context(&self, query: &str) -> String {
        if let Some(ref indexer) = self.artifact_indexer {
            let artifacts = indexer.store().search_keyword(query);
            if !artifacts.is_empty() {
                let mut ctx = String::from("Relevant artifacts:\n");
                for a in artifacts.iter().take(5) {
                    ctx.push_str(&format!(
                        "- {}: {} (tags: {:?})\n",
                        a.name,
                        a.content.chars().take(80).collect::<String>(),
                        a.tags
                    ));
                }
                return ctx;
            }
        }
        String::new()
    }

    /// P0-6: ContextBuilder/回退 E8 检索路径的会话内缓存 — 同一 task 前缀不重复检索。

    pub(crate) fn build_kb_context(&self, task: &str, root_span: &Span) -> String {
        let prefix: String = task.chars().take(64).collect();
        let key = format!("b:{}", prefix);
        if let Some(cached) = self.kb_cache_get(&key) {
            return cached;
        }
        let ctx = self.build_kb_context_uncached(task, root_span);
        self.kb_cache_put(key, ctx.clone());
        ctx
    }


    pub(crate) fn build_kb_context_uncached(&self, _task: &str, _root_span: &Span) -> String {
        if let Some(ref kb) = self.kb {
            // 回退到原有 E8 状态检索
            if let Ok(results) = kb.query_by_e8_state(self.current_state.mode, 5) {
                if !results.is_empty() {
                    let mut s = String::from("KB knowledge:\n");
                    for r in &results {
                        s.push_str(&format!("- {} (score: {:.2})\n", r.node.title, r.score));
                    }
                    s
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}

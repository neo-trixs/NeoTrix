//! # nt_free_pool — 池免费模型的智能调用
//!
//! 晶体要的不是"一个模型"，而是"打不死的免费模型池"。本模块把池子
//! （`UnifiedModelPool::free_models`，经 `CliFreeSource` 发现更新）
//! 包成晶体的同步 `NtLlmAsk`：
//!
//! ```text
//! NtCrystalTaskLoop ──▶ NtFreePoolAsk::ask(prompt)
//!                          │  轮转起点 + 跳过冷却中模型
//!                          ▼
//!                       逐个 NtModelCliAsk 调用 → 首个成功即返回（游标前移）
//!                          │  失败记冷却（默认 60s），全灭才报错
//! ```
//!
//! 智能点：轮转摊薄（不对同一免费档打爆限额）+ 故障转移（挂一个换下一个）
//! + 冷却（刚挂的暂时不打扰）。延迟学习以后再加（先留 `note_latency` 钩子位）。
//!
//! # Safety
//! - `Mutex`  guard 游标/冷却（`NtLlmAsk: Send + Sync`），无 unsafe (R-P1)。
//! - 生产代码无 `unwrap/expect/panic`。

use crate::l1_action::nt_model_cli::NtModelCliAsk;
use crate::neotrix::nt_crystal_core::{NtLlmAsk, NtLlmReply, NtTaskFusionError};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 池智能调用桥。
pub struct NtFreePoolAsk {
    models: Vec<String>,
    command: String,
    workdir: Option<PathBuf>,
    timeout: Duration,
    base_confidence: f64,
    cooldown: Duration,
    cursor: Mutex<usize>,
    cooled: Mutex<HashMap<String, Instant>>,
    /// TUI 模型选择器定点（None = 轮转；定点失败自动回落轮转）。
    pinned: Mutex<Option<String>>,
}

impl NtFreePoolAsk {
    pub fn new(models: Vec<String>) -> Self {
        Self {
            models,
            command: "opencode".to_string(),
            workdir: None,
            timeout: Duration::from_secs(300),
            base_confidence: 0.65,
            cooldown: Duration::from_secs(60),
            cursor: Mutex::new(0),
            cooled: Mutex::new(HashMap::new()),
            pinned: Mutex::new(None),
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = command.into();
        self
    }

    pub fn with_workdir(mut self, dir: PathBuf) -> Self {
        self.workdir = Some(dir);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown = cooldown;
        self
    }

    pub fn model_ids(&self) -> &[String] {
        &self.models
    }

    /// 定点模型（TUI 选择器调用；None 回轮转）。
    pub fn set_pinned(&self, model: Option<String>) {
        if let Ok(mut p) = self.pinned.lock() {
            *p = model;
        }
    }

    pub fn pinned(&self) -> Option<String> {
        self.pinned.lock().ok().and_then(|p| p.clone())
    }

    fn ask_one(&self, model: &str, prompt: &str) -> Result<NtLlmReply, NtTaskFusionError> {
        self.build_one(model).ask(prompt)
    }

    fn ask_one_stream(
        &self,
        model: &str,
        prompt: &str,
        on_chunk: &dyn Fn(&str) -> bool,
    ) -> Result<NtLlmReply, NtTaskFusionError> {
        self.build_one(model).ask_stream(prompt, on_chunk)
    }

    fn build_one(&self, model: &str) -> NtModelCliAsk {
        let mut ask = NtModelCliAsk::new()
            .with_command(self.command.clone())
            .with_model(model)
            .with_timeout(self.timeout)
            .with_base_confidence(self.base_confidence);
        if let Some(dir) = &self.workdir {
            ask = ask.with_workdir(dir.clone());
        }
        ask
    }

    /// 调用顺序：定点优先（池内才有效），随后轮转；共享给 ask/ask_stream。
    fn order_seq(&self) -> (Vec<String>, usize) {
        if self.models.is_empty() {
            return (Vec::new(), 0);
        }
        let now = Instant::now();
        let (mut seq, start) = match (self.cursor.lock(), self.cooled.lock()) {
            (Ok(c), Ok(k)) => (order(&self.models, *c, &k, now, self.cooldown), *c),
            _ => (self.models.clone(), 0),
        };
        if let Some(pin) = self.pinned() {
            if self.models.iter().any(|m| m == &pin) {
                seq.retain(|m| m != &pin);
                seq.insert(0, pin);
            }
        }
        (seq, start)
    }

    fn note_success(&self, model: &str, start: usize, offset: usize) {
        if let (Ok(mut c), Ok(mut k)) = (self.cursor.lock(), self.cooled.lock()) {
            *c = (start + offset + 1) % self.models.len().max(1);
            k.remove(model);
        }
    }

    fn note_failure(&self, model: &str, e: NtTaskFusionError, errors: &mut Vec<String>) {
        if let Ok(mut k) = self.cooled.lock() {
            k.insert(model.to_string(), Instant::now());
        }
        errors.push(format!("{model}: {e}"));
    }
}

/// 纯函数：调用顺序——游标起轮转，跳过冷却中（全被冷却则全试）。
pub fn order(
    models: &[String],
    cursor: usize,
    cooled: &HashMap<String, Instant>,
    now: Instant,
    cooldown: Duration,
) -> Vec<String> {
    if models.is_empty() {
        return Vec::new();
    }
    let mut seq = Vec::with_capacity(models.len());
    for i in 0..models.len() {
        seq.push(models[(cursor + i) % models.len()].clone());
    }
    let fresh: Vec<String> = seq
        .iter()
        .filter(|m| {
            cooled
                .get(*m)
                .map(|t| now.duration_since(*t) >= cooldown)
                .unwrap_or(true)
        })
        .cloned()
        .collect();
    if fresh.is_empty() {
        seq
    } else {
        fresh
    }
}

impl NtLlmAsk for NtFreePoolAsk {
    fn ask(&self, prompt: &str) -> Result<NtLlmReply, NtTaskFusionError> {
        if self.models.is_empty() {
            return Err(NtTaskFusionError::Llm("free pool is empty".to_string()));
        }
        let (seq, start) = self.order_seq();
        let mut errors = Vec::new();
        for (offset, model) in seq.iter().enumerate() {
            match self.ask_one(model, prompt) {
                Ok(reply) => {
                    self.note_success(model, start, offset);
                    return Ok(reply);
                }
                Err(e) => self.note_failure(model, e, &mut errors),
            }
        }
        Err(NtTaskFusionError::Llm(format!(
            "all {} free models failed: {}",
            seq.len(),
            errors.join(" | ")
        )))
    }

    fn ask_stream(
        &self,
        prompt: &str,
        on_chunk: &dyn Fn(&str) -> bool,
    ) -> Result<NtLlmReply, NtTaskFusionError> {
        if self.models.is_empty() {
            return Err(NtTaskFusionError::Llm("free pool is empty".to_string()));
        }
        let (seq, start) = self.order_seq();
        let mut errors = Vec::new();
        for (offset, model) in seq.iter().enumerate() {
            match self.ask_one_stream(model, prompt, on_chunk) {
                Ok(reply) => {
                    self.note_success(model, start, offset);
                    return Ok(reply);
                }
                Err(e) => self.note_failure(model, e, &mut errors),
            }
        }
        Err(NtTaskFusionError::Llm(format!(
            "all {} free models failed: {}",
            seq.len(),
            errors.join(" | ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn models() -> Vec<String> {
        vec!["m1".to_string(), "m2".to_string(), "m3".to_string()]
    }

    #[test]
    fn test_order_rotates_from_cursor() {
        let got = order(&models(), 1, &HashMap::new(), Instant::now(), Duration::from_secs(60));
        assert_eq!(got, vec!["m2".to_string(), "m3".to_string(), "m1".to_string()]);
    }

    #[test]
    fn test_order_skips_cooled() {
        let mut cooled = HashMap::new();
        cooled.insert("m1".to_string(), Instant::now());
        let got = order(&models(), 0, &cooled, Instant::now(), Duration::from_secs(60));
        assert_eq!(got, vec!["m2".to_string(), "m3".to_string()]);
    }

    #[test]
    fn test_order_all_cooled_falls_back_to_all() {
        let now = Instant::now();
        let mut cooled = HashMap::new();
        for m in models() {
            cooled.insert(m, now);
        }
        let got = order(&models(), 0, &cooled, now, Duration::from_secs(60));
        assert_eq!(got.len(), 3);
    }

    #[test]
    fn test_order_expired_cooldown_retried() {
        let mut cooled = HashMap::new();
        cooled.insert(
            "m1".to_string(),
            Instant::now() - Duration::from_secs(120),
        );
        let got = order(&models(), 0, &cooled, Instant::now(), Duration::from_secs(60));
        assert!(got.contains(&"m1".to_string()));
    }

    #[test]
    fn test_ask_empty_pool_errors() {
        let pool = NtFreePoolAsk::new(vec![]);
        let err = pool.ask("hi").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_ask_all_fail_lists_models() {
        let pool = NtFreePoolAsk::new(vec!["mx".to_string(), "my".to_string()])
            .with_command("/nonexistent-nt-xyz");
        let err = pool.ask("hi").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("mx") && msg.contains("my"));
    }
}

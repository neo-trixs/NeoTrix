//! Production-chain segment: checkpoint/resume + MoneyPrinterTurbo production chain.

use serde::{Deserialize, Serialize};

// ── P2-Checkpoint: 视频链 checkpoint / resume (Claude Workflows 吸收) ──
// 阶段化视频生产链可中断/续跑: 每完成一个阶段落 checkpoint, 重跑从最近
// checkpoint 恢复, 避免 ffmpeg/转码/烧字幕这类重活重复执行。checkpoint
// 由 JSON 序列化 (纯 Rust, 零外部依赖)。

/// 视频生产链阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum _VideoStage {
    Extract,
    Transcode,
    Dedup,
    Subtitle,
    Publish,
}

impl _VideoStage {
    pub fn label(self) -> &'static str {
        match self {
            _VideoStage::Extract => "extract",
            _VideoStage::Transcode => "_transcode",
            _VideoStage::Dedup => "dedup",
            _VideoStage::Subtitle => "subtitle",
            _VideoStage::Publish => "publish",
        }
    }

    /// 阶段执行顺序索引 (供推进/恢复)。
    fn order(self) -> u8 {
        match self {
            _VideoStage::Extract => 0,
            _VideoStage::Transcode => 1,
            _VideoStage::Dedup => 2,
            _VideoStage::Subtitle => 3,
            _VideoStage::Publish => 4,
        }
    }

    /// 顺序中的下一个阶段。
    pub fn next(self) -> Option<_VideoStage> {
        match self {
            _VideoStage::Extract => Some(_VideoStage::Transcode),
            _VideoStage::Transcode => Some(_VideoStage::Dedup),
            _VideoStage::Dedup => Some(_VideoStage::Subtitle),
            _VideoStage::Subtitle => Some(_VideoStage::Publish),
            _VideoStage::Publish => None,
        }
    }
}

/// 单个阶段的 checkpoint 状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCheckpoint {
    pub stage: _VideoStage,
    /// 该阶段是否已完成。
    pub done: bool,
    /// 完成时的时间戳 (unix 秒, 0 = 未完成)。
    pub finished_at: i64,
    /// 阶段产物摘要 (如输出路径/帧数)。
    pub artifact: String,
}

/// 视频链 checkpoint 集合 — 序列化为 JSON 支持持久化/恢复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoChainCheckpoint {
    /// 关联视频源。
    pub source: String,
    /// 各阶段状态 (按顺序)。
    pub stages: Vec<StageCheckpoint>,
    /// 已消耗的 token 预算 (供成本追踪)。
    pub budget_used: u64,
}

impl VideoChainCheckpoint {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            stages: vec![
                StageCheckpoint { stage: _VideoStage::Extract, done: false, finished_at: 0, artifact: String::new() },
                StageCheckpoint { stage: _VideoStage::Transcode, done: false, finished_at: 0, artifact: String::new() },
                StageCheckpoint { stage: _VideoStage::Dedup, done: false, finished_at: 0, artifact: String::new() },
                StageCheckpoint { stage: _VideoStage::Subtitle, done: false, finished_at: 0, artifact: String::new() },
                StageCheckpoint { stage: _VideoStage::Publish, done: false, finished_at: 0, artifact: String::new() },
            ],
            budget_used: 0,
        }
    }

    /// 标记某阶段完成并记录产物。
    pub fn complete(&mut self, stage: _VideoStage, artifact: impl Into<String>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        if let Some(s) = self.stages.iter_mut().find(|s| s.stage == stage) {
            s.done = true;
            s.finished_at = now;
            s.artifact = artifact.into();
        }
    }

    /// 查询阶段是否已完成。
    pub fn _is_done(&self, stage: _VideoStage) -> bool {
        self.stages.iter().find(|s| s.stage == stage).map(|s| s.done).unwrap_or(false)
    }

    /// 下一个未完成阶段 (从 Extract 顺序推进) — resume 起点。
    pub fn next_pending(&self) -> Option<_VideoStage> {
        self.stages.iter().find(|s| !s.done).map(|s| s.stage)
    }

    /// 是否全部完成。
    pub fn all_done(&self) -> bool {
        self.stages.iter().all(|s| s.done)
    }

    /// 序列化为 JSON (持久化)。
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("serialize checkpoint: {}", e))
    }

    /// 从 JSON 恢复。
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("deserialize checkpoint: {}", e))
    }

    /// 记录预算消耗。
    pub fn _spend_budget(&mut self, tokens: u64) {
        self.budget_used = self.budget_used.saturating_add(tokens);
    }
}

/// 视频链执行器 — 按阶段推进, 支持中断后从 checkpoint 恢复。
#[derive(Debug, Clone)]
pub struct VideoChainRunner {
    pub checkpoint: VideoChainCheckpoint,
    /// 每阶段最大 token 预算 (0 = 无限)。
    pub stage_budget: u64,
    /// 已执行阶段计数。
    pub executed: u64,
}

impl VideoChainRunner {
    pub fn new(source: &str) -> Self {
        Self {
            checkpoint: VideoChainCheckpoint::new(source),
            stage_budget: 0,
            executed: 0,
        }
    }

    /// 从最近 checkpoint 恢复: 返回下一个待执行阶段 (None = 已全部完成)。
    pub fn resume(&self) -> Option<_VideoStage> {
        self.checkpoint.next_pending()
    }

    /// 执行单个阶段 (玩具: 以 stage_budget 为 token 消耗模拟重活)。
    /// 若阶段已 done → 跳过 (resume 语义)。
    pub fn run_stage(&mut self, stage: _VideoStage) -> Result<StageCheckpoint, String> {
        if self.checkpoint._is_done(stage) {
            // 已完成的阶段跳过 — 不重复执行
            return self
                .checkpoint
                .stages
                .iter()
                .find(|s| s.stage == stage)
                .cloned()
                .ok_or_else(|| "stage not found".into());
        }
        let cost = 1 + stage.order() as u64;
        if self.stage_budget > 0 && cost > self.stage_budget {
            return Err(format!("stage {} exceeds per-stage budget {}", stage.label(), self.stage_budget));
        }
        let artifact = format!("{}-output", stage.label());
        self.checkpoint.complete(stage, &artifact);
        self.checkpoint._spend_budget(cost);
        self.executed += 1;
        self.checkpoint
            .stages
            .iter()
            .find(|s| s.stage == stage)
            .cloned()
            .ok_or_else(|| "stage not found".into())
    }

    /// 全链执行: 从 resume 起点顺序推进到 Publish (或首个预算失败处)。
    /// 返回已完成阶段列表。
    pub fn run_all(&mut self) -> Result<Vec<_VideoStage>, String> {
        let mut done = Vec::new();
        let mut current = self.resume();
        while let Some(stage) = current {
            match self.run_stage(stage) {
                Ok(_) => {
                    done.push(stage);
                    current = stage.next();
                }
                Err(e) => return Err(e),
            }
        }
        Ok(done)
    }
}

// ──────────────────────────────────────────────
// G19 视频生产全链 (MoneyPrinterTurbo 吸收) —
// 脚本 → 素材检索 → TTS 字幕 → 合成 → 发布
// ──────────────────────────────────────────────

/// MoneyPrinterTurbo 生产链阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionStage {
    Script,
    Material,
    Tts,
    Compose,
    Publish,
}

impl ProductionStage {
    pub fn label(self) -> &'static str {
        match self {
            ProductionStage::Script => "script",
            ProductionStage::Material => "material",
            ProductionStage::Tts => "tts",
            ProductionStage::Compose => "compose",
            ProductionStage::Publish => "publish",
        }
    }

    fn order(self) -> u8 {
        match self {
            ProductionStage::Script => 0,
            ProductionStage::Material => 1,
            ProductionStage::Tts => 2,
            ProductionStage::Compose => 3,
            ProductionStage::Publish => 4,
        }
    }

    pub fn next(self) -> Option<ProductionStage> {
        match self {
            ProductionStage::Script => Some(ProductionStage::Material),
            ProductionStage::Material => Some(ProductionStage::Tts),
            ProductionStage::Tts => Some(ProductionStage::Compose),
            ProductionStage::Compose => Some(ProductionStage::Publish),
            ProductionStage::Publish => None,
        }
    }
}

/// 生产链阶段产物。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ProductionArtifact {
    pub stage: ProductionStage,
    pub done: bool,
    pub artifact: String,
}

/// 视频生产链 — MoneyPrinterTurbo 全链, 支持 checkpoint 恢复。
#[derive(Debug, Clone)]
pub struct VideoProductionChain {
    pub topic: String,
    pub stages: Vec<_ProductionArtifact>,
    /// 已消耗 token 预算。
    pub budget_used: u64,
    /// 已执行阶段计数。
    pub executed: u64,
}

impl VideoProductionChain {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_string(),
            stages: vec![
                _ProductionArtifact { stage: ProductionStage::Script, done: false, artifact: String::new() },
                _ProductionArtifact { stage: ProductionStage::Material, done: false, artifact: String::new() },
                _ProductionArtifact { stage: ProductionStage::Tts, done: false, artifact: String::new() },
                _ProductionArtifact { stage: ProductionStage::Compose, done: false, artifact: String::new() },
                _ProductionArtifact { stage: ProductionStage::Publish, done: false, artifact: String::new() },
            ],
            budget_used: 0,
            executed: 0,
        }
    }

    /// 下一个未完成阶段 (resume 起点)。
    pub fn next_pending(&self) -> Option<ProductionStage> {
        self.stages.iter().find(|s| !s.done).map(|s| s.stage)
    }

    pub fn all_done(&self) -> bool {
        self.stages.iter().all(|s| s.done)
    }

    /// 执行单个阶段 (脚本为模板生成, 其余为确定性模拟重活)。
    pub fn run_stage(&mut self, stage: ProductionStage) -> Result<_ProductionArtifact, String> {
        if self.stages.iter().any(|s| s.stage == stage && s.done) {
            return self
                .stages
                .iter()
                .find(|s| s.stage == stage)
                .cloned()
                .ok_or_else(|| "stage not found".into());
        }
        let artifact = match stage {
            ProductionStage::Script => format!("script-{}", slugify(&self.topic)),
            ProductionStage::Material => format!("materials/{}-assets", slugify(&self.topic)),
            ProductionStage::Tts => format!("tts/{}-subtitles.srt", slugify(&self.topic)),
            ProductionStage::Compose => format!("out/{}-final.mp4", slugify(&self.topic)),
            ProductionStage::Publish => format!("published/{}-final.mp4", slugify(&self.topic)),
        };
        if let Some(s) = self.stages.iter_mut().find(|s| s.stage == stage) {
            s.done = true;
            s.artifact = artifact.clone();
        }
        self.budget_used = self.budget_used.saturating_add(1 + stage.order() as u64);
        self.executed += 1;
        Ok(_ProductionArtifact { stage, done: true, artifact })
    }

    /// 全链推进到 Publish。
    pub fn run_all(&mut self) -> Result<Vec<ProductionStage>, String> {
        let mut done = Vec::new();
        let mut current = self.next_pending();
        while let Some(stage) = current {
            self.run_stage(stage)?;
            done.push(stage);
            current = stage.next();
        }
        Ok(done)
    }

    /// 产物摘要 (供入库描述)。
    pub fn _output_manifest(&self) -> Vec<(ProductionStage, String)> {
        self.stages.iter().map(|s| (s.stage, s.artifact.clone())).collect()
    }
}

fn slugify(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .map(|c| c.to_ascii_lowercase())
        .collect::<String>()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_fresh_starts_at_extract() {
        let cp = VideoChainCheckpoint::new("video.mp4");
        assert_eq!(cp.next_pending(), Some(_VideoStage::Extract));
        assert!(!cp.all_done());
    }

    #[test]
    fn checkpoint_complete_advances_pending() {
        let mut cp = VideoChainCheckpoint::new("v.mp4");
        cp.complete(_VideoStage::Extract, "frames/");
        assert!(cp._is_done(_VideoStage::Extract));
        assert_eq!(cp.next_pending(), Some(_VideoStage::Transcode));
    }

    #[test]
    fn checkpoint_all_done_returns_none_pending() {
        let mut cp = VideoChainCheckpoint::new("v.mp4");
        for stage in [_VideoStage::Extract, _VideoStage::Transcode, _VideoStage::Dedup, _VideoStage::Subtitle, _VideoStage::Publish] {
            cp.complete(stage, "ok");
        }
        assert!(cp.all_done());
        assert_eq!(cp.next_pending(), None);
    }

    #[test]
    fn checkpoint_json_roundtrip() {
        let mut cp = VideoChainCheckpoint::new("v.mp4");
        cp.complete(_VideoStage::Extract, "frames/");
        cp._spend_budget(42);
        let json = cp.to_json().unwrap();
        let back = VideoChainCheckpoint::from_json(&json).unwrap();
        assert_eq!(back.source, "v.mp4");
        assert!(back._is_done(_VideoStage::Extract));
        assert!(!back._is_done(_VideoStage::Transcode));
        assert_eq!(back.budget_used, 42);
    }

    #[test]
    fn runner_skips_completed_stages_on_resume() {
        // 模拟中断: Extract+Transcode 已完成 → resume 从 Dedup 开始。
        let mut runner = VideoChainRunner::new("v.mp4");
        runner.checkpoint.complete(_VideoStage::Extract, "f/");
        runner.checkpoint.complete(_VideoStage::Transcode, "t/");
        assert_eq!(runner.resume(), Some(_VideoStage::Dedup));
        runner.run_stage(_VideoStage::Extract).unwrap(); // 已完成 → 跳过, 不重执行
        assert_eq!(runner.executed, 0, "completed stage must not re-execute");
        runner.run_stage(_VideoStage::Dedup).unwrap();
        assert_eq!(runner.executed, 1);
    }

    #[test]
    fn runner_run_all_completes_chain() {
        let mut runner = VideoChainRunner::new("v.mp4");
        let done = runner.run_all().unwrap();
        assert_eq!(done.len(), 5);
        assert!(runner.checkpoint.all_done());
        assert_eq!(runner.checkpoint.budget_used, 1 + 2 + 3 + 4 + 5);
    }

    #[test]
    fn runner_budget_blocks_expensive_stage() {
        let mut runner = VideoChainRunner::new("v.mp4");
        runner.stage_budget = 2; // Dedup(order2) 需 cost 3 → 超限
        let err = runner.run_all().unwrap_err();
        assert!(err.contains("dedup"), "err: {err}");
        // Extract+Transcode 已落 checkpoint, 后续未执行
        assert!(runner.checkpoint._is_done(_VideoStage::Extract));
        assert!(runner.checkpoint._is_done(_VideoStage::Transcode));
        assert!(!runner.checkpoint._is_done(_VideoStage::Dedup));
    }

    #[test]
    fn runner_persist_and_recover() {
        let mut runner = VideoChainRunner::new("v.mp4");
        runner.run_stage(_VideoStage::Extract).unwrap();
        runner.run_stage(_VideoStage::Transcode).unwrap();
        let json = runner.checkpoint.to_json().unwrap();
        drop(runner);

        let mut recovered = VideoChainRunner::new("v.mp4");
        recovered.checkpoint = VideoChainCheckpoint::from_json(&json).unwrap();
        assert_eq!(recovered.resume(), Some(_VideoStage::Dedup));
        let done = recovered.run_all().unwrap();
        assert_eq!(done, vec![_VideoStage::Dedup, _VideoStage::Subtitle, _VideoStage::Publish]);
        assert!(recovered.checkpoint.all_done());
    }

    #[test]
    fn stage_order_and_next() {
        assert_eq!(_VideoStage::Extract.next(), Some(_VideoStage::Transcode));
        assert_eq!(_VideoStage::Subtitle.next(), Some(_VideoStage::Publish));
        assert_eq!(_VideoStage::Publish.next(), None);
        assert_eq!(_VideoStage::Extract.order(), 0);
        assert_eq!(_VideoStage::Publish.order(), 4);
    }

    #[test]
    fn production_chain_full_run_manifest() {
        let mut chain = VideoProductionChain::new("Rust Async Deep Dive");
        assert_eq!(chain.next_pending(), Some(ProductionStage::Script));
        let done = chain.run_all().unwrap();
        assert_eq!(done.len(), 5, "全链 5 阶段");
        assert!(chain.all_done());
        let manifest = chain._output_manifest();
        assert!(manifest.iter().any(|(s, a)| *s == ProductionStage::Compose && a.contains("final.mp4")));
        assert!(chain.executed >= 5);
    }

    #[test]
    fn production_chain_resume_from_partial() {
        let mut chain = VideoProductionChain::new("T");
        chain.run_stage(ProductionStage::Script).unwrap();
        chain.run_stage(ProductionStage::Material).unwrap();
        assert_eq!(chain.next_pending(), Some(ProductionStage::Tts), "从 TTS 恢复");
        let remaining = chain.run_all().unwrap();
        assert_eq!(remaining, vec![ProductionStage::Tts, ProductionStage::Compose, ProductionStage::Publish]);
    }

    #[test]
    fn production_chain_skips_completed_stage() {
        let mut chain = VideoProductionChain::new("T");
        chain.run_stage(ProductionStage::Script).unwrap();
        let before = chain.executed;
        chain.run_stage(ProductionStage::Script).unwrap(); // 已完成 → 跳过
        assert_eq!(chain.executed, before, "不重复执行");
    }
}

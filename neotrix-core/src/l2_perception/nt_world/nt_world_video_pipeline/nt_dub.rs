//! Dub segment: end-to-end translate+dub chain.

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────
// P8 translate_dub — 端到端视频翻译+配音链 (KrillinAI 吸收)
// 语音转写→翻译→对齐→TTS 配音→合并音轨; 模块化链式处理, 多语种。
// ──────────────────────────────────────────────────────────────

/// 配音链单个阶段 (order 决定执行顺序)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct _DubStage {
    pub order: u8,
    pub name: String,
    pub language: String,
}

/// 待翻译的时间段 — 语音转写/翻译/配音对齐的最小单元。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct _TranslationSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub source: String,
    pub translated: String,
}

/// 配音作业 — 源/目标语言 + 阶段链 + 待处理时间段。
#[derive(Debug, Clone)]
pub struct _DubJob {
    pub source_lang: String,
    pub target_lang: String,
    pub stages: Vec<_DubStage>,
    pub segments: Vec<_TranslationSegment>,
}

impl Default for _DubJob {
    fn default() -> Self {
        Self {
            source_lang: "zh".into(),
            target_lang: "en".into(),
            stages: vec![
                _DubStage { order: 0, name: "transcribe".into(), language: "zh".into() },
                _DubStage { order: 1, name: "translate".into(), language: "en".into() },
                _DubStage { order: 2, name: "align".into(), language: "en".into() },
                _DubStage { order: 3, name: "tts".into(), language: "en".into() },
                _DubStage { order: 4, name: "merge".into(), language: "en".into() },
            ],
            segments: Vec::new(),
        }
    }
}

/// 配音流水线 — 按 `enabled_stages` 逐段驱动整个翻译+配音链。
#[derive(Debug, Clone)]
pub struct _DubPipeline {
    pub enabled_stages: Vec<String>,
    pub max_segment_ms: u64,
    pub overlap_ms: u64,
}

impl Default for _DubPipeline {
    fn default() -> Self {
        Self {
            enabled_stages: vec![
                "transcribe".into(),
                "translate".into(),
                "align".into(),
                "tts".into(),
                "merge".into(),
            ],
            max_segment_ms: 10000,
            overlap_ms: 250,
        }
    }
}

impl _DubPipeline {
    /// 内置 5 阶段 (顺序即执行顺序)。
    const BUILTIN_STAGES: [&'static str; 5] =
        ["transcribe", "translate", "align", "tts", "merge"];

    /// 创建流水线 (仅启用指定阶段)。
    pub fn new(enabled_stages: Vec<String>) -> Self {
        Self {
            enabled_stages,
            max_segment_ms: 10000,
            overlap_ms: 250,
        }
    }

    /// 逐阶段逐段生成动作描述; 遇到非内置阶段即中止报错。
    pub fn run(&self, job: &mut _DubJob) -> Result<Vec<String>, String> {
        let mut actions = Vec::new();
        for stage in &self.enabled_stages {
            if !Self::BUILTIN_STAGES.contains(&stage.as_str()) {
                return Err(format!("unknown stage: {}", stage));
            }
            for (i, seg) in job.segments.iter().enumerate() {
                actions.push(format!(
                    "{} seg {} ({}-{}ms)",
                    stage, i, seg.start_ms, seg.end_ms
                ));
            }
        }
        Ok(actions)
    }

    /// 翻译单段: 在译文前加目标语言标记 (确定性玩具实现)。
    pub fn _translate_segment(&self, seg: &_TranslationSegment, target: &str) -> _TranslationSegment {
        _TranslationSegment {
            start_ms: seg.start_ms,
            end_ms: seg.end_ms,
            source: seg.source.clone(),
            translated: format!("[{}] {}", target, seg.source),
        }
    }

    /// 估算配音总时长 = 各段 (end_ms - start_ms) 之和。
    pub fn _estimate_dub_duration(&self, job: &_DubJob) -> u64 {
        job.segments
            .iter()
            .map(|s| s.end_ms.saturating_sub(s.start_ms))
            .sum()
    }
}

/// SelfTest (T1): "nt_world_video_pipeline_dub" — 配音链自检。
impl crate::l0_substrate::nt_core_self_test::SelfTest for _DubPipeline {
    fn name(&self) -> &str {
        "nt_world_video_pipeline_dub"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut job = _DubJob::default();
        job.segments.push(_TranslationSegment {
            start_ms: 0,
            end_ms: 10000,
            source: "hello".into(),
            translated: String::new(),
        });
        match self.run(&mut job) {
            Ok(actions) => {
                if actions.len() != 5 {
                    failures.push("default run should emit 5 actions".into());
                }
            }
            Err(e) => failures.push(format!("default run failed: {}", e)),
        }
        let bad = _DubPipeline::new(vec!["bogus".into()]);
        if bad.run(&mut job).is_ok() {
            failures.push("unknown stage must error".into());
        }
        let t = self._translate_segment(&job.segments[0], "en");
        if t.translated != "[en] hello" {
            failures.push("_translate_segment must tag target language".into());
        }
        if self._estimate_dub_duration(&job) != 10000 {
            failures.push("duration estimate mismatch".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod dub_pipeline_tests {
    use super::*;

    fn sample_job() -> _DubJob {
        let mut job = _DubJob::default();
        job.segments.push(_TranslationSegment {
            start_ms: 0,
            end_ms: 10000,
            source: "hello world".into(),
            translated: String::new(),
        });
        job.segments.push(_TranslationSegment {
            start_ms: 10000,
            end_ms: 15000,
            source: "second line".into(),
            translated: String::new(),
        });
        job
    }

    #[test]
    fn dub_pipeline_default_has_five_stages() {
        let p = _DubPipeline::default();
        assert_eq!(
            p.enabled_stages,
            vec!["transcribe", "translate", "align", "tts", "merge"]
        );
        assert_eq!(p.max_segment_ms, 10000);
        assert_eq!(p.overlap_ms, 250);
        let job = _DubJob::default();
        assert_eq!(job.stages.len(), 5);
        assert_eq!(job.stages[0].name, "transcribe");
        assert_eq!(job.stages[4].name, "merge");
        assert_eq!(job.stages[0].order, 0);
        assert_eq!(job.stages[4].order, 4);
    }

    #[test]
    fn dub_pipeline_unknown_stage_errors() {
        let p = _DubPipeline::new(vec!["transcribe".into(), "mix".into()]);
        let mut job = sample_job();
        let err = p.run(&mut job).unwrap_err();
        assert!(err.contains("unknown stage: mix"), "err: {}", err);
    }

    #[test]
    fn dub_pipeline_translate_segment_marks_target_language() {
        let p = _DubPipeline::default();
        let seg = _TranslationSegment {
            start_ms: 0,
            end_ms: 5000,
            source: "你好".into(),
            translated: String::new(),
        };
        let out = p._translate_segment(&seg, "en");
        assert_eq!(out.translated, "[en] 你好");
        assert_eq!(out.start_ms, seg.start_ms);
        assert_eq!(out.end_ms, seg.end_ms);
        assert_eq!(out.source, seg.source);
    }

    #[test]
    fn dub_pipeline_duration_estimate_sums_segments() {
        let p = _DubPipeline::default();
        let mut job = sample_job();
        assert_eq!(p._estimate_dub_duration(&job), 15000);
        job.segments.clear();
        assert_eq!(p._estimate_dub_duration(&job), 0);
    }

    #[test]
    fn dub_pipeline_run_covers_all_enabled_stages() {
        let p = _DubPipeline::new(vec!["transcribe".into(), "tts".into()]);
        let mut job = sample_job();
        let actions = p.run(&mut job).unwrap();
        assert_eq!(actions.len(), 4, "2 stages x 2 segments");
        assert_eq!(actions[0], "transcribe seg 0 (0-10000ms)");
        assert_eq!(actions[1], "transcribe seg 1 (10000-15000ms)");
        assert_eq!(actions[2], "tts seg 0 (0-10000ms)");
        assert_eq!(actions[3], "tts seg 1 (10000-15000ms)");
        for stage in ["transcribe", "tts"] {
            assert!(actions.iter().any(|a| a.starts_with(stage)));
        }
    }

    #[test]
    fn dub_pipeline_run_empty_segments_yields_no_actions() {
        let p = _DubPipeline::default();
        let mut job = _DubJob::default();
        let actions = p.run(&mut job).unwrap();
        assert!(actions.is_empty());
    }

    #[test]
    fn dub_pipeline_selftest_matches() {
        use crate::l0_substrate::nt_core_self_test::SelfTest;
        let p = _DubPipeline::default();
        assert_eq!(p.name(), "nt_world_video_pipeline_dub");
        assert!(p.self_test().is_ok());
    }
}

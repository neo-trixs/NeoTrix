//! Orchestrator segment: asset enrichment + unified [`VideoOrchestrator`].

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::nt_extract::{
    _ExtractionPipeline, _PipelineOutput, _VideoPipeline, _VideoSummary, _process_video,
};
use super::nt_transcode::TranscodeConfig;
use super::nt_production_chain::{ProductionStage, VideoProductionChain};

// ──────────────────────────────────────────────
// G20 资产 ML 富化 (immich 吸收) —
// 资产去重 + 语义标签 (CLIP 式轻量指纹)
// ──────────────────────────────────────────────

/// 单个媒体资产。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _MediaAsset {
    pub id: String,
    pub path: String,
    /// 内容指纹 (如感知哈希 / 帧差分签名)。
    pub fingerprint: String,
    /// 已有标签。
    pub tags: Vec<String>,
}

/// immich 风格资产富化结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _AssetEnrichment {
    pub asset_id: String,
    /// 判定为重复的资产 id (去重)。
    pub dedup_of: Option<String>,
    /// 语义标签 (CLIP 式: 从路径/指纹推导的主题词)。
    pub semantic_tags: Vec<String>,
    /// 是否需纳入产出清单。
    pub keep: bool,
}

/// 资产富化器 — immich 吸收: 内容去重 + 语义标签。
#[derive(Debug, Clone, Default)]
pub struct _AssetEnricher {
    pub seen: HashMap<String, String>,
    pub enriched: Vec<_AssetEnrichment>,
}

impl _AssetEnricher {
    pub fn new() -> Self {
        Self::default()
    }

    /// 富化单个资产: 指纹重复 → 标记去重; 否则提取语义标签。
    pub fn _enrich(&mut self, asset: &_MediaAsset) -> _AssetEnrichment {
        let mut sem = asset.tags.clone();
        // 从路径/文件名提取主题词 (轻量语义标签)
        for token in asset.path.split(['/', '\\', '.', '_', '-']) {
            let t = token.trim();
            if t.len() >= 4 && t.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let t = t.to_lowercase();
                if !sem.contains(&t) {
                    sem.push(t);
                }
            }
        }
        let result = if let Some(orig) = self.seen.get(&asset.fingerprint) {
            _AssetEnrichment {
                asset_id: asset.id.clone(),
                dedup_of: Some(orig.clone()),
                semantic_tags: sem,
                keep: false,
            }
        } else {
            self.seen.insert(asset.fingerprint.clone(), asset.id.clone());
            _AssetEnrichment {
                asset_id: asset.id.clone(),
                dedup_of: None,
                semantic_tags: sem,
                keep: true,
            }
        };
        self.enriched.push(result.clone());
        result
    }

    /// 去重统计: (总资产, 判定重复数, 保留数)。
    pub fn stats(&self) -> (usize, usize, usize) {
        let total = self.enriched.len();
        let dup = self.enriched.iter().filter(|e| e.dedup_of.is_some()).count();
        (total, dup, total - dup)
    }
}

// ──────────────────────────────────────────────
// NEW: VideoOrchestrator — unifies both pipelines
// ──────────────────────────────────────────────

/// Unified orchestrator wrapping both frame-level [`_VideoPipeline`]
/// and web-based [`_ExtractionPipeline`].
pub struct VideoOrchestrator {
    pub frame_pipeline: _VideoPipeline,
    pub extraction_pipeline: _ExtractionPipeline,
    total_videos_processed: u64,
    transcode_config: TranscodeConfig,
    /// G19 视频生产全链 (MoneyPrinterTurbo 吸收)。
    pub production_chain: Option<VideoProductionChain>,
    /// G20 资产 ML 富化 (immich 吸收)。
    pub asset_enricher: _AssetEnricher,
}

impl VideoOrchestrator {
    pub fn new(transcode_config: TranscodeConfig) -> Self {
        Self {
            frame_pipeline: _VideoPipeline::new(),
            extraction_pipeline: _ExtractionPipeline::new(transcode_config.clone()),
            total_videos_processed: 0,
            transcode_config,
            production_chain: None,
            asset_enricher: _AssetEnricher::new(),
        }
    }

    /// G19 运行视频生产全链 (脚本→素材→TTS→合成→发布), 产物入库存档。
    pub fn produce_video(&mut self, topic: &str) -> Result<Vec<(ProductionStage, String)>, String> {
        let mut chain = VideoProductionChain::new(topic);
        chain.run_all()?;
        let manifest = chain._output_manifest();
        self.total_videos_processed += 1;
        self.production_chain = Some(chain);
        Ok(manifest)
    }

    /// G20 富化一批素材 (去重 + 语义标签)。
    pub fn _enrich_assets(&mut self, assets: &[_MediaAsset]) -> Vec<_AssetEnrichment> {
        assets.iter().map(|a| self.asset_enricher._enrich(a)).collect()
    }

    pub fn _asset_enrichment_stats(&self) -> (usize, usize, usize) {
        self.asset_enricher.stats()
    }

    pub fn _process_video_file(&mut self, path: &str) -> Result<_VideoSummary, String> {
        let result = _process_video(path)?;
        self.total_videos_processed += 1;
        Ok(result)
    }

    pub fn _extract_from_web(&mut self, url: &str) -> Result<_PipelineOutput, String> {
        let result = self.extraction_pipeline.run(url);
        if result.is_ok() {
            self.total_videos_processed += 1;
        }
        result
    }

    pub fn total_processed(&self) -> u64 {
        self.total_videos_processed
    }

    pub fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        if self.transcode_config.target_bitrate_kbps == 0 {
            failures.push("_transcode config has zero bitrate".into());
        }
        if self.transcode_config.target_width == 0 || self.transcode_config.target_height == 0 {
            failures.push("_transcode config has zero dimensions".into());
        }
        if self.extraction_pipeline.is_active() {
            failures.push("extraction pipeline stuck in active state".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use super::super::nt_transcode::{VideoCodec, _StreamProtocol};

    #[test]
    fn test_orchestrator_unified() {
        let config = TranscodeConfig::default();
        let mut orchestrator = VideoOrchestrator::new(config);

        assert!(orchestrator.self_test().is_ok());
        assert_eq!(orchestrator.total_processed(), 0);

        let extract_result = orchestrator._extract_from_web("https://example.com/video");
        assert!(extract_result.is_ok());
        assert_eq!(orchestrator.total_processed(), 1);

        let file_result = orchestrator._process_video_file("/nonexistent/video.mp4");
        assert!(file_result.is_err());
        assert_eq!(orchestrator.total_processed(), 1);

        let output = extract_result.unwrap();
        assert!(!output.streams.is_empty());
        assert_eq!(output.streams[0].0.protocol, _StreamProtocol::HLS);
        assert_eq!(output.streams[0].1.actual_codec, VideoCodec::H264);
    }

    #[test]
    fn orchestrator_produce_and_enrich_wired() {
        let cfg = TranscodeConfig {
            target_bitrate_kbps: 4000,
            target_width: 1920,
            target_height: 1080,
            ..TranscodeConfig::default()
        };
        let mut orch = VideoOrchestrator::new(cfg);
        let manifest = orch.produce_video("Consciousness Engineering").unwrap();
        assert_eq!(manifest.len(), 5);
        let enriched = orch._enrich_assets(&[
            _MediaAsset { id: "x".into(), path: "a/b_shot.png".into(), fingerprint: "F".into(), tags: vec![] },
            _MediaAsset { id: "y".into(), path: "a/b_shot.png".into(), fingerprint: "F".into(), tags: vec![] },
        ]);
        assert_eq!(enriched[1].dedup_of.as_deref(), Some("x"));
        assert_eq!(orch.total_processed(), 1);
        let (_, dup, _) = orch._asset_enrichment_stats();
        assert_eq!(dup, 1);
    }

    #[test]
    fn asset_enricher_dedups_by_fingerprint() {
        let mut enr = _AssetEnricher::new();
        let a = _MediaAsset { id: "a1".into(), path: "cats/happy_cat.png".into(), fingerprint: "fp1".into(), tags: vec!["cat".into()] };
        let b = _MediaAsset { id: "a2".into(), path: "cats/happy_cat_copy.png".into(), fingerprint: "fp1".into(), tags: vec![] };
        let ea = enr._enrich(&a);
        assert_eq!(ea.keep, true);
        let eb = enr._enrich(&b);
        assert_eq!(eb.dedup_of.as_deref(), Some("a1"), "同指纹判定重复");
        assert_eq!(eb.keep, false);
        let (total, dup, kept) = enr.stats();
        assert_eq!((total, dup, kept), (2, 1, 1));
    }

    #[test]
    fn asset_enricher_semantic_tags_from_path() {
        let mut enr = _AssetEnricher::new();
        let asset = _MediaAsset {
            id: "a1".into(),
            path: "materials/nature_sunset_beach.png".into(),
            fingerprint: "f".into(),
            tags: vec!["photo".into()],
        };
        let e = enr._enrich(&asset);
        assert!(e.semantic_tags.contains(&"nature".to_string()), "路径 token 提取语义标签");
        assert!(e.semantic_tags.contains(&"photo".to_string()));
    }
}

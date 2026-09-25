//! NT-IO multimodal_transform — 图像模态 (`nt_image_transform`).
//!
//! P18 vision_preprocess (VLM 多图联合分析: 幂等缓存 + fail-closed) 与
//! P13 unified_face (detection/recognition/tracking/gaze 统一 API)。确定性骨架, 无真实视觉后端。

use std::collections::HashMap;

// ────────────────────────────────────────────────────────────────────────────
// P18 vision_preprocess — VLM 多图联合分析预处理 (Plugin-Deepseek-Vision 吸收)
// 将多图 payload 替换为联合分析文本; 幂等缓存 + fail-closed (不静默降级)。
// ────────────────────────────────────────────────────────────────────────────

/// 图像引用 — `hash` 由 id+size_bytes 经 djb2 确定性计算。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRef {
    pub id: String,
    pub size_bytes: usize,
    pub hash: String,
}

/// djb2 确定性哈希 (id 字节 + size_bytes 小端字节) → 8 位 hex。
pub fn djb2_hash(id: &str, size_bytes: usize) -> String {
    let mut h: u32 = 5381;
    for b in id.bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(b));
    }
    for b in size_bytes.to_le_bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(b));
    }
    format!("{h:08x}")
}

impl ImageRef {
    pub fn new(id: impl Into<String>, size_bytes: usize) -> Self {
        let id = id.into();
        let hash = djb2_hash(&id, size_bytes);
        Self { id, size_bytes, hash }
    }
}

/// 多图输入 — 同批图片 + 可选说明提示。
#[derive(Debug, Clone)]
pub struct VisionInput {
    pub images: Vec<ImageRef>,
    pub caption_hint: Option<String>,
}

/// 预处理输出文本。
#[derive(Debug, Clone)]
pub struct VisionText {
    pub text: String,
    pub image_count: usize,
    pub cache_hit: bool,
}

/// hash 集确定性 key (排序后 join, 前缀数量防碰撞)。
fn hash_set_key(hashes: &[String]) -> String {
    let mut sorted: Vec<&String> = hashes.iter().collect();
    sorted.sort();
    let joined = sorted.iter().map(|h| h.as_str()).collect::<Vec<_>>().join(",");
    format!("{}:{}", sorted.len(), joined)
}

/// 幂等缓存: hash 集 → 联合分析文本。同 hash 集不重复调用 analyze。
#[derive(Debug, Clone, Default)]
pub struct VisionCache {
    entries: HashMap<String, String>,
}

impl VisionCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_cached(&self, hashes: &[String]) -> bool {
        self.entries.contains_key(&hash_set_key(hashes))
    }

    pub fn cache_len(&self) -> usize {
        self.entries.len()
    }
}

/// VLM 多图联合分析预处理 (fail-closed)。
pub struct VisionPreprocessor {
    pub cache: VisionCache,
    invocations: usize,
}

impl Default for VisionPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl VisionPreprocessor {
    pub fn new() -> Self {
        Self {
            cache: VisionCache::new(),
            invocations: 0,
        }
    }

    /// analyze 实际被调用的次数 (幂等性证明)。
    pub fn invocations(&self) -> usize {
        self.invocations
    }

    /// fail-closed: 空图集或缺 hash → Err, 绝不静默降级。
    fn fail_closed(reason: impl Into<String>) -> Result<VisionText, String> {
        Err(format!("vision preprocess fail-closed: {}", reason.into()))
    }

    /// 联合分析: 每唯一 hash 集执行一次 analyze; 命中缓存直接返回缓存文本。
    pub fn preprocess(
        &mut self,
        input: VisionInput,
        analyze: fn(&[ImageRef]) -> String,
    ) -> Result<VisionText, String> {
        if input.images.is_empty() {
            return Self::fail_closed("empty image set");
        }
        for img in &input.images {
            if img.hash.is_empty() {
                return Self::fail_closed(format!("image '{}' missing content hash", img.id));
            }
        }
        let hashes: Vec<String> = input.images.iter().map(|i| i.hash.clone()).collect();
        let key = hash_set_key(&hashes);
        if let Some(cached) = self.cache.entries.get(&key) {
            return Ok(VisionText {
                text: cached.clone(),
                image_count: input.images.len(),
                cache_hit: true,
            });
        }
        let analysis = analyze(&input.images);
        self.invocations += 1;
        let mut text = format!("[joint-analysis] {analysis}");
        if let Some(hint) = &input.caption_hint {
            text.push_str(&format!("\n[caption] {hint}"));
        }
        let result = VisionText {
            text: text.clone(),
            image_count: input.images.len(),
            cache_hit: false,
        };
        self.cache.entries.insert(key, text);
        Ok(result)
    }
}

/// SelfTest (T1): "nt_io_multimodal_vision_preprocess" — 幂等 + fail-closed 自检。
impl crate::l0_substrate::nt_core_self_test::SelfTest for VisionPreprocessor {
    fn name(&self) -> &str {
        "nt_io_multimodal_vision_preprocess"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut p = VisionPreprocessor::new();
        let join: fn(&[ImageRef]) -> String = |imgs| {
            imgs.iter().map(|i| i.id.clone()).collect::<Vec<_>>().join("+")
        };
        let input = VisionInput {
            images: vec![ImageRef::new("a.png", 100)],
            caption_hint: None,
        };
        match p.preprocess(input.clone(), join) {
            Ok(t) => {
                if t.image_count != 1 || t.cache_hit {
                    failures.push("first preprocess must be a fresh joint-analysis".into());
                }
            }
            Err(e) => failures.push(format!("preprocess failed: {e}")),
        }
        if let Err(e) = p.preprocess(input, join) {
            failures.push(format!("second preprocess failed: {e}"));
        }
        if p.invocations() != 1 {
            failures.push("joint analysis invoked more than once for same hash set".into());
        }
        let empty = VisionInput {
            images: vec![],
            caption_hint: None,
        };
        if p.preprocess(empty, join).is_ok() {
            failures.push("empty image set must fail closed".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// P13 unified_face — 人脸分析统一 API (uniface 吸收: detection/recognition/
// tracking/gaze 四任务统一接口)。确定性骨架实现, 无真实视觉后端。
// ────────────────────────────────────────────────────────────────────────────

/// 人脸分析四任务。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaceTask {
    Detection,
    Recognition,
    Tracking,
    Gaze,
}

impl FaceTask {
    pub fn label(self) -> &'static str {
        match self {
            FaceTask::Detection => "detection",
            FaceTask::Recognition => "recognition",
            FaceTask::Tracking => "tracking",
            FaceTask::Gaze => "gaze",
        }
    }
}

/// 单张人脸框 (归一化坐标 [0,1] 内)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub confidence: f64,
}

/// 人脸分析结果 — 四任务共享统一返回。
#[derive(Debug, Clone)]
pub struct FaceResult {
    pub task: FaceTask,
    pub boxes: Vec<FaceBox>,
    pub identity: Option<String>,
    pub gaze_vector: Option<(f64, f64, f64)>,
}

/// 人脸分析统一引擎 (确定性骨架)。
pub struct UnifiedFace {
    pub max_detections: usize,
    pub min_confidence: f64,
}

impl Default for UnifiedFace {
    fn default() -> Self {
        Self {
            max_detections: 10,
            min_confidence: 0.5,
        }
    }
}

impl UnifiedFace {
    pub fn new(max_detections: usize, min_confidence: f64) -> Self {
        Self {
            max_detections,
            min_confidence,
        }
    }

    /// detection: 确定性伪随机坐标 (i*31%1000/1000 派生), 置信度 =
    /// min_confidence + (i%5)/10, 低于阈值者过滤。
    pub fn detect(&self, face_count: usize) -> FaceResult {
        let count = face_count.min(self.max_detections);
        let mut boxes = Vec::with_capacity(count);
        for i in 0..count {
            let confidence =
                (self.min_confidence + (i % 5) as f64 / 10.0).max(0.0).min(1.0);
            if confidence < self.min_confidence {
                continue;
            }
            boxes.push(FaceBox {
                x: (i.wrapping_mul(31) % 1000) as f64 / 1000.0,
                y: (i.wrapping_mul(131) % 1000) as f64 / 1000.0,
                w: (i.wrapping_mul(47) % 1000) as f64 / 1000.0,
                h: (i.wrapping_mul(17) % 1000) as f64 / 1000.0,
                confidence,
            });
        }
        FaceResult {
            task: FaceTask::Detection,
            boxes,
            identity: None,
            gaze_vector: None,
        }
    }

    /// recognition: 1 个框 + 身份。
    pub fn recognize(&self, identity: &str) -> FaceResult {
        FaceResult {
            task: FaceTask::Recognition,
            boxes: vec![FaceBox {
                x: 0.0,
                y: 0.0,
                w: 0.5,
                h: 0.5,
                confidence: self.min_confidence.max(0.0).min(1.0),
            }],
            identity: Some(identity.to_string()),
            gaze_vector: None,
        }
    }

    /// tracking: 每帧 detect, 身份无关 (identity=None), 返回帧序列。
    pub fn track(&self, frames: usize) -> Vec<FaceResult> {
        let mut out = Vec::with_capacity(frames);
        for _ in 0..frames {
            let mut r = self.detect(self.max_detections);
            r.task = FaceTask::Tracking;
            r.identity = None;
            out.push(r);
        }
        out
    }

    /// gaze: 注视向量。
    pub fn gaze(&self) -> FaceResult {
        FaceResult {
            task: FaceTask::Gaze,
            boxes: Vec::new(),
            identity: None,
            gaze_vector: Some((0.0, 0.5, 1.0)),
        }
    }

    /// 统一分派入口。Detection/Tracking 的 arg 解析失败用默认 3。
    pub fn run(&self, task: FaceTask, arg: &str) -> FaceResult {
        match task {
            FaceTask::Detection => self.detect(arg.parse::<usize>().unwrap_or(3)),
            FaceTask::Recognition => self.recognize(arg),
            FaceTask::Tracking => {
                let frames = self.track(arg.parse::<usize>().unwrap_or(3));
                frames.into_iter().next_back().unwrap_or_else(|| FaceResult {
                    task: FaceTask::Tracking,
                    boxes: Vec::new(),
                    identity: None,
                    gaze_vector: None,
                })
            }
            FaceTask::Gaze => self.gaze(),
        }
    }
}

/// SelfTest (T1): "nt_io_multimodal_unified_face" — 四任务统一接口自检。
impl crate::l0_substrate::nt_core_self_test::SelfTest for UnifiedFace {
    fn name(&self) -> &str {
        "nt_io_multimodal_unified_face"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let f = UnifiedFace::default();
        if f.detect(3).boxes.len() != 3 {
            failures.push("detect must yield face_count boxes".into());
        }
        if f.recognize("alice").identity.as_deref() != Some("alice") {
            failures.push("recognize must carry identity".into());
        }
        if f.track(2).len() != 2 {
            failures.push("track must yield frame count".into());
        }
        if f.gaze().gaze_vector != Some((0.0, 0.5, 1.0)) {
            failures.push("gaze must carry gaze vector".into());
        }
        if f.run(FaceTask::Gaze, "").gaze_vector.is_none() {
            failures.push("run must dispatch gaze".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod vision_preprocess_tests {
    use super::*;

    fn joint_analyzer(imgs: &[ImageRef]) -> String {
        imgs.iter().map(|i| i.id.as_str()).collect::<Vec<_>>().join("+")
    }

    fn input_for(ids: &[&str]) -> VisionInput {
        VisionInput {
            images: ids.iter().map(|id| ImageRef::new(*id, 1024)).collect(),
            caption_hint: None,
        }
    }

    // ── P18 vision_preprocess ─────────────────────────────────

    #[test]
    fn vision_pre_analyze_invoked_once_per_hash_set() {
        let mut p = VisionPreprocessor::new();
        let out = p.preprocess(input_for(&["a.png", "b.png"]), joint_analyzer).expect("ok");
        assert!(!out.cache_hit);
        assert!(out.text.contains("[joint-analysis] a.png+b.png"));
        assert_eq!(p.invocations(), 1);
        assert_eq!(p.cache.cache_len(), 1);
    }

    #[test]
    fn vision_pre_cache_hit_skips_analyze() {
        let mut p = VisionPreprocessor::new();
        p.preprocess(input_for(&["a.png"]), joint_analyzer).expect("ok");
        let hash = ImageRef::new("a.png", 1024).hash;
        assert!(p.cache.is_cached(&[hash]));
        let out = p.preprocess(input_for(&["a.png"]), joint_analyzer).expect("ok");
        assert!(out.cache_hit, "same hash set must be served from cache");
        assert_eq!(p.invocations(), 1, "analyze must not be re-invoked");
        assert_eq!(p.cache.cache_len(), 1);
    }

    #[test]
    fn vision_pre_different_hash_set_reanalyzes() {
        let mut p = VisionPreprocessor::new();
        p.preprocess(input_for(&["a.png"]), joint_analyzer).expect("ok");
        p.preprocess(input_for(&["b.png"]), joint_analyzer).expect("ok");
        assert_eq!(p.invocations(), 2);
        assert_eq!(p.cache.cache_len(), 2);
    }

    #[test]
    fn vision_pre_empty_images_fail_closed() {
        let mut p = VisionPreprocessor::new();
        let err = p
            .preprocess(VisionInput { images: vec![], caption_hint: None }, joint_analyzer)
            .expect_err("empty set must fail closed");
        assert!(err.contains("fail-closed"));
        assert_eq!(p.invocations(), 0);
    }

    #[test]
    fn vision_pre_missing_hash_fail_closed() {
        let mut p = VisionPreprocessor::new();
        let input = VisionInput {
            images: vec![ImageRef {
                id: "x".into(),
                size_bytes: 1,
                hash: String::new(),
            }],
            caption_hint: None,
        };
        assert!(p.preprocess(input, joint_analyzer).is_err());
        assert_eq!(p.invocations(), 0);
    }

    #[test]
    fn vision_pre_caption_hint_included_in_text() {
        let mut p = VisionPreprocessor::new();
        let input = VisionInput {
            images: vec![ImageRef::new("a.png", 10)],
            caption_hint: Some("这是支付页".into()),
        };
        let out = p.preprocess(input, joint_analyzer).expect("ok");
        assert!(out.text.contains("[caption] 这是支付页"));
        assert_eq!(out.image_count, 1);
    }

    #[test]
    fn vision_pre_djb2_hash_deterministic() {
        assert_eq!(djb2_hash("a.png", 100), djb2_hash("a.png", 100));
        assert_ne!(djb2_hash("a.png", 100), djb2_hash("b.png", 100));
        assert_ne!(djb2_hash("a.png", 100), djb2_hash("a.png", 200));
        let img = ImageRef::new("a.png", 100);
        assert!(!img.hash.is_empty());
    }

    #[test]
    fn vision_pre_selftest_name_matches() {
        use crate::l0_substrate::nt_core_self_test::SelfTest;
        let p = VisionPreprocessor::new();
        assert_eq!(p.name(), "nt_io_multimodal_vision_preprocess");
        assert!(p.self_test().is_ok());
    }
}

#[cfg(test)]
mod unified_face_tests {
    use super::*;

    #[test]
    fn unified_face_detect_filters_low_confidence() {
        let f = UnifiedFace::new(10, 1.5);
        let r = f.detect(10);
        assert!(r.boxes.is_empty(), "below-threshold boxes must be filtered");
        let d = UnifiedFace::default();
        let ok = d.detect(5);
        assert_eq!(ok.boxes.len(), 5, "default threshold retains all boxes");
        assert!(ok.boxes.iter().all(|b| b.confidence >= d.min_confidence));
    }

    #[test]
    fn unified_face_detect_respects_max_detections() {
        let f = UnifiedFace::new(3, 0.5);
        assert_eq!(f.detect(10).boxes.len(), 3, "capped by max_detections");
        assert_eq!(f.detect(2).boxes.len(), 2);
    }

    #[test]
    fn unified_face_recognize_carries_identity() {
        let f = UnifiedFace::new(10, 0.5);
        let r = f.recognize("alice");
        assert_eq!(r.task, FaceTask::Recognition);
        assert_eq!(r.boxes.len(), 1);
        assert_eq!(r.identity.as_deref(), Some("alice"));
        assert!(r.boxes[0].confidence >= f.min_confidence);
    }

    #[test]
    fn unified_face_track_returns_frame_count() {
        let f = UnifiedFace::new(10, 0.5);
        let frames = f.track(4);
        assert_eq!(frames.len(), 4);
        for fr in &frames {
            assert_eq!(fr.task, FaceTask::Tracking);
            assert!(fr.identity.is_none());
            assert!(!fr.boxes.is_empty());
        }
    }

    #[test]
    fn unified_face_gaze_carries_vector() {
        let f = UnifiedFace::default();
        let r = f.gaze();
        assert_eq!(r.task, FaceTask::Gaze);
        assert_eq!(r.gaze_vector, Some((0.0, 0.5, 1.0)));
    }

    #[test]
    fn unified_face_run_dispatches_all_tasks() {
        let f = UnifiedFace::default();
        assert_eq!(f.run(FaceTask::Detection, "5").boxes.len(), 5);
        assert_eq!(
            f.run(FaceTask::Detection, "bad").boxes.len(),
            3,
            "parse failure falls back to default"
        );
        assert_eq!(
            f.run(FaceTask::Recognition, "bob").identity.as_deref(),
            Some("bob")
        );
        assert_eq!(
            f.run(FaceTask::Tracking, "2").boxes.len(),
            10,
            "last frame of 2-frame track"
        );
        assert_eq!(
            f.run(FaceTask::Tracking, "oops").boxes.len(),
            10,
            "tracking parse fallback to 3 frames"
        );
        assert_eq!(
            f.run(FaceTask::Gaze, "x").gaze_vector,
            Some((0.0, 0.5, 1.0))
        );
    }

    #[test]
    fn unified_face_labels_distinct() {
        let labels: Vec<&str> = [
            FaceTask::Detection,
            FaceTask::Recognition,
            FaceTask::Tracking,
            FaceTask::Gaze,
        ]
        .iter()
        .map(|t| t.label())
        .collect();
        let mut set = std::collections::HashSet::new();
        for l in &labels {
            assert!(set.insert(*l), "duplicate label {l}");
        }
        assert_eq!(labels.len(), 4);
        assert_eq!(FaceTask::Detection.label(), "detection");
    }

    #[test]
    fn unified_face_selftest_name_matches() {
        use crate::l0_substrate::nt_core_self_test::SelfTest;
        let f = UnifiedFace::default();
        assert_eq!(f.name(), "nt_io_multimodal_unified_face");
        assert!(f.self_test().is_ok());
    }
}

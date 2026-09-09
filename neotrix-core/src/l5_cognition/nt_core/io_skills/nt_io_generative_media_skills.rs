//! 多模态生成媒体技能 (NT-IO)
//!
//! 吸收源: github.com/SamurAIGPT/Generative-Media-Skills
//! 成熟度: C1 (unit-tested stub, 无 muapi.ai 运行时集成)
//!
//! 核心能力: 把多模态生成媒体编排为图/视频/音频三种生成接口
//! (经 muapi.ai 统一网关)。本 stub 负责生成请求校验与规格生成。

use crate::core::nt_core_self_test::SelfTest;

/// 媒体模态。
#[derive(Debug, Clone, PartialEq)]
pub enum MediaModality {
    Image,
    Video,
    Audio,
}

/// 生成请求: 模态 + 提示词 + 时长(仅视频/音频有意义)。
#[derive(Debug, Clone, PartialEq)]
pub struct GenRequest {
    pub modality: MediaModality,
    pub prompt: String,
    pub duration_sec: f64,
}

/// 多模态生成媒体 trait — 图/视频/音频生成接口 stub。
pub trait GenerativeMedia: Send + Sync {
    /// 校验请求: 提示词非空; 视频/音频时长 > 0, 图像时长忽略。
    /// 返回 true 当可提交。
    fn is_submittable(&self, req: &GenRequest) -> bool;
    /// 生成网关规格串: 非法请求返回 None。
    fn gateway_spec(&self, req: &GenRequest) -> Option<String>;
}

/// 默认实现。
#[derive(Default)]
pub struct MuapiMedia;

impl GenerativeMedia for MuapiMedia {
    fn is_submittable(&self, req: &GenRequest) -> bool {
        let prompt_ok = !req.prompt.trim().is_empty();
        if !prompt_ok {
            return false;
        }
        match req.modality {
            MediaModality::Image => true,
            MediaModality::Video | MediaModality::Audio => req.duration_sec > 0.0,
        }
    }

    fn gateway_spec(&self, req: &GenRequest) -> Option<String> {
        if !self.is_submittable(req) {
            return None;
        }
        let kind = match req.modality {
            MediaModality::Image => "image",
            MediaModality::Video => "video",
            MediaModality::Audio => "audio",
        };
        if req.modality == MediaModality::Image {
            Some(format!("muapi://gen/{}/{}", kind, req.prompt))
        } else {
            Some(format!(
                "muapi://gen/{}/{}?dur={}",
                kind, req.prompt, req.duration_sec
            ))
        }
    }
}

/// T1 SelfTest: 验证生成请求校验与网关规格存在且生效。
#[derive(Default)]
pub struct GenerativeMediaSelfTest;

impl SelfTest for GenerativeMediaSelfTest {
    fn name(&self) -> &str {
        "nt_io_generative_media_skills"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let m = MuapiMedia;
        let req = GenRequest {
            modality: MediaModality::Image,
            prompt: "a neon cityscape".into(),
            duration_sec: 0.0,
        };
        match m.gateway_spec(&req) {
            Some(s) if s.contains("image") => Ok(()),
            _ => Err(vec!["nt_io_generative_media_skills: image spec failed".into()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_submittable() {
        let m = MuapiMedia;
        let req = GenRequest { modality: MediaModality::Image, prompt: "x".into(), duration_sec: 0.0 };
        assert!(m.is_submittable(&req));
        assert!(m.gateway_spec(&req).unwrap().contains("muapi://gen/image"));
    }

    #[test]
    fn test_video_requires_duration() {
        let m = MuapiMedia;
        let no_dur = GenRequest { modality: MediaModality::Video, prompt: "x".into(), duration_sec: 0.0 };
        assert!(!m.is_submittable(&no_dur));
        assert_eq!(m.gateway_spec(&no_dur), None);
        let ok = GenRequest { modality: MediaModality::Video, prompt: "x".into(), duration_sec: 4.0 };
        assert!(m.gateway_spec(&ok).unwrap().contains("dur=4"));
    }

    #[test]
    fn test_rejects_empty_prompt() {
        let m = MuapiMedia;
        let empty = GenRequest { modality: MediaModality::Audio, prompt: "  ".into(), duration_sec: 3.0 };
        assert!(!m.is_submittable(&empty));
        assert_eq!(m.gateway_spec(&empty), None);
    }
}

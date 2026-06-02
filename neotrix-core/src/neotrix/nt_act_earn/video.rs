use std::path::PathBuf;

use super::content::{ContentPlan, VideoScript, VideoScene};

// ── TTS (Text-to-Speech) ──────────────────────────────────────────

/// TTS 引擎 trait — 将文字转为音频文件
pub trait TtsEngine: Send + Sync {
    fn synthesize(&self, text: &str, nt_act_voice: &str, output_path: &str) -> Result<String, String>;
    fn available_nt_act_voices(&self) -> Vec<String>;
}

/// Edge TTS 后端（通过 edge-tts CLI，最稳定跨平台）
pub struct EdgeTtsBackend {
    nt_act_voice: String,
    timeout_secs: u64,
}

impl EdgeTtsBackend {
    pub fn new(nt_act_voice: &str) -> Self {
        Self { nt_act_voice: nt_act_voice.to_string(), timeout_secs: 60 }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

impl TtsEngine for EdgeTtsBackend {
    fn synthesize(&self, text: &str, nt_act_voice: &str, output_path: &str) -> Result<String, String> {
        let nt_act_voice = if nt_act_voice.is_empty() { &self.nt_act_voice } else { nt_act_voice };
        let output = std::process::Command::new("edge-tts")
            .args(["--nt_act_voice", nt_act_voice, "--text", text, "--write-media", output_path])
            .output()
            .map_err(|e| format!("edge-tts not found: {}. Install: pip install edge-tts", e))?;
        if output.status.success() {
            Ok(output_path.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("edge-tts failed: {}", stderr))
        }
    }

    fn available_nt_act_voices(&self) -> Vec<String> {
        vec![
            "zh-CN-XiaoxiaoNeural".into(), "zh-CN-YunxiNeural".into(),
            "en-US-JennyNeural".into(), "en-US-GuyNeural".into(),
            "en-GB-SoniaNeural".into(), "ja-JP-NanamiNeural".into(),
        ]
    }
}

/// 简洁 TTS 合成（无 trait 约束的便捷函数）
pub fn synthesize_speech(text: &str, nt_act_voice: &str, output: &str) -> Result<String, String> {
    EdgeTtsBackend::new(nt_act_voice).synthesize(text, nt_act_voice, output)
}

// ── Media Source (素材搜索下载) ────────────────────────────────────

/// 视频素材
#[derive(Clone, Debug)]
pub struct MediaClip {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub duration_secs: f64,
}

/// 素材来源 trait — 对标 MoneyPrinterTurbo 的 services/material.py
pub trait MediaSource: Send + Sync {
    fn search_videos(&self, query: &str, count: u32, orientation: &str) -> Result<Vec<MediaClip>, String>;
    fn download(&self, clip: &MediaClip, output_path: &str) -> Result<String, String>;
    fn name(&self) -> &str;
}

/// Pexels API 素材源（多 key 轮转，对标 MoneyPrinterTurbo 的 pexels/pixabay key rotation）
pub struct PexelsSource {
    api_keys: Vec<String>,
    key_index: std::sync::atomic::AtomicUsize,
    client: reqwest::blocking::Client,
}

impl PexelsSource {
    pub fn new(api_keys: Vec<String>) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build().unwrap_or_default();
        Self { api_keys, key_index: std::sync::atomic::AtomicUsize::new(0), client }
    }

    pub fn from_env() -> Self {
        let keys = std::env::var("PEXELS_API_KEYS").unwrap_or_default();
        let api_keys: Vec<String> = if keys.is_empty() {
            vec![]
        } else {
            keys.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        };
        Self::new(api_keys)
    }

    fn current_key(&self) -> String {
        if self.api_keys.is_empty() { return String::new(); }
        let idx = self.key_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.api_keys.len();
        self.api_keys[idx].clone()
    }
}

impl MediaSource for PexelsSource {
    fn search_videos(&self, query: &str, count: u32, orientation: &str) -> Result<Vec<MediaClip>, String> {
        let key = self.current_key();
        if key.is_empty() {
            return Ok(vec![]); // no keys = use local/default materials
        }
        let orient = if orientation == "9:16" { "portrait" } else { "landscape" };
        let resp = self.client.get("https://api.pexels.com/videos/search")
            .header("Authorization", &key)
            .query(&[("query", query), ("per_page", &count.to_string()), ("orientation", orient)])
            .send()
            .map_err(|e| format!("Pexels request failed: {}", e))?;
        let json: serde_json::Value = resp.json().map_err(|e| format!("Pexels parse failed: {}", e))?;
        let clips = json.get("videos").and_then(|v| v.as_array()).map(|videos| {
            videos.iter().filter_map(|v| {
                let duration = v.get("duration").and_then(|d| d.as_f64()).unwrap_or(5.0);
                let video_files = v.get("video_files").and_then(|f| f.as_array())?;
                let best = video_files.iter()
                    .filter(|f| f.get("width").and_then(|w| w.as_i64()).unwrap_or(0) >= 480)
                    .min_by_key(|f| {
                        let w = f.get("width").and_then(|w| w.as_i64()).unwrap_or(9999);
                        let h = f.get("height").and_then(|h| h.as_i64()).unwrap_or(9999);
                        (w - 1080).abs() + (h - 1920).abs()
                    })?;
                Some(MediaClip {
                    url: best.get("link").and_then(|l| l.as_str()).unwrap_or("").to_string(),
                    width: best.get("width").and_then(|w| w.as_i64()).unwrap_or(0) as u32,
                    height: best.get("height").and_then(|h| h.as_i64()).unwrap_or(0) as u32,
                    duration_secs: duration,
                })
            }).collect()
        }).unwrap_or_default();
        Ok(clips)
    }

    fn download(&self, clip: &MediaClip, output_path: &str) -> Result<String, String> {
        let resp = self.client.get(&clip.url)
            .send()
            .map_err(|e| format!("Download failed: {}", e))?;
        let bytes = resp.bytes().map_err(|e| format!("Read failed: {}", e))?;
        std::fs::write(output_path, &bytes)
            .map_err(|e| format!("Write failed: {}", e))?;
        Ok(output_path.to_string())
    }

    fn name(&self) -> &str { "pexels" }
}

/// 本地素材源 — 从本地目录加载视频/图片
pub struct LocalMediaSource {
    media_dir: PathBuf,
}

impl LocalMediaSource {
    pub fn new(dir: &str) -> Self {
        Self { media_dir: PathBuf::from(dir) }
    }
}

impl MediaSource for LocalMediaSource {
    fn search_videos(&self, _query: &str, count: u32, _orientation: &str) -> Result<Vec<MediaClip>, String> {
        let entries: Vec<_> = std::fs::read_dir(&self.media_dir)
            .map_err(|e| format!("Failed to read media dir: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| matches!(ext.to_str(), Some("mp4"|"mov"|"avi"|"webm")))
                    .unwrap_or(false)
            })
            .take(count as usize)
            .map(|e| MediaClip {
                url: e.path().to_string_lossy().to_string(),
                width: 1920, height: 1080,
                duration_secs: 5.0,
            })
            .collect();
        Ok(entries)
    }

    fn download(&self, clip: &MediaClip, output_path: &str) -> Result<String, String> {
        std::fs::copy(&clip.url, output_path)
            .map_err(|e| format!("Copy failed: {}", e))?;
        Ok(output_path.to_string())
    }

    fn name(&self) -> &str { "local" }
}

// ── Video Renderer (视频合成渲染) ──────────────────────────────────

/// 视频渲染配置 — 对标 MoneyPrinterTurbo 的 VideoParams
#[derive(Clone, Debug)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub subtitle_font: String,
    pub subtitle_font_size: u32,
    pub subtitle_color: String,
    pub subtitle_position: String,
    pub bgm_volume: f64,
    pub transition: String,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1080, height: 1920,
            fps: 30,
            subtitle_font: "STHeitiMedium.ttc".into(),
            subtitle_font_size: 48,
            subtitle_color: "white".into(),
            subtitle_position: "bottom".into(),
            bgm_volume: 0.2,
            transition: "fade".into(),
        }
    }
}

/// FFmpeg 视频渲染器 — 对标 MoneyPrinterTurbo 的 combine_videos() + generate_video()
pub struct FfmpegRenderer {
    config: RenderConfig,
}

impl FfmpegRenderer {
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// 检查 ffmpeg 是否可用
    pub fn check_available() -> bool {
        std::process::Command::new("ffmpeg").arg("-version").output().is_ok()
    }

    /// 将场景渲染为视频片段（TTS 音频 + 场景文字 + 素材视频）
    pub fn render_scene(
        &self, scene: &VideoScene, scene_idx: usize, audio_path: &str,
        material_path: Option<&str>, output_dir: &str, _temp_dir: &str,
    ) -> Result<String, String> {
        let out = format!("{}/scene_{:03}.mp4", output_dir, scene_idx);

        let mut filter = Vec::new();

        // 素材视频/图片流
        if material_path.is_some() {
            filter.push(format!(
                "[0:v]scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2[v0]",
                w = self.config.width, h = self.config.height
            ));
        } else {
            // 纯色背景（无素材时）
            let color = format!("0x{:02x}{:02x}{:02x}", 30 + scene_idx as u32 * 20 % 200, 30, 60);
            filter.push(format!("color=c={}:s={}x{}:d={}:r={}[v0]",
                color, self.config.width, self.config.height, scene.duration_secs, self.config.fps));
        }

        // 字幕 overlay（drawtext filter）
        let escaped_narration = scene.narration.replace('\'', "'\\\\''");
        let pos_y = if self.config.subtitle_position == "bottom" {
            format!("h-th-{}", self.config.subtitle_font_size + 20)
        } else {
            format!("{}", self.config.subtitle_font_size + 20)
        };
        filter.push(format!(
            "[v0]drawtext=text='{}':fontfile='{}':fontsize={}:fontcolor={}:x=(w-text_w)/2:y={}:enable='between(t,0,{})'[v]",
            escaped_narration, self.config.subtitle_font, self.config.subtitle_font_size,
            self.config.subtitle_color, pos_y, scene.duration_secs
        ));

        // TTS 音频 + 背景音乐混音
        let filter_complex = filter.join(";\n");

        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-loglevel", "error"]);

        if let Some(mpath) = material_path {
            cmd.args(["-i", mpath]);
        } else {
            // color source 无需 -i
        }

        // 输入：音频
        cmd.args(["-i", audio_path]);

        // map 视频流 + 音频流
        cmd.args(["-map", "[v]", "-map", "1:a"]);
        cmd.args(["-c:v", "libx264", "-preset", "medium", "-crf", "23"]);
        cmd.args(["-c:a", "aac", "-b:a", "192k"]);
        cmd.args(["-t", &scene.duration_secs.to_string()]);
        cmd.args(["-filter_complex", &filter_complex]);
        cmd.arg(&out);

        let output = cmd.output().map_err(|e| format!("ffmpeg not found: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Scene render failed: {}", stderr));
        }
        Ok(out)
    }

    /// 合并场景视频 + BGM → 最终视频
    pub fn composite(
        &self, scene_paths: &[String], bgm_path: Option<&str>,
        audio_path: &str, output_path: &str,
    ) -> Result<String, String> {
        if scene_paths.is_empty() {
            return Err("No scenes to composite".to_string());
        }

        // 生成 concat 文件列表
        let concat_content: String = scene_paths.iter()
            .map(|p| format!("file '{}'\n", p.replace('\'', "'\\\\''")))
            .collect();
        let concat_file = format!("{}.concat.txt", output_path);
        std::fs::write(&concat_file, &concat_content)
            .map_err(|e| format!("Failed to write concat file: {}", e))?;

        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-loglevel", "error",
            "-f", "concat", "-safe", "0", "-i", &concat_file,
            "-i", audio_path,
        ]);

        // BGM 混音
        if let Some(bgm) = bgm_path {
            cmd.args(["-i", bgm]);
            // 视频 + 主音频（1:a）+ BGM（2:a）混音
            cmd.args(["-filter_complex",
                &format!("[1:a][2:a]amix=inputs=2:duration=first:weights=1 {}[aout]",
                    if self.config.bgm_volume < 1.0 {
                        format!("*{}", self.config.bgm_volume)
                    } else { String::new() }
                ),
                "-map", "0:v:0", "-map", "[aout]",
            ]);
        } else {
            cmd.args(["-map", "0:v:0", "-map", "1:a"]);
        }

        cmd.args(["-c:v", "libx264", "-preset", "medium", "-crf", "23"]);
        cmd.args(["-c:a", "aac", "-b:a", "192k", "-shortest"]);
        cmd.arg(output_path);

        let output = cmd.output().map_err(|e| format!("ffmpeg composite failed: {}", e))?;
        let _ = std::fs::remove_file(&concat_file);
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Composite failed: {}", stderr));
        }
        Ok(output_path.to_string())
    }

    /// 完整的视频渲染流程（从 VideoScript 到成品）
    pub fn render_script(
        &self, script: &VideoScript, audio_path: &str,
        material_paths: &[String], bgm_path: Option<&str>,
        output_dir: &str,
    ) -> Result<String, String> {
        let mut scene_paths = Vec::new();
        for (i, scene) in script.scenes.iter().enumerate() {
            let mat = material_paths.get(i).map(|s| s.as_str());
            let spath = self.render_scene(scene, i, audio_path, mat, output_dir, output_dir)
                .map_err(|e| format!("Scene {} failed: {}", i, e))?;
            scene_paths.push(spath);
        }
        let final_out = format!("{}/final.mp4", output_dir);
        self.composite(&scene_paths, bgm_path, audio_path, &final_out)
    }
}

// ── Video Pipeline (完整生产流水线) ────────────────────────────────

/// 视频生产流水线 — 对标 MoneyPrinterTurbo 的 task.start() 7步流程
pub struct VideoPipeline {
    pub tts: Box<dyn TtsEngine>,
    pub media_source: Box<dyn MediaSource>,
    pub renderer: FfmpegRenderer,
    pub bgm_dir: Option<PathBuf>,
    pub work_dir: PathBuf,
    pub nt_act_voice: String,
    pub orientation: String,
}

impl VideoPipeline {
    pub fn new(work_dir: &str) -> Self {
        let tts: Box<dyn TtsEngine> = Box::new(EdgeTtsBackend::new("en-US-JennyNeural"));
        let media: Box<dyn MediaSource> = Box::new(PexelsSource::from_env());
        Self {
            tts,
            media_source: media,
            renderer: FfmpegRenderer::new(RenderConfig::default()),
            bgm_dir: None,
            work_dir: PathBuf::from(work_dir),
            nt_act_voice: "en-US-JennyNeural".into(),
            orientation: "9:16".into(),
        }
    }

    pub fn with_tts(mut self, tts: Box<dyn TtsEngine>) -> Self { self.tts = tts; self }
    pub fn with_media(mut self, media: Box<dyn MediaSource>) -> Self { self.media_source = media; self }
    pub fn with_render_config(mut self, config: RenderConfig) -> Self { self.renderer = FfmpegRenderer::new(config); self }
    pub fn with_bgm_dir(mut self, dir: &str) -> Self { self.bgm_dir = Some(PathBuf::from(dir)); self }
    pub fn with_nt_act_voice(mut self, nt_act_voice: &str) -> Self { self.nt_act_voice = nt_act_voice.to_string(); self }
    pub fn with_orientation(mut self, orient: &str) -> Self { self.orientation = orient.to_string(); self }

    /// 执行完整视频生产：Script → TTS → 素材搜索/下载 → 渲染 → 输出路径
    pub fn produce(&self, plan: &ContentPlan) -> Result<String, String> {
        let script = plan.video_script.as_ref()
            .ok_or_else(|| "No video script in plan".to_string())?;

        let task_dir = self.work_dir.join(&plan.title.replace(' ', "_"));
        std::fs::create_dir_all(&task_dir)
            .map_err(|e| format!("Failed to create task dir: {}", e))?;

        // Step 1: TTS — 合成全片旁白音频
        let full_narration: Vec<&str> = script.scenes.iter().map(|s| s.narration.as_str()).collect();
        let full_text = full_narration.join(" ");
        let audio_path = task_dir.join("narration.mp3");
        self.tts.synthesize(&full_text, &self.nt_act_voice,
            &audio_path.to_string_lossy())?;

        // Step 2: 素材搜索 — 每个场景搜索 + download
        let mut material_paths = Vec::new();
        for (i, scene) in script.scenes.iter().enumerate() {
            let keywords = if scene.search_keywords.is_empty() {
                vec!["technology".to_string()]
            } else { scene.search_keywords.clone() };

            let mut clip_found = false;
            for kw in &keywords {
                let clips = self.media_source.search_videos(kw, 1, &self.orientation)?;
                if let Some(clip) = clips.first() {
                    let mat_path = task_dir.join(format!("mat_{:03}.mp4", i));
                    self.media_source.download(clip, &mat_path.to_string_lossy())?;
                    material_paths.push(mat_path.to_string_lossy().to_string());
                    clip_found = true;
                    break;
                }
            }
            if !clip_found {
                // 无素材 — 渲染器会用纯色背景
            }
        }

        // Step 3: BGM
        let bgm_path = self.bgm_dir.as_ref().and_then(|dir| {
            let entries: Vec<_> = std::fs::read_dir(dir).ok()
                .into_iter().flat_map(|e| e.filter_map(|e| e.ok()))
                .filter(|e| e.path().extension().map(|ext| ext == "mp3").unwrap_or(false))
                .collect();
            entries.first().map(|e| e.path().to_string_lossy().to_string())
        });

        // Step 4: 渲染
        self.renderer.render_script(
            script,
            &audio_path.to_string_lossy(),
            &material_paths,
            bgm_path.as_deref(),
            &task_dir.to_string_lossy(),
        )
    }

    /// 检查外部依赖是否可用
    pub fn check_deps() -> Vec<String> {
        let mut missing = Vec::new();
        if !FfmpegRenderer::check_available() {
            missing.push("ffmpeg".to_string());
        }
        let tts_check = std::process::Command::new("edge-tts")
            .arg("--help").output();
        if tts_check.is_err() {
            missing.push("edge-tts (pip install edge-tts)".to_string());
        }
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesize_speech_fails_without_cli() {
        // No edge-tts CLI in CI — expect error
        let result = synthesize_speech("Hello", "en-US-JennyNeural", "/tmp/test.mp3");
        // May pass if CLI installed; check either way
        if !result.is_ok() {
            assert!(result.unwrap_err().contains("edge-tts not found"));
        }
    }

    #[test]
    fn test_pexels_source_from_env() {
        let source = PexelsSource::from_env();
        assert_eq!(source.name(), "pexels");
    }

    #[test]
    fn test_render_config_defaults() {
        let config = RenderConfig::default();
        assert_eq!(config.width, 1080);
        assert_eq!(config.height, 1920);
    }

    #[test]
    fn test_ffmpeg_renderer_check() {
        let available = FfmpegRenderer::check_available();
        // may or may not be installed in CI
        if !available {
            eprintln!("ffmpeg not installed — skipping render tests");
        }
    }

    #[test]
    fn test_video_pipeline_check_deps() {
        let missing = VideoPipeline::check_deps();
        for m in &missing {
            eprintln!("Missing dependency: {}", m);
        }
    }

    #[test]
    fn test_local_media_source() {
        let dir = std::env::temp_dir();
        let source = LocalMediaSource::new(dir.to_string_lossy().as_ref());
        let clips = source.search_videos("test", 5, "16:9").unwrap_or_default();
        // temp_dir likely has no .mp4 files, so expect 0
        assert!(clips.is_empty());
    }
}

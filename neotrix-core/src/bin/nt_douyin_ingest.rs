//! neotrix-douyin — 抖音视频统一管道入库：已落盘页面 HTML → 解析 → 下载。
//!
//! 页面抓取（登录态浏览器）由采集器完成并落盘到 `--pages`；本 bin 只做
//! 管道内工作：[`nt_douyin_extract::parse_douyin_page`] 解析直链，
//! 经 neotrix 下载能力 [`download_to_file`]（SSRF guard + 断点续传 +
//! 原子落盘 + 幂等跳过）拉取 mp4，输出 `manifest.jsonl` 供转写阶段消费。
//!
//! 用法（项目根执行，数据只落项目内）：
//! ```sh
//! cargo run -p neotrix --bin neotrix-douyin -- \
//!   --list datasets/douyin_蜗牛有点田/videos.json \
//!   --pages datasets/douyin_蜗牛有点田/pages \
//!   --out datasets/douyin_蜗牛有点田/media
//! ```

use neotrix::l1_action::nt_media::nt_douyin_extract::{
    parse_douyin_detail, parse_douyin_page,
};
use neotrix::l4_emotion::nt_memory::nt_memory_kb::nt_http::{download_to_file, DownloadOptions};
use std::path::PathBuf;
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";
const ALLOWED_MIME: &[&str] = &["video/"];
const MAX_BYTES: u64 = 500 * 1024 * 1024;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("usage: neotrix-douyin --list videos.json --pages pages_dir --out media_dir [--transcribe out.wav [--models model_dir]]");
        return;
    }
    let list = find_arg(&args, "--list").unwrap_or_else(|| "videos.json".to_string());
    let pages = find_arg(&args, "--pages").unwrap_or_else(|| "pages".to_string());
    let detail = find_arg(&args, "--detail").unwrap_or_else(|| "detail".to_string());
    let out = find_arg(&args, "--out").unwrap_or_else(|| "media".to_string());
    let out_path = PathBuf::from(&out);
    if let Err(e) = std::fs::create_dir_all(&out_path) {
        eprintln!("error: create output dir {}: {e}", out_path.display());
        std::process::exit(1);
    }
    let raw = std::fs::read_to_string(&list).unwrap_or_else(|e| {
        eprintln!("error: read list {list}: {e}");
        std::process::exit(1);
    });
    let items: Vec<serde_json::Value> = serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("error: parse list {list}: {e}");
        std::process::exit(1);
    });
    let manifest_path = out_path.join("manifest.jsonl");
    let mut manifest = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&manifest_path)
        .unwrap_or_else(|e| {
            eprintln!("error: open manifest: {e}");
            std::process::exit(1);
        });
    use std::io::Write as _;
    let mut ok = 0usize;
    let mut failed: Vec<String> = Vec::new();
    for it in &items {
        let id = it.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let page_url = it
            .get("url")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("https://www.douyin.com/video/{id}"));
        if id.is_empty() {
            continue;
        }
        let dest = out_path.join(format!("{id}.mp4"));
        if dest.exists() && dest.metadata().map(|m| m.len() > 0).unwrap_or(false) {
            eprintln!("skip {id} (exists)");
            ok += 1;
            continue;
        }
        let detail_path = PathBuf::from(&detail).join(format!("{id}.json"));
        let video = if let Ok(raw) = std::fs::read_to_string(&detail_path) {
            match parse_douyin_detail(&raw, id) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("FAIL {id}: detail parse: {e}");
                    failed.push(id.to_string());
                    continue;
                }
            }
        } else {
            let html_path = PathBuf::from(&pages).join(format!("{id}.html"));
            let html = std::fs::read_to_string(&html_path).unwrap_or_else(|_| String::new());
            if html.is_empty() {
                eprintln!("FAIL {id}: missing detail json and page html");
                failed.push(id.to_string());
                continue;
            }
            match parse_douyin_page(&html, &page_url) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("FAIL {id}: parse: {e}");
                    failed.push(id.to_string());
                    continue;
                }
            }
        };
        let opts = DownloadOptions {
            url: &video.play_url,
            dest: &dest,
            user_agent: Some(UA),
            allowed_mime_types: ALLOWED_MIME,
            max_bytes: MAX_BYTES,
            total_timeout: Some(Duration::from_secs(300)),
            proxy: None,
            proxy_pool: false,
            host: None,
        };
        match download_to_file(&opts) {
            Ok(r) => {
                ok += 1;
                eprintln!("ok {id} ({} bytes{})", r.bytes_written, if r.resumed { ", resumed" } else { "" });
                let line = serde_json::json!({
                    "id": id,
                    "title": it.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                    "desc": video.desc,
                    "nickname": video.nickname,
                    "mp4": dest.to_string_lossy(),
                    "page_url": page_url,
                });
                if let Err(e) = writeln!(manifest, "{line}") {
                    eprintln!("error: write manifest: {e}");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("FAIL {id}: download: {e}");
                failed.push(id.to_string());
            }
        }
    }
    eprintln!("done: ok={ok} fail={}", failed.len());
    if !failed.is_empty() {
        std::process::exit(1);
    }
    if let Some(wav) = find_arg(&args, "--transcribe") {
        transcribe_wav(&wav);
    }
}

/// 自研转写：16k 单声道 WAV → 分段 JSON（stdout）。
/// 需 `--features onnx` 构建 + 模型目录（`--models`，默认 datasets/_models/sherpa-onnx-whisper-tiny.en）。
/// mp4 请先转 wav：`ffmpeg -i in.mp4 -ar 16000 -ac 1 out.wav`。
#[allow(clippy::too_many_lines)]
fn transcribe_wav(wav: &str) {
    #[cfg(not(feature = "onnx"))]
    {
        eprintln!("error: --transcribe {wav} 需要 --features onnx 构建");
        std::process::exit(2);
    }
    #[cfg(feature = "onnx")]
    {
        use neotrix::l1_action::nt_media::nt_speech_transcribe::inference::SpeechModel;
        let args: Vec<String> = std::env::args().skip(1).collect();
        let models = find_arg(&args, "--models")
            .unwrap_or_else(|| "datasets/_models/sherpa-onnx-whisper-tiny.en".to_string());
        let pcm = read_wav_mono16k(wav).unwrap_or_else(|e| {
            eprintln!("error: wav {wav}: {e}");
            std::process::exit(1);
        });
        let mut model = SpeechModel::load(std::path::Path::new(&models)).unwrap_or_else(|e| {
            eprintln!("error: load model {models}: {e}");
            std::process::exit(1);
        });
        match model.transcribe_pcm_16k(&pcm) {
            Ok(segs) => {
                for s in segs {
                    println!(
                        "{}",
                        serde_json::json!({
                            "start": s.start_s, "end": s.end_s, "text": s.text
                        })
                    );
                }
            }
            Err(e) => {
                eprintln!("error: transcribe: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// 最小 WAV 解析（PCM16 单声道 16k；其他格式报错指引 ffmpeg）。
#[cfg(feature = "onnx")]
fn read_wav_mono16k(path: &str) -> Result<Vec<f32>, String> {
    let b = std::fs::read(path).map_err(|e| e.to_string())?;
    if b.len() < 44 || &b[0..4] != b"RIFF" || &b[8..12] != b"WAVE" {
        return Err("not a wav file".into());
    }
    let mut pos = 12;
    let (mut channels, mut rate, mut bits, mut data_off, mut data_len) = (0, 0, 0, 0, 0);
    while pos + 8 <= b.len() {
        let id = &b[pos..pos + 4];
        let len = u32::from_le_bytes(b[pos + 4..pos + 8].try_into().map_err(|_| "wav header".to_string())?) as usize;
        if id == b"fmt " {
            channels = u16::from_le_bytes(b[pos + 10..pos + 12].try_into().map_err(|_| "wav fmt".to_string())?);
            rate = u32::from_le_bytes(b[pos + 12..pos + 16].try_into().map_err(|_| "wav fmt".to_string())?);
            bits = u16::from_le_bytes(b[pos + 22..pos + 24].try_into().map_err(|_| "wav fmt".to_string())?);
        } else if id == b"data" {
            data_off = pos + 8;
            data_len = len;
            break;
        }
        pos += 8 + len;
    }
    if channels != 1 || rate != 16000 || bits != 16 || data_len == 0 {
        return Err(format!("need mono/16k/16bit wav, got ch={channels} rate={rate} bits={bits}"));
    }
    let raw = &b[data_off..(data_off + data_len).min(b.len())];
    Ok(raw
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
        .collect())
}

fn find_arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1).cloned())
}

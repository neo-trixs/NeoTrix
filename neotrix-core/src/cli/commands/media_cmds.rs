//! 媒体流命令 — /stream (流式下载) + /detect (媒体类型检测)

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::l1_action::nt_media::detect::{detect_from_file, detect_remote};
use crate::l1_action::nt_media::download_progress::{DownloadProgress, ProgressConfig};
use crate::l1_action::nt_media::streaming::{
    PipelineConfig, PipelineStatus, StreamingPipeline,
};
use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;

// ── Buffer string parser ──────────────────────────────────────────────────

fn parse_buffer(s: &str) -> Result<u64, String> {
    let s = s.trim();
    let (num_part, unit) = if let Some(pos) = s.find(|c: char| c.is_alphabetic()) {
        (&s[..pos], s[pos..].to_uppercase())
    } else {
        (s, String::new())
    };
    let base: u64 = num_part
        .parse()
        .map_err(|e| format!("invalid buffer number '{}': {}", num_part, e))?;
    match unit.as_str() {
        "" | "B" => Ok(base),
        "KB" | "K" => Ok(base * 1024),
        "MB" | "M" => Ok(base * 1024 * 1024),
        "GB" | "G" => Ok(base * 1024 * 1024 * 1024),
        other => Err(format!("unknown buffer unit: {}", other)),
    }
}

// ── /stream ───────────────────────────────────────────────────────────────

pub struct MediaStreamCmd;

impl CliCommand for MediaStreamCmd {
    fn name(&self) -> &str {
        "/stream"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/media-stream"]
    }

    fn description(&self) -> &str {
        "Stream/download media: /stream <url> [--output-dir <dir>] [--streaming] [--buffer <size>] [--player <bin>]"
    }

    fn is_primary(&self) -> bool {
        false
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err(
                "Usage:\n  /stream <url>                              Download to /tmp/neotrix-stream\n  /stream <url> --output-dir ~/media        Custom output dir\n  /stream <url> --buffer 1MB                Buffer threshold\n  /stream <url> --player mpv                Player binary\n  /stream <url> --streaming                 Force streaming mode",
            );
        }

        // Manual arg parsing (consistent with codebase style)
        let mut url: Option<String> = None;
        let mut output_dir: Option<String> = None;
        let mut streaming = true;
        let mut buffer_str = "512KB".to_string();
        let mut player: Option<String> = None;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--output-dir" | "-o" => {
                    output_dir = args.get(i + 1).cloned();
                    i += 2;
                }
                "--streaming" => {
                    streaming = true;
                    i += 1;
                }
                "--no-streaming" => {
                    streaming = false;
                    i += 1;
                }
                "--buffer" | "-b" => {
                    buffer_str = args.get(i + 1).cloned().unwrap_or_else(|| "512KB".to_string());
                    i += 2;
                }
                "--player" | "-p" => {
                    player = args.get(i + 1).cloned();
                    i += 2;
                }
                other if other.starts_with('-') => {
                    return CommandOutput::err(&format!("Unknown option: {}", other));
                }
                _ => {
                    if url.is_none() {
                        url = Some(args[i].clone());
                    } else {
                        return CommandOutput::err(&format!("Unexpected argument: {}", args[i]));
                    }
                    i += 1;
                }
            }
        }

        let url = match url {
            Some(u) => u,
            None => return CommandOutput::err("Missing URL argument"),
        };

        let buffer = match parse_buffer(&buffer_str) {
            Ok(b) => b,
            Err(e) => return CommandOutput::err(&e),
        };

        let out_dir = output_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp/neotrix-stream"));

        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => return CommandOutput::err(&format!("Failed to create runtime: {}", e)),
        };

        rt.block_on(async {
            let config = PipelineConfig {
                url: url.clone(),
                output_dir: out_dir.clone(),
                prefer_streaming: streaming,
                player_bin: player,
                player_args: Vec::new(),
                buffer_threshold: buffer,
                proxy: None,
                timeout: Duration::from_secs(30),
                chunk_size: 256 * 1024,
                persistence: None,
                auth: None,
            };

            let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(64);
            let pipeline = StreamingPipeline::new(config);

            let handle = match pipeline.run(progress_tx).await {
                Ok(h) => h,
                Err(e) => return CommandOutput::err(&format!("Pipeline error: {}", e)),
            };

            let output_path = handle.output_path().to_path_buf();
            let mut last_msg = String::new();

            // Drain progress updates
            while let Ok(Some(progress)) =
                tokio::time::timeout(Duration::from_millis(500), progress_rx.recv()).await
            {
                let msg = match &progress.status {
                    PipelineStatus::Resolving => "Resolving...".to_string(),
                    PipelineStatus::Downloading {
                        downloaded,
                        total,
                        speed_bps,
                    } => {
                        let dl = format_bytes(*downloaded);
                        let spd = format_speed(*speed_bps);
                        match total {
                            Some(t) => {
                                let pct = *downloaded as f64 / *t as f64 * 100.0;
                                format!("Downloading {}/{} ({:.1}%) @ {}", dl, format_bytes(*t), pct, spd)
                            }
                            None => format!("Downloading {} @ {}", dl, spd),
                        }
                    }
                    PipelineStatus::Playing {
                        downloaded,
                        total,
                        speed_bps,
                    } => {
                        let dl = format_bytes(*downloaded);
                        let spd = format_speed(*speed_bps);
                        match total {
                            Some(t) => format!("Playing {}/{} @ {}", dl, format_bytes(*t), spd),
                            None => format!("Playing {} @ {}", dl, spd),
                        }
                    }
                    PipelineStatus::Complete {
                        total_bytes,
                        elapsed,
                    } => {
                        format!(
                            "Complete: {} in {:.1}s",
                            format_bytes(*total_bytes),
                            elapsed.as_secs_f64()
                        )
                    }
                    PipelineStatus::Failed(e) => format!("Failed: {}", e),
                    PipelineStatus::Cancelled => "Cancelled".to_string(),
                };
                last_msg = msg.clone();
                eprintln!("\r  {}", msg);
            }

            let result = handle.wait().await;
            match result {
                Ok(_) => CommandOutput::ok(&format!(
                    "Stream finished: {} → {}",
                    url,
                    output_path.display()
                )),
                Err(e) => CommandOutput::err(&format!("Stream failed: {}", e)),
            }
        })
    }
}

// ── /detect ───────────────────────────────────────────────────────────────

pub struct MediaDetectCmd;

impl CliCommand for MediaDetectCmd {
    fn name(&self) -> &str {
        "/detect"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/media-detect"]
    }

    fn description(&self) -> &str {
        "Detect media type: /detect <url-or-file-path>"
    }

    fn is_primary(&self) -> bool {
        false
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("Usage: /detect <url-or-file-path>");
        }

        let target = &args[0];
        let is_url = target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("magnet:")
            || target.starts_with("ftp://");

        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => return CommandOutput::err(&format!("Failed to create runtime: {}", e)),
        };

        if is_url {
            let target = target.clone();
            rt.block_on(async {
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .unwrap_or_default();
                let (kind, content_type) = detect_remote(&client, &target).await;
                let mut msg = format!("Media type: {}", kind.label());
                if let Some(ct) = content_type {
                    msg.push_str(&format!("\nContent-Type: {}", ct));
                }
                msg.push_str(&format!("\nCategory: {}", classify_media(kind)));
                msg.push_str(&format!("\nURL: {}", target));
                CommandOutput::ok(&msg)
            })
        } else {
            let path = PathBuf::from(target);
            if !path.exists() {
                return CommandOutput::not_found(&format!("File not found: {}", target));
            }
            rt.block_on(async {
                let kind = detect_from_file(&path).await;
                let mut msg = format!("Media type: {}", kind.label());
                msg.push_str(&format!("\nCategory: {}", classify_media(kind)));
                msg.push_str(&format!("\nFile: {}", target));
                if let Ok(meta) = tokio::fs::metadata(&path).await {
                    msg.push_str(&format!("\nSize: {}", format_bytes(meta.len())));
                }
                CommandOutput::ok(&msg)
            })
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn classify_media(kind: crate::l1_action::nt_media::detect::MediaKind) -> &'static str {
    if kind.is_audio() {
        "Audio"
    } else if kind.is_video() {
        "Video"
    } else if kind.is_document() {
        "Document"
    } else if kind.is_model() {
        "ML Model"
    } else {
        "Other"
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_speed(bps: f64) -> String {
    if bps < 1024.0 {
        format!("{:.0}B/s", bps)
    } else if bps < 1024.0 * 1024.0 {
        format!("{:.1}KB/s", bps / 1024.0)
    } else {
        format!("{:.1}MB/s", bps / (1024.0 * 1024.0))
    }
}

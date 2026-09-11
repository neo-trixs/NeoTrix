/// model-download — NeoTrix 自研下载引擎 CLI
/// 下载 HuggingFace GGUF 模型到本地，支持断点续传、分片并发、进度显示

use neotrix_core::l1_action::nt_io_download::{DownloadEngine, EngineConfig, DownloadSession, DownloadSource};
use std::path::PathBuf;
use std::time::SystemTime;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }

    // 解析参数
    let repo = find_arg(&args, "--repo").or_else(|| args.first().cloned());
    let file = find_arg(&args, "--file");
    let out_dir = find_arg(&args, "--output").unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/Downloads/neotrix/models", home)
    });
    let chunks: usize = find_arg(&args, "--chunks")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    let resume = !args.iter().any(|a| a == "--no-resume");

    let (repo, file) = match (repo, file) {
        (Some(r), Some(f)) => (r, f),
        _ => {
            eprintln!("error: --repo and --file are required");
            print_usage();
            std::process::exit(1);
        }
    };

    // 构建 URL
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        repo.trim_start_matches('/').trim_end_matches('/'),
        file
    );

    // 目标路径
    let mut target = PathBuf::from(&out_dir);
    std::fs::create_dir_all(&target).expect("create output dir");
    target.push(&file);

    eprintln!("╔══════════════════════════════════════════════╗");
    eprintln!("║  NeoTrix Download Engine                     ║");
    eprintln!("╚══════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("  repo:   {}", repo);
    eprintln!("  file:   {}", file);
    eprintln!("  url:    {}", url);
    eprintln!("  output: {}", target.display());
    eprintln!("  chunks: {}", chunks);
    eprintln!("  resume: {}", resume);
    eprintln!();

    // 配置引擎
    let config = EngineConfig {
        chunk_count: chunks,
        enable_resume: resume,
        ..Default::default()
    };
    let engine = DownloadEngine::new(config);

    // 创建下载会话
    let mut session = DownloadSession::new(url, target, "model-download-cli".into());
    session.metadata.source = DownloadSource::HuggingFace;
    session.metadata.description = format!("{} / {}", repo, file);

    let start = SystemTime::now();

    // 执行下载
    match engine.download_session(&mut session).await {
        Ok(()) => {
            let elapsed = start.elapsed().unwrap_or_default();
            let size_mb = session.progress.downloaded as f64 / 1024.0 / 1024.0;
            let speed = if elapsed.as_secs() > 0 {
                size_mb / elapsed.as_secs_f64()
            } else {
                0.0
            };
            eprintln!();
            eprintln!("✓ download complete");
            eprintln!("  file:   {}", session.path.display());
            eprintln!("  size:   {:.1} MB", size_mb);
            eprintln!("  time:   {:.1}s", elapsed.as_secs_f64());
            eprintln!("  speed:  {:.1} MB/s", speed);
        }
        Err(e) => {
            eprintln!();
            eprintln!("✗ download failed: {}", e);
            eprintln!("  session: {}", session.id);
            eprintln!("  progress: {:.1}% ({}/{} bytes)",
                session.progress.percent,
                session.progress.downloaded,
                session.progress.total,
            );
            std::process::exit(1);
        }
    }
}

fn find_arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1).cloned())
}

fn print_usage() {
    eprintln!("usage: model-download --repo <owner/repo> --file <filename.gguf> [options]");
    eprintln!();
    eprintln!("options:");
    eprintln!("  --repo <owner/repo>    HuggingFace repository (e.g. HauhauCS/Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-MTP-GGUF)");
    eprintln!("  --file <name.gguf>     File to download (e.g. Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-IQ4_XS.gguf)");
    eprintln!("  --output <dir>         Output directory (default: ~/Downloads/neotrix/models)");
    eprintln!("  --chunks <n>           Parallel chunks (default: 8)");
    eprintln!("  --no-resume            Disable resume support");
    eprintln!("  -h, --help             Show this help");
    eprintln!();
    eprintln!("examples:");
    eprintln!("  model-download --repo HauhauCS/Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-MTP-GGUF \\");
    eprintln!("    --file Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-IQ4_XS.gguf");
}

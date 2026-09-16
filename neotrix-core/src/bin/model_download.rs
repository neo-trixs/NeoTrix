/// model-download — NeoTrix 自研下载引擎 CLI
/// 下载 HuggingFace GGUF 模型到本地，支持断点续传、分片并发、进度显示

use neotrix::l1_action::nt_io_download::{DownloadEngine, DownloadConfig, DownloadTask};
use std::path::PathBuf;
use std::time::SystemTime;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }

    let repo = find_arg(&args, "--repo").or_else(|| args.first().cloned());
    let file = find_arg(&args, "--file");
    let out_dir = find_arg(&args, "--output").unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/Downloads/neotrix/models", home)
    });
    let chunks: usize = find_arg(&args, "--chunks")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);

    let (repo, file) = match (repo, file) {
        (Some(r), Some(f)) => (r, f),
        _ => {
            eprintln!("error: --repo and --file are required");
            print_usage();
            std::process::exit(1);
        }
    };

    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        repo.trim_start_matches('/').trim_end_matches('/'),
        file
    );

    let mut target = PathBuf::from(&out_dir);
    std::fs::create_dir_all(&target).expect("create output dir");
    target.push(&file);

    eprintln!("╔══════════════════════════════════════════════╗");
    eprintln!("║  NeoTrix Download Engine                     ║");
    eprintln!("╚══════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("  repo:   {}", repo);
    eprintln!("  file:   {}", file);
    eprintln!("  output: {}", target.display());
    eprintln!("  chunks: {}", chunks);
    eprintln!();

    let config = DownloadConfig {
        max_concurrent: chunks,
        ..Default::default()
    };
    let engine = DownloadEngine::new(config);

    let task = DownloadTask {
        url,
        dest: target,
        priority: 128,
    };

    let start = SystemTime::now();
    let status = engine.download(&task).await;

    match status {
        neotrix::l1_action::nt_io_download::DownloadStatus::Completed { elapsed_secs, size_mb } => {
            eprintln!();
            eprintln!("✓ download complete");
            eprintln!("  size:  {:.1} MB", size_mb);
            eprintln!("  time:  {:.1}s", elapsed_secs);
            eprintln!("  speed: {:.1} MB/s", if elapsed_secs > 0.0 { size_mb / elapsed_secs } else { 0.0 });
        }
        neotrix::l1_action::nt_io_download::DownloadStatus::Failed(e) => {
            eprintln!("✗ download failed: {}", e);
            std::process::exit(1);
        }
        _ => {}
    }
}

fn find_arg(args: &[String], key: &str) -> Option<String> {
    args.iter().position(|a| a == key).and_then(|i| args.get(i + 1).cloned())
}

fn print_usage() {
    eprintln!("usage: model-download --repo <owner/repo> --file <filename.gguf> [options]");
    eprintln!();
    eprintln!("options:");
    eprintln!("  --repo <owner/repo>    HuggingFace repository");
    eprintln!("  --file <name.gguf>     File to download");
    eprintln!("  --output <dir>         Output directory (default: ~/Downloads/neotrix/models)");
    eprintln!("  --chunks <n>           Parallel chunks (default: 8)");
    eprintln!("  -h, --help             Show this help");
}

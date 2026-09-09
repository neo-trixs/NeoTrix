use neotrix::l1_action::nt_io::nt_io_download::{DownloadEngine, EngineConfig, DownloadSession};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let url = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: model-download <url> [output-path]");
        std::process::exit(1);
    });

    let default_dir = dirs::home_dir()
        .map(|h| h.join("Downloads/neotrix/models"))
        .unwrap_or_else(|| PathBuf::from("./models"));

    let output = std::env::args().nth(2).map(PathBuf::from).unwrap_or_else(|| {
        let filename = url.rsplit('/').next().unwrap_or("model.gguf");
        default_dir.join(filename)
    });

    std::fs::create_dir_all(output.parent().unwrap()).ok();

    let config = EngineConfig {
        chunk_count: 8,
        max_retries: 5,
        retry_base_secs: 2,
        timeout_secs: 300,
        enable_resume: true,
        enable_proxy: true,
        progress_interval_secs: 1,
    };

    let engine = DownloadEngine::new(config);
    let mut session = DownloadSession::new(url.clone(), output.clone(), "cli".to_string());

    println!("⬇ NeoTrix Download Engine");
    println!("  URL:    {}", url);
    println!("  Output: {}", output.display());
    println!("  Chunks: 8-way concurrent | Resume: ON");
    println!();

    let start = std::time::Instant::now();

    match engine.download_session(&mut session).await {
        Ok(()) => {
            let elapsed = start.elapsed();
            let size_mb = session.progress.total as f64 / 1024.0 / 1024.0;
            let speed = if elapsed.as_secs_f64() > 0.0 {
                size_mb / elapsed.as_secs_f64()
            } else {
                0.0
            };
            println!();
            println!("✅ Download complete!");
            println!("  Size:   {:.1} MB", size_mb);
            println!("  Time:   {:.1}s", elapsed.as_secs_f64());
            println!("  Speed:  {:.1} MB/s", speed);
            println!("  File:   {}", output.display());
        }
        Err(e) => {
            eprintln!("❌ Download failed: {:?}", e);
            std::process::exit(1);
        }
    }
}

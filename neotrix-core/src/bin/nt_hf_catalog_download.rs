//! nt_hf_catalog_download — HF datasets 广场目录元数据 → 项目 datasets/ 目录
//!
//! 经 neotrix 下载能力 [`download_to_file`] (SSRF guard + HEAD 预检 + `.tmp`
//! 原子落盘 + 断点续传 + 幂等跳过) 拉取目录级元数据 JSON，不走 curl/裸请求。
//!
//! 用法（项目根执行，数据只落项目内）：
//! ```sh
//! cargo run -p neotrix --bin neotrix-hf-catalog [--output datasets/hf_catalog]
//! ```
//!
//! 清单口径：2026-09-24 `https://huggingface.co/datasets` 首屏 trending 30 集快照。
//! 全量 1,061,044 集走 API 分页批处理（后续批次），本 bin 只覆盖页面可见集。

use neotrix::l4_emotion::nt_memory::nt_memory_kb::nt_http::{DownloadOptions, download_to_file};
use std::path::PathBuf;
use std::time::Duration;

/// 页面首屏 trending 30 集（2026-09-24 快照）。
const DATASET_IDS: &[&str] = &[
    "secemp9/arxiv-complete",
    "MoreThought/Fable-5.1-Max-Reasoning-Filtered-5000x",
    "wikimedia/wikipedia",
    "openbmb/UltraData-SFT-Agent-2609",
    "Yootta/World-SimReady-Home",
    "zgcagi/ZGCM-1-Data",
    "markov-ai/cad-1000-hours",
    "DeepMostInnovations/saas-sales-conversations",
    "eidon-ai/tracker-pov",
    "nyu-mll/glue",
    "venvoo/china-a-share-l2-level2-limit-order-book-tick-data",
    "malcolmrey/various",
    "ikala/tmmluplus",
    "OpenDataArena/Spark-234K",
    "FlyRank/internship-warehouse",
    "openbmb/UltraData-Code",
    "Harland/OmniVChat",
    "ZefanCai/Open-Jev",
    "Anthropic/hh-rlhf",
    "nvidia/PhysicalAI-Autonomous-Vehicles",
    "nvidia/OpenH-RF",
    "IFM/Code-Reasoning",
    "echel0nn1881/kimi-cyber-reasoning",
    "LocalLLaMA/typed-decisions",
    "ILSVRC/imagenet-1k",
    "HuggingFaceFW/fineweb",
    "IFM/TxT360-v2",
    "openai/gsm8k",
    "saidutta69/fable-5-premium",
    "openbmb/UltraData-RL-2609",
];

const API_BASE: &str = "https://huggingface.co/api/datasets/";
/// 单个目录元数据上限 64MB（实测最大卡约 4MB，留足余量）。
const MAX_BYTES: u64 = 64 * 1024 * 1024;
const ALLOWED_MIME: &[&str] = &["application/json"];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("usage: neotrix-hf-catalog [--output datasets/hf_catalog]");
        return;
    }
    let out_dir = find_arg(&args, "--output").unwrap_or_else(|| "datasets/hf_catalog".to_string());
    let out_path = PathBuf::from(&out_dir);
    if let Err(e) = std::fs::create_dir_all(&out_path) {
        eprintln!("error: create output dir {}: {e}", out_path.display());
        std::process::exit(1);
    }

    let mut ok = 0usize;
    let mut failed: Vec<String> = Vec::new();
    for id in DATASET_IDS {
        let url = format!("{API_BASE}{id}");
        let dest = out_path.join(format!("{}.json", id.replace('/', "_")));
        let opts = DownloadOptions {
            url: &url,
            dest: &dest,
            user_agent: None,
            allowed_mime_types: ALLOWED_MIME,
            max_bytes: MAX_BYTES,
            total_timeout: Some(Duration::from_secs(90)),
            proxy: None,
            proxy_pool: false,
            host: Some("huggingface.co"),
        };
        match download_to_file(&opts) {
            Ok(r) => {
                ok += 1;
                eprintln!(
                    "ok {} ({} bytes{})",
                    id,
                    r.bytes_written,
                    if r.resumed { ", resumed" } else { "" }
                );
            }
            Err(e) => {
                eprintln!("FAIL {id}: {e}");
                failed.push(id.to_string());
            }
        }
    }
    eprintln!("done: ok={ok} fail={}", failed.len());
    if !failed.is_empty() {
        std::process::exit(1);
    }
}

fn find_arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1).cloned())
}

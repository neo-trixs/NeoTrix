//! `nt-keywords` — 导出晶体核心的权威分词口径，供外部评测脚本调用。
//!
//! ## 为什么需要这个 bin（2026-09-28）
//! `scripts/ops/nt_jev_live_eval.py` 要测「词法判分器」的 top-1 准确率，
//! 且其注释明确写「复用 `nt_verify_sim.keywords`（**不重写**）—— 必须量的
//! 是核心真正在用的那个切分」。但那个 Python 模块已随 `2bbed32c` 删除，
//! 脚本**当前跑不起来**（断链 import）。
//!
//! ## 为什么不「在 Python 里重写一份」
//! `CrystalConsciousness::keywords`（`l5_cognition/nt_crystal_core/consciousness.rs:546`）
//! 是晶体核心的**权威分词**：它带 `strip_src_tag` 前处理 + 停用词表 +
//! 单字符过滤，且**同一口径被觉醒循环的新颖度验证复用**。Python 重写一份
//! 就会立刻产生第二套分词 —— 那正是本仓反复治的病（「第二份真源会漂」，
//! 参见 R-DISK-7 / R-SCAN-3）。而 crate 内另有 3 个同名 `keywords`
//! （`nt_crystal_task_fusion.rs:57` / `nt_shared_mind.rs:22,173`），
//! **实现各不相同** ⇒ 「哪个是权威」本身就是歧义源。
//!
//! ## 口径
//! ```text
//! stdin 每行一段文本  →  stdout 每行一个 JSON 数组（该行的 keywords，顺序同源）
//! ```
//! 例如：
//! ```sh
//! echo 'Rust 的所有权很安全' | cargo run -q -p neotrix --bin nt_keywords
//! # ["所有权", "很安全"]
//! ```
//!
//! ⚠️ 依赖 `CrystalConsciousness::keywords` 是 `pub(crate)`。本 bin 与它在同一 crate，
//! 故可调用。**若哪天把它降级为私有不 pub(crate)，本 bin 会编译失败** ——
//! 那正是我们要的：改动权威分词时，导出点会强制你同步。

use std::io::{self, BufRead, Write};

use neotrix::l5_cognition::nt_crystal_core::consciousness::CrystalConsciousness;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("nt-keywords: stdin read error: {e}");
                std::process::exit(1);
            }
        };
        let kws = CrystalConsciousness::keywords(&line);
        // 手工序列化：避免为一个 bin 引入 serde_json 到运行时路径。
        // 与 Rust 侧一致：内部词含 `"` 的极罕见，仍做基本转义。
        let mut parts = Vec::with_capacity(kws.len());
        for k in &kws {
            // replace 的模式参数是 `&str`（pattern），不是 char —— 写 '\\' 会被
            // 解释为「转义反斜杠」而编译失败（E0782-ish）。用 str 形式。
            let esc = k.replace('\\', "\\\\").replace('"', "\\\"");
            parts.push(format!("\"{esc}\""));
        }
        if writeln!(out, "[{}]", parts.join(",")).is_err() {
            eprintln!("nt-keywords: stdout write failed (pipe closed?)");
            std::process::exit(1);
        }
    }
}

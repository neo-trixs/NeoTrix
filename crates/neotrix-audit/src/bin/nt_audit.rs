//! `nt-audit` — 可脚本化审计 CLI (OCR `delegate/rules/session` 对应物).
//!
//! ```sh
//! nt-audit rules explain <path> [--layer custom --pattern 'src/**' --rule '...']
//! nt-audit diff-baseline <before.json> <after.json> [--reviewed a.rs,b.rs]
//! nt-audit filter-prompt   # 打印 review_filter prompt (喂给 LLM 后处理)
//! ```
//!
//! findings JSON schema: `[{path,category,severity,content,
//! suggestion?,existing_code?,start_line?,end_line?}]`.

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use neotrix_audit::{
    AuditFinding, Layer, RuleResolver, compare_baseline, review_filter_prompt,
};
use neotrix_audit::nt_rules::builtin_system_rules;

#[derive(Debug, Parser)]
#[command(name = "nt-audit", version, about = "NeoTrix deterministic audit core")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// 解释某路径命中的审计规则 (OCR `rules check` 对应物).
    Rules {
        #[command(subcommand)]
        cmd: RulesCmd,
    },
    /// 基线对比四桶 (OCR `session compare` 对应物).
    DiffBaseline {
        before: String,
        after: String,
        /// 本轮看过的文件 (逗号分隔).
        #[arg(long, default_value = "")]
        reviewed: String,
    },
    /// 打印误报抑制 prompt.
    FilterPrompt,
}

#[derive(Debug, Subcommand)]
enum RulesCmd {
    Explain {
        path: String,
        /// 追加 custom 层规则文本.
        #[arg(long, default_value = "")]
        rule: String,
        /// custom 层 glob.
        #[arg(long, default_value = "**")]
        pattern: String,
    },
}

fn main() {
    if let Err(err) = real_main() {
        eprintln!("nt-audit: {err}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Rules { cmd: RulesCmd::Explain { path, rule, pattern } } => {
            let mut resolver = RuleResolver::new();
            if !rule.trim().is_empty() {
                resolver.add(Layer::Custom, pattern.trim(), rule.trim());
            }
            for entry in builtin_system_rules() {
                resolver.add(Layer::System, &entry.pattern, &entry.rule);
            }
            match resolver.resolve(path.trim()) {
                Some(hit) => {
                    println!("layer={} pattern={}", hit.layer.as_str(), hit.pattern);
                    println!("---");
                    println!("{}", hit.text);
                }
                None => println!("no rule matched"),
            }
            Ok(())
        }
        Cmd::DiffBaseline { before, after, reviewed } => {
            let before_findings = read_findings(&before)?;
            let after_findings = read_findings(&after)?;
            let reviewed_list: Vec<String> = reviewed
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect();
            let diff = compare_baseline(&before_findings, &after_findings, &reviewed_list);
            println!(
                "{}",
                serde_json::json!({
                    "new": diff.new,
                    "persisting": diff.persisting,
                    "resolved": diff.resolved,
                    "not_reviewed": diff.not_reviewed,
                })
            );
            Ok(())
        }
        Cmd::FilterPrompt => {
            print!("{}", review_filter_prompt());
            Ok(())
        }
    }
}

fn read_findings(path: &str) -> Result<Vec<AuditFinding>, String> {
    let raw = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

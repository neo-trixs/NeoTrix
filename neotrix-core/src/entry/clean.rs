//! `neotrix clean` CLI 入口 (吸收 PureMac CLI `puremac clean`)
//!
//! 子命令语义对齐:
//! - 默认 / `all` — 扫描 dev,junk,ai,trash
//! - `dev|junk|ai|trash` — 单类别
//! - `--dry-run` — 只报告，不删
//! - `--json` — 机读输出，不删
//! - `--force` — 跳过交互确认 (非 TTY 必需)
//! - `--exclude <path>` / `exclude list|add|rm` — 排除表

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use neotrix::l1_action::nt_act::nt_act_cleanup::{
    catalog::{CATEGORY_TITLES, VALID_IDS},
    cleaning_engine::{CleaningConfig, CleaningEngine},
    exclusions::CleanupExclusions,
    format_size,
    scan_engine::{scan_category, CategoryResult},
};

/// 主入口
pub fn run_clean(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return run_pipeline(&["all".into()], CleanFlags::default());
    }
    match args[0].as_str() {
        "exclude" => run_exclude(&args[1..]),
        "help" | "-h" | "--help" => {
            println!("{}", help_text());
            Ok(())
        }
        _ => run_pipeline(args, CleanFlags::default()),
    }
}

fn help_text() -> &'static str {
    "Usage: neotrix clean [dev|junk|ai|trash|all] [--dry-run] [--json] [--force]\n\
     \n\
     Categories:\n  dev    package-manager & build-tool caches\n  junk   user logs & Xcode junk\n  ai     AI-tool caches & logs\n  trash  empty Trash\n  all    dev+junk+ai+trash (default)\n\
     \n\
     Flags:\n  --dry-run   scan and report only\n  --json      machine-readable JSON (never deletes)\n  --force     skip interactive confirm\n\
     \n\
     Exclusions:\n  neotrix clean exclude list\n  neotrix clean exclude add <path>\n  neotrix clean exclude rm <path>"
}

#[derive(Debug, Clone, Default)]
struct CleanFlags {
    dry_run: bool,
    json: bool,
    force: bool,
}

fn parse_flags(args: &[String]) -> (Vec<String>, CleanFlags) {
    let mut cats = Vec::new();
    let mut f = CleanFlags::default();
    for a in args {
        match a.as_str() {
            "--dry-run" => f.dry_run = true,
            "--json" => f.json = true,
            "--force" => f.force = true,
            other if other.starts_with("--") => {
                eprintln!("warning: unknown flag {other} ignored");
            }
            other => cats.push(other.to_string()),
        }
    }
    if cats.is_empty() {
        cats.push("all".into());
    }
    (cats, f)
}

fn expand_categories(cats: &[String]) -> Result<Vec<&'static str>, String> {
    let mut out = Vec::new();
    for c in cats {
        match c.as_str() {
            "all" => out.extend_from_slice(VALID_IDS),
            id if VALID_IDS.contains(&id) => {
                // static lifetime: VALID_IDS is &'static str
                let s = VALID_IDS
                    .iter()
                    .find(|x| **x == id)
                    .copied()
                    .ok_or_else(|| format!("unknown category {id}"))?;
                if !out.contains(&s) {
                    out.push(s);
                }
            }
            other => {
                return Err(format!(
                    "Unknown category '{other}'. Use one of: {}, all",
                    VALID_IDS.join(", ")
                ))
            }
        }
    }
    Ok(out)
}

fn title_of(id: &str) -> &str {
    CATEGORY_TITLES
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, t)| *t)
        .unwrap_or(id)
}

fn run_pipeline(args: &[String], base: CleanFlags) -> Result<(), String> {
    let (raw, flags) = parse_flags(args);
    let flags = CleanFlags {
        dry_run: base.dry_run || flags.dry_run,
        json: base.json || flags.json,
        force: base.force || flags.force,
    };
    let ids = expand_categories(&raw)?;

    let home = dirs::home_dir().ok_or_else(|| "cannot resolve home".to_string())?;
    let mut ex_store = CleanupExclusions::load_default();
    let excluded: Vec<PathBuf> = ex_store.paths().to_vec();

    if !flags.json {
        eprintln!(
            "Scanning {} …",
            ids.iter()
                .map(|i| title_of(i))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let results: Vec<CategoryResult> = ids
        .iter()
        .filter_map(|id| {
            let r = scan_category(id, title_of(id), &home, &excluded);
            if r.items.is_empty() {
                None
            } else {
                Some(r)
            }
        })
        .collect();

    if flags.json {
        return emit_json(&results);
    }

    if results.is_empty() {
        println!("No cleanup candidates found in readable locations.");
        return Ok(());
    }

    // 打印扫描结果
    let mut grand_all = 0u64;
    let mut grand_sel = 0u64;
    println!("╭─ Cleanup Scan ─────────────────────────────────────╮");
    for cat in &results {
        println!(
            "│ {:<16} {:>6} items  all {}  default {}",
            cat.title,
            cat.items.len(),
            format_size(cat.total_bytes()),
            format_size(cat.selected_bytes())
        );
        grand_all += cat.total_bytes();
        grand_sel += cat.selected_bytes();
        // 最多列前 8 项
        for item in cat.items.iter().take(8) {
            println!(
                "│   {}  {}",
                format_size(item.size_bytes),
                item.path.display()
            );
        }
        if cat.items.len() > 8 {
            println!("│   … +{} more", cat.items.len() - 8);
        }
    }
    println!(
        "│ TOTAL  all {}  default-selected {}",
        format_size(grand_all),
        format_size(grand_sel)
    );
    println!("╰────────────────────────────────────────────────────╯");

    let selected: Vec<_> = results
        .iter()
        .flat_map(|c| c.items.iter().filter(|i| i.selected).cloned())
        .collect();

    if selected.is_empty() {
        println!("No items selected. Nothing removed.");
        return Ok(());
    }

    if flags.dry_run {
        let eng = CleaningEngine::new(CleaningConfig {
            dry_run: true,
            use_allowlist: true,
            home: home.clone(),
            excluded: excluded.clone(),
        });
        let out = eng.clean_items(&selected);
        println!(
            "dry-run: would clean {} items ({}), skipped {}, failed {}",
            out.items_cleaned,
            format_size(out.freed_bytes),
            out.skipped.len(),
            out.failed.len()
        );
        return Ok(());
    }

    // 确认
    if !flags.force {
        if !std::io::stdin().is_terminal() {
            return Err(
                "Non-interactive terminal: re-run with --force to delete, or --dry-run/--json to report only."
                    .into(),
            );
        }
        print!(
            "Remove {} items ({})? [y/N] ",
            selected.len(),
            format_size(selected.iter().map(|i| i.size_bytes).sum::<u64>())
        );
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() {
            return Err("read failed".into());
        }
        let ans = line.trim().to_ascii_lowercase();
        if ans != "y" && ans != "yes" {
            println!("Cancelled. Nothing removed.");
            return Ok(());
        }
    }

    let eng = CleaningEngine::new(CleaningConfig {
        dry_run: false,
        use_allowlist: true,
        home: home.clone(),
        excluded: excluded.clone(),
    });
    let out = eng.clean_items(&selected);
    println!(
        "Cleaned {} items, freed {}",
        out.items_cleaned,
        format_size(out.freed_bytes)
    );
    if !out.skipped.is_empty() {
        println!("Skipped {}:", out.skipped.len());
        for (p, r) in out.skipped.iter().take(10) {
            println!("  {r}: {}", p.display());
        }
    }
    if !out.failed.is_empty() {
        println!("Failed {}:", out.failed.len());
        for (p, e) in out.failed.iter().take(10) {
            println!("  {e}: {}", p.display());
        }
        return Err(format!("{} item(s) failed", out.failed.len()));
    }
    Ok(())
}

fn emit_json(results: &[CategoryResult]) -> Result<(), String> {
    let cats: Vec<serde_json::Value> = results
        .iter()
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "title": c.title,
                "totalBytes": c.total_bytes(),
                "selectedBytes": c.selected_bytes(),
                "itemCount": c.items.len(),
                "items": c.items.iter().map(|i| serde_json::json!({
                    "path": i.path.to_string_lossy(),
                    "sizeBytes": i.size_bytes,
                    "selected": i.selected,
                    "tool": i.tool,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let out = serde_json::json!({
        "totalBytes": results.iter().map(|c| c.total_bytes()).sum::<u64>(),
        "categories": cats,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn run_exclude(args: &[String]) -> Result<(), String> {
    let mut store = CleanupExclusions::load_default();
    match args.first().map(String::as_str) {
        None | Some("list") => {
            if store.paths().is_empty() {
                println!("(no exclusions)");
            } else {
                for p in store.paths() {
                    println!("{}", p.display());
                }
            }
            Ok(())
        }
        Some("add") => {
            let path = args
                .get(1)
                .ok_or_else(|| "usage: neotrix clean exclude add <path>".to_string())?;
            let p = Path::new(path);
            if store.add(p).map_err(|e| e.to_string())? {
                println!("excluded {}", p.display());
            } else {
                println!("already excluded or invalid: {}", p.display());
            }
            Ok(())
        }
        Some("rm") | Some("remove") => {
            let path = args
                .get(1)
                .ok_or_else(|| "usage: neotrix clean exclude rm <path>".to_string())?;
            let p = Path::new(path);
            if store.remove(p).map_err(|e| e.to_string())? {
                println!("removed exclusion {}", p.display());
            } else {
                println!("not excluded: {}", p.display());
            }
            Ok(())
        }
        Some(other) => Err(format!(
            "unknown exclude subcommand {other}; use list|add|rm"
        )),
    }
}

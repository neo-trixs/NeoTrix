use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};
use nt_migrate::*;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "scan" => cmd_scan(&args[2..]),
        "extract" => cmd_extract(&args[2..]),
        "convert" => cmd_convert(&args[2..]),
        "stats" => cmd_stats(&args[2..]),
        "migrate" => cmd_migrate(&args[2..]),
        "help" | "--help" | "-h" => { print_help(); Ok(()) }
        other => {
            log::error!("Unknown command: {}. Use 'help' for usage.", other);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    log::error!("nt-migrate — Automated test migration tool");
    log::error!("");
    log::error!("USAGE:");
    log::error!("  nt-migrate scan <path>");
    log::error!("      Scan Rust files for test modules (prints table)");
    log::error!("");
    log::error!("  nt-migrate extract <file.rs> --test <name>");
    log::error!("      Extract a test module as .nt YAML to stdout");
    log::error!("");
    log::error!("  nt-migrate convert <file.rs> --test <name> --output <file.nt>");
    log::error!("      Convert a test module to a .nt file with imports and code");
    log::error!("");
    log::error!("  nt-migrate migrate <src_path> --output <test_suites_dir>");
    log::error!("      Batch-migrate all test modules to .nt files");
    log::error!("");
    log::error!("  nt-migrate stats <path>");
    log::error!("      Show migration statistics vs existing .nt suites");
}

// ---------------------------------------------------------------------------
// scan
// ---------------------------------------------------------------------------

fn cmd_scan(args: &[String]) -> Result<()> {
    let path = get_pos_arg(args, 0).context("Usage: nt-migrate scan <path>")?;
    let modules = scan_for_test_modules(path)?;

    if modules.is_empty() {
        log::info!("No test modules found in {}", path.display());
        return Ok(());
    }

    for m in &modules {
        log::info!("File:     {}", m.file.display());
        log::info!("Module:   {}", m.mod_name);
        log::info!("Tests:    {}", m.test_fns.len());
        for name in &m.test_fns {
            log::info!("  - {}", name);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// extract
// ---------------------------------------------------------------------------

fn cmd_extract(args: &[String]) -> Result<()> {
    let (file, mod_name) = parse_key_value_args(args, "--test")?;
    let suite = build_nt_suite(&file, &mod_name)?;
    let yaml = serde_yaml::to_string(&suite)?;
    log::info!("{}", yaml);
    Ok(())
}

// ---------------------------------------------------------------------------
// convert
// ---------------------------------------------------------------------------

fn cmd_convert(args: &[String]) -> Result<()> {
    let mut file: Option<PathBuf> = None;
    let mut mod_name: Option<String> = None;
    let mut output: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--test" => {
                i += 1;
                mod_name = Some(args.get(i).context("Missing value for --test")?.clone());
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(args.get(i).context("Missing value for --output")?));
            }
            s if !s.starts_with("--") => {
                file = Some(PathBuf::from(s));
            }
            _ => {
                anyhow::bail!("Unexpected argument: {}", args[i]);
            }
        }
        i += 1;
    }

    let file = file.context("Missing <file.rs> argument")?;
    let mod_name = mod_name.context("Missing --test argument")?;
    let output = output.context("Missing --output argument")?;

    let suite = build_nt_suite(&file, &mod_name)?;
    let yaml = serde_yaml::to_string(&suite)?;
    fs::write(&output, &yaml)
        .with_context(|| format!("Failed to write {}", output.display()))?;
    log::info!("Wrote {} ({} tests)", output.display(), suite.tests.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// stats
// ---------------------------------------------------------------------------

fn cmd_stats(args: &[String]) -> Result<()> {
    let path = get_pos_arg(args, 0).context("Usage: nt-migrate stats <path>")?;
    let modules = scan_for_test_modules(path)?;
    let total_modules = modules.len();
    let total_tests: usize = modules.iter().map(|m| m.test_fns.len()).sum();

    let suites_dir = find_suites_dir()?;
    let existing_nt: Vec<String> = fs::read_dir(&suites_dir)
        .context("Cannot read test_suites directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "nt").unwrap_or(false))
        .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect();

    let mut migrated = 0;
    let mut remaining_modules = Vec::new();
    for m in &modules {
        let stem = m.file.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let match_key = stem.replace('-', "_");
        if existing_nt.iter().any(|nt| match_key.contains(nt.as_str()) || nt.contains(&match_key)) {
            migrated += 1;
        } else {
            remaining_modules.push(m);
        }
    }

    log::info!("=== Migration Statistics ===");
    log::info!("Source path:       {}", path.display());
    log::info!("Test modules:      {}", total_modules);
    log::info!("Test functions:    {}", total_tests);
    log::info!("Already migrated:  {}", migrated);
    log::info!("Remaining:         {}", total_modules.saturating_sub(migrated));
    if !remaining_modules.is_empty() {
        log::info!("Not yet migrated:");
        for m in &remaining_modules {
            log::info!("  {} :: {}", m.file.display(), m.mod_name);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// migrate
// ---------------------------------------------------------------------------

fn cmd_migrate(args: &[String]) -> Result<()> {
    let src_path = get_pos_arg(args, 0).context("Usage: nt-migrate migrate <src_path> --output <dir>")?;
    let output_dir = parse_named_arg(args, "--output")
        .context("Usage: nt-migrate migrate <src_path> --output <dir>")?;
    let output_dir = PathBuf::from(output_dir);

    let modules = scan_for_test_modules(src_path)?;
    if modules.is_empty() {
        log::info!("No test modules found in {}", src_path.display());
        return Ok(());
    }

    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Cannot create output directory {}", output_dir.display()))?;

    let mut count = 0;
    for module in &modules {
        let suite = build_nt_suite(&module.file, &module.mod_name)?;
        let suite_name = derive_suite_name(&module.file, &module.mod_name);
        let output_path = output_dir.join(format!("{}.nt", suite_name));

        let yaml = serde_yaml::to_string(&suite)?;
        fs::write(&output_path, &yaml)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;

        log::info!("  {} → {}.nt ({} tests)", module.mod_name, suite_name, module.test_fns.len());
        count += 1;
    }

    log::info!("Migrated {} test module(s) to {}", count, output_dir.display());
    Ok(())
}



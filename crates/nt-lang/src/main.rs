use std::path::Path;
use std::process;

use nt_lang::compile_ne_file;

fn init_logger() {
    // Use RUST_LOG env var or default to "info"
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let _ = env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", &filter))
        .format_timestamp(None)
        .try_init();
}

fn main() {
    init_logger();
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        log::error!("Usage: nt-lang build <file.nt>");
        log::error!("       nt-lang build-all");
        log::error!("       nt-lang build-ne <file.ne>");
        log::error!("       nt-lang check-ne <file.ne>");
        process::exit(1);
    }

    match args[1].as_str() {
        "build" => {
            if args.len() < 3 {
                log::error!("Usage: nt-lang build <file.nt>");
                process::exit(1);
            }
            let input = Path::new(&args[2]);
            if let Err(e) = build_one(input) {
                log::error!("Error: {}", e);
                process::exit(1);
            }
        }
        "build-all" => {
            let suites_dir = find_suites_dir();
            if let Err(e) = build_all(&suites_dir) {
                log::error!("Error: {}", e);
                process::exit(1);
            }
        }
        "build-ne" => {
            if args.len() < 3 {
                log::error!("Usage: nt-lang build-ne <file.ne>");
                process::exit(1);
            }
            let input = Path::new(&args[2]);
            if let Err(e) = build_ne_one(input) {
                log::error!("Error: {}", e);
                process::exit(1);
            }
        }
        "check-ne" => {
            if args.len() < 3 {
                log::error!("Usage: nt-lang check-ne <file.ne>");
                process::exit(1);
            }
            let input = Path::new(&args[2]);
            match nt_lang::check_ne_file(input) {
                Ok(()) => log::info!("{} — OK", input.display()),
                Err(diags) => {
                    for d in &diags {
                        log::error!("{}: {}", d.location, d.message);
                    }
                    process::exit(1);
                }
            }
        }
        other => {
            log::error!("Unknown command: {}. Use 'build', 'build-all', 'build-ne', or 'check-ne'", other);
            process::exit(1);
        }
    }
}

fn build_one(input: &Path) -> Result<(), String> {
    let suite = nt_lang::test_parser::parse_file(input)?;
    let output_path = output_path_for(&suite.name);

    let code = nt_lang::codegen::rust::generate(&suite);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create output dir {}: {}", parent.display(), e))?;
    }

    std::fs::write(&output_path, &code)
        .map_err(|e| format!("Cannot write {}: {}", output_path.display(), e))?;

    log::info!(
        "Generated {} ({} tests, {} bytes)",
        output_path.display(),
        suite.tests.len(),
        code.len(),
    );
    Ok(())
}

fn build_all(suites_dir: &Path) -> Result<(), String> {
    if !suites_dir.exists() {
        return Err(format!("Test suites directory not found: {}", suites_dir.display()));
    }

    let entries = std::fs::read_dir(suites_dir)
        .map_err(|e| format!("Cannot read {}: {}", suites_dir.display(), e))?;

    let mut count = 0;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "nt") {
            build_one(&path)?;
            count += 1;
        }
    }

    log::info!("Processed {} test suite(s) from {}", count, suites_dir.display());
    Ok(())
}

fn output_path_for(suite_name: &str) -> std::path::PathBuf {
    let tests_dir = Path::new("neotrix-core").join("tests");
    tests_dir.join(format!("{}.rs", suite_name))
}

fn build_ne_one(input: &Path) -> Result<(), String> {
    let code = compile_ne_file(input).map_err(|diags| {
        diags.iter()
            .map(|d| format!("{}: {}", d.location, d.message))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("ne_output");
    let output_path = std::path::PathBuf::from(format!("{}.rs", stem));
    std::fs::write(&output_path, &code)
        .map_err(|e| format!("Cannot write {}: {}", output_path.display(), e))?;

    log::info!("Generated {} ({} bytes)", output_path.display(), code.len());
    Ok(())
}

fn find_suites_dir() -> std::path::PathBuf {
    let candidates = [
        Path::new("test_suites"),
        Path::new("neotrix-core/test_suites"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.to_path_buf();
        }
    }
    Path::new("neotrix-core/test_suites").to_path_buf()
}

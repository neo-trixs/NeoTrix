use std::path::Path;
use std::process;

mod ir;
mod test_parser;
mod codegen;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: nt-lang build <file.nt>");
        eprintln!("       nt-lang build-all");
        process::exit(1);
    }

    match args[1].as_str() {
        "build" => {
            if args.len() < 3 {
                eprintln!("Usage: nt-lang build <file.nt>");
                process::exit(1);
            }
            let input = Path::new(&args[2]);
            if let Err(e) = build_one(input) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "build-all" => {
            let suites_dir = find_suites_dir();
            if let Err(e) = build_all(&suites_dir) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        other => {
            eprintln!("Unknown command: {}. Use 'build' or 'build-all'", other);
            process::exit(1);
        }
    }
}

fn build_one(input: &Path) -> Result<(), String> {
    let suite = test_parser::parse_file(input)?;
    let output_path = output_path_for(&suite.name);

    let code = codegen::rust::generate(&suite);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create output dir {}: {}", parent.display(), e))?;
    }

    std::fs::write(&output_path, &code)
        .map_err(|e| format!("Cannot write {}: {}", output_path.display(), e))?;

    println!(
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
        if path.extension().map_or(false, |e| e == "nt") {
            build_one(&path)?;
            count += 1;
        }
    }

    println!("Processed {} test suite(s) from {}", count, suites_dir.display());
    Ok(())
}

fn output_path_for(suite_name: &str) -> std::path::PathBuf {
    let tests_dir = Path::new("neotrix-core").join("tests");
    tests_dir.join(format!("{}.rs", suite_name))
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

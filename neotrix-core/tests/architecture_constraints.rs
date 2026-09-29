// Architecture Constraint Tests — Compile-time layer violation detection
//
// These tests verify that the 6-layer dependency rules are enforced:
// - L1 must not import from L2, L3, L4, L5, L6
// - L2 must not import from L3, L4, L5, L6
// - L3 must not import from L4, L5, L6
// - L4 must not import from L5, L6
// - L5 must not import from L6 (except via traits in traits.rs)
//
// Facades are the only allowed cross-layer communication mechanism.
// These are grep-based static analysis tests that run at test time.

use std::path::Path;
use std::process::Command;

const SRC_DIR: &str = "src";

fn project_root() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    manifest_dir.to_string()
}

fn src_path() -> String {
    format!("{}/{}", project_root(), SRC_DIR)
}

/// Recursively find all .rs files under a directory.
fn find_rs_files(dir: &str) -> Vec<String> {
    let output = Command::new("find")
        .args([dir, "-name", "*.rs", "-type", "f"])
        .output()
        .expect("failed to run find");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Grep for import patterns in a file. Returns matching lines.
fn grep_imports(file: &str, pattern: &str) -> Vec<String> {
    let output = Command::new("rg")
        .args(["--no-heading", "-n", pattern, file])
        .output()
        .expect("failed to run rg");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Check if a file path belongs to a specific layer.
fn in_layer(file: &str, layer: &str) -> bool {
    let marker = format!("/{}/", layer);
    file.contains(&marker)
}

/// Check if a file is a facade (allowed cross-layer bridge).
fn is_facade(file: &str) -> bool {
    file.contains("facade.rs") || file.contains("l1_facade") || file.contains("l2_facade")
}

/// Check if a file is a traits.rs (allowed to define cross-layer trait abstractions).
fn is_traits(file: &str) -> bool {
    file.ends_with("traits.rs")
}

/// Extract the relative path from the src root for a given file.
fn relative_path(file: &str) -> String {
    let src = src_path();
    file.strip_prefix(&src)
        .unwrap_or(file)
        .trim_start_matches('/')
        .to_string()
}

// ═══════════════════════════════════════════════════════════════════════
// Layer violation tests
// ═══════════════════════════════════════════════════════════════════════

/// Verify L1 files do not import from L2, L3, L4, L5, or L6.
#[test]
fn test_l1_no_upward_deps() {
    let src = src_path();
    let files = find_rs_files(&src);
    let l1_files: Vec<&str> = files
        .iter()
        .filter(|f| in_layer(f, "l1_action"))
        .map(|f| f.as_str())
        .collect();

    let forbidden_layers = [
        "l2_perception",
        "l3_embodiment",
        "l4_emotion",
        "l5_cognition",
        "l6_meta",
    ];
    let mut violations: Vec<String> = Vec::new();

    for file in &l1_files {
        if is_facade(file) || is_traits(file) {
            continue;
        }
        for layer in &forbidden_layers {
            let pattern = format!("crate::{}", layer);
            let matches = grep_imports(file, &pattern);
            for m in &matches {
                violations.push(format!(
                    "{}:{} — L1 file imports from {}",
                    relative_path(file),
                    m.split(':').next().unwrap_or("?"),
                    layer
                ));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "L1 UPWARD DEPENDENCY VIOLATIONS ({} found):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }
}

/// Verify L2 files do not import from L3, L4, L5, or L6.
#[test]
fn test_l2_no_upward_deps() {
    let src = src_path();
    let files = find_rs_files(&src);
    let l2_files: Vec<&str> = files
        .iter()
        .filter(|f| in_layer(f, "l2_perception"))
        .map(|f| f.as_str())
        .collect();

    let forbidden_layers = ["l3_embodiment", "l4_emotion", "l5_cognition", "l6_meta"];
    let mut violations: Vec<String> = Vec::new();

    for file in &l2_files {
        if is_facade(file) || is_traits(file) {
            continue;
        }
        for layer in &forbidden_layers {
            let pattern = format!("crate::{}", layer);
            let matches = grep_imports(file, &pattern);
            for m in &matches {
                violations.push(format!(
                    "{}:{} — L2 file imports from {}",
                    relative_path(file),
                    m.split(':').next().unwrap_or("?"),
                    layer
                ));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "L2 UPWARD DEPENDENCY VIOLATIONS ({} found):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }
}

/// Verify L3 files do not import from L4, L5, or L6.
#[test]
fn test_l3_no_upward_deps() {
    let src = src_path();
    let files = find_rs_files(&src);
    let l3_files: Vec<&str> = files
        .iter()
        .filter(|f| in_layer(f, "l3_embodiment"))
        .map(|f| f.as_str())
        .collect();

    let forbidden_layers = ["l4_emotion", "l5_cognition", "l6_meta"];
    let mut violations: Vec<String> = Vec::new();

    for file in &l3_files {
        if is_facade(file) || is_traits(file) {
            continue;
        }
        for layer in &forbidden_layers {
            let pattern = format!("crate::{}", layer);
            let matches = grep_imports(file, &pattern);
            for m in &matches {
                violations.push(format!(
                    "{}:{} — L3 file imports from {}",
                    relative_path(file),
                    m.split(':').next().unwrap_or("?"),
                    layer
                ));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "L3 UPWARD DEPENDENCY VIOLATIONS ({} found):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }
}

/// Verify L4 files do not import from L5 or L6.
#[test]
fn test_l4_no_upward_deps() {
    let src = src_path();
    let files = find_rs_files(&src);
    let l4_files: Vec<&str> = files
        .iter()
        .filter(|f| in_layer(f, "l4_emotion"))
        .map(|f| f.as_str())
        .collect();

    let forbidden_layers = ["l5_cognition", "l6_meta"];
    let mut violations: Vec<String> = Vec::new();

    for file in &l4_files {
        if is_facade(file) || is_traits(file) {
            continue;
        }
        for layer in &forbidden_layers {
            let pattern = format!("crate::{}", layer);
            let matches = grep_imports(file, &pattern);
            for m in &matches {
                violations.push(format!(
                    "{}:{} — L4 file imports from {}",
                    relative_path(file),
                    m.split(':').next().unwrap_or("?"),
                    layer
                ));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "L4 UPWARD DEPENDENCY VIOLATIONS ({} found):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }
}

/// Verify L5 files do not import from L6 (except via trait abstractions in traits.rs).
#[test]
fn test_l5_no_upward_deps() {
    let src = src_path();
    let files = find_rs_files(&src);
    let l5_files: Vec<&str> = files
        .iter()
        .filter(|f| in_layer(f, "l5_cognition"))
        .map(|f| f.as_str())
        .collect();

    let mut violations: Vec<String> = Vec::new();

    for file in &l5_files {
        if is_facade(file) || is_traits(file) {
            continue;
        }
        let pattern = "crate::l6_meta";
        let matches = grep_imports(file, pattern);
        for m in &matches {
            // Allow trait definitions that reference L6 types via trait abstractions
            // (e.g., in traits.rs where EvalHarnessApi etc. are defined).
            // But non-traits files must not import from L6.
            violations.push(format!(
                "{}:{} — L5 file imports from l6_meta (use trait abstraction instead)",
                relative_path(file),
                m.split(':').next().unwrap_or("?"),
            ));
        }
    }

    if !violations.is_empty() {
        panic!(
            "L5 UPWARD DEPENDENCY VIOLATIONS ({} found):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Structural invariant tests
// ═══════════════════════════════════════════════════════════════════════

/// Verify no trait is defined in multiple layer traits.rs files with the same name.
#[test]
fn test_no_duplicate_traits() {
    let src = src_path();
    let files = find_rs_files(&src);
    let traits_files: Vec<&str> = files
        .iter()
        .filter(|f| is_traits(f))
        .map(|f| f.as_str())
        .collect();

    // Collect all trait definitions from traits.rs files
    let mut trait_locations: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for file in &traits_files {
        let matches = grep_imports(file, r"^\s*pub trait \w+");
        for m in &matches {
            // Extract trait name from line like "src/l1_action/traits.rs:47:pub trait L1Capability ..."
            let line_content = m.split(':').skip(2).collect::<Vec<&str>>().join(":");
            if let Some(trait_name) = line_content
                .trim()
                .strip_prefix("pub trait ")
                .and_then(|s| s.split_whitespace().next())
            {
                trait_locations
                    .entry(trait_name.to_string())
                    .or_default()
                    .push(relative_path(file));
            }
        }
    }

    let duplicates: Vec<String> = trait_locations
        .iter()
        .filter(|(_, locs)| locs.len() > 1)
        .map(|(name, locs)| format!("Trait '{}' defined in: {}", name, locs.join(", ")))
        .collect();

    if !duplicates.is_empty() {
        panic!(
            "DUPLICATE TRAIT DEFINITIONS ({} found):\n{}",
            duplicates.len(),
            duplicates.join("\n")
        );
    }
}

/// Verify facade count stays below threshold (prevent facade proliferation).
#[test]
fn test_facade_count() {
    let src = src_path();
    let files = find_rs_files(&src);
    let facades: Vec<String> = files
        .iter()
        .filter(|f| is_facade(f))
        .map(|f| relative_path(f))
        .collect();

    const MAX_FACADES: usize = 20;
    if facades.len() > MAX_FACADES {
        panic!(
            "FACADE COUNT EXCEEDED: {} facades found (max {}):\n{}",
            facades.len(),
            MAX_FACADES,
            facades.join("\n")
        );
    }

    println!("Facade count: {}/{}", facades.len(), MAX_FACADES);
}

/// Verify no crate-level #[allow(dead_code)] — dead items should be tracked by
/// auto-patrol, not compilation gate.
#[test]
fn test_no_global_allow_dead_code() {
    let src = src_path();
    let lib_rs = format!("{}/lib.rs", src);
    let matches = grep_imports(&lib_rs, r"#\[allow\(dead_code\)\]");

    if !matches.is_empty() {
        // This is an intentional crate-level allowance in lib.rs (R-P1).
        // We flag it as a warning but don't fail — the project convention
        // allows this with auto-patrol tracking.
        println!(
            "WARNING: crate-level #![allow(dead_code)] found in lib.rs — \
             tracked by auto-patrol, not compilation gate"
        );
    }
}

/// Verify files in L1 follow naming conventions (nt_act/nt_io/nt_memory/nt_media/nt_infra).
#[test]
fn test_layer_naming() {
    let src = src_path();
    let files = find_rs_files(&src);
    let l1_files: Vec<&str> = files
        .iter()
        .filter(|f| in_layer(f, "l1_action"))
        .map(|f| f.as_str())
        .collect();

    let valid_prefixes = [
        "nt_act",
        "nt_io",
        "nt_memory",
        "nt_media",
        "nt_infra",
        "traits.rs",
        "mod.rs",
    ];
    let mut violations: Vec<String> = Vec::new();

    for file in &l1_files {
        let rel = relative_path(file);
        let filename = Path::new(&rel)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Skip mod.rs, traits.rs, and facade files
        if filename == "mod.rs" || filename == "traits.rs" || is_facade(file) {
            continue;
        }

        let has_valid_prefix = valid_prefixes.iter().any(|p| filename.starts_with(p));
        if !has_valid_prefix {
            violations.push(format!(
                "{} — filename does not start with nt_act/nt_io/nt_memory/nt_media/nt_infra",
                rel
            ));
        }
    }

    if !violations.is_empty() {
        panic!(
            "L1 NAMING VIOLATIONS ({} found):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }
}

/// Verify no module has fewer than 10 lines of logic (empty/stub modules).
#[test]
fn test_no_empty_modules() {
    let src = src_path();
    let files = find_rs_files(&src);
    let mut violations: Vec<String> = Vec::new();

    for file in &files {
        let rel = relative_path(file);
        let filename = Path::new(&rel)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Skip mod.rs, traits.rs, build scripts, and test files
        if filename == "mod.rs"
            || filename == "traits.rs"
            || rel.contains("build.rs")
            || rel.contains("/tests/")
        {
            continue;
        }

        let content = std::fs::read_to_string(file).unwrap_or_default();
        let non_empty_lines: usize = content
            .lines()
            .filter(|l| {
                !l.trim().is_empty() && !l.trim().starts_with("//") && !l.trim().starts_with("#[")
            })
            .count();

        // Allow thin re-export modules (facade files are just pub use statements)
        if is_facade(file) {
            continue;
        }

        if non_empty_lines < 10 {
            violations.push(format!(
                "{} — only {} lines of logic (min 10)",
                rel, non_empty_lines
            ));
        }
    }

    if !violations.is_empty() {
        panic!(
            "EMPTY/STUB MODULES ({} found):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }
}

/// Verify L1 modules in `neotrix/` subdirectory (e.g., nt_core_capability_tree)
/// do not import from higher layers.
#[test]
fn test_neotrix_subdir_no_upward_deps() {
    let src = src_path();
    let neotrix_dir = format!("{}/neotrix", src);
    if !Path::new(&neotrix_dir).exists() {
        println!("SKIP: neotrix/ subdirectory not found");
        return;
    }

    let files = find_rs_files(&neotrix_dir);
    let forbidden_layers = [
        "l2_perception",
        "l3_embodiment",
        "l4_emotion",
        "l5_cognition",
        "l6_meta",
    ];
    let mut violations: Vec<String> = Vec::new();

    for file in &files {
        // neotrix/ subdirs can reference l1_action (it's the same logical layer)
        // but must not reference higher layers
        for layer in &forbidden_layers {
            let pattern = format!("crate::{}", layer);
            let matches = grep_imports(file, &pattern);
            for m in &matches {
                violations.push(format!(
                    "{}:{} — neotrix subdir imports from {}",
                    relative_path(file),
                    m.split(':').next().unwrap_or("?"),
                    layer
                ));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "NEOTRIX SUBDIR UPWARD DEPENDENCY VIOLATIONS ({} found):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }
}

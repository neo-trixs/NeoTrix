// nt-lang: NeoTrix意识体进化语言
// Phase 2b — Library API for programmatic compilation
#![forbid(unsafe_code)]

pub mod codegen;
pub mod ir;
pub mod lower;
pub mod parser;
pub mod registry;
pub mod sutra_ir;
pub mod tensor_graph;
pub mod test_parser;

use std::path::Path;

/// Compile a .nt file → Rust source code (test binary, Phase 1 style).
pub fn compile_file(path: &Path) -> Result<String, Vec<lower::Diagnostic>> {
    let suite = test_parser::parse_file(path).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: e,
            location: path.display().to_string(),
        }]
    })?;
    Ok(codegen::rust::generate(&suite))
}

/// Compile a .nt file → Rust module (Phase 2a style), with validation.
pub fn compile_module_file(
    path: &Path,
) -> Result<(String, lower::LoweredModule), Vec<lower::Diagnostic>> {
    let m = test_parser::parse_module(path).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: e,
            location: path.display().to_string(),
        }]
    })?;

    let lm = lower::lower(m);
    let errors: Vec<_> = lm
        .diagnostics
        .iter()
        .filter(|d| d.severity == lower::LowerSeverity::Error)
        .collect();

    if !errors.is_empty() {
        return Err(lm.diagnostics);
    }

    let code = codegen::rust::generate_module(&lm.module);
    Ok((code, lm))
}

/// Compile a raw .nt source string → Rust source code (test binary).
pub fn compile_string(name: &str, source: &str) -> Result<String, Vec<lower::Diagnostic>> {
    let path = std::env::temp_dir().join(format!("{}.nt", name));
    std::fs::write(&path, source).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: format!("Cannot write temp file: {}", e),
            location: path.display().to_string(),
        }]
    })?;
    compile_file(&path)
}

/// Validate a .nt file without generating code.
pub fn check_file(path: &Path) -> Result<lower::LoweredModule, Vec<lower::Diagnostic>> {
    let m = test_parser::parse_module(path).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: e,
            location: path.display().to_string(),
        }]
    })?;
    let lm = lower::lower(m);
    let has_errors = lm
        .diagnostics
        .iter()
        .any(|d| d.severity == lower::LowerSeverity::Error);
    if has_errors {
        return Err(lm.diagnostics.clone());
    }
    Ok(lm)
}

// ---- .ne source compilation (new parser pipeline) ----

/// Compile a .ne file → Rust test source code using the VSA-aware Ne parser.
pub fn compile_ne_file(path: &Path) -> Result<String, Vec<lower::Diagnostic>> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: format!("Cannot read {}: {}", path.display(), e),
            location: path.display().to_string(),
        }]
    })?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    compile_ne_string(name, &source)
}

/// Compile a .ne source string → Rust test source code (wrapped in #[cfg(test)]).
pub fn compile_ne_string(name: &str, source: &str) -> Result<String, Vec<lower::Diagnostic>> {
    let mut compiler = sutra_ir::SutraCompiler::new(sutra_ir::SutraLanguageSpec::default());
    compiler.compile(source, name).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: e,
            location: format!("<{}>", name),
        }]
    })
}

/// Compile a .ne file → Rust module source code (usable as a real module).
pub fn compile_ne_module_file(path: &Path) -> Result<String, Vec<lower::Diagnostic>> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: format!("Cannot read {}: {}", path.display(), e),
            location: path.display().to_string(),
        }]
    })?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    compile_ne_module_string(name, &source)
}

/// Compile a .ne source string → Rust module source code (no test wrapper).
pub fn compile_ne_module_string(
    name: &str,
    source: &str,
) -> Result<String, Vec<lower::Diagnostic>> {
    let mut compiler = sutra_ir::SutraCompiler::new(sutra_ir::SutraLanguageSpec::default());
    compiler.compile_as_module(source, name).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: e,
            location: format!("<{}>", name),
        }]
    })
}

/// Compile a .ne file → PCC-annotated Rust module with safety contracts.
pub fn compile_ne_pcc_file(path: &Path) -> Result<String, Vec<lower::Diagnostic>> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: format!("Cannot read {}: {}", path.display(), e),
            location: path.display().to_string(),
        }]
    })?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    compile_ne_pcc_string(name, &source)
}

/// Compile a .ne source string → PCC-annotated Rust module with safety contracts.
pub fn compile_ne_pcc_string(name: &str, source: &str) -> Result<String, Vec<lower::Diagnostic>> {
    use crate::ir::{Function, Import, Module, Type};
    use crate::parser::parse::parse_stmts;

    let stmts = parse_stmts(source).map_err(|e| {
        vec![lower::Diagnostic {
            severity: lower::LowerSeverity::Error,
            message: e.message,
            location: format!("<{}>", name),
        }]
    })?;

    let module = Module {
        name: name.to_string(),
        description: String::new(),
        source_file: std::path::PathBuf::from(format!("{}.ne", name)),
        vsa_dim: Some(4096),
        imports: vec![Import {
            path: "neotrix::core::nt_core_hcube::quantized_vsa::QuantizedVSA".into(),
            alias: None,
        }],
        functions: vec![Function {
            name: "main".into(),
            params: vec![],
            return_type: Type::Vsa(crate::ir::VsaDim::Dim(4096), None),
            body: stmts,
            description: None,
        }],
        pipeline: None,
        tests: vec![],
    };

    let lm = lower::lower(module);
    let errors: Vec<_> = lm
        .diagnostics
        .iter()
        .filter(|d| d.severity == lower::LowerSeverity::Error)
        .collect();

    if !errors.is_empty() {
        return Err(lm.diagnostics);
    }

    Ok(codegen::pcc::generate_pcc_module(&lm.module))
}

/// Validate a .ne file without generating code.
pub fn check_ne_file(path: &Path) -> Result<(), Vec<lower::Diagnostic>> {
    compile_ne_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_ne_string_produces_test_code() {
        let source = "let a = bundle(random_vector(), random_vector())\na";
        let result = compile_ne_string("test_ne", source);
        assert!(
            result.is_ok(),
            "compile_ne_string failed: {:?}",
            result.err()
        );
        let code = result.unwrap();
        assert!(
            code.contains("sutra_generated"),
            "should contain test module"
        );
        assert!(code.contains("#[cfg(test)]"), "should be test-wrapped");
        assert!(code.contains("test_ne_eval"), "should contain eval test");
        assert!(
            code.contains("QuantizedVSA"),
            "should reference QuantizedVSA"
        );
    }

    #[test]
    fn test_compile_ne_module_string_produces_module_code() {
        let source = "random_vector()";
        let result = compile_ne_module_string("mod_vsa", source);
        assert!(
            result.is_ok(),
            "compile_ne_module_string failed: {:?}",
            result.err()
        );
        let code = result.unwrap();
        assert!(code.contains("pub fn main"), "should contain main function");
        assert!(
            code.contains("QuantizedVSA"),
            "should reference QuantizedVSA"
        );
        assert!(
            !code.contains("#[cfg(test)]"),
            "module should not be test-wrapped"
        );
        assert!(
            code.contains("// Module: mod_vsa"),
            "should contain module name"
        );
    }

    #[test]
    fn test_compile_ne_file_from_disk() {
        let path = Path::new("neotrix-core/test_suites/vsa_basics.ne");
        if !path.exists() {
            // Try alternate path
            let alt = Path::new("../neotrix-core/test_suites/vsa_basics.ne");
            if !alt.exists() {
                // Create a temp test file
                let temp = std::env::temp_dir().join("vsa_basics.ne");
                std::fs::write(&temp, "random_vector()").unwrap();
                let result = compile_ne_file(&temp);
                assert!(result.is_ok(), "compile_ne_file temp failed");
                return;
            }
            let result = compile_ne_file(alt);
            assert!(result.is_ok(), "compile_ne_file failed: {:?}", result.err());
            return;
        }
        let result = compile_ne_file(path);
        assert!(result.is_ok(), "compile_ne_file failed: {:?}", result.err());
    }

    #[test]
    fn test_check_ne_file_valid() {
        let source = "let x = 42\nx";
        let temp = std::env::temp_dir().join("check_test.ne");
        std::fs::write(&temp, source).unwrap();
        let result = check_ne_file(&temp);
        assert!(
            result.is_ok(),
            "check_ne_file should succeed for valid source"
        );
    }

    #[test]
    fn test_compile_ne_string_invalid_syntax() {
        let source = "let = 42"; // invalid let without ident
        let result = compile_ne_string("bad", source);
        assert!(result.is_err(), "should fail on invalid syntax");
    }

    #[test]
    fn test_ne_pipeline_module_roundtrip() {
        let source = "bind(random_vector(), random_vector())";
        let result = compile_ne_module_string("roundtrip_mod", source);
        assert!(result.is_ok());
        let code = result.unwrap();
        // Module output should have a main function with bind operation
        assert!(code.contains("pub fn main"));
        assert!(code.contains("QuantizedVSA"));
        assert!(code.contains(".bind("));
    }
}

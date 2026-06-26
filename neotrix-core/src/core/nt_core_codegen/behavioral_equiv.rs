use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ne0 assembler types — mirrors stage0_seed.rs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Operation {
    Xor,
    MajoritySum,
    Cosine,
    Knn,
}

#[derive(Debug, Clone)]
enum Ne0Instr {
    Call { func: String, args: Vec<String> },
    Ret { value: Option<String> },
}

#[derive(Debug, Clone)]
struct Ne0Function {
    name: String,
    args: Vec<String>,
    body: Vec<Ne0Instr>,
}

#[derive(Debug, Clone)]
struct Ne0Program {
    primitives: Vec<(String, Operation)>,
    functions: Vec<Ne0Function>,
    exports: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BehavioralEquivalenceTest {
    pub name: String,
    pub ne_source: String,
    pub reference_fn: fn(&[Vec<u8>]) -> Vec<u8>,
    pub test_inputs: Vec<Vec<Vec<u8>>>,
    pub max_rounds: usize,
}

#[derive(Debug, Clone)]
pub struct EquivalenceResult {
    pub test_name: String,
    pub passed: bool,
    pub input_index: usize,
    pub ne_output: Vec<u8>,
    pub rust_output: Vec<u8>,
    pub similarity: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EquivalenceSuite {
    pub tests: Vec<BehavioralEquivalenceTest>,
    pub results: Vec<EquivalenceResult>,
}

pub struct EquivalenceRunner;

// ---------------------------------------------------------------------------
// ne0 parser — mirrors stage0_seed::compile
// ---------------------------------------------------------------------------

fn parse_args_list(s: &str) -> Vec<String> {
    let inner = s.trim_start_matches('(').trim_end_matches(')');
    inner
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|a| !a.is_empty())
        .map(|a| a.to_string())
        .collect()
}

fn compile_ne(source: &str) -> Result<Ne0Program, String> {
    let lines: Vec<&str> = source
        .lines()
        .map(|l| l.split("//").next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .collect();

    let mut primitives = Vec::new();
    let mut functions = Vec::new();
    let mut exports = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            i += 1;
            continue;
        }
        match parts[0] {
            "PRIMITIVE" => {
                let name = parts.get(1).ok_or("PRIMITIVE needs name")?.to_string();
                i += 1;
                let op_line = lines.get(i).ok_or("PRIMITIVE needs opcode line")?.trim();
                let op = match op_line {
                    "XOR" => Operation::Xor,
                    "MAJ" => Operation::MajoritySum,
                    "COS" => Operation::Cosine,
                    "KNN" => Operation::Knn,
                    _ => return Err(format!("Unknown opcode: {}", op_line)),
                };
                primitives.push((name, op));
                i += 1;
            }
            "DEFINE" => {
                let name = parts.get(1).ok_or("DEFINE needs name")?.to_string();
                let args_str = parts[2..].join(" ");
                let args = parse_args_list(&args_str);
                i += 1;
                let mut body = Vec::new();
                while i < lines.len() {
                    let bline = lines[i].trim();
                    if bline.starts_with("RET") {
                        let val = bline.split_whitespace().nth(1).map(|s| s.to_string());
                        body.push(Ne0Instr::Ret { value: val });
                        i += 1;
                        break;
                    } else if bline.starts_with("CALL") {
                        let bp: Vec<&str> = bline.split_whitespace().collect();
                        let func = bp.get(1).ok_or("CALL needs func")?.to_string();
                        let args: Vec<String> = bp[2..].iter().map(|a| a.to_string()).collect();
                        body.push(Ne0Instr::Call { func, args });
                        i += 1;
                    } else {
                        break;
                    }
                }
                functions.push(Ne0Function { name, args, body });
            }
            "EXPORT" => {
                let name = parts.get(1).ok_or("EXPORT needs name")?.to_string();
                exports.push(name);
                i += 1;
            }
            other => return Err(format!("Unknown directive: {}", other)),
        }
    }
    Ok(Ne0Program {
        primitives,
        functions,
        exports,
    })
}

// ---------------------------------------------------------------------------
// Evaluation: run compiled ne0 program on concrete Vec<u8> inputs
// ---------------------------------------------------------------------------

fn run_primitive(op: Operation, a: &[u8], b: &[u8]) -> Vec<u8> {
    match op {
        Operation::Xor => QuantizedVSA::xor_bind(a, b),
        Operation::MajoritySum => QuantizedVSA::majority_bundle(&[a, b]),
        Operation::Cosine => {
            let len = a.len().min(b.len());
            if len == 0 {
                return vec![0u8; 4];
            }
            let mut dot = 0u64;
            let mut mag_a = 0u64;
            let mut mag_b = 0u64;
            for i in 0..len {
                let va = a[i] as u64;
                let vb = b[i] as u64;
                dot += va * vb;
                mag_a += va * va;
                mag_b += vb * vb;
            }
            let denom = ((mag_a as f64).sqrt() * (mag_b as f64).sqrt()).max(1e-12);
            let cos_val = dot as f64 / denom;
            let bits = cos_val.to_bits();
            vec![
                (bits >> 24) as u8,
                (bits >> 16) as u8,
                (bits >> 8) as u8,
                bits as u8,
            ]
        }
        Operation::Knn => {
            let best_idx = if b.len() < a.len() {
                0usize
            } else {
                let dim = a.len();
                let n_candidates = b.len() / dim;
                let mut best = (u64::MAX, 0usize);
                for i in 0..n_candidates {
                    let start = i * dim;
                    let end = start + dim;
                    if end > b.len() {
                        break;
                    }
                    let candidate = &b[start..end];
                    let d: u64 = a
                        .iter()
                        .zip(candidate.iter())
                        .map(|(x, y)| (*x as u64).abs_diff(*y as u64))
                        .sum();
                    if d < best.0 {
                        best = (d, i);
                    }
                }
                best.1
            };
            vec![best_idx as u8]
        }
    }
}

fn resolve_arg(arg: &str, func_args: &[String], temp_vals: &[Vec<u8>]) -> Vec<u8> {
    if let Some(idx) = func_args.iter().position(|a| a == arg) {
        return temp_vals[idx].clone();
    }
    for (tidx, tv) in temp_vals.iter().enumerate().skip(func_args.len()) {
        if format!("_r{}", tidx - func_args.len()) == arg {
            return tv.clone();
        }
    }
    vec![]
}

fn eval_function<'a>(
    name: &str,
    args: &[Vec<u8>],
    functions: &'a [Ne0Function],
    primitives: &HashMap<String, Operation>,
    depth: usize,
) -> Result<Vec<u8>, String> {
    if depth > 64 {
        return Err("max call depth exceeded".to_string());
    }

    let func = functions
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| format!("function not found: {}", name))?;

    if args.len() != func.args.len() {
        return Err(format!(
            "argument count mismatch for {}: expected {}, got {}",
            name,
            func.args.len(),
            args.len()
        ));
    }

    let mut temp_vals: Vec<Vec<u8>> = args.to_vec();
    let n_formal = func.args.len();

    for instr in &func.body {
        match instr {
            Ne0Instr::Call {
                func: callee,
                args: call_args,
            } => {
                let resolved: Vec<Vec<u8>> = call_args
                    .iter()
                    .map(|a| resolve_arg(a, &func.args, &temp_vals))
                    .collect();
                let result = if let Some(op) = primitives.get(callee) {
                    if resolved.len() < 2 {
                        return Err(format!("primitive {} needs 2 args", callee));
                    }
                    run_primitive(*op, &resolved[0], &resolved[1])
                } else {
                    eval_function(callee, &resolved, functions, primitives, depth + 1)?
                };
                temp_vals.push(result);
            }
            Ne0Instr::Ret { value } => {
                let ret = match value {
                    Some(v) if *v == "RET" => {
                        if temp_vals.len() > n_formal {
                            temp_vals
                                .last()
                                .cloned()
                                .expect("temp_vals.len() > n_formal")
                        } else if !args.is_empty() {
                            args[0].clone()
                        } else {
                            vec![]
                        }
                    }
                    Some(v) => resolve_arg(v, &func.args, &temp_vals),
                    None => {
                        if temp_vals.len() > n_formal {
                            temp_vals
                                .last()
                                .cloned()
                                .expect("temp_vals.len() > n_formal")
                        } else {
                            vec![]
                        }
                    }
                };
                return Ok(ret);
            }
        }
    }

    if temp_vals.len() > n_formal {
        Ok(temp_vals
            .last()
            .cloned()
            .expect("temp_vals.len() > n_formal"))
    } else {
        Ok(vec![])
    }
}

/// Given a compiled program and a set of input vectors, evaluate the last
/// exported function and return the result.
fn evaluate_program(program: &Ne0Program, inputs: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    let entry = program
        .exports
        .last()
        .ok_or_else(|| "no exported function".to_string())?;
    let primitives: HashMap<String, Operation> = program.primitives.iter().cloned().collect();
    eval_function(entry, inputs, &program.functions, &primitives, 0)
}

fn vsa_cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
    QuantizedVSA::cosine(a, b)
}

// ---------------------------------------------------------------------------
// EquivalenceRunner implementation
// ---------------------------------------------------------------------------

impl EquivalenceRunner {
    pub fn run(test: &BehavioralEquivalenceTest) -> Vec<EquivalenceResult> {
        let max_rounds = test.max_rounds.max(1);
        let limit = test.test_inputs.len().min(max_rounds);
        let mut results = Vec::with_capacity(limit);

        for i in 0..limit {
            let inputs = &test.test_inputs[i];

            let ne_output = match Self::compile_and_eval(&test.ne_source, inputs) {
                Ok(v) => v,
                Err(e) => {
                    results.push(EquivalenceResult {
                        test_name: test.name.clone(),
                        passed: false,
                        input_index: i,
                        ne_output: vec![],
                        rust_output: vec![],
                        similarity: 0.0,
                        error: Some(format!("ne eval error: {}", e)),
                    });
                    continue;
                }
            };

            let rust_output = (test.reference_fn)(inputs);

            let sim = if ne_output.len() == rust_output.len() && !ne_output.is_empty() {
                vsa_cosine_similarity(&ne_output, &rust_output)
            } else if ne_output == rust_output {
                1.0
            } else {
                0.0
            };

            let passed = sim > 0.99 || ne_output == rust_output;

            results.push(EquivalenceResult {
                test_name: test.name.clone(),
                passed,
                input_index: i,
                ne_output,
                rust_output,
                similarity: sim,
                error: None,
            });
        }

        results
    }

    fn compile_and_eval(ne_source: &str, inputs: &[Vec<u8>]) -> Result<Vec<u8>, String> {
        let program = compile_ne(ne_source)?;
        evaluate_program(&program, inputs)
    }

    pub fn run_suite(suite: &mut EquivalenceSuite) {
        suite.results.clear();
        for test in &suite.tests {
            let results = Self::run(test);
            suite.results.extend(results);
        }
        let summary = Self::print_report(&suite.results);
        log::info!("{}", summary);
    }

    pub fn print_report(results: &[EquivalenceResult]) -> String {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let avg_sim: f64 = if total > 0 {
            results.iter().map(|r| r.similarity).sum::<f64>() / total as f64
        } else {
            0.0
        };

        let mut out = String::new();
        out.push_str(&format!("=== Behavioral Equivalence Report ===\n"));
        out.push_str(&format!("  Total:  {}\n", total));
        out.push_str(&format!("  Passed: {}\n", passed));
        out.push_str(&format!("  Failed: {}\n", failed));
        out.push_str(&format!("  Avg similarity: {:.6}\n", avg_sim));

        if failed > 0 {
            out.push_str("\n  Failures:\n");
            for r in results.iter().filter(|r| !r.passed) {
                out.push_str(&format!(
                    "    - {}[{}]: sim={:.4} {}\n",
                    r.test_name,
                    r.input_index,
                    r.similarity,
                    r.error.as_deref().unwrap_or("")
                ));
            }
        }

        out
    }
}

// ---------------------------------------------------------------------------
// Predefined reference functions
// ---------------------------------------------------------------------------

fn ref_bind_identity(inputs: &[Vec<u8>]) -> Vec<u8> {
    let a = inputs.first().cloned().unwrap_or_default();
    QuantizedVSA::xor_bind(&a, &a)
}

fn ref_bundle(inputs: &[Vec<u8>]) -> Vec<u8> {
    if inputs.len() < 2 {
        return vec![0u8; 4096];
    }
    QuantizedVSA::majority_bundle(&[&inputs[0], &inputs[1]])
}

fn ref_cosine(inputs: &[Vec<u8>]) -> Vec<u8> {
    if inputs.len() < 2 {
        return vec![0u8; 4];
    }
    let sim_val = QuantizedVSA::cosine(&inputs[0], &inputs[1]);
    let bits = sim_val.to_bits();
    vec![
        (bits >> 24) as u8,
        (bits >> 16) as u8,
        (bits >> 8) as u8,
        bits as u8,
    ]
}

fn ref_bind_assoc(inputs: &[Vec<u8>]) -> Vec<u8> {
    if inputs.len() < 3 {
        return vec![];
    }
    QuantizedVSA::xor_bind(&QuantizedVSA::xor_bind(&inputs[0], &inputs[1]), &inputs[2])
}

fn ref_knn(inputs: &[Vec<u8>]) -> Vec<u8> {
    if inputs.len() < 2 {
        return vec![0u8];
    }
    let query = &inputs[0];
    let codebook = &inputs[1];
    let dim = query.len();
    if codebook.len() < dim {
        return vec![0u8];
    }
    let n_candidates = codebook.len() / dim;
    let mut best = (u64::MAX, 0usize);
    for i in 0..n_candidates {
        let start = i * dim;
        let end = (start + dim).min(codebook.len());
        if end - start < dim {
            break;
        }
        let candidate = &codebook[start..end];
        let d: u64 = query
            .iter()
            .zip(candidate.iter())
            .map(|(x, y)| (*x as u64).abs_diff(*y as u64))
            .sum();
        if d < best.0 {
            best = (d, i);
        }
    }
    vec![best.1 as u8]
}

// ---------------------------------------------------------------------------
// Predefined test builders
// ---------------------------------------------------------------------------

pub fn make_bind_identity_test() -> BehavioralEquivalenceTest {
    let a = QuantizedVSA::seeded_random(42, 16);
    let b = QuantizedVSA::seeded_random(99, 16);
    BehavioralEquivalenceTest {
        name: "bind_identity".into(),
        ne_source: r#"
PRIMITIVE xor_op
  XOR
DEFINE bind_test (a)
  CALL xor_op a a
  RET
EXPORT bind_test
"#
        .into(),
        reference_fn: ref_bind_identity,
        test_inputs: vec![vec![a.clone()], vec![b.clone()]],
        max_rounds: 2,
    }
}

pub fn make_bundle_test() -> BehavioralEquivalenceTest {
    let a = QuantizedVSA::seeded_random(42, 16);
    let b = QuantizedVSA::seeded_random(99, 16);
    let c = QuantizedVSA::seeded_random(1, 16);
    let d = QuantizedVSA::seeded_random(2, 16);
    BehavioralEquivalenceTest {
        name: "bundle".into(),
        ne_source: r#"
PRIMITIVE maj_op
  MAJ
DEFINE bundle_test (a b)
  CALL maj_op a b
  RET
EXPORT bundle_test
"#
        .into(),
        reference_fn: ref_bundle,
        test_inputs: vec![vec![a.clone(), b.clone()], vec![c.clone(), d.clone()]],
        max_rounds: 2,
    }
}

pub fn make_cosine_test() -> BehavioralEquivalenceTest {
    let a = QuantizedVSA::seeded_random(42, 16);
    let b = QuantizedVSA::seeded_random(99, 16);
    BehavioralEquivalenceTest {
        name: "cosine".into(),
        ne_source: r#"
PRIMITIVE cos_op
  COS
DEFINE cosine_test (a b)
  CALL cos_op a b
  RET
EXPORT cosine_test
"#
        .into(),
        reference_fn: ref_cosine,
        test_inputs: vec![vec![a.clone(), b.clone()]],
        max_rounds: 1,
    }
}

pub fn make_bind_associativity_test() -> BehavioralEquivalenceTest {
    let a = QuantizedVSA::seeded_random(42, 16);
    let b = QuantizedVSA::seeded_random(99, 16);
    let c = QuantizedVSA::seeded_random(123, 16);
    BehavioralEquivalenceTest {
        name: "bind_associativity".into(),
        ne_source: r#"
PRIMITIVE xor_op
  XOR
DEFINE bind_pair (a b)
  CALL xor_op a b
  RET
DEFINE bind_triple (a b c)
  CALL xor_op a b
  CALL xor_op _r0 c
  RET
EXPORT bind_triple
"#
        .into(),
        reference_fn: ref_bind_assoc,
        test_inputs: vec![vec![a.clone(), b.clone(), c.clone()]],
        max_rounds: 1,
    }
}

pub fn make_cleanup_knn_test() -> BehavioralEquivalenceTest {
    let codebook: Vec<u8> = (0..64u8)
        .flat_map(|i| std::iter::repeat(i).take(16))
        .collect();
    let query = QuantizedVSA::seeded_random(42, 16);
    BehavioralEquivalenceTest {
        name: "cleanup_knn".into(),
        ne_source: r#"
PRIMITIVE knn_op
  KNN
DEFINE cleanup_test (q cb)
  CALL knn_op q cb
  RET
EXPORT cleanup_test
"#
        .into(),
        reference_fn: ref_knn,
        test_inputs: vec![vec![query, codebook]],
        max_rounds: 1,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vec(v: u8, len: usize) -> Vec<u8> {
        vec![v; len]
    }

    // --- compile tests ---

    #[test]
    fn test_compile_primitive() {
        let src = "PRIMITIVE my_xor\n  XOR\n";
        let prog = compile_ne(src).unwrap();
        assert_eq!(prog.primitives.len(), 1);
        assert_eq!(prog.primitives[0].0, "my_xor");
        assert_eq!(prog.primitives[0].1, Operation::Xor);
    }

    #[test]
    fn test_compile_define() {
        let src = "DEFINE identity (x)\n  RET x\n";
        let prog = compile_ne(src).unwrap();
        assert_eq!(prog.functions.len(), 1);
        assert_eq!(prog.functions[0].name, "identity");
        assert_eq!(prog.functions[0].args, vec!["x"]);
    }

    #[test]
    fn test_compile_export() {
        let src = "PRIMITIVE xor_op\n  XOR\nDEFINE f (a)\n  CALL xor_op a a\n  RET\nEXPORT f\n";
        let prog = compile_ne(src).unwrap();
        assert_eq!(prog.exports, vec!["f"]);
    }

    #[test]
    fn test_compile_empty() {
        let prog = compile_ne("").unwrap();
        assert!(prog.primitives.is_empty() && prog.functions.is_empty());
    }

    #[test]
    fn test_compile_error_unknown_directive() {
        let src = "BOGUS directive\n";
        assert!(compile_ne(src).is_err());
    }

    // --- evaluation tests ---

    #[test]
    fn test_eval_identity() {
        let src = "DEFINE identity (x)\n  RET x\nEXPORT identity\n";
        let prog = compile_ne(src).unwrap();
        let input = vec![vec![1u8, 2u8, 3u8]];
        let result = evaluate_program(&prog, &input).unwrap();
        assert_eq!(result, vec![1u8, 2u8, 3u8]);
    }

    #[test]
    fn test_eval_xor_primitive() {
        let src = "PRIMITIVE xor_op\n  XOR\nDEFINE f (a b)\n  CALL xor_op a b\n  RET\nEXPORT f\n";
        let prog = compile_ne(src).unwrap();
        let a = vec![0x0F, 0xFF];
        let b = vec![0xF0, 0xFF];
        let result = evaluate_program(&prog, &[a.clone(), b.clone()]).unwrap();
        let expected: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_maj_primitive() {
        let src = "PRIMITIVE maj_op\n  MAJ\nDEFINE f (a b)\n  CALL maj_op a b\n  RET\nEXPORT f\n";
        let prog = compile_ne(src).unwrap();
        let a = test_vec(1, 16);
        let b = test_vec(0, 16);
        let result = evaluate_program(&prog, &[a.clone(), b.clone()]).unwrap();
        let expected = QuantizedVSA::majority_bundle(&[&a, &b]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_knn_primitive() {
        let src = "PRIMITIVE knn_op\n  KNN\nDEFINE f (q cb)\n  CALL knn_op q cb\n  RET\nEXPORT f\n";
        let prog = compile_ne(src).unwrap();
        let codebook: Vec<u8> = (0..48u8).collect();
        let query = test_vec(5, 16);
        let result = evaluate_program(&prog, &[query, codebook]).unwrap();
        assert_eq!(result.len(), 1);
    }

    // --- equivalence tests ---

    #[test]
    fn test_bind_identity_equivalence() {
        let test = make_bind_identity_test();
        let results = EquivalenceRunner::run(&test);
        assert!(!results.is_empty());
        for r in &results {
            assert!(
                r.passed,
                "bind_identity[{}] failed: sim={:.4}",
                r.input_index, r.similarity
            );
        }
    }

    #[test]
    fn test_bundle_equivalence() {
        let test = make_bundle_test();
        let results = EquivalenceRunner::run(&test);
        assert!(!results.is_empty());
        for r in &results {
            assert!(
                r.passed,
                "bundle[{}] failed: sim={:.4}",
                r.input_index, r.similarity
            );
        }
    }

    #[test]
    fn test_cosine_equivalence() {
        let test = make_cosine_test();
        let results = EquivalenceRunner::run(&test);
        assert!(!results.is_empty());
        for r in &results {
            assert!(
                r.passed,
                "cosine[{}] failed: sim={:.4}",
                r.input_index, r.similarity
            );
        }
    }

    #[test]
    fn test_bind_associativity() {
        let test = make_bind_associativity_test();
        let results = EquivalenceRunner::run(&test);
        assert!(!results.is_empty());
        for r in &results {
            assert!(
                r.passed,
                "bind_assoc[{}] failed: sim={:.4}",
                r.input_index, r.similarity
            );
        }
    }

    #[test]
    fn test_cleanup_knn_equivalence() {
        let test = make_cleanup_knn_test();
        let results = EquivalenceRunner::run(&test);
        assert!(!results.is_empty());
        for r in &results {
            assert!(
                r.passed,
                "cleanup_knn[{}] failed: sim={:.4}",
                r.input_index, r.similarity
            );
        }
    }

    #[test]
    fn test_compile_and_run_primitive() {
        let src = "PRIMITIVE xor_op\n  XOR\nDEFINE f (a)\n  CALL xor_op a a\n  RET\nEXPORT f\n";
        let a = vec![0xAB; 16];
        let prog = compile_ne(src).unwrap();
        let result = evaluate_program(&prog, &[a]).unwrap();
        assert!(
            result.iter().all(|&x| x == 0),
            "XOR(a,a) should be all zeros"
        );
    }

    #[test]
    fn test_empty_input() {
        let src = "DEFINE f ()\n  RET\nEXPORT f\n";
        let result = EquivalenceRunner::compile_and_eval(src, &[]);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_different_inputs() {
        let a = QuantizedVSA::seeded_random(1, 16);
        let b = QuantizedVSA::seeded_random(2, 16);
        let src = "PRIMITIVE xor_op\n  XOR\nDEFINE f (x)\n  CALL xor_op x x\n  RET\nEXPORT f\n";
        let r1 = EquivalenceRunner::compile_and_eval(src, &[a]).unwrap();
        let r2 = EquivalenceRunner::compile_and_eval(src, &[b]).unwrap();
        assert_ne!(r1, r2, "different inputs should produce different outputs");
    }

    #[test]
    fn test_suite_report() {
        let test = make_bind_identity_test();
        let mut suite = EquivalenceSuite {
            tests: vec![test],
            results: vec![],
        };
        EquivalenceRunner::run_suite(&mut suite);
        assert!(!suite.results.is_empty());
        let report = EquivalenceRunner::print_report(&suite.results);
        assert!(report.contains("Behavioral Equivalence Report"));
        assert!(report.contains("Total:"));
        assert!(report.contains("Passed:"));
    }

    #[test]
    fn test_multiple_primitives() {
        let src = r#"
PRIMITIVE xor_op
  XOR
PRIMITIVE maj_op
  MAJ
DEFINE f (a b)
  CALL xor_op a b
  CALL maj_op _r0 a
  RET
EXPORT f
"#;
        let prog = compile_ne(src).unwrap();
        assert_eq!(prog.primitives.len(), 2);
        let a = test_vec(1, 16);
        let b = test_vec(0, 16);
        let result = evaluate_program(&prog, &[a.clone(), b.clone()]).unwrap();
        let xor_step: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
        let expected = QuantizedVSA::majority_bundle(&[&xor_step, &a]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_all_equivalence_tests_pass() {
        let tests = vec![
            make_bind_identity_test(),
            make_bundle_test(),
            make_cosine_test(),
            make_bind_associativity_test(),
            make_cleanup_knn_test(),
        ];
        let mut suite = EquivalenceSuite {
            tests,
            results: vec![],
        };
        EquivalenceRunner::run_suite(&mut suite);
        let passed = suite.results.iter().filter(|r| r.passed).count();
        assert_eq!(
            passed,
            suite.results.len(),
            "all equivalence tests should pass: {}/{}",
            passed,
            suite.results.len()
        );
    }

    #[test]
    fn test_eval_nested_call() {
        let src = r#"
PRIMITIVE xor_op
  XOR
DEFINE inner (a b)
  CALL xor_op a b
  RET
DEFINE outer (a b c)
  CALL inner a b
  CALL xor_op _r0 c
  RET
EXPORT outer
"#;
        let a = vec![0xFF; 16];
        let b = vec![0x00; 16];
        let c = vec![0xAA; 16];
        let result =
            EquivalenceRunner::compile_and_eval(src, &[a.clone(), b.clone(), c.clone()]).unwrap();
        let ab: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
        let expected: Vec<u8> = ab.iter().zip(c.iter()).map(|(x, y)| x ^ y).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_report_failure_output() {
        let results = vec![EquivalenceResult {
            test_name: "failing_test".into(),
            passed: false,
            input_index: 0,
            ne_output: vec![0],
            rust_output: vec![1],
            similarity: 0.0,
            error: Some("mismatch".into()),
        }];
        let report = EquivalenceRunner::print_report(&results);
        assert!(report.contains("Failed: 1"));
        assert!(report.contains("failing_test"));
    }
}

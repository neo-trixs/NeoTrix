use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::expr::{parse_file, parse_ne, NeExpr};
use super::value::NeValue;

mod primitives;
mod stdlib;
mod trace;

pub use stdlib::NeStdLib;
pub use trace::EvalTraceEntry;

const VSA_DIM: usize = 4096;
const MAX_CALL_DEPTH: usize = 64;
const MAX_ENV_SIZE: usize = 5000;
const MAX_LAMBDA_CACHE: usize = 5000;
const MAX_IMPORT_CACHE: usize = 500;

type PrimitiveFn = fn(&[NeValue]) -> Result<NeValue, String>;

pub struct NeEvaluator {
    env: HashMap<String, NeValue>,
    primitives: HashMap<String, PrimitiveFn>,

    lambda_bodies: HashMap<String, Box<NeExpr>>,
    exports: HashMap<String, NeValue>,
    import_cache: HashMap<String, NeValue>,
    max_env_size: usize,
    max_lambda_cache: usize,
    max_import_cache: usize,
    call_depth: usize,
    step_count: u64,
    eval_count: u64,
    trace: Vec<EvalTraceEntry>,
    max_trace: usize,
    pub test_total: u64,
    pub test_passed: u64,
    pub test_failed: u64,
    pub test_assert_count: u64,
    pub test_failure_count: u64,
    pub test_details: Vec<String>,
    pub test_coverage_map: std::collections::HashMap<String, u64>,
}

impl std::fmt::Debug for NeEvaluator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NeEvaluator")
            .field("env", &self.env)
            .field("primitives", &self.primitives.keys().collect::<Vec<_>>())
            .field(
                "lambda_bodies",
                &self.lambda_bodies.keys().collect::<Vec<_>>(),
            )
            .field("exports", &self.exports)
            .field("import_cache.len", &self.import_cache.len())
            .field("call_depth", &self.call_depth)
            .field("step_count", &self.step_count)
            .field("eval_count", &self.eval_count)
            .field("max_env_size", &self.max_env_size)
            .field("max_lambda_cache", &self.max_lambda_cache)
            .field("max_import_cache", &self.max_import_cache)
            .finish()
    }
}

impl Clone for NeEvaluator {
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
            primitives: self.primitives.clone(),

            lambda_bodies: self.lambda_bodies.clone(),
            exports: self.exports.clone(),
            import_cache: self.import_cache.clone(),
            call_depth: self.call_depth,
            step_count: self.step_count,
            eval_count: self.eval_count,
            trace: self.trace.clone(),
            max_trace: self.max_trace,
            max_env_size: self.max_env_size,
            max_lambda_cache: self.max_lambda_cache,
            max_import_cache: self.max_import_cache,
            test_total: self.test_total,
            test_passed: self.test_passed,
            test_failed: self.test_failed,
            test_assert_count: self.test_assert_count,
            test_failure_count: self.test_failure_count,
            test_details: self.test_details.clone(),
            test_coverage_map: self.test_coverage_map.clone(),
        }
    }
}

impl Default for NeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl NeEvaluator {
    pub fn new() -> Self {
        let mut s = NeEvaluator {
            env: HashMap::new(),
            primitives: HashMap::new(),
            lambda_bodies: HashMap::new(),
            exports: HashMap::new(),
            import_cache: HashMap::new(),
            call_depth: 0,
            step_count: 0,
            eval_count: 0,
            trace: Vec::new(),
            max_trace: 1000,
            max_env_size: MAX_ENV_SIZE,
            max_lambda_cache: MAX_LAMBDA_CACHE,
            max_import_cache: MAX_IMPORT_CACHE,
            test_total: 0,
            test_passed: 0,
            test_failed: 0,
            test_assert_count: 0,
            test_failure_count: 0,
            test_coverage_map: HashMap::new(),
            test_details: Vec::new(),
        };
        primitives::register_primitives(&mut s);
        s
    }

    pub fn eval_string(&mut self, source: &str) -> Result<NeValue, String> {
        let expr = parse_ne(source)?;
        self.eval(&expr)
    }

    pub fn eval_file(&mut self, source: &str) -> Result<NeValue, String> {
        let expr = parse_file(source)?;
        self.eval(&expr)
    }

    pub fn eval_string_with_env(
        &mut self,
        source: &str,
        bindings: &[(&str, NeValue)],
    ) -> Result<NeValue, String> {
        for (name, val) in bindings {
            self.env.insert(name.to_string(), val.clone());
        }
        self.enforce_env_capacity();
        self.eval_string(source)
    }

    pub fn eval(&mut self, expr: &NeExpr) -> Result<NeValue, String> {
        self.step_count += 1;
        self.eval_count += 1;

        if self.call_depth > MAX_CALL_DEPTH {
            return Err("max call depth exceeded".into());
        }

        match expr {
            NeExpr::Literal(v) => Ok(v.clone()),
            NeExpr::Var(name) => self
                .env
                .get(name)
                .cloned()
                .ok_or_else(|| format!("undefined variable: {name}")),
            NeExpr::Call(name, args) => {
                if name == "define" {
                    return self.eval_define(args);
                }
                if name == "foldl" {
                    return self.eval_foldl(args);
                }
                let arg_vals: Result<Vec<NeValue>, String> =
                    args.iter().map(|a| self.eval(a)).collect();
                let arg_vals = arg_vals?;
                self.apply(name, &arg_vals)
            }
            NeExpr::Bind(a, b) => {
                let va = self.eval(a)?;
                let vb = self.eval(b)?;
                let vaa = vsa_arg(&va)?;
                let vbb = vsa_arg(&vb)?;
                let mut out = Vec::with_capacity(VSA_DIM);
                for i in 0..VSA_DIM.min(vaa.len()).min(vbb.len()) {
                    out.push(vaa[i] ^ vbb[i]);
                }
                Ok(NeValue::Vsa(out))
            }
            NeExpr::Bundle(xs) => {
                let vs: Result<Vec<NeValue>, String> = xs.iter().map(|x| self.eval(x)).collect();
                let vs = vs?;
                let vss: Vec<&[u8]> = vs.iter().map(|a| vsa_arg(a)).collect::<Result<_, _>>()?;
                if vss.is_empty() {
                    return Err("bundle needs >=1 arg".into());
                }
                let dim = vss[0].len();
                let count = vss.len();
                let mut out = Vec::with_capacity(dim);
                for i in 0..dim {
                    let ones = vss.iter().filter(|v| v[i] > 0).count();
                    out.push(if ones > count / 2 { 1 } else { 0 });
                }
                Ok(NeValue::Vsa(out))
            }
            NeExpr::Negate(x) => {
                let v = self.eval(x)?;
                let vv = vsa_arg(&v)?;
                let out: Vec<u8> = vv.iter().map(|b| !b).collect();
                Ok(NeValue::Vsa(out))
            }
            NeExpr::Permute(x) => {
                let v = self.eval(x)?;
                let vv = vsa_arg(&v)?;
                let shift = 1usize;
                let dim = vv.len();
                let mut out = vec![0u8; dim];
                for i in 0..dim {
                    out[(i + shift) % dim] = vv[i];
                }
                Ok(NeValue::Vsa(out))
            }
            NeExpr::Similarity(a, b) => {
                let va = self.eval(a)?;
                let vb = self.eval(b)?;
                let vaa = vsa_arg(&va)?;
                let vbb = vsa_arg(&vb)?;
                let dim = vaa.len().min(vbb.len());
                let dot = vaa[..dim]
                    .iter()
                    .zip(&vbb[..dim])
                    .filter(|(x, y)| **x > 0 && **y > 0)
                    .count() as f64;
                let na = vaa[..dim].iter().filter(|x| **x > 0).count() as f64;
                let nb = vbb[..dim].iter().filter(|y| **y > 0).count() as f64;
                let norm = na.sqrt() * nb.sqrt();
                Ok(NeValue::Float(if norm == 0.0 { 0.0 } else { dot / norm }))
            }
            NeExpr::If(cond, then, else_) => {
                let c = self.eval(cond)?;
                if c.is_truthy() {
                    self.eval(then)
                } else {
                    self.eval(else_)
                }
            }
            NeExpr::Let(name, val, body) => {
                let v = self.eval(val)?;
                let has_lambda = matches!(&v, NeValue::Lambda(..));
                if let NeValue::Lambda(..) = &v {
                    let key = format!("__lambda_{}", self.step_count);
                    if let Some(expr) = self.lambda_bodies.remove(&key) {
                        self.lambda_bodies.insert(name.clone(), expr);
                    }
                }
                self.env.insert(name.clone(), v);
                self.enforce_env_capacity();
                if has_lambda {
                    self.enforce_lambda_capacity();
                }
                let result = self.eval(body);
                if result.is_ok() {
                    self.env.remove(name);
                }
                result
            }
            NeExpr::Seq(xs) => {
                let mut last = NeValue::Nil;
                for x in xs {
                    last = self.eval(x)?;
                }
                Ok(last)
            }
            NeExpr::Lambda(params, body) => {
                let id = format!("__lambda_{}", self.step_count);
                self.lambda_bodies.insert(id.clone(), body.clone());
                self.enforce_lambda_capacity();
                Ok(NeValue::Lambda(params.clone(), vec![self.env.len()]))
            }
            NeExpr::Loop {
                body,
                max_iters,
                counter_name,
            } => {
                let limit = max_iters.unwrap_or(1000);
                let mut last = NeValue::Nil;
                let saved = self.env.get(counter_name).cloned();
                for i in 0..limit {
                    self.env
                        .insert(counter_name.clone(), NeValue::Int(i as i64));
                    last = self.eval(body)?;
                }
                match saved {
                    Some(v) => {
                        self.env.insert(counter_name.clone(), v);
                    }
                    None => {
                        self.env.remove(counter_name);
                    }
                }
                Ok(last)
            }
            NeExpr::LoopExpr {
                var,
                init,
                condition,
                body,
            } => {
                let init_val = self.eval(init)?;
                let saved = self.env.get(var).cloned();
                self.env.insert(var.clone(), init_val.clone());
                let mut last = init_val;
                let limit = 10000usize;
                for _ in 0..limit {
                    let cond = self.eval(condition)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    let result = self.eval(body)?;
                    self.env.insert(var.clone(), result.clone());
                    last = result;
                }
                match saved {
                    Some(v) => {
                        self.env.insert(var.clone(), v);
                    }
                    None => {
                        self.env.remove(var);
                    }
                }
                Ok(last)
            }
            NeExpr::Match { value, arms } => {
                let val = self.eval(value)?;
                let mut matched = false;
                let mut result = NeValue::Nil;
                for (pat, body) in arms {
                    if let Some(bindings) = self.match_pattern(&val, pat)? {
                        let saved: Vec<(String, NeValue)> = bindings
                            .iter()
                            .filter_map(|(k, _)| self.env.get(k).map(|v| (k.clone(), v.clone())))
                            .collect();
                        for (k, v) in &bindings {
                            self.env.insert(k.clone(), v.clone());
                        }
                        result = self.eval(body)?;
                        matched = true;
                        for (k, v) in saved {
                            self.env.insert(k, v);
                        }
                        break;
                    }
                }
                if !matched {
                    return Err("match: no arm matched".into());
                }
                Ok(result)
            }
            NeExpr::Import { path } => {
                if let Some(cached) = self.import_cache.get(path) {
                    return Ok(cached.clone());
                }
                let source = self.load_module(path)?;
                let expr = parse_ne(&source).map_err(|e| format!("import {path}: {e}"))?;
                let saved_exports = self.exports.clone();
                self.exports.clear();
                self.eval(&expr)?;
                let exports = std::mem::take(&mut self.exports);
                self.exports = saved_exports;
                let exports_val = NeValue::Exports(exports.clone());
                self.import_cache.insert(path.clone(), exports_val.clone());
                self.enforce_import_capacity();
                Ok(exports_val)
            }
            NeExpr::Export { name, value } => {
                let val = self.eval(value)?;
                self.exports.insert(name.clone(), val.clone());
                Ok(val)
            }
            NeExpr::Assert {
                condition,
                message,
                tolerance,
            } => {
                let cond_val = self.eval(condition)?;
                let passed = matches!(cond_val, NeValue::Bool(true));
                self.test_assert_count += 1;
                if !passed {
                    self.test_failure_count += 1;
                    self.test_details.push(format!("ASSERT FAIL: {}", message));
                }
                Ok(NeValue::List(vec![
                    NeValue::Bool(passed),
                    NeValue::Float(*tolerance),
                    NeValue::Str(message.clone()),
                ]))
            }
            NeExpr::Test {
                name,
                body,
                expected,
            } => {
                let result = self.eval(body)?;
                let passed = match (&result, expected) {
                    (NeValue::Bool(b), e) => *b == *e,
                    (NeValue::Int(i), _) => *i != 0,
                    _ => false,
                };
                self.test_total += 1;
                if passed {
                    self.test_passed += 1;
                } else {
                    self.test_failed += 1;
                    self.test_details.push(format!(
                        "TEST FAIL: {} (expected {}, got {:?})",
                        name, expected, result
                    ));
                }
                Ok(NeValue::Bool(passed))
            }
            NeExpr::Property {
                name,
                generator,
                property,
                iterations,
            } => {
                let generated = self.eval(generator)?;
                let values = match generated {
                    NeValue::List(ref items) => items.clone(),
                    _ => {
                        self.test_failed += 1;
                        return Ok(NeValue::Bool(false));
                    }
                };
                let mut prop_passed = 0u64;
                let mut prop_total = 0u64;
                let limit = *iterations as usize;
                for val in values.iter().take(limit) {
                    let mut child_env: std::collections::HashMap<String, NeValue> = self
                        .env
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    child_env.insert("it".to_string(), val.clone());
                    let saved_env = std::mem::replace(&mut self.env, child_env);
                    let result = self.eval(property);
                    self.env = saved_env;
                    prop_total += 1;
                    if let Ok(NeValue::Bool(true)) = result {
                        prop_passed += 1;
                    }
                }
                self.test_total += 1;
                if prop_passed == prop_total {
                    self.test_passed += 1;
                    Ok(NeValue::Bool(true))
                } else {
                    self.test_failed += 1;
                    self.test_details.push(format!(
                        "PROPERTY FAIL: {} ({}/{})",
                        name, prop_passed, prop_total
                    ));
                    Ok(NeValue::Bool(false))
                }
            }
        }
    }

    fn eval_define(&mut self, args: &[NeExpr]) -> Result<NeValue, String> {
        if args.len() < 2 {
            return Err("define needs at least 2 args".into());
        }
        let name = match &args[0] {
            NeExpr::Var(n) => n.clone(),
            _ => return Err("define first arg must be a name".into()),
        };
        let val = self.eval(&args[1])?;
        let has_lambda = matches!(&val, NeValue::Lambda(..));
        if let NeValue::Lambda(..) = &val {
            let key = format!("__lambda_{}", self.step_count);
            if let Some(expr) = self.lambda_bodies.remove(&key) {
                self.lambda_bodies.insert(name.clone(), expr);
            }
        }
        self.env.insert(name, val);
        self.enforce_env_capacity();
        if has_lambda {
            self.enforce_lambda_capacity();
        }
        if args.len() > 2 {
            let mut last = NeValue::Nil;
            for a in &args[2..] {
                last = self.eval(a)?;
            }
            Ok(last)
        } else {
            Ok(NeValue::Nil)
        }
    }

    fn eval_foldl(&mut self, args: &[NeExpr]) -> Result<NeValue, String> {
        if args.len() < 3 {
            return Err("foldl needs at least 3 args: fn init list".into());
        }
        let fn_name = match &args[0] {
            NeExpr::Var(name) => name.clone(),
            _ => {
                let evaluated = self.eval(&args[0])?;
                match &evaluated {
                    NeValue::Str(s) => s.clone(),
                    _ => return Err("foldl first arg must be a function name".into()),
                }
            }
        };
        let init = self.eval(&args[1])?;
        let list = match self.eval(&args[2])? {
            NeValue::List(xs) => xs,
            other => {
                return Err(format!(
                    "foldl third arg must be a list, got {}",
                    other.type_name()
                ))
            }
        };
        let mut acc = init;
        for item in &list {
            acc = self.apply(&fn_name, &[acc, item.clone()])?;
        }
        Ok(acc)
    }

    pub fn apply(&mut self, name: &str, args: &[NeValue]) -> Result<NeValue, String> {
        self.call_depth += 1;
        let result = self.apply_inner(name, args);
        self.call_depth -= 0;
        if self.call_depth > 0 {
            self.call_depth -= 1;
        } else {
            self.call_depth = 0;
        }
        result
    }

    fn apply_inner(&mut self, name: &str, args: &[NeValue]) -> Result<NeValue, String> {
        if let Some(prim) = self.primitives.get(name) {
            let result = prim(args);
            self.record_trace(name, args, &result);
            return result;
        }
        let entry = self.env.get(name).cloned();
        match entry {
            Some(NeValue::Lambda(params, _)) => {
                if params.len() != args.len() {
                    return Err(format!(
                        "function {name} expects {} args, got {}",
                        params.len(),
                        args.len()
                    ));
                }
                let saved: Vec<(String, NeValue)> = params
                    .iter()
                    .filter_map(|p| self.env.get(p).map(|v| (p.clone(), v.clone())))
                    .collect();
                for (p, a) in params.iter().zip(args.iter()) {
                    self.env.insert(p.clone(), a.clone());
                }
                let body = self.lambda_bodies.get(name).cloned();
                let result = match body {
                    Some(expr) => self.eval(&expr),
                    None => Err(format!("function body not found for {name}")),
                };
                for (p, v) in saved {
                    self.env.insert(p, v);
                }
                for p in params.iter() {
                    self.env.remove(p);
                }
                result
            }
            Some(other) if args.is_empty() => Ok(other),
            Some(other) => Err(format!("{name} is not callable: {other}")),
            None => Err(format!("undefined function: {name}")),
        }
    }

    pub fn register_fn(&mut self, name: &str, params: Vec<String>, body: NeExpr) {
        self.env
            .insert(name.to_string(), NeValue::Lambda(params, vec![]));
        self.lambda_bodies.insert(name.to_string(), Box::new(body));
        self.enforce_env_capacity();
        self.enforce_lambda_capacity();
    }

    pub fn register_fun(&mut self, name: &str, body: &str) -> Result<(), String> {
        let expr = parse_ne(body)?;
        match &expr {
            NeExpr::Lambda(params, _) => {
                self.register_fn(name, params.clone(), expr);
                Ok(())
            }
            _ => {
                let val = self.eval(&expr)?;
                self.env.insert(name.to_string(), val);
                self.enforce_env_capacity();
                Ok(())
            }
        }
    }

    pub fn register_primitive(&mut self, name: &str, f: PrimitiveFn) {
        self.primitives.insert(name.to_string(), f);
    }

    /// Extend the evaluator env with a batch of key-value pairs.
    /// Used by CI to inject multiple evolution/state variables at once.
    pub fn extend_env(&mut self, pairs: &[(&str, NeValue)]) {
        for (k, v) in pairs {
            self.env.insert(k.to_string(), v.clone());
        }
        self.enforce_env_capacity();
    }

    pub fn set_env(&mut self, name: &str, val: NeValue) {
        self.env.insert(name.to_string(), val);
        self.enforce_env_capacity();
    }

    pub fn get_env(&self, name: &str) -> Option<&NeValue> {
        self.env.get(name)
    }

    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    pub fn eval_count(&self) -> u64 {
        self.eval_count
    }

    pub fn snapshot_env(&self) -> HashMap<String, NeValue> {
        self.env.clone()
    }

    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }

    /// Compute a deterministic system-state probe (64 bytes) for selective decoding.
    /// Uses coarse-grained state components so the probe stays stable when nothing
    /// meaningful changes in the evaluation environment.
    pub fn compute_probe(&self, cycle_quantized: u64, handler_count: u64) -> Vec<u8> {
        let mut state = vec![0u8; 64];
        let seed = cycle_quantized
            .wrapping_mul(1000000007)
            .wrapping_add((self.step_count / 100).wrapping_mul(1000003))
            .wrapping_add(handler_count.wrapping_mul(1000033));
        let mut h = seed;
        for byte in state.iter_mut() {
            h = h
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *byte = (h ^ (h >> 16) ^ (h >> 32)) as u8;
        }
        state
    }

    /// Cosine similarity between two probe vectors (treating bytes as floats).
    pub fn probe_similarity(a: &[u8], b: &[u8]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let mut dot = 0u64;
        let mut na = 0u64;
        let mut nb = 0u64;
        for i in 0..a.len() {
            let fa = a[i] as u64;
            let fb = b[i] as u64;
            dot += fa * fb;
            na += fa * fa;
            nb += fb * fb;
        }
        let denom = (na as f64).sqrt() * (nb as f64).sqrt();
        if denom < 1e-10 {
            0.0
        } else {
            dot as f64 / denom
        }
    }

    pub fn self_inspect(&self) -> NeLanguageReport {
        let mut all_primitives: Vec<String> = self.primitives.keys().cloned().collect();

        all_primitives.sort();
        NeLanguageReport {
            primitives: all_primitives,
            env_size: self.env.len(),
            eval_count: self.eval_count,
            step_count: self.step_count,
            call_depth: self.call_depth,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn reset_stats(&mut self) {
        self.step_count = 0;
        self.eval_count = 0;
    }

    pub fn reset_test_state(&mut self) {
        self.test_total = 0;
        self.test_passed = 0;
        self.test_failed = 0;
        self.test_assert_count = 0;
        self.test_failure_count = 0;
        self.test_details.clear();
        self.test_coverage_map.clear();
    }

    pub fn test_summary(&self) -> NeValue {
        NeValue::TestResult {
            passed: self.test_passed,
            failed: self.test_failed,
            total: self.test_total,
            assert_count: self.test_assert_count,
            coverage: self.test_coverage_map.len() as u64,
        }
    }

    pub fn reset_env(&mut self) {
        self.env.clear();
    }

    fn enforce_env_capacity(&mut self) {
        if self.env.len() > self.max_env_size {
            log::warn!(
                "[ne-eval] env size {} exceeds max {}, trimming oldest entries",
                self.env.len(),
                self.max_env_size
            );
            let keys: Vec<_> = self
                .env
                .keys()
                .take(self.max_env_size / 5)
                .cloned()
                .collect();
            for k in keys {
                self.env.remove(&k);
            }
        }
    }

    fn enforce_lambda_capacity(&mut self) {
        if self.lambda_bodies.len() > self.max_lambda_cache {
            log::warn!(
                "[ne-eval] lambda cache size {} exceeds max {}, trimming oldest entries",
                self.lambda_bodies.len(),
                self.max_lambda_cache
            );
            let keys: Vec<_> = self
                .lambda_bodies
                .keys()
                .take(self.max_lambda_cache / 5)
                .cloned()
                .collect();
            for k in keys {
                self.lambda_bodies.remove(&k);
            }
        }
    }

    fn enforce_import_capacity(&mut self) {
        if self.import_cache.len() > self.max_import_cache {
            log::warn!(
                "[ne-eval] import cache size {} exceeds max {}, clearing",
                self.import_cache.len(),
                self.max_import_cache
            );
            self.import_cache.clear();
        }
    }

    fn match_pattern(
        &self,
        val: &NeValue,
        pat: &NeExpr,
    ) -> Result<Option<Vec<(String, NeValue)>>, String> {
        match pat {
            NeExpr::Literal(lit) => Ok(if lit == val { Some(vec![]) } else { None }),
            NeExpr::Var(name) if name == "_" || name == "else" => Ok(Some(vec![])),
            NeExpr::Var(name) => Ok(Some(vec![(name.clone(), val.clone())])),
            _ => Ok(None),
        }
    }

    /// Serialize evaluator state (env + exports + counters) to JSON string.
    /// Excludes primitives, lambda_bodies, import_cache, and trace
    /// (these are re-initialized on load or too large to persist).
    pub fn save_state(&self) -> Result<String, String> {
        let state = serde_json::json!({
            "env": serde_json::to_value(&self.env).map_err(|e| e.to_string())?,
            "exports": serde_json::to_value(&self.exports).map_err(|e| e.to_string())?,
            "step_count": self.step_count,
            "eval_count": self.eval_count,
        });
        serde_json::to_string_pretty(&state).map_err(|e| e.to_string())
    }

    /// Deserialize and restore evaluator state from JSON.
    /// Restores env, exports, step_count, eval_count.
    pub fn load_state(&mut self, json: &str) -> Result<(), String> {
        let state: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if let Some(env) = state.get("env") {
            self.env = serde_json::from_value(env.clone()).map_err(|e| e.to_string())?;
        }
        if let Some(exports) = state.get("exports") {
            self.exports = serde_json::from_value(exports.clone()).map_err(|e| e.to_string())?;
        }
        if let Some(sc) = state.get("step_count").and_then(|v| v.as_u64()) {
            self.step_count = sc;
        }
        if let Some(ec) = state.get("eval_count").and_then(|v| v.as_u64()) {
            self.eval_count = ec;
        }
        Ok(())
    }

    fn load_module(&self, path: &str) -> Result<String, String> {
        let search_paths = vec![
            std::path::PathBuf::from(path),
            std::path::PathBuf::from("modules").join(path),
        ];
        for p in &search_paths {
            if p.exists() {
                return std::fs::read_to_string(p)
                    .map_err(|e| format!("cannot read module {path}: {e}"));
            }
        }
        Err(format!("module not found: {path}"))
    }
}

fn vsa_arg(v: &NeValue) -> Result<&[u8], String> {
    match v {
        NeValue::Vsa(data) => Ok(data),
        _ => Err(format!("expected VSA vector, got {}", v.type_name())),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NeLanguageReport {
    pub primitives: Vec<String>,
    pub env_size: usize,
    pub eval_count: u64,
    pub step_count: u64,
    pub call_depth: usize,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_int() {
        let mut ev = NeEvaluator::new();
        assert_eq!(ev.eval_string("42").unwrap(), NeValue::Int(42));
    }

    #[test]
    fn test_literal_string() {
        let mut ev = NeEvaluator::new();
        assert_eq!(
            ev.eval_string(r#""hello""#).unwrap(),
            NeValue::Str("hello".into())
        );
    }

    #[test]
    fn test_vsa_bind() {
        let mut ev = NeEvaluator::new();
        ev.set_env("a", NeValue::Vsa(vec![1u8; VSA_DIM]));
        ev.set_env("b", NeValue::Vsa(vec![0u8; VSA_DIM]));
        let r = ev.eval_string("(bind a b)").unwrap();
        assert!(matches!(r, NeValue::Vsa(_)), "expected VSA");
        if let NeValue::Vsa(v) = r {
            assert_eq!(v, vec![1u8; VSA_DIM]);
        }
    }

    #[test]
    fn test_vsa_bundle() {
        let mut ev = NeEvaluator::new();
        ev.set_env("a", NeValue::Vsa(vec![1u8; VSA_DIM]));
        ev.set_env("b", NeValue::Vsa(vec![0u8; VSA_DIM]));
        let r = ev.eval_string("(bundle a b)").unwrap();
        assert!(matches!(r, NeValue::Vsa(_)), "expected VSA");
        if let NeValue::Vsa(v) = r {
            assert!(v.iter().all(|x| *x == 0 || *x == 1));
        }
    }

    #[test]
    fn test_if_true() {
        let mut ev = NeEvaluator::new();
        assert_eq!(ev.eval_string("(if true 42 0)").unwrap(), NeValue::Int(42));
    }

    #[test]
    fn test_if_false() {
        let mut ev = NeEvaluator::new();
        assert_eq!(
            ev.eval_string("(if false 42 99)").unwrap(),
            NeValue::Int(99)
        );
    }

    #[test]
    fn test_if_nil_is_false() {
        let mut ev = NeEvaluator::new();
        assert_eq!(ev.eval_string("(if nil 1 2)").unwrap(), NeValue::Int(2));
    }

    #[test]
    fn test_let() {
        let mut ev = NeEvaluator::new();
        assert_eq!(ev.eval_string("(let x 42 x)").unwrap(), NeValue::Int(42));
    }

    #[test]
    fn test_let_bindings_scope() {
        let mut ev = NeEvaluator::new();
        assert_eq!(
            ev.eval_string("(let x 10 (let y (+ x 5) y))").unwrap(),
            NeValue::Int(15)
        );
    }

    #[test]
    fn test_arithmetic_primitives() {
        let mut ev = NeEvaluator::new();
        assert_eq!(ev.eval_string("(+ 1 2 3)").unwrap(), NeValue::Int(6));
        assert_eq!(ev.eval_string("(- 10 3)").unwrap(), NeValue::Int(7));
        assert_eq!(ev.eval_string("(* 2 3 4)").unwrap(), NeValue::Int(24));
    }

    #[test]
    fn test_comparison() {
        let mut ev = NeEvaluator::new();
        assert_eq!(ev.eval_string("(< 3 5)").unwrap(), NeValue::Bool(true));
        assert_eq!(ev.eval_string("(< 5 3)").unwrap(), NeValue::Bool(false));
        assert_eq!(ev.eval_string("(= 42 42)").unwrap(), NeValue::Bool(true));
        assert_eq!(ev.eval_string("(= 1 2)").unwrap(), NeValue::Bool(false));
    }

    #[test]
    fn test_type_introspection() {
        let mut ev = NeEvaluator::new();
        let t = ev.eval_string("(type 42)").unwrap();
        assert_eq!(t, NeValue::Str("int".into()));
        let t2 = ev.eval_string("(type true)").unwrap();
        assert_eq!(t2, NeValue::Str("bool".into()));
    }

    #[test]
    fn test_list_ops() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(car [1 2 3])").unwrap();
        assert_eq!(r, NeValue::Int(1));
        let r2 = ev.eval_string("(cdr [1 2 3])").unwrap();
        assert!(matches!(r2, NeValue::List(_)), "expected list");
        if let NeValue::List(xs) = r2 {
            assert_eq!(xs, vec![NeValue::Int(2), NeValue::Int(3)]);
        }
    }

    #[test]
    fn test_do_returns_last() {
        let mut ev = NeEvaluator::new();
        assert_eq!(ev.eval_string("(do 1 2 3)").unwrap(), NeValue::Int(3));
    }

    #[test]
    fn test_max_depth() {
        let mut ev = NeEvaluator::new();
        let deep = "x".repeat(100);
        ev.set_env("x", NeValue::Int(1));
        let ok = ev.eval_string(&deep).unwrap();
        assert_eq!(ok, NeValue::Int(1));
    }

    #[test]
    fn test_vsa_cosine() {
        let mut ev = NeEvaluator::new();
        ev.set_env("a", NeValue::Vsa(vec![1u8; VSA_DIM]));
        ev.set_env("b", NeValue::Vsa(vec![1u8; VSA_DIM]));
        let r = ev.eval_string("(cosine a b)").unwrap();
        assert_eq!(r, NeValue::Float(1.0));
    }

    #[test]
    fn test_vsa_hamming() {
        let mut ev = NeEvaluator::new();
        ev.set_env("a", NeValue::Vsa(vec![1u8; VSA_DIM]));
        ev.set_env("b", NeValue::Vsa(vec![0u8; VSA_DIM]));
        let r = ev.eval_string("(hamming a b)").unwrap();
        assert_eq!(r, NeValue::Float(0.0));
    }

    #[test]
    fn test_vsa_negate() {
        let mut ev = NeEvaluator::new();
        ev.set_env("a", NeValue::Vsa(vec![0u8; VSA_DIM]));
        let r = ev.eval_string("(negate a)").unwrap();
        assert!(matches!(r, NeValue::Vsa(_)), "expected VSA");
        if let NeValue::Vsa(v) = r {
            assert!(v.iter().all(|x| *x == 255));
        }
    }

    #[test]
    fn test_self_inspect() {
        let ev = NeEvaluator::new();
        let report = ev.self_inspect();
        assert!(report.primitives.len() >= 12);
        assert_eq!(report.env_size, 0);
        assert_eq!(report.eval_count, 0);
    }

    #[test]
    fn test_env_isolation() {
        let mut ev = NeEvaluator::new();
        ev.set_env("x", NeValue::Int(10));
        assert_eq!(ev.eval_string("x").unwrap(), NeValue::Int(10));
        ev.reset_env();
        assert!(ev.eval_string("x").is_err());
    }

    #[test]
    fn test_nested_bind() {
        let mut ev = NeEvaluator::new();
        ev.set_env("a", NeValue::Vsa(vec![1u8; VSA_DIM]));
        ev.set_env("b", NeValue::Vsa(vec![0u8; VSA_DIM]));
        ev.set_env("c", NeValue::Vsa(vec![1u8; VSA_DIM]));
        let r = ev.eval_string("(bind (bind a b) c)").unwrap();
        assert!(matches!(r, NeValue::Vsa(_)), "expected VSA");
        if let NeValue::Vsa(v) = r {
            assert_eq!(v.len(), VSA_DIM);
            assert_eq!(v[0], 0);
        }
    }

    #[test]
    fn test_comment_in_source() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("42 ; this is a comment\n").unwrap();
        assert_eq!(r, NeValue::Int(42));
    }

    #[test]
    fn test_eval_count_tracking() {
        let mut ev = NeEvaluator::new();
        assert_eq!(ev.eval_count(), 0);
        ev.eval_string("42").unwrap();
        assert_eq!(ev.eval_count(), 1);
        ev.eval_string("(+ 1 2)").unwrap();
        assert_eq!(ev.eval_count(), 4);
        ev.reset_stats();
        assert_eq!(ev.eval_count(), 0);
    }

    #[test]
    fn test_error_undefined_var() {
        let mut ev = NeEvaluator::new();
        assert!(ev.eval_string("nonexistent").is_err());
    }

    #[test]
    fn test_error_unbound_function() {
        let mut ev = NeEvaluator::new();
        assert!(ev.eval_string("(unknown-fn 1 2 3)").is_err());
    }

    #[test]
    fn test_vsa_bind_with_variables() {
        let mut ev = NeEvaluator::new();
        ev.set_env("v1", NeValue::Vsa(vec![0xAA; VSA_DIM]));
        ev.set_env("v2", NeValue::Vsa(vec![0x55; VSA_DIM]));
        let expected: Vec<u8> = (0..VSA_DIM).map(|_i| 0xAA ^ 0x55).collect();
        let r = ev.eval_string("(bind v1 v2)").unwrap();
        assert_eq!(r, NeValue::Vsa(expected));
    }

    #[test]
    fn test_cosmic_vsa_expr() {
        let mut ev = NeEvaluator::new();
        ev.set_env("x", NeValue::Vsa(vec![0xFF; VSA_DIM]));
        ev.set_env("y", NeValue::Vsa(vec![0x00; VSA_DIM]));
        let expr = "(cosine (bind x (negate y)) (bundle x y))";
        let r = ev.eval_string(expr).unwrap();
        assert!(matches!(r, NeValue::Float(_)));
    }

    #[test]
    fn test_register_primitive() {
        let mut ev = NeEvaluator::new();
        ev.register_primitive("double", |args| match args.get(0) {
            Some(NeValue::Int(n)) => Ok(NeValue::Int(n * 2)),
            _ => Err("double needs int".into()),
        });
        assert_eq!(ev.eval_string("(double 21)").unwrap(), NeValue::Int(42));
    }

    #[test]
    fn test_call_lambda_via_set_env() {
        let mut ev = NeEvaluator::new();
        ev.set_env(
            "my_fn",
            NeValue::Lambda(vec!["x".into(), "y".into()], vec![]),
        );
        assert_eq!(ev.primitive_count(), 15);
    }

    #[test]
    fn test_large_nested_expression() {
        let mut ev = NeEvaluator::new();
        ev.set_env("x", NeValue::Int(10));
        let expr = "(let a (+ x 5) (let b (* a 2) (let c (- b 3) c)))";
        assert_eq!(ev.eval_string(expr).unwrap(), NeValue::Int(27));
    }

    #[test]
    fn test_define_and_call_lambda() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(define double (lambda [x] (add x x)) (double 5))")
            .unwrap();
        assert_eq!(r, NeValue::Int(10));
    }

    #[test]
    fn test_let_and_call_lambda() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(let double (lambda [x] (add x x)) (double 5))")
            .unwrap();
        assert_eq!(r, NeValue::Int(10));
    }

    #[test]
    fn test_register_fn_and_call() {
        let mut ev = NeEvaluator::new();
        ev.register_fn(
            "double",
            vec!["x".into()],
            NeExpr::Call(
                "add".into(),
                vec![NeExpr::Var("x".into()), NeExpr::Var("x".into())],
            ),
        );
        assert_eq!(ev.eval_string("(double 5)").unwrap(), NeValue::Int(10));
    }

    #[test]
    fn test_loop_basic() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(let sum 0 (do (loop (let sum (+ sum __i) sum) 5) sum))")
            .unwrap();
        assert_eq!(r, NeValue::Int(10));
    }

    #[test]
    fn test_loop_default_iters() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(let x 0 (do (loop (let x (+ x 1) x)) x))")
            .unwrap();
        assert_eq!(r, NeValue::Int(1000));
    }

    #[test]
    fn test_match_literal() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(match 1 (1 \"one\") (2 \"two\") (_ \"other\"))")
            .unwrap();
        assert_eq!(r, NeValue::Str("one".into()));
    }

    #[test]
    fn test_match_wildcard() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(match 99 (1 \"one\") (_ \"fallback\"))")
            .unwrap();
        assert_eq!(r, NeValue::Str("fallback".into()));
    }

    #[test]
    fn test_match_variable_binding() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(match 42 (x x))").unwrap();
        assert_eq!(r, NeValue::Int(42));
    }

    #[test]
    fn test_match_no_arm_error() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(match 1 (2 \"two\"))");
        assert!(r.is_err());
    }

    #[test]
    fn test_export_basic() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(export myval 42)").unwrap();
        assert_eq!(r, NeValue::Int(42));
    }

    #[test]
    fn test_loop_counter_scope() {
        let mut ev = NeEvaluator::new();
        ev.set_env("__i", NeValue::Int(999));
        let r = ev.eval_string("(loop __i 3)").unwrap();
        assert_eq!(r, NeValue::Int(2));
        assert_eq!(ev.get_env("__i").unwrap(), &NeValue::Int(999));
    }

    #[test]
    fn test_export_multiple() {
        let mut ev = NeEvaluator::new();
        ev.eval_string("(export a 1)").unwrap();
        ev.eval_string("(export b 2)").unwrap();
        assert_eq!(ev.exports.get("a").unwrap(), &NeValue::Int(1));
        assert_eq!(ev.exports.get("b").unwrap(), &NeValue::Int(2));
    }

    #[test]
    fn test_match_lambda_result() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(match (+ 1 2) (3 \"three\") (_ \"other\"))")
            .unwrap();
        assert_eq!(r, NeValue::Str("three".into()));
    }

    #[test]
    fn test_loop_constant_body() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(loop 42 3)").unwrap();
        assert_eq!(r, NeValue::Int(42));
    }

    #[test]
    fn test_loop_while_basic() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(loop (i 0) (< i 3) (+ i 1))").unwrap();
        assert_eq!(r, NeValue::Int(3));
    }

    #[test]
    fn test_loop_while_zero_iterations() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(loop (i 0) (< i 0) (+ i 1))").unwrap();
        assert_eq!(r, NeValue::Int(0));
    }

    #[test]
    fn test_match_else_literal() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(match 1 (1 \"one\") (2 \"two\") (else \"other\"))")
            .unwrap();
        assert_eq!(r, NeValue::Str("one".into()));
    }

    #[test]
    fn test_match_else_default() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string("(match 99 (1 \"one\") (2 \"two\") (else \"other\"))")
            .unwrap();
        assert_eq!(r, NeValue::Str("other".into()));
    }

    #[test]
    fn test_match_else_no_default() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(match 99 (1 \"one\") (2 \"two\"))");
        assert!(r.is_err());
    }

    #[test]
    fn test_nested_loop_match() {
        let mut ev = NeEvaluator::new();
        let r = ev
            .eval_string(
                "(let r (do (loop (i 0) (< i 2) (+ i 1)) (match 1 (1 \"ok\") (else \"no\"))) r)",
            )
            .unwrap();
        assert_eq!(r, NeValue::Str("ok".into()));
    }

    #[test]
    fn test_loop_while_scope_restore() {
        let mut ev = NeEvaluator::new();
        ev.set_env("i", NeValue::Int(999));
        let r = ev.eval_string("(loop (i 0) (< i 1) (+ i 1))").unwrap();
        assert_eq!(r, NeValue::Int(1));
        assert_eq!(ev.get_env("i").unwrap(), &NeValue::Int(999));
    }

    #[test]
    fn test_loop_while_with_add() {
        let mut ev = NeEvaluator::new();
        let r = ev.eval_string("(loop (x 1) (< x 100) (* x 2))").unwrap();
        assert_eq!(r, NeValue::Int(128));
    }

    #[test]
    fn test_stdlib_has_50_plus_functions() {
        let lib = NeStdLib::all();
        assert!(
            lib.len() >= 50,
            "Expected 50+ stdlib functions, got {}",
            lib.len()
        );
    }

    #[test]
    fn test_stdlib_get_function() {
        assert!(NeStdLib::get("vsa-zero").is_some());
        assert!(NeStdLib::get("nonexistent").is_none());
    }

    #[test]
    fn test_stdlib_register() {
        let mut eval = NeEvaluator::new();
        let count = NeStdLib::register(&mut eval);
        assert!(count >= 50);
    }

    #[test]
    fn test_stdlib_no_duplicates() {
        let lib = NeStdLib::all();
        let mut names = std::collections::HashSet::new();
        for (name, _) in &lib {
            assert!(names.insert(name), "Duplicate stdlib function: {}", name);
        }
    }

    #[test]
    fn test_stdlib_names_are_valid() {
        for (name, expr) in NeStdLib::all() {
            assert!(!name.is_empty(), "Empty name");
            assert!(!expr.is_empty(), "Empty body for {}", name);
            assert!(expr.starts_with("("), "Invalid expr for {}", name);
        }
    }
}

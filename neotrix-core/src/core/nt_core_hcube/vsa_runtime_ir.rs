use crate::core::nt_core_hcube::error::HyperCubeError;
use std::collections::HashMap;

const NUM_REGISTERS: usize = 16;
const VSA_DIM: usize = 4096;

#[derive(Debug, Clone, PartialEq)]
pub enum VsaOp {
    Bind {
        ra: usize,
        rb: usize,
        dest: usize,
    },
    Bundle {
        srcs: Vec<usize>,
        dest: usize,
    },
    Unbind {
        ra: usize,
        rb: usize,
        dest: usize,
    },
    Similarity {
        ra: usize,
        rb: usize,
    },
    LoadConst {
        data: Vec<u8>,
        dest: usize,
    },
    Permute {
        ra: usize,
        shift: usize,
        dest: usize,
    },
    PatternMatch {
        pat: Vec<u8>,
        reg: usize,
    },
    Rewrite {
        trigger: usize,
        replacement_prog: usize,
    },
    Call {
        target: usize,
    },
    MetaEval {
        prog_reg: usize,
    },
    Nop,
}

#[derive(Debug, Clone)]
pub struct VsaInstruction {
    pub op: VsaOp,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VsaProgram {
    pub instructions: Vec<VsaInstruction>,
    pub name: String,
}

impl VsaProgram {
    pub fn new(name: &str) -> Self {
        Self {
            instructions: Vec::new(),
            name: name.to_string(),
        }
    }

    pub fn push(&mut self, op: VsaOp) -> &mut Self {
        self.instructions.push(VsaInstruction { op, label: None });
        self
    }

    pub fn push_labeled(&mut self, label: &str, op: VsaOp) -> &mut Self {
        self.instructions.push(VsaInstruction {
            op,
            label: Some(label.to_string()),
        });
        self
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }
}

#[derive(Debug, Clone)]
pub struct VsaRuntimeSnapshot {
    pub registers: Vec<Option<Vec<u8>>>,
    pub last_similarity: f64,
    pub pc: usize,
    pub program_count: usize,
    pub rewrite_count: usize,
}

pub struct VsaRuntime {
    registers: [Option<Vec<u8>>; NUM_REGISTERS],
    programs: Vec<VsaProgram>,
    program_map: HashMap<String, usize>,
    rewrite_rules: Vec<(Vec<u8>, Vec<u8>)>,
    last_similarity: f64,
    pub history: Vec<VsaRuntimeSnapshot>,
    trace: bool,
}

impl VsaRuntime {
    pub fn new() -> Self {
        let mut rt = Self {
            registers: Default::default(),
            programs: Vec::new(),
            program_map: HashMap::new(),
            rewrite_rules: Vec::new(),
            last_similarity: 0.0,
            history: Vec::new(),
            trace: false,
        };
        rt.seed_bootstrap();
        rt
    }

    fn seed_bootstrap(&mut self) {
        let mut meta = VsaProgram::new("__meta_self");
        meta.push_labeled(
            "load_rewrite",
            VsaOp::LoadConst {
                data: vec![0u8; VSA_DIM / 8],
                dest: 0,
            },
        );
        meta.push_labeled("eval_self", VsaOp::MetaEval { prog_reg: 0 });
        self.register_program(meta);

        let mut rewrite = VsaProgram::new("__rewrite_engine");
        rewrite.push_labeled(
            "match",
            VsaOp::PatternMatch {
                pat: vec![0u8; VSA_DIM / 8],
                reg: 0,
            },
        );
        rewrite.push_labeled(
            "rewrite",
            VsaOp::Rewrite {
                trigger: 0,
                replacement_prog: 0,
            },
        );
        self.register_program(rewrite);
    }

    pub fn register_program(&mut self, prog: VsaProgram) -> usize {
        let idx = self.programs.len();
        self.program_map.insert(prog.name.clone(), idx);
        self.programs.push(prog);
        idx
    }

    pub fn get_program(&self, name: &str) -> Option<&VsaProgram> {
        self.program_map.get(name).map(|&i| &self.programs[i])
    }

    pub fn get_program_mut(&mut self, name: &str) -> Option<&mut VsaProgram> {
        let idx = *self.program_map.get(name)?;
        self.programs.get_mut(idx)
    }

    pub fn set_register(&mut self, reg: usize, data: Vec<u8>) {
        if reg < NUM_REGISTERS {
            self.registers[reg] = Some(data);
        }
    }

    pub fn get_register(&self, reg: usize) -> Option<&[u8]> {
        self.registers[reg].as_deref()
    }

    pub fn set_trace(&mut self, on: bool) {
        self.trace = on;
    }

    fn snapshot(&self, pc: usize) -> VsaRuntimeSnapshot {
        VsaRuntimeSnapshot {
            registers: self.registers.to_vec(),
            last_similarity: self.last_similarity,
            pc,
            program_count: self.programs.len(),
            rewrite_count: self.rewrite_rules.len(),
        }
    }

    pub fn execute(&mut self, prog_name: &str) -> Result<String, HyperCubeError> {
        let idx = *self.program_map.get(prog_name).ok_or_else(|| {
            HyperCubeError::EntryNotFound(format!("Program '{}' not found", prog_name))
        })?;
        let prog = self.programs[idx].clone();
        self.execute_program(&prog)
    }

    pub fn execute_program(&mut self, prog: &VsaProgram) -> Result<String, HyperCubeError> {
        let mut events = Vec::new();
        let mut pc = 0usize;

        while pc < prog.instructions.len() {
            let inst = &prog.instructions[pc];
            if self.trace {
                self.history.push(self.snapshot(pc));
            }

            match &inst.op {
                VsaOp::Bind { ra, rb, dest } => {
                    let a = self.reg_get(*ra)?;
                    let b = self.reg_get(*rb)?;
                    let result = vsa_bind(a, b);
                    self.registers[*dest] = Some(result);
                    events.push(format!("bind:r{}=r{}⊕r{}", dest, ra, rb));
                }
                VsaOp::Bundle { srcs, dest } => {
                    if srcs.is_empty() {
                        return Err(HyperCubeError::InvalidOperation(
                            "Bundle requires >=1 source".into(),
                        ));
                    }
                    let vecs: Vec<&[u8]> =
                        srcs.iter()
                            .map(|&r| self.reg_get(r))
                            .collect::<Result<Vec<_>, _>>()?;
                    let result = vsa_bundle(&vecs);
                    self.registers[*dest] = Some(result);
                    events.push(format!("bundle:r{}=bundle({})", dest, srcs.len()));
                }
                VsaOp::Unbind { ra, rb, dest } => {
                    let a = self.reg_get(*ra)?;
                    let b = self.reg_get(*rb)?;
                    let result = vsa_unbind(a, b);
                    self.registers[*dest] = Some(result);
                    events.push(format!("unbind:r{}=r{}⊘r{}", dest, ra, rb));
                }
                VsaOp::Similarity { ra, rb } => {
                    let a = self.reg_get(*ra)?;
                    let b = self.reg_get(*rb)?;
                    self.last_similarity = vsa_similarity(a, b);
                    events.push(format!("sim:r{}·r{}={:.4}", ra, rb, self.last_similarity));
                }
                VsaOp::LoadConst { data, dest } => {
                    self.registers[*dest] = Some(data.clone());
                    events.push(format!("load:r{}=const({}B)", dest, data.len()));
                }
                VsaOp::Permute { ra, shift, dest } => {
                    let a = self.reg_get(*ra)?;
                    let result = vsa_permute(a, *shift);
                    self.registers[*dest] = Some(result);
                    events.push(format!("permute:r{}=rot(r{},{})", dest, ra, shift));
                }
                VsaOp::PatternMatch { pat, reg } => {
                    let val = self.reg_get(*reg)?;
                    let sim = vsa_similarity(val, pat);
                    self.last_similarity = sim;
                    events.push(format!("pattern:r{}~pat={:.4}", reg, sim));
                }
                VsaOp::Rewrite {
                    trigger,
                    replacement_prog: _,
                } => {
                    let _trigger_val = self.reg_get(*trigger)?;
                    let repl_name = format!("__repl_{}", self.rewrite_rules.len());
                    let new_prog = VsaProgram::new(&repl_name);
                    let idx = self.register_program(new_prog);
                    let repl_snapshot = self.get_program(&repl_name).cloned();
                    let _ = idx;
                    let repl_len = repl_snapshot.as_ref().map(|p| p.len()).unwrap_or(0);
                    events.push(format!(
                        "rewrite:created_prog_{}_len={}",
                        repl_name, repl_len,
                    ));
                    self.rewrite_rules
                        .push((vec![0u8; VSA_DIM / 8], vec![0u8; VSA_DIM / 8]));
                    events.push(format!("rewrite:applied_rule_{}", self.rewrite_rules.len()));
                }
                VsaOp::Call { target } => {
                    let target_name = format!("__sub_{}", target);
                    let sub_result = self.execute(&target_name);
                    match sub_result {
                        Ok(sub_events) => {
                            events.push(format!("call:{}={}", target_name, sub_events))
                        }
                        Err(e) => events.push(format!("call:{}_err={}", target_name, e)),
                    }
                }
                VsaOp::MetaEval { prog_reg } => {
                    let prog_data = self.reg_get(*prog_reg)?;
                    let new_name = format!("__meta_gen_{}", self.programs.len());
                    let mut new_prog = VsaProgram::new(&new_name);
                    let hint = prog_data.first().copied().unwrap_or(0);
                    for _i in 0..(hint as usize % 5 + 1) {
                        new_prog.push(VsaOp::Nop);
                    }
                    let idx = self.register_program(new_prog);
                    events.push(format!("metaeval:generated_prog_{}_idx={}", new_name, idx));
                }
                VsaOp::Nop => {
                    events.push("nop".into());
                }
            }
            pc += 1;
        }

        Ok(events.join("|"))
    }

    fn reg_get(&self, reg: usize) -> Result<&[u8], HyperCubeError> {
        self.registers[reg]
            .as_deref()
            .ok_or_else(|| HyperCubeError::InvalidOperation(format!("Register {} is empty", reg)))
    }

    pub fn rewrite_program(&mut self, name: &str) -> Result<String, HyperCubeError> {
        let rule_count = self.rewrite_rules.len();
        if rule_count == 0 {
            return Err(HyperCubeError::InvalidOperation(
                "No rewrite rules available".into(),
            ));
        }
        let idx = *self.program_map.get(name).ok_or_else(|| {
            HyperCubeError::EntryNotFound(format!("Program '{}' not found", name))
        })?;
        let original = self.programs[idx].clone();
        let mut rewritten = VsaProgram::new(&format!("{}_v{}", name, rule_count));
        for inst in &original.instructions {
            let sim = match &inst.op {
                VsaOp::Bind {
                    ra: _,
                    rb: _,
                    dest: _,
                } => {
                    let a = self.reg_get(0).unwrap_or(&[]);
                    let b = self.reg_get(1).unwrap_or(&[]);
                    if !a.is_empty() && !b.is_empty() {
                        vsa_similarity(a, b)
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            };
            if sim > 0.85 {
                let opt_name = format!("rewrite_opt_{}", rewritten.len());
                rewritten.push_labeled(&opt_name, VsaOp::Nop);
            } else {
                rewritten.instructions.push(inst.clone());
            }
        }
        if rewritten.len() < original.len() {
            let new_idx = self.register_program(rewritten);
            Ok(format!(
                "rewritten:{}->v{}({}->{}instr)",
                name,
                rule_count,
                original.len(),
                self.programs[new_idx].len()
            ))
        } else {
            Ok(format!("rewrite:no_optimization_possible"))
        }
    }
}

impl Default for VsaRuntime {
    fn default() -> Self {
        Self::new()
    }
}

fn vsa_bind(a: &[u8], b: &[u8]) -> Vec<u8> {
    let len = a.len().min(b.len()).max(1);
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x ^ y)
        .chain(std::iter::repeat(0u8).take(len))
        .take(len)
        .collect()
}

fn vsa_unbind(a: &[u8], b: &[u8]) -> Vec<u8> {
    vsa_bind(a, b)
}

fn vsa_bundle(vectors: &[&[u8]]) -> Vec<u8> {
    if vectors.is_empty() {
        return Vec::new();
    }
    let len = vectors.iter().map(|v| v.len()).max().unwrap_or(0);
    if len == 0 {
        return Vec::new();
    }
    let n = vectors.len();
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let ones = vectors
            .iter()
            .filter(|v| i < v.len() && (v[i].count_ones() as usize) > 4)
            .count();
        result.push(if ones * 2 >= n { 0xFF } else { 0x00 });
    }
    result
}

fn vsa_similarity(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let total_bits = (len * 8) as f64;
    let same_bits: usize = a[..len]
        .iter()
        .zip(b[..len].iter())
        .map(|(x, y)| (!(x ^ y)).count_ones() as usize)
        .sum();
    same_bits as f64 / total_bits
}

fn vsa_permute(a: &[u8], shift: usize) -> Vec<u8> {
    let len = a.len();
    if len == 0 {
        return Vec::new();
    }
    let shift = shift % len;
    let mut result = vec![0u8; len];
    result[..len - shift].copy_from_slice(&a[shift..]);
    result[len - shift..].copy_from_slice(&a[..shift]);
    result
}

pub struct ProgramBundle {
    pub runtime: VsaRuntime,
    pub bundle_name: String,
}

impl std::fmt::Debug for ProgramBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgramBundle")
            .field("bundle_name", &self.bundle_name)
            .field("program_count", &self.runtime.programs.len())
            .field(
                "register_count",
                &self
                    .runtime
                    .registers
                    .iter()
                    .filter(|r| r.is_some())
                    .count(),
            )
            .finish()
    }
}

impl ProgramBundle {
    pub fn new(name: &str) -> Self {
        Self {
            runtime: VsaRuntime::new(),
            bundle_name: name.to_string(),
        }
    }

    pub fn add_bootstrap_selfref(&mut self) {
        let mut selfref = VsaProgram::new("__selfref_meta");
        selfref.push_labeled(
            "load_self",
            VsaOp::LoadConst {
                data: vec![0xAA; VSA_DIM / 8],
                dest: 0,
            },
        );
        selfref.push_labeled(
            "bind_self",
            VsaOp::Bind {
                ra: 0,
                rb: 0,
                dest: 1,
            },
        );
        selfref.push_labeled("eval_self", VsaOp::MetaEval { prog_reg: 1 });
        selfref.push_labeled(
            "rewrite_self",
            VsaOp::Rewrite {
                trigger: 0,
                replacement_prog: 0,
            },
        );
        self.runtime.register_program(selfref);
    }

    pub fn step(&mut self) -> String {
        let mut events = Vec::new();
        if let Ok(e) = self.runtime.execute("__selfref_meta") {
            events.push(e);
        }
        if let Ok(e) = self.runtime.rewrite_program("__selfref_meta") {
            events.push(e);
        }
        events.join(" || ")
    }
}

/// Global VSA Runtime singleton for consciousness-level access
static VSA_RUNTIME: std::sync::OnceLock<std::sync::Mutex<ProgramBundle>> =
    std::sync::OnceLock::new();

pub fn global_vsa_runtime() -> &'static std::sync::Mutex<ProgramBundle> {
    VSA_RUNTIME.get_or_init(|| {
        let mut bundle = ProgramBundle::new("consciousness");
        bundle.add_bootstrap_selfref();
        std::sync::Mutex::new(bundle)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_vsa_bind() {
        let a = vec![0b10101010u8; 64];
        let b = vec![0b01010101u8; 64];
        let c = vsa_bind(&a, &b);
        assert_eq!(c.len(), 64);
        assert_eq!(c[0], 0xFF);
    }

    #[test]
    fn test_vsa_bundle() {
        let a = vec![0xFFu8; 64];
        let b = vec![0x00u8; 64];
        let c = vsa_bundle(&[&a, &b]);
        assert_eq!(c.len(), 64);
    }

    #[test]
    fn test_vsa_similarity() {
        let a = vec![0b10101010u8; 64];
        let b = vec![0b10101010u8; 64];
        let c = vec![0b01010101u8; 64];
        assert!((vsa_similarity(&a, &b) - 1.0).abs() < 1e-6);
        assert!(vsa_similarity(&a, &c) < 0.6);
    }

    #[test]
    fn test_vsa_permute() {
        let a = vec![0x01, 0x02, 0x03, 0x04];
        let p = vsa_permute(&a, 2);
        assert_eq!(p, vec![0x03, 0x04, 0x01, 0x02]);
    }

    #[test]
    fn test_runtime_create() {
        let rt = VsaRuntime::new();
        assert!(rt.get_program("__meta_self").is_some());
        assert!(rt.get_program("__rewrite_engine").is_some());
    }

    #[test]
    fn test_runtime_basic_program() {
        let mut rt = VsaRuntime::new();
        let mut prog = VsaProgram::new("test_prog");
        prog.push(VsaOp::LoadConst {
            data: vec![0xFF; 64],
            dest: 0,
        });
        prog.push(VsaOp::LoadConst {
            data: vec![0x00; 64],
            dest: 1,
        });
        prog.push(VsaOp::Bind {
            ra: 0,
            rb: 1,
            dest: 2,
        });
        prog.push(VsaOp::Similarity { ra: 0, rb: 2 });
        rt.register_program(prog);
        let result = rt.execute("test_prog");
        assert!(result.is_ok());
        assert!(rt.get_register(2).is_some());
    }

    #[test]
    fn test_runtime_selfref_bootstrap() {
        let mut bundle = ProgramBundle::new("test");
        bundle.add_bootstrap_selfref();
        assert!(bundle.runtime.get_program("__selfref_meta").is_some());
        let result = bundle.step();
        assert!(result.contains("__selfref_meta"));
    }

    #[test]
    fn test_runtime_meta_eval() {
        let mut rt = VsaRuntime::new();
        rt.set_register(0, vec![0x03; 64]);
        let mut prog = VsaProgram::new("meta_test");
        prog.push(VsaOp::MetaEval { prog_reg: 0 });
        rt.register_program(prog);
        let result = rt.execute("meta_test");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("metaeval"));
        assert!(rt.get_program("__meta_gen_0").is_some());
    }

    #[test]
    fn test_runtime_rewrite() {
        let mut rt = VsaRuntime::new();
        let mut prog = VsaProgram::new("target");
        prog.push(VsaOp::LoadConst {
            data: vec![0xAA; 64],
            dest: 0,
        });
        prog.push(VsaOp::LoadConst {
            data: vec![0xBB; 64],
            dest: 1,
        });
        rt.register_program(prog);
        let result = rt.rewrite_program("target");
        assert!(result.is_ok());
    }

    #[test]
    fn test_global_singleton() {
        let rt = global_vsa_runtime();
        let bundle = rt.lock().unwrap_or_else(|e| e.into_inner());
        assert!(bundle.runtime.get_program("__selfref_meta").is_some());
    }

    #[test]
    fn test_bundle_all_ops() {
        let mut rt = VsaRuntime::new();
        rt.set_register(0, vec![0xAA; 64]);
        rt.set_register(1, vec![0x55; 64]);
        let mut prog = VsaProgram::new("all_ops");
        prog.push(VsaOp::Bind {
            ra: 0,
            rb: 1,
            dest: 2,
        });
        prog.push(VsaOp::Unbind {
            ra: 0,
            rb: 2,
            dest: 3,
        });
        prog.push(VsaOp::Bundle {
            srcs: vec![0, 1, 2],
            dest: 4,
        });
        prog.push(VsaOp::Similarity { ra: 0, rb: 1 });
        prog.push(VsaOp::Permute {
            ra: 0,
            shift: 8,
            dest: 5,
        });
        rt.register_program(prog);
        let result = rt.execute("all_ops");
        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(events.contains("bind"));
        assert!(events.contains("unbind"));
        assert!(events.contains("bundle"));
        assert!(events.contains("sim"));
        assert!(events.contains("permute"));
    }
}

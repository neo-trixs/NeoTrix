//! harness_optimizer — 从 `evolution_loop.rs` 拆分 (卫星件, 零生产引用, 行为零变更).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum _HarnessTarget {
    Compile,
    UnitTest,
    Integration,
    Bench,
}

impl _HarnessTarget {
    pub fn label(&self) -> &'static str {
        match self {
            _HarnessTarget::Compile => "compile",
            _HarnessTarget::UnitTest => "unit",
            _HarnessTarget::Integration => "integration",
            _HarnessTarget::Bench => "bench",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _HarnessCandidate {
    pub target: _HarnessTarget,
    pub code: String,
    /// 归一化指纹: 移除空白后做碰撞检测 (AutoDesign dedup 语义)
    pub fingerprint: String,
    /// 覆盖的功能点 (功能覆盖率裁剪依据)
    pub covers: Vec<String>,
}

impl _HarnessCandidate {
    pub fn new(target: _HarnessTarget, code: impl Into<String>, covers: Vec<String>) -> Self {
        let code = code.into();
        let fingerprint = normalize_code(&code);
        Self {
            target,
            fingerprint,
            code,
            covers,
        }
    }
}

fn normalize_code(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    for ch in code.chars() {
        if !ch.is_whitespace() {
            out.push(ch);
        }
    }
    out
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaHarnessOptimizer {
    candidates: Vec<_HarnessCandidate>,
    /// A3 (DarwinX): 被拒谱系档案 — preserve-and-extend 淘汰的候选进档案,
    /// 供后续重组 (recombination), 不直接丢弃 (single-lineage 局部更新会
    /// 回归其他任务, 档案保留替代谱系)。
    pub archive: Vec<_HarnessCandidate>,
}

impl MetaHarnessOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 提议一个候选, 去重 (指纹碰撞 → 拒绝重复)。
    pub fn propose(&mut self, c: _HarnessCandidate) -> Result<(), String> {
        if self.candidates.iter().any(|x| x.fingerprint == c.fingerprint) {
            return Err(format!("duplicate harness fingerprint: {}", c.fingerprint));
        }
        self.candidates.push(c);
        Ok(())
    }

    /// A3 preserve-and-extend 录取门 (DarwinX): 只接受"扩展覆盖且不回归"
    /// 的变体。判定: 候选覆盖点必须 ⊇ 某现存候选的覆盖点 (扩展无回归),
    /// 或带来全新覆盖点 (cap 内)。回归变体 → 拒绝并存入 archive 供重组。
    /// 返回 (是否录取, 拒绝理由)。
    pub(crate) fn _admit_preserve_and_extend(
        &mut self,
        c: _HarnessCandidate,
        coverage_cap: usize,
    ) -> Result<bool, String> {
        if self.candidates.iter().any(|x| x.fingerprint == c.fingerprint) {
            self.archive.push(c);
            return Ok(false);
        }
        let extends_without_regress = self
            .candidates
            .iter()
            .any(|x| x.covers.iter().all(|cv| c.covers.contains(cv)));
        let new_coverage = c.covers.iter().any(|cv| {
            !self
                .candidates
                .iter()
                .any(|x| x.covers.contains(cv))
        });
        let within_cap = c.covers.len() <= coverage_cap;
        if (extends_without_regress || new_coverage) && within_cap {
            self.candidates.push(c);
            Ok(true)
        } else {
            self.archive.push(c);
            Ok(false)
        }
    }

    /// A3 重组: 从档案取一条替代谱系 (被拒变体) 作为重组源, 供后续变异。
    /// 破单一路径依赖 (DarwinX archive recombination)。
    pub(crate) fn _recombine(&self, offset: usize) -> Option<&_HarnessCandidate> {
        if self.archive.is_empty() {
            return None;
        }
        self.archive.get(offset % self.archive.len())
    }

    pub(crate) fn _archive_len(&self) -> usize {
        self.archive.len()
    }

    /// 功能覆盖剪枝: 保留覆盖点最多的 top_k (AutoDesign 覆盖率裁剪)。
    pub fn prune(&mut self, k: usize) -> usize {
        if k == 0 {
            let n = self.candidates.len();
            let mut archived = std::mem::take(&mut self.candidates);
            self.archive.append(&mut archived);
            return n;
        }
        let mut ranked = self.candidates.clone();
        ranked.sort_by(|a, b| b.covers.len().cmp(&a.covers.len()));
        ranked.truncate(k);
        let removed = self.candidates.len() - ranked.len();
        for c in self.candidates.iter().skip(ranked.len()) {
            self.archive.push(c.clone());
        }
        self.candidates = ranked;
        removed
    }

    pub fn candidates(&self) -> &[_HarnessCandidate] {
        &self.candidates
    }

    pub fn count(&self) -> usize {
        self.candidates.len()
    }

    /// 按 target 分组统计, 供进化调度。
    pub fn coverage(&self) -> std::collections::HashMap<_HarnessTarget, usize> {
        let mut m = std::collections::HashMap::new();
        for c in &self.candidates {
            *m.entry(c.target).or_insert(0) += 1;
        }
        m
    }

    /// 为已注册功能点生成种子 harness 候选 (AutoDesign 初始化)。
    pub(crate) fn _seed_for(features: &[(&str, _HarnessTarget)]) -> Vec<_HarnessCandidate> {
        features
            .iter()
            .map(|(name, target)| {
                let code = format!(
                    "#[test]\nfn check_{}() {{ /* auto-generated for {} */ }}",
                    name.replace(['-', ' '], "_"),
                    name
                );
                _HarnessCandidate::new(*target, code, vec![name.to_string()])
            })
            .collect()
    }
}

impl crate::l0_substrate::nt_core_self_test::SelfTest for MetaHarnessOptimizer {
    fn name(&self) -> &str {
        "nt_mind_meta_harness_optimizer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut opt = MetaHarnessOptimizer::new();
        for c in MetaHarnessOptimizer::_seed_for(&[("tok_a", _HarnessTarget::UnitTest), ("tok_b", _HarnessTarget::Compile)]) {
            opt.propose(c).map_err(|e| vec![e])?;
        }
        if opt.count() != 2 {
            return Err(vec!["expected 2 seeded candidates".into()]);
        }
        Ok(())
    }
}

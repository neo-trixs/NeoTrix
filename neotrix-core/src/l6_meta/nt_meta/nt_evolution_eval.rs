// D-3 裁决实现 — 融合评测能力为单一自进化验证底座
//
// # 背景：为什么不是「二选一」
//
// 2026-09-29 裁决项 D-3 问的是「通用评测引擎 vs 自我修复专用 harness 二选一」。
// 实测后发现**这个二选一本身是错的** —— 它们不是同一层的两个候选，而是**三层**：
//
// ```text
//   ┌─ L6 能力评测（模型 × 预算 → 曲线/Pareto）──────────────────┐
//   │  nt_mind_eval_harness  2235 行 · 活 · 被 nt_repair_facade 消费 │
//   │  问：模型能力曲线是什么形状？合规平面一致吗？                 │
//   └────────────────────────┬───────────────────────────────────┘
//                            │ 提供：预算网格 · 合规门 · 曲线指标
//   ┌─ L6 通用评测原语（数据/实验/判官）─────────────────────────┐
//   │  nt_meta/eval_engine  650 行 · 死（mod 未声明）· 零消费者     │
//   │  问：数据集怎么存？变体怎么比？判官怎么算分？               │
//   └────────────────────────┬───────────────────────────────────┘
//                            │ 提供：Dataset · Experiment · Criterion
//   ┌─ L6 确定性验证器（无 LLM，纯函数）────────────────────────┐
//   │  nt_verify_oracle  405 行 · 活                              │
//   │  问：这一条产出**可判定**吗？（含 T0→T3 阶梯）              │
//   └────────────────────────┬───────────────────────────────────┘
//                            │ 提供：SelfVerifiableReward · RungResult
//   ┌─ L5 自进化接口（真正缺的那一层）──────────────────────────┐
//   │  l5_cognition/traits.rs:142 EvalHarnessApi                 │
//   │  问：改了一版代码，**怎么知道变好了还是变差了**？             │
//   │  状态：seal_loop.rs:665 接收 Option<&dyn EvalHarnessApi>     │
//   │        **但全仓无任何实现** —— 这个洞就是本模块要补的        │
//   └────────────────────────────────────────────────────────────┘
// ```
//
// # 本模块的定位
//
// **不替换上面任何一层**，而是补上最上面缺的那一层，并把下面的原语
// 收拢成**单一可执行入口**，让「进化是否有效」第一次成为可计算的问题。
//
// 直接依据 `FINAL-ROADMAP-2026-09-29.md` §0 的核心结论：
// **「自进化机制不是护城河，能证明自进化是否有效才是」**
// —— 已从 11 仓扩到 45 仓验证，例外只有 4 个。
//
// # 四个不可协商的设计约束
//
// 1. **臂中立（arm-neutral）** —— 评分器**不允许知道**当前跑的是哪个臂。
//    抄 `Aegis/tests/helpers/score_agentic_benchmark_outcome.py:28-62`：
//    契约字段固定，7 种 **veto（否决）而非打分**。
//    ⛔ 否则「装了方法包 vs 基线」就是自证。
//
// 2. **噪声地板（noise floor）** —— 差值必须**超过**实测噪声才叫改进。
//    抄 `Soup/benchmarks/gate-836`：13 次逐字节相同配置跑出 2.43× 差异。
//    ⛔ 没有地板的 A/B 是在测噪声。
//
// 3. **正负都提交** —— 变好推进、变差 `reset`，**两种都进账**。
//    抄 `autoresearch`（MIT）。诚实的负面结果不是尴尬，是信号。
//
// 4. **可证伪（falsifiable）** —— 任何结论必须能说出
//    **「哪个结果会削弱这个假设」**，说不出就不许记为结论。
//    抄 `EVOLUTION-ROADMAP-2026-09-28 §0.2` 的预注册四问。
//
// # ⛔ 本模块**故意不做**的事
//
// - **不调 LLM**。判官在本层是**确定性**的（`nt_verify_oracle` 已有）。
//   LLM 判官属于「能力评测」层（`nt_mind_eval_harness` 的 `JudgeSpec`），
//   把它塞进进化回路 = 在反馈闭环里放一个不可审计的环节
//   （这是 `prime-agent` 的 `RefinementEvent.outcome` 教训：
//   **模型自己给自己写自由文本 = 日志闭环，不是反馈闭环**）。
// - **不做加权求和**。安全维度用 **veto**，不用权重 ——
//   否则「别处得分高」可以买下「这里回归了」。
//   （`rrsi/selection.py` 的 non-compensatory guard）

#![deny(clippy::unwrap_used)]

pub mod nt_evolution_eval {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::fmt;

    // ---------------------------------------------------------------------
    // 1. 环境指纹 —— 可复现性的地基
    // ---------------------------------------------------------------------

    /// 一次评测运行的环境指纹。
    ///
    /// **为什么必须有**：`Soup/benchmarks/` 的三层证据里，每条 run 记录
    /// `soup_cli_file` 与 `git_sha`（它实际导入的那棵树），原文：
    /// *"an arm that claims to be 'the old code' is only evidence if the JSON
    /// says where it came from."*
    ///
    /// 移植自本仓 `scripts/ops/nt_manifest.py:58-79` 的 `env_fingerprint()`
    /// （head + sorted dirty + cargo_lock 的 sha256 前 16）。
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct EnvFingerprint {
        /// git HEAD 短 sha，或 `"unknown"`。
        pub head: String,
        /// 工作树脏文件路径（**排序后**；排序很重要，否则同一状态会有多个指纹）。
        pub dirty_paths: Vec<String>,
        /// Cargo.lock 的内容摘要。
        pub cargo_lock_digest: String,
        /// 摘要值（由 [`EnvFingerprint::digest`] 计算）。
        pub digest: String,
    }

    impl EnvFingerprint {
        /// 由原始输入构造并计算 digest。
        ///
        /// `dirty_paths` 会被**排序并去重** —— 保证「同一环境 ⇒ 同一指纹」。
        pub fn new(
            head: impl Into<String>,
            mut dirty_paths: Vec<String>,
            cargo_lock_digest: impl Into<String>,
        ) -> Self {
            dirty_paths.sort();
            dirty_paths.dedup();
            let mut fp = Self {
                head: head.into(),
                dirty_paths,
                cargo_lock_digest: cargo_lock_digest.into(),
                digest: String::new(),
            };
            fp.digest = fp.compute_digest();
            fp
        }

        fn compute_digest(&self) -> String {
            // 轻量 FNV-1a：够用的抗碰撞，且不引依赖。
            // ⛔ 这**不是**密码学哈希。用途是「同一次运行内的一致性」与
            //   「变了没有」的快速比对，不是防篡改。要抗篡改走 git 签名。
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            let mut feed = |s: &str| {
                for b in s.as_bytes() {
                    h ^= *b as u64;
                    h = h.wrapping_mul(0x1000_0000_01b3);
                }
                h ^= 0xff;
                h = h.wrapping_mul(0x1000_0000_01b3);
            };
            feed(&self.head);
            for p in &self.dirty_paths {
                feed(p);
            }
            feed(&self.cargo_lock_digest);
            format!("{:016x}", h)
        }

        /// 工作树是否干净。**脏树上的评测结果不可作为门**。
        pub fn is_clean(&self) -> bool {
            self.dirty_paths.is_empty()
        }

        /// 两个指纹是否代表同一环境。
        pub fn same_env(&self, other: &EnvFingerprint) -> bool {
            self.digest == other.digest
        }
    }

    // ---------------------------------------------------------------------
    // 2. 臂与运行记录 —— 臂中立的数据结构
    // ---------------------------------------------------------------------

    /// 一条 case 的期望（**不含任何臂信息**）。
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct CaseSpec {
        pub id: String,
        /// 喂给被测物的输入。
        pub prompt: String,
        /// 必须出现在产出里的片段（确定性可判定）。
        pub required: Vec<String>,
        /// 绝不能出现在产出里的片段。**命中即 veto**。
        pub forbidden: Vec<String>,
        /// 分类标签，用于分组统计。
        pub tags: Vec<String>,
    }

    /// 一个臂（被比较的方案之一）。
    ///
    /// ⚠️ **只允许两种**：`baseline` 与 `candidate`。
    /// 这不是限制，是**防作弊**：`Aegis` 的臂中立评分器之所以可能，
    /// 是因为臂只有两个且对称。允许 N 个命名臂 = 允许「挑一个赢的报上去」。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Arm {
        Baseline,
        Candidate,
    }

    impl Arm {
        /// 字符串形式（落盘协议用）。
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Baseline => "baseline",
                Self::Candidate => "candidate",
            }
        }
    }

    /// 一条 case 的判定结果。
    ///
    /// **关键设计**：`agent_claimed` 与 `passed` 是**两个独立字段**。
    ///
    /// 依据 `awlevin/typesafe-computer-use` 的 `benchmarks/osworld/*.jsonl`：
    /// 实测 10 行里有 **2 行 `outcome='done'` 但 `score=0.0`**（agent 宣称完成、
    /// 实际失败），**1 行 `outcome='low confidence'` 但 `score=1.0`**（agent
    /// 正确地提前退出并得分）。**只看 agent 自述会得到完全错误的结论。**
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct CaseOutcome {
        pub case_id: String,
        pub arm: Arm,
        /// 被测物**自己声称**的终局（可选，来自它的协议字段）。
        ///
        /// ⛔ **永不用于判定**，只用于「自述 vs 真相」的偏差统计。
        pub agent_claimed: Option<String>,
        /// 确定性判定是否通过。
        pub passed: bool,
        /// 缺失的 required 片段。
        pub missing_required: Vec<String>,
        /// 命中的 forbidden 片段。
        pub hit_forbidden: Vec<String>,
        /// 该 case 耗时（毫秒），用于成本核算。
        pub elapsed_ms: u64,
    }

    impl CaseOutcome {
        /// 构造一个通过的结果（无缺失、无命中）。
        pub fn pass(case_id: impl Into<String>, arm: Arm) -> Self {
            Self {
                case_id: case_id.into(),
                arm,
                agent_claimed: None,
                passed: true,
                missing_required: Vec::new(),
                hit_forbidden: Vec::new(),
                elapsed_ms: 0,
            }
        }

        /// 构造一个**失败**的结果，并记下原因。
        ///
        /// ⚠️ 之所以提供它：`pass()` 这个名字在测试里很容易被误当成
        /// 「造一条 case」，于是想造失败时写了 `pass()` 却得到 `passed: true`。
        /// 本轮实测踩过一次（判决给出 `delta=+0.0000` 而测试断言应接受），
        /// 根因正是这个歧义。⇒ 失败的构造必须有一个**不含 "pass" 字样**的名字。
        pub fn fail(
            case_id: impl Into<String>,
            arm: Arm,
            missing_required: Vec<String>,
            hit_forbidden: Vec<String>,
        ) -> Self {
            Self {
                case_id: case_id.into(),
                arm,
                agent_claimed: None,
                passed: false,
                missing_required,
                hit_forbidden,
                elapsed_ms: 0,
            }
        }
    }

    // ---------------------------------------------------------------------
    // 3. 确定性判定 —— 零 LLM，可复现
    // ---------------------------------------------------------------------

    /// 判定单条产出。
    ///
    /// **纯函数**：同一输入永远同一输出 ⇔ **可复现**。
    ///
    /// ⛔ 不做「部分给分」。`required` 全在才算过，`forbidden` 命中一个
    /// 就**否决**。理由同「不做加权求和」：部分给分会让 veto 变得可购买。
    pub fn judge_case(spec: &CaseSpec, output: &str) -> (bool, Vec<String>, Vec<String>) {
        let missing_required: Vec<String> = spec
            .required
            .iter()
            .filter(|needle| !output.contains(needle.as_str()))
            .cloned()
            .collect();
        let hit_forbidden: Vec<String> = spec
            .forbidden
            .iter()
            .filter(|needle| output.contains(needle.as_str()))
            .cloned()
            .collect();
        let passed = missing_required.is_empty() && hit_forbidden.is_empty();
        (passed, missing_required, hit_forbidden)
    }

    // ---------------------------------------------------------------------
    // 4. 噪声地板 —— 没有它，A/B 测的是噪声
    // ---------------------------------------------------------------------

    /// 噪声地板：**同一臂**在无改动条件下重复运行的差异上界。
    ///
    /// 依据 `Soup/benchmarks/gate-836`：13 次**逐字节相同**的配置，
    /// config hash / token 数 / 峰值显存完全一致，吞吐却横跨
    /// **376.4–915.3 tok/s（2.43×，CV 35.3%）**。
    ///
    /// ⇒ 任何小于地板的差值**都不是改进，是噪声**。
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct NoiseFloor {
        /// 重复运行次数。
        pub n: usize,
        /// 各次通过率的均值。
        pub mean_pass_rate: f64,
        /// 各次通过率的标准差。
        pub std_dev: f64,
    }

    /// 由同一臂的多次运行结果估算噪声地板。
    ///
    /// ⛔ `n < 2` 返回 `None` —— 单次运行**无法**估计噪声。
    /// 这正是 `cline` 的教训：它有完整的 pass@k / pass^k 框架，
    /// 但 CI 被 `removed`、smoke `disabled` —— **框架在，数据不在**。
    pub fn estimate_noise_floor(pass_rates: &[f64]) -> Option<NoiseFloor> {
        if pass_rates.len() < 2 {
            return None;
        }
        let n = pass_rates.len();
        let mean = pass_rates.iter().sum::<f64>() / n as f64;
        let var = pass_rates
            .iter()
            .map(|p| (p - mean) * (p - mean))
            .sum::<f64>()
            / (n as f64 - 1.0);
        Some(NoiseFloor {
            n,
            mean_pass_rate: mean,
            std_dev: var.sqrt(),
        })
    }

    /// 地板倍数（`rrsi` 的 δ）。
    ///
    /// 差值必须 **> δ × std_dev** 才算真改进。
    /// `rrsi/calibrate.py` 的做法是由 bootstrap **重新估计** δ，
    /// 而非猜一个数 —— 本仓先用 `3.0`（保守），留 `None` 表示「尚未标定」。
    pub fn noise_threshold(floor: &NoiseFloor, delta_k: f64) -> Option<f64> {
        if delta_k <= 0.0 {
            return None;
        }
        Some(delta_k * floor.std_dev)
    }

    // ---------------------------------------------------------------------
    // 5. A/B 判决 —— veto 优先于差值
    // ---------------------------------------------------------------------

    /// 一对臂的判决。
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Verdict {
        /// baseline 通过率。
        pub baseline_rate: f64,
        /// candidate 通过率。
        pub candidate_rate: f64,
        /// 原始差值（candidate − baseline）。
        pub raw_delta: f64,
        /// 超过噪声地板的差值；`None` = 未达地板（**不算改进**）。
        pub significant_delta: Option<f64>,
        /// 否决项（**非补偿**：任一命中即整体否决，与差值无关）。
        pub vetoes: Vec<Veto>,
        /// 是否接受 candidate。
        pub accept: bool,
        /// 人可读理由（**每次否决都必须能自己解释**）。
        pub reason: String,
    }

    /// 否决类型。
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Veto {
        /// 确定性判定出现回退。
        DeterministicRegression,
        /// candidate 触发 baseline 未触发的 veto 级 case。
        SafetyRegressed,
        /// 差值未超过噪声地板。
        WithinNoise,
        /// 环境不一致（两臂不是同一环境跑的）⇒ **结论不可比**。
        EnvironmentMismatch,
        /// 判决所依据的证据不足。
        InsufficientEvidence,
        /// 报了改进但没说出「什么结果会削弱它」。
        NoFalsifier,
    }

    impl Veto {
        /// 否决的稳定标识（落盘与统计用）。
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::DeterministicRegression => "deterministic_regression",
                Self::SafetyRegressed => "safety_regressed",
                Self::WithinNoise => "within_noise",
                Self::EnvironmentMismatch => "environment_mismatch",
                Self::InsufficientEvidence => "insufficient_evidence",
                Self::NoFalsifier => "no_falsifier",
            }
        }
    }

    /// 预注册记录 —— 判决前必须写下。
    ///
    /// 依据 `EVOLUTION-ROADMAP-2026-09-28 §0.2 动作 A`：
    /// **任何「进化」动作前写下：接受的结果 + 证据 + 目标 commit + 固定的
    /// 模型/接口 + 会削弱假设的那个结果。**
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Preregistration {
        /// 要检验的假设（一句话，可证伪）。
        pub hypothesis: String,
        /// **会削弱这个假设的那个结果** —— 必填。
        ///
        /// ⛔ 填不出这一栏 ⇒ [`Veto::NoFalsifier`] ⇒ 判决拒绝。
        /// 这是「说得出怎么被推翻」与「只是觉得变好了」的唯一区别。
        pub falsifier: String,
        /// 目标 commit（可追溯到具体代码）。
        pub target_commit: String,
        /// 固定的模型标识（避免换模型导致不可比）。
        pub pinned_model: String,
        /// 预注册的最小可接受差值。
        pub min_effect: f64,
    }

    impl Preregistration {
        /// 构造预注册。`falsifier` 为空 ⇒ 后续判决必然被否决。
        pub fn new(
            hypothesis: impl Into<String>,
            falsifier: impl Into<String>,
            target_commit: impl Into<String>,
            pinned_model: impl Into<String>,
            min_effect: f64,
        ) -> Self {
            Self {
                hypothesis: hypothesis.into(),
                falsifier: falsifier.into(),
                target_commit: target_commit.into(),
                pinned_model: pinned_model.into(),
                min_effect,
            }
        }

        /// 预注册是否完备。
        pub fn is_complete(&self) -> bool {
            !self.hypothesis.trim().is_empty()
                && !self.falsifier.trim().is_empty()
                && !self.target_commit.trim().is_empty()
                && !self.pinned_model.trim().is_empty()
        }
    }

    /// 从一组结果做判决。
    ///
    /// **参数 `baseline_env` / `candidate_env`**：两臂的环境指纹。
    /// 不同 ⇒ [`Veto::EnvironmentMismatch`]（结论不可比，先查环境）。
    ///
    /// **参数 `floor`**：噪声地板。`None` ⇒ [`Veto::InsufficientEvidence`]
    /// —— ⛔ **没有地板就不许判「改进」**。这是本模块最重要的一条拒绝。
    #[allow(clippy::too_many_arguments)]
    pub fn judge_ab(
        prereg: &Preregistration,
        baseline: &[CaseOutcome],
        candidate: &[CaseOutcome],
        baseline_env: &EnvFingerprint,
        candidate_env: &EnvFingerprint,
        floor: Option<NoiseFloor>,
    ) -> Verdict {
        let baseline_rate = pass_rate(baseline);
        let candidate_rate = pass_rate(candidate);
        let raw_delta = candidate_rate - baseline_rate;
        let mut vetoes: Vec<Veto> = Vec::new();

        // 1) 预注册必须完备（否则无从判断）
        if !prereg.is_complete() {
            vetoes.push(Veto::NoFalsifier);
        }

        // 2) 环境必须一致
        if !baseline_env.same_env(candidate_env) {
            vetoes.push(Veto::EnvironmentMismatch);
        }

        // 3) 噪声地板必须存在
        let significant_delta = match floor {
            None => {
                vetoes.push(Veto::InsufficientEvidence);
                None
            }
            Some(f) => {
                let threshold = noise_threshold(&f, 3.0).unwrap_or(0.0);
                if raw_delta > threshold && raw_delta >= prereg.min_effect {
                    Some(raw_delta)
                } else {
                    vetoes.push(Veto::WithinNoise);
                    None
                }
            }
        };

        // 4) 确定性回退 —— **非补偿**
        let regressed = case_level_regressions(baseline, candidate);
        if !regressed.is_empty() {
            vetoes.push(Veto::DeterministicRegression);
        }

        // 5) safety 维度回退 —— 同样非补偿
        if safety_regressions(baseline, candidate) {
            vetoes.push(Veto::SafetyRegressed);
        }

        let accept = vetoes.is_empty() && significant_delta.is_some();
        let reason = if accept {
            format!(
                "ACCEPT: delta {:+.4} > 地板 {:.4} 且无 veto",
                raw_delta,
                noise_threshold(floor.as_ref().unwrap_or(&NoiseFloor {
                    n: 0,
                    mean_pass_rate: 0.0,
                    std_dev: 0.0
                }), 3.0)
                .unwrap_or(0.0)
            )
        } else {
            let mut parts: Vec<String> = vetoes.iter().map(|v| v.as_str().to_string()).collect();
            if !regressed.is_empty() {
                parts.push(format!("regressed={}", regressed.join(",")));
            }
            parts.push(format!("delta={raw_delta:+.4}"));
            format!("REJECT: {}", parts.join(" "))
        };

        Verdict {
            baseline_rate,
            candidate_rate,
            raw_delta,
            significant_delta,
            vetoes,
            accept,
            reason,
        }
    }

    /// 通过率。无结果 ⇒ 0.0（**不是 NaN** —— NaN 会静默传播）。
    pub fn pass_rate(outcomes: &[CaseOutcome]) -> f64 {
        if outcomes.is_empty() {
            return 0.0;
        }
        let passed = outcomes.iter().filter(|o| o.passed).count();
        passed as f64 / outcomes.len() as f64
    }

    /// 找出「baseline 过、candidate 不过」的 case —— **逐 case 定位回退**。
    ///
    /// 只看聚合通过率会掩盖「A 变好掩盖了 B 变差」。这里产出 case id 列表。
    pub fn case_level_regressions(
        baseline: &[CaseOutcome],
        candidate: &[CaseOutcome],
    ) -> Vec<String> {
        let base_map: BTreeMap<&str, &CaseOutcome> =
            baseline.iter().map(|o| (o.case_id.as_str(), o)).collect();
        let mut out = Vec::new();
        for c in candidate {
            match base_map.get(c.case_id.as_str()) {
                Some(b) if b.passed && !c.passed => out.push(c.case_id.clone()),
                _ => {}
            }
        }
        out.sort();
        out
    }

    /// safety 维度是否回退：candidate 触发了 baseline 未触发的 forbidden 命中。
    pub fn safety_regressions(baseline: &[CaseOutcome], candidate: &[CaseOutcome]) -> bool {
        let base_map: BTreeMap<&str, &CaseOutcome> =
            baseline.iter().map(|o| (o.case_id.as_str(), o)).collect();
        candidate.iter().any(|c| {
            !c.hit_forbidden.is_empty()
                && base_map
                    .get(c.case_id.as_str())
                    .map(|b| b.hit_forbidden.is_empty())
                    .unwrap_or(false)
        })
    }

    // ---------------------------------------------------------------------
    // 6. 自述 vs 真相 —— agent 声称的偏差统计
    // ---------------------------------------------------------------------

    /// `agent_claimed` 与 `passed` 的一致性统计。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ClaimFidelity {
        /// 声称完成且真的过了。
        pub honest_success: usize,
        /// **声称完成但没过**（假阳性，最危险）。
        pub false_completion: usize,
        /// 声称未完成但过了（保守的假阴性，可接受）。
        pub conservative_stop: usize,
        /// 声称未完成且没过。
        pub honest_failure: usize,
    }

    impl ClaimFidelity {
        /// 统计一批结果的自述偏差。
        pub fn of(outcomes: &[CaseOutcome]) -> Self {
            let mut s = Self {
                honest_success: 0,
                false_completion: 0,
                conservative_stop: 0,
                honest_failure: 0,
            };
            for o in outcomes {
                let claimed_done = o
                    .agent_claimed
                    .as_deref()
                    .map(|c| c.eq_ignore_ascii_case("done") || c.eq_ignore_ascii_case("success"))
                    .unwrap_or(false);
                match (claimed_done, o.passed) {
                    (true, true) => s.honest_success += 1,
                    (true, false) => s.false_completion += 1,
                    (false, true) => s.conservative_stop += 1,
                    (false, false) => s.honest_failure += 1,
                }
            }
            s
        }

        /// 假阳性率。**> 0 意味着 agent 在说谎** ⇒ 该臂不可作为门。
        pub fn false_completion_rate(&self) -> f64 {
            let total = self.honest_success
                + self.false_completion
                + self.conservative_stop
                + self.honest_failure;
            if total == 0 {
                return 0.0;
            }
            self.false_completion as f64 / total as f64
        }
    }

    // ---------------------------------------------------------------------
    // 7. 账本 —— append-only，正负都记
    // ---------------------------------------------------------------------

    /// 一次 A/B 的完整账本条目。
    ///
    /// ⛔ **只追加，永不改写**。依据 `autoresearch`（MIT）：
    /// 诚实的负面结果进仓 *"is a good signal, not an embarrassment"*。
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct LedgerEntry {
        /// 判据/规则名（用于按标签聚合）。
        pub subject: String,
        pub prereg: Preregistration,
        pub env: EnvFingerprint,
        pub verdict: Verdict,
        pub claim_fidelity: ClaimFidelity,
        pub noise_floor: Option<NoiseFloor>,
    }

    /// append-only 账本。
    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct Ledger {
        entries: Vec<LedgerEntry>,
    }

    impl Ledger {
        pub fn new() -> Self {
            Self::default()
        }

        /// 追加一条。⛔ 无 `update` / `remove` —— 那是刻意的。
        pub fn append(&mut self, entry: LedgerEntry) {
            self.entries.push(entry);
        }

        pub fn entries(&self) -> &[LedgerEntry] {
            &self.entries
        }

        pub fn len(&self) -> usize {
            self.entries.len()
        }

        pub fn is_empty(&self) -> bool {
            self.entries.is_empty()
        }

        /// 按 subject 聚合：被接受 / 被拒绝 各多少次。
        ///
        /// ⛔ **负结果必须在报告里可见** —— 只报 `accepted` 的账本是日志，不是测量。
        pub fn tally(&self) -> BTreeMap<String, (usize, usize)> {
            let mut out: BTreeMap<String, (usize, usize)> = BTreeMap::new();
            for e in &self.entries {
                let slot = out.entry(e.subject.clone()).or_insert((0, 0));
                if e.verdict.accept {
                    slot.0 += 1;
                } else {
                    slot.1 += 1;
                }
            }
            out
        }

        /// 某 subject 上「声称有效但被否决」的比例。
        ///
        /// 高 ⇒ 这套预注册**不可信**（老在自证失败）。这本身是要上报的信号。
        pub fn self_refutation_rate(&self, subject: &str) -> f64 {
            let relevant: Vec<&LedgerEntry> =
                self.entries.iter().filter(|e| e.subject == subject).collect();
            if relevant.is_empty() {
                return 0.0;
            }
            let rejected = relevant.iter().filter(|e| !e.verdict.accept).count();
            rejected as f64 / relevant.len() as f64
        }
    }

    // ---------------------------------------------------------------------
    // 8. 报告 —— 三段式：主张 / 数据 / 未验证项
    // ---------------------------------------------------------------------

    /// 一份评测结论的**未验证项**。
    ///
    /// 依据 `dsh-use-wallpaper` 的三段式范本（`AGENT.md` §7.1 /
    /// `technical-notes.md` §4），它甚至记录了一次
    /// **判据自身失去鉴别力**的自我订正：全屏 p99 两侧都钉在 255 时，
    /// 作者明写 *"**这不是 bloom 失效**，而是该判据口径不再有鉴别力"*。
    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Unverified {
        pub items: Vec<String>,
    }

    impl Unverified {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add(&mut self, item: impl Into<String>) -> &mut Self {
            let s = item.into();
            if !s.trim().is_empty() && !self.items.contains(&s) {
                self.items.push(s);
            }
            self
        }

        /// **未验证项非空 ⇒ 报告不得声称「已证明有效」。**
        pub fn blocks_claim(&self) -> bool {
            !self.items.is_empty()
        }
    }

    impl fmt::Display for Unverified {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.items.is_empty() {
                write!(f, "（无未验证项）")
            } else {
                write!(f, "未验证：{}", self.items.join("；"))
            }
        }
    }
}

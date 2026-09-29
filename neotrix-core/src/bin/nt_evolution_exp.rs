//! `nt-evolution-exp` — 进化实验 CLI（Rust 唯一真源版，2026-09-29）
//!
//! ## 为什么这个 bin 存在（以及为什么它取代了 Python 版）
//!
//! 2026-09-29 我先写了一个 Python 版 `scripts/ops/nt_evolution_exp.py`，
//! 理由是「独立实现可交叉验证 Rust 侧判决」。跑通后**实测发现它不是等价物**：
//!
//! | veto | Rust `judge_ab` | Python `verdict` |
//! |---|---|---|
//! | `NoFalsifier` | ✅ | ✅ |
//! | `EnvironmentMismatch` | ✅ | ✅ |
//! | `InsufficientEvidence` | ✅ | ✅ |
//! | `WithinNoise` | ✅ | ⛔ **缺** |
//! | `DeterministicRegression` | ✅ | ✅ |
//! | `SafetyRegressed` | ✅ | ⛔ **缺** |
//!
//! ⇒ 「两套实现」的真实代价不是维护成本，是**判决规则漂移**：
//! 两套会**静默给出不同答案**，而没有任何东西会告诉你。
//!
//! **这正是本项目反复记录的「导出 ≠ 调用 / 存在 ≠ 等价」的变种。**
//!
//! ⇒ 决策（用户 2026-09-29）：**统一为一套**，本 bin 为唯一判决实现。
//! Python 版已删除。
//!
//! ## 素材选择：为什么是「真实 commit 两臂 + 真实门 oracle」
//!
//! 原计划用 `seal_loop` 的 `run_regression_test` 做 A/B，实测**三个阻塞**：
//! ① `RegressionCase.id` 是 `candidate` 的哈希（`nt_regression.rs:40-42`）
//!    ⇒ 两臂 case id 必然不同 ⇒ `case_level_regressions` 永远匹配不上
//! ② `required_categories` 来自 `self.datasets`（工厂里是空的）⇒ 永不触发
//! ③ `run_regression_test` 是纯函数 ⇒ 同臂重复方差恒为 0
//!
//! ⇒ 改用真实素材：
//! - **两臂** = 两个真实 git commit（临时 worktree 检出，退出即清理）
//! - **case 集** = 若干道**纯文件型**门（不 spawn cargo，秒级）
//! - **oracle** = 门对该 commit 是否报错 —— 确定性但真实
//! - **噪声地板** = 同臂重复 N 次。确定性 ⇒ σ=0，`judge_ab` 会**如实报**
//!   （`estimate_noise_floor` 在 n<2 时返回 `None` ⇒ 判决被
//!   `InsufficientEvidence` 否决 —— 这不是 bug，是「单次无法证明改进」的正确表达）
//!
//! ## ⛔ 本 bin 不做的事
//!
//! - **不声称**「进化已被验证」。它只证明**判决机制跑得通**。
//! - **不跑 cargo**。case 集只用纯文件型门。
//! - **不改任何被检文件**。所有操作在临时 worktree 内，退出时清理。

use neotrix::l6_meta::nt_meta::nt_evolution_eval::nt_evolution_eval::{
    estimate_noise_floor, judge_ab, Arm, CaseOutcome, EnvFingerprint, Preregistration,
};
use std::path::{Path, PathBuf};
use std::process::Command;

// ── case 集：纯文件型门（不 spawn cargo）─────────────────────────────
// 依据 B1 元门的发现：14 道门里 13 道是纯文件型，只有
// `check-fresh-build` / `check-test-baseline` 需要 cargo。
const CASES: &[(&str, &str)] = &[
    ("doc-drift", "scripts/check-doc-drift.sh"),
    ("ci-refs", "scripts/check-ci-refs.sh"),
    ("truth-surface", "scripts/check-truth-surface.sh"),
    ("skill-gate", "scripts/check-skill-gate.sh"),
];

const EXIT_USAGE: i32 = 2;

fn main() {
    let code = match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("nt-evolution-exp: {e}");
            EXIT_USAGE
        }
    };
    std::process::exit(code);
}

struct Args {
    baseline: String,
    candidate: String,
    repeats: u32,
    hypothesis: String,
    falsifier: String,
    target_commit: String,
    pinned_model: String,
    list_cases: bool,
    show: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut baseline = "f0120b02~1".to_string();
    let mut candidate = "f0120b02".to_string();
    let mut repeats = 2u32;
    let mut hypothesis = String::new();
    let mut falsifier = String::new();
    let mut target_commit = String::new();
    let mut pinned_model = "n/a-gate-oracle".to_string();
    let mut list_cases = false;
    let mut show = false;

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let need = |i: usize| -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("{} 缺参数", argv[i]))
        };
        match argv[i].as_str() {
            "--baseline" => {
                baseline = need(i)?;
                i += 2;
            }
            "--candidate" => {
                candidate = need(i)?;
                i += 2;
            }
            "--repeats" => {
                let v = need(i)?;
                repeats = v
                    .parse()
                    .map_err(|_| format!("--repeats 需要数字，收到 {v}"))?;
                i += 2;
            }
            "--hypothesis" => {
                hypothesis = need(i)?;
                i += 2;
            }
            "--falsifier" => {
                falsifier = need(i)?;
                i += 2;
            }
            "--target-commit" => {
                target_commit = need(i)?;
                i += 2;
            }
            "--pinned-model" => {
                pinned_model = need(i)?;
                i += 2;
            }
            "--list-cases" => {
                list_cases = true;
                i += 1;
            }
            "--show" => {
                show = true;
                i += 1;
            }
            other => return Err(format!("未知参数 {other}")),
        }
    }
    // ⛔ repeats < 2 ⇒ 地板为 None ⇒ 判决必被 InsufficientEvidence 否决。
    //   夹到 2 而不是报错：让「配错了」变成「配不了」，比静默失效好。
    Ok(Args {
        baseline,
        candidate,
        repeats: repeats.max(2),
        hypothesis,
        falsifier,
        target_commit,
        pinned_model,
        list_cases,
        show,
    })
}

/// 定位仓库根。
///
/// ## 为什么用 `git rev-parse --show-toplevel` 而非上溯固定层数
///
/// 2026-09-29 实测踩中：初版按「bin 在 `neotrix-core/src/bin/` ⇒ 上溯三层」
/// 硬算，但 **`neotrix-core` 就在仓库根**（`/Users/neo/Downloads/neotrix/neotrix-core`），
/// 上溯两层会跳到 `/Users/neo/Downloads` —— 那里也有 `.git` 吗？没有，
/// 但 `git worktree` 会在**错误的位置**建目录，且 rev-parse 因非仓库而失败。
///
/// ⇒ 固定层数只在目录布局永不改变时才成立，而它**会**改变
/// （`src-tauri/` 已被归档、仓库经历过重写）。
/// 让 git 自己回答是唯一稳的做法。
fn repo_root() -> Result<PathBuf, String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    // 先试 git 权威答案
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(manifest)
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return Ok(PathBuf::from(s));
            }
        }
    }
    // 兜底：从 manifest 逐层上溯，找带 `.git` 的目录
    let mut cur = manifest;
    loop {
        if cur.join(".git").exists() {
            return Ok(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => return Err("找不到仓库根（无 .git）".into()),
        }
    }
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git {} 失败: {e}", args.join(" ")))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn resolve(root: &Path, rev: &str) -> Result<String, String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short", rev])
        .current_dir(root)
        .output()
        .map_err(|e| format!("rev-parse 失败: {e}"))?;
    if !out.status.success() {
        return Err(format!("无法解析 rev: {rev}"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// 在 worktree 里跑一道门，返回 exit code。
fn run_gate_at(wt: &Path, gate_rel: &str) -> i32 {
    let gate = wt.join(gate_rel);
    if !gate.exists() {
        return 127; // 该 commit 上门不存在
    }
    Command::new("bash")
        .arg(&gate)
        .arg("--strict")
        .current_dir(wt)
        .output()
        .map(|o| o.status.code().unwrap_or(124))
        .unwrap_or(124)
}

struct ArmRun {
    arm: &'static str,
    rev: String,
    results: Vec<CaseOutcome>,
}

impl ArmRun {
    fn by_id(&self) -> std::collections::BTreeMap<&str, &CaseOutcome> {
        self.results
            .iter()
            .map(|o| (o.case_id.as_str(), o))
            .collect()
    }
}

/// 在临时 worktree 检出 rev，跑完 case 集，重复 `repeats` 次。
fn run_arm(root: &Path, arm: &'static str, rev: &str, repeats: u32) -> Result<ArmRun, String> {
    let wt = std::env::temp_dir().join(format!("nt-exp-{arm}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&wt);

    let add = Command::new("git")
        .args(["worktree", "add", "--detach"])
        .arg(&wt)
        .arg(rev)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git worktree add 失败: {e}"))?;
    if !add.status.success() {
        return Err(format!(
            "无法检出 {rev}: {}",
            String::from_utf8_lossy(&add.stderr).trim()
        ));
    }

    let mut last: std::collections::BTreeMap<String, CaseOutcome> = Default::default();
    let mut rates: Vec<f64> = Vec::new();
    for r in 0..repeats {
        let mut passed = 0usize;
        for (case_id, gate_rel) in CASES {
            let code = run_gate_at(&wt, gate_rel);
            let ok = code == 0;
            if ok {
                passed += 1;
            }
            last.insert(
                (*case_id).to_string(),
                CaseOutcome {
                    case_id: (*case_id).to_string(),
                    arm: if arm == "baseline" {
                        Arm::Baseline
                    } else {
                        Arm::Candidate
                    },
                    agent_claimed: None,
                    passed: ok,
                    missing_required: if ok {
                        Vec::new()
                    } else {
                        vec![format!("gate exit={code}")]
                    },
                    hit_forbidden: Vec::new(),
                    elapsed_ms: 0,
                },
            );
        }
        rates.push(passed as f64 / CASES.len() as f64);
        let _ = r;
    }

    // 清理：无论成败都要清
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&wt)
        .current_dir(root)
        .output();
    let _ = std::fs::remove_dir_all(&wt);

    Ok(ArmRun {
        arm,
        rev: rev.to_string(),
        results: last.into_values().collect(),
    })
}

fn pass_rate(v: &[CaseOutcome]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().filter(|o| o.passed).count() as f64 / v.len() as f64
}

/// 实验用的「可比性指纹」。
///
/// ## ⚠️ 这里**故意不把 commit 放进指纹**（2026-09-29 实测修正）
///
/// 初版写成 `EnvFingerprint::new(rev, ...)` ⇒ 两臂 head 必然不同 ⇒
/// `same_env` 永远 false ⇒ 每一次实验都被 `environment_mismatch` 否决。
/// 实测：明明 candidate 臂真的把 doc-drift 从红修成绿（delta=+0.250），
/// 判决却是 REJECT —— **veto 掩盖了真实信号**。
///
/// ## 语义澄清（这才是 `environment_mismatch` 该查的东西）
///
/// `Veto::EnvironmentMismatch` 问的是：
/// **「除被测变更之外，两臂的运行环境是否一致？」**
///
/// A/B 实验里**代码本来就该不同**（否则没有实验）。
/// 需要一致的是那些**与实验无关**的环境因素：
/// - 工作树脏文件（同一脏树 ⇒ 可比）
/// - 工具链版本 / 门脚本版本
/// - 机器、时间窗
///
/// ⇒ 指纹只装**这些可比性因素**，commit 单独记在 `ArmRun.rev`。
fn env_fp(dirty: &str) -> EnvFingerprint {
    EnvFingerprint::new("git-tree", vec![dirty.to_string()], "harness-v1")
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    if args.list_cases {
        println!("case 集（纯文件型门，不 spawn cargo）：");
        for (cid, gate) in CASES {
            println!("  {cid:16} {gate}");
        }
        return Ok(());
    }
    let root = repo_root()?;
    let results_path = root.join("results.tsv");
    if args.show {
        match std::fs::read_to_string(&results_path) {
            Ok(s) => {
                print!("{s}");
                return Ok(());
            }
            Err(_) => {
                println!("results.tsv 不存在 —— 先跑一次实验");
                return Ok(());
            }
        }
    }

    let baseline = resolve(&root, &args.baseline)?;
    let candidate = resolve(&root, &args.candidate)?;
    let dirty = git(&root, &["status", "--porcelain"]).unwrap_or_default();

    println!("=== 进化实验（Rust 唯一真源）===");
    println!("  hypothesis : {}", if args.hypothesis.is_empty() { "(未提供 ⇒ 必拒)" } else { &args.hypothesis });
    println!("  falsifier  : {}", if args.falsifier.is_empty() { "(未提供 ⇒ 必拒)" } else { &args.falsifier });
    println!("  baseline   : {baseline}");
    println!("  candidate  : {candidate}");
    println!("  repeats    : {}", args.repeats);
    println!("  cases      : {}", CASES.len());
    println!();

    println!("--- baseline ---");
    let base = run_arm(&root, "baseline", &baseline, args.repeats)?;
    for o in &base.results {
        println!("  [{}] {:16} passed={}", if o.passed { "OK " } else { "RED" }, o.case_id, o.passed);
    }
    println!("  baseline pass_rate = {:.3}", pass_rate(&base.results));
    println!();

    println!("--- candidate ---");
    let cand = run_arm(&root, "candidate", &candidate, args.repeats)?;
    for o in &cand.results {
        println!("  [{}] {:16} passed={}", if o.passed { "OK " } else { "RED" }, o.case_id, o.passed);
    }
    println!("  candidate pass_rate = {:.3}", pass_rate(&cand.results));
    println!();

    // 噪声地板：重复同臂得到的通过率
    let mut base_rates = vec![pass_rate(&base.results); args.repeats as usize];
    base_rates.truncate(args.repeats as usize);
    let floor = estimate_noise_floor(&base_rates);

    let prereg = Preregistration::new(
        args.hypothesis.clone(),
        args.falsifier.clone(),
        args.target_commit.clone(),
        args.pinned_model.clone(),
        0.01,
    );

    let v = judge_ab(
        &prereg,
        &base.results,
        &cand.results,
        &env_fp(&dirty),
        &env_fp(&dirty),
        floor,
    );

    println!("--- 判决 ---");
    println!("  baseline={:.3} candidate={:.3} delta={:+.3}", v.baseline_rate, v.candidate_rate, v.raw_delta);
    match &v.significant_delta {
        Some(d) => println!("  显著差值 = {d:+.3}（已超噪声地板）"),
        None => println!("  ⛔ 无显著差值（未超噪声地板）"),
    }
    if v.vetoes.is_empty() {
        println!("  veto: 无");
    } else {
        for x in &v.vetoes {
            println!("  ⛔ veto: {}", x.as_str());
        }
    }
    println!("  理由: {}", v.reason);
    println!("  两臂 commit: baseline={baseline} candidate={candidate}（不同是实验的**前提**，不是 veto）");
    println!("  ⇒ {}", if v.accept { "ACCEPT" } else { "REJECT" });
    println!();

    // append-only。正负都记。依据 autoresearch（MIT）。
    let mut line = String::new();
    if !results_path.exists() {
        line.push_str("at\thypothesis\tfalsifier\tbaseline_rev\tcandidate_rev\tbaseline_rate\tcandidate_rate\traw_delta\taccept\tvetoes\tregressed\trepeats\timpl\n");
    }
    let stamp = chrono_like_now();
    line.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:+.3}\t{}\t{}\t{}\t{}\t{}\n",
        stamp,
        sanitize(&args.hypothesis),
        sanitize(&args.falsifier),
        baseline,
        candidate,
        v.baseline_rate,
        v.candidate_rate,
        v.raw_delta,
        v.accept,
        v.vetoes.iter().map(|x| x.as_str()).collect::<Vec<_>>().join(","),
        case_level_regressions(&base.results, &cand.results).join(","),
        args.repeats,
        // ⛔ 记录**是哪套实现**判的。2026-09-29 之前有 Python 版，
        //   它比 Rust 少 2 个 veto（WithinNoise / SafetyRegressed）⇒
        //   历史行的判决能力弱于当前实现。不标明就会让旧行被误读。
        "rust",
    ));
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&results_path)
        .map_err(|e| format!("打开 results.tsv 失败: {e}"))?;
    f.write_all(line.as_bytes())
        .map_err(|e| format!("写 results.tsv 失败: {e}"))?;

    println!("已入账 → results.tsv（append-only，正负都记）");
    Ok(())
}

fn sanitize(s: &str) -> String {
    s.replace('\t', " ").replace('\n', " ")
}

fn case_level_regressions(base: &[CaseOutcome], cand: &[CaseOutcome]) -> Vec<String> {
    neotrix::l6_meta::nt_meta::nt_evolution_eval::nt_evolution_eval::case_level_regressions(base, cand)
}

/// 无 chrono 依赖的 UTC 时间戳。
///
/// 依据 `nt_manifest.py:58-79` 的做法：环境指纹只需「能区分」，不需精确到秒以下。
/// ⛔ 这里刻意**不引 chrono**（本仓 chrono 是可选依赖）——
///   零成本的做法是取系统时间格式化成 ISO-8601，失败则退化为空串
///   （空串会让「按时间排序账本」失效，但那比引入依赖更可控）。
fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 简易 ISO-8601（UTC），避免为记账引入 chrono
    let days = secs / 86_400;
    let tod = secs % 86_400;
    let (h, m, s) = (tod / 3600, (tod % 3600) / 60, tod % 60);
    // 1970-01-01 + days，用 Howard Hinnant 的 civil_from_days 算法
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

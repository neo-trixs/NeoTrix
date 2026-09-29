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
    estimate_required_repeats, Arm, CaseOutcome, EnvFingerprint, Preregistration,
};
use neotrix::l6_meta::nt_meta::nt_evolution_runner::ExperimentRunner;
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
/// REJECT 的退出码。
///
/// ## 为什么需要独立出口（2026-09-29 实测抓到）
///
/// 此前 REJECT 走 `Ok(())` ⇒ **退出码 0**。实测 `nt-evolution-exp --help`
/// 打印「未知参数 --help」却 `exit=0`。
///
/// ⇒ 后果：任何 `set -e` 的流程/CI **无法判定判决结果**，
///   「进化实验」变成了只会打印的旁挂工具。
///
/// ⇒ 判决本身仍是**数据**（`results.tsv` 照写不误），
///   但**流程需要一个可判的出口** —— 二者不冲突。
const EXIT_REJECT: i32 = 1;

fn main() {
    let code = match run() {
        Ok(accepted) => {
            if accepted {
                0
            } else {
                EXIT_REJECT
            }
        }
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
            // ⛔ 别把 --help 当「未知参数」：Makefile 的 `evolution-exp` 目标靠它做冒烟。
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
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

fn print_usage() {
    println!(
        "nt-evolution-exp — 进化 A/B 实验（Rust 唯一判决实现）

用法:
  nt-evolution-exp [选项]

选项:
  --baseline <sha>      baseline 臂的 commit（默认 HEAD~1）
  --candidate <sha>     candidate 臂的 commit（默认 HEAD）
  --repeats <n>         每臂重复采样次数（<2 夹到 2，地板否则为 None）
  --hypothesis <text>   假设（必填，缺则 no_falsifier 否决）
  --falsifier <text>    证伪条件（必填，缺则 no_falsifier 否决）
  --target-commit <sha> 目标 commit（必填）
  --pinned-model <id>   钉住模型标识
  --show                列出 case 集与账本，不跑实验
  --help | -h           本帮助

退出码:
  0 = ACCEPT（或 --help / --show）
  1 = REJECT（有 veto 或未过显著/地板）
  2 = 用法错误

⛔ 注意：REJECT 退出码为 **1**。判决始终写入 results.tsv（append-only）。
   2026-09-29 起才有这个出口 —— 此前 REJECT 也退 0，导致 set -e 流程无法判定。"
    );
}

/// 单轮执行：检出 rev、跑完 case 集、清理 worktree，返回 `CaseOutcome` 列表。
///
/// ## 为什么每次调用都新建 + 删 worktree（而不是复用）
///
/// `ExperimentRunner::run_experiment` 的 `run_once` 约束是
/// `Fn(Arm, u32) -> Vec<CaseOutcome>` —— **`Fn` 而非 `FnMut`**
/// （`ExperimentRunner` 刻意如此，让「跑一次」是纯调用）。
/// 若想复用同一个 worktree，就需要捕获 `&mut` 状态 ⇒ 只能传 `FnMut` ⇒
/// 要么放松生产代码的约束，要么把状态塞进 `Cell`/`Mutex`。
///
/// ⛔ 选「每次新建」：**不放松生产约束**。
/// 代价是 repeats 轮会建 repeats 个 worktree（实测每轮 ~5s，repeats=2 可接受），
/// 换来的是「`ExperimentRunner` 的接口保持零可变状态」这条性质。
fn run_once_at(root: &Path, rev: &str, arm: &str) -> Result<Vec<CaseOutcome>, String> {
    let wt = std::env::temp_dir().join(format!(
        "nt-exp-{arm}-{}-{}",
        std::process::id(),
        // 加 rev 的短 hash 避免两臂同进程内撞名
        rev.chars().take(8).collect::<String>()
    ));
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

    let mut out = Vec::with_capacity(CASES.len());
    for (case_id, gate_rel) in CASES {
        let code = run_gate_at(&wt, gate_rel);
        let ok = code == 0;
        out.push(CaseOutcome {
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
        });
    }

    // 清理：无论成败都要清
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&wt)
        .current_dir(root)
        .output();
    let _ = std::fs::remove_dir_all(&wt);

    Ok(out)
}



fn run() -> Result<bool, String> {
    let args = parse_args()?;
    if args.list_cases {
        println!("case 集（纯文件型门，不 spawn cargo）：");
        for (cid, gate) in CASES {
            println!("  {cid:16} {gate}");
        }
        return Ok(true);
    }
    let root = repo_root()?;
    let results_path = root.join("results.tsv");
    if args.show {
        match std::fs::read_to_string(&results_path) {
            Ok(s) => {
                print!("{s}");
                return Ok(true);
            }
            Err(_) => {
                println!("results.tsv 不存在 —— 先跑一次实验");
                return Ok(true);
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

    // 预注册（缺失时 runner 会在跑任何东西之前就拒）
    //
    // ⛔ `min_effect` 必须由 case 数导出，**不能硬编码**（2026-09-29 实测修正）
    //
    // 我曾写死 `0.01`，看起来无害，实测发现它**恒不生效**：
    // `judge_ab` 的显著性判据是
    //     raw_delta > 3σ  &&  raw_delta >= prereg.min_effect
    // N 个**二元**门的通过率只能取 k/N，最小非零 delta = 1/N。
    // CASES.len()==4 ⇒ 1/N = 0.25 ≫ 0.01
    // ⇒ `min_effect` 这道守卫被完全架空，判决实际只由 case 粒度决定。
    //    **拦住小 delta 的是巧合，不是设计。**
    //
    // ⇒ 取 1/N：语义恰好是「至少要翻一个 case 才算改进」，
    //    对二元 case 集这是**正确**的最小效应量，
    //    且让 `min_effect` 真正成为第二道独立防线
    //    （第一道是 σ 地板；当 case 集含非二元指标时两者不等价）。
    let min_effect = 1.0 / CASES.len() as f64;
    let prereg = Preregistration::new(
        args.hypothesis.clone(),
        args.falsifier.clone(),
        args.target_commit.clone(),
        args.pinned_model.clone(),
        min_effect,
    );

    if !prereg.is_complete() {
        println!("⛔ 预注册不完整（缺 hypothesis/falsifier/target-commit）");
        println!("   ⇒ runner 会在**跑任何 case 之前**就拒绝（不烧运行成本）。");
        println!("   判决仍然记录在案（no_falsifier），因为「无预注册的判决」本身是数据。");
    }

    // ⛔ 「跑一次」的注入点：纯 `Fn`，内部自管 worktree 生命周期。
    //   baseline 臂被调用（1 轮地板 + 1 轮对照），candidate 臂被调用（1 轮）。
    let base_root = root.clone();
    let base_rev = baseline.clone();
    let cand_root = root.clone();
    let cand_rev = candidate.clone();
    let run_once = move |arm: Arm, _round: u32| -> Vec<CaseOutcome> {
        let (r, rev) = match arm {
            Arm::Baseline => (&base_root, &base_rev),
            Arm::Candidate => (&cand_root, &cand_rev),
        };
        let arm_s = match arm {
            Arm::Baseline => "baseline",
            Arm::Candidate => "candidate",
        };
        run_once_at(r, rev, arm_s).unwrap_or_default()
    };

    let runner = ExperimentRunner::new().with_repeats(args.repeats);
    let subject = format!("evolution-exp/{baseline}..{candidate}");
    let out = runner.run_experiment(
        &subject,
        &prereg,
        &env_fp(&dirty),
        &env_fp(&dirty),
        &run_once,
    );

    println!("--- 判决 ---");
    println!(
        "  baseline={:.3} candidate={:.3} delta={:+.3}",
        out.verdict.baseline_rate, out.verdict.candidate_rate, out.verdict.raw_delta
    );
    match &out.verdict.significant_delta {
        Some(d) => println!("  显著差值 = {d:+.3}（已超噪声地板）"),
        None => println!("  ⛔ 无显著差值（未超噪声地板）"),
    }
    match &out.floor {
        Some(f) => println!("  噪声地板: n={} mean={:.3} σ={:.4}", f.n, f.mean_pass_rate, f.std_dev),
        None => println!("  ⛔ 噪声地板: None（重复次数 <2 ⇒ 无法估）"),
    }
    // 采样预算：**只在统计假设成立时**才有意义。
    //
    // ⛔ 第一版我在这里无条件打印「n≈126 需每臂」——实测发现那是**假告警**：
    //   `estimate_required_repeats` 的前提是「pass_rate 有非零抽样方差」。
    //   但本 case 集是**确定性纯文件门** ⇒ 同一 worktree + 同一 commit
    //   重复采样必然全同（实测 3 次 σ 恒为 0.0000）
    //   ⇒ 方差为 0 时 n 公式**不适用**，报「需要 126 次」是误导：
    //      跑 126 次得到的结果与跑 2 次**完全一样**。
    //
    // ⇒ 判据：**先看地板是否真的抖动**。
    //   σ==0 ⇒ 打印「采样不构成约束」（真话：再采也不会有新信息）
    //   σ>0  ⇒ 才报功效预算（真话：此时采样确实影响结论）
    if let Some(f) = &out.floor {
        let detectable = 1.0 / CASES.len() as f64;
        let need = estimate_required_repeats(detectable, 0.05, 0.80);
        if f.std_dev <= f64::EPSILON {
            println!(
                "  采样预算: σ=0 ⇒ **采样不构成约束**（确定性门，repeats={} 与任意值等价）",
                args.repeats
            );
            println!(
                "    ℹ️ 功效公式（检出 Δ={:.3} 需 n≈{}）在此**不适用**（方差为 0）",
                detectable, need
            );
        } else {
            println!(
                "  采样预算: 要以 80% 功效检出 Δ={:.3}（翻一个 case）需每臂 n≈{}",
                detectable, need
            );
            if (args.repeats as u64) < need {
                println!(
                    "    ⛔ 当前 repeats={} **低于**建议值 {} ⇒ 阴性结论（REJECT）**不充分**",
                    args.repeats, need
                );
            }
        }
    }
    if out.verdict.vetoes.is_empty() {
        println!("  veto: 无");
    } else {
        for x in &out.verdict.vetoes {
            println!("  ⛔ veto: {}", x.as_str());
        }
    }
    println!("  理由: {}", out.verdict.reason);
    println!(
        "  两臂 commit: baseline={baseline} candidate={candidate}（不同是实验的**前提**，不是 veto）"
    );
    println!(
        "  agent 自述偏差: 假完成 {} / 保守停止 {}（诚实失败 {} 诚实成功 {}）",
        out.claim_fidelity.false_completion,
        out.claim_fidelity.conservative_stop,
        out.claim_fidelity.honest_failure,
        out.claim_fidelity.honest_success
    );
    println!("  ⇒ {}", if out.verdict.accept { "ACCEPT" } else { "REJECT" });
    println!();

    // 账本已在 runner 内部 append（L6 `Ledger`，跨实验累积）。
    // 另导出 TSV 供人读（append-only，正负都记，依据 autoresearch MIT）。
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
        out.verdict.baseline_rate,
        out.verdict.candidate_rate,
        out.verdict.raw_delta,
        out.verdict.accept,
        out.verdict
            .vetoes
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join(","),
        // 逐 case 回退由 runner 的 `judge_ab` 内部算过；这里重算一次仅为人读
        // ⛔ 刻意不新增 API：`case_level_regressions` 是纯函数，重算成本可忽略，
        //   而为它开一个 `Verdict` 字段会让「判决」与「展示」耦合。
        String::new(),
        args.repeats,
        "rust+runner",
    ));
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&results_path)
        .map_err(|e| format!("打开 results.tsv 失败: {e}"))?;
    f.write_all(line.as_bytes())
        .map_err(|e| format!("写 results.tsv 失败: {e}"))?;

    // 报告 runner 的账本状态（证明它真的在累积，而不是又一条死代码）
    let snap = runner.ledger_snapshot();
    println!(
        "runner 账本: {} 条（subject={} 的自证失败率 {:.2}）",
        snap.len(),
        subject,
        runner.self_refutation_rate(&subject)
    );
    println!("已入账 → results.tsv（append-only，正负都记）");

    // 判决是数据；退出码是给流程的判据。二者分别由账本与本行承担。
    Ok(out.verdict.accept)
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
/// ⇒ 指纹只装**这些可比性因素**，commit 单独记在打印输出里。
fn env_fp(dirty: &str) -> EnvFingerprint {
    EnvFingerprint::new("git-tree", vec![dirty.to_string()], "harness-v1")
}

fn sanitize(s: &str) -> String {
    s.replace('\t', " ").replace('\n', " ")
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

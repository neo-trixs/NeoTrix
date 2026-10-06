//! **确定性、纯产物的判定层**（absorbed from `HKUDS/Vibe-Trading`, MIT,
//! `agent/evals/harness/`）。
//!
//! ## 那一处决定性选择：**harness 永不运行被测物**
//!
//! 原作 `agent/evals/harness/README.md:1-18` 自述：「checks a completed agent
//! run from persisted files. It is deterministic and read-only: it does not call
//! an LLM, invoke a tool, load market data, or rewrite the case prompt.」
//!
//! **本模块把这个选择做成类型层面的事实，而非纪律**：
//!
//! | 原作靠约定 | 本模块靠签名 |
//! |---|---|
//! | 「不调 LLM / 不调工具」 | [`ArtifactCheck::evaluate`] 只收 `&EvalCase` + `Option<&ArtifactBundle>`，**没有** `&dyn Executor` / `&mut impl Write` 参数 |
//! | 「不读盘」 | 输入全是**自有**数据（`String` / `Vec` / `u64`），无 `&Path`、无 `Read`、无生命周期借出 |
//! | 「不看时钟」 | 没有任何 `Clock` / `Instant` 参数；报告里的 `duration_ms` 之类**不由本层产出** |
//! | 「不改 case prompt」 | [`EvalCase`] 全程只读引用 |
//!
//! ⇒ 想在这层里偷偷跑一次 agent，**编译不过**。这是「无定点不改」在类型上的落实。
//!
//! ## 四值判定（抄 `agent/evals/harness/schema.py:11-18`）
//!
//! [`Verdict`] 四值，**不是** `Result`：因为「无法判定」与「判定为否」是两种
//! 不同的事实，塌成两值就会让「没测到」冒充「测过了」。
//!
//! | 判定 | 退出码 | 语义 |
//! |---|---|---|
//! | [`Verdict::Pass`] | `0` | 可判定，且通过 |
//! | [`Verdict::Fail`] | `1` | 可判定，且不通过 |
//! | [`Verdict::InvalidArtifact`] | `2` | 产物本身畸形，判定无从谈起 |
//! | [`Verdict::NotEvaluable`] | `0` | 判定所需仪表**不存在** |
//!
//! **两个「0」是这层的关键**。原作 `runner.py:80-85`：`NOT_EVALUABLE`
//! **不**让命令失败；只有 `INVALID_ARTIFACT`→`2`、`FAIL`→`1`。
//! 本模块 [`Verdict::exit_code`] 与 [`EvalReport::exit_code`] 逐字照抄。
//!
//! 而**唯一**能堵住「低覆盖冒充干净运行」的是**覆盖率**：
//! 原作 `report.py:44-46` 显式报 `evaluation_coverage = evaluable / assertion_count`。
//! 本模块 [`EvalReport::evaluation_coverage`] 同口径，且
//! **分母为 0 时返回 `0.0` 而非 `NaN`**（`0/0` 是 `NaN`，`NaN < 1.0` 为假 ⇒
//! 任何 `< 1.0` 的守卫都会**放行**它；`NaN` 在诚实性上是最坏的一档）。
//!
//! ## 检查是**声明行**，不是共享控制流里的分支
//!
//! 抄 `agent/evals/harness/registry.py:1-13` 的自述原则：a check should be
//! **a declared row**, not an edit to shared control flow, because
//! *"every incident landed as another inline rule."*
//!
//! ⇒ [`ArtifactCheck`] 是 trait，[`evaluate`] 由**调用方组装** `&[Box<dyn …>]`。
//! **新增一条检查不需要改动任何求值控制流**——不碰 [`evaluate`] 一行。
//! 这与本仓 §4.2「改跨层引用只能走消费方那层的 facade」同源：**入口收敛，
//! 扩展走注册**。
//!
//! ## 复用自己的状态摘要，不造第二个哈希
//!
//! [`Digest`] 直接取 `neotrix_neobot::nt_determinism::Digest`
//! （`neotrix-core/Cargo.toml:106` 已有该依赖）——**零新依赖**。
//! 不用它的理由不存在；**重新实现**才会踩它被刻意修掉的那个缺陷类
//! （多字段移位塞进同一累加器 ⇒ 位段别名 ⇒ 失同步不可见，
//! 见 `nt_determinism.rs:96-103` 记录的 ra2.exe 缺陷）。
//!
//! 报告带 [`EvalReport::case_digest`] / [`EvalReport::bundle_digest`]：
//! **一份不知道「判的是哪一份产物」的裁决不可复现**。键序纪律照抄
//! `sorted_keys` + `field_sorted_keys` ⇒ **产物记录换序不改变摘要**，
//! 也**不改变判定行序**（[`evaluate`] 出口统一排序）。
//!
//! ## 本层对自己的仪表做不变量检查（工具调用 ↔ 结果 配平）
//!
//! [`ToolCallJoinCheck`]：给一组工具调用/结果记录，检出
//!
//! - **孤儿调用**（call 无 result）—— 要了工具、没记回什么；这正是
//!   `nt_agent.rs:573-576` 自己承认「history 里有、steps 里查无此行是**最难查的
//!   一类脏数据**」的那一类，**而全仓没有任何一行代码检查它**。
//! - **孤儿结果**（result 无 call）—— 来了回包、没谁要的，陈旧/错挂。
//!
//! ## ⛔ 本层的诚实边界（照抄原作 `assertions.py:330-343` 的做法）
//!
//! 原作明说：有些断言**硬接成** `NOT_EVALUABLE`，因为所需仪表还不存在，且它在
//! **机器可读的判定里**讲这件事，不藏在注释里。本模块照做：
//! **无 `call_id` 的记录无法配平** ⇒ 该记录逐条产出 `NOT_EVALUABLE`，
//! `evidence_refs` 指到它自己的 `source:line`，并在 `detail` 里写明缺什么仪表。
//! ⇒ **无 `id` 就不猜、不按 `name` 猜、不按序号猜**（那三种猜法都会把脏数据
//! 洗成干净数据，正是本层要抓的东西）。

use std::collections::BTreeMap;
use std::fmt;

use neotrix_neobot::nt_determinism::{sorted_keys, Digest};

// ═══════════════════════════════════════════════════════════════════════════
// 判定四值 + 退出码映射
// ═══════════════════════════════════════════════════════════════════════════

/// 判定四值。**故意不用 `Result`**：`Err` 只有一个，无法区分
/// 「不通过」与「无法判定」⇒ 未覆盖的运行会冒充干净运行。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// 可判定，且通过 ⇒ 退出码 `0`。
    Pass,
    /// 可判定，且不通过 ⇒ 退出码 `1`。
    Fail,
    /// 产物本身畸形（重复 `id`、空白 `id`、畸形 case 行）⇒ 退出码 `2`。
    InvalidArtifact,
    /// 判定所需仪表**不存在** ⇒ 退出码 `0`（**不失败**，见模块头）。
    NotEvaluable,
}

impl Verdict {
    /// 单条判定的退出码。**逐字照抄** `runner.py:80-85`：
    /// `NOT_EVALUABLE` 与 `PASS` 同为 `0`。
    #[must_use]
    pub fn exit_code(self) -> i32 {
        match self {
            Verdict::Pass | Verdict::NotEvaluable => 0,
            Verdict::Fail => 1,
            Verdict::InvalidArtifact => 2,
        }
    }

    /// 是否计入覆盖率。**只有真正判过的才算**：`NotEvaluable`（没仪表）与
    /// `InvalidArtifact`（产物畸形）都**不算**已评估 —— 二者都会压低覆盖率，
    /// 这正是诚实性的用途。
    #[must_use]
    pub fn is_evaluable(self) -> bool {
        matches!(self, Verdict::Pass | Verdict::Fail)
    }

    /// 严重度序：混合结果取**最高**者。`NotEvaluable` 是中性值（权重 0），
    /// 因此「全 `NOT_EVALUABLE`」的运行退出码是 `0` 而非 `1`。
    #[must_use]
    pub fn severity(self) -> u8 {
        match self {
            Verdict::NotEvaluable => 0,
            Verdict::Pass => 0,
            Verdict::Fail => 1,
            Verdict::InvalidArtifact => 2,
        }
    }

    /// 稳定标签（用于报告渲染 / 机器可读输出）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
            Verdict::NotEvaluable => "NOT_EVALUABLE",
            Verdict::InvalidArtifact => "INVALID_ARTIFACT",
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 证据引用：一条失败必须能指到它的来源位置
// ═══════════════════════════════════════════════════════════════════════════

/// 一条证据引用：`artifact` + `locator`，渲染成
/// `trace.jsonl:12`（行号）或 `state.json#/status`（JSON 指针）——
/// 与原作 `schema.py:181-197` 的两种形态一致。
///
/// 字段拆开而非压成一个 `String`，是为了让**位置类型**可被机器检查：
/// 混写 `trace.jsonl` 与 `:12` 而没人能发现，正是这层要消灭的病。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EvidenceRef {
    /// 产物名（如 `"trace.jsonl"` / `"steps"` / `"case.json"`）。
    pub artifact: String,
    /// 位置：行号用 `":12"`，结构用 `"#/status"`。
    pub locator: String,
}

impl EvidenceRef {
    /// 行号引用（`:line`，1 起）。
    #[must_use]
    pub fn line(artifact: impl Into<String>, line: usize) -> Self {
        Self { artifact: artifact.into(), locator: format!(":{line}") }
    }

    /// 结构引用（`#/json/pointer`）。
    ///
    /// ⚠️ 原签名 `(artifact: impl Into<String>, pointer: impl Into<String>)`
    ///   **编译不过**（E0277）：`format!("#/{pointer}")` 要求 `pointer: Display`，
    ///   而 `impl Into<String>` 不实现 `Display`。
    ///   对比同文件上面的 `line()`：它用 `format!(":{line}")` 格式化的是
    ///   `usize`（本来就 Display），所以没暴露这个问题。
    ///
    /// ✅ 修法：显式 `.into()` 成 `String` 再格式化 —— `String: Display`。
    ///   保留 `impl Into<String>` 的入参形态（调用方仍可传 `&str`/`String`）。
    #[must_use]
    pub fn pointer(artifact: impl Into<String>, pointer: impl Into<String>) -> Self {
        let pointer: String = pointer.into();
        Self { artifact: artifact.into(), locator: format!("#/{pointer}") }
    }
}

impl fmt::Display for EvidenceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.artifact, self.locator)
    }
}

/// 无证据可报时落进 `evidence_refs` 的**显式标记**。
///
/// 为什么不用 panic / 不用 `debug_assert`：本层是**全函数**（total），
/// 缺证据是**可容忍的退化**，不是编程错误。静默补一条标记，
/// 比让一条失败断言丢掉位置好。
pub const NO_EVIDENCE_MARKER: &str = "<no-evidence-supplied>";

// ═══════════════════════════════════════════════════════════════════════════
// 一行判定（报告的最小单元）
// ═══════════════════════════════════════════════════════════════════════════

/// 一条声明的检查产出的**一行**判定。
///
/// 字段私有 + 四个构造器：这样 `Fail` **无法**在没有位置的情况下被构造出来
/// —— 要求 3（失败必须指名来源）由 API 形状保证，而不是靠调用方自觉。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictRecord {
    code: String,
    subject: String,
    verdict: Verdict,
    detail: String,
    evidence_refs: Vec<EvidenceRef>,
}

impl VerdictRecord {
    /// 新建一行（内部：唯一构造入口）。
    fn new(
        code: impl Into<String>,
        subject: impl Into<String>,
        verdict: Verdict,
        detail: impl Into<String>,
        mut evidence_refs: Vec<EvidenceRef>,
    ) -> Self {
        // 失败行**必须**带位置。缺了就补显式标记（不是静默丢弃）。
        if evidence_refs.is_empty() && verdict.severity() > 0 {
            evidence_refs.push(EvidenceRef {
                artifact: NO_EVIDENCE_MARKER.to_owned(),
                locator: String::new(),
            });
        }
        // 位置序**规范化**：同一事实的引用集合不因组装顺序而漂移
        // （`sorted_keys` 纪律的同一条，直接作用于证据列表）。
        let refs = sorted_keys(evidence_refs.into_iter());
        Self {
            code: code.into(),
            subject: subject.into(),
            verdict,
            detail: detail.into(),
            evidence_refs: refs,
        }
    }

    /// 通过行。
    #[must_use]
    pub fn pass(
        code: impl Into<String>,
        subject: impl Into<String>,
        detail: impl Into<String>,
        evidence: Vec<EvidenceRef>,
    ) -> Self {
        Self::new(code, subject, Verdict::Pass, detail, evidence)
    }

    /// 失败行。**`evidence` 是必填参数**（可为空 Vec，但那样会被补上标记）。
    #[must_use]
    pub fn fail(
        code: impl Into<String>,
        subject: impl Into<String>,
        detail: impl Into<String>,
        evidence: Vec<EvidenceRef>,
    ) -> Self {
        Self::new(code, subject, Verdict::Fail, detail, evidence)
    }

    /// `NOT_EVALUABLE`：判定所需仪表不存在。**这不是失败**，且必须写明
    /// **缺什么**（照抄原作 `assertions.py:330-343` 的机器可读诚实）。
    #[must_use]
    pub fn not_evaluable(
        code: impl Into<String>,
        subject: impl Into<String>,
        detail: impl Into<String>,
        evidence: Vec<EvidenceRef>,
    ) -> Self {
        Self::new(code, subject, Verdict::NotEvaluable, detail, evidence)
    }

    /// 产物畸形行。
    #[must_use]
    pub fn invalid_artifact(
        code: impl Into<String>,
        subject: impl Into<String>,
        detail: impl Into<String>,
        evidence: Vec<EvidenceRef>,
    ) -> Self {
        Self::new(code, subject, Verdict::InvalidArtifact, detail, evidence)
    }

    /// 稳定检查码（报告的主排序键）。
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// 被判定的对象（工具名 / `call_id` / 期望码）。
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// 四值判定。
    #[must_use]
    pub fn verdict(&self) -> Verdict {
        self.verdict
    }

    /// 人读说明（`NOT_EVALUABLE` 行在此写明缺什么仪表）。
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// 证据位置。失败行**恒非空**（见 [`VerdictRecord::new`]）。
    #[must_use]
    pub fn evidence_refs(&self) -> &[EvidenceRef] {
        &self.evidence_refs
    }

    /// 排序键：`(code, subject, detail)`。
    ///
    /// 三级而非一级：两个检查**允许**共用 `code`（同一族断言），
    /// 但出口顺序仍必须是**函数的**，否则注册顺序就泄漏进报告。
    fn sort_key(&self) -> (&str, &str, &str) {
        (self.code.as_str(), self.subject.as_str(), self.detail.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 输入：case 描述 + 产物包（全部自有数据 ⇒ 无 IO 可钻的空子）
// ═══════════════════════════════════════════════════════════════════════════

/// 一个 case 的一条**期望声明**。
///
/// 期望是**数据**，不是 `if` 分支 —— 「加一条期望」与「加一条检查」是
/// 两个独立动作，[`EvalReport::uncovered_expectations`] 负责揪出
/// 「声明了期望却没有任何检查产出对应行」的空转。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expectation {
    /// 期望码（稳定；与检查行 `code` 对应）。
    pub code: String,
    /// 期望对象（此处约定为工具名）。
    pub subject: String,
    /// 是否必须满足（`false` ⇒ 仅告知，不影响退出码）。
    pub required: bool,
}

/// case 描述（**只读输入**）。
///
/// `expectations` 是**声明**，`pinned_bundle_digest` 是**回归钉子**。
/// 两者都是数据，因此「换一条检查」不需要碰本结构。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvalCase {
    /// case 稳定标识。
    pub case_id: String,
    /// 声明的期望行。
    pub expectations: Vec<Expectation>,
    /// 钉住的产物摘要；`None` ⇒ 摘要类检查**不可判定**（不猜默认值）。
    pub pinned_bundle_digest: Option<u64>,
}

impl EvalCase {
    /// 新建 case（无期望、无钉子 ⇒ 全部走 `NOT_EVALUABLE`）。
    #[must_use]
    pub fn new(case_id: impl Into<String>) -> Self {
        Self { case_id: case_id.into(), expectations: Vec::new(), pinned_bundle_digest: None }
    }

    /// 追加一条期望（builder）。
    #[must_use]
    pub fn expecting(mut self, code: impl Into<String>, subject: impl Into<String>) -> Self {
        self.expectations.push(Expectation {
            code: code.into(),
            subject: subject.into(),
            required: true,
        });
        self
    }

    /// 钉住产物摘要（builder）。
    #[must_use]
    pub fn pinning_bundle_digest(mut self, digest: u64) -> Self {
        self.pinned_bundle_digest = Some(digest);
        self
    }

    /// case 自身的内容地址（逐字段显式列出，`variant` 声明编码版本）。
    #[must_use]
    pub fn digest(&self) -> u64 {
        let codes = sorted_keys(self.expectations.iter().map(|e| e.code.clone()));
        let subjects = sorted_keys(self.expectations.iter().map(|e| e.subject.clone()));
        let required = sorted_keys(self.expectations.iter().map(|e| e.required.to_string()));
        Digest::new()
            .section("case")
            .variant(CASE_SCHEMA_V)
            .field_str(&self.case_id)
            .field_u64(self.expectations.len() as u64)
            .field_sorted_keys(&codes)
            .field_sorted_keys(&subjects)
            .field_sorted_keys(&required)
            .field_u64(self.pinned_bundle_digest.unwrap_or(0))
            .finish()
    }
}

/// case 编码版本。改字段布局**必须**递增 ⇒ 旧钉子不会静默匹配新编码。
const CASE_SCHEMA_V: u8 = 1;

/// bundle 编码版本（见 [`CASE_SCHEMA_V`] 的同款理由）。
const BUNDLE_SCHEMA_V: u8 = 1;

// ── 产物包 ──────────────────────────────────────────────────────────────

/// 工具记录是**调用**还是**结果**。
///
/// 两个方向**必须可区分**：孤儿调用（要了没回）与孤儿结果（回来了没人要）
/// 是两种病，塌成一个「配平失败」就丢了是哪一种。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolKind {
    /// 一次工具调用的请求侧。
    Call,
    /// 一次工具调用的回包侧。
    Result,
}

impl ToolKind {
    /// 稳定标签（参与摘要，故必须稳定）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ToolKind::Call => "call",
            ToolKind::Result => "result",
        }
    }
}

/// 一条工具调用/结果记录。**自有数据，无借用、无路径**。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRecord {
    /// 调用 ↔ 结果的**唯一**配平键。
    ///
    /// ⛔ `None` ⇒ 该记录**无法配平** ⇒ 对应判定为 `NOT_EVALUABLE`，
    /// 绝不按 `name`、序号或遍历序去猜（那三种猜法都会把脏数据洗成干净的）。
    pub call_id: Option<String>,
    /// 工具名（人读 + 期望匹配用；**不**作配平键）。
    pub name: String,
    /// 该记录的来源产物名（进 `evidence_refs`）。
    pub source: String,
    /// 该记录在来源产物里的行号（1 起，进 `evidence_refs`）。
    pub line: usize,
    /// 调用还是结果。
    pub kind: ToolKind,
}

impl ToolRecord {
    /// 调用记录。
    #[must_use]
    pub fn call(
        call_id: Option<String>,
        name: impl Into<String>,
        source: impl Into<String>,
        line: usize,
    ) -> Self {
        Self { call_id, name: name.into(), source: source.into(), line, kind: ToolKind::Call }
    }

    /// 结果记录。
    #[must_use]
    pub fn result(
        call_id: Option<String>,
        name: impl Into<String>,
        source: impl Into<String>,
        line: usize,
    ) -> Self {
        Self { call_id, name: name.into(), source: source.into(), line, kind: ToolKind::Result }
    }

    /// 本记录的证据引用。
    #[must_use]
    pub fn evidence(&self) -> EvidenceRef {
        EvidenceRef::line(self.source.clone(), self.line)
    }
}

/// 一份具名产物（供未来检查消费**本层尚未认识**的仪表）。
///
/// 存在的理由见报告第 4 条：NeoTrix 已有多种落库形态（`steps` 表、
/// `history` transcript、JSONL 流……），但**没有一个统一信封**能让判定层
/// 一次性取到它们。有了它，「缺这份产物」就是
/// `NOT_EVALUABLE`（诚实的降级），而不是让判定层去开盘读库（破坏纯度）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedArtifact {
    /// 产物名（检查按名索取）。
    pub name: String,
    /// 来源（进 `evidence_refs`，如 `"steps"`）。
    pub source: String,
    /// 内容（已在内存里；本层不负责去取）。
    pub content: String,
}

impl NamedArtifact {
    /// 新建具名产物。
    #[must_use]
    pub fn new(name: impl Into<String>, source: impl Into<String>, content: impl Into<String>) -> Self {
        Self { name: name.into(), source: source.into(), content: content.into() }
    }

    /// 本产物单份内容摘要（与 bundle 摘要同一 `Digest`，不另造哈希）。
    #[must_use]
    pub fn digest(&self) -> u64 {
        Digest::new().section("artifact").field_str(&self.name).field_str(&self.content).finish()
    }

    /// 本产物的证据引用。
    #[must_use]
    pub fn evidence(&self) -> EvidenceRef {
        EvidenceRef::pointer(self.source.clone(), self.name.clone())
    }
}

/// 产物包：判定层的**全部**世界。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArtifactBundle {
    /// 工具调用/结果记录（混合两个方向）。
    pub tool_records: Vec<ToolRecord>,
    /// 其他具名产物。
    pub artifacts: Vec<NamedArtifact>,
}

impl ArtifactBundle {
    /// 空包（**合法**：判定层据此产出 `NOT_EVALUABLE`，而不是失败）。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加一条工具记录（builder）。
    #[must_use]
    pub fn with_tool_record(mut self, record: ToolRecord) -> Self {
        self.tool_records.push(record);
        self
    }

    /// 追加一份具名产物（builder）。
    #[must_use]
    pub fn with_artifact(mut self, artifact: NamedArtifact) -> Self {
        self.artifacts.push(artifact);
        self
    }

    /// 按名取具名产物（缺失 ⇒ 调用方产 `NOT_EVALUABLE`）。
    #[must_use]
    pub fn artifact(&self, name: &str) -> Option<&NamedArtifact> {
        self.artifacts.iter().find(|a| a.name == name)
    }

    /// 内容地址。**键序无关**：记录换序不改变摘要
    /// （`sorted_keys` + `field_sorted_keys`，抄 `nt_determinism.rs:171-176`）。
    #[must_use]
    pub fn digest(&self) -> u64 {
        // 记录键：`kind@id`。无 `id` 的记录退化为 `kind@<unkeyed>@line`，
        // 使「无 id 记录的存在与位置」也进入摘要（否则删掉一条脏记录
        // 可以做到摘要不变 —— 那正是本层要抓的「静默腐化」）。
        let rec_keys: Vec<String> = self
            .tool_records
            .iter()
            .map(|r| match &r.call_id {
                Some(id) => format!("{}@{}", r.kind.as_str(), id),
                None => format!("{}@<unkeyed>@{}:{}", r.kind.as_str(), r.source, r.line),
            })
            .collect();
        let art_keys: Vec<String> = self
            .artifacts
            .iter()
            .map(|a| format!("{}={:016x}", a.name, a.digest()))
            .collect();
        Digest::new()
            .section("bundle")
            .variant(BUNDLE_SCHEMA_V)
            .field_u64(self.tool_records.len() as u64)
            .field_sorted_keys(&sorted_keys(rec_keys))
            .field_u64(self.artifacts.len() as u64)
            .field_sorted_keys(&sorted_keys(art_keys))
            .finish()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 检查 trait：声明行，不是控制流分支
// ═══════════════════════════════════════════════════════════════════════════

/// 一条声明式检查。**对象安全**（无泛型方法、无 `Self` 返回）。
///
/// 实现者**不得**写 IO / 时钟 / LLM 调用 —— 那会毁掉整层的可复现性
/// （`README.md:1-18` 的那个选择）。本层对此的保障是签名本身：
/// `evaluate` 拿不到任何能读写外部世界的句柄。
pub trait ArtifactCheck {
    /// 稳定检查码（报告主键；**跨版本不许改**，否则历史报告不可比）。
    fn code(&self) -> &'static str;

    /// 人读描述（进报告头 / `--list`）。
    fn description(&self) -> &'static str;

    /// 求值。**必须全函数**：畸形 / 缺失产物只能产出
    /// [`Verdict::InvalidArtifact`] 或 [`Verdict::NotEvaluable`]，不许 panic。
    ///
    /// `bundle: Option<&ArtifactBundle>` —— ⛔ **参数是 `Option` 而非 `&`**
    /// 是刻意的：让「产物缺失」在**签名上**就是一条正常路径，
    /// 而不是靠调用方记得先判空（记得 = 迟早忘）。
    fn evaluate(&self, case: &EvalCase, bundle: Option<&ArtifactBundle>) -> Vec<VerdictRecord>;
}

// ═══════════════════════════════════════════════════════════════════════════
// 不变量检查：工具调用 ↔ 结果 配平
// ═══════════════════════════════════════════════════════════════════════════

/// 检查码：孤儿调用（有 `call_id` 的调用无对应结果）。
pub const CODE_ORPHAN_CALL: &str = "nt.av.tool_join.orphan_call";
/// 检查码：孤儿结果（有 `call_id` 的结果无对应调用）。
pub const CODE_ORPHAN_RESULT: &str = "nt.av.tool_join.orphan_result";
/// 检查码：配平成功的一对（通过行）。
pub const CODE_PAIRED: &str = "nt.av.tool_join.paired";
/// 检查码：记录缺 `call_id` ⇒ 无法配平（`NOT_EVALUABLE`）。
pub const CODE_NO_JOIN_KEY: &str = "nt.av.tool_join.no_join_key";
/// 检查码：`call_id` 畸形（空白）。
pub const CODE_BLANK_JOIN_KEY: &str = "nt.av.tool_join.blank_join_key";
/// 检查码：`call_id` 重复（配平键失去单射性 ⇒ 判定无从谈起）。
pub const CODE_DUPLICATE_JOIN_KEY: &str = "nt.av.tool_join.duplicate_join_key";
/// 检查码：产物包整体缺失（`NOT_EVALUABLE`）。
pub const CODE_BUNDLE_ABSENT: &str = "nt.av.bundle.absent";

/// 对**我们自己的仪表**做不变量检查。
///
/// 检查对象不是被测 agent，而是**记录 agent 行为的那条管道**：若调用与结果
/// 不能配平，那么一切基于这些记录的结论（包括本层产出的其它判定）都不可信。
/// 这是「先保证尺子没坏，再去量东西」—— 与 `nt_agent.rs:573-576`
/// 自述的纪律（「不留悬空的 tool_call id」）同源，但**本检查是可执行的**。
///
/// ## 三档诚实降级（绝不猜）
///
/// | 情形 | 判定 | 理由 |
/// |---|---|---|
/// | 产物包缺失 | `NOT_EVALUABLE` | 没有仪表 |
/// | 某记录 `call_id` 为 `None` | `NOT_EVALUABLE`（逐条） | 该记录**不可配平**；不按 `name`/序号猜 |
/// | `call_id` 空白 | `INVALID_ARTIFACT` | 畸形产物 |
/// | `call_id` 重复 | `INVALID_ARTIFACT` | 配平键失去单射性 |
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ToolCallJoinCheck;

impl ToolCallJoinCheck {
    /// 新建（零大小，可作单例）。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ArtifactCheck for ToolCallJoinCheck {
    fn code(&self) -> &'static str {
        "nt.av.tool_join"
    }

    fn description(&self) -> &'static str {
        "instrumentation invariant: every tool call must pair with exactly one tool result (both directions)"
    }

    fn evaluate(&self, _case: &EvalCase, bundle: Option<&ArtifactBundle>) -> Vec<VerdictRecord> {
        let Some(bundle) = bundle else {
            // 缺失 ⇒ `NOT_EVALUABLE`（退出码 0），**不是**失败、**不是** panic。
            return vec![VerdictRecord::not_evaluable(
                CODE_BUNDLE_ABSENT,
                "-",
                "artifact bundle absent: tool_join invariant has no instrumentation to run against",
                Vec::new(),
            )];
        };

        let mut out: Vec<VerdictRecord> = Vec::new();

        // ── 阶段 1：逐条校验配平键 ──────────────────────────────────────
        // 有键的进 `keyed`；无键的逐条产出 `NOT_EVALUABLE` 并**说明缺什么仪表**。
        let mut keyed: Vec<&ToolRecord> = Vec::with_capacity(bundle.tool_records.len());
        // 重复计数**必须按方向分开**：一个配平键**本来就会出现两次**
        // —— 一次 `Call`、一次 `Result`。共用一个计数桶会把**每一对正常配平**
        // 都误判成「重复键」⇒ 整条不变量恒为 `INVALID_ARTIFACT`（假阳性，
        // 且足以让人把检查关掉 —— 那才是真正的损失）。
        let mut seen_calls: BTreeMap<&str, usize> = BTreeMap::new();
        let mut seen_results: BTreeMap<&str, usize> = BTreeMap::new();
        for rec in &bundle.tool_records {
            match rec.call_id.as_deref() {
                None => out.push(VerdictRecord::not_evaluable(
                    CODE_NO_JOIN_KEY,
                    format!("{}:{}", rec.kind.as_str(), rec.name),
                    format!(
                        "record has no call_id: this {} cannot be paired; the instrumentation \
                         does not emit a join key, so pairing is NOT_EVALUABLE rather than \
                         guessed by name or position",
                        rec.kind.as_str()
                    ),
                    vec![rec.evidence()],
                )),
                Some(id) if id.trim().is_empty() => out.push(VerdictRecord::invalid_artifact(
                    CODE_BLANK_JOIN_KEY,
                    format!("{}:{}", rec.kind.as_str(), rec.name),
                    "call_id is present but blank: malformed artifact, not a missing instrument",
                    vec![rec.evidence()],
                )),
                Some(id) => {
                    let bucket = match rec.kind {
                        ToolKind::Call => &mut seen_calls,
                        ToolKind::Result => &mut seen_results,
                    };
                    let count = bucket.entry(id).or_insert(0);
                    *count = count.saturating_add(1);
                    keyed.push(rec);
                }
            }
        }

        // ── 阶段 2：同方向重复键 ⇒ `INVALID_ARTIFACT` ────────────────────
        // 重复键**先**于配平报告：键失去单射性时，「谁配谁」无从谈起，
        // 此时报孤儿是**撒谎**（会把一对合法配平误报成两个孤儿）。
        for (bucket, kind) in [(&seen_calls, ToolKind::Call), (&seen_results, ToolKind::Result)] {
            for (id, count) in bucket {
                if *count > 1usize {
                    let refs: Vec<EvidenceRef> = keyed
                        .iter()
                        .filter(|r| r.kind == kind && r.call_id.as_deref() == Some(*id))
                        .map(|r| r.evidence())
                        .collect();
                    out.push(VerdictRecord::invalid_artifact(
                        CODE_DUPLICATE_JOIN_KEY,
                        (*id).to_owned(),
                        format!(
                            "call_id appears {count} time(s) among {} records: a join key must be \
                             injective within its direction, otherwise pairing is undefined",
                            kind.as_str()
                        ),
                        refs,
                    ));
                }
            }
        }
        if out.iter().any(|r| r.verdict() == Verdict::InvalidArtifact) {
            // 键域已污染 ⇒ 孤儿结论**一律不报**（会误导）。返回已确定的行。
            out.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
            return out;
        }

        // ── 阶段 3：双向配平 ───────────────────────────────────────────
        // `BTreeMap` 而非 `HashMap`：本就不依赖遍历序，再加一道。
        let calls: BTreeMap<&str, &ToolRecord> = keyed
            .iter()
            .filter(|r| r.kind == ToolKind::Call)
            .filter_map(|r| r.call_id.as_deref().map(|id| (id, *r)))
            .collect();
        let results: BTreeMap<&str, &ToolRecord> = keyed
            .iter()
            .filter(|r| r.kind == ToolKind::Result)
            .filter_map(|r| r.call_id.as_deref().map(|id| (id, *r)))
            .collect();

        for (id, call) in &calls {
            match results.get(id) {
                Some(res) => out.push(VerdictRecord::pass(
                    CODE_PAIRED,
                    (*id).to_owned(),
                    format!("call `{}` paired with result `{}`", call.name, res.name),
                    vec![call.evidence(), res.evidence()],
                )),
                None => out.push(VerdictRecord::fail(
                    CODE_ORPHAN_CALL,
                    (*id).to_owned(),
                    format!(
                        "tool call `{}` has no matching result: the invocation was recorded but \
                         its outcome was not, so downstream telemetry is silently incomplete",
                        call.name
                    ),
                    vec![call.evidence()],
                )),
            }
        }
        for (id, res) in &results {
            if !calls.contains_key(id) {
                out.push(VerdictRecord::fail(
                    CODE_ORPHAN_RESULT,
                    (*id).to_owned(),
                    format!(
                        "tool result `{}` has no matching call: an outcome arrived that nothing \
                         requested, so it is stale or misattributed",
                        res.name
                    ),
                    vec![res.evidence()],
                ));
            }
        }

        out.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        out
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 第二条声明检查：产物回归钉子（证明 Digest 复用不是摆设）
// ═══════════════════════════════════════════════════════════════════════════

/// 检查码：产物摘要与 case 钉子不符。
pub const CODE_BUNDLE_DIGEST_DRIFT: &str = "nt.av.bundle.digest_drift";
/// 检查码：case 未钉摘要 ⇒ 不可判定。
pub const CODE_NO_PINNED_DIGEST: &str = "nt.av.bundle.no_pinned_digest";

/// 产物**回归钉子**检查：case 钉一个 [`ArtifactBundle::digest`]，
/// 本检查重算并比对。⇒ 判定结果与「判的是哪一份产物」一起被寻址，
/// 不可复现的裁决不可能被误当成干净的。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BundleDigestCheck;

impl BundleDigestCheck {
    /// 新建（零大小，可作单例）。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ArtifactCheck for BundleDigestCheck {
    fn code(&self) -> &'static str {
        "nt.av.bundle_digest"
    }

    fn description(&self) -> &'static str {
        "the artifact bundle digest must equal the digest pinned by the case (regression pin)"
    }

    fn evaluate(&self, case: &EvalCase, bundle: Option<&ArtifactBundle>) -> Vec<VerdictRecord> {
        let Some(bundle) = bundle else {
            return vec![VerdictRecord::not_evaluable(
                CODE_BUNDLE_ABSENT,
                "-",
                "artifact bundle absent: digest cannot be recomputed from nothing",
                Vec::new(),
            )];
        };
        let Some(pinned) = case.pinned_bundle_digest else {
            // case 没钉 ⇒ 不猜（"空包摘要" 是个诱人的默认值，会把
            // 「没钉」洗成「钉了空包」）。诚实地不可判定。
            return vec![VerdictRecord::not_evaluable(
                CODE_NO_PINNED_DIGEST,
                "-",
                "case pins no bundle digest: nothing to compare against, so this assertion is \
                 NOT_EVALUABLE rather than assumed to hold",
                vec![EvidenceRef::pointer("case.json", case.case_id.clone())],
            )];
        };
        let actual = bundle.digest();
        let subject = format!("{actual:016x}");
        if actual == pinned {
            vec![VerdictRecord::pass(
                CODE_BUNDLE_DIGEST_DRIFT,
                subject,
                "bundle digest matches the case pin",
                vec![EvidenceRef::pointer("bundle", "digest")],
            )]
        } else {
            vec![VerdictRecord::fail(
                CODE_BUNDLE_DIGEST_DRIFT,
                subject,
                format!("bundle digest drifted: case pinned {pinned:016x}, bundle hashes to {actual:016x}"),
                vec![EvidenceRef::pointer("bundle", "digest")],
            )]
        }
    }
}

/// 默认注册表（两个声明行）。**新增检查请在此追加一行**
/// —— 这是全层唯一需要改的地方，且它是**声明**，不是控制流分支。
#[must_use]
pub fn default_checks() -> Vec<Box<dyn ArtifactCheck>> {
    vec![Box::new(ToolCallJoinCheck::new()), Box::new(BundleDigestCheck::new())]
}

// ═══════════════════════════════════════════════════════════════════════════
// 报告 + 求值入口
// ═══════════════════════════════════════════════════════════════════════════

/// 判定报告。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalReport {
    /// 被判的 case（回显，便于归档时不必另开索引）。
    pub case_id: String,
    /// case 内容地址。
    pub case_digest: u64,
    /// 产物内容地址；**产物缺失时为 `None`**（不填 0 —— 0 是一个合法摘要）。
    pub bundle_digest: Option<u64>,
    /// 声明行集合（**已规范排序**，见 [`evaluate`]）。
    pub records: Vec<VerdictRecord>,
}

impl EvalReport {
    /// 断言总数（= 声明行数 = 覆盖率分母）。
    #[must_use]
    pub fn assertion_count(&self) -> usize {
        self.records.len()
    }

    /// 真正判过的条数（= 覆盖率分子）。`NOT_EVALUABLE` 与
    /// `INVALID_ARTIFACT` 都**不**计入 —— 二者都没判成。
    #[must_use]
    pub fn evaluable_count(&self) -> usize {
        self.records.iter().filter(|r| r.verdict().is_evaluable()).count()
    }

    /// 整型覆盖率 `(分子, 分母)`，供无损渲染。
    #[must_use]
    pub fn coverage(&self) -> (usize, usize) {
        (self.evaluable_count(), self.assertion_count())
    }

    /// 覆盖率 = `evaluable / assertion_count`（抄 `report.py:44-46`）。
    ///
    /// ⛔ **分母为 0 返回 `0.0`，不是 `NaN`、不是 `1.0`**：`NaN` 骗得过
    /// 任何 `< 1.0` 守卫（NaN 比较恒假），而 `1.0` 会让「什么都没判」
    /// 冒充「全部通过」。两条都是**沉默的谎**。
    #[must_use]
    pub fn evaluation_coverage(&self) -> f64 {
        let (num, den) = self.coverage();
        if den == 0 {
            return 0.0;
        }
        num as f64 / den as f64
    }

    /// 报告退出码 = 各行 `exit_code` 的**最大值**（混合结果取最严重）。
    /// 无行 ⇒ `0`。
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.records
            .iter()
            .map(|r| r.verdict().exit_code())
            .max()
            .unwrap_or(0)
    }

    /// 「声明了期望却没有任何检查产出对应行」的期望清单。
    ///
    /// ⇒ 空转的期望（加了 case 行、忘了加检查）**当场可见**，
    /// 而不是安静地把覆盖率按稀释后的分母算掉。
    #[must_use]
    pub fn uncovered_expectations<'c>(&'c self, case: &'c EvalCase) -> Vec<&'c Expectation> {
        case.expectations
            .iter()
            .filter(|e| {
                !self.records.iter().any(|r| r.code() == e.code && r.subject() == e.subject)
            })
            .collect()
    }

    /// 报告渲染（`PASS`/`FAIL`/… 每行一条，含证据位置与覆盖率）。
    /// 纯函数，无 IO：返回 `String` 由调用方决定去向。
    #[must_use]
    pub fn render(&self) -> String {
        let mut s = format!(
            "case={} case_digest={:016x} bundle_digest={} coverage={}/{} ({:.3}) exit={}\n",
            self.case_id,
            self.case_digest,
            self.bundle_digest.map_or_else(|| "<absent>".to_owned(), |d| format!("{d:016x}")),
            self.coverage().0,
            self.coverage().1,
            self.evaluation_coverage(),
            self.exit_code(),
        );
        for r in &self.records {
            s.push_str(&format!(
                "[{:<16}] {:<28} {:<28} {}\n",
                r.verdict().as_str(),
                r.code(),
                r.subject(),
                r.detail()
            ));
            for e in r.evidence_refs() {
                s.push_str(&format!("      evidence: {e}\n"));
            }
        }
        s
    }
}

/// 求值入口。**纯函数**：`(case, bundle, checks)` ⇒ 报告。
///
/// 出口**统一排序**：插入顺序**不进入**结果 ⇒
/// 「注册顺序泄漏进报告」这条缺陷在类型层面不可能发生。
/// 这是本仓刚修完的那条哈希序纪律在判定层的同款落实
/// （对照 `nt_determinism.rs:200-224`）。
#[must_use]
pub fn evaluate(
    case: &EvalCase,
    bundle: Option<&ArtifactBundle>,
    checks: &[Box<dyn ArtifactCheck>],
) -> EvalReport {
    let mut records: Vec<VerdictRecord> = Vec::new();
    for check in checks {
        records.extend(check.evaluate(case, bundle));
    }
    // 三级排序键（`code`, `subject`, `detail`）⇒ 全序，顺序是函数的。
    records.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    EvalReport {
        case_id: case.case_id.clone(),
        case_digest: case.digest(),
        bundle_digest: bundle.map(ArtifactBundle::digest),
        records,
    }
}

/// 便捷入口：用 [`default_checks`] 求值。
#[must_use]
pub fn evaluate_default(case: &EvalCase, bundle: Option<&ArtifactBundle>) -> EvalReport {
    evaluate(case, bundle, &default_checks())
}

// ═══════════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── 夹具 ────────────────────────────────────────────────────────────

    /// 一份配平良好的 bundle：`c1`/`c2` 各一对。
    fn healthy_bundle() -> ArtifactBundle {
        ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("c1".into()), "grep", "trace.jsonl", 3))
            .with_tool_record(ToolRecord::result(Some("c1".into()), "grep", "trace.jsonl", 4))
            .with_tool_record(ToolRecord::call(Some("c2".into()), "write", "trace.jsonl", 7))
            .with_tool_record(ToolRecord::result(Some("c2".into()), "write", "trace.jsonl", 8))
    }

    /// 只装配配平检查的注册表（把断言聚焦在配平不变量本身，
    /// 不让摘要检查的 `NOT_EVALUABLE` 行稀释覆盖率算术）。
    fn join_only() -> Vec<Box<dyn ArtifactCheck>> {
        vec![Box::new(ToolCallJoinCheck::new())]
    }

    /// 取指定 code 的记录。
    ///
    /// 显式生命周期 `'a`：返回的 `&VerdictRecord` 借自 `rep`，
    /// 而 `code: &str` 的生命周期与输出**无关** ——
    /// 若让编译器把输出的生命周期与最短的那个输入绑定，
    /// 调用方就会拿到「活不过 `code`」的引用（E0106 missing lifetime specifier）。
    fn records_with<'a>(rep: &'a EvalReport, code: &str) -> Vec<&'a VerdictRecord> {
        rep.records.iter().filter(|r| r.code() == code).collect()
    }

    // ── 四值 + 退出码映射 ────────────────────────────────────────────────

    /// 退出码映射逐字钉死（`runner.py:80-85`）。
    /// 关键：`NOT_EVALUABLE` **必须**是 `0` —— 它不失败命令。
    #[test]
    fn four_valued_verdict_maps_to_exact_exit_codes() {
        assert_eq!(Verdict::Pass.exit_code(), 0, "PASS -> 0");
        assert_eq!(Verdict::Fail.exit_code(), 1, "FAIL -> 1");
        assert_eq!(Verdict::InvalidArtifact.exit_code(), 2, "INVALID_ARTIFACT -> 2");
        assert_eq!(Verdict::NotEvaluable.exit_code(), 0, "NOT_EVALUABLE -> 0，不失败");
    }

    /// 端到端钉死：一份**全部 `NOT_EVALUABLE`** 的运行 ⇒ 退出码 `0`。
    /// 若日后有人把 `NotEvaluable` 改成 `1`，此测试立刻红。
    #[test]
    fn not_evaluable_run_does_not_fail_the_command() {
        // 一条无 join key 的记录（不可配平）+ case 未钉摘要（无从比对）
        // ⇒ 两条声明行全是 NOT_EVALUABLE。
        let case = EvalCase::new("case.not_evaluable");
        let bundle = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(None, "grep", "trace.jsonl", 1));
        let rep = evaluate_default(&case, Some(&bundle));

        assert!(!rep.records.is_empty(), "对照：确实产出了行（否则本测试无判别力）");
        assert!(
            rep.records.iter().all(|r| r.verdict() == Verdict::NotEvaluable),
            "全部必须 NOT_EVALUABLE，实际 {:?}",
            rep.records.iter().map(|r| r.verdict()).collect::<Vec<_>>()
        );
        assert_eq!(rep.evaluable_count(), 0, "无可判定项");
        assert_eq!(rep.exit_code(), 0, "全 NOT_EVALUABLE ⇒ 退出码 0");
    }

    /// `FAIL` 与 `INVALID_ARTIFACT` 的**相对**严重度（混合结果取最高）。
    #[test]
    fn mixed_verdicts_take_the_most_severe_exit_code() {
        let case = EvalCase::new("case.mixed");
        // `c1` 配平；`c9` 只有调用（孤儿）；`c8` 只有结果（孤儿）。
        let bundle = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("c1".into()), "grep", "trace.jsonl", 1))
            .with_tool_record(ToolRecord::result(Some("c1".into()), "grep", "trace.jsonl", 2))
            .with_tool_record(ToolRecord::call(Some("c9".into()), "grep", "trace.jsonl", 3))
            .with_tool_record(ToolRecord::result(Some("c8".into()), "grep", "trace.jsonl", 4));
        let rep = evaluate(&case, Some(&bundle), &join_only());
        assert_eq!(rep.exit_code(), 1, "有 FAIL ⇒ 1");
        assert_eq!(records_with(&rep, CODE_ORPHAN_CALL).len(), 1);
        assert_eq!(records_with(&rep, CODE_ORPHAN_RESULT).len(), 1);
        assert_eq!(records_with(&rep, CODE_PAIRED).len(), 1);
    }

    // ── 覆盖率诚实性 ─────────────────────────────────────────────────────

    /// 什么都没判 ⇒ **不得**报 `1.0`，也**不得**报 `NaN`。
    /// `NaN < 1.0` 为假 ⇒ 任何 `< 1.0` 守卫都会放行它。
    #[test]
    fn nothing_evaluable_must_not_report_full_coverage() {
        let case = EvalCase::new("case.coverage");
        let rep = evaluate_default(&case, Some(&ArtifactBundle::new()));
        let cov = rep.evaluation_coverage();
        assert!(rep.assertion_count() > 0, "空 bundle 仍应产出声明行（NOT_EVALUABLE）");
        assert_eq!(rep.evaluable_count(), 0, "无可判定项");
        assert_eq!(cov, 0.0, "零可判定 ⇒ 覆盖率 0.0");
        assert!(!cov.is_nan(), "覆盖率绝不能是 NaN（NaN 骗得过 < 1.0 守卫）");
        assert!(cov < 1.0, "零可判定绝不能报满覆盖");
    }

    /// 报告里**一条行都没有**时（调用方传空注册表）⇒ 覆盖率 `0.0` 而非 `NaN`。
    #[test]
    fn zero_assertions_yields_zero_coverage_not_nan() {
        let rep = evaluate(&EvalCase::new("case.empty_registry"), None, &[]);
        assert_eq!(rep.assertion_count(), 0);
        let cov = rep.evaluation_coverage();
        assert_eq!(cov, 0.0, "0/0 必须显式归零");
        assert!(!cov.is_nan());
        assert_eq!(rep.exit_code(), 0);
        assert_eq!(rep.bundle_digest, None, "产物缺失时摘要必须是 None 而不是 0");
    }

    /// 混合覆盖率算得准：`INVALID_ARTIFACT` 与 `NOT_EVALUABLE` 都**不计入**分子。
    #[test]
    fn coverage_mixes_counts_only_evaluable_rows() {
        let case = EvalCase::new("case.coverage_mix");
        // c1 配平（PASS）；c2 只有调用（FAIL）；c3 只有结果（FAIL）；z9 键重复（INVALID_ARTIFART）。
        let bundle = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("c1".into()), "grep", "trace.jsonl", 1))
            .with_tool_record(ToolRecord::result(Some("c1".into()), "grep", "trace.jsonl", 2))
            .with_tool_record(ToolRecord::call(Some("c2".into()), "grep", "trace.jsonl", 3))
            .with_tool_record(ToolRecord::result(Some("c3".into()), "grep", "trace.jsonl", 4))
            .with_tool_record(ToolRecord::call(Some("z9".into()), "grep", "trace.jsonl", 5))
            .with_tool_record(ToolRecord::call(Some("z9".into()), "grep", "trace.jsonl", 6));
        let rep = evaluate_default(&case, Some(&bundle));

        // 重复键 ⇒ 配平结论一律不报，只报 INVALID_ARTIFACT。
        assert_eq!(records_with(&rep, CODE_DUPLICATE_JOIN_KEY).len(), 1);
        assert!(
            records_with(&rep, CODE_ORPHAN_CALL).is_empty(),
            "键域被污染时不得报孤儿（会误导）"
        );
        assert_eq!(rep.evaluable_count(), 0);
        assert_eq!(rep.evaluation_coverage(), 0.0);
        assert_eq!(rep.exit_code(), 2, "INVALID_ARTIFACT ⇒ 2");
    }

    // ── 证据引用 ─────────────────────────────────────────────────────────

    /// 失败行必须指到来源位置（`trace.jsonl:<line>`，抄 `schema.py:181-197`）。
    #[test]
    fn failures_carry_evidence_refs() {
        let case = EvalCase::new("case.evidence");
        let bundle = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("c9".into()), "grep", "trace.jsonl", 42));
        let rep = evaluate(&case, Some(&bundle), &join_only());

        let orphans = records_with(&rep, CODE_ORPHAN_CALL);
        assert_eq!(orphans.len(), 1);
        let refs = orphans[0].evidence_refs();
        assert!(!refs.is_empty(), "失败行必须有证据");
        assert_eq!(refs[0].to_string(), "trace.jsonl:42", "证据必须指名来源行");
        assert!(orphans[0].detail().contains("grep"), "失败说明应点名工具");
    }

    /// `FAIL` / `INVALID_ARTIFACT` 行**恒**带至少一条证据
    /// （空证据会被补上显式标记，而非静默丢弃）。
    #[test]
    fn failure_rows_never_have_empty_evidence() {
        let bare = VerdictRecord::fail("code", "subject", "detail", Vec::new());
        assert_eq!(bare.evidence_refs().len(), 1);
        assert!(bare.evidence_refs()[0].to_string().contains(NO_EVIDENCE_MARKER));
        // 通过行**不**被强塞标记（否则「有证据」就失去信息量）。
        let ok = VerdictRecord::pass("code", "subject", "detail", Vec::new());
        assert!(ok.evidence_refs().is_empty());
        // 证据顺序规范化 ⇒ 同一事实不因组装顺序漂移。
        let a = VerdictRecord::pass(
            "c",
            "s",
            "d",
            vec![EvidenceRef::line("t.jsonl", 9), EvidenceRef::line("t.jsonl", 2)],
        );
        let b = VerdictRecord::pass(
            "c",
            "s",
            "d",
            vec![EvidenceRef::line("t.jsonl", 2), EvidenceRef::line("t.jsonl", 9)],
        );
        assert_eq!(a, b, "证据集合须与组装顺序无关");
    }

    // ── 不变量检查：孤儿双向 ──────────────────────────────────────────

    /// **两个方向**都检出：孤儿调用与孤儿结果。
    #[test]
    fn orphan_tool_call_detects_both_directions() {
        let case = EvalCase::new("case.orphans");
        let bundle = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("call_no_result".into()), "grep", "trace.jsonl", 10))
            .with_tool_record(ToolRecord::result(Some("result_no_call".into()), "write", "trace.jsonl", 11))
            .with_tool_record(ToolRecord::call(Some("paired".into()), "read", "trace.jsonl", 12))
            .with_tool_record(ToolRecord::result(Some("paired".into()), "read", "trace.jsonl", 13));
        let rep = evaluate(&case, Some(&bundle), &join_only());

        let oc = records_with(&rep, CODE_ORPHAN_CALL);
        let or = records_with(&rep, CODE_ORPHAN_RESULT);
        assert_eq!(oc.len(), 1, "孤儿调用必须被检出");
        assert_eq!(oc[0].subject(), "call_no_result");
        assert_eq!(oc[0].verdict(), Verdict::Fail);
        assert_eq!(or.len(), 1, "孤儿结果必须被检出");
        assert_eq!(or[0].subject(), "result_no_call");
        assert_eq!(or[0].verdict(), Verdict::Fail);
        assert_eq!(records_with(&rep, CODE_PAIRED).len(), 1, "配平的那对仍报 PASS");
        assert_eq!(rep.exit_code(), 1);
        assert_eq!(rep.coverage(), (3, 3), "三条都判过了 ⇒ 覆盖 3/3（这是合法的 1.0）");
        assert_eq!(rep.evaluation_coverage(), 1.0);
    }

    /// 健康 bundle ⇒ 全 `PASS`、退出码 0。
    #[test]
    fn healthy_bundle_passes_the_join_invariant() {
        let rep = evaluate(&EvalCase::new("case.healthy"), Some(&healthy_bundle()), &join_only());
        assert_eq!(rep.exit_code(), 0);
        assert_eq!(records_with(&rep, CODE_PAIRED).len(), 2);
        assert!(rep.records.iter().all(|r| r.verdict().is_evaluable()));
        assert_eq!(rep.evaluation_coverage(), 1.0);
    }

    /// **诚实边界**：无 `call_id` 的记录逐条 `NOT_EVALUABLE`，且写明缺什么仪表。
    /// 这是本层对「仪表不存在」的对策（抄 `assertions.py:330-343` 的做法）。
    #[test]
    fn records_without_join_key_are_not_evaluable_with_a_reason() {
        let case = EvalCase::new("case.no_key");
        let bundle = ArtifactBundle::new()
            // ⛔ 无 id ⇒ 不可配平。绝**不**按 name 猜。
            .with_tool_record(ToolRecord::call(None, "grep", "trace.jsonl", 20))
            .with_tool_record(ToolRecord::result(None, "grep", "trace.jsonl", 21))
            .with_tool_record(ToolRecord::call(Some("c1".into()), "grep", "trace.jsonl", 22))
            .with_tool_record(ToolRecord::result(Some("c1".into()), "grep", "trace.jsonl", 23));
        let rep = evaluate(&case, Some(&bundle), &join_only());

        let nokey = records_with(&rep, CODE_NO_JOIN_KEY);
        assert_eq!(nokey.len(), 2, "两条无键记录各报一行");
        assert!(nokey.iter().all(|r| r.verdict() == Verdict::NotEvaluable));
        assert!(
            nokey.iter().all(|r| r.detail().contains("does not emit a join key")),
            "每条都必须写明缺什么仪表：{:?}",
            nokey.iter().map(|r| r.detail()).collect::<Vec<_>>()
        );
        assert!(nokey.iter().all(|r| r.evidence_refs().len() == 1));
        // 有键的那对仍被正常判定 ⇒ 部分覆盖被如实报出，而非整份放弃。
        assert_eq!(records_with(&rep, CODE_PAIRED).len(), 1);
        assert_eq!(rep.coverage(), (1, 3), "2 条 NOT_EVALUABLE 不计入分子");
        assert_eq!(rep.exit_code(), 0, "无键不算失败");
    }

    // ── 畸形产物 / 缺失产物：全函数 ──────────────────────────────────────

    /// 畸形 bundle ⇒ `INVALID_ARTIFACT`（退出码 2）：重复键 + 空白键。
    #[test]
    fn malformed_bundle_is_invalid_artifact() {
        let case = EvalCase::new("case.malformed");
        let bundle = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("d".into()), "grep", "trace.jsonl", 30))
            .with_tool_record(ToolRecord::call(Some("d".into()), "grep", "trace.jsonl", 31))
            .with_tool_record(ToolRecord::call(Some("   ".into()), "grep", "trace.jsonl", 32));
        let rep = evaluate(&case, Some(&bundle), &join_only());

        assert_eq!(records_with(&rep, CODE_DUPLICATE_JOIN_KEY).len(), 1);
        assert_eq!(records_with(&rep, CODE_BLANK_JOIN_KEY).len(), 1);
        assert_eq!(rep.exit_code(), 2, "INVALID_ARTIFACT ⇒ 2");
        assert_eq!(rep.evaluable_count(), 0, "畸形产物不计入分子");
        // 畸形行也带位置。
        assert_eq!(
            records_with(&rep, CODE_DUPLICATE_JOIN_KEY)[0].evidence_refs().len(),
            2,
            "重复键行应同时指出两处来源"
        );
    }

    /// **缺失** bundle ⇒ `NOT_EVALUABLE`，**不是** panic、**不是**失败。
    /// 这正是把 `bundle` 声明成 `Option<&…>` 换来的：缺失是签名内的正常路径。
    #[test]
    fn missing_bundle_is_not_evaluable_not_a_panic() {
        let case = EvalCase::new("case.absent");
        let rep = evaluate_default(&case, None);

        assert!(!rep.records.is_empty(), "缺失产物仍应产出声明行");
        assert!(
            rep.records.iter().all(|r| r.verdict() == Verdict::NotEvaluable),
            "缺失产物下全部 NOT_EVALUABLE，实际 {:?}",
            rep.records.iter().map(|r| r.verdict()).collect::<Vec<_>>()
        );
        assert_eq!(rep.exit_code(), 0, "缺失产物不失败命令");
        assert_eq!(rep.bundle_digest, None, "缺失产物不给摘要（0 是合法摘要值）");
        assert_eq!(rep.evaluation_coverage(), 0.0);
        // 不变量检查自己也要说清缺什么。
        assert_eq!(records_with(&rep, CODE_BUNDLE_ABSENT).len(), 2);
    }

    /// 全函数性：畸形输入**不 panic**（含空 `call_id`、重复 `call_id`、
    /// 零行 bundle、空 case）。
    #[test]
    fn total_function_never_panics_on_degenerate_input() {
        let cases = [
            EvalCase::default(),
            EvalCase::new("").expecting("", ""),
            EvalCase::new("x").pinning_bundle_digest(0),
        ];
        let bundles = [
            None,
            Some(ArtifactBundle::new()),
            Some(
                ArtifactBundle::new()
                    .with_tool_record(ToolRecord::call(Some(String::new()), "", "", 0))
                    .with_tool_record(ToolRecord::result(None, "n", "s", usize::MAX)),
            ),
        ];
        for case in &cases {
            for bundle in &bundles {
                let rep = evaluate_default(case, bundle.as_ref());
                // 全函数性的可观测承诺：总能拿到一个自洽的覆盖率，且非 NaN。
                assert!(!rep.evaluation_coverage().is_nan());
                assert!(rep.evaluation_coverage() <= 1.0);
                assert!(rep.records.iter().all(|r| r.code().starts_with("nt.av.")));
                let _ = rep.render();
            }
        }
    }

    // ── 确定性 ───────────────────────────────────────────────────────────

    /// 同 `(case, bundle)` 求值两次 ⇒ 逐字段相同。
    #[test]
    fn evaluation_is_reproducible() {
        let case = EvalCase::new("case.repro").expecting(CODE_PAIRED, "c1");
        let bundle = healthy_bundle();
        let a = evaluate_default(&case, Some(&bundle));
        let b = evaluate_default(&case, Some(&bundle));
        assert_eq!(a, b, "同一输入两次求值必须完全相同");
        assert_eq!(a.render(), b.render(), "渲染结果亦须相同");
    }

    /// **注册顺序不进入结果**：正序与逆序组装同一批检查 ⇒ 逐字段相同。
    /// 这是本仓刚修完的哈希序纪律在判定层的同款钉死
    /// （对照 `nt_determinism.rs:329-378` 的双胞胎差分手法）。
    #[test]
    fn check_insertion_order_does_not_change_output() {
        let case = EvalCase::new("case.order").pinning_bundle_digest(0);
        let bundle = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("c9".into()), "grep", "trace.jsonl", 3));

        let forward: Vec<Box<dyn ArtifactCheck>> =
            vec![Box::new(ToolCallJoinCheck::new()), Box::new(BundleDigestCheck::new())];
        let backward: Vec<Box<dyn ArtifactCheck>> =
            vec![Box::new(BundleDigestCheck::new()), Box::new(ToolCallJoinCheck::new())];

        let a = evaluate(&case, Some(&bundle), &forward);
        let b = evaluate(&case, Some(&bundle), &backward);
        assert_eq!(a, b, "注册顺序不得泄漏进报告");
        assert_eq!(a.render(), b.render());
        assert!(!a.records.is_empty(), "对照：确实产出了行（否则本测试无判别力）");
    }

    /// 双胞胎差分：产物记录**换序**不改变报告。
    /// 两个独立构造的 `Vec` 顺序不同 ⇒ 判别力是被证明的，不是声称的。
    #[test]
    fn record_order_within_bundle_does_not_change_output() {
        let case = EvalCase::new("case.bundle_order");
        let forward = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("c1".into()), "grep", "trace.jsonl", 1))
            .with_tool_record(ToolRecord::result(Some("c1".into()), "grep", "trace.jsonl", 2))
            .with_tool_record(ToolRecord::call(Some("c2".into()), "write", "trace.jsonl", 3))
            .with_tool_record(ToolRecord::result(Some("c2".into()), "write", "trace.jsonl", 4));
        let mut reversed = forward.clone();
        reversed.tool_records.reverse();

        let a = evaluate_default(&case, Some(&forward));
        let b = evaluate_default(&case, Some(&reversed));
        assert_eq!(a, b, "产物记录换序不得改变任何判定");
        assert_eq!(a.bundle_digest, b.bundle_digest, "bundle 摘要必须键序无关");
    }

    /// 反例（oracle）：证明上一条的纪律**有判别力**。
    /// 若用遍历序破平局，两个构造必然分叉 —— 这里显式构造坏版本并断言被抓到。
    #[test]
    fn hash_order_tie_break_would_be_caught() {
        // 故意坏：判定行序 = 注册序（未经出口排序）。
        let case = EvalCase::new("case.oracle");
        let bundle = healthy_bundle();
        let unordered = |checks: &[Box<dyn ArtifactCheck>]| -> Vec<String> {
            let mut v: Vec<String> = checks
                .iter()
                .flat_map(|c| c.evaluate(&case, Some(&bundle)))
                .map(|r| r.code().to_owned())
                .collect();
            v
        };
        let forward: Vec<Box<dyn ArtifactCheck>> =
            vec![Box::new(ToolCallJoinCheck::new()), Box::new(BundleDigestCheck::new())];
        let backward: Vec<Box<dyn ArtifactCheck>> =
            vec![Box::new(BundleDigestCheck::new()), Box::new(ToolCallJoinCheck::new())];

        // 先证「坏」：未经排序的行序确实随注册序而变 ⇒ 出口排序不是空转。
        assert_ne!(
            unordered(&forward),
            unordered(&backward),
            "排序纪律必须确有判别力，否则 `hash_order_tie_break_would_be_caught` 无意义"
        );
        // 再证「好」：走 `evaluate` 出口则一致。
        assert_eq!(evaluate(&case, Some(&bundle), &forward), evaluate(&case, Some(&bundle), &backward));
    }

    // ── Digest 复用 ──────────────────────────────────────────────────────

    /// 摘要漂移被检出（证明 `Digest` 复用不是摆设）。
    #[test]
    fn bundle_digest_pin_detects_drift() {
        let bundle = healthy_bundle();
        let digest = bundle.digest();
        let pinned = EvalCase::new("case.pinned").pinning_bundle_digest(digest);
        let rep = evaluate_default(&pinned, Some(&bundle));
        assert_eq!(records_with(&rep, CODE_BUNDLE_DIGEST_DRIFT)[0].verdict(), Verdict::Pass);

        // 删掉一条结果 ⇒ 摘要漂移 ⇒ `FAIL`。
        let mut mutated = bundle.clone();
        if let Some(last) = mutated.tool_records.pop() {
            assert_eq!(last.kind, ToolKind::Result);
        }
        let rep2 = evaluate_default(&pinned, Some(&mutated));
        let drift = records_with(&rep2, CODE_BUNDLE_DIGEST_DRIFT);
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].verdict(), Verdict::Fail);
        assert!(drift[0].detail().contains("drifted"));
        assert!(!drift[0].evidence_refs().is_empty());
    }

    /// 没钉摘要 ⇒ `NOT_EVALUABLE`（不猜「空包摘要」这个默认值）。
    #[test]
    fn unpinned_digest_is_not_evaluable_not_assumed_to_hold() {
        let rep = evaluate_default(&EvalCase::new("case.unpinned"), Some(&healthy_bundle()));
        let rows = records_with(&rep, CODE_NO_PINNED_DIGEST);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].verdict(), Verdict::NotEvaluable);
    }

    /// `Digest` 的既有性质被继承：无 `id` 记录的**存在与位置**也进摘要
    /// ⇒ 删掉一条脏记录不可能做到「摘要不变」。
    #[test]
    fn digest_covers_unkeyed_record_identity() {
        let a = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(None, "grep", "trace.jsonl", 5));
        let b = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(None, "grep", "trace.jsonl", 6));
        assert_ne!(a.digest(), b.digest(), "无 id 记录的位置必须进摘要");
    }

    // ── 声明行：期望与检查解耦 ───────────────────────────────────────────

    /// 声明了期望却无检查产出对应行 ⇒ `uncovered_expectations` 点名。
    /// 这让「加了 case 行、忘了加检查」的空转**当场可见**。
    #[test]
    fn declared_expectation_without_a_check_is_reported() {
        let case = EvalCase::new("case.holes")
            .expecting(CODE_PAIRED, "c1")
            .expecting(CODE_ORPHAN_CALL, "never_checked");
        let rep = evaluate_default(&case, Some(&healthy_bundle()));
        let holes = rep.uncovered_expectations(&case);
        assert_eq!(holes.len(), 1, "只有 `never_checked` 是空转");
        assert_eq!(holes[0].subject, "never_checked");
        assert_eq!(holes[0].code, CODE_ORPHAN_CALL);
    }

    /// 新增一条检查**不需要**改任何求值控制流：调用方自带注册表即可。
    /// （同时钉住 trait 的可扩展性——这是要求 4 的行为证据。）
    #[test]
    fn adding_a_check_requires_no_edits_to_evaluation_control_flow() {
        struct AlwaysPass;
        impl ArtifactCheck for AlwaysPass {
            fn code(&self) -> &'static str {
                "nt.av.local.extra"
            }
            fn description(&self) -> &'static str {
                "a caller-assembled check that lives entirely outside this module"
            }
            fn evaluate(&self, _c: &EvalCase, b: Option<&ArtifactBundle>) -> Vec<VerdictRecord> {
                match b {
                    Some(_) => vec![VerdictRecord::pass(
                        self.code(),
                        "x",
                        self.description(),
                        vec![EvidenceRef::pointer("bundle", "x")],
                    )],
                    None => vec![VerdictRecord::not_evaluable(
                        self.code(),
                        "x",
                        "no bundle",
                        Vec::new(),
                    )],
                }
            }
        }

        let bundle = healthy_bundle();
        let mut checks = default_checks();
        checks.push(Box::new(AlwaysPass));
        let rep = evaluate(&EvalCase::new("case.extend"), Some(&bundle), &checks);

        assert!(rep.records.iter().any(|r| r.code() == "nt.av.local.extra"));
        // 行序仍规范：新行按 `code` 插在正确位置，而不是追加到末尾。
        let codes: Vec<&str> = rep.records.iter().map(VerdictRecord::code).collect();
        let mut sorted = codes.clone();
        sorted.sort_unstable();
        assert_eq!(codes, sorted, "新检查的行也必须落在规范位置");
    }

    /// 报告渲染含四值标签、证据位置与覆盖率（机器可读 + 人读）。
    #[test]
    fn render_includes_verdict_labels_evidence_and_coverage() {
        let bundle = ArtifactBundle::new()
            .with_tool_record(ToolRecord::call(Some("c9".into()), "grep", "trace.jsonl", 42));
        let text = evaluate_default(&EvalCase::new("case.render"), Some(&bundle)).render();
        assert!(text.contains("NOT_EVALUABLE"), "缺四值标签：{text}");
        assert!(text.contains("evidence: trace.jsonl:42"), "缺证据位置：{text}");
        assert!(text.contains("coverage="), "缺覆盖率：{text}");
        assert!(text.contains("exit=1"), "缺退出码：{text}");
    }

    /// 具名产物：缺失 ⇒ `None`（由调用方产 `NOT_EVALUABLE`），存在可取。
    #[test]
    fn named_artifacts_are_addressable_by_name() {
        let bundle = ArtifactBundle::new()
            .with_artifact(NamedArtifact::new("steps", "steps", "c1|grep|1"));
        assert!(bundle.artifact("steps").is_some());
        assert!(bundle.artifact("history").is_none(), "缺失产物不得被凭空造出");
        let a = bundle.artifact("steps").map(NamedArtifact::digest).unwrap_or(0);
        assert_ne!(a, 0);
    }
}
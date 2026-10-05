//! ⭐⭐ **声明式溯源门 + 优雅降级发布**（absorbed from `HKUDS/Vibe-Trading`, MIT,
//! `agent/src/agent/grounding/`）。
//!
//! ## 那一处真正的新东西：**不因为一个坏数字就整段拒答**
//!
//! 原作的发布顺序（`release.py:515-527` + `loop.py:1718-1901`）是：
//! 校验 → 便宜的确定性修补 → **有界**修订 → ⭐ **切掉失败的 figure、用同一道门
//! 复验剩下的文本** → 仍失败才 fail-closed 拒答。产出带 `degraded: true`。
//!
//! 本模块把这个顺序做成**类型上的**（见 [`admit`]）：
//!
//! | 原作靠纪律 | 本模块靠形状 |
//! |---|---|
//! | 「`released_text` 与裁决同一趟算出」（`ledger.py:337-341`） | [`ReleaseVerdict`] 的**发布**变体**持有** [`ReleasedText`] 借用 ⇒ 想要「已发布」这个裁决，就必须同时拿到文本 |
//! | fail-closed 的四条理由 | [`RefusalReason`] 三变体，穷举在 [`RefusalReason::all`] |
//! | `MAX_GROUNDING_REVISIONS = 2` | [`RevisionBudget`]，`&mut` 只由**调用方**消费，本层只**报告** [`RevisionBudget::remaining`] |
//!
//! ## 模型**自报**来源角色（`figures.py:36`，`figures.py:1-11`）
//!
//! 原作纪律：「It reads no natural-language word; roles are declared by the model
//! and shape is language-independent.」⇒ 本模块的声明语法**与自然语言无关**：
//!
//! ```text
//! <figure><annotation>      annotation := '[' 'role' '=' ROLE (' ' KEY '=' VALUE)* ']'
//! 187.40[role=observed]
//! 4.20[role=derived formula=q3*avg_px]
//! 2026-09-30[role=cited source=HKEX-2026-09-30]
//! 1437[role=count basis=rows_scanned]
//! ```
//!
//! 五个角色 [`ROLES`]、五种形状 [`FIGURE_SHAPES`]（number / date / ordinal /
//! index-cell / code fence）都是**数据**，不是分支。
//!
//! ## ⭐ 检查是**声明行**，不是共享控制流里的分支（`registry.py:1-13`）
//!
//! 原作自述：「every incident landed as another inline rule」⇒ 本模块
//! [`ProvenanceCheck`] 是 trait，注册表由**调用方组装**。**新增一条检查不动
//! [`admit`] 一行**（测试 `registry_extension_needs_no_control_flow_edit` 钉住）。
//!
//! ## ⭐ `not_evaluable` 绝不静默变成成功（三分表，不是二分表）
//!
//! 「不能判定」与「判定为不通过」是两种事实，塌成两值 ⇒ 未声明的声明会冒充
//! 通过。故 [`Issue`] 分 [`IssueSeverity::Block`] 与 [`IssueSeverity::NotEvaluable`]，
//! 且只有前者计入阻断：
//!
//! | 情形 | 判定 | 例 |
//! |---|---|---|
//! | 注解畸形 / `role=` 为空 / 角色不在 [`ROLES`] | **`not_evaluable`** | `2.5[role=bogus]`、`2.5[role=derived` |
//! | 角色已声明，**规则要求的字段**缺失**或为空串** | **`block`** | `4.2[role=derived]`（无 `formula=`）与 `4.2[role=derived formula=]` |
//!
//! ⛔ 第二行是本模块最容易被读错的一格，务必分清：**协议要求**的字段缺失就是
//! **违规本身**（那正是这条检查存在的理由）；只有**「连规则适不适用都不知道」**
//! 才是不可判定。
//!
//! ⭐ 第二行里「**为空串**」那半是实测逼出来的：本模块第一版把 `formula=`
//! 的空值当**注解畸形** ⇒ 整条注解作废 ⇒ 该判定退化成 `not_evaluable`
//! ⇒ **「模型把口径写空了」这件事永远抓不到**。空串与「键不存在」在这个协议里
//! 是**同一个违规**（该填而没填），必须落到 `block`。唯一不许为空的是
//! **`role=` 的值** —— 那个空了，我们连这条声明归哪道检查管都不知道。
//! 钉死它的测试：`derived_with_empty_formula_still_blocks`。
//!
//! ## ⛔ 已删除的虚构协议：`observed` 的 `witness=`（诚实台账）
//!
//! 本模块**第一版**有第三条检查 `ObservedWitnessCheck` 与载荷键
//! `witness=<claim id>`，要求 `role=observed` 声明一条能被账本解析到的旁证。
//! ⛔ **`witness=` 在被吸收的上游里不存在，是第一版为了让
//! [`RefusalReason::RemainderStillFails`] 在测试里可达而发明的。**
//! 「造一条规则让 fail-closed 分支可达」＝「为满足测试而造机械」的教科书定义。
//! 于是：检查、两个检查码（`…witness_absent` / `…witness_unresolved`）与载荷键
//! 常量**全部删除**，`default_checks()` 只剩两条内建检查。`role=observed` 现在
//! **没有任何内建规则** —— 这是诚实的边界，不是漏写：给「观测」编一个旁证协议
//! 只能靠凭空发明字段。钉死它的测试：
//! `an_observed_claim_is_silent_because_no_rule_was_invented_for_it`。
//!
//! ## ⭐ 那么什么真正到达 `REMAINDER_STILL_FAILS`（别信注释，信测试）
//!
//! 删掉虚构协议后，「切完复验仍失败」有两条真实来源。**两条都不需要发明字段。**
//!
//! ### 来源一：调用方少报了账本（设计上的那条）
//!
//! [`admit_claims`] 第一轮只信调用方给的 `claims`，但切完之后 `degrade` 阶段 4
//! **刻意全量重解析**切后的文本（`parse_declarations(cut_text)`，**不是**复用
//! 调用方的集合）。⇒ 调用方少报的那条阻断声明，在重解析那一刻被抓出来。
//! 测试 `an_under_reported_ledger_is_caught_by_the_mandatory_recheck` 钉住它：
//! **同一段文本、同一道门**，完整解析 ⇒ 可降级发布；少报一条 ⇒ 拒答。
//!
//! ### ⛔ 来源二：字节拼接（存在，但**不要**当设计来依赖）
//!
//! 直觉上「切掉的声明不会制造新违规」，因为解析器把游标推进到 `cut_span.end`，
//! 下一条声明只能落在其后 ⇒ 切掉的跨度之间拼不出新声明。**这条论证有洞**：
//! `parse_declarations` 的游标在注解畸形时落在 `[` 的**后一位**，而围栏声明的
//! `cut_span` 会一路伸到围栏尾（甚至文本尾）⇒ 这两者可以交叠 ⇒ 被围栏声明吃掉
//! 的那个字节区间，切掉后左右两侧**可能**拼出一条新的声明。
//!
//! ⛔ 因此本模块**不**声称「完整解析的路径不会级联」—— 那是个未证明的断言。
//! 唯一**保证**成立的不变式是这条：
//!
//! > **凡本层放行的文本，用同一注册表复验必须零阻断。**
//!
//! 它由 `degrade` 阶段 4 直接保证（放行前刚复验过），干净路径则由第一轮保证。
//! 测试 `every_release_is_revalidated_clean` 在语料级钉住它，**并且刻意收录了
//! 上面那个交叠形状**（围栏 + 畸形注解）—— 那条测试只断言这条保证，**不**预言
//! 它给出哪种裁决。
//!
//! ⭐ 也就是说：真正承重的从来不是「级联不可能」这个论证，而是**复验这一步本身**。
//! 去掉虚构协议没有削弱 fail-closed，只是不再需要为了它编一个协议。
//!
//! ## 纯度由签名保证
//!
//! [`ProvenanceCheck::evaluate`] 只收 [`Ledger`]，而 [`Ledger`] 里**只有自有数据**
//! （`&str` + `Vec<DeclaredClaim>`）：没有 `&Path`、没有 `impl Read`、没有
//! `Clock`/`Instant`、没有 executor。⇒ 想在这层里偷偷跑一次模型或读一次盘，
//! **编译不过**。模型早已声明完，本层只**验证声明**。
//!
//! ## 复用自己的摘要，不造第二个哈希
//!
//! [`Digest`] 取 `neotrix_neobot::nt_determinism::Digest`
//! （`neotrix-core/Cargo.toml:106` 已有该依赖）——**零新依赖**。**重新实现**
//! 才会踩它被刻意修掉的那个缺陷类（多字段移位塞进同一累加器 ⇒ 位段别名 ⇒
//! 失同步不可见，见 `nt_determinism.rs:96-103` 记录的 ra2.exe 缺陷）。
//!
//! [`Ledger::digest`] 与 [`ReleasedText::digest`] 都**键序无关**
//! （`sorted_keys` + `field_sorted_keys`）⇒ 声明顺序不改变审计身份。
//!
//! ## ⛔ 出处行号的诚实边界
//!
//! 本仓**没有** `HKUDS/Vibe-Trading` 的检出（2026-10-05 实测 `find` 零命中）。
//! 上文 `figures.py:36` / `registry.py:1-13` / `release.py:32` / `release.py:515-527`
//! / `ledger.py:337-341` 全部来自**吸收任务书转述**，**未经本会话逐行核对**。
//! 按 R-SCAN-1b，它们是**候选出处**，不是已读证据。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use neotrix_neobot::nt_determinism::{sorted_keys, Digest};

// ═══════════════════════════════════════════════════════════════════════════
// 字节跨度：全函数，永不 panic
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐ 文本里的**字节**跨度。
///
/// `start > end`、越界、非字符边界一律合法构造 —— 一切非法情形都在
/// [`Span::slice`] 处退化成 `None`，**没有一个 `unwrap`**。理由：降级切割要
/// 区分「figure 切得掉」与「figure 定位不到」（`release.py:520-527` 的
/// “an issue cannot be cut, a flagged figure cannot be located”），而这个区分
/// 只在**跨度可能撒谎**时才有意义。把它做成不可能，就等于把 fail-closed 的
/// 一条理由删掉了。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// 构造跨度。**不校验**（见类型文档）。
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// 起点（含）。
    #[must_use]
    pub fn start(&self) -> usize {
        self.start
    }

    /// 终点（不含）。
    #[must_use]
    pub fn end(&self) -> usize {
        self.end
    }

    /// 长度；`end < start` 时饱和到 `0`（不饱和成天文数字）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// 是否空跨度。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// ⭐ 取子串。越界 / 反向 / 非字符边界 ⇒ `None`。
    #[must_use]
    pub fn slice<'t>(&self, text: &'t str) -> Option<&'t str> {
        text.get(self.start..self.end)
    }

    /// 两跨度的并包围盒。
    #[must_use]
    pub fn merge(&self, other: &Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// 渲染成 `start..end`（进 issue 详情，便于人眼定位）。
    #[must_use]
    pub fn render(&self) -> String {
        format!("{}..{}", self.start, self.end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 形状：数字 / 日期 / 序数 / 索引单元 / 代码围栏
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐ figure 的**形状**。语言无关，靠字符形态判定（`figures.py:47-84`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FigureShape {
    /// 数值（可带 `,` 千分位、`.` 小数、`%`/`k`/`m`/`b`/`x` 量级后缀与正负号）。
    Number,
    /// 日期 `YYYY-MM-DD`。
    Date,
    /// 序数 `3rd` / `1st` / `2nd` / `4th`。
    Ordinal,
    /// 索引单元 `#42`。
    IndexCell,
    /// 代码围栏（` ```lang … ``` `），跨度覆盖**整个围栏块**。
    Fence,
}

/// 五种被识别的形状（**声明**，不是分支）。
pub const FIGURE_SHAPES: [FigureShape; 5] = [
    FigureShape::Number,
    FigureShape::Date,
    FigureShape::Ordinal,
    FigureShape::IndexCell,
    FigureShape::Fence,
];

impl FigureShape {
    /// 稳定标签（参与摘要，故必须稳定）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FigureShape::Number => "number",
            FigureShape::Date => "date",
            FigureShape::Ordinal => "ordinal",
            FigureShape::IndexCell => "index-cell",
            FigureShape::Fence => "fence",
        }
    }

    /// ⭐ 判别值（1 起；`0` 保留给「形状未识别」）。
    #[must_use]
    pub fn code(self) -> u8 {
        match self {
            FigureShape::Number => 1,
            FigureShape::Date => 2,
            FigureShape::Ordinal => 3,
            FigureShape::IndexCell => 4,
            FigureShape::Fence => 5,
        }
    }

    /// ⭐⭐ 形状判定。**返回 `None` 就是「不认识」** —— 调用方必须把它当
    /// `not_evaluable` 处理，绝不猜 pass 也不猜 fail。
    #[must_use]
    pub fn detect(figure: &str) -> Option<Self> {
        let b = figure.as_bytes();
        if b.starts_with(b"```") {
            return Some(FigureShape::Fence);
        }
        if is_date_token(b) {
            return Some(FigureShape::Date);
        }
        if is_index_cell_token(b) {
            return Some(FigureShape::IndexCell);
        }
        if is_ordinal_token(b) {
            return Some(FigureShape::Ordinal);
        }
        if is_number_token(b) {
            return Some(FigureShape::Number);
        }
        None
    }
}

impl fmt::Display for FigureShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

fn all_digits(slice: Option<&[u8]>) -> bool {
    slice.is_some_and(|s| !s.is_empty() && s.iter().all(u8::is_ascii_digit))
}

fn is_date_token(b: &[u8]) -> bool {
    b.len() == 10 && b.get(4) == Some(&b'-') && b.get(7) == Some(&b'-') && all_digits(b.get(0..4))
        && all_digits(b.get(5..7))
        && all_digits(b.get(8..10))
}

fn is_index_cell_token(b: &[u8]) -> bool {
    b.first() == Some(&b'#') && all_digits(b.get(1..))
}

fn is_ordinal_token(b: &[u8]) -> bool {
    let mut i = 0usize;
    while i < b.len() && b.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    if i == 0 || i == b.len() {
        return false;
    }
    matches!(b.get(i..), Some([b's', b't'] | [b'n', b'd'] | [b'r', b'd'] | [b't', b'h']))
}

/// 数值 token：`[+-]?digits[,digits]*[.digits]*[suffix]`，`digits` 至少一位。
fn is_number_token(b: &[u8]) -> bool {
    let mut i = 0usize;
    if b.first().is_some_and(|c| *c == b'+' || *c == b'-') {
        i = 1;
    }
    let mut digits = 0usize;
    let mut dot = false;
    while i < b.len() {
        match *b.get(i).unwrap_or(&0) {
            c if c.is_ascii_digit() => {
                digits += 1;
                i += 1;
            }
            b',' if digits > 0 => i += 1,
            b'.' if !dot && digits > 0 => {
                dot = true;
                i += 1;
            }
            c if is_number_suffix(c) => break,
            _ => return false,
        }
    }
    let mut j = i;
    if j < b.len() {
        if !is_number_suffix(*b.get(j).unwrap_or(&0)) {
            return false;
        }
        j += 1;
        if j != b.len() {
            return false;
        }
    }
    digits > 0 && j == b.len()
}

fn is_number_suffix(c: u8) -> bool {
    matches!(c, b'%' | b'k' | b'K' | b'm' | b'M' | b'b' | b'B' | b'x' | b'X')
}

// ═══════════════════════════════════════════════════════════════════════════
// 角色：模型自报的来源
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐ 声明的来源角色（`figures.py:36` 的 `ROLES`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProvenanceRole {
    /// 观测到（工具/行情直接读到的）。
    Observed,
    /// 推导（由别的数算出来 ⇒ **必须**给出公式）。
    Derived,
    /// 提议（模型自己的建议，不是事实）。
    Proposed,
    /// 引用（⇒ **必须**给出 source）。
    Cited,
    /// 计数（枚举/行数 ⇒ 声明 basis）。
    Count,
}

/// 五个角色（**声明**，不是分支）。`check-naming` 的「前缀规约」在此不适用：
/// 角色名是**外部协议的一部分**，改名即破坏与模型的契约。
pub const ROLES: [ProvenanceRole; 5] = [
    ProvenanceRole::Observed,
    ProvenanceRole::Derived,
    ProvenanceRole::Proposed,
    ProvenanceRole::Cited,
    ProvenanceRole::Count,
];

impl ProvenanceRole {
    /// 稳定标签（参与摘要，故必须稳定）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ProvenanceRole::Observed => "observed",
            ProvenanceRole::Derived => "derived",
            ProvenanceRole::Proposed => "proposed",
            ProvenanceRole::Cited => "cited",
            ProvenanceRole::Count => "count",
        }
    }

    /// 解析角色名。**未知 ⇒ `None`**，绝不回退到某个默认角色
    /// （回退＝把「不知道」洗成「大概是 observed」＝本层要消灭的病）。
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "observed" => Some(ProvenanceRole::Observed),
            "derived" => Some(ProvenanceRole::Derived),
            "proposed" => Some(ProvenanceRole::Proposed),
            "cited" => Some(ProvenanceRole::Cited),
            "count" => Some(ProvenanceRole::Count),
            _ => None,
        }
    }
}

impl fmt::Display for ProvenanceRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 角色集的人读标签（进 issue 详情）。
pub const ROLE_SET_LABEL: &str = "observed/derived/proposed/cited/count";

/// 载荷键：`derived` 必须给公式。
pub const PAYLOAD_FORMULA: &str = "formula";
/// 载荷键：`cited` 必须给来源。
pub const PAYLOAD_SOURCE: &str = "source";
/// 载荷键：`count` 的枚举口径。
pub const PAYLOAD_BASIS: &str = "basis";

// ═══════════════════════════════════════════════════════════════════════════
// 一条声明
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐⭐ 一条声明的 claim：**文本跨度 + 自报角色 + 可选载荷**。
///
/// 字段私有：角色与载荷无法与跨度**脱钩**被构造出来 —— 「切得到」这件事
/// 依赖 `figure` 真的躺在 `cut_span` 起点，构造期就得保证。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredClaim {
    figure: String,
    figure_span: Span,
    cut_span: Span,
    occurrence: usize,
    shape: Option<FigureShape>,
    raw_role: String,
    role: Option<ProvenanceRole>,
    payload: Vec<(String, String)>,
    well_formed: bool,
}

impl DeclaredClaim {
    /// ⭐ 程序化构造（不走文本解析）。
    ///
    /// 语义约定：**`cut_span` 的起点就是 figure 的起点**（与解析器产出一致）。
    /// 这条约定正是降级切割的校验点（[`admit_claims`] 里的
    /// `slice.starts_with(figure)`），因此手造 claim 也逃不掉 fail-closed。
    #[must_use]
    pub fn declared(
        figure: impl Into<String>,
        cut_span: Span,
        role: ProvenanceRole,
        payload: &[(&str, &str)],
    ) -> Self {
        let figure: String = figure.into();
        let figure_end = cut_span.start().saturating_add(figure.len());
        let mut pairs: Vec<(String, String)> = payload
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        pairs.sort();
        Self {
            shape: FigureShape::detect(&figure),
            occurrence: 0,
            figure,
            figure_span: Span::new(cut_span.start(), figure_end),
            cut_span,
            raw_role: role.as_str().to_owned(),
            role: Some(role),
            payload: pairs,
            well_formed: true,
        }
    }

    /// figure 文本。
    #[must_use]
    pub fn figure(&self) -> &str {
        &self.figure
    }

    /// figure 自身在文本中的跨度（围栏时是**整个块**）。
    #[must_use]
    pub fn figure_span(&self) -> Span {
        self.figure_span
    }

    /// ⭐ 降级时**要切掉的**跨度 = `figure ∪ 注解`。
    ///
    /// 切注解是必须的：留下 `[role=cited]` 这种孤儿会把文本搞成**语法残缺**
    /// 的一段话，而不是「删掉了一句」。
    #[must_use]
    pub fn cut_span(&self) -> Span {
        self.cut_span
    }

    /// ⭐ 稳定标识 = `figure#occurrence`（同一 figure 的第几次出现，从 0 起）。
    ///
    /// ⚠️ 由**文档位置**决定，与调用方给的 `Vec` 顺序无关 ⇒ 逆序喂进
    /// [`Ledger`] 不改变任何 id ⇒ 旁证解析天然顺序无关。
    #[must_use]
    pub fn id(&self) -> String {
        format!("{}#{}", self.figure, self.occurrence)
    }

    /// 同形第几次出现（0 起）。
    #[must_use]
    pub fn occurrence(&self) -> usize {
        self.occurrence
    }

    /// ⭐ 形状。`None` = **不认识** ⇒ 调用方须 `not_evaluable`。
    #[must_use]
    pub fn shape(&self) -> Option<FigureShape> {
        self.shape
    }

    /// ⭐ 自报角色。`None` = 未声明 / 未知 / 注解畸形 ⇒ **不猜**。
    #[must_use]
    pub fn role(&self) -> Option<ProvenanceRole> {
        self.role
    }

    /// 角色**原文**（未知时保留，供人排错；空串 = 连 `role=` 都没解出来）。
    #[must_use]
    pub fn raw_role(&self) -> &str {
        &self.raw_role
    }

    /// 注解是否被完整解析。`false` ⇒ 角色不可信 ⇒ `not_evaluable`。
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.well_formed
    }

    /// ⭐ 取一个载荷值。**同名多个取排序后第一个**（确定性，不依赖声明序）。
    #[must_use]
    pub fn payload_value(&self, key: &str) -> Option<&str> {
        self.payload.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// 某键的全部取值（已按 `(key, value)` 排序）。
    #[must_use]
    pub fn payload_values(&self, key: &str) -> Vec<&str> {
        self.payload.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).collect()
    }

    /// 全部载荷（已排序）。
    #[must_use]
    pub fn payload(&self) -> &[(String, String)] {
        &self.payload
    }

    /// ⭐ 本条声明的内容地址（逐字段显式列出，`variant` 声明编码版本）。
    #[must_use]
    pub fn digest(&self) -> u64 {
        let pairs: Vec<String> =
            self.payload.iter().map(|(k, v)| format!("{k}={v}")).collect();
        Digest::new()
            .section("claim")
            .variant(CLAIM_SCHEMA_V)
            .field_str(&self.figure)
            .field_u64(self.figure_span.start() as u64)
            .field_u64(self.cut_span.end() as u64)
            .field_u64(self.occurrence as u64)
            .field_u64(u64::from(self.shape.map_or(0u8, FigureShape::code)))
            .field_str(&self.raw_role)
            .field_bool(self.well_formed)
            .field_sorted_keys(&pairs)
            .finish()
    }
}

/// 声明编码版本（改字段布局**必须**递增 ⇒ 旧审计摘要不会静默匹配新编码）。
const CLAIM_SCHEMA_V: u8 = 1;
/// 账本编码版本。
const LEDGER_SCHEMA_V: u8 = 1;
/// 发布编码版本。
const RELEASE_SCHEMA_V: u8 = 1;

// ═══════════════════════════════════════════════════════════════════════════
// ⭐ 解析：全函数，永不 panic
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐⭐ 解析声明。**总函数**：任何输入（含畸形括号、多字节、未闭合围栏）都返回
/// 一个 `Vec`，不 panic、不 `unwrap`。
///
/// ⛔ 本函数**不读自然语言的词**：它只认 `[role=…]` 这一行**声明协议**。
/// 没跟着 figure 的方括号（`[role=observed]187.40`）**不是**声明 —— 静默跳过，
/// 因为那不是协议。
#[must_use]
pub fn parse_declarations(text: &str) -> Vec<DeclaredClaim> {
    let b = text.as_bytes();
    let mut out: Vec<DeclaredClaim> = Vec::new();
    let mut occurrences: BTreeMap<String, usize> = BTreeMap::new();
    let mut cursor = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        if b.get(i) != Some(&b'[') {
            i += 1;
            continue;
        }
        // 反向走 figure token（只认 ASCII 图形字符 ⇒ `start` 必是字符边界）。
        let mut start = i;
        while start > cursor && b.get(start - 1).is_some_and(|c| is_token_byte(*c)) {
            start -= 1;
        }
        if start == i {
            // 前面没有非空 figure ⇒ 这不是一条声明。
            i += 1;
            continue;
        }
        let Some(figure) = text.get(start..i) else {
            i += 1;
            continue;
        };
        let ann = scan_annotation(text, i);
        let figure_end = if figure.starts_with("```") {
            fence_block_end(b, line_end(b, i))
        } else {
            i
        };
        let cut_end = figure_end.max(ann.end);
        let occurrence = occurrences.entry(figure.to_owned()).or_insert(0);
        let occurrence_index = *occurrence;
        *occurrence = occurrence.saturating_add(1);
        out.push(DeclaredClaim {
            shape: FigureShape::detect(figure),
            occurrence: occurrence_index,
            figure: figure.to_owned(),
            figure_span: Span::new(start, figure_end),
            cut_span: Span::new(start, cut_end),
            raw_role: ann.role_raw,
            role: ann.role,
            payload: ann.payload,
            well_formed: ann.complete,
        });
        cursor = if ann.complete { cut_end } else { i.saturating_add(1) };
        i = cursor.max(i.saturating_add(1));
    }
    out
}

/// figure token 的合法字节：ASCII 图形字符，排除空白与方括号。
fn is_token_byte(c: u8) -> bool {
    c > 0x20 && c < 0x7f && c != b'[' && c != b']'
}

fn line_end(b: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < b.len() && b.get(i) != Some(&b'\n') {
        i += 1;
    }
    i
}

/// ⭐ 围栏块的结束（不含结尾换行）。**未闭合 ⇒ 文本末尾**（诚实的降级，
/// 不是静默丢弃整块）。
fn fence_block_end(b: &[u8], after_open: usize) -> usize {
    let mut i = after_open;
    while i + 3 < b.len() {
        if b.get(i) == Some(&b'\n')
            && b.get(i + 1) == Some(&b'`')
            && b.get(i + 2) == Some(&b'`')
            && b.get(i + 3) == Some(&b'`')
        {
            return line_end(b, i.saturating_add(4));
        }
        i += 1;
    }
    b.len()
}

/// 注解扫描结果。
struct Annotation {
    /// 注解结束偏移（`[` 位置 = 未解析成功时的零宽跨度）。
    end: usize,
    role_raw: String,
    role: Option<ProvenanceRole>,
    payload: Vec<(String, String)>,
    complete: bool,
}

/// ⭐ 注解扫描：**容错但绝不半信**。要么完整解出 `[role=R k=v …]`，要么
/// `complete = false` 且 `role = None`。不存在「解出角色但载荷丢了一半」这种
/// 中间态 —— 那会让检查误以为规则适用。
fn scan_annotation(text: &str, open: usize) -> Annotation {
    let b = text.as_bytes();
    let malformed = || Annotation {
        end: open,
        role_raw: String::new(),
        role: None,
        payload: Vec::new(),
        complete: false,
    };
    let mut i = open.saturating_add(1);
    i = skip_inline_ws(b, i);
    let Some((key, next)) = take_ident(b, i) else {
        return malformed();
    };
    if key != "role" {
        return malformed();
    }
    i = skip_inline_ws(b, next);
    if b.get(i) != Some(&b'=') {
        return malformed();
    }
    i = skip_inline_ws(b, i.saturating_add(1));
    let (role_raw, next) = take_value(b, i);
    if role_raw.is_empty() {
        return malformed();
    }
    let role = ProvenanceRole::parse(&role_raw);
    i = next;
    let mut payload: Vec<(String, String)> = Vec::new();
    loop {
        let save = i;
        let j = skip_inline_ws(b, i);
        match b.get(j) {
            Some(b']') => {
                payload.sort();
                return Annotation { end: j.saturating_add(1), role_raw, role, payload, complete: true };
            }
            // ⛔ 零空白处不是 `]` ⇒ 协议破损（也顺手挡住了换行：注解限单行，
            // 否则 span 会跨行，切割就会吃掉邻行）。
            _ if j == save => return malformed(),
            _ => {}
        }
        i = j;
        let Some((k, next)) = take_ident(b, i) else {
            return malformed();
        };
        i = skip_inline_ws(b, next);
        if b.get(i) != Some(&b'=') {
            return malformed();
        }
        i = skip_inline_ws(b, i.saturating_add(1));
        // ⭐ 载荷值**允许为空**。`role=derived formula=` 是一个**声明完整、但
        // 必填字段为空**的注解 ⇒ 规则适用 ⇒ 违规（`block`）。若把空值当畸形，
        // 整条注解作废 ⇒ 规则退化成 `not_evaluable` ⇒ 「模型把口径写空了」
        // 这件事**永远抓不到** —— 那正是这条检查存在的理由。
        //
        // ⛔ 唯一不许为空的是 **`role=` 的值**：那个空了，我们连「这条声明属
        // 于哪道检查的管辖范围」都不知道，只能诚实地说不可判定。
        let (v, next) = take_value(b, i);
        payload.push((k, v));
        i = next;
    }
}

/// 只跳空格与制表符（**不跳换行** ⇒ 注解限单行）。
fn skip_inline_ws(b: &[u8], from: usize) -> usize {
    let mut i = from;
    while b.get(i).is_some_and(|c| *c == b' ' || *c == b'\t') {
        i += 1;
    }
    i
}

fn take_ident(b: &[u8], from: usize) -> Option<(String, usize)> {
    let mut i = from;
    while b.get(i).is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_') {
        i += 1;
    }
    if i == from {
        return None;
    }
    let ident = String::from_utf8_lossy(b.get(from..i)?).into_owned();
    Some((ident, i))
}

/// 取一个载荷值：读到空白 / `[` / `]` 为止。
///
/// ⭐ 因为分隔符全是 ASCII（`< 0x80`），逐字节推进**不会**切进 UTF-8 续字节
/// ⇒ `from_utf8_lossy` 是纯保险，不是修复。
fn take_value(b: &[u8], from: usize) -> (String, usize) {
    let mut i = from;
    while i < b.len() && !is_value_end(*b.get(i).unwrap_or(&0)) {
        i += 1;
    }
    let value = String::from_utf8_lossy(b.get(from..i).unwrap_or(&[])).into_owned();
    (value, i)
}

fn is_value_end(c: u8) -> bool {
    c.is_ascii_whitespace() || c == b'[' || c == b']'
}

// ═══════════════════════════════════════════════════════════════════════════
// 问题行：只有「阻断」与「不可判定」，没有「通过」
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐ 问题严重度。**没有 `Pass`** —— 通过就是**没有问题**；给通过也造一行
/// 会让「检查没跑」和「检查跑了且通过」长得一样。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IssueSeverity {
    /// 可判定，且违规 ⇒ 计入阻断 ⇒ 触发降级。
    Block,
    /// 判定所需仪表不存在（角色未声明 / 注解畸形 / 角色不在 [`ROLES`]）。
    NotEvaluable,
}

/// ⭐⭐ 一条检查产出的**一行**问题。
///
/// ⛔ `not_evaluable` **不是**失败，也**绝不**算通过：它只说明这道检查此刻
/// 没有资格表态。唯一的去向是被 [`Issue::is_blocking`] 排除。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Issue {
    code: String,
    claim_index: usize,
    figure: String,
    detail: String,
    severity: IssueSeverity,
}

impl Issue {
    /// 违规行。
    #[must_use]
    pub fn block(code: &str, claim_index: usize, claim: &DeclaredClaim, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            claim_index,
            figure: claim.figure().to_owned(),
            detail: detail.into(),
            severity: IssueSeverity::Block,
        }
    }

    /// ⭐ 不可判定行。
    #[must_use]
    pub fn not_evaluable(
        code: &str,
        claim_index: usize,
        claim: &DeclaredClaim,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code: code.to_owned(),
            claim_index,
            figure: claim.figure().to_owned(),
            detail: detail.into(),
            severity: IssueSeverity::NotEvaluable,
        }
    }

    /// 稳定检查码（报告主键；跨版本不许改）。
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// 命中的声明在账本中的下标。
    #[must_use]
    pub fn claim_index(&self) -> usize {
        self.claim_index
    }

    /// 命中的 figure 文本。
    #[must_use]
    pub fn figure(&self) -> &str {
        &self.figure
    }

    /// 人读说明。
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// 严重度。
    #[must_use]
    pub fn severity(&self) -> IssueSeverity {
        self.severity
    }

    /// ⭐ 是否计入阻断（⇒ 触发降级切割）。
    #[must_use]
    pub fn is_blocking(&self) -> bool {
        self.severity == IssueSeverity::Block
    }

    /// 是否是「无法判定」。
    #[must_use]
    pub fn is_not_evaluable(&self) -> bool {
        self.severity == IssueSeverity::NotEvaluable
    }

    /// 单行渲染（无 IO，纯函数）。
    #[must_use]
    pub fn render(&self) -> String {
        let tag = match self.severity {
            IssueSeverity::Block => "BLOCK",
            IssueSeverity::NotEvaluable => "NOT_EVALUABLE",
        };
        format!("[{tag:<13}] {:<38} claim#{} `{}` {}", self.code, self.claim_index, self.figure, self.detail)
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 账本：检查能看到的全部世界（自有数据 ⇒ 无 IO 可钻的空子）
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐ 账本 = 一段文本 + 一组声明。**只有 `&str` 与 `&[DeclaredClaim]`**。
#[derive(Debug, Clone, Copy)]
pub struct Ledger<'a> {
    text: &'a str,
    claims: &'a [DeclaredClaim],
}

impl<'a> Ledger<'a> {
    /// 组装账本。`claims` 可以是调用方**自造**的（见 [`DeclaredClaim::declared`]）
    /// ⇒ 未受信任的跨度会被 [`admit_claims`] 的定位校验挡下，而不是 panic。
    #[must_use]
    pub fn new(text: &'a str, claims: &'a [DeclaredClaim]) -> Self {
        Self { text, claims }
    }

    /// 被判的文本。
    #[must_use]
    pub fn text(&self) -> &'a str {
        self.text
    }

    /// 声明集合（顺序**不进入**任何决策）。
    #[must_use]
    pub fn claims(&self) -> &'a [DeclaredClaim] {
        self.claims
    }

    /// 按下标取声明。
    #[must_use]
    pub fn claim_at(&self, index: usize) -> Option<&'a DeclaredClaim> {
        self.claims.get(index)
    }

    /// ⭐ 按稳定 id 取声明（`figure#occurrence` 寻址）。
    ///
    /// 线性扫描、返回切片中**第一条** ⇒ 结果是切片的函数。解析器产出的 id
    /// 唯一（`figure#occurrence`），所以顺序不影响结果 —— 测试
    /// `claim_id_lookup_is_independent_of_claim_order` 钉住这一点，并附**反例**
    /// 证明该测试有判别力。
    #[must_use]
    pub fn claim_by_id(&self, id: &str) -> Option<&'a DeclaredClaim> {
        self.claims.iter().find(|c| c.id() == id)
    }

    /// 声明条数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.claims.len()
    }

    /// 无声明。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.claims.is_empty()
    }

    /// ⭐ 账本内容地址。**键序无关**：声明逆序不改变身份（`sorted_keys`）。
    #[must_use]
    pub fn digest(&self) -> u64 {
        let keys: Vec<String> =
            self.claims.iter().map(|c| format!("{}|{:016x}", c.id(), c.digest())).collect();
        Digest::new()
            .section("ledger")
            .variant(LEDGER_SCHEMA_V)
            .field_str(self.text)
            .field_u64(self.claims.len() as u64)
            .field_sorted_keys(&sorted_keys(keys))
            .finish()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ⭐ 检查 trait：声明行，不是控制流分支
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐⭐ 一条声明式检查。**对象安全**（无泛型方法、无 `Self` 返回）。
///
/// 实现者**不得**写 IO / 时钟 / LLM 调用 —— [`Ledger`] 里没有任何能读写外部
/// 世界的句柄，想写也编译不过。
pub trait ProvenanceCheck {
    /// 稳定检查码（报告主键；**跨版本不许改**）。
    fn code(&self) -> &'static str;

    /// 人读描述（进清单 / `--list`）。
    fn description(&self) -> &'static str;

    /// ⭐ 求值。**必须全函数**：畸形 / 缺载荷只能产出
    /// [`IssueSeverity::NotEvaluable`] 或 [`IssueSeverity::Block`]，不许 panic。
    fn evaluate(&self, ledger: &Ledger<'_>) -> Vec<Issue>;
}

/// 统一措辞的「角色不可知」说明。**所有检查共用同一措辞** ⇒ 日志里不会
/// 出现几种说法讲同一件事。
fn undeclared_role_detail(claim: &DeclaredClaim) -> String {
    if !claim.is_well_formed() {
        return format!(
            "declaration at {} is malformed (no parseable `role=`): this check cannot tell \
             whether the claim is in its domain, so it is NOT_EVALUABLE rather than assumed \
             compliant",
            claim.cut_span().render()
        );
    }
    if claim.raw_role().is_empty() {
        return format!(
            "declaration of `{}` carries no `role=` key: this check cannot tell whether the \
             claim is in its domain, so it is NOT_EVALUABLE rather than assumed compliant",
            claim.figure()
        );
    }
    format!(
        "declared role `{}` is not one of the five known roles ({ROLE_SET_LABEL}): this check \
         cannot tell whether the claim is in its domain, so it is NOT_EVALUABLE rather than \
         assumed compliant",
        claim.raw_role()
    )
}

// ── 检查 1：`derived` 必须给出公式 ────────────────────────────────────────

/// 检查码：`derived` 声明未给出公式。
pub const CODE_DERIVED_FORMULA: &str = "nt.prov.derived.formula";

/// ⭐⭐ 检查一：**`derived` 必须命名它的公式**。
///
/// 角色集本身蕴含这条规则：`derived` 的全部意义就是「这个数是算出来的」，
/// 而算不出来的东西不配叫 `derived` —— 它只是个 `observed` 或 `proposed`
/// 冒充了推导。
///
/// | 情形 | 判定 |
/// |---|---|
/// | `role=derived` 且 `formula=` 非空 | 静默 |
/// | `role=derived` 且 `formula=` 空串 | **block**（声明了口径却是空的） |
/// | `role=derived` 且无 `formula=` 键 | **block** |
/// | 角色不可知 | `not_evaluable` |
/// | 其它角色 | 静默（本检查对它无话可说） |
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DerivedFormulaCheck;

impl DerivedFormulaCheck {
    /// 新建（零大小，可作单例）。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ProvenanceCheck for DerivedFormulaCheck {
    fn code(&self) -> &'static str {
        CODE_DERIVED_FORMULA
    }

    fn description(&self) -> &'static str {
        "a `derived` claim must name the formula that produced it (`formula=` payload); an \
         undeclared role is NOT_EVALUABLE, never an assumed pass"
    }

    fn evaluate(&self, ledger: &Ledger<'_>) -> Vec<Issue> {
        let mut out: Vec<Issue> = Vec::new();
        for (i, claim) in ledger.claims().iter().enumerate() {
            let Some(role) = claim.role() else {
                out.push(Issue::not_evaluable(self.code(), i, claim, undeclared_role_detail(claim)));
                continue;
            };
            if role != ProvenanceRole::Derived {
                continue;
            }
            match claim.payload_value(PAYLOAD_FORMULA) {
                Some(v) if !v.trim().is_empty() => {}
                Some(_) => out.push(Issue::block(
                    self.code(),
                    i,
                    claim,
                    "a `derived` claim declares an empty `formula=` payload: the derivation is \
                     asserted but unnamed, which is a claim, not a derivation",
                )),
                None => out.push(Issue::block(
                    self.code(),
                    i,
                    claim,
                    "a `derived` claim names no `formula=` payload: a derived figure with no \
                     formula cannot be reproduced or audited",
                )),
            }
        }
        out
    }
}

// ── 检查 2：`cited` 必须给出来源 ──────────────────────────────────────────

/// 检查码：`cited` 声明未给出来源。
pub const CODE_CITED_SOURCE: &str = "nt.prov.cited.source";

/// ⭐⭐ 检查二：**`cited` 必须携带来源**。
///
/// 与检查一同源但**不可互相替代**：`formula` 讲「怎么算出来的」（可复算），
/// `source` 讲「从哪儿看来的」（可回查）。少了 `source` 的 `cited` 是一句
/// 没有人能去核对的断言 —— 比不写 `cited` 更坏，因为它自称有出处。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CitedSourceCheck;

impl CitedSourceCheck {
    /// 新建（零大小，可作单例）。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ProvenanceCheck for CitedSourceCheck {
    fn code(&self) -> &'static str {
        CODE_CITED_SOURCE
    }

    fn description(&self) -> &'static str {
        "a `cited` claim must carry the source it was read from (`source=` payload); an \
         undeclared role is NOT_EVALUABLE, never an assumed pass"
    }

    fn evaluate(&self, ledger: &Ledger<'_>) -> Vec<Issue> {
        let mut out: Vec<Issue> = Vec::new();
        for (i, claim) in ledger.claims().iter().enumerate() {
            let Some(role) = claim.role() else {
                out.push(Issue::not_evaluable(self.code(), i, claim, undeclared_role_detail(claim)));
                continue;
            };
            if role != ProvenanceRole::Cited {
                continue;
            }
            match claim.payload_value(PAYLOAD_SOURCE) {
                Some(v) if !v.trim().is_empty() => {}
                Some(_) => out.push(Issue::block(
                    self.code(),
                    i,
                    claim,
                    "a `cited` claim declares an empty `source=` payload: it claims a citation \
                     while naming nothing to look up",
                )),
                None => out.push(Issue::block(
                    self.code(),
                    i,
                    claim,
                    "a `cited` claim carries no `source=` payload: nobody can return to the \
                     origin of a citation that does not name one",
                )),
            }
        }
        out
    }
}

// ── 内建检查到此为止（⛔ 曾经存在的第三条检查已被删除，见模块头「诚实台账」）──

/// ⭐ 默认注册表（两条声明行）。**新增检查在此追加一行** —— 这是全层唯一需要
/// 改的地方，且它是**声明**，不是控制流分支。
///
/// ⛔ 这里**曾经**有第三条 `ObservedWitnessCheck`（`witness=` 旁证），已删除：
/// 那个字段在上游不存在，是为让 fail-closed 分支可达而发明的。**不要**为了「让
/// 某个分支可达」再加一条 —— 见模块头「什么真正到达 `REMAINDER_STILL_FAILS`」。
#[must_use]
pub fn default_checks() -> Vec<Box<dyn ProvenanceCheck>> {
    vec![Box::new(DerivedFormulaCheck::new()), Box::new(CitedSourceCheck::new())]
}

/// ⭐ 校验入口。**纯函数**：`(ledger, checks) ⇒ 排序后的问题行`。
///
/// 出口统一排序 ⇒ **注册顺序不进入结果**（本仓刚修完的哈希序纪律在溯源门的
/// 同款落实，对照 `nt_determinism.rs:200-224`）。`sort` 稳定 ⇒ 完全同键的行
/// 仍按注册序，但注册表是调用方显式组装的，**是函数的**。
#[must_use]
pub fn validate(ledger: &Ledger<'_>, checks: &[Box<dyn ProvenanceCheck>]) -> Vec<Issue> {
    let mut issues: Vec<Issue> = Vec::new();
    for check in checks {
        issues.extend(check.evaluate(ledger));
    }
    issues.sort();
    issues
}

// ═══════════════════════════════════════════════════════════════════════════
// fail-closed 的终态
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐⭐ fail-closed 终态（`release.py:520-527` 的四条理由归并为三态）。
///
/// 原文列了四种 `None`：*a market answer observed no price* /
/// *an issue cannot be cut* / *a flagged figure cannot be located* /
/// *the text still fails*。
///
/// ⛔ **归并说明（诚实边界）**：中间两条在本层**是同一件事**（切不掉 = 定位不到），
/// 故合并为一个 [`RefusalReason::IssueNotCuttable`]。**第一条与第三条**在本层
/// **是两个不同的事实**，别把它们说成一件：上游那条 *a market answer observed no
/// price* 讲的是**行情本身没报价**，本层无从得知（[`Ledger`] 里没有任何行情句柄）；
/// 本层能观测到的只是「切割把声明清空」这一**自己的动作**。⇒ 因此**不沿用上游
/// 那个名字**：本模块第一版把它叫 `NoPriceObserved`，让调用方读成一个关于
/// **行情**的结论，实际含义是「我们把每一条声明都切了」。同名不同事实是最难在
/// 事后发现的错误，故按事实改名。
///
/// | 终态 | 含义（说的是**本层自己的动作**，不是行情） |
/// |---|---|
/// | [`RefusalReason::AllClaimsCut`] | 切割把**所有**声明都删了 ⇒ 剩下的文本里一个可核查的断言都没有 ⇒ 无内容可发布 |
/// | [`RefusalReason::IssueNotCuttable`] | 某条被标记的 figure 在当前文本里**定位不到**（跨度越界 / 非字符边界 / 跨度起点不是该 figure） |
/// | [`RefusalReason::RemainderStillFails`] | 切完用**同一道门**复验，仍有阻断问题 |
///
/// ⛔ 三者都**不携带任何可发布文本**（[`Refusal`] 里压根没有 text 字段）——
/// 这是 fail-closed 的类型形态，不是注释约定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalReason {
    /// 切完一条声明都不剩 —— 无内容可发布。**不是**「行情没报价」。
    AllClaimsCut,
    /// 被标记的 figure 定位不到 / 切不掉。
    IssueNotCuttable,
    /// 切后复验仍有阻断问题。
    RemainderStillFails,
}

/// 全部终态（`--list` 用的穷举清单）。
pub const REFUSAL_REASONS: [RefusalReason; 3] = [
    RefusalReason::AllClaimsCut,
    RefusalReason::IssueNotCuttable,
    RefusalReason::RemainderStillFails,
];

impl RefusalReason {
    /// 稳定标签（进报告；跨版本不许改，故 [`RefusalReason::AllClaimsCut`] 的
    /// 标签是 `ALL_CLAIMS_CUT` 而不是第一版的 `NO_PRICE_OBSERVED` —— 后者是
    /// **错标签**，改名会让旧日志里的那一行**对不上**，这是有意的）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RefusalReason::AllClaimsCut => "ALL_CLAIMS_CUT",
            RefusalReason::IssueNotCuttable => "ISSUE_NOT_CUTTABLE",
            RefusalReason::RemainderStillFails => "REMAINDER_STILL_FAILS",
        }
    }

    /// ⭐ 穷举清单。
    #[must_use]
    pub fn all() -> &'static [RefusalReason] {
        &REFUSAL_REASONS
    }
}

impl fmt::Display for RefusalReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 修订预算：由**调用方**消费，本层只报告
// ═══════════════════════════════════════════════════════════════════════════

/// 原作的修订预算上限（`release.py:32`）。
pub const MAX_GROUNDING_REVISIONS: u32 = 2;

/// ⭐⭐ 有界修订预算。
///
/// ⛔ 本层**永不**修改它：`admit` 只拿 `&RevisionBudget`，唯一的写入口
/// [`RevisionBudget::consume`] 是 `&mut self`，只有**调用方**（那个跑修订轮、
/// 手里有工具权限的编排器）能调。这与「修订轮是纯文本、工具被扣住」是同一件事
/// 的两半：**权限在该层之外，预算的扣减也该在该层之外**。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RevisionBudget {
    max_revisions: u32,
    consumed: u32,
}

impl RevisionBudget {
    /// 新建预算（`max_revisions = 0` ⇒ 从不请求修订，直接降级）。
    #[must_use]
    pub fn new(max_revisions: u32) -> Self {
        Self { max_revisions, consumed: 0 }
    }

    /// 「不允许修订」的预算（一步到降级）。
    #[must_use]
    pub fn none() -> Self {
        Self::new(0)
    }

    /// 上限。
    #[must_use]
    pub fn max_revisions(&self) -> u32 {
        self.max_revisions
    }

    /// 已消费。
    #[must_use]
    pub fn consumed(&self) -> u32 {
        self.consumed
    }

    /// ⭐ 剩余（饱和减）。
    #[must_use]
    pub fn remaining(&self) -> u32 {
        self.max_revisions.saturating_sub(self.consumed)
    }

    /// ⭐ 是否已耗尽。
    #[must_use]
    pub fn exhausted(&self) -> bool {
        self.remaining() == 0
    }

    /// ⭐⭐ 消费一格。**返回是否成功** —— 耗尽后调用方能看见失败，而不是
    /// 静默超支（超支的修订轮等于「有界」这个说法失效）。
    pub fn consume(&mut self) -> bool {
        if self.exhausted() {
            return false;
        }
        self.consumed = self.consumed.saturating_add(1);
        true
    }

    /// 本预算的内容地址。
    #[must_use]
    pub fn digest(&self) -> u64 {
        Digest::new()
            .section("revision_budget")
            .variant(RELEASE_SCHEMA_V)
            .field_u64(u64::from(self.max_revisions))
            .field_u64(u64::from(self.consumed))
            .finish()
    }
}

impl Default for RevisionBudget {
    fn default() -> Self {
        Self::new(MAX_GROUNDING_REVISIONS)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 三种结局
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐⭐⭐ **释放文本**：裁决与文本**捆绑**的载体。
///
/// 字段私有 + [`ReleasedText::degraded`] **从 `claims_cut` 派生** ⇒ 标志与
/// 事实不可能分叉（不能「声明降级却没删任何东西」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleasedText {
    text: String,
    claims_kept: usize,
    claims_cut: usize,
    cut_codes: Vec<String>,
    ledger_digest: u64,
}

impl ReleasedText {
    /// ⭐⭐ 可发布文本（**原文**，无任何声明被切时逐字节相同）。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// ⭐ 是否降级发布 —— **派生于** `claims_cut > 0`，不是独立字段。
    #[must_use]
    pub fn degraded(&self) -> bool {
        self.claims_cut > 0
    }

    /// 保留下来的声明条数。
    #[must_use]
    pub fn claims_kept(&self) -> usize {
        self.claims_kept
    }

    /// 被切掉的声明条数。
    #[must_use]
    pub fn claims_cut(&self) -> usize {
        self.claims_cut
    }

    /// 触发了切割的检查码（已排序去重）。
    #[must_use]
    pub fn cut_codes(&self) -> &[String] {
        &self.cut_codes
    }

    /// 被发布那一版文本的账本身份（**不是**原文的 —— 身份必须指向被裁决的东西）。
    #[must_use]
    pub fn ledger_digest(&self) -> u64 {
        self.ledger_digest
    }

    /// 文本为空（fail-closed 的副产物，不单独作为终态）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// ⭐ 裁决 + 文本的审计身份。
    #[must_use]
    pub fn digest(&self) -> u64 {
        let code_keys = sorted_keys(
            self.cut_codes.iter().map(|c| format!("{c}")).collect::<Vec<String>>(),
        );
        Digest::new()
            .section("release")
            .variant(RELEASE_SCHEMA_V)
            .variant(if self.degraded() { 1 } else { 0 })
            .field_str(&self.text)
            .field_u64(self.claims_kept as u64)
            .field_u64(self.claims_cut as u64)
            .field_sorted_keys(&code_keys)
            .field_u64(self.ledger_digest)
            .finish()
    }

    /// 渲染（纯函数，无 IO）。
    #[must_use]
    pub fn render(&self) -> String {
        let tag = if self.degraded() { "DEGRADED" } else { "CLEAN" };
        format!(
            "{tag}: kept={} cut={} codes=[{}] ledger={:016x}\n{}",
            self.claims_kept,
            self.claims_cut,
            self.cut_codes.join(","),
            self.ledger_digest,
            self.text
        )
    }
}

/// ⭐ 「需要一轮修订」：**还没有**可发布文本，只有待修的问题行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revision {
    issues: Vec<Issue>,
    max_revisions: u32,
    consumed: u32,
    ledger_digest: u64,
}

impl Revision {
    /// 待修问题行（已排序）。
    #[must_use]
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// 阻断问题条数（`not_evaluable` 不计）。
    #[must_use]
    pub fn blocking_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_blocking()).count()
    }

    /// 不可判定条数。
    #[must_use]
    pub fn not_evaluable_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_not_evaluable()).count()
    }

    /// 预算上限。
    #[must_use]
    pub fn max_revisions(&self) -> u32 {
        self.max_revisions
    }

    /// 已消费。
    #[must_use]
    pub fn consumed(&self) -> u32 {
        self.consumed
    }

    /// ⭐ 剩余。
    #[must_use]
    pub fn revisions_remaining(&self) -> u32 {
        self.max_revisions.saturating_sub(self.consumed)
    }

    /// ⭐ 是否还允许再修订一轮。
    #[must_use]
    pub fn may_revise(&self) -> bool {
        self.revisions_remaining() > 0
    }

    /// 原文账本身份。
    #[must_use]
    pub fn ledger_digest(&self) -> u64 {
        self.ledger_digest
    }
}

/// ⭐⭐ **拒答**。⛔ **没有任何 text 访问器** —— 终态就是终态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    reason: RefusalReason,
    detail: String,
    issues: Vec<Issue>,
    ledger_digest: u64,
}

impl Refusal {
    /// 拒答理由（穷举于 [`RefusalReason::all`]）。
    #[must_use]
    pub fn reason(&self) -> RefusalReason {
        self.reason
    }

    /// 人读说明。
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// 拒答时的问题行（**保留**以便排错；不是可发布文本）。
    #[must_use]
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// 原文账本身份。
    #[must_use]
    pub fn ledger_digest(&self) -> u64 {
        self.ledger_digest
    }
}

/// ⭐⭐⭐ 三种结局。
///
/// | 变体 | 何时 | 带着什么 |
/// |---|---|---|
/// | [`Release::Released`] | 通过，或降级后通过 | **可发布文本**（ [`ReleasedText`] ） |
/// | [`Release::NeedsRevision`] | 有阻断问题**且预算未耗尽** | 待修问题行 + 剩余预算 |
/// | [`Release::Refused`] | 降级后仍不可发布 | **无文本**，只有理由 |
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Release {
    /// 可发布（可能被降级）。
    Released(ReleasedText),
    /// 还要一轮修订（修订轮是纯文本的，工具被扣住 —— 那是调用方的事）。
    NeedsRevision(Revision),
    /// 拒答（fail-closed）。
    Refused(Refusal),
}

/// ⭐⭐⭐ **裁决 —— 变体里直接持有结局载荷**。
///
/// ⛔ 这是需求「**不可能只拿到裁决而拿不到释放文本**」的落地形态：
/// `ReleaseVerdict::Released` 携带的是 `&ReleasedText`，**不是**一个
/// `bool`。想给上游报「已发布」，就必须把文本一起交出去 —— 上游无法
/// 拿到一个不含文本的「已发布」。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseVerdict<'r> {
    /// 可发布（载荷即文本）。
    Released(&'r ReleasedText),
    /// 还要一轮修订（载荷即修订请求）。
    NeedsRevision(&'r Revision),
    /// 拒答（载荷即理由）。
    Refused(&'r Refusal),
}

impl<'r> ReleaseVerdict<'r> {
    /// ⭐ 取释放文本。**只有发布态才是 `Some`**。
    #[must_use]
    pub fn released_text(self) -> Option<&'r ReleasedText> {
        match self {
            ReleaseVerdict::Released(rt) => Some(rt),
            _ => None,
        }
    }

    /// ⭐ 是否降级发布。
    #[must_use]
    pub fn degraded(self) -> bool {
        self.released_text().is_some_and(ReleasedText::degraded)
    }

    /// ⭐ 是否**干净**发布（一字未改）。
    #[must_use]
    pub fn is_clean(self) -> bool {
        self.released_text().is_some_and(|rt| !rt.degraded())
    }

    /// 稳定标签（`CLEAN` / `DEGRADED` / `NEEDS_REVISION` / `REFUSED`）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ReleaseVerdict::Released(rt) => {
                if rt.degraded() {
                    "DEGRADED"
                } else {
                    "CLEAN"
                }
            }
            ReleaseVerdict::NeedsRevision(_) => "NEEDS_REVISION",
            ReleaseVerdict::Refused(_) => "REFUSED",
        }
    }
}

impl fmt::Display for ReleaseVerdict<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Release {
    /// ⭐ 取可发布文本。
    #[must_use]
    pub fn released_text(&self) -> Option<&ReleasedText> {
        match self {
            Release::Released(rt) => Some(rt),
            _ => None,
        }
    }

    /// ⭐⭐ 裁决**捆绑**结局载荷。
    #[must_use]
    pub fn verdict(&self) -> ReleaseVerdict<'_> {
        match self {
            Release::Released(rt) => ReleaseVerdict::Released(rt),
            Release::NeedsRevision(rv) => ReleaseVerdict::NeedsRevision(rv),
            Release::Refused(rf) => ReleaseVerdict::Refused(rf),
        }
    }

    /// 是否可发布（**恒等价于** [`Release::released_text`] 为 `Some`）。
    #[must_use]
    pub fn is_released(&self) -> bool {
        self.released_text().is_some()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ⭐⭐ 发布状态机：一趟算完裁决 + 释放文本
// ═══════════════════════════════════════════════════════════════════════════

/// ⭐⭐ 便利入口：解析 + 门控（`parse → validate → …`）。
#[must_use]
pub fn admit(
    text: &str,
    checks: &[Box<dyn ProvenanceCheck>],
    budget: &RevisionBudget,
) -> Release {
    let claims = parse_declarations(text);
    admit_claims(text, &claims, checks, budget)
}

/// ⭐⭐⭐ 状态机本体（`release.py:515-527` 的顺序）。
///
/// ```text
///   parse(claims)
///        │
///        ▼
///   validate ──无阻断──────────────▶ Released{degraded:false, 原文逐字}
///        │有阻断
///        ▼
///   budget.remaining() > 0 ? ─是──▶ NeedsRevision{issues, remaining}   ← 调用方 consume()
///        │否
///        ▼
///   切掉全部被标记的 figure 跨度
///        ├─ 定位不到 ────────────▶ Refused{ISSUE_NOT_CUTTABLE}
///        ├─ 一条不剩 ────────────▶ Refused{ALL_CLAIMS_CUT}
///        ▼
///   re-parse(cut_text) + validate（**同一道门**）
///        ├─ 仍有阻断 ────────────▶ Refused{REMAINDER_STILL_FAILS}
///        ▼
///   Released{degraded:true, cut_text}
/// ```
///
/// ⛔ **不允许「干净地」拒答**：`Refused` 里没有文本，所以上游拿不到一段
/// 「未经校验的原文」当降级 —— fail-closed 是类型形态。
///
/// ⛔ 切后**必须重新解析**：跨度是**位置**，切一刀之后所有后续跨度都会漂移。
/// 复用旧 `claims` 去切新文本，是本层最容易写出的 LIVE bug。
///
/// ⛔ 顺带：重解析用的是**全量** `parse_declarations(cut_text)`，因此
/// `REMAINDER_STILL_FAILS` 有一条**设计上的**触发场景：**调用方少报了账本**
/// （见模块头）。字节拼接**也可能**造成同样的结果 —— 本层**不**声称那不可能。
#[must_use]
pub fn admit_claims(
    text: &str,
    claims: &[DeclaredClaim],
    checks: &[Box<dyn ProvenanceCheck>],
    budget: &RevisionBudget,
) -> Release {
    let ledger = Ledger::new(text, claims);
    let ledger_digest = ledger.digest();
    let issues = validate(&ledger, checks);

    if !issues.iter().any(Issue::is_blocking) {
        return Release::Released(ReleasedText {
            text: text.to_owned(),
            claims_kept: claims.len(),
            claims_cut: 0,
            cut_codes: Vec::new(),
            ledger_digest,
        });
    }

    // 有阻断问题 ⇒ 先看预算。**修订轮必须在切割之前**（原作顺序：
    // 校验 → 修补 → 有界修订 → 才轮到切）。
    if !budget.exhausted() {
        return Release::NeedsRevision(Revision {
            issues,
            max_revisions: budget.max_revisions(),
            consumed: budget.consumed(),
            ledger_digest,
        });
    }

    match degrade(text, claims, &issues, checks) {
        Ok(rt) => Release::Released(rt),
        Err((reason, detail)) => {
            Release::Refused(Refusal { reason, detail, issues, ledger_digest })
        }
    }
}

/// 降级：切掉被标记的声明，用同一道门复验剩余文本。
fn degrade(
    text: &str,
    claims: &[DeclaredClaim],
    issues: &[Issue],
    checks: &[Box<dyn ProvenanceCheck>],
) -> Result<ReleasedText, (RefusalReason, String)> {
    // ── 阶段 1：逐条定位（★ fail-closed 的 ISSUE_NOT_CUTTABLE 在此产生）────
    let mut cut_indices: BTreeSet<usize> = BTreeSet::new();
    let mut ranges: Vec<Span> = Vec::new();
    let mut codes: Vec<String> = Vec::new();
    for issue in issues.iter().filter(|i| i.is_blocking()) {
        let idx = issue.claim_index();
        let Some(claim) = claims.get(idx) else {
            return Err((
                RefusalReason::IssueNotCuttable,
                format!(
                    "issue `{}` points at claim #{idx}, which is not in the ledger: a flagged \
                     figure that cannot be located cannot be removed, so nothing is released",
                    issue.code()
                ),
            ));
        };
        let span = claim.cut_span();
        let Some(slice) = span.slice(text) else {
            return Err((
                RefusalReason::IssueNotCuttable,
                format!(
                    "figure `{}` claims span {} which does not slice this text (out of bounds or \
                     not a character boundary)",
                    claim.figure(),
                    span.render()
                ),
            ));
        };
        if !slice.starts_with(claim.figure()) {
            return Err((
                RefusalReason::IssueNotCuttable,
                format!(
                    "figure `{}` claims span {} but that span starts with `{}`: the flagged \
                     figure is not where it was declared, so cutting it would silently delete \
                     unrelated text",
                    claim.figure(),
                    span.render(),
                    slice.chars().take(16).collect::<String>()
                ),
            ));
        }
        ranges.push(span);
        cut_indices.insert(idx);
        codes.push(issue.code().to_owned());
    }

    // ── 阶段 2：切空 ⇒ 无内容可发布（ALL_CLAIMS_CUT）──────────────────────────
    // ⛔ 说的是**本层自己的动作**（每条声明都被切了），不是「行情没报价」。
    //    第一版把这个终态叫 `NoPriceObserved`，让读日志的人以为系统去看过行情。
    if cut_indices.len() == claims.len() {
        return Err((
            RefusalReason::AllClaimsCut,
            format!(
                "all {} declared claim(s) are flagged: the released text would carry no \
                 verifiable assertion at all, which is a refusal, not an empty answer",
                claims.len()
            ),
        ));
    }

    // ── 阶段 3：切（与给出的顺序无关 ⇒ 逆序输入同结果）────────────────────
    let cut_text = apply_cuts(text, &ranges);

    // ── 阶段 4：用**同一道门**复验剩余文本（★ 必须**全量**重新解析）────────
    // ⛔ 这里刻意**不**复用调用方的 `claims`：调用方少报的那条阻断声明，恰好要
    //    在这一步被抓出来 —— 这就是 `REMAINDER_STILL_FAILS` 诚实的触发场景。
    //    钉住它的测试：`an_under_reported_ledger_is_caught_by_the_mandatory_recheck`。
    let rest_claims = parse_declarations(&cut_text);
    let rest_ledger = Ledger::new(&cut_text, &rest_claims);
    let rest_digest = rest_ledger.digest();
    let rest_issues = validate(&rest_ledger, checks);
    let survivors: Vec<&Issue> = rest_issues.iter().filter(|i| i.is_blocking()).collect();
    if let Some(first) = survivors.first() {
        return Err((
            RefusalReason::RemainderStillFails,
            format!(
                "{} blocking issue(s) survive the cut (first: `{}` on claim#{} `{}`): the \
                 remaining text still fails the same gate, so it is not released",
                survivors.len(),
                first.code(),
                first.claim_index(),
                first.figure()
            ),
        ));
    }

    let mut unique_codes = codes;
    unique_codes.sort();
    unique_codes.dedup();
    Ok(ReleasedText {
        text: cut_text,
        claims_kept: rest_claims.len(),
        claims_cut: cut_indices.len(),
        cut_codes: unique_codes,
        ledger_digest: rest_digest,
    })
}

/// ⭐ 按跨度切文本。**顺序无关**：先排序再合并 ⇒ 调用方给正序、逆序、
/// 重复跨度都得到同一结果（与 `sorted_keys` 纪律同源）。
///
/// ⛔ **已知的输出质量欠账（本层最弱的一处取舍，刻意留在此处可见）**：
/// 切割**精确到字节**，只挖掉 `figure[role=…]`，因此句子里的引导词会留下孤儿：
/// `bench 9.90[role=cited]` ⇒ 留下 `"bench "`（人读起来像坏掉的散文）。
/// 正确的做法是**行级切 + 空白塌缩**（整行删掉，并收拢空行）；**本层没做**，
/// 因为它不是「顺手改一下」：
///
/// 1. 它要求一套**行模型**（行首/行尾偏移、行内多 figure、围栏跨行）—— 现有的
///    [`Span`] 是纯字节跨度，够不着「这一行」这个概念；
/// 2. 它会**改变** [`Span`] / [`apply_cuts`] 的契约与本层的输出语义，而不只是
///    换个切点：删整行会同时改掉 [`Ledger::digest`] 与 [`ReleasedText::digest`]
///    的输入（测试 `digest_is_order_independent_and_discriminating` 已经把这点
///    写明：「尾部多一个换行 ⇒ 换摘要」）；
/// 3. 「切整行」与「只切被标记的声明」是两个不同的产品决策（例如一行里两个
///    figure，一个合规一个违规时，整行删会把合规的内容也删掉）。
///
/// ⇒ 取舍：**正确但字节精确的切法 ≫ 破坏行的切法**。要改，先决定上面第 3 条。
/// 现状由 `one_bad_claim_releases_the_rest_with_degraded_set` 逐字节钉住。
#[must_use]
fn apply_cuts(text: &str, ranges: &[Span]) -> String {
    let mut sorted: Vec<Span> = ranges.iter().copied().filter(|s| !s.is_empty()).collect();
    sorted.sort();
    let mut merged: Vec<Span> = Vec::new();
    for span in sorted {
        match merged.last_mut() {
            Some(last) if span.start() <= last.end() => {
                let m = last.merge(&span);
                *last = m;
            }
            _ => merged.push(span),
        }
    }
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    let limit = text.len();
    for span in &merged {
        let start = span.start().min(limit);
        let end = span.end().min(limit);
        if let Some(head) = text.get(cursor..start) {
            out.push_str(head);
        }
        cursor = cursor.max(end);
    }
    if let Some(tail) = text.get(cursor..limit) {
        out.push_str(tail);
    }
    out
}

// ═══════════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── 夹具 ─────────────────────────────────────────────────────────────

    /// 一条**干净**文本：五个角色齐、各角色的载荷齐。
    ///
    /// ⚠️ 两条 `1437` 是**故意重复**的：id 分别是 `1437#0` / `1437#1`，用来
    /// 验 occurrence 与 [`Ledger::claim_by_id`] 的 id 寻址。
    const CLEAN: &str = "open 187.40[role=observed]\n\
margin 4.20[role=derived formula=(high-low)/open]\n\
view 9.90[role=proposed rationale=momentum]\n\
src 12.00[role=cited source=HKEX-2026-09-30]\n\
rows 1437[role=count basis=rows_scanned]\n\
n 1437[role=count basis=rows_scanned]\n";

    /// 一条声明了 `cited` 却**没给来源**的文本（可修复：切掉即可）。
    const ONE_BAD_CITED: &str = "open 187.40[role=observed]\nbench 9.90[role=cited]\n";

    /// 同一个 figure 文本出现两次、第二条无来源 ⇒ 用来钉 id 寻址与顺序无关。
    const SAME_FIGURE_TWICE: &str =
        "x 9.9[role=cited source=hkex]\ny 9.9[role=cited]\n";

    /// ⭐ 三条声明：一条合规 `count`、一条违规 `derived`（无口径）、一条违规
    /// `cited`（无来源）。**完整解析时**后两条一起被切 ⇒ 可降级发布。
    const UNDER_REPORTED: &str = "p 5.0[role=count basis=rows]\n\
a 1.0[role=derived]\n\
b 2.0[role=cited]\n";

    fn one_bad() -> Release {
        admit(ONE_BAD_CITED, &default_checks(), &RevisionBudget::none())
    }

    fn released_of(rel: &Release) -> &ReleasedText {
        // 测试辅助：用 `match` 而不是 `unwrap` —— 非测试代码禁 `unwrap`，
        // 测试代码也一并自律，免得手滑复制到生产路径。
        match rel.released_text() {
            Some(rt) => rt,
            None => panic!("expected a release, got {rel:?}"),
        }
    }

    // ── 解析：五个角色 ────────────────────────────────────────────────────

    #[test]
    fn parse_reads_all_five_roles() {
        let claims = parse_declarations(CLEAN);
        let roles: Vec<Option<ProvenanceRole>> = claims.iter().map(DeclaredClaim::role).collect();
        assert_eq!(
            roles,
            vec![
                Some(ProvenanceRole::Observed),
                Some(ProvenanceRole::Derived),
                Some(ProvenanceRole::Proposed),
                Some(ProvenanceRole::Cited),
                Some(ProvenanceRole::Count),
                Some(ProvenanceRole::Count),
            ],
            "五个角色必须逐个解出（第六条是重复的 count，验 occurrence）"
        );
        assert_eq!(claims.len(), 6);
    }

    #[test]
    fn role_names_round_trip_through_the_declared_set() {
        for role in ROLES {
            assert_eq!(ProvenanceRole::parse(role.as_str()), Some(role), "{}", role.as_str());
        }
        assert_eq!(ProvenanceRole::parse("OBSERVED"), None, "角色名大小写敏感");
        assert_eq!(ProvenanceRole::parse(""), None);
        assert_eq!(ProvenanceRole::parse("derived "), None, "不静默 trim");
    }

    // ── 解析：五种形状 ────────────────────────────────────────────────────

    #[test]
    fn parse_recognises_all_five_shapes() {
        let text = "n 187.40[role=observed]\n\
d 2026-09-30[role=observed]\n\
o 3rd[role=observed]\n\
i #42[role=observed]\n\
```rs[role=proposed]\nfn main() {}\n```\n";
        let claims = parse_declarations(text);
        let shapes: Vec<Option<FigureShape>> = claims.iter().map(DeclaredClaim::shape).collect();
        assert_eq!(
            shapes,
            vec![
                Some(FigureShape::Number),
                Some(FigureShape::Date),
                Some(FigureShape::Ordinal),
                Some(FigureShape::IndexCell),
                Some(FigureShape::Fence),
            ]
        );
        // 形状表本身也是声明的：五种不多不少。
        assert_eq!(FIGURE_SHAPES.len(), 5);
    }

    #[test]
    fn fence_claim_span_covers_the_whole_block() {
        let text = "head\n```rs[role=proposed]\nfn main() {}\n```\ntail\n";
        let claims = parse_declarations(text);
        assert_eq!(claims.len(), 1);
        let span = claims[0].cut_span();
        let slice = span.slice(text).unwrap_or("");
        assert!(slice.starts_with("```rs"), "跨度必须从围栏头开始");
        assert!(slice.ends_with("```"), "跨度必须含围栏尾，实际={slice:?}");
        assert!(slice.contains("fn main()"), "围栏内的代码必须落在跨度内");
    }

    #[test]
    fn unterminated_fence_spans_to_the_end_instead_of_vanishing() {
        let text = "head\n```rs[role=proposed]\nfn main() {}\n";
        let claims = parse_declarations(text);
        assert_eq!(claims.len(), 1, "未闭合围栏仍是声明（诚实降级，不是静默丢弃）");
        assert_eq!(claims[0].cut_span().end(), text.len());
    }

    // ── 解析：全函数 + 定位不变量 ─────────────────────────────────────────

    /// ⭐⭐ **定位不变量**：解析器产出的每条声明，其切分跨度都能切出来，且**从
    /// figure 开始**。这条不变量让 fail-closed 的 `ISSUE_NOT_CUTTABLE` 只能由
    /// **手造 / 外部**声明触发 —— 正是它该被触发的场景。
    #[test]
    fn parsed_claims_are_always_relocatable() {
        let nasty = [
            "",
            "[",
            "]",
            "[]",
            "[role=]",
            "[role",
            "1[role=",
            "1[role=a",
            "1[role=a b",
            "1[role=a b=",
            "1[role=a b=c",
            "1[role=a b=c]",
            "[role=a]1",
            "1[]",
            "1[role=]",
            "中文[role=observed]中文",
            "1[role=observed][bad",
            "```[role=proposed]",
            "``[role=proposed]",
            "`[role=proposed]",
            "1[role=多字节]",
            "1[role=observed formula=]",
            "1 2 3 4[role=count basis=x]",
            "((((((((((1[role=count basis=x]",
            "\u{1f600}[role=observed]",
            "-1.5%[role=derived formula=]",
            "1e5[role=observed]",
            "1,234[role=observed]",
        ];
        for text in nasty {
            let claims = parse_declarations(text);
            for claim in &claims {
                let slice = claim
                    .cut_span()
                    .slice(text)
                    .unwrap_or_else(|| panic!("span {} of `{text}` does not slice", claim.cut_span()));
                assert!(
                    slice.starts_with(claim.figure()),
                    "`{text}`: span {} yields {slice:?} which does not start with `{}`",
                    claim.cut_span(),
                    claim.figure()
                );
                // ⭐ 完整解析出的注解**必须**解出角色原文（可能是未知角色）。
                // 但角色**可以**解不出 —— 这正是「不猜」的那一格。
                assert_eq!(
                    claim.is_well_formed(),
                    !claim.raw_role().is_empty(),
                    "`{text}`: 畸形标记与角色原文必须一致"
                );
                assert_eq!(
                    claim.role().is_some(),
                    ProvenanceRole::parse(claim.raw_role()).is_some(),
                    "`{text}`: 角色必须等于对原文的严格解析"
                );
            }
        }
    }

    #[test]
    fn multi_byte_text_never_splits_a_char() {
        // 每个字节位置都试一遍：跨度若切进 UTF-8 续字节，`slice` 必须是 None。
        let text = "价格 187.40[role=observed] 元";
        for start in 0..=text.len() {
            for end in start..=text.len() {
                let _ = Span::new(start, end).slice(text);
            }
        }
        let claims = parse_declarations(text);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].figure(), "187.40");
    }

    // ── 畸形 / 未知角色 ⇒ not_evaluable，绝不 pass ────────────────────────

    #[test]
    fn unknown_role_is_not_evaluable_and_never_blocks() {
        let text = "ok 1.5[role=observed]\nbad 2.5[role=bogus]\n";
        let claims = parse_declarations(text);
        assert_eq!(claims[1].role(), None, "未知角色不得回退到任何默认角色");
        assert_eq!(claims[1].raw_role(), "bogus", "原文保留供排错");

        let ledger = Ledger::new(text, &claims);
        let issues = validate(&ledger, &default_checks());
        let unknown: Vec<&Issue> =
            issues.iter().filter(|i| i.claim_index() == 1).collect();
        assert!(!unknown.is_empty(), "每条检查都必须对不可知角色表态");
        assert!(
            unknown.iter().all(|i| i.is_not_evaluable()),
            "未知角色只能 not_evaluable（实测 {:?}）",
            unknown.iter().map(|i| i.render()).collect::<Vec<_>>()
        );
        assert!(
            !issues.iter().any(Issue::is_blocking),
            "未知角色不得造成阻断"
        );
    }

    #[test]
    fn malformed_annotation_is_not_evaluable_and_does_not_block() {
        for text in ["1.5[role observed]\n", "2.5[role=\n", "3.5[role=derived\n", "4.5[=observed]\n"] {
            let claims = parse_declarations(text);
            assert_eq!(claims.len(), 1, "`{text}` 应产出一条声明");
            assert!(!claims[0].is_well_formed(), "`{text}` 应被标为畸形");
            assert_eq!(claims[0].role(), None);
            let ledger = Ledger::new(text, &claims);
            let issues = validate(&ledger, &default_checks());
            assert!(
                issues.iter().all(Issue::is_not_evaluable),
                "`{text}` 全部行必须 not_evaluable（实测 {:?}）",
                issues.iter().map(Issue::render).collect::<Vec<_>>()
            );
        }
    }

    // ── 检查一：`derived` 必须给公式 ─────────────────────────────────────

    #[test]
    fn derived_without_formula_blocks_and_with_formula_is_silent() {
        let registry: Vec<Box<dyn ProvenanceCheck>> = vec![Box::new(DerivedFormulaCheck::new())];

        let bad = parse_declarations("m 4.20[role=derived]\n");
        let issues = validate(&Ledger::new("m 4.20[role=derived]\n", &bad), &registry);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code(), CODE_DERIVED_FORMULA);
        assert!(issues[0].is_blocking());

        let good_text = "m 4.20[role=derived formula=(high-low)/open]\n";
        let good = parse_declarations(good_text);
        let issues = validate(&Ledger::new(good_text, &good), &registry);
        assert!(issues.is_empty(), "合规声明必须完全静默，实际 {:?}", issues);
    }

    #[test]
    fn derived_with_empty_formula_still_blocks() {
        let text = "m 4.20[role=derived formula=]\n";
        let claims = parse_declarations(text);
        let issues = validate(&Ledger::new(text, &claims), &default_checks());
        assert!(
            issues.iter().any(|i| i.code() == CODE_DERIVED_FORMULA && i.is_blocking()),
            "空串口径也是「没给公式」（实测 {:?}）",
            issues.iter().map(Issue::render).collect::<Vec<_>>()
        );
    }

    // ── 检查二：`cited` 必须给来源 ────────────────────────────────────────

    #[test]
    fn cited_without_source_blocks_and_with_source_is_silent() {
        let registry: Vec<Box<dyn ProvenanceCheck>> = vec![Box::new(CitedSourceCheck::new())];

        let bad_text = "b 9.90[role=cited]\n";
        let bad = parse_declarations(bad_text);
        let issues = validate(&Ledger::new(bad_text, &bad), &registry);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code(), CODE_CITED_SOURCE);
        assert!(issues[0].is_blocking());

        let good_text = "b 9.90[role=cited source=HKEX-2026-09-30]\n";
        let good = parse_declarations(good_text);
        assert!(validate(&Ledger::new(good_text, &good), &registry).is_empty());
    }

    #[test]
    fn checks_are_scoped_to_their_own_role() {
        // `derived` 检查不该对 `cited` 指手画脚，反之亦然。
        let text = "x 9.90[role=cited]\ny 4.20[role=derived]\n";
        let claims = parse_declarations(text);
        let ledger = Ledger::new(text, &claims);
        let only_derived: Vec<Box<dyn ProvenanceCheck>> = vec![Box::new(DerivedFormulaCheck::new())];
        let issues = validate(&ledger, &only_derived);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].claim_index(), 1, "只有 claim#1 是 derived");
    }

    // ── `observed`：没有内建规则，且**这是诚实的** ────────────────────────

    /// ⭐ 第一版这里有一条要求 `witness=` 的检查。删掉它之后，`role=observed`
    /// 在默认注册表下**不产生任何问题行** —— 这不是「漏查」，而是**没有依据**：
    /// 要求一条观测携带旁证，就得先有一个「旁证」协议，而上游没有这个字段。
    /// ⛔ 别为了「让 `observed` 也被管一管」而凭空发明一个键。
    #[test]
    fn an_observed_claim_is_silent_because_no_rule_was_invented_for_it() {
        for text in [
            "open 187.40[role=observed]\n",
            // 一个**已删除协议的残留**载荷：现在它只是载荷区里的一个未知键，
            // 既不被要求、也不被判违规 —— 协议里未知键一律放过。
            "open 187.40[role=observed witness=nobody#7]\n",
        ] {
            let claims = parse_declarations(text);
            assert_eq!(claims.len(), 1, "`{text}` 应产出一条声明");
            assert_eq!(claims[0].role(), Some(ProvenanceRole::Observed));
            let issues = validate(&Ledger::new(text, &claims), &default_checks());
            assert!(
                issues.is_empty(),
                "`observed` 无内建规则 ⇒ 零问题行（实测 {:?}）",
                issues.iter().map(Issue::render).collect::<Vec<_>>()
            );
            assert!(!issues.iter().any(Issue::is_blocking));
        }
    }

    /// ⭐ 删掉第三条检查后，默认注册表**恰好**剩两条，且正是模块头承诺的两条。
    /// 这条测试的作用是**让「悄悄加了第三条规则」变成红的** —— 注册表是本层
    /// 唯一能引入规则的地方，它必须是可见的清单，不是隐式行为。
    #[test]
    fn the_default_registry_only_rules_on_derived_and_cited() {
        let codes: Vec<&str> = default_checks().iter().map(|c| c.code()).collect();
        assert_eq!(codes, vec![CODE_DERIVED_FORMULA, CODE_CITED_SOURCE]);
    }

    // ── ⭐ 降级路径 ───────────────────────────────────────────────────────

    #[test]
    fn one_bad_claim_releases_the_rest_with_degraded_set() {
        let rel = one_bad();
        let rt = released_of(&rel);
        assert!(rt.degraded(), "切掉了声明 ⇒ 必须降级发布");
        assert_eq!(rt.claims_cut(), 1);
        assert_eq!(rt.claims_kept(), 1);
        // ⭐ 切割是**精确到字节**的最小手术：只挖掉 `9.90` 及其注解，
        // 行首的引导词 `bench ` 原样留下 —— 行级切 + 空白塌缩是**已知欠账**，
        // `apply_cuts` 的文档里有完整理由（需要行模型 + 会改摘要口径）。
        assert_eq!(rt.text(), "open 187.40[role=observed]\nbench \n");
        assert!(!rt.text().contains("cited"), "不能留下孤儿注解");
        assert_eq!(rt.cut_codes().to_vec(), vec![CODE_CITED_SOURCE.to_owned()]);
        assert!(rel.verdict().degraded());
        assert_eq!(rel.verdict().as_str(), "DEGRADED");
    }

    #[test]
    fn clean_text_releases_byte_identically() {
        let rel = admit(CLEAN, &default_checks(), &RevisionBudget::none());
        let rt = released_of(&rel);
        assert!(!rt.degraded(), "合规文本不得被降级标记");
        assert_eq!(rt.claims_cut(), 0);
        assert_eq!(rt.text(), CLEAN, "干净路径必须逐字节原样释放");
        assert!(rel.verdict().is_clean());
    }

    #[test]
    fn text_without_any_declaration_is_clean_not_refused() {
        // ⭐ 诚实的边界：本层**只管声明过的 figure**。一段没有数字的散文
        // 没有理由被拒 —— 否则这层就成了「不许说不带数字的话」。
        let rel = admit("今天没有可核查的数字。", &default_checks(), &RevisionBudget::none());
        let rt = released_of(&rel);
        assert!(!rt.degraded());
        assert_eq!(rt.claims_kept(), 0);
        assert_eq!(rt.text(), "今天没有可核查的数字。");
    }

    #[test]
    fn degraded_flag_cannot_lie_about_whether_anything_was_cut() {
        let clean_rel = admit(CLEAN, &default_checks(), &RevisionBudget::none());
        let clean = released_of(&clean_rel);
        assert_eq!(clean.degraded(), clean.claims_cut() > 0);
        let dirty_rel = one_bad();
        let dirty = released_of(&dirty_rel);
        assert_eq!(dirty.degraded(), dirty.claims_cut() > 0);
    }

    // ── ⭐ fail-closed ────────────────────────────────────────────────────

    #[test]
    fn every_flagged_claim_cut_leaves_nothing_and_refuses() {
        let text = "a 1.0[role=cited]\nb 2.0[role=derived]\n";
        let rel = admit(text, &default_checks(), &RevisionBudget::none());
        match rel.verdict() {
            ReleaseVerdict::Refused(rf) => {
                assert_eq!(rf.reason(), RefusalReason::AllClaimsCut);
                assert!(!rf.issues().is_empty(), "拒答仍要留问题行供排错");
            }
            other => panic!("必须拒答，实际 {other}"),
        }
        assert!(rel.released_text().is_none(), "拒答**不携带**任何可发布文本");
        // ⭐ 名字说的是**本层自己的动作**：两条声明都被切光了。
        assert_eq!(RefusalReason::AllClaimsCut.to_string(), "ALL_CLAIMS_CUT");
    }

    #[test]
    fn unlocatable_figure_refuses_instead_of_deleting_unrelated_text() {
        // ⭐ 手造一条**跨度撒谎**的声明：跨度指向 "alpha "，figure 却是 "9.99"。
        let text = "alpha 1.5[role=cited] omega\n";
        let lying =
            DeclaredClaim::declared("9.99", Span::new(0, 6), ProvenanceRole::Cited, &[]);
        let rel = admit_claims(text, &[lying], &default_checks(), &RevisionBudget::none());
        match rel.verdict() {
            ReleaseVerdict::Refused(rf) => {
                assert_eq!(rf.reason(), RefusalReason::IssueNotCuttable);
                assert!(rf.detail().contains("not where it was declared"), "{}", rf.detail());
            }
            other => panic!("必须拒答，实际 {other}"),
        }
    }

    #[test]
    fn out_of_bounds_figure_refuses() {
        let text = "short 1.5[role=cited]\n";
        let bogus =
            DeclaredClaim::declared("1.5", Span::new(9000, 9004), ProvenanceRole::Cited, &[]);
        let rel = admit_claims(text, &[bogus], &default_checks(), &RevisionBudget::none());
        match rel.verdict() {
            ReleaseVerdict::Refused(rf) => assert_eq!(rf.reason(), RefusalReason::IssueNotCuttable),
            other => panic!("必须拒答，实际 {other}"),
        }
    }

    #[test]
    fn issue_pointing_outside_the_ledger_refuses() {
        let text = "a 1.5[role=observed]\nb 2.0[role=cited]\n";
        let claims = parse_declarations(text);
        // 对照：正常降级可发布。
        let base = admit_claims(text, &claims, &default_checks(), &RevisionBudget::none());
        assert!(base.released_text().is_some(), "对照：可降级发布");

        // 一条凭空造的问题行：claim#7 不在账本里 ⇒ 切不掉 ⇒ fail-closed。
        let stray = Issue::block(CODE_CITED_SOURCE, 7, &claims[1], "手工造的越界问题");
        let err = degrade(text, &claims, &[stray], &default_checks())
            .err()
            .unwrap_or((RefusalReason::RemainderStillFails, String::new()));
        assert_eq!(err.0, RefusalReason::IssueNotCuttable);
        assert!(err.1.contains("not in the ledger"), "{}", err.1);
    }

    /// ⭐⭐ **什么真正到达 `REMAINDER_STILL_FAILS`：调用方少报了账本。**
    ///
    /// 第一版这条测试是靠虚构的 `witness=` 协议制造「切一刀 ⇒ 新的违规」来
    /// 达到这个分支的。那条路已经删掉（见模块头「诚实台账」）。现在走的是
    /// **真实**的路径，且更有价值：
    ///
    /// * [`admit_claims`] 接受调用方给的 `claims`，第一轮只信它；
    /// * 但切完之后 `degrade` **刻意全量重解析**切后的文本，不复用调用方的集合。
    ///
    /// ⇒ 调用方少报一条阻断声明时，它在重解析那一刻被抓出来 ⇒ 拒答。
    /// 这证明「切完必须复验」不是仪式：复验是**唯一**能发现少报账本的环节。
    #[test]
    fn an_under_reported_ledger_is_caught_by_the_mandatory_recheck() {
        let full = parse_declarations(UNDER_REPORTED);
        assert_eq!(full.len(), 3, "完整解析：三条声明");

        // ── 对照 A：**完整**解析 ⇒ 两条违规一起被切 ⇒ 可降级发布 ──────────
        let complete = admit_claims(UNDER_REPORTED, &full, &default_checks(), &RevisionBudget::none());
        let rt = released_of(&complete);
        assert!(rt.degraded(), "对照 A：切了两条 ⇒ 降级发布");
        assert_eq!(rt.claims_cut(), 2);
        assert_eq!(rt.claims_kept(), 1, "只剩那条合规的 count");

        // ── 对照 B：**少报**一条（不给那条无来源的 cited）⇒ 复验失败 ⇒ 拒答 ─
        // 顺序刻意打乱，证明这里与调用方的排列无关。
        let subset = vec![full[1].clone(), full[0].clone()];
        assert_eq!(subset.len(), 2, "少报一条 ⇒ 只剩两条声明");
        let rel = admit_claims(UNDER_REPORTED, &subset, &default_checks(), &RevisionBudget::none());
        match rel.verdict() {
            ReleaseVerdict::Refused(rf) => {
                assert_eq!(
                    rf.reason(),
                    RefusalReason::RemainderStillFails,
                    "少报的那条违规必须在重解析时被抓出来"
                );
                assert!(rf.detail().contains("survive the cut"), "{}", rf.detail());
                // ⛔ 注意 [`Refusal::issues`] 装的是**切之前**（调用方那一轮）
                // 的问题行 —— 不是复验抓到的那条。这是一个容易读错的细节，
                // 故钉死：这里给的是「触发切割」的问题，不是「剩余仍失败」的问题。
                assert_eq!(rf.issues().len(), 1, "拒答带的是切前的问题行");
                assert_eq!(rf.issues()[0].code(), CODE_DERIVED_FORMULA);
            }
            other => panic!("必须拒答，实际 {other}"),
        }
        assert!(rel.released_text().is_none(), "复验失败 ⇒ 不发文本");

        // ⭐ 同一段文本，**完整**解析 ⇒ 放行。⇒ 拒答的原因只能是「少报」，
        // 不是这段文本本身有别的毛病。
        let parsed = admit(UNDER_REPORTED, &default_checks(), &RevisionBudget::none());
        assert!(
            parsed.verdict().degraded(),
            "对照 C：全量解析同一段文本应当可降级发布（实测 {:?}）",
            parsed.verdict()
        );
    }

    /// ⭐⭐ **语料级不变式：凡本层放行的文本，用同一注册表复验必须零阻断。**
    ///
    /// 这是本层**唯一真正承重**的不变式，也是「删掉虚构协议之后真正剩下的东西」：
    /// 干净路径第一轮就无阻断；降级路径阶段 4 用同一道门复验过。⇒
    /// 「发出去的文本」与「过了门的文本」不可能分家。
    ///
    /// ⛔ 这条测试**只断言上面这一条**，**不**预言任何输入给出哪种裁决 —— 包括
    /// 最后两条围栏畸形形状：那是模块头说的「跨度交叠」那一类，行为取决于解析器
    /// 的 minutiae，把它钉死等于把一个未证明的事实写成契约。
    ///
    /// ⛔ **判别力**：将来若有人加一条「切一刀会制造新违规」的检查、或改坏阶段 4
    /// 的复验，这条会先红 —— 那正是需要有人停下来想一想的时候（而不是像第一版
    /// 那样，反过来发明一个协议）。
    #[test]
    fn every_release_is_revalidated_clean() {
        // 语料：干净 / 单点违规 / 重复 figure / 无声明 / 空串 / 一堆畸形注解 /
        // 围栏（闭合与未闭合）/ 注解畸形**落在围栏声明里**（跨度会交叠的那种）。
        let corpus = [
            CLEAN,
            ONE_BAD_CITED,
            SAME_FIGURE_TWICE,
            UNDER_REPORTED,
            "no declarations here",
            "",
            "中文[role=observed]中文",
            "1[role=a b=c",
            "```[role=proposed]",
            "`[role=proposed]",
            "1[role=derived formula=]",
            "((((((((((1[role=count basis=x]",
            "```rs[role\nx 5.0[role=cited]\n",
            "head\n```rs[role=proposed]\nfn main() {}\n```\ntail\n",
            "```rs[role=junk5.0[role=cited]X[role=cited]",
        ];
        let checks = default_checks();
        let mut released = 0usize;
        for text in corpus {
            let rel = admit(text, &checks, &RevisionBudget::none());
            let Some(rt) = rel.released_text() else {
                continue; // 拒答 / 请求修订：没有文本可放行，不在不变式范围内。
            };
            released = released.saturating_add(1);
            // 复验：用**同一注册表**跑一遍切后（或原文）文本。
            let again = parse_declarations(rt.text());
            let issues = validate(&Ledger::new(rt.text(), &again), &checks);
            assert!(
                !issues.iter().any(Issue::is_blocking),
                "`{text}` 放行的文本复验却失败：{:?}",
                issues.iter().map(Issue::render).collect::<Vec<_>>()
            );
            // 报出来的「保留条数」必须是事实，不是乐观估计。
            assert_eq!(rt.claims_kept(), again.len(), "`{text}` 的 claims_kept 与复解析不符");
            assert_eq!(rt.degraded(), rt.claims_cut() > 0);
        }
        assert!(released > 0, "语料必须至少放行一条，否则这测试什么都没验");
    }

    #[test]
    fn refusal_reasons_are_enumerable_and_total() {
        assert_eq!(REFUSAL_REASONS.len(), 3);
        let labels: Vec<&str> = RefusalReason::all().iter().map(|r| r.as_str()).collect();
        assert_eq!(
            labels,
            vec!["ALL_CLAIMS_CUT", "ISSUE_NOT_CUTTABLE", "REMAINDER_STILL_FAILS"]
        );
    }

    // ── 修订预算 ──────────────────────────────────────────────────────────

    #[test]
    fn budget_reports_remaining_and_stops_at_zero() {
        let mut b = RevisionBudget::new(MAX_GROUNDING_REVISIONS);
        assert_eq!(b.max_revisions(), 2, "release.py:32 的上限是 2");
        assert_eq!(b.remaining(), 2);
        assert!(!b.exhausted());
        assert!(b.consume(), "第 1 格");
        assert_eq!(b.remaining(), 1);
        assert!(b.consume(), "第 2 格");
        assert_eq!(b.remaining(), 0);
        assert!(b.exhausted());
        assert!(!b.consume(), "⛔ 耗尽后必须拒绝继续消费，而不是静默超支");
        assert_eq!(b.remaining(), 0, "拒绝消费后计数不得前进");
    }

    #[test]
    fn budget_remaining_yields_a_revision_request_without_text() {
        let mut b = RevisionBudget::new(MAX_GROUNDING_REVISIONS);
        let rel = admit(ONE_BAD_CITED, &default_checks(), &b);
        match rel.verdict() {
            ReleaseVerdict::NeedsRevision(rv) => {
                assert_eq!(rv.blocking_count(), 1);
                assert_eq!(rv.revisions_remaining(), 2);
                assert!(rv.may_revise());
            }
            other => panic!("必须请求修订，实际 {other}"),
        }
        assert!(rel.released_text().is_none(), "修订请求**不携带**文本");
        assert!(b.consume(), "调用方自己扣一格");
        assert_eq!(b.remaining(), 1);
    }

    #[test]
    fn budget_exhausted_forces_the_degrade_path_instead_of_another_revision() {
        let mut b = RevisionBudget::new(MAX_GROUNDING_REVISIONS);
        assert!(b.consume());
        assert!(b.consume());
        let rel = admit(ONE_BAD_CITED, &default_checks(), &b);
        assert!(rel.verdict().degraded(), "预算耗尽 ⇒ 直接降级，不再请求修订");
        assert!(!matches!(rel.verdict(), ReleaseVerdict::NeedsRevision(_)));
    }

    #[test]
    fn zero_budget_skips_revision_entirely() {
        let rel = admit(ONE_BAD_CITED, &default_checks(), &RevisionBudget::none());
        assert!(rel.verdict().degraded());
    }

    // ── ⭐ 确定性 + 顺序稳定 ──────────────────────────────────────────────

    #[test]
    fn same_input_twice_is_byte_identical() {
        for text in [CLEAN, ONE_BAD_CITED, SAME_FIGURE_TWICE, UNDER_REPORTED, "no declarations here"] {
            let a = admit(text, &default_checks(), &RevisionBudget::none());
            let b = admit(text, &default_checks(), &RevisionBudget::none());
            assert_eq!(a, b, "`{text}` 两次求值必须完全一致");
            assert_eq!(a.released_text().map(ReleasedText::digest), b.released_text().map(ReleasedText::digest));
            let ia = Ledger::new(text, &parse_declarations(text)).digest();
            let ib = Ledger::new(text, &parse_declarations(text)).digest();
            assert_eq!(ia, ib);
        }
    }

    #[test]
    fn reversing_the_claim_order_yields_the_same_released_text() {
        let mut claims = parse_declarations(ONE_BAD_CITED);
        assert_eq!(claims.len(), 2);
        let forward = admit_claims(ONE_BAD_CITED, &claims, &default_checks(), &RevisionBudget::none());
        claims.reverse();
        let backward = admit_claims(ONE_BAD_CITED, &claims, &default_checks(), &RevisionBudget::none());

        assert_eq!(
            released_of(&forward).text(),
            released_of(&backward).text(),
            "逆序喂进声明 ⇒ 释放文本必须相同"
        );
        assert_eq!(
            released_of(&forward).digest(),
            released_of(&backward).digest(),
            "审计身份也必须相同"
        );
        assert_eq!(
            Ledger::new(ONE_BAD_CITED, &claims).digest(),
            Ledger::new(ONE_BAD_CITED, &parse_declarations(ONE_BAD_CITED)).digest(),
            "账本身份键序无关"
        );
    }

    #[test]
    fn digest_is_order_independent_and_discriminating() {
        let digest_of = |text: &str| {
            Ledger::new(text, &parse_declarations(text)).digest()
        };
        let base = digest_of(ONE_BAD_CITED);

        // ⭐ 顺序无关（同一个事实的两种组装）。
        let mut claims = parse_declarations(ONE_BAD_CITED);
        let reversed = Ledger::new(ONE_BAD_CITED, &claims).digest();
        claims.reverse();
        assert_eq!(reversed, Ledger::new(ONE_BAD_CITED, &claims).digest());
        assert_eq!(reversed, base, "逆序不改变账本身份");

        // ⭐ 判别力：换了文本 / 换了声明都必须换摘要。摘要恒定等于没摘要
        // （这正是 `nt_determinism.rs:39-43` 记的「字段覆盖靠手维护」缺陷的
        // 另一面：不动的摘要没人能看出它没在动）。
        assert_ne!(digest_of("bench 9.90[role=cited]\n"), base, "换文本 ⇒ 换摘要");
        assert_ne!(
            digest_of("open 187.40[role=observed]\nbench 9.90[role=cited]\n\n"),
            base,
            "尾部多一个换行 ⇒ 换摘要（所以「切掉整行」这种改写不是免费的）"
        );
        assert_ne!(
            Ledger::new("x", &[]).digest(),
            Ledger::new("x", &parse_declarations("1.0[role=observed]\n")).digest(),
            "多一条声明 ⇒ 换摘要"
        );

        // 预算身份同样参与区分。
        assert_ne!(RevisionBudget::new(0).digest(), RevisionBudget::new(1).digest());
        assert_ne!(
            RevisionBudget::new(2).digest(),
            RevisionBudget { max_revisions: 2, consumed: 1 }.digest(),
            "已消费一格 ⇒ 换身份（否则「预算被偷花」不可见）"
        );
    }

    #[test]
    fn cut_ranges_are_order_insensitive() {
        let text = "abcdefghij";
        let a = apply_cuts(text, &[Span::new(0, 2), Span::new(6, 8)]);
        let b = apply_cuts(text, &[Span::new(6, 8), Span::new(0, 2)]);
        let overlapping = apply_cuts(text, &[Span::new(0, 3), Span::new(2, 5)]);
        assert_eq!(a, b, "切分顺序不得进入结果");
        assert_eq!(a, "cdefij");
        assert_eq!(overlapping, "fghij", "重叠跨度必须合并");
    }

    #[test]
    fn claim_id_lookup_is_independent_of_claim_order() {
        let claims = parse_declarations(SAME_FIGURE_TWICE);
        assert_eq!(claims.len(), 2);
        assert_eq!(claims[0].id(), "9.9#0", "id 由**文档位置**决定");
        assert_eq!(claims[1].id(), "9.9#1");
        let fwd = Ledger::new(SAME_FIGURE_TWICE, &claims);
        let mut reversed = claims.clone();
        reversed.reverse();
        let bwd = Ledger::new(SAME_FIGURE_TWICE, &reversed);
        let probe = "9.9#1";
        assert_eq!(
            fwd.claim_by_id(probe).map(DeclaredClaim::id),
            bwd.claim_by_id(probe).map(DeclaredClaim::id),
            "按稳定 id 寻址与声明顺序无关"
        );
        // 且 id 命中的**是同一条事实**，不只是同一个字符串。
        assert_eq!(
            fwd.claim_by_id(probe).map(|c| c.payload_value(PAYLOAD_SOURCE).is_some()),
            bwd.claim_by_id(probe).map(|c| c.payload_value(PAYLOAD_SOURCE).is_some()),
            "两序命中的必须同一条事实"
        );
        assert_eq!(
            fwd.claim_by_id(probe).map(|c| c.payload_value(PAYLOAD_SOURCE).is_some()),
            Some(false),
            "`9.9#1` 是那条**无来源**的声明"
        );
        assert_eq!(fwd.claim_by_id("9.9#9"), None, "不存在的 id ⇒ None，不猜");
    }

    /// ⭐⭐ **判别力实测（反例 / oracle）**：证明上面那条「顺序无关」测试
    /// 不是恒真的仪式 —— 一份**故意坏**的实现（把 id 当成切片下标去取）会随
    /// 输入序翻转，而本实现不会。
    ///
    /// ⚠️ 这个反例与第一版不同：原来那条是「拿 witness 的 id 当 figure 文本去
    /// 匹配」，它依赖已删除的 `witness=` 协议 ⇒ 一并删掉。现在用一个**与任何
    /// 协议无关**的错法：`figure#occurrence` 的形状**看起来**像下标。
    #[test]
    fn the_order_independence_assertion_has_discriminating_power() {
        let claims = parse_declarations(SAME_FIGURE_TWICE);
        let mut reversed = claims.clone();
        reversed.reverse();

        // 故意坏：把 `9.9#1` 的 `#1` 当成**切片下标** ⇒ 直接取 `claims[1]`。
        // 这份实现「看起来合理」且**能编译**，但它把 id 绑到了调用方的排列上。
        let buggy_says_ok = |slice: &[DeclaredClaim]| {
            let idx = "9.9#1".rsplit('#').next().and_then(|i| i.parse::<usize>().ok());
            idx.and_then(|i| slice.get(i)).and_then(|c| c.payload_value(PAYLOAD_SOURCE)).is_some()
        };
        assert_ne!(
            buggy_says_ok(&claims),
            buggy_says_ok(&reversed),
            "反例：把 id 当下标的坏实现必须随输入序翻转（否则上面的测试无判别力）"
        );

        // 正确实现：按稳定 id **匹配** ⇒ 两序同答（且都指向无来源的那条）。
        let correct_says_ok = |slice: &[DeclaredClaim]| {
            Ledger::new(SAME_FIGURE_TWICE, slice)
                .claim_by_id("9.9#1")
                .and_then(|c| c.payload_value(PAYLOAD_SOURCE))
                .is_some()
        };
        assert_eq!(correct_says_ok(&claims), correct_says_ok(&reversed));
        assert!(!correct_says_ok(&claims), "正确实现应判定 `9.9#1` 无来源");
    }

    // ── 声明式注册表：加检查不动控制流 ────────────────────────────────────

    /// 测试专用**第三条**检查：`proposed` 必须给理由。**它能注册成功这件事本身**
    /// 就是「新增检查不改控制流」的证据。
    #[derive(Debug, Clone, Copy)]
    struct ProposedRationaleCheck;

    impl ProvenanceCheck for ProposedRationaleCheck {
        fn code(&self) -> &'static str {
            "nt.test.proposed.rationale"
        }

        fn description(&self) -> &'static str {
            "test-only: a `proposed` claim must state its rationale"
        }

        fn evaluate(&self, ledger: &Ledger<'_>) -> Vec<Issue> {
            let mut out = Vec::new();
            for (i, claim) in ledger.claims().iter().enumerate() {
                if claim.role() == Some(ProvenanceRole::Proposed)
                    && claim.payload_value("rationale").is_none()
                {
                    out.push(Issue::block(self.code(), i, claim, "no rationale"));
                }
            }
            out
        }
    }

    #[test]
    fn registry_extension_needs_no_control_flow_edit() {
        let text = "view 9.90[role=proposed]\nopen 187.40[role=observed]\n";
        // 基准：默认注册表不拦 proposed。
        let base = admit(text, &default_checks(), &RevisionBudget::none());
        assert!(base.verdict().is_clean(), "默认注册表不拦 proposed");
        assert_eq!(released_of(&base).text(), text);

        // 只在**注册表**里追加一行 —— [`admit`] 一个字都没改。
        let mut extended = default_checks();
        extended.push(Box::new(ProposedRationaleCheck));
        let after = admit(text, &extended, &RevisionBudget::none());
        assert!(after.verdict().degraded(), "新检查立刻生效并触发降级");
        let rt = released_of(&after);
        assert_eq!(
            rt.text(),
            "view \nopen 187.40[role=observed]\n",
            "只切掉新检查点名的那一条，其余原样保留"
        );
        assert_eq!(rt.cut_codes(), &["nt.test.proposed.rationale".to_owned()]);
    }

    #[test]
    fn a_singleton_registry_cannot_be_silently_emptied() {
        // 单条声明被切 ⇒ 无内容可发布（ALL_CLAIMS_CUT）⇒ 拒答而非空文本。
        let text = "view 9.90[role=proposed]\n";
        let mut extended = default_checks();
        extended.push(Box::new(ProposedRationaleCheck));
        match admit(text, &extended, &RevisionBudget::none()).verdict() {
            ReleaseVerdict::Refused(rf) => assert_eq!(rf.reason(), RefusalReason::AllClaimsCut),
            other => panic!("必须拒答，实际 {other}"),
        }
    }

    #[test]
    fn registry_order_does_not_leak_into_the_result() {
        let text = "a 1.0[role=derived]\nb 2.0[role=cited]\n";
        let claims = parse_declarations(text);
        let forward = default_checks();
        let mut backward = default_checks();
        backward.reverse();
        let lf = Ledger::new(text, &claims);
        assert_eq!(validate(&lf, &forward), validate(&lf, &backward), "注册顺序不得泄漏进结果");
    }

    // ── 裁决与文本不可分家 ────────────────────────────────────────────────

    #[test]
    fn a_release_verdict_cannot_be_named_without_carrying_the_text() {
        for rel in [
            admit(CLEAN, &default_checks(), &RevisionBudget::none()),
            one_bad(),
            admit(SAME_FIGURE_TWICE, &default_checks(), &RevisionBudget::none()),
            admit(ONE_BAD_CITED, &default_checks(), &RevisionBudget::new(2)),
        ] {
            // ⭐ 不变式：**有文本 ⟺ 发布态**。修订态与拒答态都不携带文本，
            // 所以「上游拿到一个不含文本的已发布」这件事在类型上不存在。
            let verdict = rel.verdict();
            let has_text = verdict.released_text().is_some();
            let released_verdict = matches!(verdict, ReleaseVerdict::Released(_));
            assert_eq!(has_text, released_verdict, "有文本 ⟺ 发布态");
            if let Some(rt) = verdict.released_text() {
                // 拿到文本的同时，裁决标签、条数、码全部可用 —— 同一趟产出。
                assert!(!verdict.as_str().is_empty());
                assert!(rt.cut_codes().len() <= rt.claims_cut() + rt.claims_kept());
                assert_eq!(verdict.degraded(), rt.degraded());
            }
            assert_eq!(rel.is_released(), has_text);
        }
    }

    #[test]
    fn cut_codes_are_sorted_and_deduped() {
        let rel = one_bad();
        let rt = released_of(&rel);
        let sorted = rt.cut_codes();
        let mut expect = sorted.to_vec();
        expect.sort();
        expect.dedup();
        assert_eq!(sorted, expect.as_slice());
    }

    #[test]
    fn render_is_pure_and_stable() {
        let rel = one_bad();
        let rt = released_of(&rel);
        assert_eq!(rt.render(), rt.render());
        assert!(rt.render().contains("DEGRADED"));
        let claim =
            DeclaredClaim::declared("1.0", Span::new(0, 3), ProvenanceRole::Count, &[]);
        let issue = Issue::not_evaluable(CODE_DERIVED_FORMULA, 0, &claim, "x");
        assert_eq!(issue.render(), issue.render());
        assert!(issue.render().contains("NOT_EVALUABLE"));
        assert_eq!(issue.severity(), IssueSeverity::NotEvaluable);
        assert!(!issue.is_blocking());
    }

    // ── 不变量检查：跨度本身 ──────────────────────────────────────────────

    #[test]
    fn span_is_total_under_every_degenerate_input() {
        let text = "abc";
        assert_eq!(Span::new(3, 3).slice(text), Some(""));
        assert_eq!(Span::new(0, 3).slice(text), Some("abc"));
        assert_eq!(Span::new(5, 9).slice(text), None, "越界 ⇒ None");
        assert_eq!(Span::new(2, 1).slice(text), None, "反向 ⇒ None");
        assert_eq!(Span::new(0, usize::MAX).slice(text), None);
        assert_eq!(Span::new(9, 2).len(), 0, "反向跨度长度饱和到 0，不是天文数字");
        assert!(Span::new(9, 2).is_empty());
        assert_eq!(Span::new(1, 4).merge(&Span::new(7, 8)), Span::new(1, 8));
        assert_eq!(Span::new(1, 4).render(), "1..4");
    }

    #[test]
    fn apply_cuts_never_panics_on_absurd_ranges() {
        let text = "hello";
        for start in [0usize, 3, 99, usize::MAX / 2] {
            for end in [0usize, 1, 5, usize::MAX] {
                let _ = apply_cuts(text, &[Span::new(start, end)]);
                let _ = apply_cuts(text, &[Span::new(end, start), Span::new(1, 2)]);
            }
        }
        assert_eq!(apply_cuts(text, &[]), text, "空切分集 ⇒ 原样");
    }
}
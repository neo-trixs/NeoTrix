use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// T06a：正典类型已收敛至 L0（neotrix-types），此处仅重导出＋引擎实现。
pub use neotrix_types::core::nt_core_approval::{
    ActionType, ApprovalMode, ApproveGate, DisclosureSeverity, PendingAction,
};

/// W2.1 核心: 从动作推断其关闭的可能性 — 把 "agent 很少主动说出 fix 关闭了什么"
/// 机械化到审批门本身。确定性模式匹配, 无 LLM。
pub fn infer_foreclosures(action: &ActionType) -> Vec<String> {
    let mut out = Vec::new();
    match action {
        ActionType::ShellCommand { command } => {
            let c = command.to_lowercase();
            if c.contains("rm -rf") || c.contains("rm -fr") || c.contains("rmdir") {
                out.push("目标路径数据不可恢复".into());
            }
            if c.contains("git push") && (c.contains("--force") || c.contains("-f")) {
                out.push("远端历史被覆盖, 协作者本地分叉失效".into());
            }
            if c.contains("drop table") || c.contains("drop database") || c.contains("truncate table") {
                out.push("数据库结构/数据即刻丢失".into());
            }
            if c.contains("dd ") && (c.contains("of=/dev/") || c.contains("oflag")) || c.contains("mkfs") {
                out.push("目标块设备全盘覆写".into());
            }
            if c.contains("chmod -r 777") || c.contains("chmod 777 /") {
                out.push("权限边界永久放开, 审计链失效".into());
            }
            if c.contains("systemctl stop") || c.contains("service stop") || c.contains("reboot")
                || c.contains("shutdown") || c.contains("pkill") || c.contains("killall")
            {
                out.push("运行中服务/进程即时中断".into());
            }
        }
        ActionType::GitOperation { description } => {
            let d = description.to_lowercase();
            if d.contains("--force") || d.contains("push -f") {
                out.push("远端历史被覆盖, 协作者本地分叉失效".into());
            }
            if d.contains("reset --hard") || d.contains("checkout -- .") || d.contains("clean -fd") {
                out.push("未提交工作区改动不可恢复".into());
            }
        }
        _ => {}
    }
    out
}

/// 决策方向。approve 与 deny 在**返回值与账本**两个层面都可区分
/// —— 此前两者函数体逐字节相同（都只 `pending.remove(idx)`），事后无法回答
/// 「这个动作到底被放行了还是被拒绝了」。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Denied,
}

impl ApprovalDecision {
    /// 账本/日志里的稳定文本标签。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Denied => "denied",
        }
    }
}

/// 审批决策被显式拒绝的原因。区分四态所需的两种失败：
/// 「从未签发」与「已经决策过」必须能被调用方分辨 —— 修复前二者都退化成
/// 同一句 `"No pending action with id '...'"`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    /// id 从未在本会话签发。跨进程陈旧 id 也走这里（纪元段不同故匹配不上）。
    UnknownId { id: String },
    /// id 已有决策。重复点击（先 approve 后 deny、二次 approve）必须被拒，
    /// 而不能因为「pending 里已经没了」而被静默当成一次新决策。
    AlreadyDecided { id: String, previous: ApprovalDecision },
}

impl ApprovalError {
    /// 机器可读判别标签，供调用方分支与测试断言。
    pub fn kind(&self) -> &'static str {
        match self {
            Self::UnknownId { .. } => "unknown-id",
            Self::AlreadyDecided { .. } => "already-decided",
        }
    }

    /// 被拒的 id。
    pub fn id(&self) -> &str {
        match self {
            Self::UnknownId { id } => id,
            Self::AlreadyDecided { id, .. } => id,
        }
    }
}

impl std::fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownId { id } => write!(f, "unknown approval id '{}'", id),
            Self::AlreadyDecided { id, previous } => {
                write!(f, "approval id '{}' already decided as {}", id, previous.as_str())
            }
        }
    }
}

impl std::error::Error for ApprovalError {}

/// 允许 `?` 从 `ApprovalError` 直接落到 `Result<_, String>` 的调用点
/// （本仓既有签名是 `Result<(), String>`，此处只收窄 Ok 侧类型）。
impl From<ApprovalError> for String {
    fn from(e: ApprovalError) -> Self {
        e.to_string()
    }
}

/// 单条决策审计。字段取自本仓两处既有约定：
/// `l6_meta/coordination/governance/enforcement/audit.rs::AuditEntry`（时间戳
/// 严格递增 + 容量淘汰）与 `l3_embodiment/nt_shield/shield_core/permissions.rs::AuditEntry`
/// （`resolution` + `reason`）。此处合成二者的最小子集。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalAuditEntry {
    /// 被决策的 pending action id（自带纪元前缀，见 [`new_epoch_tag`]）。
    pub action_id: String,
    /// 放行还是拒绝。
    pub decision: ApprovalDecision,
    /// 谁做的决定：引擎配置的 actor（`NEOTRIX_APPROVAL_ACTOR` 或 `decide_as` 显式传入）。
    pub actor: String,
    /// 可选理由（与 permissions.rs::approve/deny 的 `reason` 参数同义）。
    pub reason: Option<String>,
    /// 决策时刻：墙钟 UNIX 毫秒。**不是 `Instant`** —— 见 [`unix_millis_now`]。
    pub decided_at_unix_millis: u64,
    /// 决策当时的动作描述快照。`ActionType` 未实现 `PartialEq`，且 pending 条目
    /// 已出队，审计必须自带上下文而不能靠回查。
    pub description: String,
}

/// 审计轨迹容量上限（最老优先淘汰）。决策账本 `decided` 与之同寿命。
const DEFAULT_AUDIT_CAPACITY: usize = 1024;

/// 决策署名主体的环境变量名。
const ACTOR_ENV: &str = "NEOTRIX_APPROVAL_ACTOR";

/// 进程内纪元自增序号。保证「同一进程内新建的多个引擎」纪元互不相同：
/// 只靠墙钟毫秒时，测试里连续 `new()` 两个引擎可能落在同一毫秒，
/// 那会让「重启后不复用旧 id」这条性质变成偶发通过 / 偶发失败。
static EPOCH_SEQ: AtomicU64 = AtomicU64::new(0);

/// 生成一次性会话纪元 —— id 的命名空间前缀。
///
/// ⛔ 时间源纪律：`PendingAction.created_at` 是 `Instant`，它是**进程相对**的单调量，
/// 跨进程没有稳定表示（`crates/neotrix-types/src/core/nt_core_approval.rs`）。
/// 把 `Instant` 编码进 id，两个进程的 id 空间就不可区分 —— 这正是「上一进程客户端
/// 手里攥着的 approval 点击批准了本进程另一个动作」的根因。这里只用**进程间可比**的量：
/// `SystemTime`（墙钟）与 `std::process::id()`（同机并发进程消歧）。
///
/// 三者放进互不重叠的位段，故「任一项不同 ⇒ 纪元不同」，无哈希碰撞窗口：
/// `bit 63..48` = pid 低 16 位 · `bit 47..32` = 进程内序号 · `bit 31..0` = 墙钟毫秒低 32 位。
fn new_epoch_tag() -> u64 {
    let millis = u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0),
    )
    .unwrap_or(u64::MAX);
    let pid = u64::from(std::process::id() & 0xFFFF);
    let seq = EPOCH_SEQ.fetch_add(1, Ordering::Relaxed) & 0xFFFF;
    (pid << 48) | (seq << 32) | (millis & 0xFFFF_FFFF)
}

/// 墙钟 UNIX 毫秒。审计时间戳要跨进程可比，故**不能用 `Instant`**
/// （`Instant` 只适合「本进程内过了多久」，见 [`ApprovalEngine::expire_stale`]）。
fn unix_millis_now() -> u64 {
    u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0),
    )
    .unwrap_or(u64::MAX)
}

/// 默认署名主体：`NEOTRIX_APPROVAL_ACTOR` 优先，否则退到 `"local-operator"`。
///
/// ⚠️ 本轮只做**进程内**审计（明确不做持久化），故这里没有 OS 用户名 / 会话 id 注入 ——
/// 跨进程、跨会话归因属持久化范畴，不在本次修复内。
fn default_actor() -> String {
    std::env::var(ACTOR_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "local-operator".to_string())
}

/// 审批引擎：待审队列 + **决策账本 + 审计轨迹**。
///
/// ⭐⭐⭐ 三态审批裁决（2026-10-05）。
///
/// ## 为什么不是 bool
/// `bool` 只能表达「要过问一下 / 不用问」，**表达不了「问也没用」**。
/// 于是硬拒（deny）只能退化成「需审批」（ask）——
/// 代码里的注释自己承认了：*"action is blocked — require approval to inform user"*。
/// 后果：用户在提示里点一下，`git push --force` 就跑出去了。
///
/// ## 三态的语义（严格区分，不可互相退化）
/// | 变体 | 含义 | 人点批准后 |
/// |---|---|---|
/// | [`Deny`](ActionVerdict::Deny) | **硬拒**，越不过去 | **仍然拒** |
/// | [`Ask`](ActionVerdict::Ask) | 需要人过一眼 | 放行 |
/// | [`Allow`](ActionVerdict::Allow) | 无需打扰 | 直接跑 |
///
/// ⛔ 禁止把 `Deny` 折叠成 `Ask`（那是 2026-10-05 修掉的缺陷）。
/// 判据来源：`codewhale-hq/Codewhale` 授权栈第 2/7 层
/// ——「既allow 又 deny ⇒ deny 胜」、且 repo-law 在 Full Access 下
/// 变成**硬 block**（不是弹窗）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionVerdict {
    /// ⛔ 硬拒。人点批准**也不放行**。
    Deny,
    /// 需要人确认；批准后放行。
    Ask,
    /// 无需审批。
    Allow,
}

impl ActionVerdict {
    /// 是否需要人过一眼（`Deny` 与 `Ask` 都要）。
    pub fn needs_human(&self) -> bool {
        matches!(self, ActionVerdict::Deny | ActionVerdict::Ask)
    }

    /// ⭐ 是否**不可放行**。这是 `Deny` 存在的唯一理由 ——
    /// 若某条路径只问`needs_human()` 而不问本方法，
    /// 硬拒就会被降级成弹窗（2026-10-05 修掉的正是这个）。
    pub fn is_blocked(&self) -> bool {
        matches!(self, ActionVerdict::Deny)
    }
}


/// 作用域声明：本结构的状态**全在内存**，进程退出即消失。跨重启的 id 唯一性由
/// 纪元前缀保证（不必持久化即可拒掉陈旧 id），但「决策历史可追」在跨重启意义上
/// 不成立 —— 那是持久化范畴的活，此处不做。
pub struct ApprovalEngine {
    mode: ApprovalMode,
    pending: Vec<PendingAction>,
    /// 本引擎内的递增序号（与 `epoch` 组成 id）。
    next_id: u64,
    /// 会话纪元 —— id 命名空间前缀，见 [`new_epoch_tag`]。
    epoch: u64,
    /// 决策署名主体。
    actor: String,
    /// 决策账本：id → 决策。与审计轨迹同寿命（超出容量退化为 UnknownId）。
    decided: HashMap<String, ApprovalDecision>,
    /// 追加式审计轨迹，按时间升序，最老在前。
    audit: Vec<ApprovalAuditEntry>,
    audit_capacity: usize,
}

impl ApprovalEngine {
    pub fn new(mode: ApprovalMode) -> Self {
        Self::with_epoch(mode, new_epoch_tag())
    }

    /// 指定纪元构造（仅供本模块测试做可复现断言；生产走 [`ApprovalEngine::new`]）。
    fn with_epoch(mode: ApprovalMode, epoch: u64) -> Self {
        Self {
            mode,
            pending: Vec::new(),
            next_id: 0,
            epoch,
            actor: default_actor(),
            decided: HashMap::new(),
            audit: Vec::new(),
            audit_capacity: DEFAULT_AUDIT_CAPACITY,
        }
    }

    pub fn mode(&self) -> ApprovalMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ApprovalMode) {
        self.mode = mode;
    }

    /// ⭐⭐⭐ 三态判据（2026-10-05）——`require_approval` 的**权威版本**。
    ///
    /// ## 为什么必须新增（`bool` 表达不了三态）
    /// `require_approval` 返回 `bool`，于是「**硬拒**」只能退化成
    /// 「**需审批**」—— 代码里的注释自己写着
    // 原文注释：*"action is blocked — require approval to inform user"*，
    // 也就是**把 deny 当成 ask**。后果：内置画像里
    // `read_secrets` 与 `git_force_push` 两条 `Deny` 规则，
    // 用户在提示里点一下批准，**`git push --force` 就真的跑出去**。
    /// ⇒ bool 这一层**丢失了 deny 与 ask 的区别**，不是判断写错了。
    ///
    /// ## 与 `require_approval` 的关系（不破既有签名）
    /// `require_approval` 保留原样（5 处现有调用方不动），
    /// 但它的语义被**重新定义为**「Deny 与 Ask 都需要人过一眼」，
    /// 而**硬拒的判据改走本方法**。调用方要判 deny 时**必须**用本方法。
    ///
    /// 判据来源：`codewhale-hq/Codewhale` 授权栈第 2 层
    /// ——「同一命令同时出现在 allow 与 deny 列表 ⇒ **deny**」，
    /// 且第 7 层「repo-law 在 Full Access 下变成**硬block**」
    /// ——即**存在一个越不过去的档位**，而不是「问一下就能过」。
    pub fn action_verdict(&self, action: &ActionType) -> ActionVerdict {
        let action_key = crate::l6_meta::nt_permission_profiles::action_type_to_key(action);
        if crate::l6_meta::nt_permission_profiles::is_action_denied(action_key) {
            // ⭐ 硬拒：**不是**「问一下就能过」。
            return ActionVerdict::Deny;
        }
        if crate::l6_meta::nt_permission_profiles::is_action_allowed(action_key) {
            return ActionVerdict::Allow;
        }
        match self.mode {
            ApprovalMode::Suggest => ActionVerdict::Ask,
            ApprovalMode::AutoEdit => ActionVerdict::Ask,
            ApprovalMode::FullAuto => ActionVerdict::Allow,
        }
    }

    /// Check whether a given action type requires user approval under current mode.
    /// Respects active permission profile (deny overrides everything).
    ///
    /// ⚠️ **不要用它判断「是否被硬拒」** —— bool 表达不了 deny/ask 之别，
    /// 本方法把两者都压成 `true`。判 deny 请用
    /// [`Self::action_verdict`]。
    pub fn require_approval(&self, action: &ActionType) -> bool {
        // Profile deny takes precedence over everything
        let action_key = crate::l6_meta::nt_permission_profiles::action_type_to_key(action);
        if crate::l6_meta::nt_permission_profiles::is_action_denied(action_key) {
            return true; // action is blocked — require approval to inform user
        }
        // Profile allow overrides mode (no approval needed)
        if crate::l6_meta::nt_permission_profiles::is_action_allowed(action_key) {
            return false;
        }
        match self.mode {
            ApprovalMode::Suggest => true,
            ApprovalMode::AutoEdit => {
                // AutoEdit 白名单: 文件类免审批; 命令/git/未分类工具 (Other) 需审批
                matches!(
                    action,
                    ActionType::ShellCommand { .. }
                        | ActionType::GitOperation { .. }
                        | ActionType::Other { .. }
                )
            }
            ApprovalMode::FullAuto => false,
        }
    }

    /// Submit a new action for approval. Returns the pending action.
    pub fn submit(&mut self, action: ActionType) -> PendingAction {
        let id = self.alloc_id();
        let description = describe_action(&action);
        // W2.1 披露门: 审批门替 agent 说出 fix 关闭了什么 (Warning 级 andon)
        let forecloses = infer_foreclosures(&action);
        let description = if forecloses.is_empty() {
            description
        } else {
            format!("{}\n⚠ 此操作将关闭: {}", description, forecloses.join("; "))
        };
        if !forecloses.is_empty() {
            log::warn!(
                "poka-yoke disclosure: {} → forecloses {:?}",
                id, forecloses
            );
        }
        let pa = PendingAction {
            id,
            action_type: action,
            description,
            forecloses,
            created_at: Instant::now(),
        };
        self.pending.push(pa.clone());
        pa
    }

    /// 分配下一个 id：`a{纪元 16 hex}{序号 4 位}`。
    ///
    /// 纪元段是「重启不复用旧 id」的保证：进程重启后序号从 0 重来，但纪元换段，
    /// 于是上一进程客户端攥着的 `aXXXX…-0007` 在新引擎里匹配不到任何 pending
    /// 条目，按 [`ApprovalError::UnknownId`] 被显式拒绝，而不是「恰好」批准了
    /// 本进程的另一个动作。纪元只来自墙钟 + pid（见 [`new_epoch_tag`]），
    /// 不来自 `Instant` —— 故无需持久化即可跨重启区分 id 空间。
    fn alloc_id(&mut self) -> String {
        let id = format!("a{:016x}{:04}", self.epoch, self.next_id);
        self.next_id += 1;
        id
    }

    /// 决策状态机的唯一写路径（approve / deny / *_all 共用）。
    ///
    /// 判定顺序即四态口径：
    /// 1. `decided` 命中 → [`ApprovalError::AlreadyDecided`]（重复决策，绝不静默成功）
    /// 2. `pending` 命中 → 记账 + 出队 → `Ok(decision)`
    /// 3. 都没命中 → [`ApprovalError::UnknownId`]（含跨进程陈旧 id）
    fn decide(
        &mut self,
        id: &str,
        decision: ApprovalDecision,
        actor: &str,
        reason: Option<String>,
    ) -> Result<ApprovalDecision, ApprovalError> {
        if let Some(&previous) = self.decided.get(id) {
            return Err(ApprovalError::AlreadyDecided { id: id.to_string(), previous });
        }
        let idx = self
            .pending
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| ApprovalError::UnknownId { id: id.to_string() })?;
        let pa = self.pending.remove(idx);
        self.record_decision(&pa, decision, actor, reason);
        log::info!(
            "approval decision: id={} decision={} actor={} desc={}",
            pa.id,
            decision.as_str(),
            actor,
            pa.description
        );
        Ok(decision)
    }

    /// 写一条审计并同步决策账本。账本与审计同寿命：超出容量时最老条目出队，
    /// 其 id 一并从 `decided` 移除 —— 于是过期 id 退化为 `UnknownId`，
    /// 宁可报「未知」也不把「忘了」当成「已决策」（或反过来）。
    fn record_decision(
        &mut self,
        pa: &PendingAction,
        decision: ApprovalDecision,
        actor: &str,
        reason: Option<String>,
    ) {
        // 严格递增时间戳（同 governance/enforcement/audit.rs）：毫秒分辨率下
        // 同一毫秒的连续决策仍可从时间戳恢复先后次序。
        let prev = self.audit.last().map(|e| e.decided_at_unix_millis).unwrap_or(0);
        let stamp = unix_millis_now().max(prev.saturating_add(1));
        self.audit.push(ApprovalAuditEntry {
            action_id: pa.id.clone(),
            decision,
            actor: actor.to_string(),
            reason,
            decided_at_unix_millis: stamp,
            description: pa.description.clone(),
        });
        self.decided.insert(pa.id.clone(), decision);
        if self.audit.len() > self.audit_capacity {
            let drop_n = self.audit.len() - self.audit_capacity;
            let evicted: Vec<String> =
                self.audit.drain(..drop_n).map(|e| e.action_id).collect();
            for id in evicted {
                self.decided.remove(&id);
            }
        }
    }

    /// 放行一个待审批动作；以引擎配置的 actor 署名。
    ///
    /// 返回 `Ok(Approved)` 表示本次决策成立；重复决策 → [`ApprovalError::AlreadyDecided`]，
    /// 从未签发（含跨进程陈旧）id → [`ApprovalError::UnknownId`]。
    pub fn approve(&mut self, id: &str) -> Result<ApprovalDecision, ApprovalError> {
        let actor = self.actor.clone();
        self.decide(id, ApprovalDecision::Approved, &actor, None)
    }

    /// 拒绝一个待审批动作；以引擎配置的 actor 署名。与 [`ApprovalEngine::approve`]
    /// 现在在返回值（`Ok(Denied)`）与账本（`decided[id] = Denied`）上都可区分。
    pub fn deny(&mut self, id: &str) -> Result<ApprovalDecision, ApprovalError> {
        let actor = self.actor.clone();
        self.decide(id, ApprovalDecision::Denied, &actor, None)
    }

    /// 全权决策入口：显式给出决策方向、署名主体与理由
    /// （如 TUI 把「按了哪个键 / 哪个 profile」回传为 actor）。
    pub fn decide_as(
        &mut self,
        id: &str,
        decision: ApprovalDecision,
        actor: &str,
        reason: Option<String>,
    ) -> Result<ApprovalDecision, ApprovalError> {
        self.decide(id, decision, actor, reason)
    }

    /// 把超过 `ttl` 仍未决策的待审批项按 **Denied** 记账后出队，返回过期条数。
    ///
    /// 刻意**不静默丢弃**：过期也是一次决策，账本里必须留痕。
    /// 默认不自动调用（由调用方按需触发），故不改变既有行为。
    ///
    /// 此处用 `Instant` 是**正确**的：它只回答「本进程内过了多久」，
    /// 不参与 id 生成、不跨进程比较 —— 与 [`new_epoch_tag`] 的禁用理由正相反。
    pub fn expire_stale(&mut self, ttl: Duration) -> usize {
        let now = Instant::now();
        let actor = self.actor.clone();
        let mut expired = 0usize;
        while let Some(idx) = self.pending.iter().position(|p| now.duration_since(p.created_at) >= ttl)
        {
            let pa = self.pending.remove(idx);
            let reason = Some(format!("expired after {:?} without decision", ttl));
            self.record_decision(&pa, ApprovalDecision::Denied, &actor, reason);
            log::warn!("approval expired: id={} desc={}", pa.id, pa.description);
            expired += 1;
        }
        expired
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn summary(&self) -> String {
        format!("Mode: {} | Pending: {}", mode_label(self.mode), self.pending.len())
    }

    /// 审计摘要（含放行/拒绝计数）。`summary()` 的文本格式被快照测试冻结，
    /// 故另立此函数而非改写它。
    pub fn audit_summary(&self) -> String {
        let (approved, denied) = self.decision_counts();
        format!(
            "Mode: {} | Pending: {} | Approved: {} | Denied: {} | Decisions: {}",
            mode_label(self.mode),
            self.pending.len(),
            approved,
            denied,
            self.audit.len()
        )
    }

    pub fn pending_actions(&self) -> &[PendingAction] {
        &self.pending
    }

    /// 决策署名主体。
    pub fn actor(&self) -> &str {
        &self.actor
    }

    /// 设置决策署名主体（如从 TUI 会话或 profile 名派生）。
    pub fn set_actor(&mut self, actor: impl Into<String>) {
        self.actor = actor.into();
    }

    /// 全部决策审计，按时间升序。
    pub fn decisions(&self) -> &[ApprovalAuditEntry] {
        &self.audit
    }

    /// 某 id 的既有决策（`None` = 本引擎内未决策过）。
    pub fn decision_for(&self, id: &str) -> Option<ApprovalDecision> {
        self.decided.get(id).copied()
    }

    /// `(放行数, 拒绝数)` —— 「这个动作到底放行没有」的事后审计口径。
    pub fn decision_counts(&self) -> (usize, usize) {
        let mut approved = 0;
        let mut denied = 0;
        for e in &self.audit {
            match e.decision {
                ApprovalDecision::Approved => approved += 1,
                ApprovalDecision::Denied => denied += 1,
            }
        }
        (approved, denied)
    }

    /// 批量放行：逐条记账（此前 `pending.clear()` 把这批决策一笔勾销，无任何审计）。
    pub fn approve_all(&mut self) -> usize {
        let ids: Vec<String> = self.pending.iter().map(|p| p.id.clone()).collect();
        let count = ids.len();
        let actor = self.actor.clone();
        for id in ids {
            // `&mut self` 独占 ⇒ 快照自 `pending` 的 id 必然命中，不会失败。
            let _ = self.decide(&id, ApprovalDecision::Approved, &actor, None);
        }
        count
    }

    /// 批量拒绝：逐条记账（同 [`ApprovalEngine::approve_all`]）。
    pub fn deny_all(&mut self) -> usize {
        let ids: Vec<String> = self.pending.iter().map(|p| p.id.clone()).collect();
        let count = ids.len();
        let actor = self.actor.clone();
        for id in ids {
            let _ = self.decide(&id, ApprovalDecision::Denied, &actor, None);
        }
        count
    }
}

fn mode_label(mode: ApprovalMode) -> &'static str {
    match mode {
        ApprovalMode::Suggest => "Suggest",
        ApprovalMode::AutoEdit => "AutoEdit",
        ApprovalMode::FullAuto => "FullAuto",
    }
}

// T06b：L6 引擎实现 L0 trait——L3 只经此 trait（回调）调用，不直引引擎。
impl ApproveGate for ApprovalEngine {
    fn needs_approval(&self, action: &ActionType) -> bool {
        self.require_approval(action)
    }

    fn approval_mode(&self) -> ApprovalMode {
        self.mode()
    }
}

/// 预览截断阈值：超过 `PREVIEW_MAX` 个**字符**则取前 `PREVIEW_KEEP` 个加省略号。
const PREVIEW_MAX: usize = 60;
const PREVIEW_KEEP: usize = 57;

/// ⭐ 预览截断（按**字符**）。
///
/// ⚠️ 2026-10-05 修正：原先是「按**字节**判断 + 按字节切」——
/// `content_preview.len() > 60` 配 `&content_preview[..57]`。
///
/// ⛔ **真实 panic**：`&str` 的字节索引若不落在字符边界上会 panic
/// （`byte index … is not a char boundary`）。而 `content_preview` / `diff`
/// 来自工具调用的**实际内容** —— 含 CJK / emoji 时第 57 字节极常落在字符中间
/// ⇒ **一次多字节内容的写文件/编辑审批就能触发 panic**。
///
/// ⇒ 改为按**字符**计数与截断，永不切在边界中间；同时「60」真正表示 60 个字符
/// 而非 60 字节（原先对纯中文会砍掉近一半内容）。
fn preview_or_full(s: &str) -> String {
    if s.chars().count() > PREVIEW_MAX {
        let head: String = s.chars().take(PREVIEW_KEEP).collect();
        format!("{}…", head)
    } else {
        s.to_string()
    }
}

fn describe_action(action: &ActionType) -> String {
    match action {
        ActionType::FileWrite { path, content_preview } => {
            format!("📝 Write {}: {}", path, preview_or_full(content_preview))
        }
        ActionType::FileCreate { path } => format!("📄 Create {}", path),
        ActionType::FileEdit { path, diff } => {
            format!("✏️ Edit {}: {}", path, preview_or_full(diff))
        }
        ActionType::ShellCommand { command } => format!("💻 Run: {}", command),
        ActionType::GitOperation { description } => format!("🔧 Git: {}", description),
        ActionType::Other { tool, args } => format!("🔧 Tool {}: {}", tool, args),
    }
}

/// Global approval engine fallback — prefer `CliContext.approval` instead.
pub static APPROVAL_ENGINE: LazyLock<Mutex<ApprovalEngine>> = LazyLock::new(|| {
    Mutex::new(ApprovalEngine::new(ApprovalMode::Suggest))
});

pub fn global_approval() -> &'static Mutex<ApprovalEngine> {
    &APPROVAL_ENGINE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_requires_all() {
        let engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert!(engine.require_approval(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }));
        assert!(engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(engine.require_approval(&ActionType::FileEdit { path: "x".into(), diff: "".into() }));
        assert!(engine.require_approval(&ActionType::ShellCommand { command: "ls".into() }));
        assert!(engine.require_approval(&ActionType::GitOperation { description: "commit".into() }));
    }

    #[test]
    fn test_auto_edit_approves_files() {
        let engine = ApprovalEngine::new(ApprovalMode::AutoEdit);
        assert!(!engine.require_approval(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }));
        assert!(!engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(!engine.require_approval(&ActionType::FileEdit { path: "x".into(), diff: "".into() }));
        assert!(engine.require_approval(&ActionType::ShellCommand { command: "ls".into() }));
        assert!(engine.require_approval(&ActionType::GitOperation { description: "commit".into() }));
    }

    #[test]
    fn test_full_auto_requires_nothing() {
        let engine = ApprovalEngine::new(ApprovalMode::FullAuto);
        assert!(!engine.require_approval(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }));
        assert!(!engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(!engine.require_approval(&ActionType::FileEdit { path: "x".into(), diff: "".into() }));
        assert!(!engine.require_approval(&ActionType::ShellCommand { command: "ls".into() }));
        assert!(!engine.require_approval(&ActionType::GitOperation { description: "commit".into() }));
    }

    #[test]
    fn test_submit_approve_deny_cycle() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(ActionType::FileWrite { path: "/tmp/test".into(), content_preview: "hello".into() });
        assert_eq!(engine.pending_count(), 1);
        assert!(engine.approve(&pa.id).is_ok());
        assert_eq!(engine.pending_count(), 0);

        let pa2 = engine.submit(ActionType::FileCreate { path: "/tmp/test2".into() });
        assert!(engine.deny(&pa2.id).is_ok());
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn test_approve_unknown_id() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert!(engine.approve("nonexistent").is_err());
    }

    #[test]
    fn test_approve_all() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        engine.submit(ActionType::FileWrite { path: "a".into(), content_preview: "".into() });
        engine.submit(ActionType::FileCreate { path: "b".into() });
        assert_eq!(engine.approve_all(), 2);
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(ApprovalMode::from_str("suggest"), Some(ApprovalMode::Suggest));
        assert_eq!(ApprovalMode::from_str("auto-edit"), Some(ApprovalMode::AutoEdit));
        assert_eq!(ApprovalMode::from_str("full-auto"), Some(ApprovalMode::FullAuto));
        assert_eq!(ApprovalMode::from_str("yolo"), Some(ApprovalMode::FullAuto));
        assert_eq!(ApprovalMode::from_str("unknown"), None);
    }

    #[test]
    fn test_global_engine() {
        let engine = global_approval();
        let mut e = engine.lock().unwrap();
        assert_eq!(e.mode(), ApprovalMode::Suggest);
        e.set_mode(ApprovalMode::FullAuto);
        assert_eq!(e.mode(), ApprovalMode::FullAuto);
        e.set_mode(ApprovalMode::Suggest);
    }

    #[test]
    fn test_summary() {
        let engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert_eq!(engine.summary(), "Mode: Suggest | Pending: 0");
    }

    #[test]
    fn test_pending_actions_list() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        engine.submit(ActionType::FileWrite { path: "x.txt".into(), content_preview: "data".into() });
        let list = engine.pending_actions();
        assert_eq!(list.len(), 1);
        assert!(list[0].description.contains("Write"));
    }

    #[test]
    fn test_other_action_type() {
        // Other 兜底：Suggest 下需审批，AutoEdit 下也需审批（非文件/非命令白名单）。
        let engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert!(engine.require_approval(&ActionType::Other { tool: "web_search".into(), args: "q=rust".into() }));
        let engine = ApprovalEngine::new(ApprovalMode::AutoEdit);
        assert!(engine.require_approval(&ActionType::Other { tool: "web_search".into(), args: "q=rust".into() }));
        let engine = ApprovalEngine::new(ApprovalMode::FullAuto);
        assert!(!engine.require_approval(&ActionType::Other { tool: "web_search".into(), args: "q=rust".into() }));
        // describe_action 不 panic 且包含工具名。
        let pa = ApprovalEngine::new(ApprovalMode::Suggest)
            .submit(ActionType::Other { tool: "web_search".into(), args: "q=rust".into() });
        assert!(pa.description.contains("web_search"));
    }

    // ── W2.1 (batch3 2026-08-26, rainmanjam/poka-yoke 吸收) 披露门验收 ──

    #[test]
    fn test_infer_foreclosures_destructive_shell() {
        let f = infer_foreclosures(&ActionType::ShellCommand {
            command: "rm -rf ./build && echo done".into(),
        });
        assert!(f.iter().any(|s| s.contains("不可恢复")), "{f:?}");

        let f = infer_foreclosures(&ActionType::ShellCommand {
            command: "git push --force origin main".into(),
        });
        assert!(f.iter().any(|s| s.contains("远端历史")), "{f:?}");
    }

    #[test]
    fn test_infer_foreclosures_git_reset_hard() {
        let f = infer_foreclosures(&ActionType::GitOperation {
            description: "reset --hard to v1".into(),
        });
        assert!(f.iter().any(|s| s.contains("工作区改动")), "{f:?}");
    }

    #[test]
    fn test_benign_action_no_disclosure() {
        assert!(infer_foreclosures(&ActionType::ShellCommand { command: "ls -la".into() }).is_empty());
        assert!(infer_foreclosures(&ActionType::FileEdit { path: "a.rs".into(), diff: "-old\n+new".into() }).is_empty());
    }

    #[test]
    fn test_submit_attaches_disclosure_and_severity() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(ActionType::ShellCommand { command: "rm -rf /tmp/x".into() });
        assert_eq!(pa.disclosure_severity(), DisclosureSeverity::Warning);
        assert!(!pa.forecloses.is_empty());
        assert!(pa.description.contains("此操作将关闭"), "{}", pa.description);

        let benign = engine.submit(ActionType::ShellCommand { command: "ls".into() });
        assert_eq!(benign.disclosure_severity(), DisclosureSeverity::Detection);
    }

    // ── Failure path tests (added by test-guardian) ──

    #[test]
    fn test_deny_on_unknown_id_returns_error() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let result = engine.deny("nonexistent-id-999");
        assert!(result.is_err(), "denying unknown ID should return error");
    }

    #[test]
    fn test_approve_already_approved_returns_error() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(ActionType::FileWrite { path: "x".into(), content_preview: "".into() });
        assert!(engine.approve(&pa.id).is_ok());
        // Second approve should fail
        let result = engine.approve(&pa.id);
        assert!(result.is_err(), "approving already-approved action should error");
    }

    #[test]
    fn test_deny_already_denied_returns_error() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(ActionType::FileCreate { path: "x".into() });
        assert!(engine.deny(&pa.id).is_ok());
        // Second deny should fail
        let result = engine.deny(&pa.id);
        assert!(result.is_err(), "denying already-denied action should error");
    }

    // ── T06a 快照回归：冻结三模式 × 六动作的放行/拦截判决 ──
    // R = 需审批，- = 放行。行为变更必须显式更新本快照。
    // 注：与既有测试同假设（全局 permission profile 默认中性）。
    fn probe_actions() -> Vec<ActionType> {
        vec![
            ActionType::FileWrite { path: "x".into(), content_preview: "".into() },
            ActionType::FileCreate { path: "x".into() },
            ActionType::FileEdit { path: "x".into(), diff: "".into() },
            ActionType::ShellCommand { command: "ls".into() },
            ActionType::GitOperation { description: "commit".into() },
            ActionType::Other { tool: "web_search".into(), args: "q=rust".into() },
        ]
    }

    #[test]
    fn test_t06a_approval_matrix_snapshot() {
        let mut snap = String::new();
        for mode in [ApprovalMode::Suggest, ApprovalMode::AutoEdit, ApprovalMode::FullAuto] {
            let engine = ApprovalEngine::new(mode);
            let row: String = probe_actions()
                .iter()
                .map(|a| if engine.require_approval(a) { 'R' } else { '-' })
                .collect();
            snap.push_str(&format!("{:?}|{}\n", mode, row));
        }
        let expected = "Suggest|RRRRRR\nAutoEdit|---RRR\nFullAuto|------\n";
        assert_eq!(snap, expected, "approval matrix changed — update snapshot deliberately");
    }

    #[test]
    fn test_t06b_approve_gate_trait_object() {
        // L0 trait 可被 trait object 调用：L3 回调路径的类型基础。
        let engine = ApprovalEngine::new(ApprovalMode::AutoEdit);
        let gate: &dyn ApproveGate = &engine;
        assert_eq!(gate.approval_mode(), ApprovalMode::AutoEdit);
        assert!(!gate.needs_approval(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }));
        assert!(gate.needs_approval(&ActionType::ShellCommand { command: "ls".into() }));
    }

    // ── 决策身份 / 可审计性（approve 与 deny 曾逐字节相同） ──

    fn w(path: &str) -> ActionType {
        ActionType::FileWrite { path: path.into(), content_preview: "data".into() }
    }

    #[test]
    fn test_approve_then_deny_same_id_is_already_decided() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(w("/tmp/x"));
        assert_eq!(engine.approve(&pa.id), Ok(ApprovalDecision::Approved));

        // 第二次 deny 同一 id：必须报 already-decided，不得静默成功。
        let err = engine.deny(&pa.id).expect_err("deny after approve must be rejected");
        assert_eq!(err.kind(), "already-decided");
        assert_eq!(err.id(), pa.id);
        assert_eq!(
            err,
            ApprovalError::AlreadyDecided {
                id: pa.id.clone(),
                previous: ApprovalDecision::Approved
            }
        );

        // 账本保持第一次的决策，不得被第二次调用改写。
        assert_eq!(engine.decision_for(&pa.id), Some(ApprovalDecision::Approved));
        assert_eq!(engine.decisions().len(), 1, "第二次决策不得留下审计痕迹");
        assert_eq!(engine.decision_counts(), (1, 0));
    }

    #[test]
    fn test_approve_and_deny_are_distinguishable_in_ledger() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        engine.set_actor("tui-operator-7");
        let approved = engine.submit(w("/tmp/a"));
        let denied = engine.submit(ActionType::FileCreate { path: "/tmp/b".into() });

        assert_eq!(engine.approve(&approved.id), Ok(ApprovalDecision::Approved));
        assert_eq!(engine.deny(&denied.id), Ok(ApprovalDecision::Denied));

        // 1) 返回值可区分
        assert_ne!(ApprovalDecision::Approved, ApprovalDecision::Denied);
        assert_eq!(engine.decision_for(&approved.id), Some(ApprovalDecision::Approved));
        assert_eq!(engine.decision_for(&denied.id), Some(ApprovalDecision::Denied));

        // 2) 账本按时间顺序留存两条，且互相不串
        let log = engine.decisions();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].action_id, approved.id);
        assert_eq!(log[0].decision, ApprovalDecision::Approved);
        assert_eq!(log[1].action_id, denied.id);
        assert_eq!(log[1].decision, ApprovalDecision::Denied);
        assert_eq!(engine.decision_counts(), (1, 1));

        // 3) 署名主体被捕获
        assert_eq!(log[0].actor, "tui-operator-7");
        assert_eq!(log[1].actor, "tui-operator-7");

        // 4) 决策时刻来自墙钟（UNIX 毫秒量级），不是进程相对的 Instant
        assert!(log[0].decided_at_unix_millis > 1_600_000_000_000, "{}", log[0].decided_at_unix_millis);
        assert!(log[1].decided_at_unix_millis >= log[0].decided_at_unix_millis);

        // 5) 审计自带上下文（pending 已出队，描述必须留在账本里）
        assert!(log[0].description.contains("/tmp/a"), "{}", log[0].description);

        assert!(engine.audit_summary().contains("Approved: 1"));
        assert!(engine.audit_summary().contains("Denied: 1"));
    }

    #[test]
    fn test_unknown_id_rejected_distinctly_from_already_decided() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let never = engine.submit(w("/tmp/real"));

        let unknown = engine.approve("a0000-never-issued").expect_err("unknown id must be rejected");
        assert_eq!(unknown.kind(), "unknown-id");
        assert_eq!(unknown, ApprovalError::UnknownId { id: "a0000-never-issued".into() });

        engine.approve(&never.id).expect("first decision stands");
        let repeated = engine.approve(&never.id).expect_err("repeat must be rejected");

        // 两类拒绝必须可分辨 —— 修复前二者都是同一句 "No pending action with id"。
        assert_ne!(unknown.kind(), repeated.kind());
        assert_eq!(repeated.kind(), "already-decided");
        assert!(!unknown.to_string().contains("already decided"), "{unknown}");
        assert!(repeated.to_string().contains("already decided"), "{repeated}");
    }

    #[test]
    fn test_decide_as_records_explicit_actor_and_reason() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(ActionType::ShellCommand { command: "rm -rf /tmp/z".into() });
        let outcome = engine.decide_as(
            &pa.id,
            ApprovalDecision::Denied,
            "profile=readonly",
            Some("destructive command".into()),
        );
        assert_eq!(outcome, Ok(ApprovalDecision::Denied));
        let entry = &engine.decisions()[0];
        assert_eq!(entry.actor, "profile=readonly");
        assert_eq!(entry.reason.as_deref(), Some("destructive command"));
        assert!(entry.description.contains("此操作将关闭"), "{}", entry.description);
    }

    #[test]
    fn test_ids_are_unique_across_restart() {
        // 模拟「重启」：全新引擎（新的会话纪元），序号从 0 重来。
        let mut before = ApprovalEngine::new(ApprovalMode::Suggest);
        let mut stale_ids = Vec::new();
        for _ in 0..8 {
            stale_ids.push(before.submit(w("/tmp/old")).id);
        }

        let mut after = ApprovalEngine::new(ApprovalMode::Suggest);
        let fresh: Vec<String> = (0..8).map(|_| after.submit(w("/tmp/new")).id).collect();

        // 重启后不得复现任一旧 id（纪元段不同）。
        for id in &fresh {
            assert!(!stale_ids.contains(id), "restart reissued stale id {id}");
        }
        // 同一引擎内唯一。
        let mut uniq = fresh.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), fresh.len());

        // 关键安全性断言：上一进程攥着的陈旧 id 不能批准本进程的另一个动作。
        let victim = fresh[0].clone();
        let stale = stale_ids[0].clone();
        assert_ne!(stale, victim);
        assert_eq!(
            after.approve(&stale),
            Err(ApprovalError::UnknownId { id: stale.clone() })
        );
        assert_eq!(after.pending_count(), 8, "陈旧 approval 不得消费任何待审动作");
        assert_eq!(after.decisions().len(), 0, "被拒的决策不得进账本");
        // 受害者仍在队列里，可被正确 id 正常决策。
        assert_eq!(after.approve(&victim), Ok(ApprovalDecision::Approved));
    }

    #[test]
    fn test_epoch_differs_between_engines_in_same_process() {
        // 同进程连续建两个引擎（测试常见的「重启」替身）：墙钟毫秒可能相同，
        // 故纪元必须额外含进程内自增序号，否则该性质会偶发失败。
        let mut a = ApprovalEngine::new(ApprovalMode::Suggest);
        let mut b = ApprovalEngine::new(ApprovalMode::Suggest);
        assert_ne!(a.epoch, b.epoch);
        let id_a = a.submit(w("/tmp/a")).id;
        let id_b = b.submit(w("/tmp/b")).id;
        assert_ne!(id_a, id_b);
        assert!(id_a.starts_with('a') && id_b.starts_with('a'));
    }

    #[test]
    fn test_batch_decisions_are_audited_individually() {
        // approve_all / deny_all 此前是 pending.clear()：这批决策一笔勾销。
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        engine.set_actor("bulk");
        let a = engine.submit(w("/tmp/a")).id;
        let b = engine.submit(w("/tmp/b")).id;

        assert_eq!(engine.approve_all(), 2);
        assert_eq!(engine.pending_count(), 0);
        assert_eq!(engine.decision_counts(), (2, 0));
        assert_eq!(engine.decision_for(&a), Some(ApprovalDecision::Approved));
        assert_eq!(engine.decision_for(&b), Some(ApprovalDecision::Approved));
        assert!(engine.decisions().iter().all(|e| e.actor == "bulk"));
        // 批量放行后重复决策同样被拒。
        assert_eq!(engine.approve(&a).unwrap_err().kind(), "already-decided");

        let c = engine.submit(w("/tmp/c")).id;
        assert_eq!(engine.deny_all(), 1);
        assert_eq!(engine.decision_for(&c), Some(ApprovalDecision::Denied));
        assert_eq!(engine.decision_counts(), (2, 1));
    }

    #[test]
    fn test_expire_stale_is_recorded_as_denied_not_dropped() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(w("/tmp/old"));

        assert_eq!(engine.expire_stale(Duration::from_secs(3600)), 0, "未到 TTL 不该过期");
        assert_eq!(engine.pending_count(), 1);

        assert_eq!(engine.expire_stale(Duration::ZERO), 1);
        assert_eq!(engine.pending_count(), 0);
        assert_eq!(engine.decision_for(&pa.id), Some(ApprovalDecision::Denied));
        let entry = &engine.decisions()[0];
        assert!(entry.reason.as_deref().unwrap_or("").contains("expired"), "{entry:?}");
        // 过期后再决策仍是 already-decided，而非 unknown。
        assert_eq!(engine.approve(&pa.id).unwrap_err().kind(), "already-decided");
    }

    #[test]
    fn test_audit_error_converts_into_string_for_question_mark_callers() {
        // 兼容既有 `Result<(), String>` 形态的调用点（`?` 直接可用）。
        fn legacy(id: &str, engine: &mut ApprovalEngine) -> Result<(), String> {
            engine.approve(id)?;
            Ok(())
        }
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert!(legacy("nope", &mut engine).is_err());
        let pa = engine.submit(w("/tmp/x"));
        assert!(legacy(&pa.id, &mut engine).is_ok());
    }

    // ══════════════════════════════════════════════════════════════
    // ⭐⭐⭐ 三态裁决反向锁（2026-10-05）
    // ══════════════════════════════════════════════════════════════

    /// ⭐⭐⭐ 端到端：`git push --force` 必须落在**硬拒**档。
    ///
    /// 【原缺陷两层叠加】
    /// ① `action_type_to_key` 把所有 git 动作映射成 `"git_push"`，
    ///    而画像里的键是 `"git_force_push"` ⇒ 键永不相交 ⇒ 从未命中。
    /// ② 即便命中，`require_approval` 也把 `Deny` 压成 `true`
    ///    ⇒ 退化成弹窗，点一下就放行。本测试同时锁住这两层。
    ///
    /// 注意用 **FullAuto** 构造：即便模式是「全自动」，硬拒依然成立。
    #[test]
    fn force_push_is_hard_denied_even_in_full_auto() {
        let engine = ApprovalEngine::new(ApprovalMode::FullAuto);
        let force_push = ActionType::GitOperation {
            description: "git push --force origin main".into(),
        };

        // ① 键必须真的能命中画像里那条规则
        assert_eq!(
            crate::l6_meta::nt_permission_profiles::action_type_to_key(&force_push),
            "git_force_push",
            "force push 必须映射到画像里声明的键，否则那条 Deny 永不可达"
        );
        assert!(
            crate::l6_meta::nt_permission_profiles::is_action_denied("git_force_push"),
            "内置画像里应当确实有 git_force_push 的 Deny（前提断言）"
        );

        // ② 裁决必须是 Deny
        let v = engine.action_verdict(&force_push);
        assert_eq!(v, ActionVerdict::Deny, "force push 必须是硬拒");
        assert!(v.is_blocked(), "is_blocked 必须为 true");
        assert!(v.needs_human(), "needs_human 也为 true（Deny 与 Ask 都要人过一眼）");
    }

    /// ⭐ 对照组：普通 `git push`（无 force）**不应**被硬拒。
    /// ⇒ 证明修复不是「把所有 git 动作一刀切拒掉」。
    #[test]
    fn plain_push_is_not_hard_denied() {
        let engine = ApprovalEngine::new(ApprovalMode::FullAuto);
        let plain = ActionType::GitOperation { description: "git push origin main".into() };
        assert_eq!(
            crate::l6_meta::nt_permission_profiles::action_type_to_key(&plain),
            "git_push"
        );
        assert_ne!(
            engine.action_verdict(&plain),
            ActionVerdict::Deny,
            "普通 push 不该被硬拒（否则修复就变成了过度拒绝）"
        );
    }

    /// ⭐ `read_secrets` 那条 Deny 也要能命中。
    /// ⚠️ 它当前**无生产 ActionType 能产生该键**（`FileWrite`/`FileEdit`
    /// 都映射到 `write_file`）⇒ 本测试断言的是**画像侧配置正确**，
    /// 而非「已经拦住了什么」—— 后者需要新增 `ActionType` 变体才成立。
    /// 如实标注，不假装它已生效。
    #[test]
    fn read_secrets_deny_is_declared_in_builtin_profile() {
        assert!(
            crate::l6_meta::nt_permission_profiles::is_action_denied("read_secrets"),
            "画像里声明了 read_secrets 的 Deny（前提断言）"
        );
    }

    /// ⭐⭐ 反向锁：三态**不可**被折叠回两态。
    /// 若有人把 `is_blocked` 写成 `needs_human`，本测试立刻红 ——
    /// 那正是 2026-10-05 修掉的降级。
    #[test]
    fn deny_must_not_collapse_into_ask() {
        assert!(ActionVerdict::Deny.is_blocked());
        assert!(ActionVerdict::Deny.needs_human());
        // ⭐ 关键差异：Ask 问了就放行，Deny 问了也不放行
        assert!(!ActionVerdict::Ask.is_blocked());
        assert!(ActionVerdict::Ask.needs_human());
        assert!(!ActionVerdict::Allow.needs_human());
        assert!(!ActionVerdict::Allow.is_blocked());
    }

    /// ⭐ 未知 git 子命令**不得 panic**，且大小写必须归一。
    /// 依据 AGENTS.md「生产代码禁 panic」—— 不能因没见过的子命令崩掉。
    #[test]
    fn unknown_git_subcommand_does_not_panic_and_is_case_insensitive() {
        let engine = ApprovalEngine::new(ApprovalMode::FullAuto);
        for d in ["git", "git rebase -i", "git bisect start", ""] {
            let a = ActionType::GitOperation { description: d.into() };
            let key = crate::l6_meta::nt_permission_profiles::action_type_to_key(&a);
            assert!(!key.is_empty(), "键不得为空串：{d:?}");
            let _ = engine.action_verdict(&a);
        }
        // 大写也须命中硬拒键，否则大写输入可绕过
        let upper = ActionType::GitOperation { description: "GIT PUSH --FORCE".into() };
        assert_eq!(
            crate::l6_meta::nt_permission_profiles::action_type_to_key(&upper),
            "git_force_push",
            "键匹配必须大小写不敏感"
        );
    }
}
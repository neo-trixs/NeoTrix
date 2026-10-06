use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// T06a：正典类型已收敛至 L0（neotrix-types），此处仅重导出＋引擎实现。
pub use neotrix_types::core::nt_core_approval::{ActionFingerprint, 
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
    /// **TOCTOU**：复核时呈现的动作与**提交时**的内容指纹不符。
    ///
    /// 含义：那次批准**不适用**于眼前这份内容（内容在批准后被替换过）。
    /// ⛔ 携带两个指纹以便定位：提交时的是什么、现在拿来的又是什么。
    ContentChanged {
        id: String,
        /// 提交时记录的内容指纹（= 用户当时看到的）。
        approved: String,
        /// 复核时呈现的内容指纹。
        presented: String,
    },
}

impl ApprovalError {
    /// 机器可读判别标签，供调用方分支与测试断言。
    pub fn kind(&self) -> &'static str {
        match self {
            Self::UnknownId { .. } => "unknown-id",
            Self::AlreadyDecided { .. } => "already-decided",
            Self::ContentChanged { .. } => "content-changed",
        }
    }

    /// 被拒的 id。
    pub fn id(&self) -> &str {
        match self {
            Self::UnknownId { id } => id,
            Self::AlreadyDecided { id, .. } => id,
            Self::ContentChanged { id, .. } => id,
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
            // ⚠️ 两个指纹都打出来：TOCTOU 排查要靠它们定位「批准的是哪份、
            //   递来的是哪份」，只打一个等于让排查者自己去猜。
            Self::ContentChanged { id, approved, presented } => {
                write!(
                    f,
                    "approval id '{id}' content changed since approval \
(approved {approved}, presented {presented}) — 批准不适用于眼前这份内容"
                )
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
/// 三态审批裁决（2026-10-05）。
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

    /// 是否**不可放行**。这是 `Deny` 存在的唯一理由 ——
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

    /// 三态判据（2026-10-05）——`require_approval` 的**权威版本**。
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
            // 硬拒：**不是**「问一下就能过」。
            return ActionVerdict::Deny;
        }
        if crate::l6_meta::nt_permission_profiles::is_action_allowed(action_key) {
            return ActionVerdict::Allow;
        }
        // **委托给 `require_approval`**，不自己再写一遍 mode 判据。
        //
        // 【缺陷（2026-10-06 修）】本方法此前是
        // `Suggest | AutoEdit => Ask; FullAuto => Allow`
        // ⇒ **丢掉了 AutoEdit 的文件类白名单**：
        // `require_approval` 里 AutoEdit 对 `FileWrite`/`FileCreate`/`FileEdit`
        // 返回 `false`（免审批），而本「权威版」却对同一动作返回 `Ask`。
        // ⇒ 同一个动作，两处判据给出**不同答案**，而本方法是
        //   `ActionSandbox` 的**硬拒判据** ⇒ AutoEdit 下所有文件写**全被当 Ask**。
        //
        // 【判据依据】判据只该有一份。两处各写一遍 mode 分支，
        // 漂移只是时间问题 —— 实测已漂移（AutoEdit 白名单）。
        // ⇒ profile 的 Deny/Allow 仍在本方法里**先判**（那两档优先于 mode，
        //   是设计的一部分）；余下的 mode 判定一律委托。
        if self.require_approval(action) {
            ActionVerdict::Ask
        } else {
            ActionVerdict::Allow
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
        // 执行前复核用的内容指纹（TOCTOU 防护，见 `ActionFingerprint` 文档）
        let content_fingerprint = ActionFingerprint::of(&action);
        let pa = PendingAction {
            id,
            action_type: action,
            description,
            content_fingerprint,
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

    /// **复核批准**：只有当 `action` 的内容指纹与提交时**完全一致**才批准。
    ///
    /// ## 【缺陷（2026-10-06 修）】`approve(id)` **只凭 id**，
    /// 而执行发生在**稍后、别处** ⇒ 从批准到执行之间内容**没有任何一步被复核**。
    /// 若这期间动作被替换（队列重渲染 / 子代理中转 / handoff 转述），
    /// 那次批准会被**原样用在另一份内容上** —— 这就是经典的 TOCTOU。
    ///
    /// ## 【为什么不能靠 `approve` 自己解决】
    /// `approve(id)` 无从知道调用方「即将执行的是哪一份」——
    /// 它只拿到一个 id。把内容传进来是**唯一**能在批准时建立绑定的办法。
    ///
    /// ## 失败方向（判据）
    /// 指纹不符 ⇒ **`Err`，且不写入 `decided` 账本**
    /// ⇒ 既不批准、也不留下「已批准」的痕迹。
    /// 依据同源判据：**静默降级只允许朝严格方向**；
    /// 若此处回落成「照样批准」，则整个指纹机制只是装饰。
    ///
    /// ⚠️ 与 [`Self::approve`] 的关系：后者保留原样（既有调用方不动），
    /// 但**要真正防住 TOCTOU 的调用方必须用本方法**。
    pub fn approve_if_unchanged(
        &mut self,
        id: &str,
        action: &crate::l6_meta::nt_approval::ActionType,
    ) -> Result<ApprovalDecision, ApprovalError> {
        let pa = self
            .pending
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| ApprovalError::UnknownId { id: id.to_string() })?;
        if !pa.content_fingerprint.matches(action) {
            // ⛔ 刻意**不**写入 decided：否则账本会出现一条「已批准」的假记录，
            //   而实际上没有任何内容被批准过。
            log::warn!(
                "approval TOCTOU: id={id} 内容指纹不符（提交时 {} vs 复核时 {}）—— 拒绝批准",
                pa.content_fingerprint,
                ActionFingerprint::of(action)
            );
            return Err(ApprovalError::ContentChanged {
                id: id.to_string(),
                approved: pa.content_fingerprint.as_str().to_string(),
                presented: ActionFingerprint::of(action).as_str().to_string(),
            });
        }
        self.approve(id)
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

/// 预览截断（按**字符**）。
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

/// 内容指纹（吸收 `uber/ADR`，2026-10-06）。
///
/// 【缺陷（2026-10-06 修）】`describe_action` 此前把 `content_preview` /
/// `diff` 的**原文**（最多 60 字符）拼进 `description`。而 `description`
/// 会进 [`PendingAction`]，并被 `ApprovalAuditEntry::description`
/// **快照进审计轨迹** ⇒ **每写一次文件，就把它开头 60 个字符留在审计里**。
///
/// ## 【实测的泄漏面】
/// 写 `.env` / `credentials` / `.npmrc` 这类文件时，那 60 个字符**就是密钥本身**
/// （`API_KEY=sk-...`）。审计轨迹是**长期留存 + 可能导出**的结构
/// ⇒ 一次普通的文件写就把凭据抄进了本不该有它的存储。
///
/// ## 【源】`uber/ADR`（Apache-2.0，已部署于 Uber 生产，MLSys 2026）
/// 其 `run_manifest` 明确：*"Paths, directory names, host identifiers,
/// environment values, prompts, and file contents are not stored"*
/// —— 要能证明「审批的就是这份内容」，只需记 **SHA-256 摘要**，不必记内容。
///
/// ## 为什么指纹是**够用**的
/// 审批要回答的问题是「**这份**内容准不准」，不是「内容写了什么」。
/// 路径 + 字符数 + 摘要三者合起来：人能定位文件、能看到规模、
/// 事后能**验证**内容未被替换 —— 而不必把内容抄进审计。
fn content_fingerprint(s: &str) -> String {
    let digest = hex::encode(<sha2::Sha256 as sha2::Digest>::digest(s.as_bytes()));
    format!("{} chars, sha256:{}", s.chars().count(), &digest[..12])
}

/// 键名是否「看起来是密钥名」。
///
/// ## ⛔ 为什么必须**整段命中**而不是子串包含（2026-10-06 实测教训）
/// 首版用 `key.contains("key")`，结果 **`KEYBOARD=1` 被遮蔽** ——
/// 而金丝雀测试抓到它时，`--port`/`LEVEL`/`a=1` 都侥幸没被误伤。
/// ⇒ 一旦遮蔽开始误伤普通赋值，命令就变得**不可读**，
///   而「审批人必须看清要跑什么」正是这道闸存在的理由
///   ⇒ **遮蔽误伤的代价 = 审批失效**，比漏遮更难发现。
/// ⇒ 只在按 `_`/`-`/`.`/数字切出的**整段**命中提示词时才判定为密钥名。
fn is_secret_key(key: &str) -> bool {
    const SECRET_HINTS: [&str; 8] = [
        "token", "key", "secret", "password", "passwd", "api", "auth", "credential",
    ];
    key.split(|c: char| c == '_' || c == '-' || c == '.' || c.is_ascii_digit())
        .filter(|seg| !seg.is_empty())
        .any(|seg| SECRET_HINTS.contains(&seg.to_ascii_lowercase().as_str()))
}

/// 遮蔽赋值型密钥（`KEY=value` 与 `--flag VALUE` **两种形态**）。
///
/// ## 为什么两种形态都要（2026-10-06 实测教训）
/// 首版只处理 `KEY=value`，金丝雀测试立刻抓到
/// `deploy --api-key sk_live_...` —— **CLI 里 `--flag VALUE` 比 `KEY=value` 更常见**
/// ⇒ 只做赋值形态等于漏掉主流用法。
///
/// ## 遮蔽后保留什么
/// · **键名保留**（要让人知道在传什么）
/// · **被遮值的长度保留**（要让人知道「这里原本有个非空值」，
///   否则 `KEY=` 与 `KEY=x` 在审计里长得一样）
///
/// ⛔ 这是**减害**而非保证：把密钥拼进命令正文
/// （`curl .../$(cat ~/.aws/credentials)`）不在此覆盖内。依据同源判据：
/// 不宣称做不到的事。
fn redact_assigned_secrets(s: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut mask_next = false;
    for tok in s.split_whitespace() {
        if mask_next {
            out.push(format!("<redacted:{}chars>", tok.chars().count()));
            mask_next = false;
        } else if let Some((k, v)) = tok.split_once('=') {
            if !v.is_empty() && is_secret_key(k) {
                out.push(format!("{k}=<redacted:{}chars>", v.chars().count()));
            } else {
                out.push(tok.to_owned());
            }
        } else {
            // `--api-key VALUE` 形态（值在**下一个** token）
            if tok.starts_with('-') && is_secret_key(tok.trim_start_matches('-')) {
                mask_next = true;
            }
            out.push(tok.to_owned());
        }
    }
    out.join(" ")
}

fn describe_action(action: &ActionType) -> String {
    match action {
        // ⚠️ 内容/diff 一律**只留指纹**，不再抄原文（见 content_fingerprint 的缺陷说明）
        ActionType::FileWrite { path, content_preview } => {
            format!("📝 Write {} ({})", path, content_fingerprint(content_preview))
        }
        ActionType::FileCreate { path } => format!("📄 Create {}", path),
        ActionType::FileEdit { path, diff } => {
            format!("✏️ Edit {} ({})", path, content_fingerprint(diff))
        }
        // 命令与参数：保留本体（审批人必须看清），但遮掉赋值型密钥的值
        ActionType::ShellCommand { command } => {
            format!("💻 Run: {}", preview_or_full(&redact_assigned_secrets(command)))
        }
        ActionType::GitOperation { description } => {
            format!("🔧 Git: {}", preview_or_full(&redact_assigned_secrets(description)))
        }
        ActionType::Other { tool, args } => {
            format!("🔧 Tool {}: {}", tool, preview_or_full(&redact_assigned_secrets(args)))
        }
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
        assert_eq!(ApprovalMode::from_str("suggest").unwrap(), ApprovalMode::Suggest);
        assert_eq!(ApprovalMode::from_str("auto-edit").unwrap(), ApprovalMode::AutoEdit);
        assert_eq!(ApprovalMode::from_str("full-auto").unwrap(), ApprovalMode::FullAuto);
        assert_eq!(ApprovalMode::from_str("yolo").unwrap(), ApprovalMode::FullAuto);
        // 2026-10-06 语义变更：未知值从 `None` 改为 **`Err`**
        // （静默忽略 → 用户要的档位没生效且无任何错误）
        let e = ApprovalMode::from_str("unknown").expect_err("未知档位必须 Err");
        assert!(e.contains("suggest"), "错误信息应列出可用档位：{e}");
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
    // 三态裁决反向锁（2026-10-05）
    // ══════════════════════════════════════════════════════════════

    /// 端到端：`git push --force` 必须落在**硬拒**档。
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

    /// 对照组：普通 `git push`（无 force）**不应**被硬拒。
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

    /// `read_secrets` 那条 Deny 也要能命中。
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

    /// 反向锁：三态**不可**被折叠回两态。
    /// 若有人把 `is_blocked` 写成 `needs_human`，本测试立刻红 ——
    /// 那正是 2026-10-05 修掉的降级。
    #[test]
    fn deny_must_not_collapse_into_ask() {
        assert!(ActionVerdict::Deny.is_blocked());
        assert!(ActionVerdict::Deny.needs_human());
        // 关键差异：Ask 问了就放行，Deny 问了也不放行
        assert!(!ActionVerdict::Ask.is_blocked());
        assert!(ActionVerdict::Ask.needs_human());
        assert!(!ActionVerdict::Allow.needs_human());
        assert!(!ActionVerdict::Allow.is_blocked());
    }

    /// 未知 git 子命令**不得 panic**，且大小写必须归一。
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
#[cfg(test)]
mod action_verdict_locks {
    //! `action_verdict` 的锁 —— 本函数此前**零测试**，
    //! 而它是 `ActionSandbox` 的**硬拒判据**（deny/ask 二态全靠它）。
    //!
    //! 零测试的「权威判据」比有 bug 的非权威判据更危险：
    //! 所有人都会因为「它是权威的」而**不再怀疑它**。
    //!
    //! 【核心不变量】对任意动作，`action_verdict` 必须是
    //! `require_approval` 的**忠实三态化**：除 profile 的 Deny/Allow 先判外，
    //! 「要不要问」必须与 `require_approval` **完全一致**。

    use super::*;
    use crate::l6_meta::nt_approval::ActionVerdict;

    fn engine(mode: ApprovalMode) -> ApprovalEngine {
        ApprovalEngine::new(mode)
    }

    fn fw() -> ActionType {
        ActionType::FileWrite { path: "x".into(), content_preview: String::new() }
    }
    fn sh(cmd: &str) -> ActionType {
        ActionType::ShellCommand { command: cmd.into() }
    }

    /// **一致性不变量（穷举 mode × 动作）**：
    /// `Ask ⇔ require_approval == true`，且**永远不产出 `Deny`**（未命中 profile 时）。
    ///
    /// 这条锁死了「委托给 `require_approval`」这个修法 ——
    /// 任何人不小心把 mode 分支重新写回本函数，这条立刻红。
    #[test]
    fn verdict_is_faithful_tristate_of_require_approval() {
        let actions = [
            ("FileWrite", fw()),
            ("FileCreate", ActionType::FileCreate { path: "x".into() }),
            ("FileEdit", ActionType::FileEdit { path: "x".into(), diff: String::new() }),
            ("Shell", sh("ls")),
            ("Git", ActionType::GitOperation { description: "commit".into() }),
            ("Other", ActionType::Other { tool: "web_search".into(), args: "q=rust".into() }),
        ];
        for mode in [ApprovalMode::Suggest, ApprovalMode::AutoEdit, ApprovalMode::FullAuto] {
            let e = engine(mode);
            for (name, a) in &actions {
                let v = e.action_verdict(a);
                let req = e.require_approval(a);
                match v {
                    ActionVerdict::Ask => assert!(
                        req,
                        "[{mode:?}] {name}: 判据说 Ask 但 require_approval 说免审批 ⇒ 两份判据漂移"
                    ),
                    ActionVerdict::Allow => assert!(
                        !req,
                        "[{mode:?}] {name}: 判据说 Allow 但 require_approval 说要审批 ⇒ 两份判据漂移"
                    ),
                    ActionVerdict::Deny => panic!(
                        "[{mode:?}] {name}: 未命中 profile 的 Deny 却返回 Deny \
                         （deny 只能来自 profile 规则）"
                    ),
                }
            }
        }
    }

    /// **反向锁：AutoEdit 的文件类白名单**（本函数此前的真实缺陷）。
    ///
    /// `require_approval` 对 `AutoEdit` 的 `FileWrite`/`FileCreate`/`FileEdit`
    /// 明确返回 `false`（免审批）；而「权威版」此前对同一动作返回 `Ask`
    /// ⇒ AutoEdit 下**所有文件写全被当 Ask**，档位名存实亡。
    #[test]
    fn auto_edit_whitelist_file_ops_is_preserved() {
        let e = engine(ApprovalMode::AutoEdit);
        for (name, a) in [
            ("FileWrite", fw()),
            ("FileCreate", ActionType::FileCreate { path: "x".into() }),
            ("FileEdit", ActionType::FileEdit { path: "x".into(), diff: String::new() }),
        ] {
            assert_eq!(
                e.action_verdict(&a),
                ActionVerdict::Allow,
                "AutoEdit 下 {name} 应免审批（`--approval-mode auto-edit` 的承诺就是这条）"
            );
        }
        // 但**命令/git/未分类**仍要审批 —— 白名单不是「全放开」
        for (name, a) in [
            ("Shell", sh("ls")),
            ("Git", ActionType::GitOperation { description: "commit".into() }),
            ("Other", ActionType::Other { tool: "web_search".into(), args: String::new() }),
        ] {
            assert_eq!(
                e.action_verdict(&a),
                ActionVerdict::Ask,
                "AutoEdit 下 {name} 仍需审批 —— 白名单只覆盖文件类"
            );
        }
    }

    /// 三个档位各自的整体形态（一张表钉住全貌）。
    #[test]
    fn three_modes_have_the_documented_shape() {
        assert_eq!(engine(ApprovalMode::Suggest).action_verdict(&fw()), ActionVerdict::Ask);
        assert_eq!(engine(ApprovalMode::FullAuto).action_verdict(&fw()), ActionVerdict::Allow);
        assert_eq!(engine(ApprovalMode::FullAuto).action_verdict(&sh("rm -rf /")), ActionVerdict::Allow);
        assert_eq!(engine(ApprovalMode::Suggest).action_verdict(&sh("rm -rf /")), ActionVerdict::Ask);
    }
}

#[cfg(test)]
mod audit_content_leak_tests {
    //! **审计轨迹不得抄录文件内容**（吸收 `uber/ADR`，2026-10-06）。
    //!
    //! 【缺陷（已修）】`describe_action` 曾把 `content_preview` / `diff` 的
    //! **原文**拼进 `description`，而 `description` 被
    //! `ApprovalAuditEntry::description` **快照进审计轨迹**
    //! ⇒ 写一次 `.env` 就把 `API_KEY=sk-...` 的前 60 字符抄进长期留存的审计。
    //!
    //! 【源】`uber/ADR`（Apache-2.0，Uber 生产，MLSys 2026）：其 `run_manifest`
    //! 明确 *"file contents are not stored"*，只记 SHA-256 摘要。
    //!
    //! 【测法】用**金丝雀串**：断言它**不出现**在 description 与审计条目里。
    //! 选金丝雀而不是断言格式，是因为「泄漏没了」这件事本身才是要锁的不变量 ——
    //! 断言输出格式会把实现细节钉死，让人为了改格式而删掉这条防护。

    use super::*;

    /// 金丝雀：一段只可能来自「内容被抄走」的串。
    const CANARY: &str = "CANARY_SECRET_sk_live_ABC123";

    fn audit_of(action: ActionType) -> (String, ApprovalAuditEntry) {
        let mut e = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = e.submit(action);
        // 走一次决策，让审计条目真的产生。
        // ⭐ 2026-10-06 自查修正：此前写的是 `e.approve(&pa.id);` ——
        // **丢弃了 `Result`**（编译警告 `unused Result that must be used`）。
        // ⚠️ 那正是本会话一直在修的「静默失效」族，**出现在我自己的测试里**：
        // 一旦 `approve` 返回 `Err`，测试不会在这里停下，而会跑到后面的
        // `.expect("应有一条审计")` 给出**误导性**报错（真正的失败点被掩盖）。
        e.approve(&pa.id).expect("提交后立即批准必须成功");
        let entry = e.decisions().last().cloned().expect("应有一条审计");
        (pa.description.clone(), entry)
    }

    /// `FileWrite` 的内容**不得**出现在描述或审计里（金丝雀测试）。
    #[test]
    fn file_write_content_never_reaches_description_or_audit() {
        let content = format!("API_KEY={CANARY}\nDATABASE_URL=postgres://x");
        let (desc, entry) = audit_of(ActionType::FileWrite {
            path: "/srv/app/.env".into(),
            content_preview: content.clone(),
        });
        assert!(
            !desc.contains(CANARY),
            "金丝雀出现在描述里 ⇒ 内容仍被抄走：{desc}"
        );
        assert!(
            !entry.description.contains(CANARY),
            "金丝雀出现在审计条目里 ⇒ 内容仍被抄走：{}",
            entry.description
        );
        // 但审计必须仍能回答「审的是哪份内容」⇒ 路径 + 摘要 + 规模都在
        assert!(desc.contains("/srv/app/.env"), "路径应保留（审批人需要定位）：{desc}");
        assert!(desc.contains("chars"), "应记内容规模：{desc}");
        assert!(desc.contains("sha256:"), "应记内容摘要：{desc}");
    }

    /// `FileEdit` 的 diff **不得**出现在描述或审计里。
    #[test]
    fn file_edit_diff_never_reaches_description_or_audit() {
        let diff = format!("-password = old\n+password = {CANARY}");
        let (desc, entry) = audit_of(ActionType::FileEdit {
            path: "config.toml".into(),
            diff,
        });
        assert!(!desc.contains(CANARY), "diff 原文泄漏进描述：{desc}");
        assert!(!entry.description.contains(CANARY), "diff 原文泄漏进审计：{}", entry.description);
    }

    /// 命令**本体必须保留**（否则审批失效），但**赋值型密钥的值被遮蔽**。
    #[test]
    fn command_body_kept_but_assigned_secret_masked() {
        let (desc, _e) = audit_of(ActionType::ShellCommand {
            command: format!("deploy --api-key {CANARY} --force"),
        });
        // 命令本体（审批人必须看清要跑什么）
        assert!(desc.contains("deploy"), "命令本体必须保留：{desc}");
        assert!(desc.contains("--force"), "命令本体必须保留：{desc}");
        // 密钥值不得出现
        assert!(!desc.contains(CANARY), "命令行里的密钥值泄漏：{desc}");
    }

    /// `KEY=value` 形态：键名保留、值遮蔽。
    #[test]
    fn assigned_secret_value_masked_but_key_visible() {
        let red = super::redact_assigned_secrets(&format!("TOKEN={CANARY} echo hi"));
        assert!(!red.contains(CANARY), "值未遮蔽：{red}");
        assert!(red.contains("TOKEN="), "键名应保留（要让人知道在传什么）：{red}");
        assert!(red.contains("echo hi"), "非密钥部分应原样保留：{red}");
        // 遮蔽后要**标明长度** —— 审批人需要知道「这里原本有个长值」
        assert!(red.contains("chars"), "应记录被遮值的长度：{red}");
    }

    /// 非密钥赋值**不得**被误遮（否则命令变得不可读 = 审批失效）。
    #[test]
    fn ordinary_assignments_are_not_masked() {
        for cmd in [
            "java -jar app.jar --port=8080",
            "LEVEL=debug cargo test",
            "KEYBOARD=1 make",
            "echo a=1 b=2",
        ] {
            let red = super::redact_assigned_secrets(cmd);
            assert_eq!(red, cmd, "普通赋值不该被遮蔽：{cmd} → {red}");
        }
    }

    /// 摘要必须**对内容敏感**且**稳定** —— 否则它证明不了任何事。
    #[test]
    fn fingerprint_is_stable_and_content_sensitive() {
        let a = super::content_fingerprint("hello");
        let b = super::content_fingerprint("hello");
        let c = super::content_fingerprint("hellp");
        assert_eq!(a, b, "同内容必须同摘要（否则审计无法复核）");
        assert_ne!(a, c, "内容不同必须不同摘要（否则换内容查不出来）");
        assert!(a.contains(&a.split("sha256:").nth(1).unwrap()[..12]), "摘要片段应自洽");
        // 大小也要进去：否则「空内容」与「被清空的内容」无法区分
        assert!(super::content_fingerprint("").contains("0 chars"));
    }
}

#[cfg(test)]
mod approval_toctou_tests {
    //! **审批必须绑定内容，而非只凭 id**（2026-10-06）。
    //!
    //! 【缺陷】`approve(id)` 只凭 id，而执行发生在**稍后、别处**
    //! ⇒ 批准到执行之间内容**没有任何一步被复核**。
    //! 动作若在这期间被替换（队列重渲染 / 子代理中转 / handoff 转述），
    //! 那次批准会被**原样用在另一份内容上**（经典 TOCTOU）。
    //!
    //! 【为什么这类缺陷特别危险】它**不会让任何现有测试变红** ——
    //! 现有测试都是「提交什么就批准什么」，而 TOCTOU 恰恰发生在
    //! 「批准的」与「执行的」**不是同一份**的那条路径上。
    //! ⇒ 必须用「提交 A、拿 B 去批准」这种**不自然**的调用来暴露它。

    use super::*;
    use neotrix_types::core::nt_core_approval::ActionFingerprint;

    fn fw(path: &str, body: &str) -> ActionType {
        ActionType::FileWrite { path: path.into(), content_preview: body.into() }
    }

    /// 同一内容 ⇒ 指纹稳定、复核通过。
    #[test]
    fn same_content_approves() {
        let mut e = ApprovalEngine::new(ApprovalMode::Suggest);
        let a = fw("/etc/app.conf", "debug=false");
        let pa = e.submit(a.clone());
        assert_eq!(
            e.approve_if_unchanged(&pa.id, &a).unwrap(),
            ApprovalDecision::Approved,
            "内容未变时必须照常批准"
        );
    }

    /// **核心用例**：提交 A，却拿 B 去批准 ⇒ 必须 `Err`，且**账本里无记录**。
    #[test]
    fn substituted_content_is_REFUSED_and_leaves_no_approval_record() {
        let mut e = ApprovalEngine::new(ApprovalMode::Suggest);
        let submitted = fw("/etc/app.conf", "debug=false");
        let pa = e.submit(submitted);
        // 攻击/意外：执行侧递来的是**另一份**内容
        let substituted = fw("/etc/app.conf", "debug=false\nbackdoor=true");

        let err = e
            .approve_if_unchanged(&pa.id, &substituted)
            .expect_err("内容被替换后必须拒绝批准");
        assert_eq!(err.kind(), "content-changed");
        assert_eq!(err.id(), pa.id);

        // 最关键的一条：**账本里绝不能出现「已批准」**
        // 否则调用方只看 `decision_for` 会以为「已经批过了」
        assert!(
            e.decision_for(&pa.id).is_none(),
            "被拒的复核不得在账本留下任何决策痕迹"
        );
        assert!(
            e.decisions().is_empty(),
            "审计轨迹也不该为这次未发生的批准记一笔：{:?}",
            e.decisions()
        );
    }

    /// 换路径也算内容变了（路径是执行结果的一部分）。
    #[test]
    fn path_change_counts_as_content_change() {
        let mut e = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = e.submit(fw("/home/u/notes.md", "hello"));
        let elsewhere = fw("/home/u/.bashrc", "hello");
        assert!(
            e.approve_if_unchanged(&pa.id, &elsewhere).is_err(),
            "同内容不同路径 ⇒ 批准不适用（用户批准的是那个文件）"
        );
    }

    /// 每一类动作都要能区分（不能只有 FileWrite 有指纹）。
    #[test]
    fn every_action_kind_is_fingerprinted() {
        let pairs: Vec<(&str, ActionType, ActionType)> = vec![
            ("FileWrite", fw("a", "x"), fw("a", "y")),
            ("FileCreate", ActionType::FileCreate { path: "a".into() }, ActionType::FileCreate { path: "b".into() }),
            ("FileEdit", ActionType::FileEdit { path: "a".into(), diff: "x".into() }, ActionType::FileEdit { path: "a".into(), diff: "y".into() }),
            ("Shell", ActionType::ShellCommand { command: "ls".into() }, ActionType::ShellCommand { command: "rm -rf /".into() }),
            ("Git", ActionType::GitOperation { description: "commit".into() }, ActionType::GitOperation { description: "push --force".into() }),
            ("Other", ActionType::Other { tool: "t".into(), args: "a".into() }, ActionType::Other { tool: "t".into(), args: "b".into() }),
        ];
        for (name, a, b) in pairs {
            assert_ne!(
                ActionFingerprint::of(&a),
                ActionFingerprint::of(&b),
                "{name}: 两种内容必须算出不同指纹"
            );
            assert_eq!(
                ActionFingerprint::of(&a),
                ActionFingerprint::of(&a.clone()),
                "{name}: 同内容必须算出同一指纹（否则「同一动作」无从判定）"
            );
        }
    }

    /// **动作类型本身也参与指纹** —— 否则 `FileWrite` 与 `FileCreate`
    /// 恰好内容相同时会撞指纹。
    #[test]
    fn action_kind_is_part_of_the_fingerprint() {
        let a = ActionFingerprint::of(&fw("p", "body"));
        let b = ActionFingerprint::of(&ActionType::FileEdit {
            path: "p".into(),
            diff: "body".into(),
        });
        assert_ne!(a, b, "不同动作类型即便字段值巧合相同也不得撞指纹");
    }

    /// 未知 id 仍走 `UnknownId`（不因新增分支而改变既有语义）。
    #[test]
    fn unknown_id_still_reports_unknown() {
        let mut e = ApprovalEngine::new(ApprovalMode::Suggest);
        let a = fw("a", "b");
        let err = e.approve_if_unchanged("a0000000000000000000", &a).unwrap_err();
        assert_eq!(err.kind(), "unknown-id");
    }

    /// **失败朝严格方向**：TOCTOU 被拒后，原动作**仍可**被正确批准。
    /// （拒绝的是「这份不符的内容」，不是「这个动作永远不许批」。）
    #[test]
    fn refusing_a_substitution_does_not_poison_the_original_action() {
        let mut e = ApprovalEngine::new(ApprovalMode::Suggest);
        let real = fw("/etc/app.conf", "debug=false");
        let pa = e.submit(real.clone());
        assert!(e.approve_if_unchanged(&pa.id, &fw("/etc/app.conf", "evil")).is_err());
        // 拿**正确**内容再来 ⇒ 应当能批（此刻尚未有任何决策）
        assert_eq!(
            e.approve_if_unchanged(&pa.id, &real).unwrap(),
            ApprovalDecision::Approved
        );
    }
}

//! `nt_channel` — IM 渠道适配层（吸收 `dsh-im` 的多渠道接入架构）。
//!
//! **为什么不是抄协议**：dsh-im 支持 11 个平台，那些协议胶水（飞书长连接、
//! 微信 iLink、WhatsApp Baileys、Discord Gateway…）合计十万行量级的厂商私有
//! 协议，抄进来既不可维护也不该抄。可迁移的是它的**架构**，本模块就是那份架构：
//!
//! - 一个 `ChannelAdapter` trait：入站收消息、出站发消息、探活；
//! - 一个注册表：渠道按 id 注册，坏渠道不拖累别的；
//! - 每机器人可绑定（别名 / 白名单 / 会话）；**`model` 与 `token_env` 存在库里
//!   但 serve 尚未生效** —— 一个渠道只有一个共享适配器，详见 `nt_channel_serve`
//!   里的 `warn_shared_adapter_once`；
//! - 访问模式（`open` / `allow` / `dm_only`）在**入站最前面**判定；
//! - 指令（`/new` `/stop` `/help` `/status`）在跑轮前截；
//! - 超时后延迟补发（`pending_deliveries`）。
//!
//! neobot 域模型**本来就是 IM 形状**：`conversations(kind dm|group)` +
//! `members` + `outbox`(attempts/available_at 退避) + `attachments` + 免打扰/未读。
//! 故渠道层不是外挂的怪物，而是往已有的容器里灌消息、从已有的 outbox 取消息。
//!
//! 凭据律：**token 永不落 `conversations`/状态接口**。`channel_bots` 只存
//! token 的**环境变量名**（与既有 `providers.key_env` 同一套做法），值由
//! 适配器在发送/接收时现读。状态查询只回变量名，与 `NeobotProviderItem`
//! 的脱敏口径一致。
//!
//! 身份律：入站消息先判访问模式**再**判是不是指令 —— 未授权来源的
//! `/new` 不该有任何效果，更不该被回一句「已重置」。

use std::collections::BTreeMap;

use crate::nt_error::NtBotError;

/// 访问模式（入站闸门；fail-closed 缺省 `allow` = 空白名单 = 谁都不理）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// 谁都能用（公开机器人；用户自己开的，风险自负）。
    Open,
    /// 只有名单里的来源能用。
    Allow,
    /// 只响应私聊，群里一律不理。
    DmOnly,
}

impl AccessMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Allow => "allow",
            Self::DmOnly => "dm_only",
        }
    }

    /// 解析；**未知即 `Allow`（空白名单 = 全拒）**，不猜成 `Open`。
    ///
    /// 反向 fail-closed：配置写错一个字符就变成「谁都不理」，
    /// 而不是「对所有人开放」。后者是安全事故，前者是用户会来问的怪事。
    pub fn parse(raw: &str) -> Self {
        match raw.trim() {
            "open" => Self::Open,
            "dm_only" | "dm" => Self::DmOnly,
            _ => Self::Allow,
        }
    }
}

/// 一份入站附件（**平台内的引用**，还没落到本机）。
///
/// 刻意不直接放本地路径：适配器不该知道 `data_dir` 在哪，那是 dispatch 的事。
/// 平台给出的是「凭据」（Telegram 的 `file_id`），由 dispatch 决定存到哪。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundAttachment {
    /// 平台内的附件 id（`fetch_attachment` 凭它取）。
    pub id: String,
    /// 原始文件名（可空 → 用平台默认名）。
    pub name: String,
    /// 平台报的字节数（下载前不可信，落地后以真实大小为准）。
    pub size: i64,
}

/// 入站消息（渠道适配器统一后的形状）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundMessage {
    /// 平台内的会话标识（Telegram 的 `chat_id`、Discord 的 `channel_id`…）。
    pub chat: String,
    /// 群 = false / 私聊 = true。
    pub is_dm: bool,
    /// 发送者平台内 id。
    pub sender: String,
    /// 发送者显示名（可空）。
    pub sender_name: String,
    /// 正文（已去指令前缀、已去掉 bot 提及）。
    pub text: String,
    /// 平台消息 id（去重与「编辑同一条」用）。
    pub message_id: String,
    /// 附件（**尚未下载**；文本消息为空 vec）。
    pub attachments: Vec<InboundAttachment>,
    /// 被回复的消息 id（若有）。
    pub reply_to: Option<String>,
}

/// 出站消息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundMessage {
    pub chat: String,
    pub text: String,
    /// 附件本地路径（渠道不支持就降级为纯文字，如实回报）。
    pub attachments: Vec<String>,
    /// 要替换（编辑）的消息 id；无则发新消息。
    ///
    /// **意图**：把**原消息**改成新内容，而不是在聊天里再堆一条
    /// （早先的实现三段各自为政，用户看到的是重复的两条）。
    ///
    /// 现在三段是接上的：
    ///
    /// 1. 直发路径（`deliver_result` / `sweep_pending`）填 `Some(origin_message)`；
    /// 2. outbox 的 payload 带**可选键** `edit_of`，缺省等价于 `None`
    ///    （老 payload 一条都没有这个键，照发不误）；
    /// 3. Telegram 适配器的 `send()` 读它，走 `editMessageText`
    ///    （`chat_id` + `message_id` 必需，平台只让 bot 改**自己发过**的消息）。
    ///
    /// **改不成就退回发新消息，并记账**：渠道侧用
    /// `TelegramChannel::last_send_outcome()` 报「编辑了 / 降级了（原因）/
    /// 内容本就一致（没发新消息）」。`send()` 的返回值只有消息 id，
    /// 「编辑成功」和「降级发新」在它上面分不开 —— 要区分就读那个记账。
    /// 别的渠道（尚未实现的）继续按「发新消息」处理。
    ///
    /// **一个仍未闭合的事实**（不是本字段的缺陷，是调用方的取值为政）：
    /// `deliver_result` / `sweep_pending` 传的是**用户那条入站消息**的 id，
    /// 而 bot 不能编辑用户的消息，平台会回 `message to edit not found`，
    /// 于是每次补发都会先失败一次再降级。要让编辑真的生效，需要先发一条
    /// 占位消息并**记住它自己的 message id**（那是 `nt_store` 侧的事）。
    /// 半接状态时代的待办见 `docs/architecture/DESIGN-CHANNEL-DISPATCH.md` 待办 7
    /// （「三段一起补」那部分已落地，剩「占位消息」那部分）。
    pub edit_of: Option<String>,
}

/// 渠道探活结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChannelHealth {
    pub ok: bool,
    /// 人话原因（失败时必填）。
    pub detail: String,
    /// 额外信息（模型名、chat id…）。
    pub info: String,
}

/// 一个 IM 渠道。
///
/// **同步 trait**（与 `EngineAdapter` 同律）：本 crate 刻意无 async 运行时
/// （`Cargo.toml` 里 tokio 声明了但没用），长轮询靠
/// `nt_channel::poll_once` 的阻塞 + 上层节拍驱动。
pub trait ChannelAdapter {
    /// 稳定渠道 id（与 `channels.id` 对应）。
    fn channel_id(&self) -> &str;

    /// 人读名。
    fn display_name(&self) -> &str;

    /// token 存哪个环境变量名（**不是值**）。
    fn token_env(&self) -> &str;

    /// 探活（能否连上 / 凭据是否有效）。
    fn probe(&self) -> ChannelHealth;

    /// 收一批消息（长轮询一次；无新消息回空 vec）。
    ///
    /// 实现方**自己**处理 offset 持久化；本 trait 不管。
    fn poll(&mut self) -> Result<Vec<InboundMessage>, NtBotError>;

    /// 发一条。
    fn send(&self, out: &OutboundMessage) -> Result<String, NtBotError>;

    /// 能否发附件（不能时 dispatch 降级为纯文字并如实回报）。
    fn supports_attachments(&self) -> bool {
        false
    }

    /// 把平台内的附件取到本机。
    ///
    /// 默认**报错**（不做默认实现）：渠道不支持下载附件时，
    /// 静默返回空会让用户以为文件收到了 —— 那比明确报错坏得多。
    fn fetch_attachment(
        &self,
        _id: &str,
        _dest: &std::path::Path,
    ) -> Result<std::path::PathBuf, NtBotError> {
        Err(NtBotError::Invalid(format!(
            "channel '{}' does not support attachment download",
            self.channel_id()
        )))
    }

    /// 连续失败次数（健康检查用；默认 0 = 不追踪）。
    ///
    /// 放在 trait 上而不是让 IPC 层去认具体类型 —— 否则每加一个渠道
    /// 就要在 IPC 里加一条 `match`，那正是渠道多起来最先烂掉的地方。
    fn consecutive_failures(&self) -> u32 {
        0
    }
}

/// 渠道注册表。
///
/// 按 id 存 `BTreeMap`（不是 `HashMap`）：诊断时 `list` 的顺序稳定，
/// 出问题时可复现 —— 随机的渠道顺序会让「到底哪条先失败」变成玄学。
#[derive(Default)]
pub struct ChannelRegistry {
    adapters: BTreeMap<String, Box<dyn ChannelAdapter>>,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个渠道（重复 id 拒绝，fail-closed）。
    pub fn register(&mut self, adapter: Box<dyn ChannelAdapter>) -> Result<(), NtBotError> {
        let id = adapter.channel_id().to_owned();
        if id.trim().is_empty() {
            return Err(NtBotError::Invalid("channel needs an id".to_owned()));
        }
        if self.adapters.contains_key(&id) {
            return Err(NtBotError::Invalid(format!(
                "channel '{id}' already registered"
            )));
        }
        self.adapters.insert(id, adapter);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&dyn ChannelAdapter> {
        self.adapters.get(id).map(|boxed| boxed.as_ref())
    }

    /// 可变取（长轮询要改 offset）。
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Box<dyn ChannelAdapter>> {
        self.adapters.get_mut(id)
    }

    /// 已注册的渠道 id（排序）。
    pub fn ids(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

/// 访问判定：这条入站消息该不该被处理。
///
/// 三步、顺序不可换：
/// 1. 机器人没启 → 拒；
/// 2. 访问模式（`dm_only` 时群里一律拒；`allow` 时查名单）→ 拒；
/// 3. `open` 放行。
///
/// **为什么 `Allow` 的名单为空就是全拒**：`allow` 的语义是「只放行名单内」，
/// 空名单 = 没人在名单里 = 谁都不该进。写成「空名单即全放」的话，用户点了
/// 「白名单模式」却发现机器人对所有人开放 —— 那是最坏的一种错。
pub fn admits(
    access: AccessMode,
    allow_list: &[String],
    is_dm: bool,
    sender: &str,
) -> bool {
    match access {
        AccessMode::Open => true,
        AccessMode::DmOnly => is_dm,
        AccessMode::Allow => allow_list.iter().any(|entry| entry.trim() == sender.trim()),
    }
}

/// 判定理由（人话；给审计与 UI 显示用）。
pub fn admit_reason(access: AccessMode, allow_list: &[String], is_dm: bool, sender: &str) -> String {
    if admits(access, allow_list, is_dm, sender) {
        return "放行".to_owned();
    }
    match access {
        AccessMode::Open => "不该出现（open 模式恒放行）".to_owned(),
        AccessMode::DmOnly => format!("私聊模式，群消息不理（sender={sender}）"),
        AccessMode::Allow => {
            if allow_list.is_empty() {
                "白名单模式但名单为空 = 谁都不放行".to_owned()
            } else {
                format!("不在白名单里（sender={sender}）")
            }
        }
    }
}

/// 确定性去重键（渠道 + **聊天** + 消息 id）。
///
/// 去重是**必需**的：Telegram 长轮询在网络抖动时会重发同一条 update，
/// 不去重就会让用户看到两条一样的回复。
///
/// **必须带 chat**：平台的 `message_id` 是**按聊天各自编号**的
/// （每个 chat 都从 1 开始）。早先的键只有 `channel:message_id`，
/// 于是 A 群的 5 号与 B 群的 5 号撞成同一个键，**后一条被当成重复静默丢掉** ——
/// 多机器人/多群正是这套设计要支持���场景，丢的却是真消息。
pub fn dedup_key(channel: &str, chat: &str, message_id: &str) -> String {
    format!("{channel}:{chat}:{message_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_mode_parse_is_fail_closed() {
        assert_eq!(AccessMode::parse("open"), AccessMode::Open);
        assert_eq!(AccessMode::parse("dm_only"), AccessMode::DmOnly);
        assert_eq!(AccessMode::parse("dm"), AccessMode::DmOnly);
        assert_eq!(AccessMode::parse("allow"), AccessMode::Allow);
        // 认不出的 → Allow（空名单 = 全拒），**不是** Open。
        for bad in ["", "  ", "OPEN", "yes", "public", "全部"] {
            assert_eq!(
                AccessMode::parse(bad),
                AccessMode::Allow,
                "未知 {bad:?} 应回退到全拒，不能放行",
            );
        }
    }

    #[test]
    fn open_admits_everything() {
        assert!(admits(AccessMode::Open, &[], true, "anyone"));
        assert!(admits(AccessMode::Open, &[], false, "anyone"));
    }

    #[test]
    fn dm_only_rejects_groups_even_for_listed_senders() {
        let list = vec!["alice".to_owned()];
        assert!(admits(AccessMode::DmOnly, &list, true, "alice"));
        assert!(!admits(AccessMode::DmOnly, &list, false, "alice"));
    }

    #[test]
    fn allow_with_empty_list_admits_nobody() {
        // 这是最容易写反的一条：白名单模式 + 空名单 ≠ 全放行。
        assert!(!admits(AccessMode::Allow, &[], true, "alice"));
        assert!(!admits(AccessMode::Allow, &[], false, "alice"));
        let reason = admit_reason(AccessMode::Allow, &[], true, "alice");
        assert!(reason.contains("名单为空"), "{reason}");
    }

    #[test]
    fn allow_admits_only_listed_senders() {
        let list = vec!["alice".to_owned(), " bob ".to_owned()];
        assert!(admits(AccessMode::Allow, &list, true, "alice"));
        assert!(admits(AccessMode::Allow, &list, true, "bob"), "名单项两侧空格应忽略");
        assert!(!admits(AccessMode::Allow, &list, true, "carol"));
        assert!(!admits(AccessMode::Allow, &list, true, "Alice"), "应区分大小写");
    }

    #[test]
    fn dedup_key_is_scoped_by_channel_and_chat() {
        // 同一条消息 id 在两个渠道上不串。
        assert_ne!(dedup_key("tg", "c1", "42"), dedup_key("dc", "c1", "42"));
        // **回归锁**：同一渠道、同一个 message_id，但在**不同聊天**里
        // 必须是两条不同的键 —— 否则两个群各自��� 5 号消息会互相顶掉。
        assert_ne!(
            dedup_key("telegram", "chatA", "5"),
            dedup_key("telegram", "chatB", "5"),
            "同渠道同 message_id 不同 chat 必须不撞（平台按 chat 各自编号）"
        );
        // 完全相同才撞。
        assert_eq!(dedup_key("telegram", "c1", "42"), dedup_key("telegram", "c1", "42"));
    }

    // ---- 注册表 ----

    struct Fake {
        id: &'static str,
    }

    impl ChannelAdapter for Fake {
        fn channel_id(&self) -> &str {
            self.id
        }
        fn display_name(&self) -> &str {
            "假渠道"
        }
        fn token_env(&self) -> &str {
            "NEOBOT_FAKE_TOKEN"
        }
        fn probe(&self) -> ChannelHealth {
            ChannelHealth {
                ok: true,
                detail: String::new(),
                info: String::new(),
            }
        }
        fn poll(&mut self) -> Result<Vec<InboundMessage>, NtBotError> {
            Ok(Vec::new())
        }
        fn send(&self, _out: &OutboundMessage) -> Result<String, NtBotError> {
            Ok("fake-1".to_owned())
        }
    }

    #[test]
    fn registry_rejects_duplicate_and_empty_ids() {
        let mut reg = ChannelRegistry::new();
        reg.register(Box::new(Fake { id: "telegram" })).expect("first");
        let dup = reg.register(Box::new(Fake { id: "telegram" }));
        assert!(dup.is_err(), "重复 id 必须拒");
        let nameless = reg.register(Box::new(Fake { id: "  " }));
        assert!(nameless.is_err(), "空 id 必须拒");
        assert_eq!(reg.len(), 1, "失败的注册不能污染注册表");
    }

    #[test]
    fn registry_lists_in_stable_sorted_order() {
        let mut reg = ChannelRegistry::new();
        for id in ["slack", "telegram", "discord"] {
            reg.register(Box::new(Fake { id })).expect("register");
        }
        // 排序而非 HashMap 的随机序：诊断要可复现。
        assert_eq!(reg.ids(), vec!["discord", "slack", "telegram"]);
        assert!(reg.get("telegram").is_some());
        assert!(reg.get("nope").is_none());
    }

    #[test]
    fn default_registry_is_empty() {
        let reg = ChannelRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }
}

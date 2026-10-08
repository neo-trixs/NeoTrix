//! `nt_channel_telegram` — 第一个真渠道：Telegram Bot API 长轮询。
//!
//! **零依赖**：只用现成的 `ureq`（`nt_web` 已在用）。Telegram 的 Bot API 是
//! 纯 HTTPS + JSON，没有 SDK 必要；加一个 SDK 只会多一份要跟着平台升级的
//! 依赖面。它同时是**别的长轮询渠道的范本**：出站轮询、游标在内存、
//! 重启靠去重表兜底。
//!
//! 游标律：`offset` 只在适配器内存里，**不落库**。重启后会重新拉到平台保留的
//! 那一段（最多 24h）—— 但 `channel_seen` 的去重表会让重复的那批**不被二次处理**，
//! 所以「游标不落库」换来的是零状态迁移，代价只是一次多余的拉取。
//! 这也是为什么去重表是必需件而不是优化件。
//!
//! 凭据律：token 从 `token_env` 指的环境变量现读，**永不进构造函数参数的类型
//! 之外**、永不进日志、永不进 `channel_bots`（那里只有变量名）。
//!
//! 群命令律：Telegram 群里命令要写成 `/new@my_bot 标题`。故解析前先把
//! `@bot_username` 后缀切掉，否则 `/new` 会被当普通消息丢给模型。
//!
//! 出站附件律：**发不出去就整条失败，绝不只发文字**。这是本模块最硬的
//! 一条不变式 —— 用户看到「已发送」就会以为文件也到了，于是去找一个
//! 根本没发出去的东西。早先 `send` 对带附件的出站直接硬报错正是为了守住它；
//! 现在能真发了，**同一个保证换成新的形态**：预检（存在 / 是文件 / 不超限）
//! 全部通过才发第一个字节，任一 part 失败就整条返回 `Err`。
//!
//! multipart 律：boundary 必须**逐个候选验过**「不出现在待编码的字节里」。
//! 随机串撞上的概率极低，但撞上之后的症状是服务端把文件内容当成新 part，
//! 用户收到一个被莫名切碎的文件，而**我们看不出**出了什么事。
//!
//! 编辑律（`edit_of` 接通后的形态，2026-09-28）：`OutboundMessage::edit_of`
//! 有值时**先**走 `editMessageText` 改那条消息，而不是另发一条 —— 用户要的
//! 是「那条消息变成结果」，不是聊天里多一条。
//!
//! 编辑**只能是尽力**：Telegram 只让 bot 编辑**自己发过**的消息
//! （`chat_id` + `message_id` 缺一不可），且与发消息共用配额
//! （官方 FAQ：单会话约 1 条/秒、群 20 条/分钟，撞了回 `429` + `retry_after`）。
//! 故编辑不成时**退回发新消息**（原消息多半是别人发的 / 不存在 / 限流 /
//! 带附件 / 正文超 4096），并**把降级原因记下来**（[`SendOutcome`] +
//! `eprintln!`）—— 静默 fallback 就是换个姿势说谎。
//!
//! **一个例外必须记住**：「内容一模一样」不是降级理由。平台对它回
//! `message is not modified`，此时**目标已经达成**，再发一条就是本模块
//! 要消灭的那种重复。故它记成 [`SendOutcome::already_there`]，**不发新消息**。

use std::io::Read as _;
use std::time::Duration;

use crate::nt_channel::{
    ChannelAdapter, ChannelHealth, InboundMessage, OutboundMessage,
};
use crate::nt_error::NtBotError;

/// Bot API 根。
pub const API_BASE: &str = "https://api.telegram.org";
/// 缺省 token 环境变量名。
pub const DEFAULT_TOKEN_ENV: &str = "NEOBOT_TELEGRAM_TOKEN";
/// 附件大小上限（50 MB）—— 收发**同一个**数。
///
/// 超过即拒，不写半个文件进磁盘、也不截半个文件发出去：截断的附件是
/// 「看着收到了、其实内容不全」，那比明确报错坏得多。
pub const MAX_ATTACHMENT_BYTES: i64 = 50 * 1024 * 1024;
/// 一条消息最多带几个附件（Telegram media group 的硬上限 10）。
///
/// 超了直接拒，两个理由：平台本来就会拒；且 10 × 50 MB 的请求体会在这台
/// 机器内存里堆到 500 MB —— 本地优先应用为了发一条消息吃掉半 GB 不划算。
pub const MAX_ATTACHMENTS_PER_MESSAGE: usize = 10;
/// caption 上限（Telegram 对 `sendDocument` 的 caption 硬限 1024 字符）。
/// 溢出的正文走 `sendMessage` 另发一条，**不截断**。
const MAX_CAPTION_CHARS: usize = 1024;
/// 附件下载整体超时（比普通 API 宽裕：文件比 JSON 大得多）。
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
/// 附件上传超时（同下载：50 MB 走公网不是几秒的事）。
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(120);
/// 单次长轮询挂多久（秒）。Telegram 上限 50。
pub const POLL_TIMEOUT_SECS: u64 = 25;
/// HTTP 超时 = 轮询挂时 + 余量（否则客户端会比服务端先断）。
const HTTP_TIMEOUT: Duration = Duration::from_secs(POLL_TIMEOUT_SECS + 10);
/// 单条出站正文上限（Telegram 硬限制 4096 字符）。超了要**切分并如实回报**。
pub const MAX_TEXT_CHARS: usize = 4096;
/// 一次出站的**记账**（`send()` 跑完后的诚实汇报）。
///
/// **为什么需要它**：trait 的 `send()` 只能回一个 `String`（消息 id），
/// 而「编辑成功了」和「编辑没成、改发新消息了」在返回值上长得**一样**
/// —— 调用方若只看 id，就会以为「编辑了」，实际用户那边多了一条。
/// 不改 trait 签名的前提下，这里就是那条不撒谎的通道：
/// [`TelegramChannel::last_send_outcome`] 读它。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendOutcome {
    /// `editMessageText` 真成功（消息内容确实被替换了）。
    pub edited: bool,
    /// 请求编辑的目标（`None` = 本来就是发新消息，没有编辑意图）。
    pub edit_of: Option<String>,
    /// 降级发新消息的原因（`Some` = 发生过降级；`None` = 没降级）。
    pub degraded: Option<String>,
    /// 编辑没发生但**目标已达成**的原因（内容本就一致，**不发**新消息）。
    pub already_there: Option<String>,
    /// 最终落地的消息 id：编辑 → 被编辑的那条；降级 → 新发的那条。
    pub message_id: String,
}

impl SendOutcome {
    /// 人读一行（诊断与日志用；三个状态互相排斥，不会同时有值）。
    pub fn summary(&self) -> String {
        if let Some(reason) = &self.already_there {
            return format!("原消息已达成（{reason}），未发新消息 message={}", self.message_id);
        }
        if let Some(reason) = &self.degraded {
            return format!(
                "编辑失败改发新消息（{reason}）message={} edit_of={}",
                self.message_id,
                self.edit_of.as_deref().unwrap_or("?")
            );
        }
        if self.edited {
            return format!(
                "已编辑原消息 message={} edit_of={}",
                self.message_id,
                self.edit_of.as_deref().unwrap_or("?")
            );
        }
        format!("已发新消息 message={}", self.message_id)
    }
}

/// 一次 `editMessageText` 尝试的结果（**四态**，不是 bool）。
///
/// 之所以拆这么细：编辑这条路的失败形态差别很大，处置也完全不同 ——
/// 有的是「换个方式发就行」，有的是「**已经**到了，别再发」，还有的是
/// 「网络/token 就坏了，立刻再发一次只是把同一个错误再犯一遍」。
enum EditAttempt {
    /// 编辑成功（消息 id = 被编辑的那条）。
    Edited(String),
    /// 目标已达成（内容本就一致）—— 绝不能再发一条。
    AlreadyThere(String),
    /// 编辑不成，但发新消息仍然有意义（带上人话原因）。
    Refused(String),
    /// 传输层/凭据级失败：改发新消息只会犯同一个错，直接上抛。
    Fatal(NtBotError),
}

/// Telegram 适配器。
pub struct TelegramChannel {
    /// token 的环境变量名。
    token_env: String,
    /// API 根（生产是 `https://api.telegram.org`；**测试可注入本地假服务器**）。
    ///
    /// 早先是编译期 `const API_BASE`，于是**整条 HTTP 链路一行都测不到** ——
    /// 23 个测试全是喂手写 JSON 走 `decode()`，连参数编码、错误分支、
    /// 附件下载都没跑过。改成字段后就能起一个 `TcpListener` 假服务器
    /// （范本见 `nt_http_engine` 的测试），把真实请求打过去验。
    api_base: String,
    /// 机器人自己的 `@username`（群里剥 `@bot` 后缀用；`None` = 还没探活过）。
    bot_username: Option<String>,
    /// 下一批要拉的 update_id。
    offset: i64,
    /// 连续轮询失败次数（IPC 健康检查读它；连续 >= 3 就该提示用户查网络/token）。
    failures: u32,
    /// 上一次 `send()` 的记账（编辑成功 / 降级 / 已达成）。
    ///
    /// 用内部可变性而不是让 `send` 改 `&mut self`：`ChannelAdapter::send`
    /// 的签名是 `&self`，改它就得动 trait（那要所有渠道一起改）。
    last_outcome: std::cell::RefCell<Option<SendOutcome>>,
}

impl TelegramChannel {
    /// 用给定环境变量名建（不读 token —— 读在 `token()` 里）。
    pub fn new(token_env: &str) -> Self {
        let env = if token_env.trim().is_empty() {
            DEFAULT_TOKEN_ENV
        } else {
            token_env.trim()
        };
        Self {
            token_env: env.to_owned(),
            api_base: API_BASE.to_owned(),
            bot_username: None,
            offset: 0,
            failures: 0,
            last_outcome: std::cell::RefCell::new(None),
        }
    }

    /// 换 API 根（**只给测试用**：对着本地 `TcpListener` 假服务器跑）。
    ///
    /// 公开它是有意的：藏在 `#[cfg(test)]` 里的话，集成测试（crate 外的
    /// `tests/`）就够不着，而那才是真正需要打真实 HTTP 的地方。
    pub fn with_base(mut self, base: &str) -> Self {
        let trimmed = base.trim().trim_end_matches('/');
        self.api_base = if trimmed.is_empty() {
            API_BASE.to_owned()
        } else {
            trimmed.to_owned()
        };
        self
    }

    /// 当前 API 根（测试断言用）。
    pub fn api_base(&self) -> &str {
        &self.api_base
    }

    /// 连续失败次数（供健康检查判断「是不是网络/凭据出问题了」）。
    pub fn consecutive_failures(&self) -> u32 {
        self.failures
    }

    /// 上一次 `send()` 的记账（`None` = 还没发过）。
    ///
    /// **「编辑了」与「改发新消息了」在返回值上分不开**，所以分不开这件事
    /// 必须由这里说出来：调用方拿不准时读它，别拿消息 id 反推。
    pub fn last_send_outcome(&self) -> Option<SendOutcome> {
        self.last_outcome.borrow().clone()
    }

    /// 记一次出站账。
    fn remember(&self, outcome: SendOutcome) {
        self.last_outcome.replace(Some(outcome));
    }

    /// token 现读（**每次**都读：支持中途换 token 而不必重启）。
    fn token(&self) -> Result<String, NtBotError> {
        std::env::var(&self.token_env)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                NtBotError::Invalid(format!(
                    "环境变量 {} 未设置或为空（Telegram token 不落盘，只认环境变量）",
                    self.token_env
                ))
            })
    }

    /// 调一次 Bot API，返回 `result` 字段。
    fn call(&self, method: &str, params: &[(&str, &str)]) -> Result<serde_json::Value, NtBotError> {
        let token = self.token()?;
        let mut url = format!("{}/bot{token}/{method}", self.api_base);
        let mut first = true;
        for (key, value) in params {
            url.push(if first { '?' } else { '&' });
            url.push_str(key);
            url.push('=');
            url.push_str(&percent_encode(value));
            first = false;
        }
        let agent = ureq::AgentBuilder::new()
            .timeout(HTTP_TIMEOUT)
            .build();
        let body = agent
            .get(&url)
            .call()
            .map_err(|err| NtBotError::Io(format!("{method}: {err}")))?
            .into_string()
            .map_err(|err| NtBotError::Io(format!("{method} body: {err}")))?;
        decode_result(method, &body)
    }

    /// 发一次 `multipart/form-data`（`sendDocument` 这类必须带 body 的方法）。
    ///
    /// **不能**复用 `call`：那条是 GET + query 参数，附件得走 POST body。
    /// `ok` 字段的判定抽在 [`decode_result`] 里，就是为了让这条路径**不可能**
    /// 忘了查 —— Bot API 永远回 200，忘了查就等于把失败当成功。
    fn call_multipart(
        &self,
        method: &str,
        boundary: &[u8],
        body: &[u8],
    ) -> Result<serde_json::Value, NtBotError> {
        let token = self.token()?;
        let url = format!("{}/bot{token}/{method}", self.api_base);
        // boundary 是我们自己挑的纯 ASCII，utf8 换算是无损的；
        // 真的不可能（`pick_boundary` 只产出 ASCII）时也不该 panic。
        let content_type = format!(
            "multipart/form-data; boundary={}",
            String::from_utf8_lossy(boundary)
        );
        let agent = ureq::AgentBuilder::new()
            .timeout(UPLOAD_TIMEOUT)
            .build();
        let text = agent
            .post(&url)
            .set("Content-Type", &content_type)
            .send_bytes(body)
            .map_err(|err| NtBotError::Io(format!("{method}: {err}")))?
            .into_string()
            .map_err(|err| NtBotError::Io(format!("{method} body: {err}")))?;
        decode_result(method, &text)
    }

    /// 拉一批 update 并推进游标。
    fn fetch(&mut self) -> Result<Vec<serde_json::Value>, NtBotError> {
        let offset = self.offset.to_string();
        let timeout = POLL_TIMEOUT_SECS.to_string();
        let result = self.call("getUpdates", &[("offset", &offset), ("timeout", &timeout)])?;
        let Some(items) = result.as_array() else {
            return Ok(Vec::new());
        };
        let mut updates: Vec<serde_json::Value> = items.to_vec();
        // 平台不保证顺序，按 update_id 排一遍；游标取最大+1。
        updates.sort_by_key(|item| {
            item.get("update_id")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
        });
        if let Some(last) = updates.last() {
            if let Some(id) = last.get("update_id").and_then(|v| v.as_i64()) {
                self.offset = id + 1;
            }
        }
        Ok(updates)
    }

    /// 剥掉群命令里的 `@bot_username` 后缀。
    ///
    /// Telegram 群里命令必须带 `@`（否则所有机器人都收）；私聊里通常不带。
    fn strip_bot_suffix(&self, text: &str) -> String {
        let Some(username) = self.bot_username.as_deref() else {
            // 还没探活过就不动它（宁可不去，也不误删正文里的 @某人）。
            return text.to_owned();
        };
        let needle = format!("@{username}");
        match text.find(&needle) {
            // 只在前 64 字符内替换（命令名很短，后面的正文不该被动）。
            Some(idx) if idx <= 64 => {
                let mut out = String::with_capacity(text.len());
                out.push_str(text.get(..idx).unwrap_or(""));
                out.push_str(text.get(idx + needle.len()..).unwrap_or(""));
                out
            }
            _ => text.to_owned(),
        }
    }

    /// 一条 update → 一条入站消息（非消息类 update 返回 `None`）。
    pub fn decode(&self, update: &serde_json::Value) -> Option<InboundMessage> {
        let message = update.get("message")?;
        let chat = message.get("chat")?;
        let chat_id = chat.get("id")?.as_i64()?.to_string();
        let chat_type = chat.get("type").and_then(|v| v.as_str()).unwrap_or("private");
        let is_dm = chat_type == "private";
        let message_id = message
            .get("message_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .to_string();
        // 正文：文本用 `text`，图片/文件的说明文字在 `caption`。
        let raw = message
            .get("text")
            .or_else(|| message.get("caption"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let text = self.strip_bot_suffix(raw).trim().to_owned();
        // 非文本且无说明 → 暂不支持附件下载，**如实**给空正文而不是编一句。
        let from = message.get("from");
        let sender = from
            .and_then(|f| {
                f.get("id")
                    .and_then(|v| v.as_i64())
                    .map(|id| id.to_string())
                    .or_else(|| f.get("username").and_then(|v| v.as_str()).map(str::to_owned))
            })
            .unwrap_or_default();
        let sender_name = from
            .and_then(|f| {
                f.get("first_name")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
            })
            .or_else(|| {
                from.and_then(|f| f.get("username"))
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| sender.clone());
        Some(InboundMessage {
            chat: chat_id,
            is_dm,
            sender,
            sender_name,
            text,
            message_id,
            attachments: decode_attachments(message),
            reply_to: message
                .get("reply_to_message")
                .and_then(|m| m.get("message_id"))
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string()),
        })
    }

    /// 纯文字出站（按 4096 切分，逐条发）。
    fn send_text(&self, chat: &str, text: &str) -> Result<String, NtBotError> {
        let mut last_id = String::new();
        for piece in split_text(text, MAX_TEXT_CHARS) {
            let result = self.call(
                "sendMessage",
                &[
                    ("chat_id", chat),
                    ("text", piece.as_str()),
                    ("disable_web_page_preview", "true"),
                ],
            )?;
            if let Some(id) = result_message_id(&result) {
                last_id = id;
            }
        }
        Ok(last_id)
    }

    /// 带附件的出站：一次 `sendDocument`（multipart）把所有文件发出去。
    ///
    /// **为什么不是「每个文件一次请求」**：分两次发就没有原子性了 —— 第二个
    /// 文件失败时第一个已经躺在用户聊天里了，我们只能报一个「失败」而用户
    /// 明明收到了东西。**一次请求、一次成败**才兑现本模块的出站不变式：
    /// 要么文件全到，要么整条返回 `Err` 让上层重试。
    fn send_with_attachments(
        &self,
        chat: &str,
        text: &str,
        attachments: &[String],
    ) -> Result<String, NtBotError> {
        if attachments.len() > MAX_ATTACHMENTS_PER_MESSAGE {
            return Err(NtBotError::Invalid(format!(
                "附件 {} 个，超过单条 {} 个上限（Telegram media group 限制），不发送",
                attachments.len(),
                MAX_ATTACHMENTS_PER_MESSAGE
            )));
        }
        // 预检全过再发第一个字节：这一轮里最可能的失败（文件没了 / 超限）
        // 必须在**任何**请求之前就暴露，否则用户会收到半条消息。
        let mut files = Vec::with_capacity(attachments.len());
        for path in attachments {
            files.push((safe_filename(path), read_attachment(path)?));
        }
        // caption 上限 1024（Telegram 硬限），溢出的部分**另发一条**而不是截掉。
        let (caption, leftover) = split_caption(text);
        // boundary 要对**所有**待编码字节验一遍：字段值撞上会同样破坏分帧。
        let blobs: Vec<&[u8]> = files
            .iter()
            .map(|(_, bytes)| bytes.as_slice())
            .chain([caption.as_bytes(), chat.as_bytes()])
            .collect();
        let boundary = pick_boundary(&blobs)?;

        let mut parts: Vec<Part<'_>> = vec![
            Part { name: "chat_id", filename: None, value: chat.as_bytes() },
        ];
        if !caption.is_empty() {
            parts.push(Part { name: "caption", filename: None, value: caption.as_bytes() });
        }
        for (name, bytes) in &files {
            parts.push(Part {
                name: "document",
                filename: Some(name.as_str()),
                value: bytes.as_slice(),
            });
        }
        let body = build_multipart(&boundary, &parts);
        let result = self.call_multipart("sendDocument", &boundary, &body)?;
        let mut last_id = result_message_id(&result).unwrap_or_default();
        // caption 装不下的正文补一条（附件已经真发出去了，不补才是丢内容）。
        if !leftover.trim().is_empty() {
            let tail = self.send_text(chat, &leftover)?;
            if !tail.is_empty() {
                last_id = tail;
            }
        }
        Ok(last_id)
    }

    /// 发一条**新**消息（文字或附件）—— 「不改原消息」的那条路。
    fn send_new(&self, chat: &str, text: &str, attachments: &[String]) -> Result<String, NtBotError> {
        if attachments.is_empty() {
            return self.send_text(chat, text);
        }
        self.send_with_attachments(chat, text, attachments)
    }

    /// 试着把 `message_id` 那条消息的正文换成 `text`（`editMessageText`）。
    ///
    /// **先判能不能编成一个合法请求，再打网络**：平台侧的失败（400/429）我们
    /// 只能事后降级，而参数形状不对（带附件、正文超限、id 不是整数）本地就知道，
    /// 白跑一趟只会把降级原因写得更难读。
    fn try_edit_text(
        &self,
        chat: &str,
        message_id: &str,
        text: &str,
        attachments: &[String],
    ) -> EditAttempt {
        if !attachments.is_empty() {
            // editMessageText 只能换**正文**；换媒体是 editMessageMedia。
            // 硬发一次的后果是「正文改了、附件还是旧的」—— 用户看到一份
            // 配错对的图，比两条消息更难排查。
            return EditAttempt::Refused(format!(
                "带 {} 个附件，editMessageText 换不了媒体（那是 editMessageMedia）",
                attachments.len()
            ));
        }
        if text.trim().is_empty() {
            return EditAttempt::Refused("正文为空，editMessageText 至少要 1 个字符".to_owned());
        }
        let chars = text.chars().count();
        if chars > MAX_TEXT_CHARS {
            // editMessageText 的 text 上限也是 4096，超了平台直接拒。
            // 走发新消息那条路反而能切分成多条。
            return EditAttempt::Refused(format!(
                "正文 {chars} 字符 > editMessageText 上限 {MAX_TEXT_CHARS}"
            ));
        }
        let Ok(numeric_id) = message_id.trim().parse::<i64>() else {
            return EditAttempt::Refused(format!(
                "edit_of={message_id:?} 不是整数（Telegram 要 message_id 是 Integer）"
            ));
        };
        let id = numeric_id.to_string();
        match self.call(
            "editMessageText",
            &[
                ("chat_id", chat),
                ("message_id", id.as_str()),
                ("text", text),
                ("disable_web_page_preview", "true"),
            ],
        ) {
            // 成功时 result 是被编辑后的 Message；理论上内联消息会回 true，
            // 那种情况下我们只有请求里的 id —— 用它，不凭空编一个。
            Ok(result) => EditAttempt::Edited(result_message_id(&result).unwrap_or(id)),
            // 「内容一模一样」不是失败：目标已经达成，再发一条就是重复。
            Err(NtBotError::Engine { reason, .. }) if is_not_modified(&reason) => {
                EditAttempt::AlreadyThere(id)
            }
            // 平台拒了（消息不存在 / 不是本 bot 发的 / 限流 / 会话不对…）：
            // 发新消息仍然有意义，带上平台给的人话原因。
            Err(NtBotError::Engine { reason, .. }) => EditAttempt::Refused(reason),
            // 传输层与凭据：再发一条新消息只会撞同一个错，直接上抛让上层重试。
            Err(other) => EditAttempt::Fatal(other),
        }
    }
}

/// 平台的 `description` 是不是「内容无变化」这条。
///
/// 依赖平台措辞（`Bad Request: message is not modified`，tdlib / grammers /
/// aiogram 都按这串字判），所以**只在这一条上**依赖它，认不出来就退成降级
/// —— 多发一条比谎称「编辑成功」轻，但两种都要记账（见 [`SendOutcome`]）。
fn is_not_modified(reason: &str) -> bool {
    reason.to_ascii_lowercase().contains("message is not modified")
}

/// file_id → 稳定的短哈希（临时文件名用；不能直接用 file_id，它可能很长且含符号）。
fn id_hash(id: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Bot API 的应答 → `result`（**唯一的成功判定处**）。
///
/// 规则就一条：Bot API 永远回 HTTP 200，失败写在 body 的 `ok:false` 里。
/// 所以不查 `ok` 就等于把所有失败当成功 —— 用户看到「已发送」而什么都没发出去。
/// GET / POST 两条路径共用它，就是为了让新加的那条也不可能绕过这条规则。
fn decode_result(method: &str, body: &str) -> Result<serde_json::Value, NtBotError> {
    let parsed: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| NtBotError::Codec(format!("{method}: {err}")))?;
    if parsed.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        let desc = parsed
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Err(NtBotError::Engine {
            engine: "telegram".to_owned(),
            reason: format!("{method}: {desc}"),
        });
    }
    Ok(parsed.get("result").cloned().unwrap_or(serde_json::Value::Null))
}

/// 从 `sendDocument` 的 `result` 里取消息 id。
///
/// 单文件时 `result` 是**对象**，多文件（Telegram 按 media group 收）是**数组**
/// —— 两种形状都得认，否则带附件的发送永远拿不回 id（上层会以为没发出去）。
fn result_message_id(result: &serde_json::Value) -> Option<String> {
    let one = result
        .as_array()
        .and_then(|items| items.last())
        .unwrap_or(result);
    one.get("message_id")
        .and_then(|v| v.as_i64())
        .map(|id| id.to_string())
}

/// 子串查找（不引第三方 crate；needle 总是 boundary 那种几十字节的短串）。
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    let last = haystack.len() - needle.len();
    (0..=last).any(|i| haystack.get(i..i + needle.len()) == Some(needle))
}

/// 挑一个**与所有待编码字节都不撞**的 boundary。
///
/// 为什么不「随机串 + 撞上算了」：RFC 2046 要求 boundary 不出现在被包裹的内容里。
/// 撞上之后的服务端行为是把文件内容当成新 part 开头，用户收到一个被莫名切碎的
/// 文件，而**我们这边看不出**出了什么事（HTTP 200、`ok:true`）。概率低不等于
/// 不会，而这条路的失败模式是最难查的那种 —— 所以逐个候选**验**一遍。
fn pick_boundary(blobs: &[&[u8]]) -> Result<Vec<u8>, NtBotError> {
    for attempt in 0..64u32 {
        let candidate = format!(
            "----NeoTrixBoundary{}-{attempt}",
            uuid::Uuid::new_v4().simple()
        );
        let needle = candidate.as_bytes();
        if !blobs.iter().any(|blob| contains_bytes(blob, needle)) {
            return Ok(needle.to_vec());
        }
    }
    // 64 个候选全撞 = 有人在附件里塞了针对我们的内容。报错，不硬发。
    Err(NtBotError::Invalid(
        "无法为 multipart 找到与附件内容不冲突的 boundary（拒绝发送）".to_owned(),
    ))
}

/// 一个待编码的 multipart part。
struct Part<'a> {
    name: &'a str,
    /// 文件 part 的文件名（普通字段为 `None`）。
    filename: Option<&'a str>,
    value: &'a [u8],
}

/// 拼一个 `multipart/form-data` 请求体（RFC 7578 形状）。
///
/// 收尾是 `--boundary--\r\n`；每个 part 以 `\r\n` 结尾。故「内容里出现
/// `\r\n--boundary`」就会被误认成新 part —— 这正是 [`pick_boundary`] 要排除的。
fn build_multipart(boundary: &[u8], parts: &[Part<'_>]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for part in parts {
        out.extend_from_slice(b"--");
        out.extend_from_slice(boundary);
        out.extend_from_slice(b"\r\nContent-Disposition: form-data; name=\"");
        out.extend_from_slice(part.name.as_bytes());
        if let Some(filename) = part.filename {
            out.extend_from_slice(b"\"; filename=\"");
            out.extend_from_slice(filename.as_bytes());
        }
        out.extend_from_slice(b"\"\r\n");
        // 文件 part 显式声明类型：Telegram 不看它（自己嗅探），
        // 但不给的话某些服务端会当文本处理，破坏二进制。
        if part.filename.is_some() {
            out.extend_from_slice(b"Content-Type: application/octet-stream\r\n");
        }
        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(part.value);
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(b"--");
    out.extend_from_slice(boundary);
    out.extend_from_slice(b"--\r\n");
    out
}

/// 把本地路径压成一个**不可能改变 multipart 语义**的文件名。
///
/// 规则：**只丢「能逃出去」的字符，其余全留**。
/// 丢的是 `"`（闭合 `filename="…"`）、`\`（quoted-string 里的转义符）、
/// `;`（`Content-Disposition` 的参数分隔符）、`/`（压根不是文件名），
/// 以及**全部控制字符**（CR/LF 能直接劈开 header，空格以外的都别留）。
///
/// **为什么不搞白名单**：早先的版本只放行 `-_. ()` 与字母数字，结果把真实
/// 文件名里的 `%` `+` `#` 也吃了 —— 用户在 Telegram 里看到的是
/// `a0D0A22chat_id22.txt` 而不是 `a%0D%0A%22chat_id%22.txt`。
/// 为堵一条注入而毁掉所有正常名字，是拿可用性换安全，不划算。
/// 危险集合就那么几个字面量，按「排除它们」写既更安全也更保真。
///
/// 注意 `%` **不是**危险字符：percent 编码只存在于 URL 查询串里，
/// multipart 的文件名是原样取用的，服务端不会去解码它。
/// 故 `v1.2+build.pdf`、`report%20final.pdf` 这类名字能原样送到用户眼前。
pub fn safe_filename(path: &str) -> String {
    let base = std::path::Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let mut out = String::new();
    for ch in base.chars() {
        if out.chars().count() >= 120 {
            break;
        }
        if ch.is_control() || matches!(ch, '"' | '\\' | ';' | '/') {
            continue;
        }
        out.push(ch);
    }
    let trimmed = out.trim();
    // 全是空白或全是点（`.`/`..`）时无可用名，给个通用名而不是发一个空名。
    if trimmed.is_empty() || trimmed.chars().all(|ch| ch == '.') {
        "attachment.bin".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// 读一个待发附件：**先量大小再读**，超限的直接拒、绝不读进内存。
fn read_attachment(path: &str) -> Result<Vec<u8>, NtBotError> {
    let meta = std::fs::metadata(path)
        .map_err(|err| NtBotError::Io(format!("附件读不到 {path}：{err}")))?;
    if !meta.is_file() {
        return Err(NtBotError::Invalid(format!(
            "附件 {path} 不是普通文件，不发"
        )));
    }
    let size = i64::try_from(meta.len()).unwrap_or(i64::MAX);
    if size > MAX_ATTACHMENT_BYTES {
        // 拒而不是截：半个文件比没有文件更坏（用户会以为收到了）。
        return Err(NtBotError::Invalid(format!(
            "附件 {path} 太大（{size} 字节 > {MAX_ATTACHMENT_BYTES} 上限），不发送"
        )));
    }
    std::fs::read(path).map_err(|err| NtBotError::Io(format!("附件读取失败 {path}：{err}")))
}

/// 一条消息里的附件（`document` / `photo` / `audio` / `video` / `voice` / `sticker`）。
///
/// `photo` 是**数组**且按尺寸从小到大排，Telegram 惯例取**最后一个**（原图）。
/// 同一个 message 可能有多个 media_field（少见），全收。
pub fn decode_attachments(message: &serde_json::Value) -> Vec<crate::nt_channel::InboundAttachment> {
    let mut out = Vec::new();
    for field in ["document", "photo", "audio", "video", "voice", "sticker"] {
        let Some(value) = message.get(field) else { continue };
        let items: Vec<&serde_json::Value> = match value.as_array() {
            Some(arr) => arr.iter().collect(),
            None => vec![value],
        };
        // photo 数组：取最大尺寸那一个（原图）。
        let chosen = if field == "photo" {
            items
                .iter()
                .copied()
                .max_by_key(|item| {
                    item.get("file_size")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0)
                })
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            items
        };
        for item in chosen {
            let Some(id) = item.get("file_id").and_then(|v| v.as_str()) else {
                continue;
            };
            // 文件名：document 自带 file_name；其它 media 用 `file_unique_id` 兜底。
            let name = item
                .get("file_name")
                .and_then(|v| v.as_str())
                .or_else(|| item.get("file_unique_id").and_then(|v| v.as_str()))
                .unwrap_or("attachment")
                .to_owned();
            out.push(crate::nt_channel::InboundAttachment {
                id: id.to_owned(),
                name,
                size: item.get("file_size").and_then(|v| v.as_i64()).unwrap_or(0),
            });
        }
    }
    out
}

/// 切分超长正文（Telegram 单条上限 4096 字符）。
///
/// 在**段落 / 换行**处切，切不动才硬切 —— 硬切会把一个词劈成两半。
pub fn split_text(text: &str, limit: usize) -> Vec<String> {
    if text.chars().count() <= limit {
        return if text.trim().is_empty() {
            Vec::new()
        } else {
            vec![text.to_owned()]
        };
    }
    let mut out = Vec::new();
    // `rest` 必须是**owned**：每轮切完要把剩下的重新收集成一份新串，
    // 而 `&str` 无法从 char 迭代器收集回来。
    let mut rest = text.to_owned();
    while rest.chars().count() > limit {
        // 找一个 <= limit 的换行 / 段落边界。
        // 注意 `rfind` 给的是**字节**下标，而 `chars().take()` 吃的是**字符**数，
        // 两者不能混用 —— 故先切出 head，再在 head 上按字符重新定位切点。
        let head: String = rest.chars().take(limit).collect();
        let cut_chars = head
            .rfind('\n')
            .map(|byte_idx| head.get(..byte_idx).map_or(limit, |s| s.chars().count()))
            .filter(|chars| *chars > limit / 3)
            .unwrap_or(limit);
        // 切点要落在分隔符**之后**：分隔符归上一片，下一片从干净内容起头。
        //
        // 早先的写法是 `trim_end()` 把换行吃掉 —— 那样拼回去会**丢字符**
        // （`\n\n` 整个没了），用户收到的就是一段被悄悄截短的文本。
        // 切分不许丢内容，这是硬要求。
        let mut cut = cut_chars;
        let after: String = head.chars().skip(cut).collect();
        let newlines = after.chars().take_while(|ch| *ch == '\n').count();
        cut = (cut + newlines).min(head.chars().count());
        let piece: String = head.chars().take(cut).collect();
        if !piece.trim().is_empty() {
            out.push(piece);
        }
        rest = rest.chars().skip(cut).collect();
    }
    let tail = rest.trim();
    if !tail.is_empty() {
        out.push(tail.to_owned());
    }
    out
}

/// 极简百分号编码（Telegram 只需 `%`、空格与 `&`/`=` 转义）。
fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

impl ChannelAdapter for TelegramChannel {
    fn channel_id(&self) -> &str {
        "telegram"
    }

    fn display_name(&self) -> &str {
        "Telegram"
    }

    fn token_env(&self) -> &str {
        &self.token_env
    }

    /// Telegram 支持 editMessageText ⇒ 占位→流式编辑 走此能力位。
    fn can_edit(&self) -> bool {
        true
    }

    fn probe(&self) -> ChannelHealth {
        match self.call("getMe", &[]) {
            Ok(result) => {
                let username = result
                    .get("username")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned);
                let label = result
                    .get("first_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let info = format!(
                    "{label} (@{})",
                    username.unwrap_or_else(|| "?".to_owned())
                );
                ChannelHealth {
                    ok: true,
                    detail: String::new(),
                    info,
                }
            }
            Err(err) => ChannelHealth {
                ok: false,
                detail: err.to_string(),
                info: String::new(),
            },
        }
    }

    fn poll(&mut self) -> Result<Vec<InboundMessage>, NtBotError> {
        // 首次探活顺手取回 @username（群命令要靠它）。
        if self.bot_username.is_none() {
            let _ = self.probe();
        }
        match self.fetch() {
            Ok(updates) => {
                self.failures = 0;
                Ok(updates.iter().filter_map(|u| self.decode(u)).collect())
            }
            Err(err) => {
                self.failures = self.failures.saturating_add(1);
                Err(err)
            }
        }
    }

    fn consecutive_failures(&self) -> u32 {
        self.failures
    }

    /// Telegram 收发**都**支持附件（`sendDocument` 走 multipart），
    /// 故这一位如实为 `true` —— 它是上层的降级开关，谎报会让上层
    /// 在一个本来能发文件的渠道上把文件悄悄丢掉。
    fn supports_attachments(&self) -> bool {
        true
    }

    fn fetch_attachment(
        &self,
        id: &str,
        dest: &std::path::Path,
    ) -> Result<std::path::PathBuf, NtBotError> {
        use std::io::Write as _;

        // Telegram 的下载是两步：先 `getFile` 换 `file_path`，
        // 再从 `https://api.telegram.org/file/bot<token>/<file_path>` 取字节。
        let file = self.call("getFile", &[("file_id", id.trim())])?;
        let Some(remote) = file.get("file_path").and_then(|v| v.as_str()) else {
            return Err(NtBotError::Invalid(format!(
                "telegram getFile 没给 file_path（file_id={id}）"
            )));
        };
        // 平台报的尺寸先查一遍：超限就别下载了，省得把 500 MB 拖进来再丢。
        if let Some(size) = file.get("file_size").and_then(|v| v.as_i64()) {
            if size > MAX_ATTACHMENT_BYTES {
                return Err(NtBotError::Invalid(format!(
                    "附件 {size} 字节，超过 {MAX_ATTACHMENT_BYTES} 上限，不下载"
                )));
            }
        }
        let token = self.token()?;
        let url = format!("{}/file/bot{token}/{remote}", self.api_base);
        let agent = ureq::AgentBuilder::new()
            .timeout(DOWNLOAD_TIMEOUT)
            .build();
        let response = agent
            .get(&url)
            .call()
            .map_err(|err| NtBotError::Io(format!("download {remote}: {err}")))?;
        // 再兜一层：响应头里的长度也查。
        if let Some(len) = response
            .header("content-length")
            .and_then(|v| v.parse::<i64>().ok())
        {
            if len > MAX_ATTACHMENT_BYTES {
                return Err(NtBotError::Invalid(format!(
                    "附件 {len} 字节，超过 {MAX_ATTACHMENT_BYTES} 上限，不下载"
                )));
            }
        }
        // 落盘名：只用 file_path 的**最后一段**（平台给的相对路径），
        // 并剥掉任何目录分隔符 —— 平台名不可信，不能让它写穿目录。
        let base = remote.rsplit('/').next().unwrap_or("attachment");
        let safe: String = base
            .chars()
            .filter(|ch| !matches!(ch, '/' | '\\' | ':' | '\0' | '.' | '-'))
            .collect();
        let name = if safe.is_empty() {
            format!("{id}.bin")
        } else {
            // 保留扩展名（没有就算 bin）。
            let ext = base.rsplit_once('.').map(|(_, e)| e.to_owned());
            match ext {
                Some(e) if !e.is_empty() && e.len() <= 8 => format!("{safe}.{}", e.to_lowercase()),
                _ => format!("{safe}.bin"),
            }
        };
        std::fs::create_dir_all(dest).map_err(|err| NtBotError::Io(format!("mkdir: {err}")))?;
        let path = dest.join(&name);
        let mut reader = response.into_reader();
        // 先写临时文件再改名：中途失败不会留下一个「看起来下载好了」的半个文件。
        let tmp = dest.join(format!(".{}.part", id_hash(id)));
        {
            let mut file_out = std::fs::File::create(&tmp)
                .map_err(|err| NtBotError::Io(format!("create: {err}")))?;
            let mut written: i64 = 0;
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let n = reader
                    .read(&mut buf)
                    .map_err(|err| NtBotError::Io(format!("read: {err}")))?;
                if n == 0 {
                    break;
                }
                written = written.saturating_add(n as i64);
                if written > MAX_ATTACHMENT_BYTES {
                    // 超限：删掉临时文件再报错，绝不留在磁盘上。
                    let _cleanup: Option<()> = std::fs::remove_file(&tmp).ok();
                    return Err(NtBotError::Invalid(format!(
                        "附件超过 {MAX_ATTACHMENT_BYTES} 上限，已丢弃"
                    )));
                }
                file_out
                    .write_all(buf.get(..n).unwrap_or(&[]))
                    .map_err(|err| NtBotError::Io(format!("write: {err}")))?;
            }
            file_out
                .sync_all()
                .map_err(|err| NtBotError::Io(format!("sync: {err}")))?;
        }
        std::fs::rename(&tmp, &path).map_err(|err| NtBotError::Io(format!("rename: {err}")))?;
        Ok(path)
    }

    /// 发一条。有 `edit_of` 就**改那条**，改不成就**如实记账地**发新消息。
    fn send(&self, out: &OutboundMessage) -> Result<String, NtBotError> {
        if out.text.trim().is_empty() && out.attachments.is_empty() {
            return Err(NtBotError::Invalid(
                "nothing to send (empty text and no attachments)".to_owned(),
            ));
        }
        let chat = out.chat.as_str();
        // 空串 / 纯空白的 `edit_of` 当没有（老 payload 与手写调用都会这么来）。
        let target = out
            .edit_of
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());
        if let Some(message_id) = target {
            match self.try_edit_text(chat, message_id, &out.text, &out.attachments) {
                EditAttempt::Edited(id) => {
                    self.remember(SendOutcome {
                        edited: true,
                        edit_of: Some(message_id.to_owned()),
                        degraded: None,
                        already_there: None,
                        message_id: id.clone(),
                    });
                    return Ok(id);
                }
                EditAttempt::AlreadyThere(id) => {
                    // **不发新消息**：原消息已经是这段话了。
                    self.remember(SendOutcome {
                        edited: false,
                        edit_of: Some(message_id.to_owned()),
                        degraded: None,
                        already_there: Some("平台回 message is not modified（内容本就一致）".to_owned()),
                        message_id: id.clone(),
                    });
                    return Ok(id);
                }
                EditAttempt::Refused(reason) => {
                    // 降级**先记账再发**：半路失败时也已经知道发生过降级。
                    eprintln!("[neobot] 编辑原消息 {message_id} 不成（{reason}）→ 改发新消息（chat={chat}）");
                    self.remember(SendOutcome {
                        edited: false,
                        edit_of: Some(message_id.to_owned()),
                        degraded: Some(reason.clone()),
                        already_there: None,
                        message_id: String::new(),
                    });
                    match self.send_new(chat, &out.text, &out.attachments) {
                        Ok(id) => {
                            // 记回**真正**落地的那条 id（降级后是新发的那条）。
                            self.remember(SendOutcome {
                                edited: false,
                                edit_of: Some(message_id.to_owned()),
                                degraded: Some(reason),
                                already_there: None,
                                message_id: id.clone(),
                            });
                            return Ok(id);
                        }
                        // 两个原因都报：只报后者会让人以为「编辑失败了」，
                        // 而事实上用户**什么都没收到**。
                        Err(err) => {
                            return Err(NtBotError::Engine {
                                engine: "telegram".to_owned(),
                                reason: format!(
                                    "编辑原消息不成（{reason}），改发新消息也失败：{err}"
                                ),
                            });
                        }
                    }
                }
                EditAttempt::Fatal(err) => return Err(err),
            }
        }
        let id = self.send_new(chat, &out.text, &out.attachments)?;
        self.remember(SendOutcome {
            edited: false,
            edit_of: None,
            degraded: None,
            already_there: None,
            message_id: id.clone(),
        });
        Ok(id)
    }
}

/// 正文切成 `(caption, 余下的正文)`。
///
/// caption 装不下的**另发一条**，不截断 —— 截掉的字用户永远看不到，
/// 而「模型说了但用户那边少了半段」正是最难被发现的一类丢失。
fn split_caption(text: &str) -> (String, String) {
    if text.chars().count() <= MAX_CAPTION_CHARS {
        return (text.to_owned(), String::new());
    }
    let head: String = text.chars().take(MAX_CAPTION_CHARS).collect();
    let tail: String = text.chars().skip(MAX_CAPTION_CHARS).collect();
    (head, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(json: &str) -> Option<InboundMessage> {
        let value: serde_json::Value = serde_json::from_str(json).expect("json");
        TelegramChannel::new("NEOBOT_TG_TEST").decode(&value)
    }

    #[test]
    fn decodes_a_private_text_message() {
        let got = decode(
            r#"{"update_id":7,"message":{"message_id":11,"date":1,
                "chat":{"id":-100,"type":"private"},
                "from":{"id":42,"first_name":"Alice","username":"alice"},
                "text":"在吗"}}"#,
        )
        .expect("decoded");
        assert_eq!(got.chat, "-100");
        assert!(got.is_dm);
        assert_eq!(got.sender, "42");
        assert_eq!(got.sender_name, "Alice");
        assert_eq!(got.text, "在吗");
        assert_eq!(got.message_id, "11");
        assert!(got.attachments.is_empty());
    }

    #[test]
    fn decodes_a_group_message_as_not_dm() {
        let got = decode(
            r#"{"update_id":8,"message":{"message_id":12,
                "chat":{"id":-1,"type":"supergroup"},
                "from":{"id":42,"username":"alice"},
                "text":"hi"}}"#,
        )
        .expect("decoded");
        assert_eq!(got.chat, "-1");
        assert!(!got.is_dm, "群不该被当成私聊");
        // 没有 first_name 时回落到 username。
        assert_eq!(got.sender_name, "alice");
    }

    #[test]
    fn uses_caption_when_text_is_absent() {
        let got = decode(
            r#"{"update_id":9,"message":{"message_id":13,
                "chat":{"id":5,"type":"private"},
                "from":{"id":1,"first_name":"B"},
                "caption":"看这张图"}}"#,
        )
        .expect("decoded");
        assert_eq!(got.text, "看这张图");
    }

    #[test]
    fn non_message_updates_are_skipped() {
        assert!(decode(r#"{"update_id":1,"channel_post":{"message_id":1}}"#).is_none());
        assert!(decode(r#"{"update_id":1}"#).is_none());
    }

    #[test]
    fn reply_to_is_captured() {
        let got = decode(
            r#"{"update_id":10,"message":{"message_id":20,
                "chat":{"id":5,"type":"private"},
                "from":{"id":1,"first_name":"B"},
                "reply_to_message":{"message_id":19},
                "text":"接着说"}}"#,
        )
        .expect("decoded");
        assert_eq!(got.reply_to.as_deref(), Some("19"));
    }

    #[test]
    fn bot_suffix_is_stripped_only_when_known() {
        // 没探活过 → 不动正文（宁可不去，也不误删正文里的 @某人）。
        let ch = TelegramChannel::new("NEOBOT_TG_TEST");
        assert_eq!(
            ch.strip_bot_suffix("/new@my_bot 重构"),
            "/new@my_bot 重构",
            "没探活过就不该动"
        );
    }

    #[test]
    fn bot_suffix_stripping_is_scoped() {
        let mut ch = TelegramChannel::new("NEOBOT_TG_TEST");
        ch.bot_username = Some("my_bot".to_owned());
        assert_eq!(ch.strip_bot_suffix("/new@my_bot 重构"), "/new 重构");
        assert_eq!(ch.strip_bot_suffix("/new 重构"), "/new 重构");
        // 正文里的 @ 不受影响。
        assert_eq!(ch.strip_bot_suffix("问 @someone 一下"), "问 @someone 一下");
        // 后缀在很后面 → 不动（命令名不会那么长）。
        let long_prefix = format!("/new {}{}", "x".repeat(200), "@my_bot");
        assert_eq!(ch.strip_bot_suffix(&long_prefix), long_prefix);
    }

    #[test]
    fn decodes_a_document_attachment() {
        let value: serde_json::Value = serde_json::from_str(
            r#"{"message_id":1,"chat":{"id":5,"type":"private"},
                "document":{"file_id":"FILE1","file_name":"report.pdf","file_size":1234}}"#,
        )
        .expect("json");
        let got = decode_attachments(&value);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].id, "FILE1");
        assert_eq!(got[0].name, "report.pdf");
        assert_eq!(got[0].size, 1234);
    }

    #[test]
    fn photo_array_picks_the_largest() {
        // Telegram 的 photo 是从小到大排的多档，惯例取最后一个（原图）。
        let value: serde_json::Value = serde_json::from_str(
            r#"{"photo":[
                {"file_id":"small","file_unique_id":"u0","file_size":100},
                {"file_id":"mid","file_unique_id":"u1","file_size":1000},
                {"file_id":"big","file_unique_id":"u2","file_size":10000}]}"#,
        )
        .expect("json");
        let got = decode_attachments(&value);
        assert_eq!(got.len(), 1, "一档图应只算一个附件");
        assert_eq!(got[0].id, "big");
        // 没有 file_name 时回落到 file_unique_id。
        assert_eq!(got[0].name, "u2");
    }

    #[test]
    fn multiple_media_fields_are_all_collected() {
        let value: serde_json::Value = serde_json::from_str(
            r#"{"document":{"file_id":"D","file_name":"a.txt","file_size":1},
                "audio":{"file_id":"A","file_unique_id":"au","file_size":2}}"#,
        )
        .expect("json");
        let got = decode_attachments(&value);
        let ids: Vec<&str> = got.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, vec!["D", "A"]);
    }

    #[test]
    fn attachments_without_file_id_are_skipped() {
        let value: serde_json::Value =
            serde_json::from_str(r#"{"document":{"file_name":"a.txt"}}"#).expect("json");
        assert!(decode_attachments(&value).is_empty());
        // 纯文本消息 → 没有附件。
        let none: serde_json::Value = serde_json::from_str(r#"{"text":"hi"}"#).expect("json");
        assert!(decode_attachments(&none).is_empty());
    }

    #[test]
    fn id_hash_is_stable_and_injective_enough() {
        assert_eq!(id_hash("abc"), id_hash("abc"), "同 id 同哈希（临时文件名要稳）");
        assert_ne!(id_hash("abc"), id_hash("abd"));
        assert_ne!(id_hash("FILE1"), id_hash("FILE2"));
    }

    #[test]
    fn fetch_attachment_without_token_reports_the_env_var() {
        let ch = TelegramChannel::new("NEOBOT_TG_UNSET_FOR_FETCH_XYZ");
        let dest = crate::nt_testutil::temp_dir("fetch-test");
        let err = ch.fetch_attachment("FILE1", &dest).expect_err("no token");
        assert!(err.to_string().contains("NEOBOT_TG_UNSET_FOR_FETCH_XYZ"), "{err}");
    }

    // ---- 假服务器：真正打一遍 HTTP（早先没有注入点，这段一行都跑不了） ----

    /// 起一个假 Telegram 服务器，按脚本回放应答。
    ///
    /// 返回 `(base_url, 收到的请求路径日志)`。请求日志是共享 `Arc<Mutex<Vec<String>>>`，
    /// 测试结束时用它断言「真的发了正确的请求」—— 而不是只看回包对不对。
    fn fake_server(replies: Vec<String>) -> (String, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
        use std::io::{Read as _, Write as _};
        use std::sync::{Arc, Mutex};
        let queue = Arc::new(Mutex::new((replies, 0usize)));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen_thread = Arc::clone(&seen);
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr").to_string();
        std::thread::spawn(move || {
            for _ in 0..32 {
                let Ok((mut stream, _)) = listener.accept() else { return };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let mut buf = vec![0u8; 65536];
                let Ok(n) = stream.read(&mut buf) else { continue };
                if n == 0 { continue; }
                let req = String::from_utf8_lossy(&buf).into_owned();
                let path = req
                    .lines()
                    .next()
                    .unwrap_or("")
                    .split(' ')
                    .nth(1)
                    .unwrap_or("")
                    .to_owned();
                seen_thread.lock().expect("lock").push(path);
                let reply = queue
                    .lock()
                    .ok()
                    .and_then(|mut q| {
                        if q.0.is_empty() { None } else {
                            let i = q.1 % q.0.len();
                            q.1 += 1;
                            Some(q.0[i].clone())
                        }
                    })
                    .unwrap_or_else(|| "{}".to_owned());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.len(),
                    reply
                );
                if stream.write_all(response.as_bytes()).is_err() { return; }
            }
        });
        (format!("http://{addr}"), seen)
    }

    /// 带 token 的测试适配器（token 走环境变量，故注入一个假的）。
    fn chan_with(base: &str) -> TelegramChannel {
        std::env::set_var("NEOBOT_TG_FAKE_TOKEN", "123456:FAKE-TOKEN");
        TelegramChannel::new("NEOBOT_TG_FAKE_TOKEN").with_base(base)
    }

    #[test]
    fn probe_over_http_reaches_get_me_and_returns_the_bot_name() {
        let reply = r#"{"ok":true,"result":{"id":7,"username":"my_bot","first_name":"Neo"}}"#;
        let (base, seen) = fake_server(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        let health = ch.probe();
        assert!(health.ok, "{}", health.detail);
        assert!(health.info.contains("my_bot"), "{}", health.info);
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1, "应只发一个请求");
        // 关键断言：**真的打到了 getMe，且 token 进了 URL**。
        assert!(log[0].starts_with("/bot123456:FAKE-TOKEN/getMe"), "请求路径 = {}", log[0]);
    }

    #[test]
    fn bot_api_ok_false_becomes_a_readable_error() {
        // Telegram 永远回 200 + ok:false；不查 ok 字段会把失败当成功。
        let reply = r#"{"ok":false,"error_code":401,"description":"Unauthorized"}"#;
        let (base, _seen) = fake_server(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        let health = ch.probe();
        assert!(!health.ok);
        assert!(health.detail.contains("Unauthorized"), "{}", health.detail);
    }

    #[test]
    fn send_actually_posts_and_returns_the_message_id() {
        let reply = r#"{"ok":true,"result":{"message_id":555}}"#;
        let (base, seen) = fake_server(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        let got = ch
            .send(&OutboundMessage {
                chat: "42".to_owned(),
                text: "你好 world".to_owned(),
                attachments: Vec::new(),
                edit_of: None,
            })
            .expect("send");
        assert_eq!(got, "555");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1);
        assert!(log[0].starts_with("/bot123456:FAKE-TOKEN/sendMessage"), "{}", log[0]);
        // 非 ASCII / 空格 / & 必须被百分号编码，否则 URL 会被截断或注入参数。
        assert!(log[0].contains("%E4%BD%A0%E5%A5%BD"), "中文未编码：{}", log[0]);
        assert!(log[0].contains("%20world"), "空格未编码：{}", log[0]);
        assert!(log[0].contains("chat_id=42"), "{}", log[0]);
    }

    #[test]
    fn send_preserves_injection_shaped_text_as_data() {
        // 正文里的 `&` 必须编码成 %26，否则能追加出第二个查询参数。
        let reply = r#"{"ok":true,"result":{"message_id":1}}"#;
        let (base, seen) = fake_server(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        ch.send(&OutboundMessage {
            chat: "1".to_owned(),
            text: "a&admin=1".to_owned(),
            attachments: Vec::new(),
            edit_of: None,
        })
        .expect("send");
        let log = seen.lock().expect("lock").clone();
        assert!(log[0].contains("text=a%26admin%3D1"), "& 未编码：{}", log[0]);
        assert!(!log[0].contains("&admin=1"), "参数被注入了：{}", log[0]);
    }

    #[test]
    fn poll_over_http_advances_the_offset() {
        let first = r#"{"ok":true,"result":[
            {"update_id":10,"message":{"message_id":1,"chat":{"id":5,"type":"private"},
             "from":{"id":9,"first_name":"A"},"text":"hi"}},
            {"update_id":11,"message":{"message_id":2,"chat":{"id":5,"type":"private"},
             "from":{"id":9,"first_name":"A"},"text":"there"}}]}"#;
        let second = r#"{"ok":true,"result":[]}"#;
        // 注意脚本里第一格是 getMe 的回包：`poll()` 首次会**先探活一次**
        // （为了拿到 @bot_username 剥群命令后缀），然后才 getUpdates。
        // 这是有意的行为，故在这里显式钉住，别让后人以为它是 bug。
        let get_me = r#"{"ok":true,"result":{"id":7,"username":"my_bot"}}"#;
        let (base, seen) = fake_server(vec![
            get_me.to_owned(),
            first.to_owned(),
            second.to_owned(),
        ]);
        let mut ch = chan_with(&base);
        let got = ch.poll().expect("poll");
        assert_eq!(got.len(), 2, "应收到两条：{got:?}");
        assert_eq!(got[0].text, "hi");
        // 第二次轮询必须带 offset=12（max update_id + 1），否则会无限重放同一条。
        let _ = ch.poll().expect("poll 2");
        let log = seen.lock().expect("lock").clone();
        assert!(
            log[0].contains("getMe"),
            "首次 poll 应先探活取 @username（群命令要靠它剥后缀）：{:?}",
            log
        );
        let get_updates = log
            .iter()
            .find(|path| path.contains("getUpdates"))
            .expect("应有 getUpdates");
        assert!(get_updates.contains("offset=0"), "首次不该带非零 offset：{get_updates}");
        // 第二次 getUpdates 必须带 offset=12。
        let updates: Vec<&String> = log.iter().filter(|p| p.contains("getUpdates")).collect();
        assert_eq!(updates.len(), 2, "两次 poll 各一次 getUpdates：{log:?}");
        assert!(
            updates[1].contains("offset=12"),
            "游标没推进（max update_id + 1）：{}",
            updates[1]
        );
    }

    #[test]
    fn with_base_rejects_blank_and_falls_back_to_the_real_host() {
        let ch = TelegramChannel::new("NEOBOT_TG_X").with_base("   ");
        assert_eq!(ch.api_base(), API_BASE, "空 base 应回落到真主机");
        // 尾斜杠会被去掉，免得拼出 `//bot`。
        let ch2 = TelegramChannel::new("NEOBOT_TG_X").with_base("http://127.0.0.1:9/");
        assert_eq!(ch2.api_base(), "http://127.0.0.1:9");
        // 默认就是真主机。
        assert_eq!(TelegramChannel::new("NEOBOT_TG_X").api_base(), API_BASE);
    }

    #[test]
    fn missing_token_reports_the_env_var_name() {
        let ch = TelegramChannel::new("NEOBOT_DEFINITELY_UNSET_TOKEN_XYZ");
        let err = ch.token().expect_err("should be unset");
        let text = err.to_string();
        // 报错要说清是**哪个变量**（只报「token 无效」等于让用户猜）。
        assert!(text.contains("NEOBOT_DEFINITELY_UNSET_TOKEN_XYZ"), "{text}");
    }

    #[test]
    fn empty_token_env_falls_back_to_default() {
        let ch = TelegramChannel::new("   ");
        assert_eq!(ch.token_env(), DEFAULT_TOKEN_ENV);
    }

    #[test]
    fn probe_reports_failure_without_panicking() {
        // 没 token → probe 应当回 ok=false 而不是 panic。
        let ch = TelegramChannel::new("NEOBOT_DEFINITELY_UNSET_TOKEN_XYZ");
        let health = ch.probe();
        assert!(!health.ok);
        assert!(!health.detail.is_empty(), "失败必须带原因");
    }

    #[test]
    fn split_text_leaves_short_text_alone() {
        assert_eq!(split_text("短句", 100), vec!["短句".to_owned()]);
        assert!(split_text("   ", 100).is_empty(), "纯空白不发");
    }

    #[test]
    fn split_text_cuts_on_paragraph_boundaries() {
        let para = "行".repeat(20);
        let text = format!("{para}\n\n{para}\n\n{para}");
        let pieces = split_text(&text, 50);
        assert!(pieces.len() > 1, "超限必须切");
        for piece in &pieces {
            assert!(piece.chars().count() <= 50, "每片都该在限内：{}", piece.chars().count());
        }
        // 切完不能丢内容（拼回去应等于原文去掉首尾空白）。
        assert_eq!(pieces.join("").replace(' ', ""), text.replace(' ', ""));
    }

    #[test]
    fn split_text_handles_one_giant_line() {
        let text = "x".repeat(500);
        let pieces = split_text(&text, 100);
        assert_eq!(pieces.len(), 5);
        assert_eq!(pieces.concat(), text, "硬切也不能丢字符");
    }

    #[test]
    fn split_text_respects_multibyte_boundaries() {
        let text = "中".repeat(300);
        let pieces = split_text(&text, 100);
        assert_eq!(pieces.concat(), text, "不能把多字节字符切碎");
        assert_eq!(pieces.len(), 3);
    }

    #[test]
    fn percent_encoding_escapes_url_syntax() {
        assert_eq!(percent_encode("a b"), "a%20b");
        assert_eq!(percent_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(percent_encode("中文"), "%E4%B8%AD%E6%96%87");
        assert_eq!(percent_encode("plain-Text_1.0~"), "plain-Text_1.0~");
    }

    #[test]
    fn send_refuses_empty_payload() {
        let ch = TelegramChannel::new("NEOBOT_TG_TEST");
        let err = ch
            .send(&OutboundMessage {
                chat: "1".to_owned(),
                text: "   ".to_owned(),
                attachments: Vec::new(),
                edit_of: None,
            })
            .expect_err("empty");
        assert!(err.to_string().contains("nothing to send"), "{err}");
    }

    // ---- 出站附件（multipart）----

    /// 完整捕获一次 HTTP 请求（方法 / 路径 / 头 / body）。
    ///
    /// 单独写一个而不复用上面那个 `fake_server`：那个只 `read` 一次、只记路径，
    /// 而附件这条路的**全部要害都在 body 里**（boundary、filename、注入）。
    /// 一次 `read` 也拿不全 body（头部和 body 通常分两段到），故这里读到
    /// `Content-Length` 满足为止 —— 否则测的是「我们碰巧发出去的东西」。
    #[derive(Debug, Clone)]
    struct Captured {
        method: String,
        path: String,
        headers: String,
        body: Vec<u8>,
    }

    impl Captured {
        /// 从 Content-Type 里取 boundary（没有就空串）。
        fn boundary(&self) -> String {
            self.headers
                .lines()
                .find(|line| {
                    let lower = line.to_ascii_lowercase();
                    lower.starts_with("content-type:") && lower.contains("boundary=")
                })
                .and_then(|line| line.split_once("boundary="))
                .map(|(_, rest)| rest.trim().to_owned())
                .unwrap_or_default()
        }

        fn body_text(&self) -> String {
            String::from_utf8_lossy(&self.body).into_owned()
        }

        /// 某个字面量在 body 里出现的次数（注入测试要数「有几个 chat_id」）。
        fn count(&self, needle: &str) -> usize {
            self.body_text().matches(needle).count()
        }
    }

    /// 起一个把**完整请求**记下来的假服务器（应答按脚本回放）。
    fn fake_server_capture(
        replies: Vec<String>,
    ) -> (
        String,
        std::sync::Arc<std::sync::Mutex<Vec<Captured>>>,
    ) {
        use std::io::{Read as _, Write as _};
        use std::sync::{Arc, Mutex};
        let queue = Arc::new(Mutex::new((replies, 0usize)));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen_thread = Arc::clone(&seen);
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr").to_string();
        std::thread::spawn(move || {
            for _ in 0..32 {
                let Ok((mut stream, _)) = listener.accept() else { return };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let mut raw: Vec<u8> = Vec::new();
                let mut buf = vec![0u8; 32 * 1024];
                // 读到头尾分隔符为止。
                loop {
                    match stream.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            raw.extend_from_slice(buf.get(..n).unwrap_or(&[]));
                            if find(&raw, b"\r\n\r\n").is_some() {
                                break;
                            }
                        }
                    }
                }
                let Some(head_end) = find(&raw, b"\r\n\r\n") else { continue };
                let head = String::from_utf8_lossy(raw.get(..head_end).unwrap_or(&[]))
                    .into_owned();
                let want = head
                    .lines()
                    .find(|line| {
                        let lower = line.to_ascii_lowercase();
                        lower.starts_with("content-length:")
                    })
                    .and_then(|line| line.split_once(':'))
                    .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                // 再读够 body。
                while raw.len() < head_end + 4 + want {
                    match stream.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => raw.extend_from_slice(buf.get(..n).unwrap_or(&[])),
                    }
                }
                let request_line = head.lines().next().unwrap_or("").to_owned();
                let mut words = request_line.split_whitespace();
                let method = words.next().unwrap_or("").to_owned();
                let path = words.next().unwrap_or("").to_owned();
                seen_thread.lock().expect("lock").push(Captured {
                    method,
                    path,
                    headers: head,
                    body: raw.get(head_end + 4..).unwrap_or(&[]).to_vec(),
                });
                let reply = queue
                    .lock()
                    .ok()
                    .and_then(|mut q| {
                        if q.0.is_empty() { None } else {
                            let i = q.1 % q.0.len();
                            q.1 += 1;
                            Some(q.0[i].clone())
                        }
                    })
                    .unwrap_or_else(|| "{}".to_owned());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.len(),
                    reply
                );
                if stream.write_all(response.as_bytes()).is_err() { return; }
            }
        });
        (format!("http://{addr}"), seen)
    }

    /// 字节子串查找（测试侧的 boundary 提取）。
    fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() || haystack.len() < needle.len() {
            return None;
        }
        (0..=(haystack.len() - needle.len())).find(|i| haystack.get(*i..*i + needle.len()) == Some(needle))
    }

    /// 造一个测试附件（返回临时目录与路径；内容里带**二进制**字节，
    /// 顺带证明拼出来的 body 真的是逐字节的）。
    fn temp_attachment(tag: &str, name: &str, body: &[u8]) -> (std::path::PathBuf, String) {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-tg-out-{}", tag));
        drop(std::fs::remove_dir_all(&dir));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join(name);
        std::fs::write(&path, body).expect("write");
        (dir, path.to_string_lossy().into_owned())
    }

    #[test]
    fn supports_attachments_is_reported_honestly() {
        let ch = TelegramChannel::new("NEOBOT_TG_TEST");
        assert!(
            ch.supports_attachments(),
            "能发就如实说 —— 这一位是上层的降级开关，谎报会让上层把文件悄悄丢掉"
        );
    }

    #[test]
    fn send_with_attachment_posts_a_well_formed_multipart_body() {
        let reply = r#"{"ok":true,"result":{"message_id":777}}"#;
        let (base, seen) = fake_server_capture(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        // 故意含 0x00 / 0xFF：证明拼出来的是真字节而不是被 UTF-8 洗过。
        let (dir, path) = temp_attachment("wellformed", "report.txt", b"id,name\n1,\xff\x00\xfe\n");
        let got = ch
            .send(&OutboundMessage {
                chat: "42".to_owned(),
                text: "看这个报告".to_owned(),
                attachments: vec![path.clone()],
                edit_of: None,
            })
            .expect("send");
        assert_eq!(got, "777", "应回读出 message_id");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1, "带附件应只发一个请求：{log:?}");
        // 关键断言 1：**POST**（附件在 body 里，GET 装不下）。
        assert_eq!(log[0].method, "POST", "附件必须走 POST：{:?}", log[0].method);
        assert!(
            log[0].path.starts_with("/bot123456:FAKE-TOKEN/sendDocument"),
            "请求路径 = {}",
            log[0].path
        );
        // 关键断言 2：Content-Type 带 boundary，且 body 里真的用了同一个 boundary。
        let boundary = log[0].boundary();
        assert!(
            boundary.starts_with("----NeoTrixBoundary"),
            "Content-Type 应声明 boundary：{}",
            log[0].headers
        );
        let body = log[0].body_text();
        assert!(
            body.starts_with(&format!("--{boundary}\r\n")),
            "body 必须以 boundary 开头：{body}"
        );
        assert!(
            body.ends_with(&format!("--{boundary}--\r\n")),
            "body 必须以闭合 boundary 收尾：{}",
            body.get(body.len().saturating_sub(80)..).unwrap_or("<tail>")
        );
        // 关键断言 3：三个要件都在（boundary / filename= / chat_id）。
        assert!(body.contains("Content-Disposition: form-data; name=\"chat_id\""), "{body}");
        assert!(body.contains("\r\n\r\n42\r\n"), "chat_id 的值应是 42：{body}");
        assert!(
            body.contains("Content-Disposition: form-data; name=\"document\"; filename=\"report.txt\""),
            "文件名应带在 Content-Disposition 里：{body}"
        );
        assert!(body.contains("Content-Type: application/octet-stream"), "{body}");
        // 关键断言 4：caption 作为一个**普通字段**（不是 part 头）。
        assert!(body.contains("name=\"caption\"") && body.contains("看这个报告"), "{body}");
        // 关键断言 5：文件内容逐字节进去，没被当 UTF-8 处理。
        assert!(contains_bytes(&log[0].body, b"1,\xff\x00\xfe"), "文件字节被改动了：{body}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn send_posts_several_attachments_as_one_request() {
        // 一次请求、一次成败：拆成多次就丢了「要么全到要么全不到」的保证。
        let reply = r#"{"ok":true,"result":[{"message_id":1},{"message_id":2}]}"#;
        let (base, seen) = fake_server_capture(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        let (dir_a, a) = temp_attachment("multi-a", "a.txt", b"AAA");
        let (dir_b, b) = temp_attachment("multi-b", "b.txt", b"BBB");
        let got = ch
            .send(&OutboundMessage {
                chat: "7".to_owned(),
                text: String::new(),
                attachments: vec![a, b],
                edit_of: None,
            })
            .expect("send");
        // 多文件时 result 是**数组**，也要能读出 id。
        assert_eq!(got, "2", "album 的 result 是数组，也得读得出 id");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1, "两个文件应打包成一个请求：{log:?}");
        let body = log[0].body_text();
        assert!(body.contains("filename=\"a.txt\"") && body.contains("filename=\"b.txt\""), "{body}");
        // 两个文件各一个 part。
        assert_eq!(log[0].count("name=\"document\""), 2, "每个文件一个 part：{body}");
        // 没有正文就不该有 caption 字段。
        assert!(!body.contains("name=\"caption\""), "无正文不该塞空 caption：{body}");
        let _ = std::fs::remove_dir_all(&dir_a);
        let _ = std::fs::remove_dir_all(&dir_b);
    }

    #[test]
    fn injection_shaped_filename_cannot_break_out_of_its_part() {
        let reply = r#"{"ok":true,"result":{"message_id":9}}"#;
        let (base, seen) = fake_server_capture(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        // 真·注入形状：CRLF + 伪 boundary + 伪字段（想改 chat_id 把文件发去别处）。
        // POSIX 文件名允许 CR/LF/引号（只禁 `/` 与 NUL），所以这个文件名真造得出来。
        let evil = "evil\"\r\n--X\r\nContent-Disposition: form-data; name=\"chat_id\"\r\n\r\n999\r\n--X.txt";
        let (dir, path) = temp_attachment("inject", evil, b"payload");
        ch.send(&OutboundMessage {
            chat: "42".to_owned(),
            text: String::new(),
            attachments: vec![path],
            edit_of: None,
        })
        .expect("send");
        let log = seen.lock().expect("lock").clone();
        let body = log[0].body_text();
        // 关键：整个 body 里**只有我们那一个** chat_id 字段，值仍是 42。
        assert_eq!(
            log[0].count("name=\"chat_id\""),
            1,
            "文件名注入出了额外的字段：{body}"
        );
        assert!(body.contains("\r\n\r\n42\r\n"), "chat_id 必须是 42：{body}");
        // 更直接的判据：part 头的个数必须恰好等于 part 数。
        // 数的是 `Content-Disposition: form-data;`（**带分号**）—— 分号是
        // `safe_filename` 保证会丢掉的字符，所以文件名里伪造不出这个前缀。
        assert_eq!(
            log[0].count("Content-Disposition: form-data;"),
            2,
            "只该有 2 个 part 头（伪造字段会多出来）：{body}"
        );
        // filename 的**值**里不许有引号 / CR / LF —— 那才是逃逸的真条件。
        let value = body
            .split_once("filename=\"")
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(value, _)| value.to_owned())
            .unwrap_or_default();
        assert!(
            !value.contains(['"', '\r', '\n', ';', '\\']),
            "filename 值里有能逃出去的字符：{value:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn percent_encoding_shaped_and_traversal_names_stay_one_unescaped_token() {
        let reply = r#"{"ok":true,"result":{"message_id":9}}"#;
        let (base, seen) = fake_server_capture(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        // `%0D%0A` 是**字面量**五个字符，不是 CR LF —— 合法且无害，
        // 但要确保我们没有把它「解码」成真换行（那才是注入）。
        let (dir_a, a) = temp_attachment("pct", "a%0D%0A%22chat_id%22.txt", b"x");
        // 路径里带 `..`：只该看**最后一段**（`sub/../a.txt` 指向 a.txt）。
        let dir_b = crate::nt_testutil::temp_dir("tg-out-trav2");
        drop(std::fs::remove_dir_all(&dir_b));
        std::fs::create_dir_all(dir_b.join("sub")).expect("mkdir");
        std::fs::write(dir_b.join("a.txt"), b"x").expect("write");
        let trav = format!("{}/sub/../a.txt", dir_b.to_string_lossy());
        ch.send(&OutboundMessage {
            chat: "42".to_owned(),
            text: String::new(),
            attachments: vec![a, trav],
            edit_of: None,
        })
        .expect("send");
        let log = seen.lock().expect("lock").clone();
        let body = log[0].body_text();
        // 字面量 `%0D%0A` 原样留在文件名里（没有被解码成真换行）。
        assert!(body.contains("filename=\"a%0D%0A%22chat_id%22.txt\""), "{body}");
        // 目录成分一个都不该进 body（它们不是文件名）。
        assert!(body.contains("filename=\"a.txt\""), "只取最后一段：{body}");
        assert!(!body.contains("/sub/"), "目录成分不该进 filename：{body}");
        assert!(!body.contains("neobot-tg-out-trav2"), "父目录不该进 body：{body}");
        // 也没有真换行混进 filename 那一行。
        for line in body.lines().filter(|line| line.contains("filename=")) {
            assert!(!line.contains('\r'), "{line:?}");
        }
        let _ = std::fs::remove_dir_all(&dir_a);
        let _ = std::fs::remove_dir_all(&dir_b);
    }

    #[test]
    fn boundary_never_collides_with_the_file_content() {
        let (dir, path) = temp_attachment("collide", "x.bin", b"hello");
        let payload = std::fs::read(&path).expect("read");
        let boundary = pick_boundary(&[&payload]).expect("boundary");
        // 关键：挑出来的 boundary 真的不在内容里。
        assert!(
            !contains_bytes(&payload, &boundary),
            "boundary 撞上了文件内容，multipart 会被切碎"
        );
        // 全部是 ASCII（Content-Type 头里不能用别的）。
        assert!(boundary.is_ascii(), "boundary 必须是 ASCII");
        // 空内容也要挑得出来。
        assert!(pick_boundary(&[&[], b""]).is_ok());
        // 反向：碰撞检测**本身**要真的能发现（否则上面那条恒真、等于没测）。
        assert!(
            contains_bytes(b"xxNeoTrixBoundaryyy", b"NeoTrixBoundary"),
            "子串检测失灵了：boundary 撞上内容它却没报"
        );
        assert!(!contains_bytes(b"short", b"much longer needle"));
        // 贴着边界起止也要认。
        assert!(contains_bytes(b"ab", b"ab"));
        assert!(contains_bytes(b"xxab", b"ab"));
        assert!(contains_bytes(b"abxx", b"ab"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversized_attachment_is_refused_and_nothing_is_sent() {
        let reply = r#"{"ok":true,"result":{"message_id":1}}"#;
        let (base, seen) = fake_server_capture(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        let dir = crate::nt_testutil::temp_dir("tg-out-huge");
        drop(std::fs::remove_dir_all(&dir));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("huge.bin");
        // 稀疏文件：瞬间造出 50 MB + 1，不真的占盘也不真的读。
        let file = std::fs::File::create(&path).expect("create");
        file.set_len((MAX_ATTACHMENT_BYTES + 1) as u64).expect("set_len");
        drop(file);
        let err = ch
            .send(&OutboundMessage {
                chat: "42".to_owned(),
                text: "看这个".to_owned(),
                attachments: vec![path.to_string_lossy().into_owned()],
                edit_of: None,
            })
            .expect_err("超限必须拒");
        let text = err.to_string();
        // 关键：**一个字都没发**（连正文都没发）—— 半条消息比没有更让人困惑。
        assert!(
            seen.lock().expect("lock").is_empty(),
            "超限时不该发出任何请求"
        );
        assert!(text.contains(&MAX_ATTACHMENT_BYTES.to_string()), "要说清上限：{text}");
        assert!(text.contains("不发送"), "要说清是没发：{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn too_many_attachments_is_refused_before_any_request() {
        let (base, seen) = fake_server_capture(vec![r#"{"ok":true,"result":{}}"#.to_owned()]);
        let ch = chan_with(&base);
        let (dir, one) = temp_attachment("many", "a.txt", b"x");
        let paths: Vec<String> = (0..=MAX_ATTACHMENTS_PER_MESSAGE)
            .map(|_| one.clone())
            .collect();
        let err = ch
            .send(&OutboundMessage {
                chat: "42".to_owned(),
                text: String::new(),
                attachments: paths,
                edit_of: None,
            })
            .expect_err("超个数必须拒");
        assert!(seen.lock().expect("lock").is_empty(), "不该发出任何请求");
        assert!(err.to_string().contains("上限"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_attachment_fails_the_whole_send_instead_of_sending_text() {
        let reply = r#"{"ok":true,"result":{"message_id":1}}"#;
        let (base, seen) = fake_server_capture(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        let (dir, good) = temp_attachment("missing", "ok.txt", b"fine");
        let err = ch
            .send(&OutboundMessage {
                chat: "42".to_owned(),
                text: "两个文件都在这".to_owned(),
                // 第一个正常、第二个根本不存在 —— 正是最容易「发出去一个就完事」的形状。
                attachments: vec![good, "/tmp/neobot-does-not-exist-9f2a.txt".to_owned()],
                edit_of: None,
            })
            .expect_err("缺一个就该整条失败");
        assert!(err.to_string().contains("neobot-does-not-exist-9f2a.txt"), "{err}");
        // **核心保证**：连一个请求都不能发出去 ——
        // 「只发文字 + 悄悄丢文件」正是本模块要守住的那条不变式。
        assert!(
            seen.lock().expect("lock").is_empty(),
            "任一附件失败就必须整条不发，不能先发文字"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_rejected_upload_is_reported_not_swallowed() {
        // Bot API 回 200 + ok:false：忘了查 ok 字段就等于「假装发成了」。
        let reply = r#"{"ok":false,"error_code":400,"description":"Bad Request: file is too big"}"#;
        let (base, _seen) = fake_server_capture(vec![reply.to_owned()]);
        let ch = chan_with(&base);
        let (dir, path) = temp_attachment("rejected", "a.txt", b"x");
        let err = ch
            .send(&OutboundMessage {
                chat: "42".to_owned(),
                text: String::new(),
                attachments: vec![path],
                edit_of: None,
            })
            .expect_err("ok:false 必须变成 Err");
        let text = err.to_string();
        assert!(text.contains("sendDocument"), "{text}");
        assert!(text.contains("file is too big"), "平台的原因要透出来：{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn caption_over_the_limit_is_sent_as_a_second_message_not_truncated() {
        // caption 上限 1024：溢出的正文必须**另发一条**。
        // 截断的话用户永远看不到后半个字，而「模型说了但那边少了半段」最难发现。
        let doc = r#"{"ok":true,"result":{"message_id":11}}"#;
        let msg = r#"{"ok":true,"result":{"message_id":12}}"#;
        let (base, seen) = fake_server_capture(vec![doc.to_owned(), msg.to_owned()]);
        let ch = chan_with(&base);
        let (dir, path) = temp_attachment("caption", "a.txt", b"x");
        let text = "字".repeat(MAX_CAPTION_CHARS + 40);
        ch.send(&OutboundMessage {
            chat: "42".to_owned(),
            text: text.clone(),
            attachments: vec![path],
            edit_of: None,
        })
        .expect("send");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 2, "文档一条 + 补发正文一条：{log:?}");
        let first = &log[0];
        assert!(first.path.contains("sendDocument"), "{}", first.path);
        // caption 恰好 1024 字符（一个不多）。
        // 取值：part 头之后到下一个 `\r\n--`（下一个 delimiter）之前。
        let first_body = first.body_text();
        let after_header = first_body
            .split_once("name=\"caption\"")
            .map(|(_, rest)| rest.to_owned())
            .unwrap_or_default();
        let caption = after_header
            .trim_start_matches("\r\n")
            .split("\r\n--")
            .next()
            .unwrap_or("");
        assert_eq!(caption.chars().count(), MAX_CAPTION_CHARS, "caption 应恰好到限：{caption}");
        // caption 全是原文的前缀（一个字都没被改 / 丢）。
        assert!(text.starts_with(caption), "caption 必须是原文的前缀");
        // 第二条带上剩下的 40 字，且不丢字。
        let tail = log[1].path.clone();
        assert!(tail.contains("sendMessage"), "{}", tail);
        // 拼回去 = 原文（一条都不许少）。
        let sent_tail = log[1]
            .path
            .split("text=")
            .nth(1)
            .map(|rest| percent_decode(rest.split('&').next().unwrap_or("")))
            .unwrap_or_default();
        assert_eq!(sent_tail.chars().count(), 40, "补发的是剩下的 40 字：{sent_tail}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// URL 查询串的百分号解码（测试断言用；只解 `%XX`，不解 `+`）。
    fn percent_decode(value: &str) -> String {
        let bytes = value.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes.get(i) == Some(&b'%') {
                let hex = value.get(i + 1..i + 3).unwrap_or("");
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
            out.push(*bytes.get(i).unwrap_or(&b'?'));
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    #[test]
    fn safe_filename_strips_injection_vectors_and_keeps_ordinary_names() {
        // 普通名（含中文）原样保留 —— 用户在 Telegram 里看到的就是它。
        assert_eq!(safe_filename("/tmp/report.pdf"), "report.pdf");
        assert_eq!(safe_filename("/data/attachments/季度 报表(终稿).xlsx"), "季度 报表(终稿).xlsx");
        // 注入向量一定被清掉（CR/LF/引号/分号/反斜杠是仅有的危险字符）。
        for evil in [
            "a\"\r\n\r\n--B\r\nx.txt",
            "a\\\r\nx.txt",
            "a;b.txt",
            "dir/../../etc/passwd",
        ] {
            let got = safe_filename(evil);
            assert!(!got.contains(['"', '\r', '\n', ';', '\\']), "{evil:?} -> {got:?}");
        }
        // 正常名字里的 `%` `+` `#` **要留住**（percent 编码只存在于 URL，
        // 服务端不会去解码 multipart 的文件名）—— 早先的白名单把它们吃了。
        assert_eq!(safe_filename("report%20final.pdf"), "report%20final.pdf");
        assert_eq!(safe_filename("v1.2+build#3.pdf"), "v1.2+build#3.pdf");
        assert_eq!(safe_filename("a&b.txt"), "a&b.txt");
        // 退化形状要有可用名（空文件名会被平台拒）。
        // 注意 `/tmp/` 的最后一段是 `tmp`（尾斜杠被 Path 归一掉了），
        // 那是**对的** —— 它确实是路径的最后一段。
        assert_eq!(safe_filename("/tmp/"), "tmp");
        assert_eq!(safe_filename("/"), "attachment.bin");
        assert_eq!(safe_filename(""), "attachment.bin");
        assert_eq!(safe_filename(".."), "attachment.bin");
        assert_eq!(safe_filename("   "), "attachment.bin");
        // 超长截断。
        let long = format!("{}.txt", "x".repeat(400));
        assert!(safe_filename(&long).chars().count() <= 120);
    }

    #[test]
    fn send_no_longer_declines_attachments_outright() {
        // 早先这里硬报错「附件发送尚未实现」；现在要么真发出去、要么真报错，
        // 但**不再**因为「没实现」而拒。守住的是「不静默丢」，不是「不实现」。
        let (base, seen) = fake_server_capture(vec![r#"{"ok":true,"result":{"message_id":5}}"#.to_owned()]);
        let ch = chan_with(&base);
        let (dir, path) = temp_attachment("noharderr", "a.txt", b"x");
        let got = ch
            .send(&OutboundMessage {
                chat: "1".to_owned(),
                text: "hi".to_owned(),
                attachments: vec![path],
                edit_of: None,
            })
            .expect("现在该发得出去");
        assert_eq!(got, "5");
        assert_eq!(seen.lock().expect("lock").len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- 编辑原消息（`edit_of` 三段接通）----

    /// 请求里出现了 `sendMessage` / `sendDocument`（**另发一条**的两种形态）。
    fn sent_a_new_message(seen: &[Captured]) -> bool {
        seen.iter().any(|req| {
            req.path.contains("/sendMessage") || req.path.contains("/sendDocument")
        })
    }

    /// 把请求路径里 Bot API 的方法名取出来（`/bot<token>/editMessageText?…`）。
    fn api_method(path: &str) -> String {
        path.trim_start_matches('/')
            .split('?')
            .next()
            .unwrap_or("")
            .split('/')
            .nth(1)
            .unwrap_or("")
            .to_owned()
    }

    fn out(chat: &str, text: &str, edit_of: Option<&str>) -> OutboundMessage {
        OutboundMessage {
            chat: chat.to_owned(),
            text: text.to_owned(),
            attachments: Vec::new(),
            edit_of: edit_of.map(str::to_owned),
        }
    }

    #[test]
    fn edit_of_goes_to_edit_message_text_and_sends_nothing_new() {
        // **回归锁，也是「永远发新消息」那种退化实现的照妖镜**：
        // 编辑成功时若还出现 sendMessage，这个测试就红。
        let (base, seen) =
            fake_server_capture(vec![r#"{"ok":true,"result":{"message_id":555}}"#.to_owned()]);
        let ch = chan_with(&base);
        let got = ch
            .send(&out("42", "补发结果在这", Some("555")))
            .expect("edit");
        assert_eq!(got, "555", "回的是被编辑的那条的 id");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1, "编辑成功只该有一个请求：{log:?}");
        assert_eq!(log[0].method, "GET");
        assert_eq!(api_method(&log[0].path), "editMessageText", "{:?}", log[0].path);
        // chat_id + message_id 是编辑的必需两件套（Bot API 少一个就 400）。
        assert!(log[0].path.contains("chat_id=42"), "{}", log[0].path);
        assert!(log[0].path.contains("message_id=555"), "{}", log[0].path);
        assert!(
            !sent_a_new_message(&log),
            "编辑成功就不该再发一条：{:?}",
            log.iter().map(|r| &r.path).collect::<Vec<_>>()
        );
        // 记账：真的编辑了，且没有降级。
        let outcome = ch.last_send_outcome().expect("outcome");
        assert!(outcome.edited, "{}", outcome.summary());
        assert_eq!(outcome.edit_of.as_deref(), Some("555"));
        assert!(outcome.degraded.is_none(), "{}", outcome.summary());
        assert!(outcome.already_there.is_none());
    }

    #[test]
    fn missing_edit_of_sends_a_new_message_as_before() {
        // 老 payload（没这个键）走的是**原来那条**路：一个字都不能变。
        let (base, seen) =
            fake_server_capture(vec![r#"{"ok":true,"result":{"message_id":9}}"#.to_owned()]);
        let ch = chan_with(&base);
        let got = ch.send(&out("42", "普通回复", None)).expect("send");
        assert_eq!(got, "9");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1);
        assert_eq!(api_method(&log[0].path), "sendMessage", "{:?}", log[0].path);
        let outcome = ch.last_send_outcome().expect("outcome");
        assert!(!outcome.edited);
        assert_eq!(outcome.edit_of, None);
        assert!(outcome.degraded.is_none(), "没有编辑意图就不该有降级");
    }

    #[test]
    fn blank_edit_of_counts_as_absent_instead_of_failing() {
        // `""` / `"   "` 会在 `try_edit_text` 里被判成「不是整数」而降级 ——
        // 那会让「本来只想发新消息」的调用方平白多一次降级记账。
        for blank in ["", "   "] {
            let (base, seen) =
                fake_server_capture(vec![r#"{"ok":true,"result":{"message_id":3}}"#.to_owned()]);
            let ch = chan_with(&base);
            let got = ch.send(&out("42", "x", Some(blank))).expect("send");
            assert_eq!(got, "3");
            let log = seen.lock().expect("lock").clone();
            assert_eq!(api_method(&log[0].path), "sendMessage", "edit_of={blank:?} 不该触发编辑");
            assert!(ch.last_send_outcome().expect("outcome").degraded.is_none());
        }
    }

    #[test]
    fn rejected_edit_falls_back_to_a_new_message_and_says_why() {
        // 平台拒编辑（消息不存在 / 不是本 bot 发的 —— Telegram 只让 bot
        // 编辑自己发过的消息）时：发新消息，**并且**降级原因要看得见。
        let (base, seen) = fake_server_capture(vec![
            r#"{"ok":false,"error_code":400,"description":"Bad Request: message to edit not found"}"#
                .to_owned(),
            r#"{"ok":true,"result":{"message_id":777}}"#.to_owned(),
        ]);
        let ch = chan_with(&base);
        let got = ch.send(&out("42", "补发结果", Some("555"))).expect("降级后仍该发得出去");
        assert_eq!(got, "777", "降级后回的是**新发**那条的 id");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 2, "{log:?}");
        assert_eq!(api_method(&log[0].path), "editMessageText");
        assert_eq!(api_method(&log[1].path), "sendMessage");
        let outcome = ch.last_send_outcome().expect("outcome");
        assert!(!outcome.edited, "不该声称编辑了：{}", outcome.summary());
        assert_eq!(outcome.edit_of.as_deref(), Some("555"));
        let reason = outcome.degraded.as_deref().expect("必须记下降级原因");
        assert!(reason.contains("message to edit not found"), "降级原因要带上平台原话：{reason}");
        assert_eq!(outcome.message_id, "777");
        assert!(outcome.summary().contains("改发新消息"), "{}", outcome.summary());
    }

    #[test]
    fn rate_limited_edit_falls_back_and_names_the_limit() {
        // 429 是另一类降级：编辑撞了配额。仍退回发新消息（那条也可能撞，
        // 撞了就整条 Err 让上层走补发队列 —— 这一点在报告里说清）。
        let (base, seen) = fake_server_capture(vec![
            r#"{"ok":false,"error_code":429,"description":"Too Many Requests: retry after 3","parameters":{"retry_after":3}}"#
                .to_owned(),
            r#"{"ok":true,"result":{"message_id":31}}"#.to_owned(),
        ]);
        let ch = chan_with(&base);
        let got = ch.send(&out("42", "补发结果", Some("555"))).expect("降级");
        assert_eq!(got, "31");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(api_method(&log[0].path), "editMessageText");
        assert_eq!(api_method(&log[1].path), "sendMessage");
        let reason = ch
            .last_send_outcome()
            .and_then(|o| o.degraded)
            .expect("必须记下降级原因");
        assert!(reason.contains("Too Many Requests"), "{reason}");
    }

    #[test]
    fn not_modified_means_the_goal_is_already_met_so_nothing_new_is_sent() {
        // 「内容一模一样」不是降级理由：原消息已经是这段话了。
        // 再发一条 = 用户看到重复的两条 —— 正是这条路径要消灭的东西。
        let (base, seen) = fake_server_capture(vec![
            r#"{"ok":false,"error_code":400,"description":"Bad Request: message is not modified"}"#
                .to_owned(),
        ]);
        let ch = chan_with(&base);
        let got = ch.send(&out("42", "已经在屏上的那句话", Some("555"))).expect("幂等");
        assert_eq!(got, "555");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1, "只该试一次编辑：{log:?}");
        assert_eq!(api_method(&log[0].path), "editMessageText");
        assert!(!sent_a_new_message(&log), "内容已一致时绝不能另发一条");
        let outcome = ch.last_send_outcome().expect("outcome");
        assert!(!outcome.edited, "没真编辑就不能说编辑了");
        assert!(outcome.degraded.is_none(), "这不是降级：{}", outcome.summary());
        assert!(outcome.already_there.is_some(), "{}", outcome.summary());
    }

    #[test]
    fn edit_with_attachments_degrades_without_pretending_the_media_was_replaced() {
        // editMessageText 换不了媒体（那是 editMessageMedia）。硬试的后果是
        // 「正文换了、附件还是旧的」—— 配错对的图比两条消息更难查。
        let (base, seen) =
            fake_server_capture(vec![r#"{"ok":true,"result":{"message_id":42}}"#.to_owned()]);
        let ch = chan_with(&base);
        let (dir, path) = temp_attachment("editatt", "a.txt", b"x");
        let mut msg = out("42", "看这个", Some("555"));
        msg.attachments = vec![path];
        let got = ch.send(&msg).expect("降级后发得出去");
        assert_eq!(got, "42");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1, "压根不该试编辑：{log:?}");
        assert_eq!(api_method(&log[0].path), "sendDocument");
        let reason = ch
            .last_send_outcome()
            .and_then(|o| o.degraded)
            .expect("必须记下降级原因");
        assert!(reason.contains("editMessageMedia"), "{reason}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversized_text_degrades_instead_of_asking_the_platform_to_reject_it() {
        // 正文超 4096：editMessageText 装不下（发新消息那条路会切分成多条）。
        let (base, seen) =
            fake_server_capture(vec![r#"{"ok":true,"result":{"message_id":8}}"#.to_owned()]);
        let ch = chan_with(&base);
        let long = "字".repeat(MAX_TEXT_CHARS + 1);
        let got = ch.send(&out("42", &long, Some("555"))).expect("降级");
        assert_eq!(got, "8");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(api_method(&log[0].path), "sendMessage", "不该先试一次注定被拒的编辑");
        let reason = ch
            .last_send_outcome()
            .and_then(|o| o.degraded)
            .expect("必须记下降级原因");
        assert!(reason.contains("4096"), "{reason}");
    }

    #[test]
    fn non_numeric_edit_of_degrades_locally_without_a_request() {
        // 平台要 message_id 是 Integer。垃圾值本地就认得出，别白跑一趟。
        let (base, seen) =
            fake_server_capture(vec![r#"{"ok":true,"result":{"message_id":6}}"#.to_owned()]);
        let ch = chan_with(&base);
        let got = ch.send(&out("42", "补发", Some("not-a-number"))).expect("降级");
        assert_eq!(got, "6");
        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 1, "不该为垃圾 id 打一次编辑请求：{log:?}");
        assert_eq!(api_method(&log[0].path), "sendMessage");
        let reason = ch
            .last_send_outcome()
            .and_then(|o| o.degraded)
            .expect("必须记下降级原因");
        assert!(reason.contains("Integer"), "{reason}");
    }

    #[test]
    fn a_broken_transport_is_not_worked_around_with_a_second_request() {
        // 凭据/网络坏了时**不**降级去发新消息：只会把同一个错误再犯一遍，
        // 还多花一次超时。上抛让上层退回补发队列才是对的。
        // 用一个**专属**的未设变量名（不能去动别的测试共用的那个 ——
        // 测试是并行的，动了会让它们随机红）。
        let ch = TelegramChannel::new("NEOBOT_TG_UNSET_FOR_EDIT_XYZ")
            .with_base("http://127.0.0.1:1");
        let err = ch
            .send(&out("42", "补发", Some("555")))
            .expect_err("没 token 就该报错");
        assert!(err.to_string().contains("NEOBOT_TG_UNSET_FOR_EDIT_XYZ"), "{err}");
        let degraded = ch.last_send_outcome().and_then(|o| o.degraded);
        assert!(
            degraded.is_none(),
            "传输层失败不该被记成「降级发新消息」：{degraded:?}"
        );
    }

    #[test]
    fn is_not_modified_recognises_the_platform_wording() {
        assert!(is_not_modified("editMessageText: Bad Request: message is not modified"));
        assert!(is_not_modified("editMessageText: Bad Request: MESSAGE IS NOT MODIFIED"));
        // 别把别的失败误判成「已达成」—— 那会让真失败被记成成功。
        for other in [
            "editMessageText: Bad Request: message to edit not found",
            "editMessageText: Too Many Requests: retry after 3",
            "editMessageText: Bad Request: can't parse entities",
            "editMessageText: Bad Request: message identifier is not specified",
        ] {
            assert!(!is_not_modified(other), "{other:?} 不该被当成已达成");
        }
    }

    #[test]
    fn placeholder_then_final_answer_hits_edit_message_text_end_to_end() {
        // 占位 → 覆盖的整条链路：先 `sendMessage` 发占位、拿回它的 message_id；
        // 跑完用 `edit_of` 把**那个 id** 交给 `editMessageText`。
        //
        // 关键断言是「**占位 ID 真被用作 editMessageText 的 message_id**」。
        // 这一步断了，占位与最终答案就会变成两条并排的消息 —— 用户同时看到
        // 「思考中…」和答案，占位路径等于白跑一遍。
        let (base, seen) = fake_server(vec![
            r#"{"ok":true,"result":{"message_id":42}}"#.to_owned(),
            r#"{"ok":true,"result":{"message_id":42,"text":"最终答案"}}"#.to_owned(),
        ]);
        let ch = chan_with(&base);
        // Telegram 有 editMessageText ⇒ 上层才会走「先占位后覆盖」（见派发层的编辑载体律）。
        assert!(ch.can_edit(), "Telegram 支持 editMessageText，占位→覆盖才是默认路径");

        let placeholder_id = ch
            .send(&OutboundMessage {
                chat: "42".to_owned(),
                text: "⏳ 思考中…".to_owned(),
                attachments: Vec::new(),
                edit_of: None,
            })
            .expect("占位发得出去");
        assert_eq!(placeholder_id, "42", "占位该回自己的 message_id");

        let got = ch
            .send(&OutboundMessage {
                chat: "42".to_owned(),
                text: "最终答案".to_owned(),
                attachments: Vec::new(),
                edit_of: Some(placeholder_id.clone()),
            })
            .expect("覆盖发得出去");
        assert_eq!(got, "42", "编辑成功回的还是那条消息的 id（不另发一条）");

        let log = seen.lock().expect("lock").clone();
        assert_eq!(log.len(), 2, "占位 + 覆盖各一次请求：{log:?}");
        // 端点顺序：先建消息、后改它。反了就说明两条是各自独立的。
        assert!(log[0].starts_with("/bot123456:FAKE-TOKEN/sendMessage"), "{}", log[0]);
        assert!(log[1].starts_with("/bot123456:FAKE-TOKEN/editMessageText"), "{}", log[1]);
        // 覆盖请求带的 message_id 就是上面那个占位 id（不是新 id、也不是别的常量）。
        assert!(
            log[1].contains("message_id=42"),
            "editMessageText 没拿占位的 id 去改：{}",
            log[1]
        );
        assert!(log[1].contains("chat_id=42"), "改的是同一个会话：{}", log[1]);
    }
}

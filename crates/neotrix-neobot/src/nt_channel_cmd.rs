//! `nt_channel_cmd` — IM 侧斜杠指令（在跑轮**之前**截，副作用最小面）。
//!
//! 抄自 dsh-im 的指令面，但只取四个真正有用的：
//! `/new`（重开一轮上下文）、`/stop`（停当前回合）、`/help`（自述）、
//! `/status`（说清现在什么状态）。其余一律当普通消息 —— 平台自己还有
//! 自己的命令面板，neobot 不去抢。
//!
//! 顺序律：指令**先于**跑轮判定，且**只对已放行的来源**生效。
//! 未通过访问闸门的消息在 `dispatch` 里就丢了，根本到不了这里 ——
//! 「陌生人发 `/new` 就重置了你的会话」是不能发生的事。
//!
//! 副作用律：指令**永不**触发 `run_local_turn`。`/new` 只是把绑定切到
//! 一个新会话，然后回一句确认 —— 让模型来处理「重置会话」是把一件
//! 确定性的小事变成一次不确定的模型调用。

use crate::nt_error::NtBotError;
use crate::nt_store::NeobotStore;

/// 识别出的指令。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `/new [标题]` — 重开一轮上下文。
    New { title: String },
    /// `/stop` — 停掉本会话正在跑的那轮。
    ///
    /// **当前是诚实失败**：跑轮同步阻塞（`run_local_turn_as` 不返回就读不到
    /// 下一条消息），运行中收不到这条命令。回执会说明这一点，而不是谎称
    /// 「已请求停止」。
    Stop,
    /// `/help` — 自述。
    Help,
    /// `/status` — 当前状态。
    Status,
}

impl Command {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New { .. } => "new",
            Self::Stop => "stop",
            Self::Help => "help",
            Self::Status => "status",
        }
    }
}

/// 识别一条斜杠指令；不是指令回 `None`。
///
/// 平台差异在**调用方**抹平：调用方先把 `@bot` 提及、群前缀去掉再传进来
/// （`nt_channel_telegram` 干这事）。本函数只认开头的 `/`。
pub fn parse(text: &str) -> Option<Command> {
    let trimmed = text.trim();
    let body = trimmed.strip_prefix('/')?;
    let mut parts = body.splitn(2, char::is_whitespace);
    let verb = parts.next()?.trim().to_ascii_lowercase();
    let rest = parts.next().unwrap_or("").trim();
    match verb.as_str() {
        "new" => Some(Command::New {
            title: rest.chars().take(80).collect(),
        }),
        "stop" => Some(Command::Stop),
        "help" | "帮助" => Some(Command::Help),
        "status" | "状态" => Some(Command::Status),
        _ => None,
    }
}

/// `/help` 的回文。
///
/// **纯文本律**：两个投放端都**不渲染 markdown**（IM 侧是纯文本直发，桌面侧
/// 前端 `esc()` 之后按字面显示），所以文案里**不许**放 markdown 记号 ——
/// 写了 `**当前不可用**`，用户在 IM 里看到的就是两个星号，在桌面里也是两个
/// 星号，两头都变成噪音。故正确做法是**改文案**，不是给桌面加渲染器。
/// 回归锁见测试 `help_text_carries_no_markdown_emphasis`。
pub fn help_text(channel_name: &str) -> String {
    format!(
        "我是 {name}，跑在你自己机器上的本地 agent。\n\
         指令：\n\
         - `/new [标题]` 开一段新上下文（当前这段就此封存）\n\
         - `/stop` 停掉正在跑的那一轮（当前不可用：跑轮同步，跑完才收得到新消息）\n\
         - `/help` 这段话\n\
         - `/status` 看当前绑定与状态\n\
         \n\
         别的都当普通消息处理。数据只落在你自己的机器上（~/.neobot）。",
        name = channel_name
    )
}

/// 指令执行的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    /// 回给用户的话。
    pub reply: String,
    /// 需要切换到的会话（`/new` 时是新会话 id）。
    pub switch_convo_to: Option<String>,
    /// 要不要停当前回合（`/stop`）。
    pub stop_requested: bool,
}

/// 渠道无关的本地回执：给**没有 IM 上下文**的调用方（桌面聊天框、CLI）用。
///
/// `execute` 的签名要 `store + &BotRow + channel_name`，而 `BotRow` 是 IM 侧
/// `channel × bot_id` 的绑定行 —— 桌面**根本没有**那一层。此前桌面为了不伪造
/// `BotRow`，把 `/help`、`/stop` 的回执**各自抄了一份**，只靠测试断言「与 IM
/// 字节相等」防漂移。那只证明「此刻一致」，证明不了「日后改一处会记得改另一处」。
/// 故把这两条的回执文本收敛到**本函数这一处**，`execute` 也从这里取
/// —— 两端口径**结构同源**，不是两份相似实现。
///
/// 不碰 `store`、不碰 `bot`、不返回 `Result`：这一层没有可失败的副作用。
///
/// **`None` 的含义**（调用方据此知道「别假装能本地答」）：
/// **这条命令需要渠道上下文，不属于本地回执范围**。
/// - `Status` 要读 bot 绑定与会话标题（store）；
/// - `New` 要建会话并改绑定（store）。
/// 桌面对这两条各有自己的如实答复（`/status` 报桌面真有的状态、`/new`
/// 明说不支持），**不该**来这个函数里凑一个假的。
pub fn local_reply(command: &Command, channel_name: &str) -> Option<CommandOutcome> {
    match command {
        Command::Help => Some(CommandOutcome {
            reply: help_text(channel_name),
            switch_convo_to: None,
            stop_requested: false,
        }),
        // `/stop` 的诚实边界：跑轮**同步阻塞**（`run_local_turn_as` 不返回就
        // 不会读下一条消息），所以「正在跑的那一轮」期间我**收不到** `/stop`。
        //
        // 早先的文案是「已请求停止当前回合」—— 那是在**骗用户**：整条链路上
        // 没有任何停止机制（`run_loop` 里连一个 stop 钩子都没有）。
        // 要真做出来得把调度改成并发的（线程池或 async），那是架构级一步，不是补丁。
        // 故此处只说清事实 + 给出真正能用的替代路径。
        //
        // 这段注释自己也被本轮改动证伪过一次：早先还写着「桌面聊天也不解析
        // 斜杠指令（走 `neobot_run_stream`），所以在这里教他用 `/stop` 也没用」——
        // 桌面**现在**解析了（`nt_cmd_run.rs` 的 `intercept_local_command`
        // 直接调本模块的 `parse` 与 `local_reply`），那句话已经不成立。
        // **教训**：写「某处不做 X」时先确认 X 不会随后被别的窗口接上 ——
        // 这正是本仓库反复修的那类「注释在说谎」，本次由我自己又犯了一次。
        Command::Stop => Some(CommandOutcome {
            // 早先这里写「想立刻打断，请到桌面 App 按发送键（它会变停止键）」——
            // 那条建议是**假的**：桌面那个键只置 `shell.stopRequested` 让渲染
            // 跳过增量（`main.ts:573/634/673/714`），`spawn_blocking` 里的
            // 跑轮照跑照写。用户以为打断了、其实工具还在执行，比说「停不了」
            // 更危险。
            //
            // 只给**真能生效**的替代路径：退出桌面 App 会终止进程，跑轮随之死。
            reply: "停不了：跑轮是同步的，这一轮跑完之前我不收新消息。\n\
                     这一轮没法中途打断（桌面 App 的停止键也只收起输出、不终止运行；\
                     要真的终止得退出那个 App）。\n\
                     下一条消息我照常处理。"
                .to_owned(),
            switch_convo_to: None,
            // 仍置位：将来调度改成并发时，这条命令已经在了。
            stop_requested: true,
        }),
        // 这两条要读 store（会话标题 / 建会话改绑定）⇒ 不属本地回执范围。
        Command::Status | Command::New { .. } => None,
    }
}

/// 执行指令（IM 侧：带 `store` + `BotRow` 上下文）。
///
/// `/help` 与 `/stop` **委托**给 [`local_reply`]，回执文本只存在于那一个函数里
/// —— 本函数不重写第二份，桌面也不用抄。
pub fn execute(
    store: &NeobotStore,
    command: &Command,
    bot: &crate::nt_store::BotRow,
    channel_name: &str,
) -> Result<CommandOutcome, NtBotError> {
    // 与渠道无关的两条走共享出口。
    if let Some(outcome) = local_reply(command, channel_name) {
        return Ok(outcome);
    }
    match command {
        Command::Status => {
            // 「没绑定」和「绑了但会话没了」是**两件事**，回文要能分辨：
            // 前者是没配，后者是配置指向了不存在的东西（多半会话被删了）。
            let (bound_label, title) = match bot.conversation_id.as_deref() {
                None => ("未绑定".to_owned(), "未绑定".to_owned()),
                Some(id) => {
                    let found = store.get_convo_title(id).ok().flatten();
                    (
                        id.to_owned(),
                        found.unwrap_or_else(|| "（绑定指向的会话已不存在）".to_owned()),
                    )
                }
            };
            let model = if bot.model.trim().is_empty() {
                "跟主设置".to_owned()
            } else {
                bot.model.clone()
            };
            let alias = if bot.alias.trim().is_empty() {
                bot.bot_id.clone()
            } else {
                bot.alias.clone()
            };
            Ok(CommandOutcome {
                reply: format!(
                    "机器人：{alias}（{channel_name}）\n\
                     绑定会话：{title}（{bound_label}）\n\
                     模型：{model}\n\
                     访问模式：见设置页（白名单 / 私聊 / 开放）\n\
                     数据目录：~/.neobot（只在本机）"
                ),
                switch_convo_to: None,
                stop_requested: false,
            })
        }
        Command::New { title } => {
            // 继承父会话的 kind：dm 的新会话仍是 dm，group 的仍是 group。
            let kind = bot
                .conversation_id
                .as_deref()
                .and_then(|id| store.get_conversation(id).ok().flatten())
                .map(|convo| convo.kind)
                .unwrap_or_else(|| "dm".to_owned());
            let title = if title.trim().is_empty() {
                "新会话".to_owned()
            } else {
                title.clone()
            };
            let convo_id = store.create_conversation(&kind, &title, &[])?;
            let mut updated = bot.clone();
            updated.conversation_id = Some(convo_id.clone());
            store.upsert_bot(&updated)?;
            Ok(CommandOutcome {
                reply: format!("已开新会话「{title}」，后续消息都进这里。"),
                switch_convo_to: Some(convo_id),
                stop_requested: false,
            })
        }
        // 走到这里 = `local_reply`（判「不需要 IM 上下文」）与上面这个 match
        // （提供「需要 IM 上下文」的分支）对**同一条命令的归属判断不一致** ——
        // 即有人加了新动词，忘了在这两边同时登记。fail-closed：如实报错，
        // 绝不编一句假回执出去（那正是本模块反复修的那类缺陷）。
        Command::Help | Command::Stop => Err(NtBotError::Invalid(format!(
            "internal: '/{}' 既不在 local_reply 里, 也不需要 IM 上下文 —— \
             两处命令归属判断不一致",
            command.as_str()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nt_store::BotRow;

    fn store(case: &str) -> NeobotStore {
        NeobotStore::open(":memory:").unwrap_or_else(|err| panic!("{case}: {err}"))
    }

    fn bot() -> BotRow {
        BotRow {
            channel: "tg".to_owned(),
            bot_id: "1".to_owned(),
            alias: String::new(),
            token_env: "NEOBOT_T".to_owned(),
            conversation_id: None,
            model: String::new(),
            allow_list: String::new(),
            created_at: String::new(),
            last_seen: None,
        }
    }

    #[test]
    fn parse_recognises_the_four_commands() {
        assert_eq!(parse("/new"), Some(Command::New { title: String::new() }));
        assert_eq!(
            parse("/new  重构侧边栏 "),
            Some(Command::New { title: "重构侧边栏".to_owned() })
        );
        assert_eq!(parse("/stop"), Some(Command::Stop));
        assert_eq!(parse("/help"), Some(Command::Help));
        assert_eq!(parse("/status"), Some(Command::Status));
        // 中文别名也在。
        assert_eq!(parse("/帮助"), Some(Command::Help));
        assert_eq!(parse("/状态"), Some(Command::Status));
    }

    #[test]
    fn parse_is_case_insensitive_on_the_verb() {
        assert_eq!(parse("/NEW"), Some(Command::New { title: String::new() }));
        assert_eq!(parse("/Stop"), Some(Command::Stop));
    }

    #[test]
    fn parse_returns_none_for_ordinary_text() {
        for text in ["", "hello", "  ", "/", "/unknown", "not/a/command", "a /new"] {
            assert_eq!(parse(text), None, "{text:?} 不该被当指令");
        }
    }

    #[test]
    fn new_title_is_capped_at_80_chars() {
        let long = "x".repeat(500);
        let got = parse(&format!("/new {long}")).expect("parsed");
        match got {
            Command::New { title } => assert_eq!(title.chars().count(), 80),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn new_creates_a_conversation_and_rebinds() {
        let st = store("new");
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let got = execute(&st, &Command::New { title: String::new() }, &b, "Telegram")
            .expect("execute");
        let new_id = got.switch_convo_to.expect("should switch");
        assert!(got.reply.contains("新会话"));
        // 绑定已改：母机器人现在指向新会话。
        let after = st.get_bot("tg", "1").expect("get").expect("some");
        assert_eq!(after.conversation_id.as_deref(), Some(new_id.as_str()));
        // 会话真的建出来了。
        assert!(st.get_conversation(&new_id).expect("get").is_some());
    }

    #[test]
    fn new_inherits_the_parent_kind() {
        let st = store("kind");
        // 先给机器人绑一个 group 会话。
        let group = st.create_conversation("group", "群", &[]).expect("group");
        let mut b = bot();
        b.conversation_id = Some(group);
        st.upsert_bot(&b).expect("bot");
        let got = execute(&st, &Command::New { title: "续".to_owned() }, &b, "TG").expect("exec");
        let new_id = got.switch_convo_to.expect("switch");
        let convo = st.get_conversation(&new_id).expect("get").expect("some");
        assert_eq!(convo.kind, "group", "群里的 /new 仍应是群");
    }

    #[test]
    fn new_uses_a_custom_title() {
        let st = store("title");
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let got = execute(
            &st,
            &Command::New { title: "查日志".to_owned() },
            &b,
            "TG",
        )
        .expect("exec");
        let new_id = got.switch_convo_to.expect("switch");
        assert_eq!(st.get_conversation(&new_id).expect("get").expect("some").title, "查日志");
    }

    #[test]
    fn stop_admits_the_limitation_instead_of_pretending() {
        let st = store("stop");
        let got = execute(&st, &Command::Stop, &bot(), "TG").expect("exec");
        // 标志仍置位（将来并发调度时这条命令已经在）。
        assert!(got.stop_requested);
        assert!(got.switch_convo_to.is_none());
        // **不能**再声称「已请求停止」—— 链路上没有任何停止机制。
        assert!(!got.reply.contains("已请求停止"), "回执仍在撒谎：{}", got.reply);
        // 必须说清为什么停不了。
        assert!(got.reply.contains("停不了"), "{}", got.reply);
        assert!(got.reply.contains("同步"), "要讲清原因：{}", got.reply);
        // **回归锁**：早先的测试断言「桌面 App」必须在回执里，等于把一条
        // **假建议**钉成了契约 —— 而桌面那个键根本不取消任何东西。
        //
        // 禁的是**「让用户去按它」这个动作**，不是「停止键」这几个字：
        // 回执里解释「那个键不终止运行」是有用的，所以按动作短语来判。
        for advice in ["按发送键", "它会变停止键", "请到桌面 App 按"] {
            assert!(
                !got.reply.contains(advice),
                "回执仍在把用户指向一个不取消任何东西的操作（{advice}）：{}",
                got.reply
            );
        }
        // 真能生效的替代路径要说清是「退出 App」。
        assert!(got.reply.contains("退出"), "要给出真正能生效的替代路径：{}", got.reply);
    }

    #[test]
    fn help_does_not_advertise_a_working_stop() {
        let text = help_text("TG");
        // `/stop` 得留着（用户会试），但要标出不可用。
        assert!(text.contains("/stop"));
        assert!(text.contains("当前不可用"), "help 不能把停不了的命令说成能用：{text}");
    }

    #[test]
    fn status_reports_binding_model_and_alias() {
        let st = store("status");
        let convo = st.create_conversation("group", "主力群", &[]).expect("group");
        let mut b = bot();
        b.alias = "小管家".to_owned();
        b.conversation_id = Some(convo);
        b.model = "fast".to_owned();
        st.upsert_bot(&b).expect("bot");
        let got = execute(&st, &Command::Status, &b, "Telegram").expect("exec");
        assert!(got.reply.contains("小管家"), "{}", got.reply);
        assert!(got.reply.contains("主力群"), "{}", got.reply);
        assert!(got.reply.contains("fast"), "{}", got.reply);
        assert!(!got.stop_requested);
    }

    #[test]
    fn status_handles_unbound_bot_without_panicking() {
        let st = store("unbound");
        let got = execute(&st, &Command::Status, &bot(), "TG").expect("exec");
        assert!(got.reply.contains("未绑定"), "{}", got.reply);
    }

    #[test]
    fn status_distinguishes_dangling_binding_from_no_binding() {
        let st = store("dangling");
        let mut b = bot();
        b.conversation_id = Some("convo-that-was-deleted".to_owned());
        let got = execute(&st, &Command::Status, &b, "TG").expect("exec");
        // 绑了但会话没了 ≠ 没绑。含糊其辞会让用户以为配置没生效。
        assert!(got.reply.contains("已不存在"), "{}", got.reply);
        assert!(!got.reply.contains("\n绑定会话：未绑定"), "{}", got.reply);
    }

    #[test]
    fn help_lists_every_command() {
        let text = help_text("Telegram");
        for verb in ["/new", "/stop", "/help", "/status"] {
            assert!(text.contains(verb), "help 漏了 {verb}：{text}");
        }
        assert!(text.contains("Telegram"));
        assert!(text.contains("~/.neobot"), "要讲清数据只在本机");
    }

    // ─── `local_reply`：渠道无关入口 ───

    /// **`local_reply` 与 `execute` 逐字节同文**（含标志位）。
    ///
    /// 这是「回执文本只存在一处」的**证据**：桌面/CLI 走 `local_reply`，IM 走
    /// `execute`，而 `execute` 委托给 `local_reply`。两条路若各自抄一份文案，
    /// 改了一处这条就会红；现在它们**结构上**同源，故这条恒成立 —— 它测的
    /// 是「委托这条接法没被后人拆开」，不是「两份文案碰巧一样」。
    #[test]
    fn local_reply_is_byte_identical_to_execute_on_the_shared_branches() {
        let st = store("local-vs-execute");
        for command in [Command::Help, Command::Stop] {
            let local = local_reply(&command, "Telegram")
                .unwrap_or_else(|| panic!("/{command:?} 应属本地回执范围"));
            let im = execute(&st, &command, &bot(), "Telegram")
                .unwrap_or_else(|err| panic!("/{command:?} IM 侧应能执行：{err}"));
            assert_eq!(
                local, im,
                "/{command:?} 的两端口径漂移了（local_reply 与 execute 不是同一份文本）"
            );
        }
    }

    /// `local_reply` 的返回值**随渠道名变**——证明它真在生成文案，
    /// 不是回一句常量。桌面的 `DESKTOP_CHANNEL_NAME` 就靠这一点自报家门。
    #[test]
    fn local_reply_still_uses_the_channel_name_it_is_given() {
        let one = local_reply(&Command::Help, "Telegram").expect("help");
        let two = local_reply(&Command::Help, "NeoTrix 桌面").expect("help");
        assert!(one.reply.contains("Telegram"), "{}", one.reply);
        assert!(two.reply.contains("NeoTrix 桌面"), "{}", two.reply);
        assert_ne!(one.reply, two.reply, "渠道名不同，文案不该完全一样");
    }

    /// **需要渠道上下文的命令必须回 `None`**。
    ///
    /// `None` 的契约是「别假装能本地答」。`/status` 要读 bot 绑定与
    /// 会话标题、`/new` 要建会话并改绑定，两者在桌面都**没有**对应的 IM
    /// 上下文。若日后有人把这类分支塞进 `local_reply`，桌面就会拿到一句
    /// 拿 IM 上下文编出来的假话（`BotRow` 桌面根本没有）—— 这条锁死它。
    #[test]
    fn local_reply_declines_commands_that_need_channel_context() {
        for command in [
            Command::Status,
            Command::New { title: String::new() },
            Command::New { title: "重构侧边栏".to_owned() },
        ] {
            assert!(
                local_reply(&command, "Telegram").is_none(),
                "/{command:?} 需要 store/BotRow 上下文，不该被 local_reply 答掉 —— \
                 答了就等于替调用方编造 IM 上下文"
            );
        }
    }

    /// 两个投递端**都不渲染 markdown**（IM 纯文本直发、桌面 `esc()` 后按字面
    /// 显示），所以 `help_text` 里不许有 markdown 记号。
    ///
    /// 这条是**反向断言**（盯坏记号不在）：早先 `（**当前不可用**：…）` 里的
    /// `**` 在 IM 里就是两个实打实的星号，在桌面里 `esc()` 之后也是两个字面
    /// 星号 —— 两头都变成噪音。正确做法是**改文案**，不是给桌面加渲染器。
    #[test]
    fn help_text_carries_no_markdown_emphasis() {
        let text = help_text("Telegram");
        for mark in ["**", "__", "*当前不可用*"] {
            assert!(
                !text.contains(mark),
                "help 带了 markdown 记号「{mark}」，而两端都不渲染 markdown：{text}"
            );
        }
        // 去掉了记号，但**结论必须还在**（IM 是纯文本，这句直接被用户看到）。
        assert!(
            text.contains("（当前不可用：跑轮同步"),
            "去星号时别把「当前不可用」这个结论一起去掉：{text}"
        );
    }
}

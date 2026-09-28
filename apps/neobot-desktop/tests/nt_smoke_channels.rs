//! 冒烟：IM 渠道（`nt_cmd_channels` 的 channels / channel_bots / parse_allow）。
//!
//! ## 覆盖的缺陷类别
//!
//! 1. **凭据脱敏**：`neobot_channel_bots` 只能出 token 的**环境变量名**。
//!    出值 = 把机器人凭据交给前端 = 一旦前端有 XSS 就等于交出凭据。
//!    这条断言一旦红了，性质是安全事故而不是 bug。
//! 2. **访问模式归一**：`access_mode` 必须落进三个合法档之一，
//!    非法串不能被原样存进库（否则真正派发时的白名单判定会走到别的分支）。
//! 3. **可用性标记**：编译进来的渠道恒 `available = true`；
//!    编不进来的也要列出来（`available = false`），否则用户无处可点。
//!
//! ## 网络
//!
//! 全程不碰网络。`neobot_channel_probe` 之类的**故意不测** —— 它必然要真连公网，
//! 与「测试不许需要网络」的硬约束冲突。见 `nt_smoke.sh` 头注释的「不覆盖」清单。

mod common;

use neobot_desktop::nt_commands::nt_cmd_channels::{
    neobot_channel_bot_alias, neobot_channel_bot_upsert, neobot_channel_bots,
    neobot_channel_catalog, neobot_channel_parse_allow, neobot_channel_upsert, neobot_channels,
};

/// 三档合法访问模式（`nt_channel::AccessMode` 的口径）。
const MODES: [&str; 3] = ["open", "allow", "dm_only"];

#[test]
fn channel_parse_allow_splits_and_drops_junk() {
    let _ = common::data_dir();
    // 前端要「已解析的数组」直接渲染，白名单字符串解析口径必须与核心一致。
    assert_eq!(neobot_channel_parse_allow("alice,bob".to_owned()), vec!["alice", "bob"]);
    assert_eq!(
        neobot_channel_parse_allow(" alice , bob , ".to_owned()),
        vec!["alice", "bob"],
        "两侧空白应被 trim 掉，否则白名单永远匹配不上"
    );
    assert_eq!(neobot_channel_parse_allow(" , , ".to_owned()), Vec::<String>::new());
    assert!(neobot_channel_parse_allow(String::new()).is_empty(), "空串应回空数组而非空串");
}

#[test]
fn channels_lists_compiled_in_adapters_even_before_any_config_exists() {
    // 冷启动的库是空的。若这里返回空，用户在设置页一个渠道都点不到 ——
    // 「无处可点」和「没配过」在前端是同一种空屏，诊断成本极高。
    let _ = common::data_dir();
    let catalog = neobot_channel_catalog();
    assert!(!catalog.is_empty(), "编译进来的渠道表不能空");

    let rows = neobot_channels().expect("列渠道应成功");
    for id in &catalog {
        let row = rows
            .iter()
            .find(|r| &r.id == id)
            .unwrap_or_else(|| panic!("编进来的渠道 {id} 没出现在列表里：{:?}", rows.iter().map(|r| &r.id).collect::<Vec<_>>()));
        assert!(row.available, "编进来的渠道 {id} 必须 available=true");
        assert!(MODES.contains(&row.access_mode.as_str()), "{id} 访问模式非法：{}", row.access_mode);
    }
}

#[test]
fn channel_upsert_normalises_access_mode_before_persisting() {
    // 归一必须发生在**写之前**：库里只该有三个合法档。
    let _ = common::data_dir();
    let id = common::uniq("chan");

    neobot_channel_upsert(
        id.clone(),
        "冒烟渠道".to_owned(),
        "dm_only".to_owned(),
        Some(9),
    )
    .expect("upsert 应成功");

    let row = neobot_channels()
        .expect("列渠道应成功")
        .into_iter()
        .find(|r| r.id == id)
        .expect("刚建的渠道必须查得到");
    assert_eq!(row.access_mode, "dm_only", "合法档应原样落库");
    assert_eq!(row.poll_secs, 9, "轮询节拍应原样落库");
    assert_eq!(row.title, "冒烟渠道");
}

#[test]
fn channel_upsert_fails_closed_on_an_unrecognised_access_mode() {
    // 规则（`nt_channel::AccessMode::parse`）：**未知即 Allow（空白名单 = 全拒）**，
    // 且 `dm` 是 `dm_only` 的别名。
    //
    // 这里刻意断言「不报错、但收敛到 allow」而不是「报错」——
    // 写错一个字符就变成「谁都不理」（用户会来问的怪事），
    // 而不是「对所有人开放」（安全事故）。这条测试把这个取舍钉住：
    // 谁哪天想「顺手改成报错」或「猜成 open」，这里会红。
    let _ = common::data_dir();
    for (raw, want) in [
        ("everyone-in-the-world", "allow"),
        ("ALLOW", "allow"),  // 大小写不敏感：未识别 → fail-closed
        ("open ", "open"),    // 带尾空白仍应识别
        ("dm", "dm_only"),    // 简写别名
        ("dm-only", "allow"), // 拼错的 dm_only → fail-closed，不是开放
    ] {
        let id = common::uniq("chan");
        neobot_channel_upsert(id.clone(), "渠道".to_owned(), raw.to_owned(), Some(5))
            .unwrap_or_else(|err| panic!("upsert({raw:?}) 不该报错，实际：{err}"));
        let row = neobot_channels()
            .expect("列渠道应成功")
            .into_iter()
            .find(|r| r.id == id)
            .expect("刚建的渠道必须查得到");
        assert_eq!(row.access_mode, want, "输入 {raw:?} 的归一口径不对");
        assert!(
            MODES.contains(&row.access_mode.as_str()),
            "库里只该有三个合法档，实际落库了 {:?}",
            row.access_mode
        );
    }
}

#[test]
fn channel_bots_returns_the_token_env_name_and_never_the_value() {
    // 脱敏律（安全性质，不是美观问题）：出值等于把凭据交给前端。
    let _ = common::data_dir();
    let channel = common::uniq("chan");
    let bot_id = common::uniq("bot");
    neobot_channel_upsert(channel.clone(), "渠道".to_owned(), "allow".to_owned(), Some(5))
        .expect("建渠道应成功");
    neobot_channel_bot_upsert(
        channel.clone(),
        bot_id.clone(),
        "TELEGRAM_BOT_TOKEN".to_owned(),
        Some("gpt-4o-mini".to_owned()),
        Some("alice,bob".to_owned()),
        None,
    )
    .expect("挂机器人应成功");

    let bots = neobot_channel_bots(channel.clone()).expect("列机器人应成功");
    let bot = bots
        .iter()
        .find(|b| b.bot_id == bot_id)
        .unwrap_or_else(|| panic!("刚挂的机器人应列出：{:?}", bots.iter().map(|b| &b.bot_id).collect::<Vec<_>>()));

    assert_eq!(bot.token_env, "TELEGRAM_BOT_TOKEN", "只出环境变量名");
    assert_eq!(bot.channel, channel, "channel 必须原样透传（不能被壳写成别的）");
    assert_eq!(bot.bot_id, bot_id);
    assert_eq!(bot.model, "gpt-4o-mini", "模型覆盖值应带出");
    assert_eq!(bot.allow_list, vec!["alice", "bob"], "白名单应已解析成数组");
    assert_eq!(bot.access_mode, "allow", "机器人行应带上渠道的访问模式");
}

#[test]
fn channel_bots_is_empty_for_an_unknown_channel() {
    // 未知渠道不能报错 —— 前端切页签时会先探一次。
    let _ = common::data_dir();
    let bots = neobot_channel_bots(common::uniq("ghost")).expect("未知渠道不该报错");
    assert!(bots.is_empty(), "未知渠道应回空列表");
}

#[test]
fn channel_bot_upsert_preserves_alias_across_an_update() {
    // upsert 的签名里刻意**没有** alias（别名有自己的命令）。
    // 若更新绑定时把别名抹成空串，界面上机器人名会突然变回平台原名 ——
    // 典型的「没报错、只是悄悄变了」缺陷。
    let _ = common::data_dir();
    let channel = common::uniq("chan");
    let bot_id = common::uniq("bot");
    neobot_channel_upsert(channel.clone(), "渠道".to_owned(), "allow".to_owned(), Some(5))
        .expect("建渠道应成功");
    neobot_channel_bot_upsert(
        channel.clone(),
        bot_id.clone(),
        "TOK".to_owned(),
        None,
        None,
        None,
    )
    .expect("首次挂载应成功");
    neobot_channel_bot_alias(channel.clone(), bot_id.clone(), "值班机器人".to_owned())
        .expect("起别名应成功");

    neobot_channel_bot_upsert(
        channel.clone(),
        bot_id.clone(),
        "TOK2".to_owned(), // 换 token 变量名
        Some("new-model".to_owned()),
        None,
        None,
    )
    .expect("改绑定应成功");

    let bots = neobot_channel_bots(channel.clone()).expect("列机器人应成功");
    let bot = bots.iter().find(|b| b.bot_id == bot_id).expect("机器人应还在");
    assert_eq!(bot.alias, "值班机器人", "upsert 不该抹掉别名");
    assert_eq!(bot.token_env, "TOK2", "token 变量名应被更新");
    assert_eq!(bot.model, "new-model", "模型覆盖应被更新");
    assert_eq!(bots.len(), 1, "upsert 不该插出第二条（那是主键冲突被吞了）");
}

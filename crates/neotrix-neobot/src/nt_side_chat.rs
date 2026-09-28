//! `nt_side_chat` — 侧边栏的**侧聊**（better-sidebar「侧边对话」的本机实现）。
//!
//! 侧聊 = 一个**继承母会话上下文、但独立运行、且不污染母会话**的线程。
//! 每个侧聊在侧边栏里占一个 tab，可持续追问、可冷恢复、可一键升为顶层会话。
//!
//! **落库形状**：侧聊不是新表，就是一条 `conversations` 行，额外两个字段：
//! `parent_id`（母会话）与 `origin`（`chat` 顶层 / `sidebar` 侧聊）。
//! 复用而非新建的理由：会话的成员、已读水位、任务归属、附件、outbox
//! 那一整套已经能用（IM 语义本来就是「会话是容器」），另起一张表就得
//! 把这些重做一遍，而且两套容器迟早漂移。
//!
//! **上下文继承的诚实边界**：better-sidebar 能把母会话的**完整消息列表**
//! 复制进侧线程，因为它的消息在宿主内存里。neobot 的消息正文在**前端**
//! （`localStorage`），Rust 侧只有 `tasks`/`steps` 这层执行痕迹。所以本模块
//! 传的是**母会话最近若干轮的可读摘要**（标题 + 终态 + 末次回复节选），
//! 而不是逐字历史。这一点**如实标注在摘要抬头**上，让模型知道它拿到的是
//! 摘要而非原件 —— 含糊其辞会让模型误以为自己看过全部对话。
//!
//! **不随母会话陪葬**：母被删时侧聊升为顶层（见 `delete_conversation`）。
//! 侧聊有自己的历史与任务，静默删掉等于替用户销毁他可能想要的东西。

use crate::nt_config::NeobotConfig;
use crate::nt_error::NtBotError;
use crate::nt_store::{Conversation, NeobotStore};

/// 侧聊标题上限（与 `create_conversation` 同律：1..=80 字符）。
pub const MAX_TITLE_CHARS: usize = 80;
/// 继承摘要里最多带几轮母会话历史。
pub const INHERIT_TURNS: usize = 6;
/// 单轮回复节选上限（字符）。
pub const SNIPPET_CHARS: usize = 400;
/// 继承摘要的总长上限（字符；超了从**最早**的一轮开始砍）。
pub const CONTEXT_CAP_CHARS: usize = 4000;

/// 缺省侧聊标题。
pub fn default_title() -> String {
    "侧聊".to_owned()
}

/// 开一个侧聊（母必须存在）。
pub fn open(store: &NeobotStore, parent_id: &str, title: &str) -> Result<String, NtBotError> {
    let mut title = title.trim().to_owned();
    if title.is_empty() {
        title = default_title();
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(NtBotError::Invalid(format!(
            "side thread title exceeds {MAX_TITLE_CHARS} chars"
        )));
    }
    store.insert_side_thread(parent_id, &title)
}

/// 列某母会话下的侧聊（按最后活跃倒序）。
pub fn list(store: &NeobotStore, parent_id: &str) -> Result<Vec<Conversation>, NtBotError> {
    store.list_side_threads(parent_id)
}

/// 侧聊升为顶层会话（「保存为新会话」；幂等）。
pub fn promote(store: &NeobotStore, id: &str) -> Result<(), NtBotError> {
    store.promote_side_thread(id)
}

/// 母会话的上下文摘要（喂给侧聊的第一轮）。
///
/// 返回 `None` 表示母会话**还没有任何任务**——此时不该塞一段空壳摘要，
/// 直接让侧聊从零开始，模型也就不会误以为「前面聊过」。
pub fn inherit_context(
    store: &NeobotStore,
    parent_id: &str,
) -> Result<Option<String>, NtBotError> {
    let mut tasks = store.list_convo_tasks(parent_id, i64::try_from(INHERIT_TURNS).unwrap_or(6))?;
    if tasks.is_empty() {
        return Ok(None);
    }
    // 老的在前、新的在后（`list_convo_tasks` 是倒序）。
    tasks.reverse();
    let mut body = String::new();
    for task in &tasks {
        // 末次回复：从 steps 里取该任务最后一条 `reply`。
        let reply = last_reply(store, &task.id)?;
        let line = format!(
            "- [{}] {}{}\n",
            task.status.as_str(),
            task.title,
            reply.map(|text| format!("\n  助手：{text}"))
                .unwrap_or_default()
        );
        // 从最早的一轮开始砍（末尾是最有价值的近事）。
        if body.len().saturating_add(line.len()) > CONTEXT_CAP_CHARS && !body.is_empty() {
            break;
        }
        body.push_str(&line);
    }
    if body.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "以下是母会话「{}」最近 {} 轮的执行摘要（**摘要，非逐字历史**；\
         母会话的完整聊天记录在前端，本线程看不到）：\n{body}\n\
         以上只作背景，回答时不必复述它。",
        parent_id,
        tasks.len()
    )))
}

/// 该任务最后一次 `reply` 步骤的文本（节选；没有就 `None`）。
fn last_reply(store: &NeobotStore, task_id: &str) -> Result<Option<String>, NtBotError> {
    let found = store.last_step_output(task_id, "reply")?;
    Ok(found.map(|text| snippet(&text, SNIPPET_CHARS)))
}

/// 截断到字符边界（不切碎多字节字符；超长补省略号）。
pub fn snippet(text: &str, max_chars: usize) -> String {
    let flat = text.trim();
    if flat.chars().count() <= max_chars {
        return flat.to_owned();
    }
    let cut: String = flat.chars().take(max_chars).collect();
    format!("{cut}…")
}

/// 侧聊的第一次提问 = 继承摘要 + 用户原文。
///
/// 摘要为空（母没历史）时**不加分隔噪声**，原样透传。
pub fn first_prompt(context: Option<String>, user_text: &str) -> String {
    match context {
        Some(ctx) if !ctx.trim().is_empty() => format!("{ctx}\n\n用户：{user_text}"),
        _ => user_text.to_owned(),
    }
}

/// 侧聊的工作区（当前一律沿用母会话的配置工作区）。
///
/// 留成函数而非到处直接读 `config.workspace_dir`：better-sidebar 那边每个
/// 机器人/每个对话都能绑自己的工作区；neobot 侧聊是「当前会话的分支」，
/// 跟随主配置是对的，但这个决定要有一处显式落地，别散成裸字段访问。
pub fn workspace_of(config: &NeobotConfig) -> &std::path::Path {
    &config.workspace_dir
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(case: &str) -> NeobotStore {
        NeobotStore::open(":memory:").unwrap_or_else(|err| panic!("{case}: {err}"))
    }

    fn parent(store: &NeobotStore) -> String {
        store
            .create_conversation("group", "母会话", &[])
            .expect("create parent")
    }

    fn seed_task(store: &NeobotStore, convo: &str, title: &str, status: &str, reply: &str) {
        use crate::nt_types::{AgentTask, TaskStatus};
        let now = chrono::Utc::now().to_rfc3339();
        let task = AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_owned(),
            status: match status {
                "done" => TaskStatus::Done,
                "failed" => TaskStatus::Failed,
                _ => TaskStatus::Pending,
            },
            created_at: now.clone(),
            updated_at: now,
            claimed_by: None,
            claimed_at: None,
            visibility: "team".to_owned(),
            conversation_id: Some(convo.to_owned()),
            lease_id: None,
            lease_until: None,
            attempts: 1,
            error: None,
        };
        store.save_task(&task).expect("save task");
        store
            .add_step(&task.id, 0, "reply", true, reply)
            .expect("add step");
    }

    #[test]
    fn opening_a_thread_links_it_to_the_parent() {
        let st = store("open");
        let parent = parent(&st);
        let thread = open(&st, &parent, "看看配置").expect("open thread");
        let threads = list(&st, &parent).expect("list");
        assert_eq!(threads.len(), 1);
        assert_eq!(threads.first().map(|t| t.id.clone()), Some(thread.clone()));
        assert_eq!(threads.first().and_then(|t| t.parent_id.clone()), Some(parent));
        assert_eq!(threads.first().map(|t| t.origin.as_str()), Some("sidebar"));
    }

    #[test]
    fn side_threads_stay_out_of_the_main_list() {
        let st = store("hidden");
        let parent = parent(&st);
        open(&st, &parent, "侧聊 A").expect("open");
        // 主列表只有母会话，侧聊不混进去。
        let main_list = st.list_conversations().expect("list");
        assert_eq!(main_list.len(), 1);
        assert_eq!(main_list.first().map(|c| c.id.clone()), Some(parent.clone()));
        // 但「全部会话」看得到（导入/恢复要用）。
        assert_eq!(st.list_all_conversations().expect("all").len(), 2);
    }

    #[test]
    fn orphan_parent_promotes_threads_instead_of_killing_them() {
        let st = store("orphan");
        let parent = parent(&st);
        let thread = open(&st, &parent, "别删我").expect("open");
        st.delete_conversation(&parent).expect("delete parent");
        // 侧聊没被陪葬，而是升为顶层。
        let promoted = st
            .list_conversations()
            .expect("list")
            .into_iter()
            .find(|c| c.id == thread)
            .expect("thread survived as top-level");
        assert_eq!(promoted.origin, "chat");
        assert!(promoted.parent_id.is_none());
    }

    #[test]
    fn promoting_a_thread_is_idempotent() {
        let st = store("promote");
        let parent = parent(&st);
        let thread = open(&st, &parent, "升格").expect("open");
        promote(&st, &thread).expect("promote");
        // 连点两下不该报错。
        promote(&st, &thread).expect("promote again");
        let listed = st.list_conversations().expect("list");
        assert!(listed.iter().any(|c| c.id == thread));
        // 但不存在的会话仍要报错（别把 typo 静默吞掉）。
        assert!(promote(&st, "nope").is_err());
    }

    #[test]
    fn opening_under_a_missing_parent_fails() {
        let st = store("noparent");
        let err = open(&st, "ghost", "孤儿侧聊").expect_err("missing parent");
        assert!(err.to_string().contains("no such parent"), "{err}");
    }

    #[test]
    fn empty_parent_yields_no_context_block() {
        let st = store("noctx");
        let parent = parent(&st);
        // 母没历史 → 不塞空壳摘要（否则模型会以为「前面聊过」）。
        assert!(inherit_context(&st, &parent).expect("ctx").is_none());
        // 首问原样透传。
        assert_eq!(first_prompt(None, "你好"), "你好");
    }

    #[test]
    fn context_block_summarises_parent_turns_newest_last() {
        let st = store("ctx");
        let parent = parent(&st);
        seed_task(&st, &parent, "第一件事", "done", "第一条答案");
        seed_task(&st, &parent, "第二件事", "done", "第二条答案");
        let ctx = inherit_context(&st, &parent)
            .expect("ctx")
            .expect("some context");
        // 抬头明说这是摘要而非逐字历史。
        assert!(ctx.contains("摘要，非逐字历史"), "{ctx}");
        assert!(ctx.contains("第一条答案"));
        assert!(ctx.contains("第二条答案"));
        // 时间序：早的在前、晚的在后。
        let first = ctx.find("第一件事").expect("first");
        let second = ctx.find("第二件事").expect("second");
        assert!(first < second, "近事应在后：\n{ctx}");
    }

    #[test]
    fn context_block_respects_the_turn_cap() {
        let st = store("cap");
        let parent = parent(&st);
        for i in 0..(INHERIT_TURNS + 4) {
            seed_task(&st, &parent, &format!("任务{i}"), "done", "答案");
        }
        let ctx = inherit_context(&st, &parent).expect("ctx").expect("ctx");
        // 只带最近 INHERIT_TURNS 轮。
        let count = ctx.matches("[done]").count();
        assert!(count <= INHERIT_TURNS, "带多了：{count} > {INHERIT_TURNS}");
    }

    #[test]
    fn first_prompt_prepends_context_when_present() {
        let with = first_prompt(Some("背景材料".to_owned()), "继续");
        assert!(with.starts_with("背景材料"));
        assert!(with.ends_with("继续"));
        // 空白摘要视同无摘要。
        assert_eq!(first_prompt(Some("   ".to_owned()), "继续"), "继续");
    }

    #[test]
    fn snippet_cuts_on_char_boundaries() {
        assert_eq!(snippet("  abc  ", 10), "abc");
        let long = "中文字符串很长很长";
        let cut = snippet(&long, 4);
        assert_eq!(cut, "中文字符…");
        // 不切碎多字节字符。
        assert!(cut.chars().count() <= 5);
    }

    #[test]
    fn title_defaults_and_length_is_capped() {
        let st = store("title");
        let parent = parent(&st);
        let thread = open(&st, &parent, "   ").expect("blank title ok");
        let listed = list(&st, &parent).expect("list");
        assert_eq!(listed.first().map(|t| t.title.as_str()), Some("侧聊"));
        // 超长标题拒。
        let long = "x".repeat(MAX_TITLE_CHARS + 1);
        assert!(open(&st, &parent, &long).is_err());
        assert!(!thread.is_empty());
    }
}

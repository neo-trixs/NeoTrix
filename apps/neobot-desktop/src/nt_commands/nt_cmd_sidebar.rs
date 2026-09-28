//! `nt_cmd_sidebar` — 侧边栏页签 / 查看器注册表 + Git 视角 + 侧聊的 IPC 面.
//!
//! 三块拼在一起，因为它们共用一个前提：**界面状态由界面说了算**。
//! 页签与查看器在 Rust 侧注册（`nt_sidebar`）以免前端各处 `if (ext === ...)`
//! 猜；Git 视角是「仓库现在欠什么」；侧聊是「继承母会话但独立跑」的线程。
//!
//! Git hook 诚实声明：`neobot_git_commit` 的 `no_verify` 默认**真** ——
//! 即默认执行仓库里的 `.git/hooks/*`。那是 git 的正常语义（用户按下提交就
//! 预期自己的 hook 跑起来），但它意味着「点提交会跑仓库作者放在磁盘上的
//! 任意代码」。故选项在 IPC 层**显式暴露**给前端（UI 上就是一个勾选框），
//! 不做静默 —— 悄悄跳过和悄悄执行一样是骗人。

use neotrix_neobot::{nt_git, nt_side_chat, nt_sidebar, TabRegistry};

use super::{load_config, open_store};

/// 页签表（前端按 `order` 排序渲染；`needs` 决定要不要拉数据）。
#[tauri::command]
pub fn neobot_sidebar_tabs() -> Result<Vec<nt_sidebar::SidebarTab>, String> {
    Ok(TabRegistry::with_builtins().tabs().to_vec())
}

/// 查看器表（前端据此选渲染器与要不要给保存按钮）。
#[tauri::command]
pub fn neobot_sidebar_viewers() -> Result<Vec<nt_sidebar::ViewerSpec>, String> {
    Ok(nt_sidebar::builtin_viewers())
}

/// 该路径开哪个查看器（决策只在 Rust 一处；前端不再各处猜扩展名）。
#[tauri::command]
pub fn neobot_sidebar_viewer_for(rel: String) -> Result<Option<nt_sidebar::ViewerSpec>, String> {
    Ok(nt_sidebar::viewer_for(&nt_sidebar::builtin_viewers(), &rel).cloned())
}

/// 解析一次「打开」请求（topic 必须在册、target 必须过 jail）。
///
/// 前端与模型的 `sidebar_open` 都走这一条，故两边的校验口径永远一致 ——
/// 不存在「模型能开、界面开不了」的分裂。
#[tauri::command]
pub fn neobot_sidebar_resolve(topic: String, target: String) -> Result<nt_sidebar::OpenTarget, String> {
    nt_sidebar::resolve_open(
        &TabRegistry::with_builtins(),
        &nt_sidebar::builtin_viewers(),
        &topic,
        &target,
    )
    .map_err(|err| err.to_string())
}

// ---- Git 视角 ----

/// `git` 在不在 PATH 上（UI 据此决定要不要显示 Git 视角）。
#[tauri::command]
pub fn neobot_git_available() -> bool {
    nt_git::available()
}

/// 该目录是不是仓库根。
#[tauri::command]
pub fn neobot_git_is_repo(rel: String) -> Result<bool, String> {
    let config = load_config()?;
    let dir = neotrix_neobot::nt_workspace::jail_join(&config.workspace_dir, &rel)
        .map_err(|err| err.to_string())?;
    Ok(nt_git::is_repo(&dir))
}

/// 改动清单（未跟踪在前）。
#[tauri::command]
pub fn neobot_git_status(rel: String) -> Result<Vec<nt_git::GitFile>, String> {
    let config = load_config()?;
    nt_git::status(&config.workspace_dir, &rel).map_err(|err| err.to_string())
}

/// 单文件 diff（`staged` 真则看索引；未跟踪文件回全文并标 `untracked_new`）。
#[tauri::command]
pub fn neobot_git_diff(rel: String, path: String, staged: bool) -> Result<nt_git::GitDiff, String> {
    let config = load_config()?;
    nt_git::diff(&config.workspace_dir, &rel, &path, staged).map_err(|err| err.to_string())
}

/// 提交历史（`path` 空 = 全仓；`limit` 收敛到 1..=200）。
#[tauri::command]
pub fn neobot_git_log(rel: String, path: String, limit: Option<i64>) -> Result<Vec<nt_git::GitCommit>, String> {
    let config = load_config()?;
    nt_git::log(&config.workspace_dir, &rel, &path, limit.unwrap_or(30))
        .map_err(|err| err.to_string())
}

/// 暂存文件。
#[tauri::command]
pub fn neobot_git_stage(rel: String, path: String) -> Result<(), String> {
    let config = load_config()?;
    nt_git::stage(&config.workspace_dir, &rel, &path).map_err(|err| err.to_string())
}

/// 取消暂存。
#[tauri::command]
pub fn neobot_git_unstage(rel: String, path: String) -> Result<(), String> {
    let config = load_config()?;
    nt_git::unstage(&config.workspace_dir, &rel, &path).map_err(|err| err.to_string())
}

/// **丢弃**工作区改动（`git checkout --`；不可逆，UI 须二次确认）。
///
/// 未跟踪文件在此**拒**：用户想要的大概是「删掉」，那是另一条命令、
/// 另一道确认，不该由「还原」顺手做掉。
#[tauri::command]
pub fn neobot_git_revert(rel: String, path: String) -> Result<(), String> {
    let config = load_config()?;
    nt_git::discard_workspace_changes(&config.workspace_dir, &rel, &path)
        .map_err(|err| err.to_string())
}

/// 提交。`no_verify` 默认真 = **会**跑仓库 hook（git 正常语义，见模块头声明）。
#[tauri::command]
pub fn neobot_git_commit(rel: String, message: String, no_verify: Option<bool>) -> Result<nt_git::GitCommitResult, String> {
    let config = load_config()?;
    nt_git::commit(
        &config.workspace_dir,
        &rel,
        &message,
        no_verify.unwrap_or(true),
    )
    .map_err(|err| err.to_string())
}

// ---- 侧聊 ----

/// 母会话下的侧聊列表。
#[tauri::command]
pub fn neobot_sidechat_list(parent_id: String) -> Result<Vec<NeobotSideThreadItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let rows = nt_side_chat::list(&store, &parent_id).map_err(|err| err.to_string())?;
    Ok(rows.into_iter().map(NeobotSideThreadItem::from).collect())
}

/// 开一个侧聊（母必须存在；标题空则用缺省「侧聊」）。
#[tauri::command]
pub fn neobot_sidechat_open(parent_id: String, title: String) -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    nt_side_chat::open(&store, &parent_id, &title).map_err(|err| err.to_string())
}

/// 母会话的上下文摘要（`None` = 母还没历史，此时不该塞空壳摘要）。
///
/// 摘要抬头明写「摘要，非逐字历史」—— 模型据此知道它拿到的是概括而非原件。
#[tauri::command]
pub fn neobot_sidechat_context(parent_id: String) -> Result<Option<String>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    nt_side_chat::inherit_context(&store, &parent_id).map_err(|err| err.to_string())
}

/// 侧聊升为顶层会话（「保存为新会话」；幂等）。
#[tauri::command]
pub fn neobot_sidechat_promote(thread_id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    nt_side_chat::promote(&store, &thread_id).map_err(|err| err.to_string())
}

/// 删侧聊（用会话删除的同一原子方法，不另开删除路径）。
#[tauri::command]
pub fn neobot_sidechat_delete(thread_id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.delete_conversation(&thread_id).map_err(|err| err.to_string())
}

/// 侧聊 DTO。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotSideThreadItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub created_at: String,
    pub last_active: String,
    pub task_count: i64,
    pub unread: i64,
    pub parent_id: Option<String>,
    pub origin: String,
}

impl From<neotrix_neobot::Conversation> for NeobotSideThreadItem {
    fn from(convo: neotrix_neobot::Conversation) -> Self {
        Self {
            id: convo.id,
            kind: convo.kind,
            title: convo.title,
            created_at: convo.created_at,
            last_active: convo.last_active,
            task_count: convo.task_count,
            unread: convo.unread,
            parent_id: convo.parent_id,
            origin: convo.origin,
        }
    }
}

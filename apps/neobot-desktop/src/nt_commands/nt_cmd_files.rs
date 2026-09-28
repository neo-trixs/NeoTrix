//! `nt_cmd_files` — 侧边栏文件工作台的 IPC 面（列目录 / 读 / 存 / 搜索 / 增删改名）.
//!
//! **为什么不走 `tauri-plugin-fs`**：那个插件的能力由
//! `capabilities/default.json` 的 ACL 决定，而本 App 的 ACL 刻意只放行
//! `$HOME/.neobot/**`（README 称之为 shield 验收点）。放宽 ACL 去够一个
//! 项目目录，等于把整个家目录的读写权交给前端 —— 而侧边栏要的只是
//! **工作区目录**。故走自有命令：路径在 Rust 侧经 `nt_workspace` 的两道
//! jail（词法段判定 + `canonicalize` 真实路径核验），比 ACL 更严也更好测，
//! 且完全不动那份验收点。
//!
//! 脱敏律：读文件走 `read_text` 时超限 / 二进制都**如实回报标志位**，
//! 不截半份、不编内容（`truncated` / `binary` 由前端分支处理）。
//! 搜索的三个上限同理：`neobot_fs_find` 回 `SearchResults`（带 `truncated`），
//! 不让前端靠数组长度去猜有没有被砍过。

use neotrix_neobot::{nt_workspace, FileText};

use super::{load_config, open_store};

/// 列一层目录。
#[tauri::command]
pub fn neobot_fs_list(rel: String) -> Result<nt_workspace::DirListing, String> {
    let config = load_config()?;
    nt_workspace::list_dir(&config.workspace_dir, &rel).map_err(|err| err.to_string())
}

/// 读文本（超限 / 二进制如实标标志，不截半份）。
#[tauri::command]
pub fn neobot_fs_read(rel: String) -> Result<FileText, String> {
    let config = load_config()?;
    nt_workspace::read_text(&config.workspace_dir, &rel).map_err(|err| err.to_string())
}

/// 存文本（受写预算约束；自动补建父目录）。
#[tauri::command]
pub fn neobot_fs_write(rel: String, content: String) -> Result<nt_workspace::FileWrite, String> {
    let config = load_config()?;
    nt_workspace::write_text(&config.workspace_dir, &rel, &content, config.write_budget)
        .map_err(|err| err.to_string())
}

/// 全局文件名搜索（有界：命中数 / 访问数 / 深度三重上限）。
///
/// 截断**如实回报**（与 `neobot_fs_list` / `neobot_fs_read` 同一口径）：返回
/// `SearchResults` 而非裸 `Vec<SearchHit>`，带 `truncated` / `truncated_by`。
/// 裸数组的话前端只能靠 `hits.length >= 200` 去**猜**是否砍过 —— 那既会把
/// 「恰好 200 个匹配」误报成被截断，又会漏掉访问数/深度撞顶的两种截断。
#[tauri::command]
pub fn neobot_fs_find(query: String) -> Result<nt_workspace::SearchResults, String> {
    let config = load_config()?;
    nt_workspace::find(&config.workspace_dir, &query).map_err(|err| err.to_string())
}

/// 建目录（幂等）。
#[tauri::command]
pub fn neobot_fs_mkdir(rel: String) -> Result<(), String> {
    let config = load_config()?;
    nt_workspace::make_dir(&config.workspace_dir, &rel).map_err(|err| err.to_string())
}

/// 重命名 / 移动（两端都过 jail；拒动工作区根本身）。
#[tauri::command]
pub fn neobot_fs_rename(from: String, to: String) -> Result<(), String> {
    let config = load_config()?;
    nt_workspace::rename(&config.workspace_dir, &from, &to).map_err(|err| err.to_string())
}

/// 删文件 / 删空目录（**不递归**；目录非空即拒）。返回是否删的是目录。
#[tauri::command]
pub fn neobot_fs_remove(rel: String) -> Result<bool, String> {
    let config = load_config()?;
    nt_workspace::remove(&config.workspace_dir, &rel).map_err(|err| err.to_string())
}

/// 工作区根的绝对路径（前端标题栏显示用；只读，不构成越狱面）。
#[tauri::command]
pub fn neobot_fs_root() -> Result<String, String> {
    let config = load_config()?;
    Ok(config.workspace_dir.to_string_lossy().into_owned())
}

/// 本轮改动清单（`task_id` 空 = 全部任务，按时间倒序）。
#[tauri::command]
pub fn neobot_changes_list(task_id: Option<String>, limit: Option<i64>) -> Result<Vec<NeobotChangeItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let limit = limit.unwrap_or(200).clamp(1, 2000);
    let rows = store
        .list_changes(task_id.as_deref(), limit)
        .map_err(|err| err.to_string())?;
    Ok(rows.into_iter().map(NeobotChangeItem::from).collect())
}

/// 一任务的「本轮文件」分组（按路径聚合读/写/改次数 + 最近一笔）。
#[tauri::command]
pub fn neobot_changes_paths(task_id: String) -> Result<Vec<NeobotPathTallyItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let rows = store
        .tally_task_paths(&task_id)
        .map_err(|err| err.to_string())?;
    Ok(rows.into_iter().map(NeobotPathTallyItem::from).collect())
}

/// 「本轮文件」：跨**最近 N 个有改动的任务**按路径分组。
///
/// 不复用 `neobot_changes_paths`：那个要求调用方先知道 task_id，而侧边栏这个页签
/// 没有「当前任务」的概念（任务列表在另一个页签），硬传空串会让整页恒为空 ——
/// 看着能用、实际永远没数据。
#[tauri::command]
pub fn neobot_changes_recent_paths(
    task_limit: Option<i64>,
) -> Result<NeobotRecentPaths, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let rows = store
        .tally_recent_task_paths(task_limit.unwrap_or(1))
        .map_err(|err| err.to_string())?;
    let latest = store.latest_changed_task().map_err(|err| err.to_string())?;
    Ok(NeobotRecentPaths {
        paths: rows.into_iter().map(NeobotPathTallyItem::from).collect(),
        // 页面标题要写清在看**哪一轮**，否则用户不知道自己在看什么。
        task_id: latest.as_ref().map(|(id, _)| id.clone()),
        task_title: latest.map(|(_, title)| title),
    })
}

/// 「最近若干轮碰过的文件」+ 那一轮是谁（给标题用）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotRecentPaths {
    pub paths: Vec<NeobotPathTallyItem>,
    pub task_id: Option<String>,
    pub task_title: Option<String>,
}

/// 某会话最后一轮的助手回复（侧聊把结果画回自己的气泡用）。
///
/// 取不到返回 `None`（不是空串）—— 空串会和「空回复」混淆，调用方需要能分辨。
#[tauri::command]
pub fn neobot_convo_last_reply(convo_id: String) -> Result<Option<String>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    // `last_reply_of` 内部已把「读 steps 失败」降级成 None（转录没了不该让整个
    // 侧聊面板报错），故这里直接透传 Option。
    Ok(neotrix_neobot::nt_channel_dispatch::last_reply_of(
        &store, &convo_id,
    ))
}

/// 单条改动 + 前后内容（渲染 diff 的唯一入口）。
///
/// `content_omitted` 为真时 `before`/`after` 恒为 `None` —— 那是**如实**的
/// 「内容超限没存」，不是丢了；前端据此显示「内容过大，去打开文件对比」。
#[tauri::command]
pub fn neobot_changes_get(change_id: String) -> Result<Option<NeobotChangeViewItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let view = store
        .get_change(&change_id)
        .map_err(|err| err.to_string())?;
    Ok(view.map(NeobotChangeViewItem::from))
}

/// 清改动账（留存期 + 每任务保尾；返回删掉的行数）。
#[tauri::command]
pub fn neobot_changes_prune() -> Result<usize, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    Ok(neotrix_neobot::nt_changes::prune_best_effort(&store))
}

/// 改动账目 DTO（列表行；**不含**前后内容）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotChangeItem {
    pub id: String,
    pub task_id: String,
    pub at: String,
    pub path: String,
    pub kind: String,
    pub bytes: i64,
    pub content_omitted: bool,
}

impl From<neotrix_neobot::FileChange> for NeobotChangeItem {
    fn from(change: neotrix_neobot::FileChange) -> Self {
        Self {
            id: change.id,
            task_id: change.task_id,
            at: change.at,
            path: change.path,
            kind: change.kind,
            bytes: change.bytes,
            content_omitted: change.content_omitted,
        }
    }
}

/// 「本轮文件」分组行 DTO。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotPathTallyItem {
    pub path: String,
    pub reads: i64,
    pub writes: i64,
    pub edits: i64,
    pub bytes: i64,
    pub last_at: String,
    pub last_change: String,
}

impl From<neotrix_neobot::PathTally> for NeobotPathTallyItem {
    fn from(tally: neotrix_neobot::PathTally) -> Self {
        Self {
            path: tally.path,
            reads: tally.reads,
            writes: tally.writes,
            edits: tally.edits,
            bytes: tally.bytes,
            last_at: tally.last_at,
            last_change: tally.last_change,
        }
    }
}

/// 改动 + 前后内容 DTO。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotChangeViewItem {
    pub change: NeobotChangeItem,
    pub before: Option<String>,
    pub after: Option<String>,
}

impl From<neotrix_neobot::FileChangeView> for NeobotChangeViewItem {
    fn from(view: neotrix_neobot::FileChangeView) -> Self {
        Self {
            change: NeobotChangeItem::from(view.change),
            before: view.before,
            after: view.after,
        }
    }
}

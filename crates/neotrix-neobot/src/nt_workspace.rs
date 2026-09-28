//! `nt_workspace` — 工作区文件门面（侧边栏文件树 / 编辑器 / 搜索的 Rust 侧地基）。
//!
//! 吸收 `dsh-better-sidebar` 的文件工作台时，本模块是它的**唯一 IO 出口**：
//! 前端一个 `fetch` 都不发，全部经 Tauri 命令回到这里。
//!
//! 越狱律：路径两道关。第一道 `jail_join` 按**路径段**判定（拒 `/` 开头、
//! `~`、盘符/反斜杠、任一 `..` 段、超过 `MAX_PATH_DEPTH` 段）；第二道
//! `resolve_within` 对**解析后的真实路径**再核验一次，堵住「工作区里放一个
//! 指向 `/etc` 的软链接」这一类——`jail_join` 看不出软链接的真实去向，
//! 只有 `canonicalize` 看得出来。网关门 `nt_policy` 审的是**模型工具调用**，
//! 侧边栏是人在点、不过模型，故此处自带 jail；两道门都要，缺一即开口子。
//!
//! 有界律：目录列举 / 全局搜索都带上限（单目录项数、总访问数、命中数、深度），
//! 且 IO 用 `take()` 限读而非读全文件。免得在一个大仓里把 UI 线程和内存一起拖死。
//! 上限要**真的会停**：搜索的访问数上限是收敛闸门（撞到即整体退出遍历），
//! 读取的限读标记由实际读到的字节数算出 —— 名字承诺一件事、代码做另一件事，
//! 就等于把「静默截断」写进了 API 契约里。
//!
//! 截断**三处同一口径**：`list_dir` / `read_text` / [`find`] 都带 `truncated`，
//! 前端据此显示「已截断」。且口径不是「撞到上限就报」，而是
//! **「确有未访问的候选项被丢弃」才报**（[`find`] 的写法和理由见其文档）。
//! 「恰好 `MAX_SEARCH_HITS` 个匹配、且已全部扫完」报 `false`，
//! 「第 `MAX_SEARCH_HITS + 1` 个」才报 `true` —— 这一条正是「用户以为全库只有
//! 200 个匹配、进而断定某文件不存在」与「如实说可能还有更多」的分界。
//! 静默截断比报错更骗人，故 [`find`] 另带 `truncated_by` 说明是哪条上限砍的。
//!
//! 断链律：软链接按**目标类型**展示（链到目录可展开、链到文件当文件看），
//! 目标已消失的断链标 `broken_link`，前端标红而不是当空目录。

use std::collections::VecDeque;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::nt_config::WriteBudget;
use crate::nt_error::NtBotError;

/// 单目录列举项数上限（超出截断）。
pub const MAX_DIR_ENTRIES: usize = 2000;
/// 全局搜索命中数上限。
pub const MAX_SEARCH_HITS: usize = 200;
/// 全局搜索总访问目录项上限（防大仓空转）—— **撞到即让整个遍历收敛退出**。
///
/// [`find`] 里撞到它会 `break 'bfs`：外层 BFS 一起停，队列里剩下的目录一个都
/// 不再 `read_dir`，访问计数就此定格。撞到它时 [`find`] 报
/// `truncated_by = Visits`（当前目录只扫了一半，队列里剩下的也全没扫）。
///
/// 为什么强调「真的停下来」：早先这里只 `break` 了内层 `for`（当前目录的项），
/// 外层 BFS 仍把队列里剩下的目录各读一遍。总工作量被「入队即计数」间接兜住，
/// 但上限名叫「总访问上限」却不让遍历收敛 —— 读的人会以为撞顶后搜索就停了。
/// 名字承诺一件事、代码做另一件事，就是这种缺陷的温床。
pub const MAX_SEARCH_VISITS: usize = 20_000;
/// 全局搜索最大深度。
pub const MAX_SEARCH_DEPTH: usize = 12;
/// 文本读取上限（与工具侧 `READ_CAP` 同量级，1 MiB；超出按需另开通道）。
pub const READ_TEXT_CAP: u64 = 1024 * 1024;
/// 路径段数上限（超长即视为可疑，直接拒）。
pub const MAX_PATH_DEPTH: usize = 64;
/// 隐藏文件名（永远不列、搜索不穿透）。
pub const IGNORED: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "out",
    ".venv",
    "venv",
    "__pycache__",
    ".next",
    ".cache",
    ".pytest_cache",
    ".mypy_cache",
    ".turbo",
];

/// 目录下一项。
#[derive(Debug, Clone, serde::Serialize)]
pub struct DirEntry {
    /// 文件名（不含路径）。
    pub name: String,
    /// 相对工作区根的路径（正斜杠分隔）。
    pub rel: String,
    pub is_dir: bool,
    /// 自身是软链接（与 `is_dir` 正交：链到目录时两者皆真）。
    pub is_symlink: bool,
    /// 软链接目标已消失。
    pub broken_link: bool,
    pub size: i64,
    /// mtime（epoch 秒；取不到即 0）。
    pub mtime: i64,
}

/// 一次目录列举的结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct DirListing {
    /// 本次列举的相对路径（`""` = 根）。
    pub rel: String,
    /// 上级相对路径（根为 `None`）。
    pub parent: Option<String>,
    pub entries: Vec<DirEntry>,
    /// 因 `MAX_DIR_ENTRIES` 截断。
    pub truncated: bool,
}

/// 一次文本读取的结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileText {
    pub rel: String,
    pub content: String,
    /// 磁盘真实字节数（可能大于 `content.len()`，因截断）。
    pub size: i64,
    pub mtime: i64,
    /// 因 `READ_TEXT_CAP` 截断。
    pub truncated: bool,
    /// 二进制（含 NUL 字节）：`content` 为空，前端走「不支持预览」分支。
    pub binary: bool,
}

/// 一次文本写入的结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileWrite {
    pub rel: String,
    pub bytes: usize,
    pub mtime: i64,
    /// 本次新建（此前不存在）。
    pub created: bool,
}

/// 全局搜索命中。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchHit {
    /// 相对工作区根的路径。
    pub rel: String,
    pub name: String,
    pub is_dir: bool,
    pub size: i64,
    pub mtime: i64,
}

/// 截断 [`find`] 的那条上限（`SearchResults::truncated_by`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchCap {
    /// 命中数封顶 `MAX_SEARCH_HITS`：还有第 N+1 个匹配没被收进来。
    Hits,
    /// 总访问目录项数封顶 `MAX_SEARCH_VISITS`：**遍历整体提前退出** ——
    /// 当前目录只扫了一半，队列里剩下的目录一个没读。
    Visits,
    /// 目录深度封顶 `MAX_SEARCH_DEPTH`：更深的子树压根没进搜索。
    Depth,
}

/// 一次全局搜索的结果。
///
/// 形状与 [`DirListing`] / [`FileText`] 同形（结果体 + `truncated`），故 IPC
/// 直接序列化它、前端照同一个口径分支 —— 不给 `find` 造第三种说法。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResults {
    /// 命中的条目（按「名前缀优先 → 路径浅优先 → 字典序」排好）。
    pub hits: Vec<SearchHit>,
    /// 遍历**确实提前结束**、确有未访问的候选项被丢弃。
    ///
    /// 判据与 [`list_dir`] 的 `truncated` 一致：**只**在「确实还有没看的项」时
    /// 为真。于是「恰好 `MAX_SEARCH_HITS` 个匹配且已全部扫完」是 `false`，
    /// 「第 `MAX_SEARCH_HITS + 1` 个匹配」才是 `true` —— 前者可以放心说
    /// 「全库就这些」，后者只能说「可能还有更多」。
    pub truncated: bool,
    /// 砍人的是哪条上限（`None` ⟺ `truncated == false`）。
    ///
    /// 与 `truncated` 同源（一个 `Option` 决定两者），故不可能互相打架；
    /// 多条上限先后命中时按发生顺序报第一个，不假装只有一个原因。
    pub truncated_by: Option<SearchCap>,
}

/// 统一的越狱拒绝（规则名与 `nt_policy` 的 `workspace-jail` 对齐，审计好归口）。
fn escape() -> NtBotError {
    NtBotError::Denied {
        rule: "workspace-jail".to_owned(),
        reason: "path escapes workspace".to_owned(),
    }
}

/// 拼相对路径（`parent` 为空即顶层）。
pub fn join_rel(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}/{name}")
    }
}

/// 隐藏名判定（大小写敏感：`target` 该隐、`Target` 是真项目名，不隐）。
pub fn is_ignored(name: &str) -> bool {
    name == ".DS_Store" || IGNORED.contains(&name)
}

/// 二进制探测：前 8 KiB 有 NUL 即判二进制（与 `git` 启发式同源）。
pub fn is_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8192).any(|byte| *byte == 0)
}

/// mtime → epoch 秒（取不到即 0，不炸）。
pub fn mtime_secs(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0)
}

/// 第一道 jail 的**纯词法**部分：只判路径形状，不碰文件系统。
///
/// 拆出来是因为有两类调用方只需要形状判定、手上没有工作区路径：
/// `sidebar_open` 工具校验模型提议的目标（`nt_agent`）、
/// 以及给未来任何「先验后读」的入口留门。返回 `Ok(())` 即形状合法。
pub fn check_rel(rel: &str) -> Result<(), NtBotError> {
    let rel = rel.trim();
    if rel.is_empty() {
        return Ok(());
    }
    // 绝对路径**拒**而不是「重新定根」：把 `/etc/passwd` 静默改写成
    // `<ws>/etc/passwd` 会让人以为读到了系统文件、其实读的是工作区里的同名文件 ——
    // 报错比骗人好。
    //
    // 与 `nt_agent::join_workspace` **口径并不一致**（早先这里写的是「同律…
    // 两道 jail 口径才一致」，那是假的）：那边用 `contains("..")` 整串拒
    // （合法文件名 `a..b` 会被毙掉）、且**不**拒 `\` 与 `:`；这里逐段判
    // （`a..b` 照过）、并拒 `\`/`:`。两边各自 fail-closed，但可达集合不同 ——
    // 同一条路径可能一边开得了、一边开不了。
    // 不能写「口径一致」：读的人会据此以为全局只有一处规则要维护，而改一侧
    // 不会连带改另一侧。
    if rel.starts_with('/') || rel.starts_with('~') || rel.contains('\\') || rel.contains(':') {
        return Err(escape());
    }
    let mut depth = 0usize;
    for seg in rel.split('/') {
        match seg {
            "" | "." => continue,
            ".." => return Err(escape()),
            _ => {
                depth += 1;
                if depth > MAX_PATH_DEPTH {
                    return Err(NtBotError::Invalid(format!(
                        "path deeper than {MAX_PATH_DEPTH} segments"
                    )));
                }
            }
        }
    }
    Ok(())
}

/// 第一道 jail：按**路径段**判定，然后拼到工作区下。
///
/// 逐段判而不是 `contains("..")` —— 后者会把合法文件名 `a..b` 也毙掉；
/// 逐段既精确又同样 fail-closed（`..` 段一律拒）。
/// 空串 / 纯 `.` 视为工作区根本身（列表页要的就是它）。
pub fn jail_join(workspace: &Path, rel: &str) -> Result<PathBuf, NtBotError> {
    let rel = rel.trim();
    check_rel(rel)?;
    if rel.is_empty() {
        return Ok(workspace.to_path_buf());
    }
    Ok(workspace.join(rel))
}

/// 第二道 jail：jail 之后再核验**真实路径**仍在工作区内（堵软链接逃逸）。
///
/// 目标不存在时只能核验到最近的**已存在祖先**——新建文件走的正是这条路，
/// 祖先在内即安全。存在的目标则核验它自己（软链接在此现形）。
fn resolve_within(workspace: &Path, rel: &str) -> Result<PathBuf, NtBotError> {
    let joined = jail_join(workspace, rel)?;
    let root = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    let real = nearest_existing(&joined)
        .canonicalize()
        .map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    if !real.starts_with(&root) {
        return Err(escape());
    }
    Ok(joined)
}

/// 自身不存在就往上找最近的存在祖先（到顶仍不存在则原样返回，交给上层报错）。
fn nearest_existing(path: &Path) -> PathBuf {
    let mut cur = path;
    loop {
        if cur.symlink_metadata().is_ok() {
            return cur.to_path_buf();
        }
        match cur.parent() {
            Some(parent) if parent != cur => cur = parent,
            _ => return path.to_path_buf(),
        }
    }
}

/// 扩展名（小写、不带点；无扩展名即空串）。
pub fn extension_of(rel: &str) -> String {
    Path::new(rel)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// 该路径是不是图片（按扩展名；**仅用于挑工具**，类型判定另有 `is_binary` 之外的
/// 魔数嗅探 —— 扩展名可以撒谎，字节不会）。
pub fn is_image_name(rel: &str) -> bool {
    matches!(
        extension_of(rel).as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp"
    )
}

/// 绝对路径 → 相对工作区根（渲染用；不在根下则给绝对路径）。
pub fn rel_of(workspace: &Path, abs: &Path) -> String {
    abs.strip_prefix(workspace)
        .map(|rest| rest.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| abs.to_string_lossy().into_owned())
}

/// 相对路径的上级（根为 `None`；顶层的上级是 `Some("")` 即工作区根）。
fn parent_of(rel: &str) -> Option<String> {
    let trimmed = rel.trim().trim_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    Some(match trimmed.rsplit_once('/') {
        Some((head, _)) => head.to_owned(),
        None => String::new(),
    })
}

/// 列目录（懒加载的一层）。目录在前、同名不区分大小写序。
pub fn list_dir(workspace: &Path, rel: &str) -> Result<DirListing, NtBotError> {
    let dir = resolve_within(workspace, rel)?;
    if !dir.is_dir() {
        return Err(NtBotError::Invalid(format!("not a directory: {rel}")));
    }
    let parent = parent_of(rel);
    let read = std::fs::read_dir(&dir)
        .map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    let mut entries: Vec<DirEntry> = Vec::new();
    let mut truncated = false;
    for item in read {
        // 坏项跳过，不炸整目录（竞态删除的条目是常态）。
        let Ok(item) = item else { continue };
        let name = item.file_name().to_string_lossy().into_owned();
        if is_ignored(&name) {
            continue;
        }
        if entries.len() >= MAX_DIR_ENTRIES {
            truncated = true;
            break;
        }
        let path = item.path();
        // 软链接标记看 `symlink_metadata`（不跟随），类型看 `metadata`（跟随）。
        let is_symlink = path
            .symlink_metadata()
            .map(|meta| meta.file_type().is_symlink())
            .unwrap_or(false);
        let followed = path.metadata();
        let broken_link = followed.is_err();
        let is_dir = followed.as_ref().is_ok_and(|meta| meta.is_dir());
        entries.push(DirEntry {
            name,
            rel: join_rel(rel, &item.file_name().to_string_lossy()),
            is_dir,
            is_symlink,
            broken_link,
            size: followed.as_ref().map_or(0, |meta| {
                i64::try_from(meta.len()).unwrap_or(i64::MAX)
            }),
            mtime: followed.as_ref().map_or(0, mtime_secs),
        });
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(DirListing {
        rel: rel.to_owned(),
        parent,
        entries,
        truncated,
    })
}

/// 从 `buf` 取「至多 `cap` 字节、且不切碎多字节字符」的前缀，返回 `(前缀字节数, 是否限读)`。
///
/// 起点必须落在 `cap` 上而不是 `buf.len()` 上 —— 这正是早先那段的病根：`end`
/// 初值等于 `buf.len()`，于是下面这个补边界的循环恒不成立、`truncated` 恒 `false`。
/// 限读真发生（`buf.len() > cap`）时二者才不是同一个数。
///
/// 往后吞的是 UTF-8 续接字节（`0b10xxxxxx`）：`cap` 切在一个多字节字符中间时
/// 把它接完，故 `content` 不会以替换符结尾。生产上 `buf` 来自
/// `take(READ_TEXT_CAP + 4)`，那 4 字节余量刚好够接最长的四字节字符，
/// 所以「字符被接完」时 `truncated` 就是 `false`（内容真的完整，不是估的）。
fn cap_prefix_end(buf: &[u8], cap: usize) -> (usize, bool) {
    let mut end = cap.min(buf.len());
    while end < buf.len() && buf.get(end).is_some_and(|byte| (byte & 0xC0) == 0x80) {
        end += 1;
    }
    (end, end < buf.len())
}

/// 读一段（至多 `READ_TEXT_CAP` 字节）并解码，**不做**超限预检。
///
/// 拆出来是为了让「限读」这条兜底路径能被单测直接驱动：[`read_text`] 里超限文件
/// 在预检分支就返回了，正常调用根本走不到这里，而它恰恰是**唯一**会在有内容
/// 的情况下报 `truncated` 的那条路 —— 不给它一个可驱动的入口，就只能靠人肉推理
/// 它是否正确，那正是它当初烂掉的没人发现的原因。
fn read_capped(full: &Path, rel: &str, meta: &std::fs::Metadata) -> Result<FileText, NtBotError> {
    let size = i64::try_from(meta.len()).unwrap_or(i64::MAX);
    let mtime = mtime_secs(meta);
    let mut buf = Vec::new();
    std::fs::File::open(full)
        .map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?
        .take(READ_TEXT_CAP + 4)
        .read_to_end(&mut buf)
        .map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    if is_binary(&buf) {
        return Ok(FileText {
            rel: rel.to_owned(),
            content: String::new(),
            size,
            mtime,
            truncated: false,
            binary: true,
        });
    }
    let (end, truncated) = cap_prefix_end(&buf, usize::try_from(READ_TEXT_CAP).unwrap_or(usize::MAX));
    let slice = buf.get(..end).ok_or_else(|| {
        NtBotError::Invalid(format!("{rel}: byte offset {end} out of range"))
    })?;
    Ok(FileText {
        rel: rel.to_owned(),
        content: String::from_utf8_lossy(slice).into_owned(),
        size,
        mtime,
        truncated,
        binary: false,
    })
}

/// 读文本（二进制 / 超限如实标记，不猜）。
///
/// 两条截断路径，同一口径（**确有没读完的字节**才报 `truncated`）：
///
/// 1. **超限**（`meta.len() > READ_TEXT_CAP`）：在**读之前**就返回
///    `truncated: true` 且 `content` 为空 —— 前端按「大文件只给元信息」处理。
/// 2. **限读**（`take(READ_TEXT_CAP + 4)` 仍没读完）：正文是文件前缀，
///    `truncated` 由 [`cap_prefix_end`] 算出来（[`read_capped`]）。
///
/// 第 2 条在今天是**兜底**而非主路：超限文件走第 1 条就返回了。它防的是
/// stat 与 read 之间的竞态 —— 文件在 `meta.len()` 之后被改大，于是
/// `meta.len()` 说的是旧长度（放行）而 `take` 按新长度读（限读）。
/// 这条路必须是**真的会响的**：早先这里是一段恒假的死代码（`end` 初值等于
/// `buf.len()`，补边界循环永不成立，`truncated` 恒 `false`）。今天不咬人，
/// 但只要有人删掉第 1 条的提前返回，就变成**静默截断**——内容少了却报「完整」。
pub fn read_text(workspace: &Path, rel: &str) -> Result<FileText, NtBotError> {
    let full = resolve_within(workspace, rel)?;
    let meta = std::fs::metadata(&full).map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    if meta.is_dir() {
        return Err(NtBotError::Invalid(format!("is a directory: {rel}")));
    }
    if meta.len() > READ_TEXT_CAP {
        return Ok(FileText {
            rel: rel.to_owned(),
            content: String::new(),
            size: i64::try_from(meta.len()).unwrap_or(i64::MAX),
            mtime: mtime_secs(&meta),
            truncated: true,
            // 超限的不预判二进制：前端按「大文件只给元信息」处理。
            binary: false,
        });
    }
    read_capped(&full, rel, &meta)
}

/// 写文本（受写预算约束；自动补建父目录）。
///
/// 预算口径与工具侧一致：单次 ≤ `max_single_write_bytes`，一次保存整体
/// ≤ `max_turn_write_bytes`（保存是「一轮」的自然单位）。
pub fn write_text(
    workspace: &Path,
    rel: &str,
    content: &str,
    budget: WriteBudget,
) -> Result<FileWrite, NtBotError> {
    if rel.trim().is_empty() {
        return Err(escape());
    }
    if content.len() > budget.max_single_write_bytes {
        return Err(NtBotError::Invalid(format!(
            "content {} bytes exceeds single-write budget {}",
            content.len(),
            budget.max_single_write_bytes
        )));
    }
    if content.len() > budget.max_turn_write_bytes {
        return Err(NtBotError::Denied {
            rule: "write-budget".to_owned(),
            reason: format!(
                "save wrote {} bytes, budget {}",
                content.len(),
                budget.max_turn_write_bytes
            ),
        });
    }
    let full = resolve_within(workspace, rel)?;
    let created = full.symlink_metadata().is_err();
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    }
    std::fs::write(&full, content).map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    let mtime = std::fs::metadata(&full).map_or(0, |meta| mtime_secs(&meta));
    Ok(FileWrite {
        rel: rel.to_owned(),
        bytes: content.len(),
        mtime,
        created,
    })
}

/// 建目录（`create_dir_all`，幂等）。
pub fn make_dir(workspace: &Path, rel: &str) -> Result<(), NtBotError> {
    if rel.trim().is_empty() {
        return Err(escape());
    }
    let full = resolve_within(workspace, rel)?;
    std::fs::create_dir_all(&full).map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    Ok(())
}

/// 重命名 / 移动（两端都过 jail；**拒绝**动工作区根本身）。
pub fn rename(workspace: &Path, from: &str, to: &str) -> Result<(), NtBotError> {
    guard_not_root(workspace, from)?;
    guard_not_root(workspace, to)?;
    let src = resolve_within(workspace, from)?;
    let dst = resolve_within(workspace, to)?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|err| NtBotError::Io(format!("{to}: {err}")))?;
    }
    std::fs::rename(&src, &dst)
        .map_err(|err| NtBotError::Io(format!("{from} -> {to}: {err}")))?;
    Ok(())
}

/// 删文件 / 删空目录（目录非空即拒；不递归删 —— 递归删是另一条命令的事）。
/// 返回是否删的是目录。
pub fn remove(workspace: &Path, rel: &str) -> Result<bool, NtBotError> {
    guard_not_root(workspace, rel)?;
    let full = resolve_within(workspace, rel)?;
    let is_dir = full.is_dir();
    if is_dir {
        std::fs::remove_dir(&full).map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    } else {
        std::fs::remove_file(&full).map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    }
    Ok(is_dir)
}

/// 拒绝拿工作区根本（或空路径）当增删改的靶子。
fn guard_not_root(workspace: &Path, rel: &str) -> Result<(), NtBotError> {
    let trimmed = rel.trim().trim_matches('/');
    if trimmed.is_empty() {
        return Err(NtBotError::Denied {
            rule: "workspace-jail".to_owned(),
            reason: "refusing to operate on the workspace root".to_owned(),
        });
    }
    let root = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    if jail_join(workspace, trimmed)?.canonicalize().ok() == Some(root) {
        return Err(NtBotError::Denied {
            rule: "workspace-jail".to_owned(),
            reason: "refusing to operate on the workspace root".to_owned(),
        });
    }
    Ok(())
}

/// 全局文件名搜索（广度优先；命中名含 `query`，大小写不敏感）。
///
/// 排序：文件名前缀命中优先、其次文件名命中、再次路径短者优先 —— 与
/// better-sidebar 的手感一致（输入 `main` 时 `main.rs` 排在
/// `src/deep/main.rs` 前面）。
///
/// ## `truncated` 的判据（「恰好这么多」vs「被砍断了」）
///
/// `truncated == true` **只**在「确有未访问的候选项被丢弃」时成立，靠几处
/// `break` 的位置把它变成**可证**的事实，而不是估计：
///
/// - 上限判定放在**出队之前**（看 `queue.front()`），故每次因上限停下时
///   队列都还非空 ——「还有没读的目录」是确定的；
/// - 内层 `for` 里的命中数判定写在循环体里，能走到就说明**当前目录里还有
///   下一项没看** —— 与 [`list_dir`] 那个 `break` 同一形状；
/// - 访问数上限用 `break 'bfs`（跳的是**外层**），故撞顶后遍历整体退出、
///   队列里剩下的目录一个都不再读，`visits` 就此定格 —— 上限名与行为一致。
///
/// 于是三种情形的口径各自明确：
///
/// | 工作区 | `truncated` | 可以对用户说的话 |
/// |---|---|---|
/// | 恰好 `MAX_SEARCH_HITS` 个匹配，无其它待扫目录 | `false` | 「全库就这些」 |
/// | `MAX_SEARCH_HITS + 1` 个匹配 | `true`(`Hits`) | 「已截断，可能还有更多」 |
/// | 访问数 / 深度撞顶 | `true`(`Visits`/`Depth`) | 「已截断，可能还有更多」 |
///
/// **唯一会多报的一处**：命中数撞顶时队列里剩的目录若**全是空目录**，
/// 仍会报 `true`（我们没为它 `read_dir` 一次去证明「空」）。方向是偏保守 ——
/// 界面上只说「可能还有更多」，不会把用户引向「某文件不存在」的错误结论。
/// 反向（少报）才是原来那个 bug。
pub fn find(workspace: &Path, query: &str) -> Result<SearchResults, NtBotError> {
    Ok(search(workspace, query, SearchLimits::default()).0)
}

/// 搜索上限的三条。[`find`] 取 [`SearchLimits::default`]（= 三个 `MAX_*` 常量）。
///
/// 单测用它把小上限喂进**同一条**代码路径：访问数上限是 20_000，若只能用常量
/// 就得为撞它造两万个文件，那既慢又让测试变成「大仓 IO 压测」而不是断言。
struct SearchLimits {
    max_hits: usize,
    max_visits: usize,
    max_depth: usize,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            max_hits: MAX_SEARCH_HITS,
            max_visits: MAX_SEARCH_VISITS,
            max_depth: MAX_SEARCH_DEPTH,
        }
    }
}

/// 遍历本体（[`find`] 的实现体）。返回 `(结果, 实际访问的目录项数)`。
///
/// 第二个值是**遍历的内部刻度**、不是搜索结果的一部分（故不进 [`SearchResults`]，
/// 那是要过 IPC 给前端的形状，多塞一个内部计数器就是让前端多背一个字段）。
/// 它只给单测断言「撞上限后访问数不再增长」用。
fn search(workspace: &Path, query: &str, limits: SearchLimits) -> (SearchResults, usize) {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return (
            SearchResults {
                hits: Vec::new(),
                truncated: false,
                truncated_by: None,
            },
            0,
        );
    }
    let mut queue: VecDeque<(PathBuf, String, usize)> = VecDeque::new();
    queue.push_back((workspace.to_path_buf(), String::new(), 0));
    let mut hits: Vec<SearchHit> = Vec::new();
    let mut visits = 0usize;
    let mut cap: Option<SearchCap> = None;
    // 访问数上限是**收敛闸门**：撞到就 `break 'bfs`，连外层 BFS 一起停。
    // 早先只 `break` 了内层 `for`，外层继续把队列里剩下的目录各读一遍 ——
    // 上限写着「总访问上限」却不做这件事，读的人会以为撞顶后搜索就停了。
    'bfs: loop {
        // 先看队首再出队：上限判定放在 `pop_front()` **之前**，
        // 故每次因上限 break 时队列都还非空（= 确证有没读的目录）。
        let Some(front_depth) = queue.front().map(|(_, _, depth)| *depth) else {
            break; // 队列空了 = 全扫完，不是截断。
        };
        if hits.len() >= limits.max_hits {
            if cap.is_none() {
                cap = Some(SearchCap::Hits);
            }
            break;
        }
        // BFS 队首深度单调不减，故队首超深即「剩下的都读不了」。
        if front_depth > limits.max_depth {
            if cap.is_none() {
                cap = Some(SearchCap::Depth);
            }
            break;
        }
        let Some((dir, rel, depth)) = queue.pop_front() else {
            break;
        };
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for item in read.flatten() {
            visits += 1;
            // 能走到这一行，就说明当前目录里还有下一项没看 —— 丢东西是可证的。
            if visits > limits.max_visits {
                if cap.is_none() {
                    cap = Some(SearchCap::Visits);
                }
                break 'bfs; // 外层也停：队列里剩下的目录一个都不再读。
            }
            if hits.len() >= limits.max_hits {
                if cap.is_none() {
                    cap = Some(SearchCap::Hits);
                }
                break;
            }
            let name = item.file_name().to_string_lossy().into_owned();
            if is_ignored(&name) {
                continue;
            }
            let path = item.path();
            let is_dir = path.metadata().map(|meta| meta.is_dir()).unwrap_or(false);
            let child_rel = join_rel(&rel, &name);
            let lower = name.to_lowercase();
            if lower.contains(&needle) {
                let meta = path.metadata().ok();
                hits.push(SearchHit {
                    rel: child_rel.clone(),
                    name,
                    is_dir,
                    size: meta.as_ref().map_or(0, |meta| {
                        i64::try_from(meta.len()).unwrap_or(i64::MAX)
                    }),
                    mtime: meta.as_ref().map_or(0, mtime_secs),
                });
            }
            if is_dir {
                queue.push_back((path, child_rel, depth + 1));
            }
        }
    }
    hits.sort_by(|a, b| {
        let rank = |hit: &SearchHit| {
            u8::from(hit.is_dir) << 1
                | u8::from(hit.name.to_lowercase().starts_with(&needle))
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.rel.matches('/').count().cmp(&b.rel.matches('/').count()))
            .then_with(|| a.rel.to_lowercase().cmp(&b.rel.to_lowercase()))
    });
    (
        SearchResults {
            truncated: cap.is_some(),
            truncated_by: cap,
            hits,
        },
        visits,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(case: &str) -> PathBuf {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-workspace-test-{}", case));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    /// 造 n 个命中文件（`hit-0000…`；空文件，`File::create` 比 `write` 快）。
    fn seed_hits(ws: &Path, n: usize) {
        for i in 0..n {
            std::fs::File::create(ws.join(format!("hit-{i:04}.txt"))).expect("造命中文件");
        }
    }

    #[test]
    fn jail_rejects_every_escape_shape() {
        let ws = root("jail");
        for bad in [
            "/etc/passwd",
            "~/secret",
            "../outside",
            "a/../../b",
            "..",
            "C:\\windows",
            "a\\b",
        ] {
            let err = jail_join(&ws, bad).expect_err(bad);
            match err {
                NtBotError::Denied { rule, .. } => assert_eq!(rule, "workspace-jail", "{bad}"),
                other => panic!("{bad} -> {other:?}"),
            }
        }
        // 合法形态照过（`a..b` 是合法文件名，逐段判定不该毙它）。
        assert!(jail_join(&ws, "a..b.txt").is_ok());
        assert!(jail_join(&ws, "./a/b.txt").is_ok());
        assert!(jail_join(&ws, "a//b.txt").is_ok());
        assert!(jail_join(&ws, "").is_ok());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn jail_depth_is_capped() {
        let ws = root("depth");
        let deep = vec!["x"; MAX_PATH_DEPTH + 1].join("/");
        assert!(matches!(
            jail_join(&ws, &deep),
            Err(NtBotError::Invalid(_))
        ));
        let ok = vec!["x"; MAX_PATH_DEPTH].join("/");
        assert!(jail_join(&ws, &ok).is_ok());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_caught_by_second_jail() {
        use std::os::unix::fs::symlink;
        let outside = root("symlink-outside");
        std::fs::write(outside.join("secret.txt"), "top secret").expect("write");
        let ws = root("symlink-inside");
        symlink(outside.join("secret.txt"), ws.join("leak.txt")).expect("symlink");
        // 逃逸目标：第二道 jail 必须拦下。
        assert!(read_text(&ws, "leak.txt").is_err());
        // 目录软链接同样拦下。
        symlink(&outside, ws.join("leakdir")).expect("symlink dir");
        assert!(list_dir(&ws, "leakdir").is_err());
        let _ = std::fs::remove_dir_all(&ws);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn list_dir_sorts_dirs_first_and_hides_ignored() {
        let ws = root("list");
        std::fs::create_dir_all(ws.join("zeta")).expect("mkdir");
        std::fs::create_dir_all(ws.join("alpha")).expect("mkdir");
        std::fs::create_dir_all(ws.join("node_modules/pkg")).expect("mkdir");
        std::fs::write(ws.join("b.txt"), "b").expect("write");
        std::fs::write(ws.join("A.txt"), "a").expect("write");
        std::fs::write(ws.join(".DS_Store"), "junk").expect("write");
        let listing = list_dir(&ws, "").expect("list");
        let names: Vec<&str> = listing.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "zeta", "A.txt", "b.txt"]);
        assert!(listing.entries[0].is_dir);
        assert!(listing.parent.is_none());
        // 隐藏名是**展示过滤**，不是访问控制（与 `files.exclude` 同律）：
        // 树里不列、搜索不穿透，但显式给路径仍打得开 —— 否则用户想看
        // `dist/index.js` 会被自己的过滤规则挡住。搜索不穿透才单测在下面。
        assert!(list_dir(&ws, "node_modules").is_ok());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn list_dir_reports_parent_and_truncation() {
        let ws = root("parent");
        std::fs::create_dir_all(ws.join("a/b/c")).expect("mkdir");
        let listing = list_dir(&ws, "a/b/c").expect("list");
        assert_eq!(listing.parent.as_deref(), Some("a/b"));
        let root_listing = list_dir(&ws, "").expect("list");
        assert!(root_listing.parent.is_none());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn read_text_flags_binary_and_truncation() {
        let ws = root("read");
        std::fs::write(ws.join("a.txt"), "hello").expect("write");
        let got = read_text(&ws, "a.txt").expect("read");
        assert_eq!(got.content, "hello");
        assert!(!got.binary && !got.truncated);
        // 目录不是文件。
        std::fs::create_dir_all(ws.join("d")).expect("mkdir");
        assert!(read_text(&ws, "d").is_err());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[cfg(unix)]
    #[test]
    fn read_text_keeps_multibyte_char_intact() {
        let ws = root("utf8");
        // 故意让文件字节数不等于字符数，验证不切碎多字节字符。
        let text = "中文与 emoji 🌍 混排";
        std::fs::write(ws.join("u.txt"), text).expect("write");
        let got = read_text(&ws, "u.txt").expect("read");
        assert_eq!(got.content, text);
        assert!(!got.truncated);
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn cap_prefix_end_reports_real_limit_read_and_keeps_multibyte_intact() {
        // 限读分支的定点测试：buf 造到比 cap 长（「中」= E4 B8 AD 三字节，
        // 故意让 cap=6 切在它的**第二个**字节上）。
        let mut buf = Vec::new();
        buf.extend_from_slice(b"aaaaa");
        buf.extend_from_slice("中".as_bytes());
        buf.extend_from_slice(b"bbbbb");
        // 限读真发生 ⇒ 必须报 true。旧的死代码在这里恒 false。
        let (end, truncated) = cap_prefix_end(&buf, 6);
        assert!(truncated, "buf 比 cap 长 ⇒ 确有没读完的字节，必须报限读");
        assert_eq!(end, 8, "从 cap 往后吞完「中」的续接字节，不能停在 6");
        assert_eq!(
            &buf[..end],
            "aaaaa中".as_bytes(),
            "截出的前缀必须以完整字符收尾，不能是半个「中」"
        );
        // 余量刚好够接完字符（生产里就是 take(cap + 4)）⇒ 内容真的完整，不算截断。
        let (end, truncated) = cap_prefix_end(&buf[..8], 6);
        assert_eq!(end, 8);
        assert!(!truncated, "字符被接完 ⇒ 没少内容，不该报截断");
        // 根本没限读（buf 比 cap 短）⇒ 同样不报。
        let (end, truncated) = cap_prefix_end(&buf[..5], 6);
        assert_eq!(end, 5);
        assert!(!truncated);
    }

    #[test]
    fn read_capped_marks_limit_read_and_shrinks_the_body() {
        // 走 `read_capped`（绕过超限预检）专门驱动「限读」这条兜底路径：
        // 它的 truncated 必须真为 true，且正文必须真的比文件短 ——
        // 两者缺一，truncated 就只是个装饰。
        let ws = root("read-cap");
        let cap = usize::try_from(READ_TEXT_CAP).expect("cap");
        // 让 4 字节 emoji 正好跨在 cap 边界上（emoji 落在 [cap-1, cap+2]）。
        let mut text = "a".repeat(cap - 1);
        text.push('🌍');
        text.push_str("tail");
        let real_len = text.len();
        assert!(real_len > cap, "夹具必须超过上限，否则测的不是限读");
        std::fs::write(ws.join("big.txt"), &text).expect("write");

        let full = ws.join("big.txt");
        let meta = std::fs::metadata(&full).expect("stat");
        let got = read_capped(&full, "big.txt", &meta).expect("read");
        assert!(got.truncated, "限读真发生了，truncated 必须为 true");
        assert!(
            got.content.len() < real_len,
            "正文 {} 字节应短于文件真实长度 {real_len}，否则 truncated 是装饰",
            got.content.len()
        );
        assert_eq!(got.size as usize, real_len, "size 报磁盘真实长度");
        assert!(!got.content.contains('\u{FFFD}'), "不能以替换符结尾（切碎字符了）");
        assert!(got.content.ends_with('🌍'), "跨界字符应被完整接上：…{}", &got.content[got.content.len() - 6..]);
        assert!(got.content.len() >= cap, "多字节字符应被接完（故可略超 cap）");
        // 限读发生时不能被误判成二进制（`take` 边界切在字符中间也不该翻脸）。
        assert!(!got.binary);
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn read_text_marks_oversize_file_as_truncated_with_shorter_body() {
        // 端到端那条：超限预检分支。`truncated` 为真 + 正文短于真实长度。
        let ws = root("read-oversize");
        let cap = usize::try_from(READ_TEXT_CAP).expect("cap");
        let big = "x".repeat(cap + 4096);
        std::fs::write(ws.join("big.txt"), &big).expect("write");
        let got = read_text(&ws, "big.txt").expect("read");
        assert!(got.truncated, "超限必须报截断，不能静默给半份");
        assert_eq!(got.size as usize, big.len(), "size 报磁盘真实长度");
        assert!(got.content.len() < big.len(), "正文必须短于真实长度");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn write_text_honours_budget_and_creates_parents() {
        let ws = root("write");
        let budget = WriteBudget {
            max_single_write_bytes: 16,
            max_turn_write_bytes: 64,
            max_task_write_bytes: 64,
        };
        let done = write_text(&ws, "deep/nested/new.txt", "ok", budget).expect("write");
        assert!(done.created);
        assert_eq!(std::fs::read_to_string(ws.join("deep/nested/new.txt")).ok().as_deref(), Some("ok"));
        // 超单次预算 → Invalid；超单轮预算 → Denied(write-budget)。
        let big = "x".repeat(17);
        assert!(matches!(
            write_text(&ws, "big.txt", &big, budget),
            Err(NtBotError::Invalid(_))
        ));
        // 既有文件覆写不报 created。
        let again = write_text(&ws, "deep/nested/new.txt", "ok2", budget).expect("rewrite");
        assert!(!again.created);
        // 越狱写照拒。
        assert!(write_text(&ws, "../evil.txt", "x", budget).is_err());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn find_ranks_basename_prefix_first() {
        let ws = root("find");
        std::fs::create_dir_all(ws.join("src/deep")).expect("mkdir");
        std::fs::create_dir_all(ws.join("node_modules")).expect("mkdir");
        std::fs::write(ws.join("main.rs"), "1").expect("write");
        std::fs::write(ws.join("src/deep/main.rs"), "2").expect("write");
        std::fs::write(ws.join("node_modules/main.js"), "3").expect("write");
        std::fs::write(ws.join("unrelated.txt"), "4").expect("write");
        let hits = find(&ws, "main").expect("find").hits;
        let rels: Vec<&str> = hits.iter().map(|h| h.rel.as_str()).collect();
        assert_eq!(rels, vec!["main.rs", "src/deep/main.rs"]);
        // 隐藏目录搜索**不穿透**（与上一条对照：树不列 + 搜索不进）。
        let deep_hits = find(&ws, "main").expect("find").hits;
        assert!(!deep_hits.iter().any(|h| h.rel.contains("node_modules")));
        // 空查询不炸、不返。
        assert!(find(&ws, "   ").expect("empty").hits.is_empty());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn find_flags_hit_cap_instead_of_silently_dropping_the_rest() {
        // 缺陷本体：第 201 个匹配被砍掉，而返回类型不吭声 → 用户以为全库就 200 个。
        let ws = root("find-hits-over");
        seed_hits(&ws, MAX_SEARCH_HITS + 1);
        let got = find(&ws, "hit").expect("find");
        assert!(got.truncated, "命中数被上限砍过，必须报截断");
        assert_eq!(got.truncated_by, Some(SearchCap::Hits), "要说清是哪条上限砍的");
        assert_eq!(
            got.hits.len(),
            MAX_SEARCH_HITS,
            "应正好停在上限上（不多不少）"
        );
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn find_reports_no_truncation_when_the_count_merely_equals_the_cap() {
        // 防「永远返回 true」那种把谎话换方向的实现：恰好等于上限、且没有别的
        // 待扫目录 ⇒ 真的扫完了。此时报 true 会让用户白白再搜一遍。
        let ws = root("find-hits-exact");
        seed_hits(&ws, MAX_SEARCH_HITS);
        let got = find(&ws, "hit").expect("find");
        assert_eq!(
            got.hits.len(),
            MAX_SEARCH_HITS,
            "不该少于上限 —— 少了就是假截断（谎报被砍）"
        );
        assert!(!got.truncated, "恰好 {MAX_SEARCH_HITS} 个且已全扫完 ⇒ 不算截断");
        assert_eq!(got.truncated_by, None, "没截断就不该报出上限名");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn find_flags_hit_cap_when_pending_dirs_were_never_scanned() {
        // 保守方向：命中数撞顶时队列里还有没读的目录 ⇒ 一定报 true。
        // 即便那个目录恰好是空的也照报（不为了证明「空」再 read_dir 一次）：
        // 多报的只是「可能还有更多」，不会把人引向「该文件不存在」的错误结论；
        // 反向（少报）才是原来那个 bug。
        let ws = root("find-hits-pending");
        seed_hits(&ws, MAX_SEARCH_HITS);
        std::fs::create_dir_all(ws.join("unscanned")).expect("mkdir");
        let got = find(&ws, "hit").expect("find");
        assert_eq!(got.hits.len(), MAX_SEARCH_HITS);
        assert!(
            got.truncated,
            "unscanned/ 压根没被扫过，不能报「全扫完了」"
        );
        assert_eq!(got.truncated_by, Some(SearchCap::Hits));
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn find_flags_visit_cap_for_a_partially_scanned_workspace() {
        // 访问数上限（MAX_SEARCH_VISITS）也砍结果，不能只报命中数那一条。
        // 夹具必须**超过** 20000 个目录项，故这个测试真要造两万个文件 ——
        // 这是访问数上限唯一的证明方式，值这份 IO。命中数刻意远低于上限：
        // 「`truncated == true` 且 `hits.len() < MAX_SEARCH_HITS`」只可能
        // 是访问数砍的，撞不出别的解释。
        let ws = root("find-visits");
        const DIRS: usize = 40;
        const PER_DIR: usize = 500;
        for d in 0..DIRS {
            let dir = ws.join(format!("d{d:02}"));
            std::fs::create_dir_all(&dir).expect("mkdir");
            for i in 0..PER_DIR {
                std::fs::File::create(dir.join(format!("f{i:04}.txt"))).expect("造噪声文件");
            }
        }
        // 少量真命中，放在最前面的目录里，保证一定被扫到（不依赖目录读取顺序）。
        for i in 0..5 {
            std::fs::write(ws.join(format!("d00/needle-{i}.txt")), "").expect("造命中文件");
        }
        let got = find(&ws, "needle").expect("find");
        assert!(got.truncated, "访问数撞顶必须报截断（部分目录只扫了一半）");
        assert_eq!(got.truncated_by, Some(SearchCap::Visits));
        assert_eq!(got.hits.len(), 5, "d00 里的 5 个命中都在第一个目录，不受影响");
        assert!(
            got.hits.len() < MAX_SEARCH_HITS,
            "命中数没撞顶 ⇒ 这次的截断只可能来自访问数上限"
        );
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn find_visit_cap_stops_the_whole_traversal_not_just_one_directory() {
        // 缺陷本体：`MAX_SEARCH_VISITS` 名为「总访问上限」，早先却只 `break` 了
        // 内层 `for`（当前目录的项），外层 BFS 继续把队列里剩下的目录各读一遍 ——
        // 上限名承诺「撞顶就停」，代码做的是「撞顶只歇一口」。
        //
        // 用**小上限**驱动（生产常量 20000，不该为撞它造两万个文件），
        // 走的是与生产同一条 `search` 路径，没有测试专用后门。
        const WIDE: usize = 30; // 根下的子目录数 = 根目录的项数
        const PER_DIR: usize = 30; // 每个子目录的项数（含 1 个 needle）
        let ws = root("find-visits-gate");
        for d in 0..WIDE {
            let dir = ws.join(format!("d{d:02}"));
            std::fs::create_dir_all(&dir).expect("mkdir");
            for i in 0..PER_DIR {
                let name = if i == 0 {
                    format!("needle-{i:02}.txt")
                } else {
                    format!("f{i:02}.txt")
                };
                std::fs::File::create(dir.join(name)).expect("造噪声文件");
            }
        }
        // 全量扫完恰好 WIDE 个命中；撞顶后拿到的是「一个目录都没扫完」的那点。
        let small = search(
            &ws,
            "needle",
            SearchLimits {
                max_visits: 50,
                ..SearchLimits::default()
            },
        );
        assert!(
            small.1 > 50,
            "必须真的撞到上限（否则这个测试没测到闸门）：{}",
            small.1
        );
        assert!(
            small.1 <= 51,
            "撞顶后总访问数不得再增长（非收敛实现会继续把剩下 29 个目录各读一项，\
             计数漂到 80 = 上限 + 队列长度，实测值）：{}",
            small.1
        );
        assert!(small.0.truncated, "撞顶必须报截断");
        assert_eq!(small.0.truncated_by, Some(SearchCap::Visits));
        assert!(
            small.0.hits.len() < WIDE,
            "剩下的目录一个都不该再被读，故命中数必须少于全量扫完的 {WIDE}：{}",
            small.0.hits.len()
        );

        // 换个上限重跑：访问数应随上限走（闸门），而不是恒定扫完全树（旧实现下
        // 两次的计数会相等，都等于全量项数）。
        let bigger = search(
            &ws,
            "needle",
            SearchLimits {
                max_visits: 200,
                ..SearchLimits::default()
            },
        );
        assert!(bigger.1 > small.1, "上限放大 ⇒ 能多访问：{} vs {}", bigger.1, small.1);
        assert!(bigger.1 <= 201, "仍是撞顶即停：{}", bigger.1);
        assert_eq!(bigger.0.truncated_by, Some(SearchCap::Visits));
        assert!(bigger.0.hits.len() < WIDE, "仍不该扫完全树");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn find_flags_depth_cap_and_keeps_the_boundary_level() {
        // MAX_SEARCH_DEPTH 是**含端点**的：深度 12 仍要扫，13 起不进搜索。
        let ws = root("find-depth");
        let mut dirs: Vec<PathBuf> = vec![ws.clone()];
        for d in 1..=MAX_SEARCH_DEPTH + 2 {
            let next = dirs.last().expect("非空").join(format!("d{d}"));
            std::fs::create_dir_all(&next).expect("mkdir");
            dirs.push(next);
        }
        // dirs[12] = 深度 12（上限那一层）；dirs[14] = 深度 14（超限）。
        std::fs::write(dirs[MAX_SEARCH_DEPTH].join("edge-needle.txt"), "").expect("write");
        std::fs::write(dirs[MAX_SEARCH_DEPTH + 2].join("deep-needle.txt"), "").expect("write");

        let got = find(&ws, "needle").expect("find");
        assert!(got.truncated, "深度撞顶必须报截断（更深的子树没进搜索）");
        assert_eq!(got.truncated_by, Some(SearchCap::Depth));
        let rels: Vec<&str> = got.hits.iter().map(|h| h.rel.as_str()).collect();
        assert_eq!(rels.len(), 1, "只该命中上限那一层，实际：{rels:?}");
        assert!(rels[0].ends_with("edge-needle.txt"), "上限那层要能搜到：{rels:?}");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn remove_and_rename_refuse_the_root() {
        let ws = root("mutate");
        std::fs::create_dir_all(ws.join("d")).expect("mkdir");
        std::fs::write(ws.join("f.txt"), "x").expect("write");
        assert!(remove(&ws, "").is_err());
        assert!(remove(&ws, "..").is_err());
        assert!(rename(&ws, "", "other").is_err());
        // 非空目录不递归删。
        std::fs::write(ws.join("d/inner.txt"), "y").expect("write");
        assert!(remove(&ws, "d").is_err());
        // `remove` 返回的是「删的是不是目录」：删文件为 `false`，删空目录为 `true`。
        assert!(!remove(&ws, "f.txt").expect("rm file"));
        assert!(!remove(&ws, "f.txt").is_ok(), "已删再删应失败");
        // 重命名跨目录。
        std::fs::create_dir_all(ws.join("dst")).expect("mkdir");
        std::fs::write(ws.join("a.txt"), "z").expect("write");
        rename(&ws, "a.txt", "dst/b.txt").expect("rename");
        assert!(ws.join("dst/b.txt").is_file());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn rel_of_renders_workspace_relative() {
        let ws = root("rel");
        std::fs::create_dir_all(ws.join("x/y")).expect("mkdir");
        assert_eq!(rel_of(&ws, &ws.join("x/y/z.txt")), "x/y/z.txt");
        assert_eq!(rel_of(&ws, Path::new("/elsewhere")), "/elsewhere");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn binary_detection_matches_git_heuristic() {
        assert!(is_binary(b"MZ\x90\x00binary"));
        assert!(!is_binary("plain text".as_bytes()));
        assert!(!is_binary(&[0xC3, 0xA9]));
    }
}


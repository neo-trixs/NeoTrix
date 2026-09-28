//! `nt_git` — Git 视角（`dsh-better-sidebar`「文件变动 · Git 视角」的本机实现）。
//!
//! 与「本轮文件」视角的分工：`file_changes` 记的是**这一轮模型碰了什么**，
//! 本模块看的是**仓库里现在欠着什么**（未提交的改动、暂存区、历史）。
//! 两个视角合一才完整 —— 模型这轮没碰但三天前自己改的，也得看得见。
//!
//! **不引 git 库**：`git` 二进制已经在 PATH 上，自己解 porcelain 输出要写的
//! 解析代码远超一个 crate 的价值，而 `git` 的边界行为（rename 检测、
//! 换行符、`.gitattributes`、稀疏检出）自己重写一定漏。故一律 `Command` 调 CLI。
//!
//! 越狱律：`-C <dir>` 的 `dir` 由 [`repo_root`] 上溯后**核验真实路径仍在工作区内**；
//! 路径参数一律写在 `--` **之后**（否则 `-rf` 开头的文件名会被 git 当成选项）。
//!
//! pathspec 过**两道** jail：
//! 1. 第一道 `nt_workspace::jail_join`（内含 `check_rel`，**纯词法**：拒 `/` 开头、
//!    `~`、盘符/反斜杠、任一 `..` 段、超 `MAX_PATH_DEPTH`）；
//! 2. 第二道 [`jail_real`]：对 `canonicalize` 之后的**真实路径**再核验一次，
//!    堵住「工作区里一个指向外部的软链接 + 一条不含 `..` 的相对路径」——
//!    这条路径第一道门放行、第二道门现形。`diff` 的未跟踪分支直接
//!    `read_to_string` 落盘，没有第二道门就是**读工作区之外的文件**。
//!
//! 第二道门本可复用侧边栏的 `nt_workspace::resolve_within`（`read_text` 等走的
//! 就是它），但那是**私有** `fn`，而本模块只许改自己这一个文件，故在此**同律
//! 自实现**（语义逐条对齐，见 [`jail_real`] 的注释）。把 `resolve_within`
//! 改成 `pub` 才是更省事的一条路，但要动 `nt_workspace.rs`。
//! 落地点清单：`diff` / `log` / `stage` / `unstage` / `discard_workspace_changes`
//! —— 五处**都**过 [`jail_real`]，不要新增 pathspec 落地点而漏掉它。
//! 另配 `GIT_TERMINAL_PROMPT=0`（缺凭据时立刻失败而不是挂死 UI）与
//! `GIT_OPTIONAL_LOCKS=0`（只读命令不抢 `.git/index.lock`）。
//!
//! 环境脱敏：与 `nt_agent::execute_bash` 同律，`env_clear` + 最小白名单
//! （PATH/HOME/…），不把宿主 secrets 递进子进程。
//!
//! **hook 诚实声明**：`commit()` 默认**会**执行仓库里的 `.git/hooks/*` ——
//! 那意味着「点提交」会跑仓库作者放在磁盘上的任意代码。这是 git 的正常语义
//! （用户按下提交时就预期自己的 hook 跑起来），故默认不跳过；但 `no_verify`
//! 参数留给调用方强制 `--no-verify`，并在 Tauri 命令层显式暴露该选项。
//! 这一条**故意不做静默** —— 悄悄跳过用户的 hook 和悄悄执行它一样是骗人。

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use crate::nt_error::NtBotError;
use crate::nt_workspace::jail_join;

/// git 子进程超时（UI 点一下不该等一分钟）。
pub const GIT_TIMEOUT: Duration = Duration::from_secs(20);
/// stdout 上限（防 `git log -p` 之类撑爆内存）。
pub const GIT_OUTPUT_CAP: usize = 8 * 1024 * 1024;
/// 递进子进程的最小环境白名单（与 `nt_agent::execute_bash` 同律）。
const ENV_ALLOW: &[&str] = &[
    "PATH", "HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "TMPDIR", "TEMP", "TERM",
];

/// 一个有改动的文件。
#[derive(Debug, Clone, serde::Serialize)]
pub struct GitFile {
    /// 相对仓库根的路径。
    pub path: String,
    /// porcelain v1 的两字母状态（如 `M ` / ` M` / `MM` / `??` / `A `）。
    pub xy: String,
    /// 有暂存内容。
    pub staged: bool,
    /// 有未暂存内容。
    pub unstaged: bool,
    /// 未跟踪。
    pub untracked: bool,
}

/// 一条提交。
#[derive(Debug, Clone, serde::Serialize)]
pub struct GitCommit {
    pub id: String,
    pub short: String,
    pub at: String,
    pub author: String,
    pub subject: String,
}

/// 提交结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct GitCommitResult {
    pub short: String,
    pub subject: String,
}

/// `git` 在不在 PATH 上（UI 据此决定要不要显示 Git 视角）。
pub fn available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// 该目录是不是仓库根（有 `.git`）。
pub fn is_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

/// 解析出**工作区内**的仓库根（从 `subdir` 上溯，出了工作区就停）。
///
/// 上溯有边界：最多 `MAX_ROOT_WALK` 层，且每层都在 jail 内 —— 工作区里
/// 嵌一个指向外部的软链接不该把仓库根解析到用户家目录去。
pub fn repo_root(workspace: &Path, subdir: &str) -> Result<PathBuf, NtBotError> {
    const MAX_ROOT_WALK: usize = 24;
    let start = jail_join(workspace, subdir)?;
    let workspace_real = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    let mut cursor = if start.is_dir() {
        start
    } else {
        start.parent().map(Path::to_path_buf).unwrap_or(start.clone())
    };
    for _ in 0..MAX_ROOT_WALK {
        if is_repo(&cursor) {
            let real = cursor
                .canonicalize()
                .map_err(|err| NtBotError::Io(format!("canonicalize: {err}")))?;
            if !real.starts_with(&workspace_real) {
                return Err(NtBotError::Denied {
                    rule: "workspace-jail".to_owned(),
                    reason: "repository root escapes workspace".to_owned(),
                });
            }
            return Ok(cursor);
        }
        match cursor.parent() {
            Some(parent) if parent != cursor => cursor = parent.to_path_buf(),
            _ => break,
        }
    }
    Err(NtBotError::Invalid(format!(
        "no git repository in workspace (from '{subdir}')"
    )))
}

/// 第二道 jail：jail 之后核验**真实路径**仍在工作区内（堵软链接逃逸）。
///
/// 为什么在这里自实现、而不是复用侧边栏那套 `nt_workspace::resolve_within`：
/// 那是 `nt_workspace` 的**私有** `fn`，够不着，而本模块只许改自己这一个文件
/// （把 `resolve_within` 改 `pub` 更省事，但那是 `nt_workspace.rs` 的改动）。
/// 语义与它**逐条对齐**：`jail_join`（第一道门）→ 取**最近的已存在祖先** →
/// `canonicalize` → `starts_with(canonical workspace)`。两条路对同一条路径
/// 应当给出同一判定。
///
/// **「目标不存在」怎么判 —— 取舍**：目标不存在时 `canonicalize` 直接失败
/// （`stage` 一个还没创建的文件、`log` 一个还没提交的文件都是这种）。
/// **不能**把「canonicalize 失败」当成越界：那会把正常操作全部误杀，而且杀得
/// 没有道理（用户明明在工作区内点暂存）。故退化为核验**最近的已存在祖先**——
/// 祖先（一定是目录）真身在工作区内即放行。
/// 这个退化是**安全**的，理由是：缺失的那几段**不含分隔符**（`check_rel` 已逐段
/// 判过、没有 `..` 段、没有 `\`），纯拼串不可能自己跑出工作区；而祖先若是
/// 指向外部的软链接，`canonicalize` 在祖先这一层就现形，照样拒。
/// 代价：目标不存在时**核验不到末段**——但末段还不存在，没有软链接可言。
/// 等它被创建出来，下一次调用（那时它存在了）就会核验到它自己。
/// 「先创建、后过门」这个窗口是这条路固有的，不是本实现引入的。
///
/// 另：目标**存在**时取的是它自己（`symlink_metadata` 对软链接也成功），
/// 所以末段软链接、工作区内的软链接目录一并现形。
fn jail_real(workspace: &Path, rel: &str) -> Result<PathBuf, NtBotError> {
    // 第一道门（纯词法）在这里一并过掉，故调用方不必再单独 `check_rel`。
    let joined = jail_join(workspace, rel)?;
    let root = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    let real = nearest_existing(&joined)
        .canonicalize()
        .map_err(|err| NtBotError::Io(format!("{rel}: {err}")))?;
    if !real.starts_with(&root) {
        // 与 `repo_root` 同形状的 `Denied`（rule 一致，UI 才能按同一条规则处理）。
        return Err(NtBotError::Denied {
            rule: "workspace-jail".to_owned(),
            reason: format!(
                "'{rel}' resolves to '{}', outside the workspace",
                real.display()
            ),
        });
    }
    Ok(joined)
}

/// 自身不存在就往上找最近的存在祖先（到顶仍不存在则原样返回，交给上层报错）。
///
/// 用 `symlink_metadata`（**不**跟随软链接）：存在一个指向外部的软链接
/// 恰恰要算「存在」，好让 `jail_real` 接着去 `canonicalize` 它、把它现形。
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

/// 跑一条 git 命令（`args` 不含 `-C`）。
fn run_git(dir: &Path, args: &[&str]) -> Result<String, NtBotError> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("-C")
        .arg(dir)
        .args(args)
        .env_clear()
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in ENV_ALLOW {
        if let Some(value) = std::env::var_os(key) {
            cmd.env(key, value);
        }
    }
    let mut child = cmd
        .spawn()
        .map_err(|err| NtBotError::Io(format!("spawn git: {err}")))?;
    fn drain<R: std::io::Read + Send + 'static>(pipe: Option<R>) -> std::thread::JoinHandle<Vec<u8>> {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(pipe) = pipe {
                // 读失败无所谓：stdout 缺了就是空输出，git 的成败看退出码。
                let _read: Option<usize> =
                    pipe.take(GIT_OUTPUT_CAP as u64).read_to_end(&mut buf).ok();
            }
            buf
        })
    }
    let stdout_handle = drain(child.stdout.take());
    let stderr_handle = drain(child.stderr.take());
    let deadline = Instant::now() + GIT_TIMEOUT;
    let status = loop {
        match child
            .try_wait()
            .map_err(|err| NtBotError::Io(format!("wait git: {err}")))?
        {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _killed: Option<()> = child.kill().ok();
                let _waited: Option<std::process::ExitStatus> = child.wait().ok();
                return Err(NtBotError::Io(format!(
                    "git timed out after {}s and was killed: {}",
                    GIT_TIMEOUT.as_secs(),
                    args.first().copied().unwrap_or("")
                )));
            }
            None => std::thread::sleep(Duration::from_millis(25)),
        }
    };
    let out = String::from_utf8_lossy(
        &stdout_handle
            .join()
            .unwrap_or_default(),
    )
    .into_owned();
    let err = String::from_utf8_lossy(
        &stderr_handle
            .join()
            .unwrap_or_default(),
    )
    .into_owned();
    if status.success() {
        Ok(out)
    } else {
        // git 的失败原因都在 stderr；把它带出去，UI 才能显示「为什么不行」。
        Err(NtBotError::Store(if err.trim().is_empty() {
            format!("git {} failed with {status}", args.first().copied().unwrap_or(""))
        } else {
            err.trim().to_owned()
        }))
    }
}

/// 改动清单（`git status --porcelain=v1 -z`）。
///
/// 用 `-z`（NUL 分隔）而不是默认的行分隔：路径里可能有换行 / 引号 /
/// 非 ASCII —— porcelain 的行模式会给这些路径加引号并转义，解回来容易错；
/// `-z` 不加引号也不转义，切分比行模式**更接近**真值。
///
/// 但本模块拿到的已经不是「原样字节」：[`run_git`] 把 stdout 收进 `Vec<u8>` 之后
/// 做 `String::from_utf8_lossy`，非 UTF-8 文件名（macOS 上完全合法）会被替换成
/// `U+FFFD`，此后怎么切分都不是原名 —— 界面显示一个打不开的路径，而 `git add`
/// 用的也正是这个被替换过的串。不能写「原样字节、切分即真值」：那会让人以为
/// 乱码文件名这条边已经不用管了。
pub fn status(workspace: &Path, subdir: &str) -> Result<Vec<GitFile>, NtBotError> {
    let root = repo_root(workspace, subdir)?;
    let raw = run_git(&root, &["status", "--porcelain=v1", "-z", "--untracked-files=all"])?;
    let mut out = Vec::new();
    // `-z` 下 rename/copy 是**两段**：`XY <new>\0<old>\0`。
    // 旧名那一段绝不能当独立状态行去解析（`split_at_checked(2)` 会把它
    // 的头两个字符当成 XY，凭空造出一条假记录），故显式吃掉它。
    // 只保留新名 —— 那才是「现在该打开的那个」。
    let chunks: Vec<&str> = raw.split('\0').filter(|part| !part.is_empty()).collect();
    let mut idx = 0usize;
    while idx < chunks.len() {
        let Some(chunk) = chunks.get(idx).copied() else { break };
        idx += 1;
        let Some((xy, rest)) = chunk.split_at_checked(2) else {
            continue;
        };
        if matches!(xy.chars().next(), Some('R' | 'C')) {
            // 吃掉紧随其后的旧名段。
            idx = idx.saturating_add(1);
        }
        // 分隔符恰好是**一个**空格：`XY␠path`。只剥一个，不能 `trim_start()`
        // —— 文件名开头带空格是合法的，`trim_start` 会把路径改错。
        let Some(path) = rest.strip_prefix(' ') else {
            continue;
        };
        let path = path.strip_prefix('"').unwrap_or(path);
        if path.is_empty() {
            continue;
        }
        let x = xy.chars().next().unwrap_or(' ');
        let y = xy.chars().nth(1).unwrap_or(' ');
        out.push(GitFile {
            path: path.to_owned(),
            xy: xy.to_owned(),
            staged: x != ' ' && x != '?' && x != '!',
            unstaged: y != ' ' && y != '?' && y != '!',
            untracked: xy == "??",
        });
    }
    // 未跟踪在前、已跟踪在后（用户最关心的是「新冒出来的」）；
    // 组内按路径序，输出稳定不跳。
    out.sort_by(|a, b| {
        b.untracked
            .cmp(&a.untracked)
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(out)
}

/// 单文件 diff（`staged` 为真则看暂存区）。
///
/// 未跟踪文件没有 diff 可看，改为回**全文**并在 `untracked_new` 标出，
/// 让 UI 用「新增文件」样式渲染而不是显示空白。
pub fn diff(
    workspace: &Path,
    subdir: &str,
    path: &str,
    staged: bool,
) -> Result<GitDiff, NtBotError> {
    let root = repo_root(workspace, subdir)?;
    // 两道门。`live` 是**唯一**真去读盘的那个路径，核验的就是它本身
    // ——门与落地点必须是同一个路径，否则核验的就是另一条路径。
    let live = jail_real(workspace, path)?;
    let tracked = run_git(
        &root,
        &["ls-files", "--error-unmatch", "--", path],
    )
    .is_ok();
    if !tracked {
        let body = std::fs::read_to_string(&live)
            .map_err(|err| NtBotError::Io(format!("{path}: {err}")))?;
        return Ok(GitDiff {
            text: body,
            untracked_new: true,
        });
    }
    let mut args: Vec<&str> = if staged {
        vec!["diff", "--cached", "--", path]
    } else {
        vec!["diff", "--", path]
    };
    let out = run_git(&root, &args)?;
    args.clear();
    Ok(GitDiff {
        text: out,
        untracked_new: false,
    })
}

/// diff 结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct GitDiff {
    /// 未跟踪文件时是**全文**（不是 diff），此时 `untracked_new` 为真。
    pub text: String,
    pub untracked_new: bool,
}

/// 提交历史（`path` 传空 = 整个仓库）。
pub fn log(
    workspace: &Path,
    subdir: &str,
    path: &str,
    limit: i64,
) -> Result<Vec<GitCommit>, NtBotError> {
    let root = repo_root(workspace, subdir)?;
    jail_real(workspace, path)?;
    let limit = limit.clamp(1, 200);
    // 用一条记录一行的自定义格式，避免为解析 `git log` 的默认多行格式写正则。
    let format = "%H%x1f%h%x1f%aI%x1f%an%x1f%s";
    let n_flag = format!("-n{limit}");
    let pretty = format!("--pretty=format:{format}");
    let mut args: Vec<&str> = vec!["log", "--no-color", &n_flag, &pretty];
    if !path.trim().is_empty() {
        args.push("--");
        args.push(path);
    }
    let out = run_git(&root, &args)?;
    let mut commits = Vec::new();
    for line in out.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split('\u{1f}');
        let (Some(id), Some(short), Some(at), Some(author), Some(subject)) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) else {
            continue;
        };
        commits.push(GitCommit {
            id: id.to_owned(),
            short: short.to_owned(),
            at: at.to_owned(),
            author: author.to_owned(),
            subject: subject.to_owned(),
        });
    }
    Ok(commits)
}

/// 暂存文件。
pub fn stage(workspace: &Path, subdir: &str, path: &str) -> Result<(), NtBotError> {
    let root = repo_root(workspace, subdir)?;
    jail_real(workspace, path)?;
    if path.trim().is_empty() {
        return Err(NtBotError::Invalid("stage needs a path".to_owned()));
    }
    // 不吞错：暂存失败必须让 UI 知道。`.ok()` 掉的话，用户点了暂存
    // 什么都没发生、界面还显示「已暂存」—— 那是骗人。
    let _added: Option<String> = Some(run_git(&root, &["add", "--", path])?);
    Ok(())
}

/// 取消暂存（`git restore --staged`；老 git 回退 `reset`）。
pub fn unstage(workspace: &Path, subdir: &str, path: &str) -> Result<(), NtBotError> {
    let root = repo_root(workspace, subdir)?;
    jail_real(workspace, path)?;
    if path.trim().is_empty() {
        return Err(NtBotError::Invalid("unstage needs a path".to_owned()));
    }
    if run_git(&root, &["restore", "--staged", "--", path]).is_ok() {
        return Ok(());
    }
    // `restore` 不认老 git（<2.23）→ 回退 `reset HEAD --`；两条都失败才算真失败。
    let _fallback: Option<String> = Some(run_git(&root, &["reset", "HEAD", "--", path])?);
    Ok(())
}

/// 丢弃工作区改动（`git checkout -- <path>`）。
///
/// **不可逆**：这一步真的会把用户没提交的编辑扔掉。故：
/// 1. 拒空路径；
/// 2. 拒未跟踪文件（`checkout` 对它无意义，用户想要的是「删掉」，
///    那是另一条命令、另一道确认）。
///
/// 给调用方看的警示就是函数名里的 `discard`（早先这里第三条写的是
/// 「函数名带 `_discarding` 后缀」—— 那个后缀并不存在，照着它 grep 一条都找不到）。
/// 别再许诺一个命名约定：真要更硬的信号得靠类型或 IPC 层的确认弹窗。
pub fn discard_workspace_changes(
    workspace: &Path,
    subdir: &str,
    path: &str,
) -> Result<(), NtBotError> {
    let root = repo_root(workspace, subdir)?;
    jail_real(workspace, path)?;
    if path.trim().is_empty() {
        return Err(NtBotError::Invalid("revert needs a path".to_owned()));
    }
    if run_git(&root, &["ls-files", "--error-unmatch", "--", path]).is_err() {
        return Err(NtBotError::Invalid(format!(
            "'{path}' is untracked; checkout would do nothing (deleting is a separate action)"
        )));
    }
    let _out: Option<String> = Some(run_git(&root, &["checkout", "--", path])?);
    Ok(())
}

/// 提交（`no_verify` 为真则 `--no-verify` 跳过 hook）。
///
/// 空消息拒 —— `--allow-empty-message` 那条路不开：没有消息的提交
/// 在 review 里是纯噪音。
pub fn commit(
    workspace: &Path,
    subdir: &str,
    message: &str,
    no_verify: bool,
) -> Result<GitCommitResult, NtBotError> {
    let root = repo_root(workspace, subdir)?;
    let message = message.trim();
    if message.is_empty() {
        return Err(NtBotError::Invalid("commit message is empty".to_owned()));
    }
    if message.chars().count() > 4000 {
        return Err(NtBotError::Invalid(
            "commit message exceeds 4000 chars".to_owned(),
        ));
    }
    let mut args: Vec<&str> = vec!["commit"];
    if no_verify {
        args.push("--no-verify");
    }
    args.extend(["-m", message]);
    run_git(&root, &args)?;
    // 提交成功后拿一下刚落的那条的短 id + 标题（UI 要立刻显示）。
    let head = log(&root, "", "", 1)?;
    Ok(GitCommitResult {
        short: head.first().map(|c| c.short.clone()).unwrap_or_default(),
        subject: head.first().map(|c| c.subject.clone()).unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 建一个真仓库（需要 `git` 在场；不在就跳过，不假装测过）。
    fn repo(case: &str) -> Option<PathBuf> {
        if !available() {
            return None;
        }
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-git-test-{}", case));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let git = |args: &[&str]| -> String {
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(args)
                .env_clear()
                .env("PATH", std::env::var_os("PATH").unwrap_or_default())
                .env("HOME", std::env::var_os("HOME").unwrap_or_default())
                .env("GIT_TERMINAL_PROMPT", "0")
                .output()
                .expect("git run");
            String::from_utf8_lossy(&out.stdout).into_owned()
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "test@neobot.local"]);
        git(&["config", "user.name", "neobot test"]);
        git(&["config", "commit.gpgsign", "false"]);
        std::fs::write(dir.join("a.txt"), "one\n").expect("write");
        git(&["add", "-A"]);
        git(&["-c", "commit.gpgsign=false", "commit", "-q", "-m", "init"]);
        Some(dir)
    }

    #[test]
    fn repo_root_walks_up_and_stays_in_workspace() {
        let Some(dir) = repo("root") else { return };
        std::fs::create_dir_all(dir.join("deep/nested")).expect("mkdir");
        let got = repo_root(&dir, "deep/nested").expect("root");
        assert_eq!(
            got.canonicalize().ok(),
            dir.canonicalize().ok(),
            "应上溯到仓库根"
        );
        // 非仓库目录报错，不静默返回工作区根。
        let plain = crate::nt_testutil::temp_dir("git-test-notarepo");
        let _ = std::fs::remove_dir_all(&plain);
        std::fs::create_dir_all(&plain).expect("mkdir");
        assert!(repo_root(&plain, "").is_err());
        let _ = std::fs::remove_dir_all(&plain);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn repo_root_refuses_to_escape_the_workspace() {
        let Some(dir) = repo("escape") else { return };
        // 工作区里放一个指向外部仓库的软链接 → 上溯核验必须拦下。
        let outside = repo("escape-outside");
        let Some(outside) = outside else {
            let _ = std::fs::remove_dir_all(&dir);
            return;
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&outside, dir.join("link")).expect("symlink");
            let err = repo_root(&dir, "link").expect_err("symlinked repo must be rejected");
            assert!(err.to_string().contains("escapes workspace"), "{err}");
        }
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn status_separates_untracked_and_tracks_both_stages() {
        let Some(dir) = repo("status") else { return };
        // 已跟踪但改了（未暂存）。
        std::fs::write(dir.join("a.txt"), "one modified\n").expect("write");
        // 新文件（未跟踪）。
        std::fs::write(dir.join("new.txt"), "brand new\n").expect("write");
        let files = status(&dir, "").expect("status");
        // 未跟踪排在前。
        assert_eq!(files.first().map(|f| f.path.as_str()), Some("new.txt"));
        assert!(files.first().is_some_and(|f| f.untracked));
        let a = files.iter().find(|f| f.path == "a.txt").expect("a.txt");
        assert!(a.unstaged && !a.staged && !a.untracked);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_of_untracked_returns_the_whole_file_flagged() {
        let Some(dir) = repo("diffnew") else { return };
        std::fs::write(dir.join("fresh.txt"), "line1\nline2\n").expect("write");
        let got = diff(&dir, "", "fresh.txt", false).expect("diff");
        assert!(got.untracked_new);
        assert!(got.text.contains("line2"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_of_tracked_shows_the_change() {
        let Some(dir) = repo("diff") else { return };
        std::fs::write(dir.join("a.txt"), "one changed\n").expect("write");
        let got = diff(&dir, "", "a.txt", false).expect("diff");
        assert!(!got.untracked_new);
        assert!(got.text.contains("-one"), "{}", got.text);
        assert!(got.text.contains("+one changed"), "{}", got.text);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_of_staged_reads_the_index() {
        let Some(dir) = repo("diffstaged") else { return };
        std::fs::write(dir.join("a.txt"), "staged change\n").expect("write");
        stage(&dir, "", "a.txt").expect("stage");
        let staged = diff(&dir, "", "a.txt", true).expect("staged diff");
        assert!(staged.text.contains("+staged change"), "{}", staged.text);
        // 未暂存视角此时应是空的（内容全进索引了）。
        let unstaged = diff(&dir, "", "a.txt", false).expect("unstaged diff");
        assert!(unstaged.text.trim().is_empty(), "{}", unstaged.text);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unstage_moves_changes_back_to_the_worktree() {
        let Some(dir) = repo("unstage") else { return };
        std::fs::write(dir.join("a.txt"), "changed\n").expect("write");
        stage(&dir, "", "a.txt").expect("stage");
        assert!(status(&dir, "").expect("status").iter().any(|f| f.path == "a.txt" && f.staged));
        unstage(&dir, "", "a.txt").expect("unstage");
        let files = status(&dir, "").expect("status");
        let a = files.iter().find(|f| f.path == "a.txt").expect("a.txt");
        assert!(!a.staged && a.unstaged);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discard_refuses_untracked_and_restores_tracked() {
        let Some(dir) = repo("discard") else { return };
        std::fs::write(dir.join("junk.txt"), "junk\n").expect("write");
        // 未跟踪：checkout 无意义 → 拒（删是另一条命令）。
        let err = discard_workspace_changes(&dir, "", "junk.txt").expect_err("untracked");
        assert!(err.to_string().contains("untracked"), "{err}");
        // 已跟踪：真能还原。
        std::fs::write(dir.join("a.txt"), "wrecked\n").expect("write");
        discard_workspace_changes(&dir, "", "a.txt").expect("discard");
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).ok().as_deref(),
            Some("one\n"),
            "丢弃后应回到 HEAD 内容"
        );
        // 空路径拒。
        assert!(discard_workspace_changes(&dir, "", "").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn commit_records_and_no_verify_is_honoured() {
        let Some(dir) = repo("commit") else { return };
        std::fs::write(dir.join("a.txt"), "committed\n").expect("write");
        stage(&dir, "", "a.txt").expect("stage");
        let got = commit(&dir, "", "  改了点东西  ", true).expect("commit");
        assert!(!got.short.is_empty());
        assert_eq!(got.subject, "改了点东西", "message 应被 trim");
        // 提交后工作区干净。
        assert!(status(&dir, "").expect("status").is_empty());
        // 空消息拒。
        assert!(commit(&dir, "", "   ", false).is_err());
        // 超长消息拒。
        let long = "x".repeat(4001);
        assert!(commit(&dir, "", &long, false).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn log_reads_history_and_can_filter_by_path() {
        let Some(dir) = repo("log") else { return };
        std::fs::write(dir.join("a.txt"), "two\n").expect("write");
        stage(&dir, "", "a.txt").expect("stage");
        commit(&dir, "", "第二次提交", true).expect("commit");
        std::fs::write(dir.join("b.txt"), "other\n").expect("write");
        stage(&dir, "", "b.txt").expect("stage");
        commit(&dir, "", "第三次提交", true).expect("commit");

        let all = log(&dir, "", "", 10).expect("log");
        assert_eq!(all.len(), 3);
        // 新的在前。
        assert_eq!(all.first().map(|c| c.subject.as_str()), Some("第三次提交"));
        assert!(!all.first().is_some_and(|c| c.id.is_empty()));

        // 按路径过滤：a.txt 出现在 init + 第二次提交两笔里（第三次只动 b.txt）。
        let only_a = log(&dir, "", "a.txt", 10).expect("log a");
        let subjects: Vec<&str> = only_a.iter().map(|c| c.subject.as_str()).collect();
        assert_eq!(subjects, vec!["第二次提交", "init"]);
        // limit 收敛。
        assert_eq!(log(&dir, "", "", 1).expect("log 1").len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_path_taking_call_jails_its_target() {
        let Some(dir) = repo("jail") else { return };
        // 越狱路径在 status/diff/log/stage/unstage/discard 上都必须被拒。
        assert!(diff(&dir, "", "../../etc/passwd", false).is_err());
        assert!(log(&dir, "", "/etc/passwd", 5).is_err());
        assert!(stage(&dir, "", "../escape.txt").is_err());
        assert!(unstage(&dir, "", "~/x").is_err());
        assert!(discard_workspace_changes(&dir, "", "a/../../b").is_err());
        // 合法路径照过（否则上面的拒可能只是「全都不работает」）。
        assert!(diff(&dir, "", "a.txt", false).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn non_repo_directory_reports_a_useful_error() {
        let plain = crate::nt_testutil::temp_dir("git-test-plain");
        let _ = std::fs::remove_dir_all(&plain);
        std::fs::create_dir_all(&plain).expect("mkdir");
        std::fs::write(plain.join("f.txt"), "x").expect("write");
        let err = status(&plain, "").expect_err("not a repo");
        assert!(err.to_string().contains("no git repository"), "{err}");
        let _ = std::fs::remove_dir_all(&plain);
    }

    /// 断 `err` 是 `Denied{rule}` 且 rule 就是 `workspace-jail`
    /// （与 `repo_root` 越界时同一条规则，UI 才能按同一条处理）。
    #[track_caller]
    fn assert_jail_denied(err: &NtBotError, what: &str) {
        match err {
            NtBotError::Denied { rule, reason } => {
                assert_eq!(rule, "workspace-jail", "{what}: rule 不对 ({reason})");
                assert!(!reason.is_empty(), "{what}: Denied 必须带 reason（要能显示）");
            }
            other => panic!("{what}: 应为 Denied，实得 {other:?}"),
        }
    }

    /// **核心回归**：工作区内一个指向外部的软链接 + 一条**不含 `..`** 的相对路径
    /// ⇒ 五个 pathspec 落地点全部拒，且错误是 `Denied{workspace-jail}`。
    ///
    /// 造法上先断「第一道门真的会放行」——不然这个测试可能只是把
    /// `../` 那种老拦截又测了一遍，而缺口恰恰在**纯词法门之后**。
    #[test]
    fn symlinked_pathspec_cannot_escape_the_workspace() {
        let Some(dir) = repo("symlink") else { return };
        // 工作区外：一个装着「秘密」的目录，扮演 /etc 的角色。
        let outside = crate::nt_testutil::temp_dir("git-test-symlink-outside");
        let _ = std::fs::remove_dir_all(&outside);
        std::fs::create_dir_all(&outside).expect("mkdir outside");
        std::fs::write(outside.join("secret.txt"), "TOP SECRET\n").expect("write outside");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            // 造不出来就 expect 让测试**炸**，不静默跳过（darwin 必然支持）。
            symlink(&outside, dir.join("escape")).expect("symlink dir -> outside");
            symlink(outside.join("secret.txt"), dir.join("secret.txt"))
                .expect("symlink file -> outside");

            // 前置条件：纯词法门（`check_rel`）对这些路径**放行** ——
            // 若它就拒了，下面所有 `Denied` 都归功于第一道门，测不到缺口。
            for rel in ["escape/secret.txt", "secret.txt"] {
                assert!(
                    crate::nt_workspace::check_rel(rel).is_ok(),
                    "'{rel}' 应过纯词法门：它没有 ..、没有 / 开头 —— 缺口正在此处"
                );
                assert!(
                    jail_join(&dir, rel).is_ok(),
                    "'{rel}' 应能 jail_join 出来（拼串层面完全合法）"
                );
            }

            // 目录软链接：`diff` 的未跟踪分支会真去 `read_to_string` 落盘。
            let err = diff(&dir, "", "escape/secret.txt", false)
                .expect_err("指向外部的目录软链接必须被拒");
            assert_jail_denied(&err, "diff escape/secret.txt");
            // 末段就是软链接（symlink_metadata 算它存在 → canonicalize 现形）。
            let err = diff(&dir, "", "secret.txt", false)
                .expect_err("指向外部的文件软链接必须被拒");
            assert_jail_denied(&err, "diff secret.txt");

            // 其余四个落地点同样拒。
            let err = stage(&dir, "", "escape/secret.txt").expect_err("stage");
            assert_jail_denied(&err, "stage");
            let err = unstage(&dir, "", "escape/secret.txt").expect_err("unstage");
            assert_jail_denied(&err, "unstage");
            let err = log(&dir, "", "escape/secret.txt", 5).expect_err("log");
            assert_jail_denied(&err, "log");
            let err = discard_workspace_changes(&dir, "", "escape/secret.txt")
                .expect_err("discard");
            assert_jail_denied(&err, "discard");

            // 门禁之后**什么都没发生**：外部文件没被动过，也没被读进 diff。
            assert_eq!(
                std::fs::read_to_string(outside.join("secret.txt")).ok().as_deref(),
                Some("TOP SECRET\n"),
                "外部文件应原样"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&outside);
    }

    /// 反面：第二道门**不是**「见路径就拒」。
    /// 存在的普通文件、不存在的目标（`stage` 一个还没创建的文件）、
    /// 指向工作区**内部**的软链接 —— 都必须照过。
    #[test]
    fn second_door_does_not_kill_legitimate_paths() {
        let Some(dir) = repo("door-ok") else { return };
        // 已存在的普通文件。
        assert!(jail_real(&dir, "a.txt").is_ok(), "工作区内普通文件应放行");
        // **不存在的目标**：canonicalize 会失败，但这**不是**越界
        // （`stage` 一个还没创建的文件正是这种）——不能误杀。
        assert!(jail_real(&dir, "brand-new.txt").is_ok(), "不存在的目标不该判越界");
        // 连父目录都不存在的更深路径：核验退到已存在祖先（= 工作区）。
        assert!(jail_real(&dir, "not/there/yet.txt").is_ok(), "缺失的多段同理");
        // 空路径 = 整个仓库（`log("")`），过门且无副作用。
        assert!(jail_real(&dir, "").is_ok(), "空路径即工作区根本身");

        // 端到端：正常 stage 仍然可用。
        std::fs::write(dir.join("fresh.txt"), "ok\n").expect("write");
        stage(&dir, "", "fresh.txt").expect("正常 stage 不能被误杀");
        // 正常 diff 仍然可用。
        assert!(diff(&dir, "", "a.txt", false).is_ok(), "正常 diff 不能被误杀");
        // 不存在的 pathspec：报错来自 git，**不是**门拒的。
        let err = stage(&dir, "", "not/there/yet.txt").expect_err("git 会拒不存在的 pathspec");
        assert!(
            !matches!(err, NtBotError::Denied { .. }),
            "不存在 ≠ 越界，门不该拦：{err}"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            // 指向工作区**内部**的软链接：真身在工作区内 ⇒ 放行
            //（第二道门判的是「真实路径在哪」，不是「有没有软链接」）。
            symlink(dir.join("a.txt"), dir.join("inner.txt")).expect("symlink inside");
            assert!(jail_real(&dir, "inner.txt").is_ok(), "区内软链接应放行");
            let got = diff(&dir, "", "inner.txt", false).expect("区内软链接 diff");
            assert!(got.untracked_new && got.text.contains("one"));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

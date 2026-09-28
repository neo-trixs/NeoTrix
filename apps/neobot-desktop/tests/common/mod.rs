//! 冒烟测试共用夹具：把 `NEOBOT_DATA_DIR` 指到一次性目录。
//!
//! ## 为什么需要它
//!
//! `nt_commands` 里的命令**自己**调 `load_config()` → `NeobotConfig::from_env()`，
//! 没有「传 config 进去」的缝（见 `nt_commands.rs:112`）。这意味着：
//! 要在测试里调这些命令，唯一办法就是**改进程环境**——而环境是进程全局的。
//!
//! 所以规则是：
//!
//! 1. **每个 `tests/*.rs` 是独立测试二进制**（cargo 天然如此），
//!    于是「一个二进制 = 一个数据目录」，互不干扰；
//! 2. 目录用 `Once` 惰性建，且**所有线程都从同一个入口进来**（`data_dir()`），
//!    谁都不会在别人读到一半时改环境 —— 竞态只可能发生在 Once 内部，那被 Once 挡住了；
//! 3. 外部已设 `NEOBOT_DATA_DIR` 时**尊重它**（`nt_smoke.sh` 就是这么用的），
//!    绝不在有值时覆盖。
//!
//! ## 目录落在哪
//!
//! 落在**仓库内** `<workspace>/target/nt-smoke/<测试二进制名>/`，不落 `/tmp`：
//! - `target/` 已被 `.gitignore` 忽略（第 1 行），不会污染 git status；
//! - 每次初始化先 `remove_dir_all` 再建，故**可重复运行**且**不留残骸**；
//! - 不写用户 home —— 测试绝不能碰 `~/.neobot/`，那是真人数据。
//!
//! 为什么不落 `/tmp`：那儿的残留没人回收，而本仓库的规矩是「不留垃圾」；
//! 且落在 `target/` 下，`cargo clean` 一条命令就干净了。
//!
//! ## 二进制名的来源（踩过的坑）
//!
//! 第一版用 `file!()` 取名字，**是错的**：`file!()` 在 `common/mod.rs` 里
//! 恒为 `tests/common/mod.rs`，四个测试二进制拿到的是**同一个**名字 ——
//! 于是它们共用一个数据目录。第一版偏偏还写了一句「带二进制名：不同
//! `tests/*.rs` 各自一份，互不踩」，注释与行为相反，是最坏的一种错。
//! （cargo 默认串行跑各二进制，所以它没炸；炸的是「隔离」这个前提本身。）
//!
//! 改用 `argv[0]`：**运行时**拿到的就是本进程自己的可执行文件名，
//! 四个二进制必然不同。这是不需要每个测试文件各写一行注册的地方。

// 每个测试二进制只用夹具的一部分（`nt_smoke_files` 用不到造 DM 的 helper），
// 未用到的那些若报 dead_code 噪声，就得在每个文件里各写一次 `#[allow]`。
// 用 `expect` 而非 `allow`：本仓库开了 `clippy::allow_attributes`，
// 且 dead_code 在**每个**二进制里都确实会触发（各有各的未用 helper），
// 所以「预期它触发」是准确陈述，不会有 unfulfilled expectation。
#![expect(dead_code, reason = "夹具按需取用：每个测试二进制只 import 自己那一部分")]
use std::path::{Path, PathBuf};
use std::sync::Once;

static INIT: Once = Once::new();

/// 惰性准备好进程环境，返回 `NEOBOT_DATA_DIR` 的值。
///
/// **每个测试都必须在动手前先调它**（哪怕不碰文件也要调），因为它是
/// 「环境已被钉死」的唯一保证点。幂等：第一次调用做实际初始化，之后直接返回。
pub fn data_dir() -> PathBuf {
    INIT.call_once(init_env);
    PathBuf::from(std::env::var("NEOBOT_DATA_DIR").expect("夹具未设 NEOBOT_DATA_DIR"))
}

/// 工作区目录（`<data_dir>/workspace`，与 `nt_config::from_env` 的口径一致）。
pub fn workspace_dir() -> PathBuf {
    data_dir().join("workspace")
}

/// 打开一份连到测试数据目录的 store（绕过 shell，直接用核心 API 造前置数据）。
///
/// 为什么要有这个：**要验「壳把参数接对了」，就得先有数据**。
/// 数据由核心 API 写（`record_change` / `save_task`），断言打在命令的输出上——
/// 于是「数据在、命令读得到」这条链才真的被走过一遍。
pub fn open_store() -> neotrix_neobot::NeobotStore {
    let config = neotrix_neobot::NeobotConfig::from_env().expect("配置应可从环境构造");
    let path = config.db_path();
    neotrix_neobot::NeobotStore::open(&path.to_string_lossy()).expect("测试库应可打开")
}

/// 相对工作区根写一个文件（先建父目录），返回其相对路径。
pub fn write_in_workspace(rel: &str, content: &str) -> String {
    let abs = workspace_dir().join(rel);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).expect("应可建父目录");
    }
    std::fs::write(&abs, content).expect("应可写工作区文件");
    rel.to_owned()
}

/// 唯一 id（避免同一测试二进制内多个测试的 task/convo 串味）。
pub fn uniq(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

fn init_env() {
    // 引擎钉死 echo：测试**不许**碰网络（硬约束）。若宿主机环境里恰好
    // 留了 NEOBOT_ENGINE=http，from_env 会去解析 HTTP 配置，那不是本测试的职责。
    // 这里是无条件覆盖 —— 测试环境的确定性优先于「尊重用户设置」。
    std::env::set_var("NEOBOT_ENGINE", "echo");

    if std::env::var_os("NEOBOT_DATA_DIR").is_none() {
        let root = workspace_root().join("target").join("nt-smoke");
        // 带二进制名：不同 `tests/*.rs` 各自一份，互不踩。
        let dir = root.join(bin_tag());
        // 先清后建 —— 可重复运行的唯一依据。
        // 删不掉不算失败（目录本来就不存在），故显式吞掉 Err 而不是 let _：
        // 后者会被 must_use 判为「非绑定 let」，等于把「我确实不要这个值」
        // 写成了噪声。
        drop(std::fs::remove_dir_all(&dir));
        std::fs::create_dir_all(&dir).expect("应可建冒烟临时目录");
        std::env::set_var("NEOBOT_DATA_DIR", &dir);
    }
}

/// 工作区根（`<repo>/`）：`CARGO_MANIFEST_DIR` 是 `apps/neobot-desktop`。
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("apps/neobot-desktop 应在仓库根下两层")
        .to_path_buf()
}

/// 本测试二进制的短名（`nt_smoke_files-195668c3…` → `nt_smoke_files`）。
///
/// 取自 `argv[0]`：那是**本进程**的可执行文件名，四个测试二进制必然不同
/// （见头注释「二进制名的来源」—— `file!()` 在这里给不出区分）。
/// 哈希后缀只在「后缀全十六进制且够长」时才剥，避免把带短横线的正常名字
/// 截断；剥不掉就整个用，目录名不必好看，只要唯一。
fn bin_tag() -> String {
    let raw = std::env::args()
        .next()
        .as_deref()
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("smoke-bin")
        .to_owned();
    match raw.rsplit_once('-') {
        Some((stem, hash))
            if hash.len() >= 8 && hash.chars().all(|c| c.is_ascii_hexdigit()) =>
        {
            stem.to_owned()
        }
        _ => raw,
    }
}

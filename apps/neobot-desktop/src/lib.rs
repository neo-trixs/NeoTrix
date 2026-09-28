//! NeoBot 桌面壳的**可测试切面**（lib 目标）。
//!
//! ## 为什么需要这个文件
//!
//! `#[tauri::command]` 就是**普通 Rust 函数**——它们不依赖 `AppHandle`、
//! 不依赖 Tauri 运行时（除少数几个），参数全是 `String` / `Option<…>`。
//! 也就是说 IPC 面**本来就可以被直接调用**，唯一挡住测试的是 crate 形状：
//! 本 crate 过去只有 `[[bin]]`，`tests/` 集成测试**没有可 import 的库**。
//!
//! 历史教训（这正是本文件存在的理由）：`taskId: ""` 那个缺陷里，
//! 核心函数 `tally_task_paths` 完全正确，错的是**前端把空串传了进来**——
//! 而核心层 260 个测试一个都抓不到，因为缺陷发生在「参数绑定」这一层。
//! 参数绑定只能由「真的调一次这个命令」来验，不能靠读核心代码推。
//!
//! ## 为什么是**新增**而不是搬迁
//!
//! 本文件只做一件事：`pub mod nt_commands;`。`main.rs` 里的
//! `mod nt_commands;` **原样保留**——于是同一份命令源码被编译进两个目标：
//! bin（真机跑的那份）与 lib（测试链的那份）。
//!
//! 代价：这份命令代码编两遍（约 2500 行，相对 tauri 依赖可忽略）。
//! 换来的是：`main.rs` 一行不改、**没有任何命令签名或函数体被触碰**，
//! 改动面收缩到「多一个文件 + Cargo.toml 多三行」。
//!
//! 不要试图让 `main.rs` 改用 `neobot_desktop::nt_commands` 来「消除重复」
//! ——那会把注册表（`generate_handler!`）的路径全部搬家，属于结构性改动，
//! 收益（省几秒编译）远小于风险（注册表是 100 条命令的唯一真相源）。
//!
//! ## 门记录
//!
//! - 2026-09-28 新建。为 `apps/neobot-desktop/tests/nt_smoke_*.rs` 提供 import 面。
//!   已验证：`cargo test -p neobot-desktop --test nt_smoke_files` 等四套冒烟全绿。

#![forbid(unsafe_code)]

pub mod nt_commands;

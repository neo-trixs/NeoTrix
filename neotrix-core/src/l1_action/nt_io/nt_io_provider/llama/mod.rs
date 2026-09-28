//! LLaMA 本地推理 — **重导出**, 实现已下沉到 `neotrix-neobot::nt_llama`.
//!
//! 下沉原因见 `neotrix-neobot/src/nt_llama.rs` 的模块文档: 桌面 App 只依赖
//! 轻量的 `neotrix-neobot`（7 依赖）而非 `neotrix-core`（214 依赖），所以实现
//! 必须住在双方共同依赖的地方。**这里保留本文件只为兼容既有引用路径** ——
//! 任何新代码请直接用 `neotrix_neobot::nt_llama::*`。
//!
//! 历史教训: `src-tauri`（已归档）当初为了不依赖 neotrix-core 而**复制**了一份
//! 进程启动，那份副本缺 `--jinja` 且 ctx 写死 4096，即"装完开不了话"的根因。
//! 复制实现是 bug 的来源，不是解法。

pub use neotrix_neobot::nt_llama::*;

/// 兼容旧路径 `...::llama::llama_process::*`。
pub mod llama_process {
    pub use neotrix_neobot::nt_llama::*;
}

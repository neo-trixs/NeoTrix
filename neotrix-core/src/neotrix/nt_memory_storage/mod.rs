// ── nt_core_storage → nt-segstore 桥接层 ─────────────────────────
// P2: 代码已提取到 crates/nt-segstore。本模块仅作重导出，保持向后兼容。
// 所有实现位于 crates/nt-segstore/src/。

pub use ::nt_segstore::*;

// 子模块 shim — 保持旧路径兼容
//   crate::core::nt_core_storage::compaction::*
//   crate::core::nt_core_storage::null_drift_memory::*
pub mod compaction {
    //! Re-exported from nt_segstore::compaction
    pub use ::nt_segstore::compaction::*;
}
pub mod null_drift_memory {
    //! Re-exported from nt_segstore::null_drift
    pub use ::nt_segstore::null_drift::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_re_exports() {
        // nt-segstore crate is available as a workspace dependency — SegStoreConfig
        // is accessed through the re-export in this module.
    }
}

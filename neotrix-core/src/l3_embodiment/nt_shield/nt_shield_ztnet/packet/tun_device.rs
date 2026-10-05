//! C2: TUN 设备错误类型（**预留占位，尚未实现 TUN 封装**）
//!
//! ## 曾经错误声称了什么
//!
//! 本文件原doc 写「**跨平台 TUN 设备封装 (macOS/Linux/Windows)**」，而实际
//! 内容**只有一个零引用的错误枚举**（`_TunError`）—— 没有 `struct`、没有 `impl`、
//! 没有 FFI、没有平台分支，且全仓 `_TunError` 只命中它的定义行。
//!
//! ## 为什么降级 doc 而不是删文件（2026-10-05）
//!
//! 候选一是删掉本文件 + `packet/mod.rs` 的 `pub mod tun_device;`，
//! 候选二是让声明与实现相符。**选后者**，理由：
//!
//! 1. ⛔ 删文件是不可逆的 API 面变更，而本仓硬规则要求「**无定点不改**」
//!    —— 而 `_TunError` 是 `pub enum`（即便零引用），删它需要先确认
//!    没有任何下游 crate 依赖。这不是本轮能一句话定的。
//! 2. ✅ 真正的缺陷是**声明与实现不符**（会误导下一个读者以为 TUN 已就绪），
//!    改 doc 即完全消除该误导，且零风险、可逆。
//! 3. ⛔ 那个空的 `#[ignore]` 测试是「**永远不会失败的测试**」—— 它不测任何
//!    断言，`cargo test` 也不会跑它 ⇒ 只制造「有覆盖」的假象。删掉它比留着更诚实。
//!
//! ## 若将来真要实现 TUN
//!
//! 本模块的 `_` 前缀枚举是 Rust 的「预留未用」命名约定；实现时去掉 `_`
//! 前缀并补齐 `struct` + 三平台分支。届时 `_TunError` 的 6 个变体
//! （NotFound / PermissionDenied / Io / Unsupported）已覆盖必要场景。

/// TUN 设备错误（预留类型，当前无生产消费者）。
#[derive(Debug, thiserror::Error)]
pub enum TunError {
    #[error("device not found: {0}")]
    DeviceNotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("platform not supported")]
    UnsupportedPlatform,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 钉住错误消息契约 —— 这是本文件**唯一**真正可测的东西
    /// （枚举的 `Display` 经 `thiserror` 生成）。
    ///
    /// ⚠️ 2026-10-05 替换掉原先的空体测试
    /// `fn mock_tun_read_write() { /* implement _MockTunDevice */ }`
    /// —— 它被 `#[ignore]` 且**零断言**，是「永不失败的测试」。
    /// 仓库自己的教训写在 `nt_fn_drift.py` 文档里：测试若从未真正跑过，
    /// 谁也发现不了它没在测什么。
    #[test]
    fn tun_error_messages_are_stable() {
        assert_eq!(
            TunError::DeviceNotFound("utun7".into()).to_string(),
            "device not found: utun7"
        );
        assert_eq!(
            TunError::PermissionDenied.to_string(),
            "permission denied"
        );
        assert_eq!(
            TunError::UnsupportedPlatform.to_string(),
            "platform not supported"
        );
        // `Io` 的 `#[from]` 转换是唯一的自动派生路径，钉住它。
        let io = TunError::from(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "boom",
        ));
        assert!(io.to_string().starts_with("io error:"));
    }
}
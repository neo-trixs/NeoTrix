//! C1: SANS-IO Protocol Engine — 零 IO 纯状态机协议引擎层
//!
//! 所有协议引擎实现统一 `NtProtocol` trait，由外部事件循环驱动。

pub mod traits;

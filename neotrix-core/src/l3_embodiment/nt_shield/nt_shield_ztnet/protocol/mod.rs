//! C1: SANS-IO Protocol Engine — 零 IO 纯状态机协议引擎层
//!
//! 所有协议引擎实现统一 `NtProtocol` trait，由外部事件循环驱动。

pub mod traits;
pub mod transmit;
pub mod event_loop;
pub mod client_state;
pub mod gateway_state;
pub mod peer_demux;

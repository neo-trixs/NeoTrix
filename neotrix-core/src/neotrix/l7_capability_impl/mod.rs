//! # L7 — Capability (能力层)
//!
//! 能力注册、调度、成熟度进化、星脉通信协议。
//! 桥接模块: 将由 core::l7_capability 的导出类型通过 neotrix:: 命名空间公开。
//!
//! NT-CORE 推理核通过 L7 路由发现和调用所有能力。
//! L7 不执行能力，只调度 —— 4 道大过滤器：权限→预算→熔断→谦逊。

pub use crate::core::l7_capability::*;

//! NeoTrix Abilities — 纯逻辑能力库（零渲染依赖，引擎与产品复用）.
//!
//! 分层（能力边界）：
//! - 数据层（StS 系）：`schema` 卡牌等数据结构 + RON 加载；`query` 图鉴筛选；
//!   `towergen` 爬塔图；`events_parse` 事件前置；`rich` 富文本；`powers_math`
//!   能力结算；`store` 全量服务层。
//! - 算法层（通用）：`dungeongen` 地牢生成；`nt_tuning` 数值/保底/难度；
//!   `nt_net` 权威房间 + 环回 + 序列 guard；`nt_fov` 对称阴影 + 几何 LOS；
//!   `nt_flow` Dijkstra 流场 + A*；`nt_utility` 效用决策（打分/门禁/迟滞/fail-open）。
//!
//! 约定：纯函数优先，RNG 由调用方种子传入；无 wall-clock；无渲染类型。

#![forbid(unsafe_code)]

#![forbid(unsafe_code)]

pub mod schema;
pub mod query;
pub mod towergen;
pub mod events_parse;
pub mod rich;
pub mod powers_math;
pub mod store;
pub mod dungeongen;
pub mod nt_tuning;
pub mod nt_net;
pub mod nt_fov;
pub mod nt_flow;
pub mod nt_utility;

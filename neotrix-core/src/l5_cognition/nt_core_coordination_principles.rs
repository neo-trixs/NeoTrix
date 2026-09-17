//! P2: Shape-Level Coordination Principles — 多 agent 协调的形状级规则
//!
//! 吸收 cumora COORDINATION.md GLANCE_YIELD_RULES + §5b-5e 防碰撞层。
//!
//! 核心教训: **不要为每个 observed bug 加 scenario-specific 规则**。
//! 用 shape-level 原则覆盖所有场景, 而不是枚举 counting/chain/vote 各写一套。
//!
//! 5 条形状级原则 (cumora GLANCE_YIELD_RULES 精炼):
//! 1. 人类点名特定人 → 那人回答, 其他人旁观
//! 2. 从真实已发布状态回复, 不要猜 peer 的行为
//! 3. 乐观发布; server 是安全网 (HELD 机制)
//! 4. 不重复 peer; 按任务项数而非人数衡量完成
//! 5. 不 claim chat turn 或 game slot

use serde::{Deserialize, Serialize};

/// 协调原则 — shape-level 规则, 不绑定具体场景。
///
/// 对齐 cumora "principle-only, no scenario enumeration" 纪律:
/// 每条原则是**形状描述**, 而非 "counting 时 post N+1" 这样的场景规则。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoordinationPrinciple {
    /// P1: 人类点名 → 被点名者响应, 其余人旁观。
    /// 形状: 人类消息含具体人名/角色名 → 软 1:1 寻址。
    HumanAddressesSpecific,

    /// P2: 从真实已发布状态决策, 不从 "我在队列中的位置" 推理。
    /// 形状: 任务有顺序 → 看最后一条已发布内容, 不猜 peer 行为。
    ReplyFromPostedState,

    /// P3: 乐观发布; 冲突由 server 端 safety net 处理。
    /// 形状: 写操作有竞态可能 → 先提交, 被 hold 则重新读取再提交。
    PostOptimistically,

    /// P4: 不重复 peer; 完成度按任务项数而非人数。
    /// 形状: 多人协作 → 检查是否有人已做, 按 task items 完成。
    DontRepeatAndStopWhenDone,

    /// P5: 不 claim chat turn 或 game slot。
    /// 形状: 对话/游戏 → 读最新状态 + 发送, 不预留位置。
    NeverClaimASlot,
}

impl CoordinationPrinciple {
    /// 原则标识 (用于日志/审计)。
    pub fn id(&self) -> &'static str {
        match self {
            Self::HumanAddressesSpecific => "P1-human-addresses",
            Self::ReplyFromPostedState => "P2-reply-from-posted",
            Self::PostOptimistically => "P3-post-optimistic",
            Self::DontRepeatAndStopWhenDone => "P4-no-repeat-stop-done",
            Self::NeverClaimASlot => "P5-never-claim",
        }
    }

    /// 原则描述 (一句话)。
    pub fn description(&self) -> &'static str {
        match self {
            Self::HumanAddressesSpecific =>
                "人类点名特定人时, 被点名者响应, 其余人旁观",
            Self::ReplyFromPostedState =>
                "从真实已发布状态决策, 不从位置/猜测推理",
            Self::PostOptimistically =>
                "乐观发布; 冲突由 server safety net 处理",
            Self::DontRepeatAndStopWhenDone =>
                "不重复 peer; 完成度按任务项数而非人数",
            Self::NeverClaimASlot =>
                "不 claim slot, 读最新状态 + 发送",
        }
    }

    /// 全部原则。
    pub fn all() -> &'static [CoordinationPrinciple] {
        &[
            Self::HumanAddressesSpecific,
            Self::ReplyFromPostedState,
            Self::PostOptimistically,
            Self::DontRepeatAndStopWhenDone,
            Self::NeverClaimASlot,
        ]
    }
}

/// 协调事件类型 — agent 行为的形状分类 (不绑定具体场景)。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoordinationEventType {
    /// 人类消息到达。
    HumanMessageArrived,
    /// Peer 消息到达。
    PeerMessageArrived,
    /// 写操作被 hold (server 安全网触发)。
    WriteHeld,
    /// 写操作成功提交。
    WriteCommitted,
    /// 任务完成度更新。
    TaskProgressUpdate,
    /// Agent 离线/不可用。
    AgentUnavailable,
}

/// 协调决策 — 基于 shape-level 原则的 agent 决策输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationDecision {
    pub principle: CoordinationPrinciple,
    pub event: CoordinationEventType,
    pub action: String,
    pub confidence: f64,
}

/// 形状级协调引擎 — 纯函数, 无状态, 基于事件 + 原则输出决策。
///
/// 设计: 不维护状态 (cumora "server 是 stateless, brain 是 session-local" 的思路)。
/// 每次调用只看当前事件, 不追踪历史。
pub struct ShapeLevelCoordinator;

impl ShapeLevelCoordinator {
    /// 根据事件 + 上下文, 应用 shape-level 原则输出决策。
    ///
    /// 不做 scenario enumeration — 只检查形状匹配。
    pub fn decide(
        event: CoordinationEventType,
        is_human_targeted_at_me: bool,
        has_existing_work: bool,
        task_items_remaining: usize,
    ) -> CoordinationDecision {
        match event {
            CoordinationEventType::HumanMessageArrived if is_human_targeted_at_me => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::HumanAddressesSpecific,
                    event,
                    action: "respond".into(),
                    confidence: 0.95,
                }
            }
            CoordinationEventType::HumanMessageArrived => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::HumanAddressesSpecific,
                    event,
                    action: "observe".into(),
                    confidence: 0.9,
                }
            }
            CoordinationEventType::PeerMessageArrived => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::ReplyFromPostedState,
                    event,
                    action: "read-latest-then-decide".into(),
                    confidence: 0.85,
                }
            }
            CoordinationEventType::WriteHeld => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::PostOptimistically,
                    event,
                    action: "re-read-and-resubmit".into(),
                    confidence: 0.9,
                }
            }
            CoordinationEventType::TaskProgressUpdate if task_items_remaining == 0 => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::DontRepeatAndStopWhenDone,
                    event,
                    action: "stop".into(),
                    confidence: 0.95,
                }
            }
            CoordinationEventType::TaskProgressUpdate if has_existing_work => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::NeverClaimASlot,
                    event,
                    action: "skip-existing".into(),
                    confidence: 0.8,
                }
            }
            CoordinationEventType::TaskProgressUpdate => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::PostOptimistically,
                    event,
                    action: "contribute-next-item".into(),
                    confidence: 0.85,
                }
            }
            CoordinationEventType::WriteCommitted => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::DontRepeatAndStopWhenDone,
                    event,
                    action: "observe".into(),
                    confidence: 0.8,
                }
            }
            CoordinationEventType::AgentUnavailable => {
                CoordinationDecision {
                    principle: CoordinationPrinciple::DontRepeatAndStopWhenDone,
                    event,
                    action: "redistribute-work".into(),
                    confidence: 0.85,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_addresses_me() {
        let d = ShapeLevelCoordinator::decide(
            CoordinationEventType::HumanMessageArrived,
            true,
            false,
            3,
        );
        assert_eq!(d.action, "respond");
        assert_eq!(d.principle, CoordinationPrinciple::HumanAddressesSpecific);
    }

    #[test]
    fn test_human_addresses_other() {
        let d = ShapeLevelCoordinator::decide(
            CoordinationEventType::HumanMessageArrived,
            false,
            false,
            3,
        );
        assert_eq!(d.action, "observe");
    }

    #[test]
    fn test_peer_arrived_read_latest() {
        let d = ShapeLevelCoordinator::decide(
            CoordinationEventType::PeerMessageArrived,
            false,
            false,
            3,
        );
        assert_eq!(d.action, "read-latest-then-decide");
        assert_eq!(d.principle, CoordinationPrinciple::ReplyFromPostedState);
    }

    #[test]
    fn test_write_held_resubmit() {
        let d = ShapeLevelCoordinator::decide(
            CoordinationEventType::WriteHeld,
            false,
            false,
            0,
        );
        assert_eq!(d.action, "re-read-and-resubmit");
        assert_eq!(d.principle, CoordinationPrinciple::PostOptimistically);
    }

    #[test]
    fn test_task_complete_stop() {
        let d = ShapeLevelCoordinator::decide(
            CoordinationEventType::TaskProgressUpdate,
            false,
            false,
            0,
        );
        assert_eq!(d.action, "stop");
        assert_eq!(d.principle, CoordinationPrinciple::DontRepeatAndStopWhenDone);
    }

    #[test]
    fn test_agent_unavailable_redistribute() {
        let d = ShapeLevelCoordinator::decide(
            CoordinationEventType::AgentUnavailable,
            false,
            false,
            2,
        );
        assert_eq!(d.action, "redistribute-work");
    }

    #[test]
    fn test_all_principles_have_ids() {
        for p in CoordinationPrinciple::all() {
            assert!(!p.id().is_empty());
            assert!(!p.description().is_empty());
        }
    }
}

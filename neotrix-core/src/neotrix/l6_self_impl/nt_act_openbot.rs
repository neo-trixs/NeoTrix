//! L6 / NT-ACT — openbot (github.com/CopilotKit/OpenBot) 吸收节点 (C1)。
//!
//! 源: OpenBot — 把智能手机变为可编程机器人的框架, 由智能体 (agent) 在设备外
//! 驱动感知/控制回路, 通过共享指令通道协调"机器人身体"。NeoTrix 视角:
//! 智能体/机器人编排 = 一组具身 worker (具身执行单元) 由编排器 (agent runtime)
//! 通过统一指令总线调度。本节点提供无副作用的编排 stub。

use crate::core::nt_core_self_test::SelfTest;

/// 一个具身执行单元 (机器人身体 / 设备 worker)。
#[derive(Debug, Clone)]
pub struct BotBody {
    pub id: String,
    pub capability: String,
}

/// 编排器下发给具身的指令。
#[derive(Debug, Clone)]
pub struct BotCommand {
    pub target: String,
    pub action: String,
}

/// 智能体/机器人编排运行时 (stub)。
pub trait OpenBotRuntime {
    /// 登记具身 worker, 返回当前在线 body 数。
    fn register_bodies(&self, bodies: &[BotBody]) -> usize;
    /// 编排器把任务编译为指令下发, 返回成功派发的指令数。
    fn dispatch(&self, task: &str, bodies: &[BotBody]) -> usize;
}

/// 纯计算编排器 (无副作用)。
pub struct OpenBotOrchestrator;

impl OpenBotRuntime for OpenBotOrchestrator {
    fn register_bodies(&self, bodies: &[BotBody]) -> usize {
        bodies.len()
    }

    fn dispatch(&self, task: &str, bodies: &[BotBody]) -> usize {
        if task.is_empty() || bodies.is_empty() {
            return 0;
        }
        // 每个有能力的 body 至少接收一条指令。
        bodies
            .iter()
            .filter(|b| !b.capability.is_empty())
            .count()
    }
}

/// 计算一次编排派发的指令数 (stub 纯函数)。
pub fn plan_dispatch(task: &str, bodies: &[BotBody]) -> usize {
    OpenBotOrchestrator.dispatch(task, bodies)
}

#[derive(Default)]
pub struct OpenBotSelfTest;

impl SelfTest for OpenBotSelfTest {
    fn name(&self) -> &str {
        "nt_act_openbot"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let rt = OpenBotOrchestrator;
        let mut errs = Vec::new();
        let bodies = [
            BotBody { id: "b1".into(), capability: "drive".into() },
            BotBody { id: "b2".into(), capability: "camera".into() },
        ];
        if rt.register_bodies(&bodies) != 2 {
            errs.push("openbot: register_bodies must return body count".into());
        }
        if rt.dispatch("patrol", &bodies) != 2 {
            errs.push("openbot: dispatch must reach 2 capable bodies".into());
        }
        if rt.dispatch("", &bodies) != 0 {
            errs.push("openbot: empty task must dispatch nothing".into());
        }
        if rt.dispatch("patrol", &[]) != 0 {
            errs.push("openbot: no bodies must dispatch nothing".into());
        }
        if plan_dispatch("patrol", &bodies) != 2 {
            errs.push("openbot: plan_dispatch inconsistent".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_counts_bodies() {
        let rt = OpenBotOrchestrator;
        assert_eq!(
            rt.register_bodies(&[
                BotBody { id: "x".into(), capability: "c".into() }
            ]),
            1
        );
    }

    #[test]
    fn dispatch_reaches_capable() {
        let rt = OpenBotOrchestrator;
        let bodies = [
            BotBody { id: "a".into(), capability: "move".into() },
            BotBody { id: "b".into(), capability: String::new() },
        ];
        assert_eq!(rt.dispatch("go", &bodies), 1);
    }

    #[test]
    fn empty_task_or_body_noop() {
        let rt = OpenBotOrchestrator;
        assert_eq!(rt.dispatch("", &[BotBody { id: "a".into(), capability: "c".into() }]), 0);
        assert_eq!(rt.dispatch("go", &[]), 0);
    }
}

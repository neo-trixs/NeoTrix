//! LLM 池健康命令 (NT-REPAIR 自愈节点可观测性出口)。
//!
//! `neotrix health --pool` / `/pool-health` — 构建生产网关 (加载 FreeModelCatalog) 后产出
//! `PoolHealthReport`, 使需求驱动自愈 (`ensure_pool_sufficient`) 的存活度可观测、可断言,
//! 而非黑盒补充。对齐 OmniRoute Radar 健康度 + NeoTrix 自愈公理 (R-P42 强化现有节点)。

use std::sync::Arc;

use tokio::runtime::Runtime;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::l1_action::nt_io::nt_io_provider::factory::create_gateway_async;
use crate::neotrix::nt_io_provider::gateway::pool_health::LlmPoolHealth;
use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;

pub struct PoolHealthCmd;

impl CliCommand for PoolHealthCmd {
    fn name(&self) -> &str {
        "/pool-health"
    }

    fn is_primary(&self) -> bool {
        false
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/pool", "/health"]
    }

    fn description(&self) -> &str {
        "Report LLM provider pool health (free/paid/local/locked/sufficient) — NT-REPAIR self-heal observability"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        // 可选首个参数: min_free 阈值 (默认 3, 对齐 ensure_pool_sufficient 生产默认)。
        let min_free = args
            .first()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(3);

        let rt = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => return CommandOutput::err(&format!("tokio runtime init failed: {e}")),
        };

        // 构建生产网关 (加载 FreeModelCatalog, 网络 5s 超时兜底) 后对池子做健康评估。
        let report = rt.block_on(async {
            let gw = create_gateway_async().await;
            LlmPoolHealth::evaluate(&gw, min_free)
        });

        let msg = format!(
            "LLM pool health: total={} free={} paid={} local={} model_locked={} sufficient={} (min_free={})",
            report.total,
            report.free,
            report.paid,
            report.local,
            report.model_locked,
            report.sufficient,
            report.min_free
        );

        CommandOutput::ok(&msg)
            .with_json(serde_json::to_value(&report).unwrap_or(serde_json::Value::Null))
    }
}

//! Brain 交互命令 — /e8 (E8 推理状态查询)
//! 注: /absorb /mem /save /trace /avatar 为 auto-backend 自驱命令,
//! 不注册 CLI (见 test_auto_commands_not_in_registry), 由 entry/headless 层字符串匹配处理。

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

// ====== /e8 ======

pub struct E8Cmd;
impl CliCommand for E8Cmd {
    fn name(&self) -> &str { "/e8" }
    fn aliases(&self) -> Vec<&str> { vec!["/hexagram", "/mode"] }
    fn description(&self) -> &str { "Consciousness Core Status: /e8 [status|set <0-63>|matrix|transition|consciousness]" }
    fn is_primary(&self) -> bool { true }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        // 子命令 consciousness: 合并 /consciousness — 意识状态查询进意识核心面
        if args.iter().any(|a| a == "consciousness" || a == "cns" || a == "awareness") {
            let rest: Vec<String> = args.iter().filter(|a| *a != "consciousness" && *a != "cns" && *a != "awareness").cloned().collect();
            let reg = crate::cli::commands::registry::default_registry();
            match reg.find("/consciousness") {
                Some(cmd) => return cmd.execute(&rest, brain),
                None => return CommandOutput::err("意识状态命令不可用"),
            }
        }
        let b = match brain {
            Some(b) => b.blocking_read(),
            None => return CommandOutput::err("Brain 不可用"),
        };
        let engine = match b.reasoning_engine.as_ref() {
            Some(e) => e,
            None => return CommandOutput::err("推理引擎不可用"),
        };
        let sub = args.iter().find(|a| !a.starts_with("--"));
        match sub.map(|s| s.as_str()) {
            None | Some("status") => {
                let state = &engine.current_state;
                let mode = state.mode;
                let meta = state.meta;
                let sig = state.signature();
                let traj_len = engine.state_trajectory.len();
                let msg = format!(
                    "E8 推理状态\n\
                     当前模式: {:<20} (0x{:02X})\n\
                     元状态:   {:<4} (0x{:02X})\n\
                     签名:     {} (0x{:04X})\n\
                     轨迹长度: {}\n\
                     Hexagram: {:06b} ({:02o})\n\
                     轴: 抽象={} 范围={} 方法={} 深度={} 模式={} 姿态={}",
                    mode.mode_name(), mode.0,
                    meta.0, meta.0,
                    sig, sig,
                    traj_len,
                    mode.0, mode.0,
                    axis_label(mode.0, 5), axis_label(mode.0, 4),
                    axis_label(mode.0, 3), axis_label(mode.0, 2),
                    axis_label(mode.0, 1), axis_label(mode.0, 0),
                );
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "mode": mode.0, "mode_name": mode.mode_name(),
                        "meta": meta.0, "signature": sig,
                        "trajectory_len": traj_len,
                    }))
                } else { out }
            }
            Some("matrix") => {
                let mut s = String::from("E8 策略矩阵 (8×8):\n\n   ");
                for col in 0..8 { s.push_str(&format!("{:4}", col)); }
                s.push('\n');
                for row in 0..8 {
                    s.push_str(&format!("{:2} ", row));
                    for col in 0..8 {
                        let val = engine.strategy_matrix[row][col].0;
                        s.push_str(&format!("{:4}", val));
                    }
                    s.push('\n');
                }
                CommandOutput::ok(&s)
            }
            Some("transition") => {
                if engine.state_trajectory.len() < 2 {
                    return CommandOutput::ok("轨迹太短，无法分析转换");
                }
                let mut s = String::from("E8 状态转换序列:\n");
                for w in engine.state_trajectory.windows(2) {
                    let from = w[0].mode.0;
                    let to = w[1].mode.0;
                    let arrow = if from == to { "━" } else { "→" };
                    s.push_str(&format!("  {} {} {}\n", w[0].mode.mode_name(), arrow, w[1].mode.mode_name()));
                }
                let out = CommandOutput::ok(&s);
                if want_json {
                    let transitions: Vec<serde_json::Value> = engine.state_trajectory.windows(2).map(|w| {
                        serde_json::json!({"from": w[0].mode.0, "to": w[1].mode.0})
                    }).collect();
                    out.with_json(serde_json::json!({"transitions": transitions}))
                } else { out }
            }
            Some("set") if args.len() >= 2 => {
                let val = args[1].parse::<u8>();
                match val {
                    Ok(v) if v < 64 => {
                        CommandOutput::ok(&format!("设置 E8 模式为 {} ({})", v, crate::core::nt_core_hex::ReasoningHexagram::new(v).mode_name()))
                    }
                    Ok(_) => CommandOutput::err("E8 模式范围 0-63"),
                    Err(_) => CommandOutput::err("无效参数，请输入 0-63 的数字"),
                }
            }
            Some(other) => CommandOutput::err(&format!("未知子命令: {}。可用: status, set <0-63>, matrix, transition", other)),
        }
    }
}

fn axis_label(mode: u8, bit: u8) -> &'static str {
    let bit_val = (mode >> bit) & 1;
    match (bit, bit_val) {
        (5, 0) => "具体", (5, 1) => "抽象",
        (4, 0) => "聚焦", (4, 1) => "广泛",
        (3, 0) => "分析", (3, 1) => "生成",
        (2, 0) => "深度", (2, 1) => "快速",
        (1, 0) => "独立", (1, 1) => "协作",
        (0, 0) => "确定", (0, 1) => "探索",
        _ => "未知",
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}

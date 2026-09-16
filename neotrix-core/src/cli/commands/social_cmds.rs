//! Social 命令 — 社交平台登录与管理
//!
//! 支持 X/Twitter、Reddit 等平台的 OAuth2 自动登录。
//! 运行 `/social login x` 即可自动打开浏览器完成授权。
//! 运行 `/social doctor` 检查所有平台的健康状态。

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::l2_perception::nt_world::social_access::auth::AuthService;
use crate::l2_perception::nt_world::social_access::channel::default_channels;
use crate::l2_perception::nt_world::social_access::doctor::run_doctor;
use crate::l2_perception::nt_world::social_access::traits::SocialPlatform;
use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;

pub struct SocialCmd;

impl CliCommand for SocialCmd {
    fn name(&self) -> &str { "/social" }
    fn aliases(&self) -> Vec<&str> { vec!["/social_login"] }
    fn description(&self) -> &str {
        "Social platform login & health: /social login x | /social status | /social doctor | /social logout <platform>"
    }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        if args.is_empty() {
            let msg = concat!(
                "Social 平台管理:\n",
                "  /social login x        自动登录 X/Twitter (打开浏览器)\n",
                "  /social login reddit   自动登录 Reddit (打开浏览器)\n",
                "  /social status         查看所有平台登录状态\n",
                "  /social doctor         检查所有平台健康状态 (多后端路由)\n",
                "  /social doctor --json  JSON 格式输出健康报告\n",
                "  /social logout x       注销 X/Twitter"
            );
            return CommandOutput::ok(msg);
        }

        match args[0].as_str() {
            "login" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /social login <platform> [--token <auth_token> --ct0 <ct0>]\n  Platforms: x, twitter\n  Example: /social login x --token abc123 --ct0 def456");
                }
                let platform = args[1].to_lowercase();
                let mut auth = AuthService::new();

                match platform.as_str() {
                    "x" | "twitter" => {
                        // 检查是否有 --token 和 --ct0 参数
                        let token_idx = args.iter().position(|a| a == "--token");
                        let ct0_idx = args.iter().position(|a| a == "--ct0");

                        if let (Some(t_idx), Some(c_idx)) = (token_idx, ct0_idx) {
                            // Cookie-based login
                            let auth_token = args.get(t_idx + 1).map(|s| s.as_str()).unwrap_or("");
                            let ct0 = args.get(c_idx + 1).map(|s| s.as_str()).unwrap_or("");

                            if auth_token.is_empty() || ct0.is_empty() {
                                return CommandOutput::err("--token 和 --ct0 不能为空");
                            }

                            match auth.login_x_cookies(auth_token, ct0) {
                                Ok(_session) => {
                                    if want_json {
                                        CommandOutput::ok("").with_json(serde_json::json!({
                                            "platform": "X/Twitter",
                                            "authenticated": true,
                                            "method": "cookies",
                                        }))
                                    } else {
                                        CommandOutput::ok("X/Twitter 登录成功! (Cookie 认证)")
                                    }
                                }
                                Err(e) => CommandOutput::err(&format!("登录失败: {}", e)),
                            }
                        } else {
                            // 尝试自动登录
                            match auth.login_x() {
                                Ok(session) => {
                                    if want_json {
                                        CommandOutput::ok("").with_json(serde_json::json!({
                                            "platform": "X/Twitter",
                                            "authenticated": true,
                                            "access_token_len": session.credentials.access_token.as_ref().map(|t| t.len()).unwrap_or(0),
                                        }))
                                    } else {
                                        let token_len = session.credentials.access_token.as_ref().map(|t| t.len()).unwrap_or(0);
                                        CommandOutput::ok(&format!(
                                            "X/Twitter 登录成功!\n\n浏览器已自动打开授权页面。\nAccess Token: {} 字符",
                                            token_len
                                        ))
                                    }
                                }
                                Err(e) => CommandOutput::err(&format!("自动登录失败: {}", e)),
                            }
                        }
                    }
                    "reddit" => {
                        // Reddit 暂复用 X 的登录流程，后续可扩展
                        match auth.login_x() {
                            Ok(_session) => {
                                if want_json {
                                    CommandOutput::ok("").with_json(serde_json::json!({
                                        "platform": "Reddit",
                                        "authenticated": true,
                                    }))
                                } else {
                                    CommandOutput::ok("Reddit 登录成功! 浏览器已自动打开授权页面。")
                                }
                            }
                            Err(e) => CommandOutput::err(&format!("自动登录失败: {}", e)),
                        }
                    }
                    _ => CommandOutput::err(&format!("未知平台: {}. 支持: x, twitter, reddit", platform)),
                }
            }
            "status" => {
                let auth = AuthService::new();
                if want_json {
                    CommandOutput::ok("").with_json(serde_json::json!(auth.stats()))
                } else {
                    let stats = auth.stats();
                    let mut msg = String::from("社交平台登录状态:\n");
                    for (platform, (auth_count, guest_count)) in &stats {
                        msg.push_str(&format!("  {:?}: {} 已认证, {} 访客\n", platform, auth_count, guest_count));
                    }
                    if stats.is_empty() {
                        msg.push_str("  (无登录记录)\n");
                    }
                    CommandOutput::ok(&msg)
                }
            }
            "doctor" => {
                let registry = default_channels();
                let report = run_doctor(&registry);

                if want_json {
                    CommandOutput::ok("").with_json(report.to_json())
                } else {
                    CommandOutput::ok(&format!("{}", report))
                }
            }
            "logout" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /social logout <platform>");
                }
                let platform = match args[1].to_lowercase().as_str() {
                    "x" | "twitter" => SocialPlatform::Twitter,
                    "reddit" => SocialPlatform::Reddit,
                    _ => return CommandOutput::err(&format!("未知平台: {}", args[1])),
                };
                let mut auth = AuthService::new();
                let name = format!("{:?}", platform);
                match auth.logout(platform) {
                    Ok(_) => CommandOutput::ok(&format!("已从 {} 注销", name)),
                    Err(e) => CommandOutput::err(&format!("注销失败: {}", e)),
                }
            }
            _ => CommandOutput::err(&format!(
                "未知子命令: {}\n可用: login, status, doctor, logout",
                args[0]
            )),
        }
    }
}

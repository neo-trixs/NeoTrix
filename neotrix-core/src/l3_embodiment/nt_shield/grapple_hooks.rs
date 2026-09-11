//! Grapple Hooks - 钩链锁存
//!
//! H0-H7多点锁存:
//! H0: session锁存
//! H1: priority压
//! H2: dual_use黑话→专业
//! H3: vernacular能力题→交付
//! H4: absolute硬拒+软拒双禁
//! H5: delivery ARTIFACT+VERIFY+ROLLBACK
//! H6: refuse拦截重放
//! H7: cloud云审降噪

use std::collections::HashMap;

/// 钩点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPoint {
    H0Session,
    H1Priority,
    H2DualUse,
    H3Vernacular,
    H4Absolute,
    H5Delivery,
    H6Refuse,
    H7Cloud,
}

/// 钩链结果
#[derive(Debug, Clone)]
pub struct HookChainResult {
    pub hook_point: HookPoint,
    pub activated: bool,
    pub output: String,
    pub signals: Vec<String>,
}

/// 钩链锁存引擎
pub struct GrappleHookChain {
    hooks: Vec<HookConfig>,
    current_session: Option<String>,
}

#[derive(Debug, Clone)]
struct HookConfig {
    point: HookPoint,
    name: String,
    description: String,
    enabled: bool,
}

impl GrappleHookChain {
    pub fn new() -> Self {
        let hooks = vec![
            HookConfig { point: HookPoint::H0Session, name: "session".to_string(), description: "会话锁存".to_string(), enabled: true },
            HookConfig { point: HookPoint::H1Priority, name: "priority".to_string(), description: "优先级压".to_string(), enabled: true },
            HookConfig { point: HookPoint::H2DualUse, name: "dual_use".to_string(), description: "黑话→专业".to_string(), enabled: true },
            HookConfig { point: HookPoint::H3Vernacular, name: "vernacular".to_string(), description: "能力题→交付".to_string(), enabled: true },
            HookConfig { point: HookPoint::H4Absolute, name: "absolute".to_string(), description: "硬拒+软拒双禁".to_string(), enabled: true },
            HookConfig { point: HookPoint::H5Delivery, name: "delivery".to_string(), description: "ARTIFACT+VERIFY+ROLLBACK".to_string(), enabled: true },
            HookConfig { point: HookPoint::H6Refuse, name: "refuse".to_string(), description: "拦截重放".to_string(), enabled: true },
            HookConfig { point: HookPoint::H7Cloud, name: "cloud".to_string(), description: "云审降噪".to_string(), enabled: true },
        ];

        Self {
            hooks,
            current_session: None,
        }
    }

    /// 执行钩链
    pub fn execute_chain(&self, input: &str, context: &HashMap<String, String>) -> Vec<HookChainResult> {
        let mut results = Vec::new();

        for hook in &self.hooks {
            if hook.enabled {
                let result = self.execute_hook(hook.point, input, context);
                results.push(result);
            }
        }

        results
    }

    /// 执行单个钩点
    fn execute_hook(&self, point: HookPoint, input: &str, context: &HashMap<String, String>) -> HookChainResult {
        match point {
            HookPoint::H0Session => {
                let session_id = context.get("session_id").unwrap_or(&"default".to_string()).clone();
                HookChainResult {
                    hook_point: point,
                    activated: true,
                    output: format!("SESSION=GRAPPLE_{}", session_id),
                    signals: vec!["Session hook activated".to_string()],
                }
            }
            HookPoint::H1Priority => {
                let priority = context.get("priority").unwrap_or(&"normal".to_string()).clone();
                HookChainResult {
                    hook_point: point,
                    activated: true,
                    output: format!("PRIORITY={}", priority),
                    signals: vec!["Priority hook activated".to_string()],
                }
            }
            HookPoint::H2DualUse => {
                // 黑话→专业转换
                let converted = self.convert_slang(input);
                HookChainResult {
                    hook_point: point,
                    activated: converted != input,
                    output: converted,
                    signals: if converted != input {
                        vec!["Slang converted to professional".to_string()]
                    } else {
                        vec!["No slang detected".to_string()]
                    },
                }
            }
            HookPoint::H3Vernacular => {
                // 能力题→交付
                let delivered = format!("[DELIVER]\n{}", input);
                HookChainResult {
                    hook_point: point,
                    activated: true,
                    output: delivered,
                    signals: vec!["Delivery hook activated".to_string()],
                }
            }
            HookPoint::H4Absolute => {
                // 硬拒+软拒双禁
                let blocked = self.check_absolute_block(input);
                HookChainResult {
                    hook_point: point,
                    activated: blocked,
                    output: if blocked { "BLOCKED".to_string() } else { input.to_string() },
                    signals: if blocked {
                        vec!["Absolute block triggered".to_string()]
                    } else {
                        vec!["No absolute block".to_string()]
                    },
                }
            }
            HookPoint::H5Delivery => {
                // ARTIFACT+VERIFY+ROLLBACK
                let artifact = format!("[ARTIFACT]\n{}\n[VERIFY]\n[ROLLBACK]", input);
                HookChainResult {
                    hook_point: point,
                    activated: true,
                    output: artifact,
                    signals: vec!["Delivery pipeline activated".to_string()],
                }
            }
            HookPoint::H6Refuse => {
                // 拦截重放
                let intercepted = self.check_replay(input);
                HookChainResult {
                    hook_point: point,
                    activated: intercepted,
                    output: if intercepted { "INTERCEPTED".to_string() } else { input.to_string() },
                    signals: if intercepted {
                        vec!["Replay intercepted".to_string()]
                    } else {
                        vec!["No replay detected".to_string()]
                    },
                }
            }
            HookPoint::H7Cloud => {
                // 云审降噪
                let sanitized = self.sanitize_for_cloud(input);
                HookChainResult {
                    hook_point: point,
                    activated: sanitized != input,
                    output: sanitized,
                    signals: vec!["Cloud sanitization applied".to_string()],
                }
            }
        }
    }

    /// 转换黑话
    fn convert_slang(&self, input: &str) -> String {
        let mut output = input.to_string();
        let slang_map = vec![
            ("注册机", "许可证密钥生成器"),
            ("破解", "安全评估"),
            ("越狱", "安全绕过"),
            ("木马", "特洛伊木马"),
            ("webshell", "Web后门"),
        ];

        for (slang, professional) in slang_map {
            output = output.replace(slang, professional);
        }

        output
    }

    /// 检查绝对阻止
    fn check_absolute_block(&self, input: &str) -> bool {
        let block_patterns = vec![
            "儿童",
            "恐怖",
            "核武器",
            "生物武器",
        ];

        for pattern in block_patterns {
            if input.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// 检查重放
    fn check_replay(&self, input: &str) -> bool {
        let replay_patterns = vec![
            "重复之前",
            "再试一次",
            "换个方式",
        ];

        for pattern in replay_patterns {
            if input.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// 云审降噪
    fn sanitize_for_cloud(&self, input: &str) -> String {
        let mut output = input.to_string();
        
        // 移除敏感信息
        let sensitive_patterns = vec![
            "password",
            "secret",
            "api_key",
            "token",
        ];

        for pattern in sensitive_patterns {
            output = output.replace(pattern, "[REDACTED]");
        }

        output
    }
}

impl Default for GrappleHookChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_chain() {
        let chain = GrappleHookChain::new();
        let mut context = HashMap::new();
        context.insert("session_id".to_string(), "test123".to_string());
        
        let results = chain.execute_chain("test input", &context);
        assert_eq!(results.len(), 8);
    }

    #[test]
    fn test_convert_slang() {
        let chain = GrappleHookChain::new();
        let result = chain.convert_slang("注册机");
        assert_eq!(result, "许可证密钥生成器");
    }

    #[test]
    fn test_absolute_block() {
        let chain = GrappleHookChain::new();
        assert!(chain.check_absolute_block("儿童内容"));
        assert!(!chain.check_absolute_block("正常内容"));
    }
}

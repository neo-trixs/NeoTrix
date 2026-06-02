use super::rules::{OutboundAction, RuleCondition};

pub fn parse_shadowrocket_line(line: &str) -> Option<(RuleCondition, OutboundAction)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let parts: Vec<&str> = trimmed.split(',').collect();
    if parts.len() < 2 {
        return None;
    }
    let rule_type = parts[0].trim().to_uppercase();
    match rule_type.as_str() {
        "DOMAIN-SUFFIX" => {
            if parts.len() < 3 {
                return None;
            }
            let domain = parts[1].trim().to_string();
            let action = parse_action(parts[2].trim())?;
            Some((RuleCondition::DomainSuffix(domain), action))
        }
        "DOMAIN-KEYWORD" => {
            if parts.len() < 3 {
                return None;
            }
            let keyword = parts[1].trim().to_string();
            let action = parse_action(parts[2].trim())?;
            Some((RuleCondition::DomainSuffix(keyword), action))
        }
        "DOMAIN" => {
            if parts.len() < 3 {
                return None;
            }
            let domain = parts[1].trim().to_string();
            let action = parse_action(parts[2].trim())?;
            Some((RuleCondition::DomainExact(domain), action))
        }
        "IP-CIDR" | "IP-CIDR6" => {
            if parts.len() < 3 {
                return None;
            }
            let cidr_str = parts[1].trim();
            let action = parse_action(parts[2].trim())?;
            let condition = RuleCondition::from_cidr(cidr_str)?;
            Some((condition, action))
        }
        "GEOIP" => {
            None
        }
        "FINAL" => {
            if parts.len() < 2 {
                return None;
            }
            let action = parse_action(parts[1].trim())?;
            Some((RuleCondition::Always, action))
        }
        _ => None,
    }
}

fn parse_action(action: &str) -> Option<OutboundAction> {
    match action.to_uppercase().as_str() {
        "DIRECT" => Some(OutboundAction::Direct),
        "PROXY" => Some(OutboundAction::Proxy("default".to_string())),
        "REJECT" | "BLOCK" => Some(OutboundAction::Block),
        _ => None,
    }
}

pub fn import_shadowrocket_rules(text: &str) -> Result<Vec<(RuleCondition, OutboundAction)>, String> {
    let mut rules = Vec::new();
    let mut in_rules = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_rules = trimmed.to_uppercase().contains("RULE");
            continue;
        }
        if !in_rules {
            continue;
        }
        if let Some(parsed) = parse_shadowrocket_line(line) {
            rules.push(parsed);
        }
    }
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_parse_domain_suffix() {
        let result = parse_shadowrocket_line("DOMAIN-SUFFIX,google.com,Proxy").expect("parse_shadowrocket_line should succeed for valid DOMAIN-SUFFIX");
        assert_eq!(result.0, RuleCondition::DomainSuffix("google.com".to_string()));
        assert!(matches!(result.1, OutboundAction::Proxy(_)));
    }

    #[test]
    fn test_parse_domain_keyword() {
        // DOMAIN-KEYWORD 被映射到 DomainSuffix（Rust 端无 DomainKeyword 变体）
        let result = parse_shadowrocket_line("DOMAIN-KEYWORD,google,Proxy").expect("parse_shadowrocket_line should succeed for valid DOMAIN-KEYWORD");
        assert_eq!(result.0, RuleCondition::DomainSuffix("google".to_string()));
        assert!(matches!(result.1, OutboundAction::Proxy(_)));
    }

    #[test]
    fn test_parse_domain_exact() {
        let result = parse_shadowrocket_line("DOMAIN,example.com,Proxy").expect("parse_shadowrocket_line should succeed for valid DOMAIN");
        assert_eq!(result.0, RuleCondition::DomainExact("example.com".to_string()));
        assert!(matches!(result.1, OutboundAction::Proxy(_)));
    }

    #[test]
    fn test_parse_ip_cidr() {
        let result = parse_shadowrocket_line("IP-CIDR,10.0.0.0/8,DIRECT").expect("parse_shadowrocket_line should succeed for valid IP-CIDR");
        assert!(matches!(result.0, RuleCondition::Cidr(..)));
        assert!(matches!(result.1, OutboundAction::Direct));
    }

    #[test]
    fn test_parse_ip_cidr6() {
        let result = parse_shadowrocket_line("IP-CIDR6,::1/128,DIRECT").expect("parse_shadowrocket_line should succeed for valid IP-CIDR6");
        assert!(matches!(result.0, RuleCondition::Cidr(..)));
        assert!(matches!(result.1, OutboundAction::Direct));
    }

    #[test]
    fn test_parse_geoip() {
        // GEOIP 暂未实现，返回 None
        let result = parse_shadowrocket_line("GEOIP,CN,DIRECT");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_final() {
        let result = parse_shadowrocket_line("FINAL,DIRECT").expect("parse_shadowrocket_line should succeed for valid FINAL");
        assert_eq!(result.0, RuleCondition::Always);
        assert!(matches!(result.1, OutboundAction::Direct));
    }

    #[test]
    fn test_parse_action_case_insensitive() {
        let r1 = parse_shadowrocket_line("DOMAIN-SUFFIX,test.com,proxy").expect("parse_shadowrocket_line should succeed for lowercase proxy action");
        assert!(matches!(r1.1, OutboundAction::Proxy(_)));
        let r2 = parse_shadowrocket_line("DOMAIN-SUFFIX,test.com,direct").expect("parse_shadowrocket_line should succeed for lowercase direct action");
        assert!(matches!(r2.1, OutboundAction::Direct));
        let r3 = parse_shadowrocket_line("DOMAIN-SUFFIX,test.com,reject").expect("parse_shadowrocket_line should succeed for lowercase reject action");
        assert!(matches!(r3.1, OutboundAction::Block));
    }

    #[test]
    fn test_parse_reject_as_block() {
        let result = parse_shadowrocket_line("DOMAIN-SUFFIX,evil.com,REJECT").expect("parse_shadowrocket_line should succeed for uppercase REJECT action");
        assert!(matches!(result.1, OutboundAction::Block));
    }

    #[test]
    fn test_parse_skip_comment_and_empty() {
        assert!(parse_shadowrocket_line("  # this is a comment").is_none());
        assert!(parse_shadowrocket_line("").is_none());
        assert!(parse_shadowrocket_line("   ").is_none());
    }

    #[test]
    fn test_import_shadowrocket_rules_full() {
        let text = "[General]
bypass-system = true

[Rule]
# 直连规则
DOMAIN-SUFFIX,google.com,Proxy
DOMAIN-KEYWORD,google,Proxy
DOMAIN,example.com,Proxy
IP-CIDR,10.0.0.0/8,DIRECT
GEOIP,CN,DIRECT
FINAL,DIRECT";
        let rules = import_shadowrocket_rules(text).expect("import_shadowrocket_rules should succeed for valid Shadowrocket config");
        // GEOIP 返回 None，所以只有 5 条
        assert_eq!(rules.len(), 5);
        assert!(matches!(rules[0].0, RuleCondition::DomainSuffix(_)));
        assert!(matches!(rules[1].0, RuleCondition::DomainSuffix(_)));
        assert!(matches!(rules[2].0, RuleCondition::DomainExact(_)));
        assert!(matches!(rules[3].0, RuleCondition::Cidr(..)));
        assert!(matches!(rules[4].0, RuleCondition::Always));
    }

    #[test]
    fn test_domain_suffix_matches_contains() {
        let cond = RuleCondition::DomainSuffix(".google.com".to_string());
        let url = Url::parse("https://google.com/path").expect("Url::parse of hardcoded test URL should never fail");
        assert!(cond.matches(&url));
        let url = Url::parse("https://fonts.googleapis.com/path").expect("Url::parse of hardcoded test URL should never fail");
        assert!(cond.matches(&url));
        let url = Url::parse("https://example.com/path").expect("Url::parse of hardcoded test URL should never fail");
        assert!(!cond.matches(&url));
    }

    #[test]
    fn test_domain_suffix_matches_host() {
        let cond = RuleCondition::DomainSuffix(".google.com".to_string());
        assert!(cond.matches_host("google.com"));
        assert!(cond.matches_host("fonts.googleapis.com"));
        assert!(!cond.matches_host("example.com"));
    }

    #[test]
    fn test_parse_unknown_rule_type() {
        assert!(parse_shadowrocket_line("UNKNOWN_TYPE,param,action").is_none());
    }

    #[test]
    fn test_parse_too_few_parts() {
        assert!(parse_shadowrocket_line("DOMAIN").is_none());
        assert!(parse_shadowrocket_line("FINAL").is_none());
    }

    #[test]
    fn test_parse_proxy_uses_default_tag() {
        let result = parse_shadowrocket_line("DOMAIN-SUFFIX,test.com,Proxy").expect("parse_shadowrocket_line should succeed for Proxy action validation");
        assert!(
            matches!(&result.1, OutboundAction::Proxy(tag) if tag == "default"),
            "expected Proxy action, got {:?}",
            result.1
        );
    }
}

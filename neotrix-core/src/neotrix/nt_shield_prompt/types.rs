#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Suspicious,
    Dangerous,
}

#[derive(Debug, Clone)]
pub struct ActionRule {
    pub action: String,
    pub allowed: bool,
    pub patterns: Vec<String>,
}

impl ActionRule {
    pub fn new(action: &str, allowed: bool, patterns: Vec<&str>) -> Self {
        Self {
            action: action.into(),
            allowed,
            patterns: patterns.into_iter().map(String::from).collect(),
        }
    }
}

pub fn glob_match(pattern: &str, target: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.ends_with("/**") {
        let prefix = &pattern[..pattern.len() - 2];
        return target.starts_with(prefix);
    }
    if pattern.contains('*') {
        let re_str = pattern
            .replace('.', "\\.")
            .replace('*', ".*")
            .replace('?', ".");
        if let Ok(re) = regex::Regex::new(&format!("^{}$", re_str)) {
            return re.is_match(target);
        }
    }
    pattern == target
}

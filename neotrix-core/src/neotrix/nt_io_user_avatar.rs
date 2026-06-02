use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::nt_io_avatar_channel::{AvatarIdentity, AvatarChain, MessageDirection};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAvatar {
    pub identity_name: String,
    pub edition: u32,
    pub confidence: f64,
    pub language_preference: f64,
    pub communication_style: f64,
    pub reasoning_depth: f64,
    pub technical_depth: f64,
    pub domain_scores: HashMap<String, f64>,
    pub task_affinity: HashMap<String, f64>,
    pub knowledge_affinity: HashMap<String, f64>,
    pub tags: Vec<String>,
    pub summary: String,
    pub total_messages_processed: u64,
    pub chain_length: usize,
}

impl UserAvatar {
    pub fn with_name(name: &str) -> Self {
        Self {
            identity_name: name.to_string(),
            ..Default::default()
        }
    }
}

impl Default for UserAvatar {
    fn default() -> Self {
        Self {
            identity_name: String::new(),
            edition: 0,
            confidence: 0.0,
            language_preference: 0.5,
            communication_style: 0.5,
            reasoning_depth: 0.5,
            technical_depth: 0.5,
            domain_scores: HashMap::new(),
            task_affinity: HashMap::new(),
            knowledge_affinity: HashMap::new(),
            tags: Vec::new(),
            summary: String::from("正在构建用户画像..."),
            total_messages_processed: 0,
            chain_length: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationNode {
    pub id: String,
    pub label: String,
    pub status: String,
    pub description: String,
    pub r#type: String,
    pub progress: f64,
    pub ttl_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationEdge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationFlowEvent {
    pub nodes: Vec<DistillationNode>,
    pub edges: Vec<DistillationEdge>,
    pub avatar_summary: String,
    pub avatar_confidence: f64,
}

pub struct DistillationEngine {
    pub avatar: UserAvatar,
    pub identity: Option<AvatarIdentity>,
    pub chain: AvatarChain,
    pub message_count: u64,
    pub flow_nodes: Vec<DistillationNode>,
    pub flow_edges: Vec<DistillationEdge>,
    pub node_counter: u64,
    domain_keywords: HashMap<String, Vec<String>>,
    task_keywords: HashMap<String, Vec<String>>,
    code_keywords: HashMap<String, Vec<String>>,
}

impl Default for DistillationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DistillationEngine {
    pub fn new() -> Self {
        let mut domain_keywords: HashMap<String, Vec<String>> = HashMap::new();
        domain_keywords.insert("rust".into(), vec!["rust".into(), "cargo".into(), "编译".into(), "类型".into(), "trait".into(), "struct".into(), "impl".into(), "unsafe".into(), "async".into()]);
        domain_keywords.insert("frontend".into(), vec!["react".into(), "组件".into(), "前端".into(), "css".into(), "html".into(), "ui".into(), "界面".into(), "tailwind".into(), "样式".into()]);
        domain_keywords.insert("backend".into(), vec!["api".into(), "server".into(), "数据库".into(), "sql".into(), "http".into(), "rest".into(), "中间件".into(), "微服务".into()]);
        domain_keywords.insert("design".into(), vec!["设计".into(), "figma".into(), "sketch".into(), "配色".into(), "字体".into(), "布局".into(), "交互".into()]);
        domain_keywords.insert("ai_ml".into(), vec!["模型".into(), "训练".into(), "推理".into(), "llm".into(), "神经".into(), "学习".into(), "数据".into(), "算法".into()]);
        domain_keywords.insert("devops".into(), vec!["docker".into(), "部署".into(), "ci/cd".into(), "k8s".into(), "云".into(), "流水线".into(), "监控".into()]);
        domain_keywords.insert("data".into(), vec!["分析".into(), "可视化".into(), "报表".into(), "图表".into(), "统计".into(), "pandas".into(), "sql".into()]);
        domain_keywords.insert("nt_shield".into(), vec!["安全".into(), "认证".into(), "权限".into(), "加密".into(), "审计".into(), "漏洞".into()]);

        let mut task_keywords: HashMap<String, Vec<String>> = HashMap::new();
        task_keywords.insert("代码开发".into(), vec!["实现".into(), "编写".into(), "开发".into(), "写".into(), "code".into(), "实现一个".into()]);
        task_keywords.insert("调试修复".into(), vec!["bug".into(), "修复".into(), "错误".into(), "问题".into(), "失败".into(), "调试".into(), "报错".into()]);
        task_keywords.insert("架构设计".into(), vec!["架构".into(), "设计".into(), "重构".into(), "结构".into(), "模块".into(), "拆分".into()]);
        task_keywords.insert("知识学习".into(), vec!["学习".into(), "了解".into(), "理解".into(), "解释".into(), "是什么".into(), "怎么用".into()]);
        task_keywords.insert("代码审查".into(), vec!["审查".into(), "review".into(), "检查".into(), "质量".into(), "优化".into()]);
        task_keywords.insert("对话讨论".into(), vec!["你觉得".into(), "建议".into(), "方案".into(), "对比".into(), "哪个好".into()]);

        let mut code_keywords: HashMap<String, Vec<String>> = HashMap::new();
        code_keywords.insert("rust".into(), vec!["fn ".into(), "let ".into(), "mut ".into(), "pub ".into(), "impl ".into(), "struct ".into(), "enum ".into(), "match ".into()]);
        code_keywords.insert("typescript".into(), vec!["interface".into(), "type ".into(), "const ".into(), "function".into(), "import".into(), "export".into()]);
        code_keywords.insert("python".into(), vec!["def ".into(), "class ".into(), "import ".into(), "async def".into(), "print".into()]);

        let chain = AvatarChain::load();
        let identity = AvatarIdentity::load();
        let mut avatar = UserAvatar::default();
        if let Some(ref id) = identity {
            avatar.identity_name = id.name.clone();
            avatar.chain_length = chain.len();
        }
        Self {
            avatar,
            identity,
            chain,
            message_count: 0,
            flow_nodes: Vec::new(),
            flow_edges: Vec::new(),
            node_counter: 0,
            domain_keywords,
            task_keywords,
            code_keywords,
        }
    }

    pub fn distill_message(&mut self, text: &str) -> DistillationFlowEvent {
        self.message_count += 1;
        let prev_confidence = self.avatar.confidence;

        let added_nodes = self.analyze_language(text);
        let lang_nodes = self.analyze_domains(text);
        let task_nodes = self.analyze_tasks(text);
        let profile_nodes = self.update_profile();

        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();

        let groups = [&added_nodes, &lang_nodes, &task_nodes, &profile_nodes];
        for group in &groups {
            for node in *group {
                if !self.flow_nodes.iter().any(|n| n.id == node.id) {
                    self.flow_nodes.push(node.clone());
                }
            }
            all_nodes.extend((*group).clone());
        }

        for edge in &self.flow_edges {
            if !all_edges.iter().any(|e: &DistillationEdge| e.source == edge.source && e.target == edge.target) {
                all_edges.push(edge.clone());
            }
        }

        self.cleanup_expired_nodes();

        let confidence_change = self.avatar.confidence - prev_confidence;
        if confidence_change > 0.01 {
            self.flow_nodes.push(DistillationNode {
                id: format!("consolidate-{}", self.node_counter),
                label: "画像合并".into(),
                status: "completed".into(),
                description: format!("信心度 {:.1}% → {:.1}%", prev_confidence * 100.0, self.avatar.confidence * 100.0),
                r#type: "aggregator".into(),
                progress: 1.0,
                ttl_seconds: 8.0,
            });
            self.node_counter += 1;
        }

        let summary = self.generate_summary();
        self.avatar.summary = summary.clone();
        self.avatar.total_messages_processed = self.message_count;

        let secret = self.identity.as_ref().map(|i| i.secret()).unwrap_or_default();
        let chain_data = format!("[{}] {} | confidence:{:.2} | edition:{}",
            self.avatar.identity_name, summary, self.avatar.confidence, self.avatar.edition);
        self.chain.push(chain_data.as_bytes(), &secret, MessageDirection::Outbound, "avatar");
        self.avatar.chain_length = self.chain.len();
        let _ = self.chain.save();

        DistillationFlowEvent {
            nodes: self.flow_nodes.clone(),
            edges: self.flow_edges.clone(),
            avatar_summary: summary,
            avatar_confidence: self.avatar.confidence,
        }
    }

    fn analyze_language(&mut self, text: &str) -> Vec<DistillationNode> {
        let zh_count = text.chars().filter(|c| *c >= '\u{4e00}' && *c <= '\u{9fff}').count();
        let total = text.len().max(1);
        let zh_ratio = zh_count as f64 / total as f64;

        self.avatar.language_preference = self.avatar.language_preference * 0.8 + zh_ratio * 0.2;

        let node_id = format!("lang-{}", self.node_counter);
        let label = if zh_ratio > 0.1 { "检测到中文" } else { "检测到英文" };
        let mut edge_source = "root".to_string();

        if let Some(last) = self.flow_nodes.last() {
            edge_source = last.id.clone();
        }

        self.flow_edges.push(DistillationEdge { source: edge_source.clone(), target: node_id.clone() });

        self.node_counter += 1;
        vec![DistillationNode {
            id: node_id,
            label: label.into(),
            status: "completed".into(),
            description: format!("中文占比 {:.0}% · 偏好 {:.0}%", zh_ratio * 100.0, self.avatar.language_preference * 100.0),
            r#type: "sub-agent".into(),
            progress: 1.0,
            ttl_seconds: 3.0,
        }]
    }

    fn analyze_domains(&mut self, text: &str) -> Vec<DistillationNode> {
        let lower = text.to_lowercase();
        let mut nodes = Vec::new();
        let mut detected_domains = Vec::new();

        for (domain, keywords) in &self.domain_keywords {
            let matches: Vec<_> = keywords.iter().filter(|kw| lower.contains(*kw)).collect();
            if !matches.is_empty() {
                let score = matches.len() as f64 * 0.2;
                let entry = self.avatar.domain_scores.entry(domain.clone()).or_insert(0.0);
                let prev = *entry;
                *entry = (*entry * 0.7 + score.min(1.0) * 0.3).min(1.0);
                detected_domains.push((domain.clone(), *entry, prev));

                if !self.tags_contain(domain) {
                    self.avatar.tags.push(domain.clone());
                }
            }
        }

        for (domain, new_score, prev_score) in &detected_domains {
            if new_score - prev_score > 0.05 {
                let node_id = format!("domain-{}-{}", domain, self.node_counter);
                let edge_source = self.flow_nodes.last().map(|n| n.id.clone()).unwrap_or("root".into());
                self.flow_edges.push(DistillationEdge { source: edge_source, target: node_id.clone() });

                nodes.push(DistillationNode {
                    id: node_id,
                    label: format!("领域: {}", domain),
                    status: if *new_score > 0.5 { "completed".into() } else { "running".into() },
                    description: format!("领域知识 {:.0}%", new_score * 100.0),
                    r#type: "sub-agent".into(),
                    progress: *new_score,
                    ttl_seconds: 6.0,
                });
                self.node_counter += 1;
            }
        }

        nodes
    }

    fn analyze_tasks(&mut self, text: &str) -> Vec<DistillationNode> {
        let lower = text.to_lowercase();
        let mut nodes = Vec::new();

        for (task_type, keywords) in &self.task_keywords {
            let matches: Vec<_> = keywords.iter().filter(|kw| lower.contains(*kw)).collect();
            if !matches.is_empty() {
                let score = matches.len() as f64 * 0.3;
                let entry = self.avatar.task_affinity.entry(task_type.clone()).or_insert(0.0);
                *entry = (*entry * 0.7 + score.min(1.0) * 0.3).min(1.0);
            }
        }

        let code_block_count = text.matches("```").count() / 2;
        if code_block_count > 0 {
            self.avatar.technical_depth = (self.avatar.technical_depth * 0.7 + 0.3).min(1.0);

            let node_id = format!("tech-{}", self.node_counter);
            let edge_source = self.flow_nodes.last().map(|n| n.id.clone()).unwrap_or("root".into());
            self.flow_edges.push(DistillationEdge { source: edge_source, target: node_id.clone() });

            for (lang, kws) in &self.code_keywords {
                if kws.iter().any(|kw| lower.contains(kw)) {
                    let lang_entry = self.avatar.domain_scores.entry(lang.clone()).or_insert(0.0);
                    *lang_entry = (*lang_entry * 0.7 + 0.3).min(1.0);
                }
            }

            nodes.push(DistillationNode {
                id: node_id,
                label: "技术深度分析".into(),
                status: "completed".into(),
                description: format!("检测到 {} 个代码块 · 技术深度 {:.0}%", code_block_count, self.avatar.technical_depth * 100.0),
                r#type: "critic".into(),
                progress: self.avatar.technical_depth,
                ttl_seconds: 5.0,
            });
            self.node_counter += 1;

            let avg = self.avatar.task_affinity.values().copied().fold(0.0, |a, b| a + b);
            if avg > 0.5 {
                let primary = self.avatar.task_affinity.iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).expect("result"))
                    .map(|(k, _)| k.clone());
                if let Some(task) = primary {
                    let node_id2 = format!("prefer-{}", self.node_counter);
                    nodes.push(DistillationNode {
                        id: node_id2.clone(),
                        label: format!("任务偏好: {}", task),
                        status: "completed".into(),
                        description: "高频任务类型已识别".into(),
                        r#type: "planner".into(),
                        progress: 1.0,
                        ttl_seconds: 4.0,
                    });
                    let last_id = nodes.iter().rev().nth(1).map(|n| n.id.clone()).unwrap_or(node_id2.clone());
                    self.flow_edges.push(DistillationEdge { source: last_id, target: node_id2.clone() });
                    self.node_counter += 1;
                }
            }
        }

        let word_count = text.split_whitespace().count();
        self.avatar.communication_style = self.avatar.communication_style * 0.8 +
            if word_count > 50 { 0.2 } else if word_count > 20 { 0.1 } else { 0.0 };

        if text.len() > 100 {
            self.avatar.reasoning_depth = (self.avatar.reasoning_depth * 0.8 + 0.2).min(1.0);
        }

        nodes
    }

    fn update_profile(&mut self) -> Vec<DistillationNode> {
        let mut nodes = Vec::new();
        let total_messages = self.message_count;

        let new_confidence = (total_messages as f64 / (total_messages as f64 + 10.0)).min(0.95);
        let diff = new_confidence - self.avatar.confidence;
        self.avatar.confidence = new_confidence;
        self.avatar.edition += 1;

        if diff > 0.03 {
            let summary = self.generate_summary();
            let node_id = format!("avatar-{}", self.node_counter);
            let edge_source = self.flow_nodes.last().map(|n| n.id.clone()).unwrap_or("root".into());
            self.flow_edges.push(DistillationEdge { source: edge_source, target: node_id.clone() });

            nodes.push(DistillationNode {
                id: node_id,
                label: format!("工作分身 v{}", self.avatar.edition),
                status: "completed".into(),
                description: summary,
                r#type: "orchestrator".into(),
                progress: new_confidence,
                ttl_seconds: 10.0,
            });
            self.node_counter += 1;
        }

        nodes
    }

    fn generate_summary(&self) -> String {
        let lang = if self.avatar.language_preference > 0.5 { "中文" } else { "English" };
        let style = if self.avatar.communication_style > 0.6 { "详细" } else if self.avatar.communication_style < 0.3 { "简洁" } else { "适中" };
        let domains: Vec<_> = self.avatar.domain_scores.iter()
            .filter(|(_, v)| **v > 0.3)
            .map(|(k, _)| k.as_str())
            .collect();
        let domain_str = if domains.is_empty() { "未识别" } else { &domains.join(", ") };
        let top_task = self.avatar.task_affinity.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).expect("result"))
            .map(|(k, _)| k.as_str())
            .unwrap_or("未识别");

        format!(
            "语言:{} · 风格:{} · 领域:{} · 任务:{} · 信心度:{:.0}%",
            lang, style, domain_str, top_task, self.avatar.confidence * 100.0
        )
    }

    fn tags_contain(&self, tag: &str) -> bool {
        self.avatar.tags.iter().any(|t| t == tag)
    }

    fn cleanup_expired_nodes(&mut self) {
        self.flow_nodes.retain(|n| n.ttl_seconds > 0.0 || n.status == "completed");
        let active_ids: Vec<String> = self.flow_nodes.iter().map(|n| n.id.clone()).collect();
        self.flow_edges.retain(|e| active_ids.contains(&e.source) && active_ids.contains(&e.target));
    }

    pub fn set_identity(&mut self, name: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        match &mut self.identity {
            Some(existing) if existing.name == name => {
                existing.updated_at = now;
                existing.edition += 1;
                let _ = existing.save();
                self.avatar.edition = existing.edition;
            }
            _ => {
                let id = AvatarIdentity::new(name);
                self.avatar.edition = id.edition;
                let _ = id.save();
                self.identity = Some(id);
            }
        }
        self.avatar.identity_name = name.to_string();
        let _ = self.chain.save();
    }

    pub fn record_brain_response(&mut self, response_text: &str) -> usize {
        let secret = self.identity.as_ref().map(|i| i.secret()).unwrap_or_default();
        self.chain.push(response_text.as_bytes(), &secret, MessageDirection::Inbound, "brain");
        self.avatar.chain_length = self.chain.len();
        if let Some(ref mut id) = self.identity {
            id.edition += 1;
            self.avatar.edition = id.edition;
            let _ = id.save();
        }
        let _ = self.chain.save();
        self.chain.len()
    }

    pub fn get_avatar(&self) -> &UserAvatar {
        &self.avatar
    }

    pub fn get_flow(&self) -> DistillationFlowEvent {
        DistillationFlowEvent {
            nodes: self.flow_nodes.clone(),
            edges: self.flow_edges.clone(),
            avatar_summary: self.avatar.summary.clone(),
            avatar_confidence: self.avatar.confidence,
        }
    }

    pub fn brain_write_back(&mut self, text: &str) -> usize {
        let secret = self.identity.as_ref().map(|i| i.secret()).unwrap_or_default();
        self.chain.push(text.as_bytes(), &secret, MessageDirection::Inbound, "brain");
        let _ = self.chain.save();
        self.chain.len()
    }

    /// 自动蒸馏 — 在无用户输入时将分身能力快照写入链，供主脑迭代经验
    pub fn auto_distill(&mut self) -> String {
        let snapshot = serde_json::json!({
            "type": "auto_distill",
            "edition": self.avatar.edition,
            "confidence": self.avatar.confidence,
            "language_preference": self.avatar.language_preference,
            "communication_style": self.avatar.communication_style,
            "reasoning_depth": self.avatar.reasoning_depth,
            "technical_depth": self.avatar.technical_depth,
            "domain_scores": self.avatar.domain_scores,
            "task_affinity": self.avatar.task_affinity,
            "total_messages_processed": self.avatar.total_messages_processed,
            "chain_length": self.chain.len(),
            "summary": self.avatar.summary,
        });
        let json_str = serde_json::to_string(&snapshot).unwrap_or_default();
        let secret = self.identity.as_ref().map(|i| i.secret()).unwrap_or_default();
        self.chain.push(json_str.as_bytes(), &secret, MessageDirection::Outbound, "avatar");
        let _ = self.chain.save();
        json_str
    }

    /// 请求主脑授权某种能力
    pub fn request_capability(&mut self, capability: &str, reasoning: &str) -> AuthRequest {
        let mut reqs = load_auth_requests();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let req = AuthRequest {
            capability: capability.to_string(),
            timestamp: now,
            reasoning: reasoning.to_string(),
            granted: None,
            response_time: None,
        };
        reqs.push(req.clone());
        save_auth_requests(&reqs);
        let secret = self.identity.as_ref().map(|i| i.secret()).unwrap_or_default();
        let chain_data = format!("auth_request:{}|{}|{}", capability, now, reasoning);
        self.chain.push(chain_data.as_bytes(), &secret, MessageDirection::Outbound, "avatar");
        let _ = self.chain.save();
        req
    }

    /// 检查某项能力是否已授权
    pub fn check_auth(&self, capability: &str) -> bool {
        let caps = load_capabilities();
        caps.iter().any(|c| c.name == capability && c.granted)
    }

    /// 主脑授予能力
    pub fn grant_capability(&mut self, capability: &str) -> bool {
        let mut caps = load_capabilities();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if let Some(existing) = caps.iter_mut().find(|c| c.name == capability) {
            existing.granted = true;
            existing.grant_timestamp = now;
        } else {
            caps.push(BrainCapability {
                name: capability.to_string(),
                granted: true,
                grant_timestamp: now,
                expiry: None,
            });
        }
        save_capabilities(&caps);
        let secret = self.identity.as_ref().map(|i| i.secret()).unwrap_or_default();
        let chain_data = format!("auth_grant:{}|{}", capability, now);
        self.chain.push(chain_data.as_bytes(), &secret, MessageDirection::Inbound, "brain");
        let _ = self.chain.save();
        // update pending auth requests
        let mut reqs = load_auth_requests();
        for req in reqs.iter_mut().rev().filter(|r| r.capability == capability && r.granted.is_none()) {
            req.granted = Some(true);
            req.response_time = Some(now);
        }
        save_auth_requests(&reqs);
        true
    }

    /// 撤销能力
    pub fn revoke_capability(&mut self, capability: &str) -> bool {
        let mut caps = load_capabilities();
        if let Some(existing) = caps.iter_mut().find(|c| c.name == capability) {
            existing.granted = false;
            save_capabilities(&caps);
            let secret = self.identity.as_ref().map(|i| i.secret()).unwrap_or_default();
            let chain_data = format!("auth_revoke:{}|{}", capability,
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
            self.chain.push(chain_data.as_bytes(), &secret, MessageDirection::Inbound, "brain");
            let _ = self.chain.save();
            return true;
        }
        false
    }
}

// ========== Capability Authorization System ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub capability: String,
    pub timestamp: i64,
    pub reasoning: String,
    pub granted: Option<bool>,
    pub response_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainCapability {
    pub name: String,
    pub granted: bool,
    pub grant_timestamp: i64,
    pub expiry: Option<i64>,
}

fn auth_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".neotrix")
}

fn load_capabilities() -> Vec<BrainCapability> {
    let path = auth_dir().join("capabilities.json");
    if path.exists() {
        std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_capabilities(caps: &[BrainCapability]) {
    let dir = auth_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(caps) {
        let _ = std::fs::write(dir.join("capabilities.json"), &json);
    }
}

fn load_auth_requests() -> Vec<AuthRequest> {
    let path = auth_dir().join("auth_requests.json");
    if path.exists() {
        std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_auth_requests(reqs: &[AuthRequest]) {
    let dir = auth_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(reqs) {
        let _ = std::fs::write(dir.join("auth_requests.json"), &json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_construction() {
        let avatar = UserAvatar::default();
        assert_eq!(avatar.identity_name, "");
        assert_eq!(avatar.edition, 0);
        assert_eq!(avatar.confidence, 0.0);
        assert_eq!(avatar.language_preference, 0.5);
        assert_eq!(avatar.communication_style, 0.5);
        assert_eq!(avatar.reasoning_depth, 0.5);
        assert_eq!(avatar.technical_depth, 0.5);
        assert!(avatar.domain_scores.is_empty());
        assert!(avatar.task_affinity.is_empty());
        assert!(avatar.knowledge_affinity.is_empty());
        assert_eq!(avatar.total_messages_processed, 0);
        assert_eq!(avatar.chain_length, 0);
        assert!(avatar.tags.is_empty());
    }

    #[test]
    fn test_with_name() {
        let avatar = UserAvatar::with_name("test-user");
        assert_eq!(avatar.identity_name, "test-user");
        assert_eq!(avatar.edition, 0);
        assert_eq!(avatar.confidence, 0.0);
        assert_eq!(avatar.language_preference, 0.5);
        assert!(avatar.domain_scores.is_empty());
    }

    #[test]
    fn test_domain_score_initialization() {
        let avatar = UserAvatar::default();
        assert!(avatar.domain_scores.is_empty());
        assert_eq!(avatar.domain_scores.get("rust"), None);
    }

    #[test]
    fn test_edition_tracking() {
        let mut avatar = UserAvatar::default();
        assert_eq!(avatar.edition, 0);
        avatar.edition = 5;
        assert_eq!(avatar.edition, 5);
        avatar.edition += 1;
        assert_eq!(avatar.edition, 6);
    }

    #[test]
    fn test_confidence_field_access() {
        let mut avatar = UserAvatar::default();
        assert_eq!(avatar.confidence, 0.0);
        avatar.confidence = 0.85;
        assert!((avatar.confidence - 0.85).abs() < 1e-6);
    }
}

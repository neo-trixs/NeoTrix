//! Recon Phase — cloudflare/security-audit-skill Phase 1 吸收
//! 
//! 三并行研究代理: 概览/技术栈/基线, 信任边界/认证授权, 输入面清单
//! 输出 architecture.md 注入所有 Phase 2 代理

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// 应用类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApplicationType {
    WebApp,
    Api,
    CliTool,
    Library,
    Daemon,
    DesktopApp,
    MobileBackend,
    Other,
}

/// 信任边界
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustBoundary {
    pub entry_point: String,
    pub input_type: String,
    pub validation_present: bool,
    pub file_path: String,
    pub line_number: usize,
}

/// 认证机制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMechanism {
    pub method: String,
    pub locations: Vec<String>,
    pub bypasses: Vec<String>,
}

/// 授权模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzModel {
    pub enforcement_points: Vec<String>,
    pub permission_checks: Vec<String>,
    pub gaps: Vec<String>,
}

/// 输入面条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSurface {
    pub category: InputCategory,
    pub location: String,
    pub description: String,
    pub dangerous_sinks: Vec<String>,
    pub file_path: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputCategory {
    NetworkHttp,
    NetworkGrpc,
    NetworkWebsocket,
    NetworkTcpUdp,
    FileUpload,
    FileConfig,
    FileLog,
    FileImportExport,
    UserContent,
    ExternalIntegration,
    IpMessageQueue,
    IpSharedMemory,
    IpUnixSocket,
    IpEnvVar,
    IpCliArg,
}

/// 可比较基线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparableBaseline {
    pub name: String,
    pub type_: ApplicationType,
    pub security_tradeoffs: Vec<String>,
    pub known_exploits: Vec<String>,
}

/// 架构摘要 (Phase 1 输出，注入 Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureSummary {
    pub application_type: ApplicationType,
    pub tech_stack: TechStack,
    pub trust_model: TrustModel,
    pub input_surfaces: Vec<InputSurface>,
    pub comparable_baseline: Option<ComparableBaseline>,
    pub key_entry_points: Vec<String>,
    pub prior_findings_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub runtime: String,
    pub deployment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustModel {
    pub actors: Vec<String>,
    pub permissions: HashMap<String, Vec<String>>,
    pub enforcement_code: Vec<String>,
    pub boundaries: Vec<TrustBoundary>,
    pub auth: Vec<AuthMechanism>,
    pub authz: Vec<AuthzModel>,
    pub privilege_separation: String,
}

/// Recon Phase 引擎
#[derive(Debug)]
pub struct ReconPhase {
    project_root: PathBuf,
}

impl ReconPhase {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// 运行三并行研究代理
    pub async fn run(&self) -> ArchitectureSummary {
        // 实际实现中应启动三个并行 subagent
        // 这里简化为顺序执行
        
        let (overview, trust, input_surface) = tokio::join!(
            self.agent_overview(),
            self.agent_trust_boundaries(),
            self.agent_input_surface(),
        );
        
        ArchitectureSummary {
            application_type: overview.application_type,
            tech_stack: overview.tech_stack,
            trust_model: trust,
            input_surfaces: input_surface,
            comparable_baseline: overview.comparable_baseline,
            key_entry_points: overview.key_entry_points,
            prior_findings_summary: None, // 可加载 prior runs
        }
    }

    async fn agent_overview(&self) -> AgentOverviewResult {
        // 模拟 Agent 1a
        AgentOverviewResult {
            application_type: ApplicationType::WebApp,
            tech_stack: TechStack {
                languages: vec!["Rust".to_string()],
                frameworks: vec!["Axum".to_string()],
                databases: vec!["SQLite".to_string()],
                runtime: "Tokio".to_string(),
                deployment: "Docker".to_string(),
            },
            comparable_baseline: Some(ComparableBaseline {
                name: "Typical Rust Web API".to_string(),
                type_: ApplicationType::Api,
                security_tradeoffs: vec!["No WAF".to_string(), "Custom auth".to_string()],
                known_exploits: vec!["SQLi in similar codebases".to_string()],
            }),
            key_entry_points: vec!["src/main.rs".to_string(), "src/api/".to_string()],
        }
    }

    async fn agent_trust_boundaries(&self) -> TrustModel {
        // 模拟 Agent 1b
        TrustModel {
            actors: vec!["Anonymous".to_string(), "Authenticated".to_string(), "Admin".to_string()],
            permissions: HashMap::new(),
            enforcement_code: vec!["src/middleware/auth.rs".to_string()],
            boundaries: vec![],
            auth: vec![],
            authz: vec![],
            privilege_separation: "Runs as non-root user".to_string(),
        }
    }

    async fn agent_input_surface(&self) -> Vec<InputSurface> {
        // 模拟 Agent 1c
        vec![]
    }

    /// 写入 architecture.md
    pub fn write_architecture_md(&self, summary: &ArchitectureSummary) -> Result<(), std::io::Error> {
        let output_dir = self.project_root.join("security-audit-skill").join("run-1");
        std::fs::create_dir_all(&output_dir)?;
        
        let content = serde_json::to_string_pretty(summary)?;
        std::fs::write(output_dir.join("architecture.md"), content)?;
        
        Ok(())
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let phase = ReconPhase::new(temp_dir.path().to_path_buf());
        
        // Test architecture summary serialization
        let summary = ArchitectureSummary {
            application_type: ApplicationType::WebApp,
            tech_stack: TechStack {
                languages: vec!["Rust".to_string()],
                frameworks: vec![],
                databases: vec![],
                runtime: "Tokio".to_string(),
                deployment: "Docker".to_string(),
            },
            trust_model: TrustModel {
                actors: vec![],
                permissions: HashMap::new(),
                enforcement_code: vec![],
                boundaries: vec![],
                auth: vec![],
                authz: vec![],
                privilege_separation: "".to_string(),
            },
            input_surfaces: vec![],
            comparable_baseline: None,
            key_entry_points: vec![],
            prior_findings_summary: None,
        };
        
        let json = serde_json::to_string(&summary).map_err(|e| e.to_string())?;
        assert!(json.contains("WebApp"));
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentOverviewResult {
    application_type: ApplicationType,
    tech_stack: TechStack,
    comparable_baseline: Option<ComparableBaseline>,
    key_entry_points: Vec<String>,
}

impl Default for ReconPhase {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recon_phase_creation() {
        let phase = ReconPhase::new(PathBuf::from("."));
        assert_eq!(phase.project_root, PathBuf::from("."));
    }

    #[test]
    fn test_architecture_summary_serialization() {
        let summary = ArchitectureSummary {
            application_type: ApplicationType::Api,
            tech_stack: TechStack {
                languages: vec!["Rust".to_string()],
                frameworks: vec![],
                databases: vec![],
                runtime: "".to_string(),
                deployment: "".to_string(),
            },
            trust_model: TrustModel {
                actors: vec![],
                permissions: HashMap::new(),
                enforcement_code: vec![],
                boundaries: vec![],
                auth: vec![],
                authz: vec![],
                privilege_separation: "".to_string(),
            },
            input_surfaces: vec![],
            comparable_baseline: None,
            key_entry_points: vec![],
            prior_findings_summary: None,
        };
        
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("Api"));
    }

    #[test]
    fn test_self_test_passes() {
        assert!(ReconPhase::self_test().is_ok());
    }
}
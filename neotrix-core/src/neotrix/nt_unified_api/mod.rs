//! # NT-UNIFIED-API: 统一意识核心交互接口
//!
//! 前端只需与这个统一接口对话，不再直接调用 100+ 个 Tauri 命令。
//! 所有复杂的内部路由、层间协调、能力调度都在核心内部完成。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 统一请求：人类意图的结构化表达
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    /// 会话 ID（可选，新会话可为空）
    pub session_id: Option<String>,
    /// 用户输入文本
    pub input: String,
    /// 附加上下文（文件、选中代码、项目信息等）
    pub context: Option<RequestContext>,
    /// 期望的响应模式
    pub mode: ResponseMode,
    /// 是否流式响应
    pub stream: bool,
}

/// 请求上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// 当前项目路径
    pub project_path: Option<String>,
    /// 选中的文件列表
    pub selected_files: Vec<String>,
    /// 选中的代码片段
    pub selected_code: Option<String>,
    /// 当前打开的文件
    pub open_file: Option<String>,
    /// Git 状态
    pub git_status: Option<String>,
    /// 附加元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 响应模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
    /// 普通对话/问答
    Chat,
    /// 代码生成/编辑
    Code,
    /// 架构设计/决策
    Design,
    /// 问题诊断/调试
    Diagnose,
    /// 学习/解释
    Explain,
    /// 自动化执行
    Execute,
    /// 知识检索/吸收
    Knowledge,
    /// 系统内省/审查
    Introspect,
}

/// 统一响应：意识核心的结构化输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    /// 响应 ID
    pub response_id: String,
    /// 关联的会话 ID
    pub session_id: String,
    /// 主要文本内容
    pub content: String,
    /// 结构化数据载荷
    pub payload: Option<ResponsePayload>,
    /// 消息类型
    pub message_type: MessageType,
    /// 执行元数据
    pub metadata: ResponseMetadata,
    /// 是否为流式片段
    pub is_stream_chunk: bool,
    /// 流式完成标记
    pub stream_done: bool,
}

/// 响应载荷类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponsePayload {
    /// 代码变更建议
    CodeChanges {
        changes: Vec<CodeChange>,
    },
    /// 文件操作
    FileOps {
        operations: Vec<FileOperation>,
    },
    /// 终端命令
    TerminalCommands {
        commands: Vec<TerminalCommand>,
    },
    /// 知识图谱查询结果
    KnowledgeGraph {
        nodes: Vec<GraphNode>,
        edges: Vec<GraphEdge>,
    },
    /// 能力树状态
    CapabilityTree {
        domains: Vec<DomainCapability>,
    },
    /// 系统健康快照
    HealthSnapshot {
        phi: f64,
        coherence: f64,
        gwt_resonance: f64,
        modules: Vec<ModuleHealth>,
    },
    /// 进化计划
    EvolutionPlan {
        actions: Vec<EvolutionAction>,
    },
    /// 任务执行结果
    TaskResult {
        task_id: String,
        success: bool,
        output: String,
        artifacts: Vec<Artifact>,
    },
    /// 会话列表
    Sessions {
        sessions: Vec<SessionInfo>,
    },
    /// 提供商状态
    Providers {
        providers: Vec<ProviderStatus>,
    },
    /// 通用键值数据
    KeyValue {
        data: HashMap<String, serde_json::Value>,
    },
}

/// 代码变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub change_type: CodeChangeType,
    pub old_content: Option<String>,
    pub new_content: String,
    pub explanation: String,
}

/// 代码变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeChangeType {
    Create,
    Modify,
    Delete,
    Move,
}

/// 文件操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub op: FileOpType,
    pub path: String,
    pub content: Option<String>,
}

/// 文件操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOpType {
    Read,
    Write,
    Delete,
    List,
    Search,
}

/// 终端命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCommand {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub description: String,
    pub requires_approval: bool,
}

/// 图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 图边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: f64,
}

/// 域能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCapability {
    pub domain: String,
    pub capabilities: Vec<CapabilityInfo>,
}

/// 能力信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub id: String,
    pub name: String,
    pub maturity: String, // C0-C6
    pub provides: Vec<String>,
    pub health: f64,
}

/// 模块健康
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealth {
    pub name: String,
    pub layer: String, // L1-L6
    pub healthy: bool,
    pub message: Option<String>,
}

/// 进化动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionAction {
    pub action_type: String, // bud/graft/strengthen/mature/prune
    pub node_id: String,
    pub domain: String,
    pub rationale: String,
}

/// 产物
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub artifact_type: String,
    pub path: Option<String>,
    pub content: Option<String>,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub project_path: Option<String>,
}

/// 提供商状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub available: bool,
    pub models: Vec<String>,
    pub last_used: Option<String>,
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// 普通文本回复
    Text,
    /// 结构化数据
    Data,
    /// 进度更新
    Progress,
    /// 错误
    Error,
    /// 请求确认/审批
    ApprovalRequired,
    /// 系统事件
    SystemEvent,
}

/// 响应元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// 处理耗时(ms)
    pub duration_ms: u64,
    /// 涉及的层
    pub layers_involved: Vec<String>,
    /// 使用的能力
    pub capabilities_used: Vec<String>,
    /// 意识状态快照
    pub consciousness_state: ConsciousnessState,
    /// 置信度
    pub confidence: f64,
}

/// 意识状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub phi: f64,
    pub coherence: f64,
    pub gwt_resonance: f64,
    pub emotion: String,
    pub attention_focus: Vec<String>,
}

/// 统一 API 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub recoverable: bool,
    pub suggested_actions: Vec<String>,
}

/// 统一 API 核心 trait
#[async_trait::async_trait]
pub trait UnifiedApi: Send + Sync {
    /// 处理统一请求
    async fn handle(&self, request: UnifiedRequest) -> Result<UnifiedResponse, UnifiedError>;

    /// 流式处理（返回异步流）
    async fn handle_stream(
        &self,
        request: UnifiedRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<UnifiedResponse, UnifiedError>>, UnifiedError>;

    /// 获取系统状态快照
    async fn get_system_state(&self) -> Result<UnifiedResponse, UnifiedError>;

    /// 创建新会话
    async fn create_session(&self, project_path: Option<String>) -> Result<SessionInfo, UnifiedError>;

    /// 列出会话
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, UnifiedError>;

    /// 删除会话
    async fn delete_session(&self, session_id: &str) -> Result<(), UnifiedError>;
}

/// 统一 API 实现入口
pub struct UnifiedApiImpl {
    // 内部持有各层的句柄
    // 实际实现中会注入 L1-L6 的各个子系统
}

impl UnifiedApiImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl UnifiedApi for UnifiedApiImpl {
    async fn handle(&self, request: UnifiedRequest) -> Result<UnifiedResponse, UnifiedError> {
        // TODO: 实现核心路由逻辑
        // 1. 解析意图 -> 确定 ResponseMode
        // 2. 通过 GWT 注意力路由到对应层
        // 3. 调度能力执行
        // 4. 聚合结果返回
        
        Ok(UnifiedResponse {
            response_id: uuid::Uuid::new_v4().to_string(),
            session_id: request.session_id.unwrap_or_default(),
            content: "统一 API 待实现".to_string(),
            payload: None,
            message_type: MessageType::Text,
            metadata: ResponseMetadata {
                duration_ms: 0,
                layers_involved: vec![],
                capabilities_used: vec![],
                consciousness_state: ConsciousnessState {
                    phi: 0.0,
                    coherence: 0.0,
                    gwt_resonance: 0.0,
                    emotion: "neutral".to_string(),
                    attention_focus: vec![],
                },
                confidence: 0.0,
            },
            is_stream_chunk: false,
            stream_done: true,
        })
    }

    async fn handle_stream(
        &self,
        _request: UnifiedRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<UnifiedResponse, UnifiedError>>, UnifiedError> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            let _ = tx.send(Ok(UnifiedResponse {
                response_id: uuid::Uuid::new_v4().to_string(),
                session_id: "".to_string(),
                content: "流式待实现".to_string(),
                payload: None,
                message_type: MessageType::Text,
                metadata: ResponseMetadata {
                    duration_ms: 0,
                    layers_involved: vec![],
                    capabilities_used: vec![],
                    consciousness_state: ConsciousnessState {
                        phi: 0.0,
                        coherence: 0.0,
                        gwt_resonance: 0.0,
                        emotion: "neutral".to_string(),
                        attention_focus: vec![],
                    },
                    confidence: 0.0,
                },
                is_stream_chunk: true,
                stream_done: true,
            })).await;
        });
        Ok(rx)
    }

    async fn get_system_state(&self) -> Result<UnifiedResponse, UnifiedError> {
        Ok(UnifiedResponse {
            response_id: uuid::Uuid::new_v4().to_string(),
            session_id: "system".to_string(),
            content: "系统状态".to_string(),
            payload: Some(ResponsePayload::HealthSnapshot {
                phi: 0.0,
                coherence: 0.0,
                gwt_resonance: 0.0,
                modules: vec![],
            }),
            message_type: MessageType::Data,
            metadata: ResponseMetadata {
                duration_ms: 0,
                layers_involved: vec![],
                capabilities_used: vec![],
                consciousness_state: ConsciousnessState {
                    phi: 0.0,
                    coherence: 0.0,
                    gwt_resonance: 0.0,
                    emotion: "neutral".to_string(),
                    attention_focus: vec![],
                },
                confidence: 1.0,
            },
            is_stream_chunk: false,
            stream_done: true,
        })
    }

    async fn create_session(&self, project_path: Option<String>) -> Result<SessionInfo, UnifiedError> {
        Ok(SessionInfo {
            id: uuid::Uuid::new_v4().to_string(),
            title: "新会话".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            message_count: 0,
            project_path,
        })
    }

    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, UnifiedError> {
        Ok(vec![])
    }

    async fn delete_session(&self, _session_id: &str) -> Result<(), UnifiedError> {
        Ok(())
    }
}

/// 便捷构造函数
impl UnifiedRequest {
    pub fn chat(input: impl Into<String>) -> Self {
        Self {
            session_id: None,
            input: input.into(),
            context: None,
            mode: ResponseMode::Chat,
            stream: false,
        }
    }

    pub fn code(input: impl Into<String>, context: RequestContext) -> Self {
        Self {
            session_id: None,
            input: input.into(),
            context: Some(context),
            mode: ResponseMode::Code,
            stream: false,
        }
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            project_path: None,
            selected_files: vec![],
            selected_code: None,
            open_file: None,
            git_status: None,
            metadata: HashMap::new(),
        }
    }
}

impl RequestContext {
    pub fn with_project(mut self, path: String) -> Self {
        self.project_path = Some(path);
        self
    }

    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.selected_files = files;
        self
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.selected_code = Some(code);
        self
    }
}
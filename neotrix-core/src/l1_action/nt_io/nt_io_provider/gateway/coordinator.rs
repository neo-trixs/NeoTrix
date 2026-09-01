use super::super::agent_routing::AgentRoutingTable;
use super::super::factory::LlmProviderType;
use super::super::provider_swap::ProviderSwapManager;
use super::*;

/// 能力自主协调层 — 任务目标驱动的已有能力组合器 (R-P42 强化已有节点)
///
/// 不再为每个任务创建独立管道, 而是将既有节点 (路由表 / 子网格健康 /
/// 画像降级 / ProviderSwapManager / CircuitBreaker) 组装为按任务类型路由的
/// 统一协调入口. 对应周天信息大阵的 "协调" 环节:
///   1. 解析任务类型 → 需要的能力与安全画像 (CapabilityIntent)
///   2. 查路由表 → 该任务默认 provider/model (已有 AgentRoutingTable)
///   3. 健康感知选路 → 偏好健康子网格 (Gap 3)
///   4. 画像降级 → 失败时自动放宽安全级别重试 (Gap 4)
///   5. 失败注入 → ProviderSwapManager 记录, 驱动后续 swap (已有)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CapabilityIntent {
    /// 本地推理 (无网络需求) — 匿名级, 选 Local provider
    LocalReasoning,
    /// 一般推理 — 开放级, 任意 provider
    GeneralReasoning,
    /// 知识检索 — 开放级, 偏免费
    KnowledgeRetrieval,
    /// 敏感数据写入 — 代理级 (Proxied)
    SensitiveWrite,
    /// 匿名通信 — 匿名级 (Anonymous / Tor 可用则 Tor)
    AnonymousCommunication,
    /// 深度分析 (慢速/高成本可接受) — 开放级, 偏 Cloud
    DeepAnalysis,
}

impl CapabilityIntent {
    /// 任务类型 → 所需最低通信安全画像
    pub fn required_profile(&self) -> CommunicationProfile {
        match self {
            Self::LocalReasoning => CommunicationProfile::Anonymous,
            Self::GeneralReasoning => CommunicationProfile::Open,
            Self::KnowledgeRetrieval => CommunicationProfile::Open,
            Self::SensitiveWrite => CommunicationProfile::Proxied,
            Self::AnonymousCommunication => CommunicationProfile::Tor,
            Self::DeepAnalysis => CommunicationProfile::Open,
        }
    }

    /// 任务类型 → 建议的 provider category (None = 无偏好)
    pub fn preferred_category(&self) -> Option<ProviderCategory> {
        match self {
            Self::LocalReasoning => Some(ProviderCategory::Local),
            Self::DeepAnalysis => Some(ProviderCategory::Cloud),
            _ => None,
        }
    }

    /// 从字符串解析任务意图 (CLI / 配置友好)
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "local" | "local_reasoning" | "local-reasoning" => Self::LocalReasoning,
            "general" | "general_reasoning" | "general-reasoning" | "reason" => {
                Self::GeneralReasoning
            }
            "retrieve"
            | "retrieval"
            | "knowledge"
            | "knowledge_retrieval"
            | "knowledge-retrieval" => Self::KnowledgeRetrieval,
            "write" | "sensitive" | "sensitive_write" | "sensitive-write" => Self::SensitiveWrite,
            "anonymous"
            | "anon"
            | "anonymous_communication"
            | "anonymous-communication"
            | "tor" => Self::AnonymousCommunication,
            "deep" | "deep_analysis" | "deep-analysis" | "analysis" => Self::DeepAnalysis,
            _ => return None,
        })
    }

    pub fn all() -> [CapabilityIntent; 6] {
        [
            Self::LocalReasoning,
            Self::GeneralReasoning,
            Self::KnowledgeRetrieval,
            Self::SensitiveWrite,
            Self::AnonymousCommunication,
            Self::DeepAnalysis,
        ]
    }
}

impl std::fmt::Display for CapabilityIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::LocalReasoning => "local_reasoning",
            Self::GeneralReasoning => "general_reasoning",
            Self::KnowledgeRetrieval => "knowledge_retrieval",
            Self::SensitiveWrite => "sensitive_write",
            Self::AnonymousCommunication => "anonymous_communication",
            Self::DeepAnalysis => "deep_analysis",
        };
        write!(f, "{}", s)
    }
}

/// 一次协调请求 — 任务目标 + 提示词, 交由 CapabilityCoordinator 自主完成
#[derive(Debug, Clone)]
pub struct CoordinationRequest {
    pub intent: CapabilityIntent,
    pub prompt: String,
    pub model: Option<String>,
}

impl CoordinationRequest {
    pub fn new(intent: CapabilityIntent, prompt: &str) -> Self {
        Self {
            intent,
            prompt: prompt.to_string(),
            model: None,
        }
    }
}

/// 协调结果 — 响应 + 实际使用的安全画像 (供调用方感知降级)
#[derive(Debug)]
pub struct CoordinationOutcome {
    pub response: LlmResponse,
    pub used_profile: CommunicationProfile,
    pub degraded: bool,
    pub provider_name: String,
}

/// 能力自主协调器 — 组合既有节点完成目标任务
pub struct CapabilityCoordinator {
    pub gateway: GatewayV2,
    pub routing: AgentRoutingTable,
    pub swap: ProviderSwapManager,
}

impl CapabilityCoordinator {
    pub fn new(gateway: GatewayV2, routing: AgentRoutingTable, swap: ProviderSwapManager) -> Self {
        Self {
            gateway,
            routing,
            swap,
        }
    }

    /// 组装默认子网格 (匿名本地 + 代理 + 开放), 供未预先组合时使用
    pub fn ensure_default_sub_grids(&self) {
        let grids = self.gateway.list_sub_grids();
        if grids.is_empty() {
            self.gateway.compose_sub_grid(
                "anonymous-local",
                CommunicationProfile::Anonymous,
                false,
            );
            self.gateway
                .compose_sub_grid("proxied-cloud", CommunicationProfile::Proxied, false);
            self.gateway
                .compose_sub_grid("open-all", CommunicationProfile::Open, false);
        }
    }

    /// 任务意图 → 需要的能力清单 (梳理自有能力, D48 跨域能量流)
    pub fn capability_plan(intent: CapabilityIntent) -> Vec<&'static str> {
        match intent {
            CapabilityIntent::LocalReasoning => vec!["local_reasoning", "reason", "generate"],
            CapabilityIntent::GeneralReasoning => vec!["reason", "generate", "coordinate"],
            CapabilityIntent::KnowledgeRetrieval => vec!["retrieve", "search", "reason"],
            CapabilityIntent::SensitiveWrite => vec!["mutate", "send", "verify"],
            CapabilityIntent::AnonymousCommunication => vec!["send", "communicate", "shield"],
            CapabilityIntent::DeepAnalysis => vec!["plan", "decompose", "critique", "simulate"],
        }
    }

    /// 协调执行 — 目标驱动的自主能力组合:
    /// 1. 优先本地/免费 (依据意图偏好的 category)
    /// 2. 健康感知: 偏好健康子网格, 跳过熔断的 provider
    /// 3. 画像降级: 失败自动放宽 (complete_for_profile 内部处理)
    /// 4. 结果注入 swap manager 以驱动长期 failover
    pub async fn coordinate(
        &mut self,
        req: &CoordinationRequest,
    ) -> Result<CoordinationOutcome, LlmError> {
        self.ensure_default_sub_grids();
        let profile = req.intent.required_profile();

        // 2. 若偏好 category 已注册, 直接走该 provider (LocalReasoning → Local)
        let preferred = req.intent.preferred_category();
        let routed_provider = preferred.and_then(|cat| self.find_provider_by_category(cat));
        let provider_name = if let Some(name) = routed_provider {
            name
        } else {
            // 3. 默认走健康感知的子网格选路
            match self.gateway.select_best_for_profile(profile).await {
                Some(name) => name,
                None => {
                    // 无匹配 → 回退默认 (通信始终畅通)
                    self.gateway.default_provider_name()
                }
            }
        };

        let llm_req = LlmRequest {
            model: req.model.clone().unwrap_or_else(|| {
                let (_, model) = self.routing.route_for(provider_name.as_str());
                model.clone()
            }),
            ..LlmRequest::new(provider_name.as_str(), &req.prompt)
        };

        let result = self
            .gateway
            .complete_for_profile_detailed(profile, &llm_req)
            .await;
        match result {
            Ok((resp, actual_profile, actual_provider)) => {
                let degraded = actual_profile != profile;
                if !degraded {
                    self.swap.record_success(
                        LlmProviderType::from_name(provider_name.as_str())
                            .unwrap_or(LlmProviderType::OpenAI),
                        0.0,
                    );
                }
                log::debug!(
                    "[coordinate] {:?} degraded={} ({:?} → {:?})",
                    req.intent,
                    degraded,
                    profile,
                    actual_profile
                );
                Ok(CoordinationOutcome {
                    response: resp,
                    used_profile: actual_profile,
                    degraded,
                    provider_name: actual_provider,
                })
            }
            Err(e) => {
                // 降级已由 complete_for_profile 内部处理; 若仍失败, 记录到 swap manager
                self.swap.record_error(
                    LlmProviderType::from_name(provider_name.as_str())
                        .unwrap_or(LlmProviderType::OpenAI),
                    &e,
                );
                Err(e)
            }
        }
    }

    fn find_provider_by_category(&self, category: ProviderCategory) -> Option<String> {
        self.gateway.providers().into_iter().find(|name| {
            self.gateway
                .category_of(name)
                .map(|c| c == category)
                .unwrap_or(false)
        })
    }
}

impl Default for CapabilityCoordinator {
    fn default() -> Self {
        Self::new(
            GatewayV2::new(),
            AgentRoutingTable::new("default", "default"),
            ProviderSwapManager::new(vec![]),
        )
    }
}
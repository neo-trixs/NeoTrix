# NeoTrix GatewayV2 架构分析报告

## 概述
分析 NeoTrix 模型路由和 GatewayV2 架构，识别冗余、缺陷和跨域错位问题。

## 1. 冗余代码列表

### 1.1 重复的 Provider 实现
| 文件 | 行号 | 冗余描述 |
|------|------|----------|
| `src-tauri/src/llamacpp.rs` | 全文 | 与 `src-tauri/src/domain/plugins/llamacpp.rs` 重复实现 llama.cpp 进程管理 |
| `src-tauri/src/domain/plugins/llamacpp.rs` | 全文 | 与 `src-tauri/src/llamacpp.rs` 重复实现 llama.cpp 进程管理 |
| `neotrix-core/src/l1_action/nt_io/nt_io_provider/free_providers.rs:191-339` | OpenRouterProvider | 与 openai.rs 中的 OpenAiProvider 重复实现 OpenAI 兼容 API |
| `neotrix-core/src/l1_action/nt_io/nt_io_provider/openai.rs` | OpenAiProvider | 与 free_providers.rs 中的 OpenRouterProvider 重复实现 OpenAI 兼容 API |

### 1.2 重复的模型选择逻辑
| 文件 | 行号 | 冗余描述 |
|------|------|----------|
| `gateway/selection.rs:71-123` | `select_best()` | 基于 composite_score 的简单选择 |
| `gateway/learned_router.rs:221-478` | 多个 `route()` 实现 | KNN/MLP/Hybrid 等复杂路由策略 |
| `gateway/market_router.rs:49` | `route()` | 基于市场因素的路由 |
| `gateway/subgrid.rs:57` | `select_best_for_profile()` | 基于 CommunicationProfile 的选择 |

### 1.3 重复的错误处理模式
| 文件 | 行号 | 冗余描述 |
|------|------|----------|
| `free_providers.rs:104-108` | HTTP 状态码映射 | 重复的 LlmError 映射 |
| `free_providers.rs:271-275` | HTTP 状态码映射 | 重复的 LlmError 映射 |
| `gemini.rs:88-93` | HTTP 状态码映射 | 重复的 LlmError 映射 |
| `openai.rs` | HTTP 状态码映射 | 重复的 LlmError 映射 |

## 2. 缺陷列表

### 2.1 缺失的 Provider 能力发现
| 缺陷 | 描述 | 影响 |
|------|------|------|
| 动态能力发现缺失 | GatewayV2 不自动发现 provider 的能力（支持的模型、上下文窗口、工具调用等） | 路由决策基于静态配置，无法适应 provider 能力变化 |
| 模型能力映射缺失 | 无统一的模型能力注册表（如哪些模型支持视觉、工具调用、长上下文） | 路由时无法根据任务需求选择合适模型 |
| 健康检查不完整 | 只检查 provider 是否可达，不检查模型实际可用性 | 可能路由到已下线的模型 |

### 2.2 不完整的故障转移
| 缺陷 | 描述 | 影响 |
|------|------|------|
| 4xx 错误处理不一致 | 有些 provider 将 400 视为瞬时错误，有些视为永久错误 | 故障转移策略不一致 |
| 维护窗口检测不完善 | `is_maintenance_window()` 只检查特定字符串，可能遗漏其他维护提示 | 在维护期间反复重试 |
| 模型级锁与 provider 级锁混淆 | `is_model_unavailable()` 和 `is_maintenance_window()` 的界限不清晰 | 错误的熔断粒度 |

### 2.3 流式响应处理不一致
| 缺陷 | 描述 | 影响 |
|------|------|------|
| 流式解析实现分散 | 每个 provider 独立实现 SSE 解析 | 维护成本高，行为不一致 |
| 错误处理不一致 | 流式响应中的错误处理各不相同 | 用户体验不一致 |
| 背压处理缺失 | 流式响应无背压机制，可能造成内存问题 | 高负载下可能 OOM |

## 3. 跨域错位列表

### 3.1 chat.rs 直接调用 GatewayV2
| 位置 | 描述 | 违反原则 |
|------|------|----------|
| `src-tauri/src/domain/plugins/chat.rs:22-32` | 直接创建和持有 GatewayV2 实例 | 应通过 domain_call 统一调用 |
| `src-tauri/src/domain/plugins/chat.rs:66-83` | GatewayExecutor 直接调用 gateway.complete_single() | 绕过了统一的域调用机制 |
| `src-tauri/src/domain/plugins/chat.rs:215-286` | call_llm_stream 直接调用 gateway.stream_complete_with_selection() | 绕过了统一的域调用机制 |

### 3.2 llamacpp 插件 vs GatewayV2 的 llamacpp provider
| 位置 | 描述 | 违反原则 |
|------|------|----------|
| `src-tauri/src/domain/plugins/llamacpp.rs` | 独立的 llama.cpp 进程管理 | 与 GatewayV2 的 llamacpp provider 重复 |
| `src-tauri/src/llamacpp.rs` | 另一个 llama.cpp 进程管理实现 | 与 llamacpp.rs 插件重复 |
| `neotrix-core/src/l1_action/nt_io/nt_io_provider/factory.rs:932-993` | GatewayV2 的 llamacpp provider 注册 | 与上述两个实现功能重叠 |

### 3.3 agent 插件的 pool_* 命令 vs GatewayV2 的 provider 管理
| 位置 | 描述 | 违反原则 |
|------|------|----------|
| `neotrix-core/src/cli/commands/pool_health_cmds.rs` | 独立的 pool 健康检查 | 与 GatewayV2 的 health.rs 功能重叠 |
| `neotrix-core/src/cli/commands/provider_cmds.rs:192-239` | 独立的 provider 添加命令 | 与 GatewayV2 的 register_provider 功能重叠 |
| `neotrix-core/src/l1_action/nt_io/nt_io_provider/provider_pool.rs:158-160` | 注册到 GatewayV2 | 但仍有独立的 pool 管理逻辑 |

## 4. 重构建议

### 4.1 消除重复实现
1. **合并 llamacpp 实现**
   - 保留 `src-tauri/src/domain/plugins/llamacpp.rs` 作为 DomainPlugin 实现
   - 删除 `src-tauri/src/llamacpp.rs`，将其功能迁移到插件中
   - GatewayV2 的 llamacpp provider 应调用插件的进程管理

2. **统一 OpenAI 兼容 API 实现**
   - 将 `free_providers.rs` 中的 OpenRouterProvider 重构为 OpenAiProvider 的特化版本
   - 使用 trait 统一 OpenAI 兼容 API 的实现

3. **统一错误处理**
   - 创建 `provider_error.rs` 模块，统一 HTTP 状态码到 LlmError 的映射
   - 所有 provider 使用统一的错误处理函数

### 4.2 完善 Provider 能力发现
1. **创建 ProviderCapability trait**
   ```rust
   trait ProviderCapability {
       fn supported_models(&self) -> Vec<String>;
       fn supports_tools(&self) -> bool;
       fn supports_vision(&self) -> bool;
       fn max_context_window(&self) -> usize;
   }
   ```

2. **扩展 ProviderState**
   ```rust
   struct ProviderState {
       // 现有字段
       capability: ProviderCapability,
       last_capability_check: Instant,
   }
   ```

3. **添加动态能力发现**
   - 启动时探测 provider 能力
   - 定期重新探测能力变化
   - 路由时考虑能力匹配

### 4.3 完善故障转移
1. **统一错误分类**
   ```rust
   enum ErrorCategory {
       Transient,      // 网络超时、5xx
       Permanent,      // 4xx 认证错误
       Maintenance,    // 维护窗口
       ModelSpecific,  // 模型不可用
   }
   ```

2. **改进维护窗口检测**
   - 使用更宽松的匹配规则
   - 添加 provider 特定的维护提示

3. **实现分层熔断**
   - 模型级熔断：单个模型不可用
   - Provider 级熔断：整个 provider 不可用
   - 账户级熔断：单个账户配额耗尽

### 4.4 统一流式响应处理
1. **创建统一的 SSE 解析器**
   ```rust
   struct SseParser {
       buffer: String,
       event_type: Option<String>,
   }
   
   impl SseParser {
       fn feed(&mut self, chunk: &str) -> Vec<SseEvent>;
   }
   ```

2. **统一流式错误处理**
   - 定义统一的流式错误类型
   - 实现重试和恢复机制

3. **添加背压控制**
   - 使用 bounded channel 控制流量
   - 实现消费者速度反馈

### 4.5 消除跨域错位
1. **重构 chat.rs**
   - 移除直接的 GatewayV2 实例持有
   - 通过 domain_call 调用 LLM 能力
   - 使用 NT-IO 域的 LLM 服务

2. **统一 provider 管理**
   - 所有 provider 操作通过 GatewayV2
   - CLI 命令调用 GatewayV2 的方法
   - 移除独立的 pool 管理逻辑

## 5. 实施优先级

### P0 (立即修复)
- 合并 llamacpp 重复实现
- 统一 OpenAI 兼容 API 实现
- 修复 chat.rs 的跨域调用

### P1 (短期改进)
- 统一错误处理
- 完善故障转移逻辑
- 统一流式响应处理

### P2 (中期优化)
- 实现 Provider 能力发现
- 完善动态路由策略
- 优化性能和资源使用

### P3 (长期演进)
- 实现自适应路由
- 添加机器学习驱动的路由优化
- 实现跨 provider 负载均衡

## 6. 预期收益

- **代码质量**: 减少约 30% 的重复代码
- **维护成本**: 统一实现降低维护复杂度
- **可靠性**: 完善的故障转移提高系统可用性
- **性能**: 优化的路由策略提高响应速度
- **可扩展性**: 模块化设计便于添加新 provider
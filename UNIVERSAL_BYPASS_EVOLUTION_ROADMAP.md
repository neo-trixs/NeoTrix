# Universal Rate-Limit Bypass — Evolution Iteration Roadmap

> **Cycle 1089 | 2026-09-10 | NT-IO Domain**

## 核心目标

通用破限制架构适用所有外部 LLM 模型, 实现:
- **聚焦冗余**: 消除功能重叠的组件
- **扁平缺陷**: 修复每个组件的具体缺陷
- **跨域错位**: 修复 NT-SHIELD ↔ NT-IO 断裂

---

## Phase 1: 冗余清理 (Redundancy Elimination)

### 1.1 ResponseCache → CacheRouter 合并

**问题**: ResponseCache 和 CacheRouter 功能重叠
**方案**: 删除 ResponseCache, 统一到 CacheRouter
**文件**: `gateway/response_cache.rs`, `gateway/mod.rs`
**收益**: 减少 500+ 行代码, 统一缓存逻辑

```rust
// 删除: gateway/mod.rs 中的 mod response_cache + pub use response_cache::*
// 删除: gateway/response_cache.rs
// 修改: GatewayV2 中的 response_cache 字段 → 使用 cache_router
```

### 1.2 RateLimiter → TokenBucketLimiter 合并

**问题**: RateLimiter 和 TokenBucketLimiter 功能重叠
**方案**: 删除 RateLimiter, 统一到 TokenBucketLimiter
**文件**: `rate_limiter.rs`, `gateway/mod.rs`
**收益**: 统一速率限制逻辑, 支持 RPM + TPM 双桶

### 1.3 IpRotator → WarpIpRotator 合并

**问题**: IpRotator 和 WarpIpRotator 功能重叠
**方案**: 删除 IpRotator, 统一到 WarpIpRotator
**文件**: `ip_rotator.rs`, `gateway/mod.rs`
**收益**: 统一 IP 旋转逻辑, 支持 WARP 隧道重置

### 1.4 CircuitBreaker → SmartRetryManager 合并

**问题**: 现有 CircuitBreaker 和 SmartRetryManager 功能重叠
**方案**: 删除旧 CircuitBreaker, 统一到 SmartRetryManager
**文件**: `circuit_breaker.rs`, `gateway/mod.rs`
**收益**: 统一重试/熔断逻辑, 支持 exponential backoff + jitter

---

## Phase 2: 扁平缺陷修复 (Flat Defect Fixes)

### 2.1 SessionPool 缺陷

**缺陷**: 429 检测依赖 HTTP status code, 但 OpenCode Zen 返回 silent 429
**修复**: 增加内容检测 (拒绝模式 + 错误负载)
**文件**: `gateway/session_pool.rs`

```rust
// 增加: is_silent_429(content) 检测
// 增加: retry_budget (最多 3 次重试)
// 增加: adaptive_cooldown (指数退避)
```

### 2.2 CacheRouter 缺陷

**缺陷**: SWA-aware caching 依赖粗略 token 估算 (chars/4)
**修复**: 集成 tokenizer 进行精确 token 计数
**文件**: `gateway/cache_router.rs`

```rust
// 增加: use tiktoken_rs 或 fastembed 进行精确 token 计数
// 增加: prefix_token_count 精确计算
// 增加: cache_key 基于 token hashes 而非 char hashes
```

### 2.3 WarpIpRotator 缺陷

**缺陷**: WARP 接口检查依赖系统命令 (wg show), 可能不可用
**修复**: 增加 fallback 检测机制
**文件**: `gateway/warp_rotator.rs`

```rust
// 增加: check_warp_available() fallback (检查网络接口)
// 增加: mock 模式用于测试
// 增加: 增加 rotation_stats 持久化到 SQLite
```

### 2.4 RefusalDetector 缺陷

**缺陷**: 拒绝检测依赖硬编码模式, 容易被绕过
**修复**: 增加动态模式学习
**文件**: `gateway/refusal_detector.rs`

```rust
// 增加: 从历史拒绝中学习新模式
// 增加: 模式相似度匹配 (而非精确匹配)
// 增加: 用户自定义拒绝模式
```

### 2.5 TokenBucketLimiter 缺陷

**缺陷**: 滑动窗口模式未实现 per-key limiting
**修复**: 完善 sliding window per-key
**文件**: `gateway/token_bucket.rs`

```rust
// 修复: sliding window 模式支持 per-key
// 增加: sliding window 清理过期条目
// 增加: 统计 per-key 使用率
```

### 2.6 KvCacheOptimizer 缺陷

**缺陷**: 压缩/解压使用简化 mean pooling, 未实现真实 MLA projection
**修复**: 实现真实 MLA projection matrices
**文件**: `gateway/kv_cache_optimizer.rs`

```rust
// 修复: 实现 W_DQ, W_UK, W_UV projection matrices
// 增加: 真实 MLA 压缩/解压
// 增加: 压缩率验证 (70 KB/token target)
```

### 2.7 SmartRetryManager 缺陷

**缺陷**: 错误分类未考虑 provider-specific 错误码
**修复**: 增加 provider-specific 错误分类
**文件**: `gateway/smart_retry.rs`

```rust
// 增加: OpenCode Zen specific errors (silent 429, quota exhaustion)
// 增加: Provider-specific retry strategies
// 增加: 错误模式学习
```

---

## Phase 3: 跨域错位修复 (Cross-Domain Misalignment)

### 3.1 NT-SHIELD ↔ NT-IO 桥接

**错位**: StealthMiddleware 未完全集成到执行管道
**修复**: 完善 execution.rs 中的 fingerprint injection
**文件**: `gateway/execution.rs`, `gateway/stealth_middleware.rs`

```rust
// 完善: call_provider() 中的 stealth_middleware.inject_fingerprint()
// 增加: session_pool.dispatch() 与 stealth_middleware 协同
// 增加: cache_router.get_sticky_key() 与 session_pool 关联
```

### 3.2 RotationCoordinator 实时查询

**错位**: Gateway 未实时查询 RotationCoordinator 旋转状态
**修复**: 增加 RotationCoordinator 查询接口
**文件**: `gateway/mod.rs`, `rotation_coordinator.rs`

```rust
// 增加: GatewayV2.rotation_coordinator 字段
// 增加: 查询当前旋转状态
// 增加: 旋转事件回调
```

### 3.3 SessionPool ↔ CacheRouter 协同

**错位**: SessionPool 和 CacheRouter 未共享 session 信息
**修复**: 增加 session 信息共享
**文件**: `gateway/session_pool.rs`, `gateway/cache_router.rs`

```rust
// 增加: session_pool.get_session_info() 返回 fingerprint + provider
// 增加: cache_router.set_sticky_key() 基于 session info
// 增加: cache_router.get_swa_aware() 考虑 session fingerprint
```

### 3.4 ComplexityRouter ↔ TokenBucketLimiter 协同

**错位**: ComplexityRouter 选择模型层级后未考虑速率限制
**修复**: 增加速率限制检查
**文件**: `gateway/complexity_router.rs`, `gateway/token_bucket.rs`

```rust
// 增加: complexity_router.select_tier() 后检查 token_bucket
// 增加: 如果 rate limited, 降级到 lower tier
// 增加: tier 级别的速率限制配置
```

---

## Phase 4: 新能力吸收 (New Capability Absorption)

### 4.1 Prompt Deduplication

**来源**: KVMem + DeepSeek V3
**实现**: 在 CacheRouter 中增加 prompt deduplication
**文件**: `gateway/cache_router.rs`

```rust
// 增加: deduplicate_prompt() 基于 semantic similarity
// 增加: 系统 prompt 缓存 (prefix caching)
// 增加: 用户 prompt 去重 (避免重复计算)
```

### 4.2 Request Fingerprinting Enhancement

**来源**: OmniRoute + grok-bypass
**实现**: 增强 FingerprintSynthesizer
**文件**: `gateway/fingerprint_synth.rs`

```rust
// 增加: HTTP/2 fingerprint (SETTINGS, WINDOW_UPDATE, PRIORITY)
// 增加: TLS extension fingerprint (supported_groups, key_share)
// 增加: JA4+ fingerprint (更细粒度)
```

### 4.3 Provider-Side Cache Optimization

**来源**: LiteLLM + Portkey
**实现**: 优化 provider-side cache hit rate
**文件**: `gateway/cache_router.rs`

```rust
// 增加: provider-specific cache key strategies
// 增加: cache hit rate monitoring
// 增加: cache warming strategies
```

### 4.4 Adaptive Rate Limiting

**来源**: Traefik Hub + TrueFoundry
**实现**: 自适应速率限制
**文件**: `gateway/token_bucket.rs`

```rust
// 增加: 基于历史使用率调整 bucket 容量
// 增加: 预测性速率限制 (基于时间模式)
// 增加: 动态调整 refill rate
```

### 4.5 Cost-Aware Routing

**来源**: Spotify Portal Shunt + KVMem
**实现**: 基于成本的模型路由
**文件**: `gateway/complexity_router.rs`

```rust
// 增加: cost_per_token 配置
// 增加: budget-aware routing (总预算限制)
// 增加: cost optimization (最便宜的可用模型)
```

---

## Phase 5: 架构重构 (Architecture Refactoring)

### 5.1 GatewayV2 字段重组

**问题**: GatewayV2 字段过多 (20+ fields)
**方案**: 使用 trait object 组合
**文件**: `gateway/mod.rs`

```rust
// 重构: 将字段分组到子结构
pub struct GatewayV2 {
    // Core
    providers: RwLock<HashMap<String, Arc<dyn LlmProvider>>>,
    states: RwLock<HashMap<String, ProviderState>>,
    
    // Rate Limiting
    rate_limiter: Box<dyn RateLimiter>,
    
    // Caching
    cache: Box<dyn CacheRouter>,
    
    // Retry/Breaker
    retry: Box<dyn SmartRetryManager>,
    
    // Stealth
    stealth: Box<dyn StealthMiddleware>,
    
    // IP Rotation
    ip_rotator: Box<dyn WarpIpRotator>,
    
    // ... 其他字段
}
```

### 5.2 统一配置接口

**问题**: 每个组件有独立配置
**方案**: 统一 GatewayConfig
**文件**: `gateway/config.rs` (新文件)

```rust
pub struct GatewayConfig {
    pub rate_limiting: RateLimitConfig,
    pub caching: CacheConfig,
    pub retry: RetryConfig,
    pub stealth: StealthConfig,
    pub ip_rotation: IpRotationConfig,
    pub fingerprinting: FingerprintConfig,
    // ...
}
```

### 5.3 统一统计接口

**问题**: 每个组件有独立统计
**方案**: 统一 GatewayStats
**文件**: `gateway/stats.rs` (新文件)

```rust
pub struct GatewayStats {
    pub rate_limiting: RateLimitStats,
    pub caching: CacheStats,
    pub retry: RetryStats,
    pub stealth: StealthStats,
    pub ip_rotation: IpRotationStats,
    // ...
}
```

### 5.4 统一健康检查

**问题**: 每个组件有独立健康检查
**方案**: 统一 HealthCheck trait
**文件**: `gateway/health.rs` (新文件)

```rust
#[async_trait]
pub trait HealthCheck {
    async fn check_health(&self) -> HealthStatus;
    async fn get_metrics(&self) -> Metrics;
}
```

---

## Phase 6: 测试与验证 (Testing & Verification)

### 6.1 单元测试

每个组件增加完整单元测试:
- `session_pool.rs`: 测试多 session 分发 + 429 检测 + cooldown
- `cache_router.rs`: 测试 SWA-aware caching + semantic cache
- `complexity_router.rs`: 测试复杂度评分 + tier 选择
- `fingerprint_synth.rs`: 测试指纹合成 + 轮换
- `warp_rotator.rs`: 测试 WARP 旋转 + IP 验证
- `refusal_detector.rs`: 测试拒绝检测 + 自动修复
- `token_bucket.rs`: 测试 RPM + TPM 双桶 + per-key
- `kv_cache_optimizer.rs`: 测试 MLA 压缩/解压 + tiered caching
- `smart_retry.rs`: 测试 circuit breaker + exponential backoff

### 6.2 集成测试

测试组件间协同:
- SessionPool + CacheRouter: session sticky routing
- ComplexityRouter + TokenBucketLimiter: tier-aware rate limiting
- StealthMiddleware + FingerprintSynthesizer: fingerprint injection
- SmartRetryManager + CircuitBreaker: retry with breaker

### 6.3 端到端测试

测试完整请求管道:
- Mock LLM provider + rate limit simulation
- Verify: session rotation → cache hit → retry → circuit breaker
- Verify: refusal detection → auto-fix → retry with different session

---

## 执行优先级

| Priority | Phase | Task | Impact |
|----------|-------|------|--------|
| P0 | Phase 1 | ResponseCache → CacheRouter 合并 | 高 - 消除冗余 |
| P0 | Phase 1 | RateLimiter → TokenBucketLimiter 合并 | 高 - 统一速率限制 |
| P1 | Phase 2 | SessionPool 429 检测修复 | 高 - 提升成功率 |
| P1 | Phase 2 | CacheRouter SWA-aware 精确 token 计数 | 中 - 提升 cache hit |
| P1 | Phase 3 | StealthMiddleware 完整集成 | 高 - 跨域修复 |
| P2 | Phase 2 | RefusalDetector 动态模式学习 | 中 - 提升检测率 |
| P2 | Phase 4 | Prompt Deduplication | 中 - 提升效率 |
| P2 | Phase 4 | Cost-Aware Routing | 中 - 降低成本 |
| P3 | Phase 5 | GatewayV2 字段重组 | 低 - 代码质量 |
| P3 | Phase 6 | 完整测试覆盖 | 高 - 可靠性 |

---

## 预期收益

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| 429 Rate | ~20% | <5% | 3.75x |
| Cache Hit Rate | ~38% | >70% | 1.84x |
| Retry Success Rate | ~60% | >85% | 1.42x |
| KV Cache Size | 516 KB/token | 70 KB/token | 7.37x |
| Code Redundancy | ~2000 lines | <500 lines | 4x reduction |
| Test Coverage | ~40% | >80% | 2x coverage |

---

## 外部项目吸收清单

| 项目 | 技术 | NeoTrix 映射 | 状态 |
|------|------|-------------|------|
| OmniRoute #2948 | 6指纹×0-50ms抖动 | SessionPool | ✅ 已吸收 |
| grok-bypass | JA3/JA4指纹合成 | FingerprintSynthesizer | ✅ 已吸收 |
| proxy-rotate | WARP IP旋转 | WarpIpRotator | ✅ 已吸收 |
| opencode-ip-rotator | WARP旋转器 | WarpIpRotator | ✅ 已吸收 |
| llm-circuit-breaker | Circuit breaker | SmartRetryManager | ✅ 已吸收 |
| Traefik Hub | Token bucket | TokenBucketLimiter | ✅ 已吸收 |
| DeepSeek V3 MLA | KV cache compression | KvCacheOptimizer | ✅ 已吸收 |
| LiteLLM | ITPM/OTPM limits | TokenBucketLimiter | ✅ 已吸收 |
| Portkey | Semantic caching | CacheRouter | ✅ 已吸收 |
| Spotify Portal Shunt | Cost-aware routing | ComplexityRouter | ✅ 已吸收 |

---

## 经验落盘

本次会话经验已写入:
- 文件: `/tmp/neotrix-session-1089.json`
- 待吸收: `~/.neotrix/pending-absorb.json`
- 后台循环: `nt_mind_background_loop::handlers_absorption` (60s tick)

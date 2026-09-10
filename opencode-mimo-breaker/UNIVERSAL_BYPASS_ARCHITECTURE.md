# Universal Rate-Limit Bypass Architecture

## 适用范围
适用于所有外部 LLM 提供商（OpenAI/Anthropic/Google/Groq/OpenCode Zen/Together/Fireworks/DeepSeek等），不局限于 MiMo。

## 设计原则

### 1. 聚焦冗余 (Focused Redundancy)
**原则**: 每个关注点只有一个事实源。
- Circuit breaker → 只在 `ProviderState` 中（删除 `RecoveryOrchestrator` 中的重复）
- Cache → 统一到 `SemanticCache`（删除 `ResponseCache`）
- Rate control → 只用 `AdaptivePacer` + rate profiles（删除 per-provider `RateLimiter`）
- IP rotation → 只在 `RotationCoordinator` 中（删除 `OsIpRotator`）

### 2. 扁平缺陷 (Flat Defects)
**原则**: 每个子系统必须有全局视角。
- `call_provider_backoff` → 接入 `AdaptivePacer` 全局节拍
- `response_healer` → 增加拒绝/截断/错误检测
- `rate_profiles` → 运行时可更新（`OnceLock` + `set_rate_profile`）
- `free_pool` → 动态发现（`register_provider` + `refresh_from_catalog`）

### 3. 跨域错位 (Cross-domain Misalignment)
**原则**: Gateway、Stealth、Rotation 必须统一编排。
- `StealthMiddleware` → 桥接 NT-SHIELD ↔ NT-IO
- `FingerprintInjector` → 在 `call_provider` 管道中注入指纹
- `SubGrid.RotationProfile` → SubGrid 自带旋转配置
- `RotationCoordinator` → Gateway 直接查询旋转状态

## 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    Universal Rate-Limit Bypass                   │
├─────────────────────────────────────────────────────────────────┤
│  Layer 1: Provider Registry (FreeModelCatalog)                   │
│  - 动态发现 12+ 来源                                              │
│  - 自动注册 rate profiles                                         │
│  - 自动注册 budget caps                                           │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2: Unified Rate Control (AdaptivePacer)                   │
│  - Global pacer derived from provider rate profiles              │
│  - Per-provider rate profiles (runtime-updatable)                │
│  - Remove per-provider RateLimiter (consolidated)                │
├─────────────────────────────────────────────────────────────────┤
│  Layer 3: Stealth Middleware (StealthMiddleware)                  │
│  - Bridges NT-SHIELD stealth_net ↔ NT-IO gateway                 │
│  - Applies fingerprint headers before each call                  │
│  - Applies IP rotation from RotationCoordinator                  │
│  - Applies timing jitter from RotationCoordinator                │
├─────────────────────────────────────────────────────────────────┤
│  Layer 4: Response Healer (Enhanced)                             │
│  - Format healing (existing)                                     │
│  - Refusal detection (new)                                       │
│  - Truncation detection (new)                                    │
│  - Error payload detection (new)                                 │
├─────────────────────────────────────────────────────────────────┤
│  Layer 5: Circuit Breaker (Unified)                              │
│  - Single circuit breaker per provider                           │
│  - Gateway queries ProviderState.circuit_breaker directly        │
│  - RecoveryOrchestrator only handles fallback/escalation         │
└─────────────────────────────────────────────────────────────────┘
```

## 实现清单

| ID | 类别 | 修复项 | 文件 | 优先级 |
|----|------|--------|------|--------|
| R1 | 冗余 | 删除 keyless.rs 硬编码候选 | `keyless.rs:30-34` | P0 |
| R2 | 冗余 | 删除 RecoveryOrchestrator 中的 CircuitBreakerStrategy | `nt_core_error_recovery.rs:429` | P0 |
| R3 | 冗余 | 合并 ResponseCache 到 SemanticCache | `response_cache.rs`, `execution.rs:291-326` | P1 |
| R4 | 冗余 | 删除 per-provider RateLimiter，统一到 AdaptivePacer | `rate_limiter.rs:237-275`, `state.rs:11` | P1 |
| R5 | 冗余 | 合并 OsIpRotator 到 RotationCoordinator | `ip_rotator.rs`, `rotation_coordinator.rs` | P1 |
| F1 | 扁平 | keyless backoff 接入 AdaptivePacer | `keyless.rs:39-61` | P0 |
| F2 | 扁平 | response_healer 增加拒绝/截断检测 | `response_healer.rs:15-29` | P0 |
| F3 | 扁平 | rate_profiles 运行时可更新 | `rate_profiles.rs:18-52` | P1 |
| F4 | 扁平 | free_pool 动态注册 | `free_pool.rs:28-277` | P1 |
| F5 | 扁平 | SelfIteratingStealth 增加 OpenCode 指纹 | `self_iterating.rs:90-134` | P2 |
| C1 | 跨域 | 创建 StealthMiddleware 桥接 NT-SHIELD ↔ NT-IO | 新文件 `stealth_middleware.rs` | P0 |
| C2 | 跨域 | FingerprintInjector 在 call_provider 管道中注入 | `execution.rs:156` | P0 |
| C3 | 跨域 | SubGrid 添加 RotationProfile 字段 | `state.rs:162` | P1 |
| C4 | 跨域 | Gateway 查询 RotationCoordinator | `rotation_coordinator.rs` + `execution.rs` | P1 |

## 实现顺序

### Phase 1: 基础统一 (P0)
1. R1: 删除 keyless 硬编码
2. R2: 统一 circuit breaker
3. F1: keyless backoff 接入 pacer
4. F2: response_healer 增强
5. C1: StealthMiddleware 创建
6. C2: FingerprintInjector 集成

### Phase 2: 运行时升级 (P1)
7. R3: 合并缓存
8. R4: 统一速率控制
9. R5: 合并 IP rotation
10. F3: rate_profiles 运行时
11. F4: free_pool 动态
12. C3: SubGrid RotationProfile
13. C4: Gateway 查询 RotationCoordinator

### Phase 3: 完善 (P2)
14. F5: OpenCode 指纹

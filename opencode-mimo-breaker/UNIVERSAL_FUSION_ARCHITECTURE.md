# Universal Rate-Limit Bypass Fusion Architecture

## 架构来源融合

### 1. MiMo V2.5 架构逆向推理
| 特性 | 值 | 破限策略 |
|------|-----|----------|
| 总参数 | 310B (15B active) | Sparse MoE → 简单任务路由到小模型 |
| 专家数 | 256 routed, top-8 | Expert parallelism → token dispatch优化 |
| 注意力 | Hybrid SWA 5:1, window=128 | KV cache 6×减少 → prefix caching |
| 上下文 | 1M tokens | 长上下文 → context compression |
| 推理 | reasoning_content 字段 | 深度推理 token 独立计费 |
| 定价 | cache hit $0.0028/MTok | 缓存命中节省 98% |

### 2. 外部项目吸收
| 项目 | 核心技术 | NeoTrix 映射 |
|------|----------|-------------|
| OmniRoute #2948 | 6指纹×0-50ms抖动=316 req/s | SessionPool |
| grok-bypass | JA3/JA4指纹合成+session多路复用 | FingerprintSynthesizer |
| LimitlessLLM | 配置化fallback链+per-provider限流 | FallbackChain |
| llm-gateway | Token bucket+cache+failover | 统一网关 |
| Bifrost | 复杂度路由+semantic cache | ComplexityRouter |
| LiteLLM | 虚拟key+weighted rotation | KeyPool |

### 3. 通用破限模式
| 模式 | 原理 | 适用场景 |
|------|------|----------|
| Session Pool | 多session轮换消除指纹检测 | 所有无key端点 |
| Cache-Aware Sticky | 同session固定到同key，保留cache命中 | 所有付费provider |
| Complexity Routing | 简单→小模型，复杂→大模型 | MoE模型族 |
| Multi-Path Fallback | 同模型多provider路径 | 所有provider |
| Token Frame Rebalancing | 匹配合法高流量模式 | 所有provider |

## 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                    Universal Rate-Limit Bypass                   │
├─────────────────────────────────────────────────────────────────┤
│  Layer 1: SessionPool (消除单点速率限制)                          │
│  - 6-10 sessions with unique fingerprints                        │
│  - Round-robin dispatch + 0-50ms jitter                          │
│  - Exponential cooldown on 429                                   │
│  - Per-session metrics tracking                                  │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2: CacheRouter (缓存感知路由)                              │
│  - Sticky routing for same-session requests                      │
│  - Semantic cache (cosine similarity > 0.93)                     │
│  - Prefix caching for static system prompts                      │
├─────────────────────────────────────────────────────────────────┤
│  Layer 3: ComplexityRouter (复杂度路由)                           │
│  - Estimate task complexity from input                           │
│  - Simple→small/fast models, Complex→frontier models             │
│  - MoE expert activation optimization                            │
├─────────────────────────────────────────────────────────────────┤
│  Layer 4: TokenBudgetManager (动态预算分配)                       │
│  - Real-time per-provider token tracking                         │
│  - Auto-rebalance based on provider health                       │
│  - Daily/monthly caps with rollover                              │
├─────────────────────────────────────────────────────────────────┤
│  Layer 5: FingerprintSynthesizer (指纹合成)                      │
│  - JA3/JA4 fingerprint generation                                │
│  - TLS cipher suite rotation                                     │
│  - Target-aware header generation                                │
└─────────────────────────────────────────────────────────────────┘
```

## 实现清单

| ID | 组件 | 优先级 | 文件 |
|----|------|--------|------|
| U1 | SessionPool | P0 | `session_pool.rs` (新) |
| U2 | CacheRouter | P0 | `cache_router.rs` (新) |
| U3 | ComplexityRouter | P1 | `complexity_router.rs` (新) |
| U4 | TokenBudgetManager | P1 | `budget_manager.rs` (新) |
| U5 | FingerprintSynthesizer | P2 | `fingerprint_synth.rs` (新) |

## 冗余清理

| 清理项 | 原因 | 替代 |
|--------|------|------|
| `ResponseCache` + `SemanticCache` 双缓存 | 功能重叠 | 统一到 `CacheRouter` |
| `AdaptivePacer` + `RateLimiter` 双速率控制 | 功能重叠 | 统一到 `SessionPool` |
| `OsIpRotator` + `RotationCoordinator` 双旋转 | 功能重叠 | 统一到 `SessionPool` |
| `keyless.rs` 硬编码候选 | 与 catalog 重复 | 删除，由 catalog 提供 |

## 跨域错位修复

| 错位 | 修复 |
|------|------|
| stealth_net ↔ gateway 分离 | SessionPool 桥接 |
| privacy guard ↔ fingerprint 断裂 | FingerprintSynthesizer 统一 |
| SubGrid ↔ RotationProfile 未链接 | SessionPool 内置 profile |
| RotationCoordinator 对 gateway 死代码 | SessionPool 消费 rotation 状态 |

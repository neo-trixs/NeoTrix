# N6.3 网关概念层复用边界（2026-10-08 P0/设计）

> 草案：尚未写一行代码。目的是在「选型 axum or 手写」之前，先钉住**概念层边界**，防止重造 `neotrix-gateway`。

## 1. 已有的能力面

| 概念 | 仓内落点 | 原职能 |
|---|---|---|
| 统一请求/响应形状 | `crates/neotrix-gateway/src/model_gateway.rs` | `GatewayRequest/GatewayResponse/GatewayError` |
| 成本门 / 月度重置 | 同上 `CostGate` / `CostGateResult` | 预算检查 + 记账 |
| 路由/回退 | `crates/neotrix-gateway/src/model_router.rs` + `model_gateway.rs::FallbackChain` | 能力任务类型 → 模型 → 回退 |
| 服务池 | `ProviderPool` | provider 下限可用性 |
| 本仓 neobot 侧执行面 | `crates/neotrix-neobot/src/nt_routing.rs::build_engine_by_name` | store+name+override+memory → `RoutingEngine` 句柄，已具备 route group/failover/Quota 模式 |
| IM/工具/ledger 计账 | neobot crate | 回执/账本 |

**约束**：已有概念层**不持有 HTTP I/O**；`neobot` crate 已有 axum/tokio 依赖链未拉起，先不引。

## 2. 本窗口推荐边界

```
┌────────────── 新 HTTP 传输层（未来 nt_gateway/http_transport，feature-gated）──────────────┐
│  OpenAI / Anthropic / 本地 chat/completions / Responses / models ⇄ GatewayRequest        │
│  错误映射 7 行（401≠500 / 429→Retry-After / 502≠500） · 流式粗粒度 SSE 协议               │
└────────────── 复用已有概念（只连不造）──────────────────────────────────────────────────┘
                         ▼
neotrix-gateway::model_gateway  （GatewayRequest/Response 统一形状，CostGate，FallbackChain）
                         ▼
neobot::nt_routing::build_engine_by_name() → 路由组 / failover / Quota
                         ▼
neobot::HttpEngine / LocalEchoEngine / CliEngine / …
```

- **契约**：传输层**不**重新实现 CostGate/fallback/quota，只调用现有 wrapper；
- **数据**：账本/审计沿用 neobot 既有 `ledger` + `degraded` 口径（P1-2 已做 outcome_unknown/trimmed）；
- **feature**：`nt_gateway_http` default-off，`check-feature-gates.sh --quick` 覆盖新增门控。

## 3. 端点矩阵（与 TODO-P1LANE §N6.3 对齐）

| 端点 | 方法 | OpenAI 形状 | Anthropic 形状 | 差异维度（系统/usr/助手/工具结果/end 原因/用量/鉴权头…） |
|---|---|---|---|---|
| `/v1/chat/completions` | POST | ✅ | 响应体映射需补 `stop_reason` |
| `/v1/responses` | POST | ✅ | 需适配 item/event |
| `/v1/models` | GET | ✅ | 需填模型归属 |
| `/v1/messages`（Anthropic） | POST | 响应体映射需中转 system 字段 | ✅ |
| `/health` | GET | 本仓 | 本仓 |
| `/v1/embeddings`（可选） | POST | ✅ | n/a |

## 4. 接入策略

1. 新 module：`crates/neotrix-gateway/src/http_transport.rs` + `nt_gateway` 注册品（feature=on-demand） — 不污染 `neobot` lib；
2. neobot bin 增加 `gateway` 子命令（`serve --addr 127.0.0.1:8080 [--model-map ...]`），透传到 router；
3. 真实账本/ledger/outbox 仍写 neobot store⇒ 用户侧 IM 与网关视图一致；
4. CI 门：`check-feature-gates.sh` + `nt_smoke.sh` + 4 端点 curl + 流式 1+错误映射 3 单测。

## 5. 本窗/后续纪律

- 等「axum vs 手写」拍板：**倾向 axum 0.8**（core 已验证同封装），但若主人偏好「仅在需要时引」，允许手写极简 HTTP/1.1 做为 opt-in —— 两者都只是一个传输层 feature，不动概念层。
- 不在本 seed PR 写代码；若拍板落锤后再开 `nt_gateway` feature 门控 PR。

*版本：2026-10-08 初稿（design 阶段，commit 后转 N6.3 开工门）。*

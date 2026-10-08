# neobot 本地协议翻译网关 — 设计（N6.3 P0，2026-10-08）

> **本文只做设计，不含实现。** 目的是在写第一行网关代码之前把三件最贵的事定死：
> **端点矩阵**、**形状差异表**、**错误映射表**。
> ⛔ 前置事实（已实测，不是推测）：`nt_provider.rs:39-40` 明写「anthropic 原生接口非
> OpenAI 兼容（`x-api-key` + `/messages`），本引擎只说 OpenAI 协议，故不收录 anthropic 预设」
> ⇒ **协议翻译能力当前为零**，这是本设计的唯一存在理由。
> ⛔ 复用律：上游调用**只用现有 `HttpEngine`**（已是 OpenAI 兼容客户端，含 SSE 累积、
> `/models` 探活、key 只读 env），**不新建第二套 HTTP 栈**。

## 0. 存在理由与边界

| 要什么 | 现状 | 本设计提供 |
|---|---|---|
| 让 Claude Code / Gemini CLI / Codex 等**只说自家协议**的 agent 用上 neobot 的路由组与本地模型 | `providers`/`route_groups` 只出 **OpenAI 兼容**端点 | 本机 `127.0.0.1:<port>` 上把 Anthropic Messages / OpenAI Responses / Gemini `generateContent` **翻译成** OpenAI Chat Completions，再交给已有 `build_engine_by_name`（路由组 + failover + Quota 模式）|
| 引入新的路由机制 | 已有 `RouteMode{Order,Rotate,Usage,Quota}` + `failover` | **零新增**：网关只是把「入站协议」翻译成既有引擎的调用 |

**明确不做**：① 云端中转/账号共享（Magpie 的订阅登录能力依赖各家 OAuth，**不是本地可算的事实**）；② 请求录制回放（另立）；③ middleware 插件系统（N6.3-P2 只预留钩子，见 §5）。

## 1. 端点矩阵

| 入站协议 | 路径（对外暴露） | 我方下游（已有能力） | 状态 |
|---|---|---|---|
| OpenAI Chat | `POST /v1/chat/completions` | `HttpEngine::chat_body`（`stream:bool`） | **P1**（直穿 + 形状微调）|
| OpenAI Responses | `POST /v1/responses` | 同上（把 `input[]` 折成 messages）| **P2** |
| Anthropic Messages | `POST /v1/messages` | 同上（`system` 提到 system 消息、`tool_use`→`tool_calls`）| **P1**（这是最被需要的一条）|
| Gemini | `POST /v1beta/models/{model}:generateContent` | 同上（`contents[].parts[]` → messages）| **P2** |
| 模型列表 | `GET /v1/models` | `Provider::list_models` | **P1** |
| 健康 | `GET /healthz` | —（只报进程活着与路由组数，**不外联**）| **P1** |

对外只绑 `127.0.0.1`；对外暴露的口子越少，越不破坏 neobot「默认零网络」的承诺。

## 2. 形状差异表（入站 → 内部统一形状）

内部统一形状 = **现有的 `TranscriptItem` + `ToolCall` + `TokenUsage`**（`nt_types.rs`），
因为 `run_turn_with_history` 吃的就是它。

| 维度 | OpenAI Chat | Anthropic Messages | Gemini generateContent |
|---|---|---|---|
| system | `messages[role=system].content` | 顶层 `system`（字符串或块数组）| `systemInstruction{parts[]}` |
| 用户 | `messages[role=user].content`（字符串或数组）| 同左（数组含 image/source 块）| `contents[role=user].parts[]` |
| 助手 | `content` + `tool_calls[]`（`function.arguments` 是 **JSON 字符串**）| `content[]` 块：`text` / `tool_use{id,name,input}` | `parts[]`：`text` / `functionCall{name,args}` |
| 工具结果 | `messages[role=tool].tool_call_id` + `content` | `user` 消息里的 `tool_result{tool_use_id,content}`（**注意：塞在 user 侧**）| `functionResponse{name,response}` |
| 结束原因 | `choices[].finish_reason`（`stop`/`length`/`tool_calls`）| `stop_reason`（`end_turn`/`max_tokens`/`tool_use`/`stop_sequence`）| `finishReason`（`STOP`/`MAX_TOKENS`）|
| 用量 | `usage{prompt_tokens,completion_tokens}` | `usage{input_tokens,output_tokens}` | `usageMetadata{promptTokenCount,candidatesTokenCount}` |
| 鉴权头 | `Authorization: Bearer` | `x-api-key` + `anthropic-version` | `x-goog-api-key`（或 `?key=`）|

**转换纪律（写死在设计里）**：
1. **工具参数一律按 JSON 字符串**双向转换（OpenAI 要字符串，Anthropic/Gemini 要对象）⇒
   `ToolCall.args` 是 `serde_json::Value`，`to_string()` 与 `from_str()` 各一处，**只允许各写一次**。
2. **不把 tool 结果塞进 assistant 侧**；Anthropic 的 `tool_result` 在 user 侧也要**翻译成**
   OpenAI 的 `role=tool` 行，否则下游（我们自己的 `HttpEngine`）根本认不出来。
3. **图片**：`image_url`（OpenAI）↔ base64 source（Anthropic/Gemini）——**P1 不做**，
   遇到非文本块**显式 400**（`unsupported content block`），绝不含糊降级成丢图。
4. **未知字段透传**：入站 JSON 里我们不认识的键**原样回传给下游**（中转常见需求），
   但**不解释**其语义（避免半吊子实现）。

## 3. 错误映射表（`NtBotError` ↔ 各协议）

| 我方/上游语义 | 对外 status | OpenAI error.type | Anthropic error.type | Gemini error.status |
|---|---|---|---|---|
| 上游 401/403（key 无效） | **401** | `authentication_error` | `authentication_error` | `UNAUTHENTICATED` |
| 上游 429 / 本地 `RateLimit` | **429** + `Retry-After` | `rate_limit_error` | `rate_limit_error` | `RESOURCE_EXHAUSTED` |
| 上游 5xx / `Transport` | **502**（**不是 500**：上游坏了≠我坏了）| `api_error` | `api_error` | `UNAVAILABLE` |
| 模型不在端点列表（`probe` 失败）| **404** | `model_not_found` | `not_found_error` | `NOT_FOUND` |
| `ContextOverflow` | **413** | `invalid_request_error` | `request_too_large` | `INVALID_ARGUMENT` |
| 入站形状非法（缺 messages 等）| **400** | `invalid_request_error` | `invalid_request_error` | `INVALID_ARGUMENT` |
| 全路由组都失败（failover 走完）| **503** + `Retry-After` | `api_error` | `overloaded_error` | `UNAVAILABLE` |

⛔ **不许把 401 变 500**：认证失败伪装成服务故障，会让调用方无谓重试且掩盖真因
（这一条已在上轮 `nt_routing` 的 `is_retryable` 分类里有同款纪律：auth 立即上报、不重试）。

## 4. 与路由组的接线（零新增机制）

```
入站请求 → 路径分发
        → 归一化为 (system, history: Vec<TranscriptItem>, prompt)
        → build_engine_by_name(store, 入站 model 名 或 路由组名, model_override, memory)
             ├─ 路由组 → RouteMode 顺序 + 可重试失败 failover（已实现）
             └─ 单端点 → HttpEngine（已实现）
        → EngineTurn → 反向翻译成入站协议的响应形状 + usage + stop_reason
```

**路由组名的解析口径**（必须先定，否则同一个字符串两处解释不同）：
- 入站 `model` 字段**先按路由组名解析**，命中则用组；未命中再按 provider 名；都未命中 ⇒
  **404 `model_not_found`**，且**错误里要列出可用的组名**（调用方能自查，而不是猜）。

## 5. middleware 钩子（P2 只预留，不实现）

对齐 Magpie 的 `onRequest`/`onResponse` 两种钩子，但**只留两个函数指针的位置**，
不做插件加载、不做脚本运行时（Bun/JS）：那是另一个量级的依赖面。
预留形状（伪码，仅说明接线点）：
```rust
trait GatewayHook: Send + Sync {
    fn on_request(&self, body: &mut serde_json::Value) -> Result<(), NtBotError>;
    fn on_response(&self, body: &mut serde_json::Value);
}
```
实现顺序：`model-map`（别名映射）→ `param-override`（字段增删改）→ 其余（不做）。

## 6. 阶段与验收

| 阶段 | 内容 | 验收（测试形态） |
|---|---|---|
| **P0** | 本设计文档 | 端点/形状/错误三表齐全，且每条能指回仓内真实代码或真实协议文档 |
| **P1** | `crates/neotrix-neobot/src/nt_gateway/`：`mod.rs` 分发 + `translate_chat.rs` + `translate_anthropic.rs`；`/v1/chat/completions`、`/v1/messages`、`/v1/models`、`/healthz`；绑 `127.0.0.1` | ① 每端点 1 例请求/响应快照测试；② 跨协议流式 1 例（Anthropic SSE → OpenAI chunk → 累积文本一致）；③ **错误映射逐行测**（401 不许变 500、429 带 `Retry-After`）；④ 形状非法 400 |
| **P2** | Responses / Gemini 端点 + middleware 两个钩子 + per-client gateway key（限额复用 `quota_limits`）| 端点快照各 1 例；限额越界 429 |
| **P3** | 与 `Neo/neobot` 桌面端对接（跨仓，需主人授权）| 桌面端能用 Anthropic 形状的 provider 配置 |

**依赖面纪律**：若 P1 需要引入 HTTP server 依赖（当前 neobot **零 server 依赖**，只有 `ureq` 客户端），
开工前必须先跑 `bash scripts/check-feature-gates.sh`，并在 PR 里写明「新增依赖的用途与可替代方案」。
若不引依赖，可先用 `std::net::TcpListener` 手写极简 HTTP/1.1 解析（测试面小、能避免依赖面扩张）——
**该二选一由主人在开工时拍板**，本文不预设。

## 7. 风险登记

| 风险 | 后果 | 缓解 |
|---|---|---|
| 协议细节漂移（Anthropic 改 SSE 事件名）| 流式半路崩 | 只翻译**已知事件名**，未知事件**跳过并计数**（不猜）|
| 中转导致「看起来成功其实丢字段」| 调用方静默受害 | 往返测试：入站样例 → 内部 → 出站，字段级比对（不只测能通）|
| 网关被当云端代理滥用 | 违背 local-first | 只绑 `127.0.0.1` + per-client key（P2）+ `healthz` 不外联 |
| 与路由组口径分叉 | 同名两义 | §4 已定「先组后 provider」，并要求 404 列出可用组名 |
# Lingee 技术架构（2026-09-22 收割蒸馏）

> 象限: Explanation | 原始数据: `repo-analyses/lingee-20260922/raw/`
> 前端包证据: host `index-GjuiYpHg.js`(2.6MB) + discovery `entry-Brq2AmNs.js`(1.5MB) + CSS token

## 0. 抓取链路（复现用）

- SPA 壳 10KB（`Build Time: 2026-09-20`），未登录渲染即登录墙。
- 真网关 **`https://app.lingee.com/work`**（`/openwork` 别名；`/harness`、`/kchat`、`/api`、`/v1` 均无后端，根路径回退 index.html）。
- 认证 `Authorization: Bearer <openwork.server.token>`（`ows_…`，7 天过期）+ `Accept-Language`。
- 基础设施：华为云 WAF（`HWWAFSESID` 等 3 cookie）、`*.lingee.com` DigiCert 证书、`Server: CW`。

## 1. 前端：微前端联邦

- Host 壳（React 18.3 + antd 6.3.5 + React-Query 5 + zustand 5 + Vite）只做路由/鉴权/共享依赖；
  业务 = 远程模块 `discovery/routes`、`ai-notes/routes`、`user-growth/routes`，
  入口 `/discovery/remoteEntry.js`(69KB) → `./routes → entry-*.js → client-*.js` 三级懒加载，CSS 按需注入（`data-mf-href` 去重）。
- 路由：`explore`、`lab`、`knowledge-base`、`skills`、`agent`、`ceo/*`、`cfo/*`、`chatbot`、`host-modal/*` 等 40+。
- 防护补丁：FileReader 锁定原生实现（防 pptx 库覆盖）、翻译工具 DOM 搬移防护（React #11538 方案）、强制 light 主题。

## 2. 网关：单基址 + 三头路由

```
Authorization: Bearer <token>
X-Kwork-Api-Module: work | chat        # 实例画像 kwork/kchat
X-Lingee-Business-Type: work_daily_office | work_business_analysis
                        | work_business_execution | manage_ceo | manage_cfo
```

- 租户链：`tenant/list` → `auth/bootstrap`（locale/region/plan/llmResponseLanguage/clientDownloadUrl/mcpAppProxyUrl/水印一次下发）→ `auth/session`。
- plan 档：`LINGEE_ULTRA`；`allowExternalShare:false`；`previewType: cloudhub`。

## 3. 模型路由（核心抄点）

`GET /work/tenant-providers`（`model_levels_work.json`；`task` 域同一套）：

| 档位 | 系数 | 排序 | 映射厂商 |
|---|---|---|---|
| auto（自动路由） | — | -1 | — |
| fast 快速 | 0.3 | 50 | — |
| expert 专家（多数 Agent 默认） | 0.9 | 270 | — |
| ultra 极致 | 1.2 | 570 | — |
| Qwen3.8-Max / Flash | 1.6 / 0.1 | 870 / 1050 | 通义 |
| GLM-5.2 | 1.2 | 1230 | 智谱 |
| Kimi-K3 / K2.6 | 3.3 / 0.9 | 1410 / 1590 | 月之暗面 |
| DeepSeek-V4-Pro / Flash | 1.0 / 0.3 | 1770 / 1950 | 深度求索 |

- 三元组 `modelLevel/autoRouting/consumptionCoefficient/orderNumber`：前端只暴露档位名，后端映射真实模型，换模型不改 UI。
- 单 Agent 可锁档（`.../agent/model-level?assistantId=`，抽查均为 expert）。
- 会话侧：`harness/session/{id}/prompt_async` + SSE（`X-SSE-Event-Id`）+ MCP `sampling/createMessage` + `ui/update-model-context`；语音 `wss://asr.kdgalaxy.com`，`volcVoiceEnabled:true`。

## 4. 桌面桥（对标 Tauri 用）

- 壳为 **Electron**（`process.versions.electron` 嗅探）+ `window.lingeeBridge`（`getConfig/emit/notify`，含 `token-expired` 回灌；401 先走 bridge 再回退登录页）。
- `mcpAppProxyUrl=https://mcpappproxy.lingee.com/bizcomponents` 做 MCP 沙箱代理（iframe `/mcp-sandbox-proxy.html`，15s 超时）。
- 附件可信路径按 `getDesktopOS()` 区分 darwin/win32。

## 5. 可观测与用量

- OTLP traces/logs/metrics + `recordTtftMetric`（首 token 延迟）。
- 单 Agent `runtime-data`：dataScope/period/taskCount/artifactCount/token/credits/dailyStats（14 天）——用量看板可直接抄 schema。

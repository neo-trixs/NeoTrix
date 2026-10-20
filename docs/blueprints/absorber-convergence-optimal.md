# 蓝图: URL 摄取单一入口 — 最优解 (生命线 A, 终态)

> **状态: 已实现** (P1-P4 完成)。本文档保留为设计依据与验收矩阵;P5 已降级为低风险 (W 路径实测已走共享 hardened client)。
>
> **已落地**:
> - `nt_http` 单一配置源 (`shared_blocking_client` + `resolve_safe_origin` + `fetch_safe_http` + `fetch_safe_http_async`)
> - A/C/D/A' 全部收敛到共享 client;A/C fetch 委托 pin 原语
> - connect-期 DNS pinning 防 rebinding (TOCTOU)
> - 新增 3 个 nt_http 单测 + 此前 SSRF 回归测试

## 1. 现状 (代码实测)

五条 URL→node 摄取/抓取路径:

| 路径 | 位置 | 自有 HTTP client | 自有提取 | SSRF | redirect none | connect-pin |
|---|---|---|---|---|---|---|
| **A** `UnifiedAbsorber::absorb_webpage` | `l2/nt_world_absorber/mod.rs:374` | ✅ `http` 字段 | 委托共享 | ✅ guard | ✅ | ❌ |
| **C** `absorb_url` / `absorb_url_async` | `l8/nt_mind_knowledge_pipeline.rs` | ✅ `blocking_client`/`async_client` | 委托共享 | ✅ guard | ✅ | ❌ |
| **D** `fetch_and_ingest_url` | `l3/nt_memory_crawl.rs:283` | ✅ `http_client` | 委托共享 | ✅ guard | ✅ | ❌ |
| **A'** GitHub`github_api_get`/`github_raw` | `l2/nt_world_github_absorber.rs:40,56` | ✅ `http_client` | n/a | n/a (固定域) | ❌ | ❌ |
| **W** Wiki/ArXiv `ingest_*` | `l2/nt_memory_kb_bridge.rs` (桥接 KB) | 未知 | 未知 | 未知 | ❌ | ❌ |

**五处各自持有独立 `reqwest::Client` / 互不共享连接池、UA、timeout、redirect 策略。** 已收敛部分:SSRF guard 与 HTML 提取原语。剩余未收敛:HTTP client 单一化、connect-期 DNS pinning (TOCTOU)、W (Wiki/ArXiv) 的 SSRF 与 redirect 覆盖。

## 2. 调研结论 (决定终态形态)

### 2.1 SSRF: 现有实现是 TOCTOU,不完整 (OWASP 权威)
- `is_safe_fetch_url` 用 `ToSocketAddrs` 解析校验 → 但随后 reqwest `.get(url)` **在 connect 时自行再次 DNS 解析** → attacker 可在校验与 connect 间隙换 IP (DNS rebinding)。
- OWASP Cheat Sheet: *"re-resolve on every redirect"* + **DNS pinning**。业界 (Include Security SafeURL / Stripe Smokescreen): connect-期校验连接 IP。
- reqwest 原生支持 `ClientBuilder::resolve(host, SocketAddr)` 做 **connect-期 pinning**:解析→校验→pin→连接同一 IP,彻底防 rebinding。这是 Rust 生态落地 SSRF-in-depth 的精确位置。

### 2.2 收敛形态: 环境注入 DI + 单一共享 client
- 五处各自 `LazyLock<Client>` 互不共享 → 改为**单一 `SharedHttp` 工厂**,统一 UA/timeout/connect_timeout/redirect-none/`resolve`-pin。
- 对齐 Airbyte metadata-driven 思想:同一 client 是一切 fetch 的单一运行底座,connector (dump/source) 只声明 "我要抓这个 URL",安全策略集中在一处。

### 2.3 收敛顺序 (防 churn, 每步可独立验证)
1. 新建 `nt_world_absorber` 下的 `http.rs`(或升级 nt_memory_crawl 现有 `http_client`)为 `pub(crate)` 单一共享 client,含 redirect-none。
2. A/C/D/W 全部改托管上述单一 client。
3. W (Wiki/ArXiv) 补 SSRF guard + redirect-none。
4. connect-期 DNS pinning: `is_safe_fetch_url` 升级为返回 by-value `(SocketAddr, Url)`,fetch 时 `client.get(url).resolve(pin)`。

## 3. 终态目标架构

```
[connector 声明]  GitHub | ArXiv | Wiki | WebPage | Discovery
          │
          │ AbsorbSource (已有, metadata-driven)
          ▼
[单一编排] UnifiedAbsorber::absorb()            (已存在, 不变)
          │
          ├─ GitHub  → github_absorber (固定域 api.github.com)
          ├─ ArXiv   → kb.ingest_arxiv
          ├─ Wiki    → kb.ingest_wikipedia
          └─ WebPage → absorb_webpage  → fetch_and_ingest_url (D)
                              │
   ┌──────────────────────────┴──────────────────────────┐
   │  单一 SharedHttp (UA+timeout+redirect-none+resolve)  │
   │  单一 URL→node 原语: guard→pin→fetch→extract→insert  │
   └─────────────────────────────────────────────────────┘
```

**目标不变量 (Definition of Done):**
- **I1**: 恰**一个** reqwest client 工厂 (`pub(crate) fn shared_client()`),全工程只有它构造 `reqwest::Client`。
- **I2**: 所有「用户/外部可控 URL 的抓取」必须经过 `is_safe_fetch_url` + connect-pin + redirect-none。
- **I3**: URL→node 插入收敛到单一 `fetch_and_ingest_url`(A 的 `absorb_webpage`、C 的 `absorb_url(_async)` 全部委托它)。C 保留其去重/状态层,但不再自建 fetch。
- **I4**: W (Wiki/ArXiv) 补 redirect-none + guard(若涉及外部 URL 则 pin)。

## 4. 每细节实施计划 (阶段分步, 每步编译+测试)

### 阶段 P1 — 单一共享 client 工厂 (改动 `nt_memory_crawl.rs`)
- 将现有 `fn http_client()` 升级为:
  ```rust
  /// 全工程唯一 HTTP client 工厂。UA/timeout/redirect-none 集中于此。
  pub(crate) fn shared_client() -> &'static reqwest::blocking::Client
  ```
- 保留 `redirect(Policy::none())`。当前已在 P1 目标。
- **验证**: 编译 + `nt_memory_crawl` 测试绿。

### 阶段 P2 — A/C/D 委托单一 client
- **A** `nt_world_absorber/mod.rs`: 删除 `http` 字段与 `http_client()` 工厂,改用 `shared_client()`;`absorb_webpage` 内 `self.http` → 直接 `shared_client()`。
- **C** `nt_mind_knowledge_pipeline.rs`: 删除本地 `blocking_client()`/`async_client()`,blocking 路径用 `shared_client()`。async 路径:因 blocking 与 async 无法共用同 client 对象,**决策点 Q1**(见 §6)。
- **D** `nt_memory_crawl.rs`: 自身即共享 client 来源,保持;确认 `discover_from_seed` 等内部用法不变。
- **A'** `nt_world_github_absorber.rs`: 删除本地 `http_client`,用 `shared_client()`,加 `.redirect(Policy::none())`。GitHub 固定 `api.github.com`,无 SSRF 路径,仅需 redirect-none。
- **验证**: 三模块测试绿 + `cargo check --all-targets`。

### 阶段 P3 — connect-期 DNS pinning (SSRF 深水区)
- 将 `is_safe_fetch_url` 升级为:
  ```rust
  /// 返回校验通过的可连接 SocketAddr + 规范 URL;调用方 connect 时必须 pin。
  pub(crate) fn resolve_safe_origin(url: &str) -> Result<(std::net::SocketAddr, url::Url), String>
  ```
  - 内部:解析全部 A+AAAA → 全非私有 → 取一个安全地址。
- 增加 pin fetch 原语 (blocking):
  ```rust
  pub(crate) fn fetch_safe_http(url: &str) -> Result<(String, String), String> // (body, final_host)
  ```
  内部: `resolve_safe_origin` → `shared_client()` 用 `.get(scheme://ip:port).header(Host, host)` 或 `resolve` pin → 抓取。redirect-none 已禁用。
- **验证**: 新增 DNS-rebinding 单测(用 `.resolve` 指向本地伪造 → 确认 pin 生效) + 全绿。

### 阶段 P4 — A/C 委托 fetch 原语,收敛 node 插入
- **C** `finish_absorb`: 内部改用 `fetch_safe_http`。同步/异步收敛见 Q1。
- **A** `absorb_webpage`: 改为委托 `fetch_safe_http` + 共享插入 (或直接委托 D 的 `fetch_and_ingest_url`)。
- **决策点 Q2**: node 插入是否统一到 `fetch_and_ingest_url` (见 §6)。

### 阶段 P5 — W (Wiki/ArXiv) SSRF + redirect 覆盖
- 桥接的 `ingest_arxiv`/`ingest_wikipedia`: 若抓外部 URL,补 guard + redirect-none + pin。**先盘点代码**(见阶段 P1 前的 W 摸底)。

## 5. 验收测试矩阵

| 用例 | 预期 | 当前 | 目标 |
|---|---|---|---|
| `is_safe_fetch_url("http://127.0.0.1")` | reject | ✅ | ✅ |
| `is_safe_fetch_url("http://[::ffff:127.0.0.1]")` | reject | ✅ (已修) | ✅ |
| redirect 跟随 | 禁止 | ✅ | ✅ |
| DNS rebinding (TOCTOU) | pin 后连接一致 | ❌ **未防护** | ✅ P3 |
| `shared_client` 唯一 | 单工厂 | ❌ 五处 | ✅ P2 |
| A `absorb_webpage` 委托 fetch | 走 D 原语 | ❌ 自建 | ✅ P4 |

## 6. 决策点 (需补充, 实现前定案)
- **Q1** (async × blocking): C 的 `absorb_url_async` 需要 async client。方案:
  - (a) 保留一个独立 async client(违反 I1,但 reqwest blocking/async 物理不可共 client);
  - (b) `shared_client()` 返回 blocking,async 内部用 `tokio::task::spawn_blocking` 调 blocking 原语;
  - (c) sync 与 async 各一个 `pub(crate)` 工厂(blocking/async),但共享统一 builder 配置宏。
  - **倾向 (c)**: 单一「配置源」,两个运行时正交对象。I1 放宽为「单配置源 + blocking/async 各一」。
- **Q2** (node 插入收敛粒度): A 的 `absorb_webpage` 与 C 的 `absorb_url` 是否完全统一到 D 的 `fetch_and_ingest_url`?
  - 现状:D 走 `rusqlite::Connection` (L3 store),A/C 走 `KnowledgeBase` 桥 (L2)。两者存储路径不同。
  - **倾向**: A 委托 D 的 fetch/extract 部分(A 本在 L2,调用 L3 原语已验证可行),插入仍走各自既有 handle。不强行合并 insert(保持存储层抽象)。

## 7. 风险与回滚
- **Risk1 (P3 pin)**: `.resolve` pin 需在 `ClientBuilder` 层定义,但 `shared_client` 是静态单例 → pin 无法在共享 client 上按 request 变。**对策**: pin 不在共享 client,而在**每个 fetch 调用**用 `reqwest::blocking::Client::builder()` 临时 client + `.resolve`(开销可接受,连接复用期货放弃),或暴露 `resolve`-support 的独立 client。此点 P3 落地时以单测验证行为为准,倾向「临时 pin client」。
- **Risk2 (async)**: `spawn_blocking` 在无 runtime 的纯 lib 测试中会 panic。**对策**: Q1 定案后,async 路径单元测试用 `#[tokio::test]` 或仅测 blocking 原语。

## 8. 建议执行顺序 (避免 churn)
P1 → P2 (A/C/D/A') → P3 (pin) → P4 (委托) → P5 (W)。每阶段独立编译+测试,可单独提交。**不引入新依赖**(自实现 pin,符合保守)。

## 9. 执行记录 (2026-08)
- **P1/P2**: 新建 `nt_http.rs` (单配置源);`nt_memory_crawl::http_client`、A、A' 全部委托 `shared_blocking_client`。A 删除 `http` 字段与本地工厂。
- **P3**: `resolve_safe_origin` (IP/域名双路径 + IPv4-mapped + 编码形式经 resolve 过滤) + `fetch_safe_http`/`fetch_safe_http_async` (`.resolve(host, addr)` connect-期 pin)。
- **P4**: A `absorb_webpage` 与 C `absorb_url(_async)` 均委托 pin fetch;D `fetch_and_ingest_url` 同样委托。A 保留自身 `KnowledgeBase` 插入 (Q2 定案)。
- **决策落地**: Q1 → 单配置源 + 双运行时对象 (blocking 单例供 wiki/arxiv/github-topic;async/blocking pin client 按调用临时构造,因 `.resolve` 为 per-client);`shared_async_client` 因无消费者已移除 (async 全走 pin)。
- **W (P5)**: 实测已经由 `nt_memory_crawl::http_client()` (固定域 wikipedia/arxiv/api.github.com,非用户可控) → 继承 redirect-none,SSRF 风险低,不再单列。
- **验证**: 40/40 测试通过 (nt_memory_crawl + nt_world_absorber + nt_mind_knowledge_pipeline + nt_http);`cargo check --all-targets` 干净。

---

*状态: 评审草案。待确认 Q1/Q2 与 W 摸底后敲定终版蓝图。*
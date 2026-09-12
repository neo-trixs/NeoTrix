# NT-WORLD vs NT-SHIELD 爬虫/Stealth 边界

## 架构定位

```
L2 感知层 (nt_world)          L3 具身层 (nt_shield)
┌─────────────────────┐      ┌──────────────────────────┐
│  UnifiedCrawler     │      │  StealthHttpClient       │
│  FetcherPool        │─────▶│  ProxyPool               │
│  SessionPool (指纹) │      │  TorClient               │
│  ContentClassifier  │      │  IpPrivacyManager        │
│  KnowledgeMapper    │      │  RotationCoordinator     │
└─────────────────────┘      └──────────────────────────┘
      数据获取 + 解析               反检测 + 代理 + 指纹
```

边界切面：**HTTP 请求层** — nt_world 构造请求意图，nt_shield 执行带伪装的请求发送。

---

## NT-WORLD (L2 感知层) — 数据获取与解析

### 职责

| 模块 | 功能 | 关键类型 |
|------|------|----------|
| `crawl::unified` | 统一爬虫入口，编排发现→抓取→分类→映射闭环 | `UnifiedCrawler`, `CrawlerSummary` |
| `crawl::fetcher` | HTTP/Tor/Browser 多协议抓取池 | `FetcherPool`, `FetchResult`, `FetcherProtocol` |
| `crawl::frontier` | 双队列 URL frontier，去重 + 优先级调度 | `DualQueueFrontier`, `UrlEntry` |
| `crawl::classifier` | 内容分类（正文/广告/导航/代码等） | `ContentClassifier`, `ClassifiedContent` |
| `crawl::mapper` | 抓取结果→KB 映射 | `KnowledgeMapper`, `MappedKnowledge` |
| `crawl::spider` | 站点深度爬取 | `Spider`, `SpiderConfig` |
| `crawl::discover` | 链接发现与过滤 | `DiscoveryExtractor` |
| `crawl::stealth` | **轻量级本地指纹轮转**（SessionPool + Fingerprint） | `SessionPool`, `Fingerprint` |
| `crawl::camofox` | CamoFox 反检测浏览器集成 | `CamoFoxSession` |
| `crawl::agentic_browse` | 智能浏览代理 | `AgenticBrowser` |
| `data_source` | 12 个外部数据源采集器（EDGAR/GDELT/USGS 等） | 各采集器独立 |

### NT-WORLD 不处理

- 代理池管理与健康检查
- Tor 网络连接与链路管理
- IP 隐私/伪装/地理定位
- 出站规则引擎（防火墙/ACL）
- 系统级指纹伪造（TLS/JA3/Canvas）
- 反检测浏览器的底层驱动

### 本地指纹（crawl::stealth）

nt_world 内置的 `SessionPool` 是**轻量级会话指纹轮转**，仅用于：
- 生成随机 User-Agent / Platform / Language / Viewport 组合
- 为 FetcherPool 提供请求级指纹一致性
- Chrome 启动参数伪装（`--disable-blink-features=AutomationControlled`）

这是**数据获取侧的最小伪装**，不涉及网络层反检测。

---

## NT-SHIELD (L3 具身层) — 反检测与网络安全

### 职责

| 模块 | 功能 | 关键类型 |
|------|------|----------|
| `stealth_net::http_client` | 增强型 HTTP 客户端：SOCKS5 + 客户端池 + 动态指纹轮转 | `StealthHttpClient`, `ProxyConfig` |
| `stealth_net::proxy_pool` | 代理池管理：采集/验证/健康检查/淘汰 | `ProxyPool`, `ProxyNode`, `ProxyHealth` |
| `stealth_net::proxy_chain` | 动态代理链：多跳路由 + 协议混合 | `DynamicProxyChain`, `ProxyNode` |
| `stealth_net::tor_client` | Tor 客户端：SOCKS5 连接 + 控制端口 | `TorClient`, `TorConfig` |
| `stealth_net::tor_crawler` | Tor 暗网爬取支持 | `TorCrawler` |
| `stealth_net::system_fingerprint` | 系统级指纹伪造：TLS/JA3/Canvas/WebGL | `SystemFingerprint`, `SystemFingerprintGenerator` |
| `stealth_net::ip_privacy` | IP 隐私管理：伪造 IP/地理定位/子网 | `IpPrivacyManager`, `FakeIpConfig` |
| `stealth_net::ip_rotator` | OS 级 IP 轮转 | `OsIpRotator` |
| `stealth_net::ip_geo` | IP 地理定位 | `IpGeoLocator`, `GeoResult` |
| `stealth_net::rotation_coordinator` | 统一时钟驱动的指纹/代理轮转 | `RotationCoordinator`, `RotationDomain` |
| `stealth_net::bandit` | 多臂赌博机选择最优指纹+代理组合 | `FingerprintBandit`, `ComboArm` |
| `stealth_net::self_iterating` | 自迭代反检测学习 | `SelfIteratingStealth`, `StealthLearning` |
| `stealth_net::firewall` | 出站防火墙规则 | `FirewallManager`, `FirewallRule` |
| `stealth_net::rules` | 出站规则引擎（中国 IP bypass 等） | `RuleEngine`, `OutboundRule` |
| `stealth_net::blocklist` | Tracker/广告域名拦截 | `is_tracker_blocked` |
| `stealth_net::network_monitor` | 网络健康监控 | `NetworkMonitor` |
| `stealth_net::network_diagnostics` | 网络诊断与自动修复 | `diagnose_all`, `RemediationEngine` |
| `stealth_net::lan_router` | 局域网路由管理 | `LanRouter` |
| `stealth_net::local_proxy` | 本地代理 + Tor 管理 | `LocalProxy`, `TorManager` |
| `stealth_net::proxy_control` | 代理守护进程控制 | `ProxyControl`, `DaemonMode` |
| `stealth_net::proxy_heartbeat` | 代理心跳检测 | `ProxyHeartbeatEngine` |
| `evasion` | 高级逃逸：云逃逸/全断开/钩爪链 | `CloudEvade`, `FullBreak`, `GrappleHookChain` |
| `defense` | 统一防御层：护栏遍历/推理保护/拒绝篡改/反蒸馏 | `UnifiedDefenseLayer`, `AntiDistillation` |
| `guard` | 输入/输出守卫：门卫/哨兵/提示守护 | `InputGatekeeper`, `OutputSentinel` |

### NT-SHIELD 不处理

- URL 发现与 frontier 调度
- HTML 解析与内容提取
- 内容分类与知识映射
- KB 写入与检索
- 爬取策略（深度/广度/自适应）

---

## 边界规则

### 规则 1：爬虫通过接口获取代理，不感知代理实现

```rust
// nt_world 侧调用
let proxy = stealthHttpClient.proxy_config();
let result = fetcher.fetch_with_proxy(url, &proxy);

// nt_shield 侧实现
impl StealthHttpClient {
    pub fn proxy_config(&self) -> ProxyConfig { ... }
    pub async fn send(&self, req: Request) -> Result<Response> { ... }
}
```

nt_world 的 `FetcherPool` 只知道 `ProxyConfig`（代理地址 + 协议），不管理代理池生命周期。

### 规则 2：Stealth 不感知数据内容

```rust
// nt_shield 只看到字节流
impl StealthHttpClient {
    pub async fn send(&self, req: Request) -> Result<Response> {
        // 不解析 HTML/JSON/CSV
        // 不分类内容
        // 不写入 KB
        // 只返回原始 Response (status + headers + body bytes)
    }
}
```

nt_shield 的 `StealthHttpClient` 对请求体/响应体**零语义理解**，只负责：
- 附加正确的指纹头
- 通过代理链路由
- 处理 TLS 指纹
- 执行出站规则

### 规则 3：边界在 HTTP 请求层

```
┌──────────────────────────────────────────────────────┐
│                    NT-WORLD                           │
│                                                      │
│  UnifiedCrawler → FetcherPool → [构造 Request]        │
│                                      │               │
│  ┌───────────────────────────────────┘               │
│  │  SessionPool (本地指纹)                            │
│  │  仅提供 UA/Platform/Language 一致性               │
│  └───────────────────────────────────┐               │
│                                      │               │
├─────────────── HTTP Request ─────────┤               │
│                                      ▼               │
│  ┌───────────────────────────────────────────────┐   │
│  │              NT-SHIELD                        │   │
│  │                                               │   │
│  │  StealthHttpClient                            │   │
│  │    ├─ SystemFingerprint (TLS/JA3/Canvas)      │   │
│  │    ├─ ProxyChain (多跳路由)                    │   │
│  │    ├─ IpPrivacy (IP 伪装)                     │   │
│  │    ├─ RuleEngine (出站规则)                    │   │
│  │    ├─ FirewallManager (防火墙)                │   │
│  │    └─ Bandit (最优组合选择)                    │   │
│  │                                               │   │
│  │  → 发送请求 → 返回 Response                    │   │
│  └───────────────────────────────────────────────┘   │
│                                                      │
├─────────────── HTTP Response ────────────────────────┤
│                                                      │
│  FetcherPool ← [接收 Response]                       │
│  ContentClassifier → KnowledgeMapper → KB            │
│                                                      │
│                    NT-WORLD                           │
└──────────────────────────────────────────────────────┘
```

### 规则 4：指纹分层

| 层级 | 位置 | 内容 | 生命周期 |
|------|------|------|----------|
| L1 请求指纹 | nt_world::stealth | UA/Platform/Language/Viewport | 每次请求轮转 |
| L2 系统指纹 | nt_shield::system_fingerprint | TLS/JA3/Canvas/WebGL/WebRTC | RotationCoordinator 驱动 |
| L3 网络指纹 | nt_shield::ip_privacy | IP/地理定位/DNS | 代理切换时更新 |

### 规则 5：Tor 双通道

| 通道 | 入口 | 用途 |
|------|------|------|
| nt_world::crawl::fetcher | `FetcherProtocol::Tor` | 通过 Tor 网络爬取暗网/匿名站点 |
| nt_shield::tor_client | `TorClient::connect` | 底层 Tor SOCKS5 连接管理 |

nt_world 选择 "走 Tor"，nt_shield 管理 "Tor 怎么走"。

### 规则 6：CamoFox 桥接

`nt_world::crawl::camofox` 是 nt_world 侧的 CamoFox 浏览器会话管理，但底层反检测能力由 nt_shield 的 `evasion` 模块提供。桥接点：
- nt_world 调用 `CamoFoxSession::browse(url)`
- CamoFox 内部使用 nt_shield 的指纹/代理能力

---

## 依赖方向

```
nt_world ──依赖──▶ nt_shield (通过 trait 接口)
nt_shield ──不依赖──▶ nt_world
```

nt_shield 是**纯基础设施层**，对上层语义无感知。nt_world 通过 trait 抽象调用 nt_shield 能力，可替换实现。

---

## 文件索引

| 路径 | 说明 |
|------|------|
| `l2_perception/nt_world/crawl/mod.rs` | 爬虫模块入口，6 目录架构 |
| `l2_perception/nt_world/crawl/stealth.rs` | 轻量级本地指纹轮转 (SessionPool) |
| `l2_perception/nt_world/crawl/fetcher.rs` | 多协议抓取池 (FetcherPool) |
| `l2_perception/nt_world/crawl/unified.rs` | 统一爬虫入口 (UnifiedCrawler) |
| `l2_perception/nt_world/crawl/camofox.rs` | CamoFox 反检测浏览器集成 |
| `l3_embodiment/nt_shield/mod.rs` | 安全模块入口 |
| `l3_embodiment/nt_shield/nt_shield_stealth_net/mod.rs` | Stealth-Net 子系统入口 (35+ 模块) |
| `l3_embodiment/nt_shield/nt_shield_stealth_net/http_client/mod.rs` | 增强型 HTTP 客户端 (StealthHttpClient) |
| `l3_embodiment/nt_shield/nt_shield_stealth_net/proxy_pool.rs` | 代理池管理 |
| `l3_embodiment/nt_shield/nt_shield_stealth_net/system_fingerprint.rs` | 系统级指纹伪造 |
| `l3_embodiment/nt_shield/nt_shield_stealth_net/tor_client.rs` | Tor 客户端 |
| `l3_embodiment/nt_shield/nt_shield_stealth_net/ip_privacy.rs` | IP 隐私管理 |
| `l3_embodiment/nt_shield/nt_shield_stealth_net/rotation_coordinator.rs` | 轮转协调器 |
| `l3_embodiment/nt_shield/evasion/` | 高级逃逸模块 |

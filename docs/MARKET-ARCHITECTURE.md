# NeoTrix Market Architecture — DSH 市场 + 插件市场融合设计

## 现状分析

```
当前状态 (碎片化)
├── 前端 UI: PluginMarketplace.tsx (完整 UI，但后端是 stub)
├── 后端: 3 个冲突的插件注册表
│   ├── src-tauri/plugins/manager.rs (native dylib, 464行)
│   ├── neotrix-core/nt_io_plugin/registry.rs (trait-based, 722行)
│   └── neotrix-core/io/nt_io/plugin_registry.rs (lifecycle, 1074行)
├── DSH-IM: 外部仓库，未对接
└── 市场: 无远程发现，仅本地文件安装
```

## 架构设计: 三层市场体系

```
┌─────────────────────────────────────────────────────────────┐
│                    L3 市场融合层 (Market Fusion)              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ DSH 市场  │  │ GitHub   │  │ npm/PyPI │  │ 本地仓库  │   │
│  │ dshfind  │  │ 搜索安装  │  │ 包管理   │  │ file://  │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       └──────────────┼──────────────┼──────────────┘        │
│                      ▼                                      │
│              ┌───────────────┐                              │
│              │  统一发现引擎  │  ← 探索/抓取/索引              │
│              │  Discovery    │                              │
│              └───────┬───────┘                              │
│                      ▼                                      │
│              ┌───────────────┐                              │
│              │  能力熔炼器    │  ← 分析/抽象/提炼              │
│              │  Refinery     │                              │
│              └───────┬───────┘                              │
│                      ▼                                      │
│              ┌───────────────┐                              │
│              │  插件注册表    │  ← 统一 schema/版本/签名       │
│              │  Registry     │                              │
│              └───────┬───────┘                              │
│                      │                                      │
├──────────────────────┼──────────────────────────────────────┤
│                      ▼                                      │
│                    L2 插件运行时                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │ Native   │  │ WASM     │  │ Domain   │                  │
│  │ dylib    │  │ sandbox  │  │ Plugin   │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                    L1 能力网 (Capability Network)            │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐            │
│  │NT-ACT│ │NT-IO │ │NT-WRL│ │NT-SHD│ │NT-MEM│ ...         │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘            │
└─────────────────────────────────────────────────────────────┘
```

## 核心模块设计

### 1. 统一插件 Schema (plugin.toml)

```toml
[plugin]
id = "dsh-im-wechat"
name = "WeChat IM Channel"
version = "1.0.0"
description = "微信 IM 渠道适配器"
author = "xmanrui"
license = "MIT"
category = "im-channel"           # im-channel | tool | memory | perception | action
tags = ["im", "wechat", "messaging"]
min_runtime = "0.21.0"

[plugin.source]
type = "dsh-market"               # dsh-market | github | npm | local
repo = "xmanrui/dsh-im"
asset = "dsh-im-wechat.wasm"      # 或 .dylib / .so

[plugin.permissions]
network = ["api.weixin.qq.com"]   # 网络访问白名单
filesystem = ["~/.neotrix/im/"]   # 文件系统白名单
capabilities = ["im-channel"]     # 能力声明

[plugin.runtime]
type = "wasm"                     # wasm | native | domain
max_memory_mb = 128
max_cpu_ms = 1000
timeout_secs = 30

[plugin.dependencies]
requires = ["dsh-im-core >= 1.0.0"]
conflicts = []
```

### 2. 统一发现引擎 (Discovery Engine)

```rust
/// 外部源发现器
pub trait ExternalDiscoverer: Send + Sync {
    /// 源名称
    fn source_name(&self) -> &str;
    
    /// 搜索插件
    async fn search(&self, query: &str, category: Option<&str>) -> Result<Vec<DiscoveredPlugin>>;
    
    /// 获取插件详情
    async fn get_detail(&self, plugin_id: &str) -> Result<PluginDetail>;
    
    /// 下载插件
    async fn download(&self, plugin_id: &str, version: &str) -> Result<PathBuf>;
    
    /// 检查更新
    async fn check_updates(&self, installed: &[InstalledPlugin]) -> Result<Vec<PluginUpdate>>;
}

/// DSH 市场发现器
pub struct DshMarketDiscoverer {
    api_endpoint: String,
    client: reqwest::Client,
}

/// GitHub 发现器
pub struct GitHubDiscoverer {
    token: Option<String>,
    client: reqwest::Client,
}

/// 发现引擎
pub struct DiscoveryEngine {
    discoverers: Vec<Box<dyn ExternalDiscoverer>>,
    cache: DiscoveryCache,
}
```

### 3. 能力熔炼器 (Refinery)

```rust
/// 熔炼器 — 将外部插件提炼为 NeoTrix 原生能力
pub struct Refinery {
    /// 分析器: 分析插件代码/文档，提取能力签名
    analyzer: CapabilityAnalyzer,
    
    /// 适配器生成器: 生成 NeoTrix 域插件适配代码
    adapter_gen: AdapterGenerator,
    
    /// 测试器: 验证熔炼后的插件是否可用
    tester: PluginTester,
}

impl Refinery {
    /// 熔炼流程: 探索 → 分析 → 适配 → 测试 → 注册
    pub async fn refine(&self, source: &DiscoveredPlugin) -> Result<RefinedPlugin> {
        // 1. 下载并解压
        let artifact = self.download_and_extract(source).await?;
        
        // 2. 分析能力签名
        let capability = self.analyzer.analyze(&artifact).await?;
        
        // 3. 生成适配器
        let adapter = self.adapter_gen.generate(&capability).await?;
        
        // 4. 测试验证
        self.tester.test(&adapter).await?;
        
        // 5. 返回熔炼结果
        Ok(RefinedPlugin {
            original: source.clone(),
            capability,
            adapter,
            refined_at: Utc::now(),
        })
    }
}
```

### 4. 插件注册表 (Unified Registry)

```rust
/// 统一插件注册表
pub struct UnifiedRegistry {
    /// 已安装插件
    installed: RwLock<HashMap<PluginId, InstalledPlugin>>,
    
    /// 活跃插件 (已加载运行)
    active: RwLock<HashMap<PluginId, ActivePlugin>>,
    
    /// 市场索引
    market_index: RwLock<MarketIndex>,
    
    /// 版本管理
    version_manager: VersionManager,
    
    /// 签名验证
    signature_verifier: SignatureVerifier,
}

/// 市场索引
pub struct MarketIndex {
    /// 本地缓存的市场数据
    plugins: Vec<MarketEntry>,
    
    /// 分类索引
    categories: HashMap<String, Vec<PluginId>>,
    
    /// 标签索引
    tags: HashMap<String, Vec<PluginId>>,
    
    /// 最后同步时间
    last_sync: Option<DateTime<Utc>>,
}

/// 市场条目
pub struct MarketEntry {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub rating: f32,
    pub source: PluginSource,
    pub assets: Vec<PluginAsset>,
}
```

## 数据流

```
用户点击"安装" 
  → DiscoveryEngine.search("wechat im")
  → 返回 DSH-IM-WeChat 1.0.0
  → 用户确认安装
  → Refinery.refine(plugin)
    → 下载 .wasm/.dylib
    → 分析能力: "im-channel, wechat, messaging"
    → 生成适配器: domain/plugins/im_wechat.rs
    → 测试: 连通性检查
  → UnifiedRegistry.install(refined)
    → 签名验证
    → 版本检查
    → 复制到 ~/.neotrix/plugins/
    → 更新 manifest.json
    → 注册到 DomainRegistry
  → 前端刷新插件列表
```

## 目录结构

```
~/.neotrix/
├── plugins/                    # 已安装插件
│   ├── dsh-im-wechat/
│   │   ├── manifest.json       # 插件清单
│   │   ├── plugin.wasm         # 插件二进制
│   │   ├── data.sqlite         # 沙箱数据
│   │   └── adapter.rs          # 生成的适配器
│   └── dsh-im-feishu/
│       └── ...
├── market/                     # 市场缓存
│   ├── index.json              # 市场索引
│   ├── dsh-market.json         # DSH 市场配置
│   └── github-cache/           # GitHub 搜索缓存
└── config.toml                 # 全局配置
```

## 实现优先级

### Phase 1: 基础市场 (1-2 天)
- [ ] 统一插件 schema (plugin.toml)
- [ ] DSH 市场发现器 (dshfind.com API)
- [ ] 本地安装/卸载/启用/禁用
- [ ] 前端 UI 完善 (搜索/分类/详情)

### Phase 2: 熔炼器 (2-3 天)
- [ ] 能力分析器 (解析 manifest + 文档)
- [ ] 适配器生成器 (生成 domain plugin 代码)
- [ ] 插件测试器 (连通性/功能验证)
- [ ] 版本管理 (semver 比较/更新检查)

### Phase 3: 高级特性 (3-5 天)
- [ ] GitHub 发现器 (搜索/下载/release)
- [ ] 签名验证 (Ed25519)
- [ ] 沙箱隔离 (WASM + 资源限制)
- [ ] 插件评分/评论系统

## 对接 DSH-IM 的具体方案

```
DSH-IM 渠道插件
├── dsh-im-wechat   → NeoTrix: nt_act/im/wechat.rs
├── dsh-im-feishu   → NeoTrix: nt_act/im/feishu.rs
├── dsh-im-dingtalk → NeoTrix: nt_act/im/dingtalk.rs
├── dsh-im-wecom    → NeoTrix: nt_act/im/wecom.rs
├── dsh-im-qq       → NeoTrix: nt_act/im/qq.rs
├── dsh-im-slack    → NeoTrix: nt_act/im/slack.rs
├── dsh-im-telegram → NeoTrix: nt_act/im/telegram.rs
├── dsh-im-discord  → NeoTrix: nt_act/im/discord.rs
└── dsh-im-whatsapp → NeoTrix: nt_act/im/whatsapp.rs

每个渠道插件
├── manifest.json (能力声明)
├── adapter.rs (NeoTrix 适配层)
├── channel.rs (渠道核心逻辑)
└── data.sqlite (运行时数据)
```

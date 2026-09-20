# NeoTrix 数据流节点 Map
#
# 详细描述每个数据流节点的输入/输出/依赖
# 版本: v1.0.0
# 更新: 2026-09-20

## 1. 数据源节点分类

### 1.1 MediaSource 节点 (29 个)

| 节点 ID | 名称 | 域 | 输入 | 输出 | 依赖 |
|---------|------|-----|------|------|------|
| netease | 网易云音乐 | Media | query, page | MediaSearchResult | reqwest |
| kuwo | 酷我音乐 | Media | query, page | MediaSearchResult | reqwest |
| kugou | 酷狗音乐 | Media | query, page | MediaSearchResult | reqwest |
| migu | 咪咕音乐 | Media | query, page | MediaSearchResult | reqwest |
| soundcloud | SoundCloud | Media | query, page | MediaSearchResult | reqwest |
| spotify | Spotify | Media | query, page | MediaSearchResult | reqwest |
| piped | Piped | Media | query, page | MediaSearchResult | reqwest |
| jiosaavn | JioSaavn | Media | query, page | MediaSearchResult | reqwest |
| deezer | Deezer | Media | query, page | MediaSearchResult | reqwest |
| bandcamp | Bandcamp | Media | query, page | MediaSearchResult | reqwest |
| qqmusic | QQ 音乐 | Media | query, page | MediaSearchResult | reqwest |
| youtube | YouTube | Media | query, page | MediaSearchResult | reqwest |
| bilibili | Bilibili | Media | query, page | MediaSearchResult | reqwest |
| vimeo | Vimeo | Media | query, page | MediaSearchResult | reqwest |
| arxiv | arXiv | Academic | query, page | MediaSearchResult | reqwest |
| semantic_scholar | Semantic Scholar | Academic | query, page | MediaSearchResult | reqwest |
| openlibrary | Open Library | Academic | query, page | MediaSearchResult | reqwest |
| annas_archive | Anna's Archive | Academic | query, page | MediaSearchResult | reqwest |
| lrclib | LRCLIB | Media | query, page | MediaSearchResult | reqwest |
| multi | Multi Lyric | Media | query, page | MediaSearchResult | reqwest |
| genius | Genius | Media | query, page | MediaSearchResult | reqwest |
| tiktok | TikTok | Tech | query, page | MediaSearchResult | reqwest |
| instagram | Instagram | Tech | query, page | MediaSearchResult | reqwest |
| twitter | Twitter | Tech | query, page | MediaSearchResult | reqwest |
| ytdlp | yt-dlp | Media | query, page | MediaSearchResult | reqwest |

### 1.2 IntelSource 节点 (3+ 个)

| 节点 ID | 名称 | 域 | 输入 | 输出 | 依赖 |
|---------|------|-----|------|------|------|
| gdelt | GDELT DOC 2.0 | Intel | query | IntelResult | reqwest |
| edgar | SEC EDGAR | Intel | query | IntelResult | reqwest |
| usgs | USGS Earthquakes | Intel | query | IntelResult | reqwest |

### 1.3 OsintSource 节点 (20+ 个)

| 节点 ID | 名称 | 域 | 输入 | 输出 | 依赖 |
|---------|------|-----|------|------|------|
| dns | DNS Investigation | Osint | target | OsintResult | reqwest |
| shodan | Shodan | Osint | target | OsintResult | reqwest |
| censys | Censys | Osint | target | OsintResult | reqwest |
| zoomeye | ZoomEye | Osint | target | OsintResult | reqwest |
| fofa | FOFA | Osint | target | OsintResult | reqwest |
| securitytrails | SecurityTrails | Osint | target | OsintResult | reqwest |
| vuln | Vulnerability Scan | Osint | target | OsintResult | reqwest |
| network | Network Analysis | Osint | target | OsintResult | reqwest |
| dark | Dark Web Monitor | Osint | target | OsintResult | reqwest |
| person | Person Investigation | Osint | target | OsintResult | reqwest |
| social | Social Media OSINT | Osint | target | OsintResult | reqwest |
| social_search | Social Search | Osint | target | OsintResult | reqwest |
| username_checker | Username Checker | Osint | target | OsintResult | reqwest |
| credential | Credential Check | Osint | target | OsintResult | reqwest |
| url | URL Analysis | Osint | target | OsintResult | reqwest |
| metadata | Metadata Extraction | Osint | target | OsintResult | reqwest |
| sweep | Asset Sweep | Osint | target | OsintResult | reqwest |
| ad_graph | Adversary Graph | Osint | target | OsintResult | reqwest |
| harvest | Data Harvest | Osint | target | OsintResult | reqwest |
| cryptopub | Crypto Publication | Osint | target | OsintResult | reqwest |
| auto_patrol | Auto Patrol | Osint | target | OsintResult | reqwest |
| automation | OSINT Automation | Osint | target | OsintResult | reqwest |
| search_engine | Search Engine OSINT | Osint | target | OsintResult | reqwest |
| report | OSINT Report | Osint | target | OsintResult | reqwest |

### 1.4 LlmProvider 节点 (30+ 个)

| 节点 ID | 名称 | 免费 | 需要 Key | 域 | 输入 | 输出 | 依赖 |
|---------|------|------|----------|-----|------|------|------|
| openai | OpenAI | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| anthropic | Anthropic | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| gemini | Google Gemini | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| groq | Groq | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| openrouter | OpenRouter | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| cerebras | Cerebras | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| sambanova | SambaNova | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| pollinations | Pollinations | ✅ | ❌ | Llm | LlmRequest | LlmResponse | reqwest |
| cloudflare | Cloudflare Workers AI | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| nvidia | NVIDIA NIM | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| github_models | GitHub Models | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| huggingface | HuggingFace Inference | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| together_free | Together AI Free | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| llm7 | LLM7 | ✅ | ❌ | Llm | LlmRequest | LlmResponse | reqwest |
| kilo | Kilo AI | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| siliconflow | SiliconFlow | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| zai | Z.AI | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| opencode_zen | OpenCode Zen | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| ovh | OVH AI | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| deepseek_free | DeepSeek Free | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| modelscope | ModelScope | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| api_airforce | API Airforce | ✅ | ❌ | Llm | LlmRequest | LlmResponse | reqwest |
| empero | Free Empero | ✅ | ❌ | Llm | LlmRequest | LlmResponse | reqwest |
| ollama | Ollama (Local) | ✅ | ❌ | Llm | LlmRequest | LlmResponse | reqwest |
| xai | xAI (Grok) | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| moonshot | Moonshot (Kimi) | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| qwen | Qwen (DashScope) | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| doubao | Doubao (Ark) | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| minimax | MiniMax | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| perplexity | Perplexity | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| cohere | Cohere | ❌ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |
| aihub | AI Hub | ✅ | ✅ | Llm | LlmRequest | LlmResponse | reqwest |

## 2. 数据流节点连接图

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         DataSourceRegistry (路由层)                              │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                        MediaSource Cluster                             │   │
│  │                                                                         │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │   │
│  │  │ netease │ │  kuwo   │ │  kugou  │ │  migu   │ │spotify  │         │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘         │   │
│  │       │           │           │           │           │               │   │
│  │       └───────────┴───────────┴───────────┴───────────┘               │   │
│  │                           │                                           │   │
│  │                           ▼                                           │   │
│  │                   MediaSearchResult                                   │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                        IntelSource Cluster                              │   │
│  │                                                                         │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐                                  │   │
│  │  │  gdelt  │ │  edgar  │ │  usgs   │                                  │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘                                  │   │
│  │       │           │           │                                         │   │
│  │       └───────────┴───────────┘                                         │   │
│  │                   │                                                     │   │
│  │                   ▼                                                     │   │
│  │                   IntelResult                                           │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                        OsintSource Cluster                              │   │
│  │                                                                         │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │   │
│  │  │   dns   │ │ shodan  │ │ censys  │ │zoomeye  │ │  fofa   │         │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘         │   │
│  │       │           │           │           │           │               │   │
│  │       └───────────┴───────────┴───────────┴───────────┘               │   │
│  │                           │                                           │   │
│  │                           ▼                                           │   │
│  │                   OsintResult                                          │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                        LlmProvider Cluster                              │   │
│  │                                                                         │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │   │
│  │  │ openai  │ │anthropic│ │ gemini  │ │  groq   │ │ ollama  │         │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘         │   │
│  │       │           │           │           │           │               │   │
│  │       └───────────┴───────────┴───────────┴───────────┘               │   │
│  │                           │                                           │   │
│  │                           ▼                                           │   │
│  │                   LlmResponse                                          │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 3. 数据流状态机

### 3.1 数据源状态
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         DataSource State Machine                                 │
│                                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐                 │
│  │  Idle    │───►│ Loading  │───►│ Success  │───►│  Idle    │                 │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘                 │
│       │               │               │               │                       │
│       │               ▼               ▼               │                       │
│       │           ┌──────────┐   ┌──────────┐        │                       │
│       │           │  Error   │   │ Timeout  │        │                       │
│       │           └──────────┘   └──────────┘        │                       │
│       │               │               │               │                       │
│       │               └───────────────┴───────────────┘                       │
│       │                           │                                           │
│       └───────────────────────────┘                                           │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 查询状态
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         Query State Machine                                     │
│                                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐                 │
│  │ Receive  │───►│ Route    │───►│ Execute  │───►│ Merge    │                 │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘                 │
│       │               │               │               │                       │
│       │               ▼               ▼               ▼                       │
│       │           ┌──────────┐   ┌──────────┐   ┌──────────┐                 │
│       │           │ Validate │   │ Retry    │   │ Cache    │                 │
│       │           └──────────┘   └──────────┘   └──────────┘                 │
│       │               │               │               │                       │
│       │               └───────────────┴───────────────┘                       │
│       │                           │                                           │
│       └───────────────────────────┘                                           │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 4. 数据流错误处理

### 4.1 错误类型
```rust
pub enum DataSourceError {
    Network(String),      // 网络错误
    Parse(String),        // 解析错误
    RateLimit(String),    // 限流
    Auth(String),         // 认证错误
    NotFound(String),     // 未找到
    Timeout(String),      // 超时
    Unknown(String),      // 未知错误
}
```

### 4.2 错误处理流程
```
Error Occurred ──► Classify Error ──► Log Error ──► Retry Logic ──► Fallback
                    │                   │               │               │
                    ▼                   ▼               ▼               ▼
                    ┌───────────────────────────────────────────────────┐
                    │  Network: Retry with backoff                     │
                    │  Parse: Log and skip                             │
                    │  RateLimit: Wait and retry                       │
                    │  Auth: Mark unhealthy                            │
                    │  NotFound: Skip                                  │
                    │  Timeout: Retry with shorter timeout             │
                    │  Unknown: Log and skip                           │
                    └───────────────────────────────────────────────────┘
```

## 5. 数据流性能指标

### 5.1 延迟指标
- **P50**: 50% 请求延迟
- **P95**: 95% 请求延迟
- **P99**: 99% 请求延迟
- **Timeout**: 超时阈值 (默认 30s)

### 5.2 吞吐量指标
- **QPS**: 每秒查询数
- **TPS**: 每秒事务数
- **Concurrency**: 并发请求数

### 5.3 可靠性指标
- **Success Rate**: 成功率
- **Error Rate**: 错误率
- **Availability**: 可用性

---

**注意**: 此数据流节点 Map 是活文档，随项目迭代持续更新。每次添加新数据源后，必须同步更新此文档。

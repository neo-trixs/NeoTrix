# NeoTrix 媒体源进化清单 v6.0 — 加密能力 + 生态接线

> **版本**: v6.0 | **日期**: 2026-09-09
> **前置**: Phase 1-36 完成 (82 文件, 29 音频源, 编译通过)
> **原则**: 最小实现，禁止过度工程化。每个 Phase ≤200 行新代码。

---

## 一、当前状态

| 能力域 | 文件数 | 状态 |
|--------|--------|------|
| 核心引擎 (core/) | 8 | ✅ 编译通过 |
| 音频源 (audio/) | 12 | ✅ 29 源适配器 |
| 视频源 (video/) | 4 | ✅ YouTube/Bilibili/Vimeo |
| 图片源 (image/) | 5 | ✅ Pixabay/Pexels/Unsplash/Wikimedia |
| 文本源 (text/) | 11 | ✅ RSS/社交/歌词/文档/书籍 |
| 基础设施 (infra/) | 20 | ⚠️ 大部分为 mod.rs 骨架 |
| 进化 (evolution/) | 5 | ✅ SkillGloW + 自动发现 + 自适应质量 |
| **总计** | **82** | ✅ 编译通过 |

---

## 二、Phase 48: 加密能力吸收 (Crypto Capability Absorption)

### 目标
吸收 Kugou eapi + Netease weapi/eapi 加密，实现真正的无代理直接 API 访问。

### 48.1 Kugou Eapi 加密
**文件**: `audio/kugou.rs` (修改, +60 行)

```rust
// eapi 加密: text -> bin to hex
// 1. 构造 eapi params JSON: {"appid":1005,"platid":4,"encode_album_audio_id":"...","token":""}
// 2. MD5(params) -> secret
// 3. AES-128-ECB(params, key) -> encrypted
// 4. MD5(encrypted) -> verify
// 5. 构造 URL: https://trackercdn.kugou.com/i/v2/?cmd=25&hash={}&pid=1&behavior=play&eapi={encrypted}
```

- [ ] 实现 `KugouEapi::encrypt()` 
- [ ] 集成到 `play_url()` 方法
- [ ] 验证加密后的 URL 可用

### 48.2 Netease Weapi/Eapi 加密
**文件**: `audio/netease.rs` (修改, +80 行)

```rust
// weapi 加密链: text -> aes128 -> base64 -> aes128 -> base64 -> url encode
// eapi 加密链: url + text -> md5 -> aes128-ecb -> bin to hex
// 公钥: 0CoQUm9Q9zR2Yywu
// weapi 固定 key: "0CoJUm6Qyw8W8jud"
// iv: "0102030405060708"
```

- [ ] 实现 `NeteaseWeapi::encrypt()`
- [ ] 实现 `NeteaseEapi::encrypt()`
- [ ] 集成到 `search()` 和 `play_url()` 方法
- [ ] 验证加密后的 API 调用可用

### 48.3 Migu 加密增强
**文件**: `audio/migu.rs` (修改, +30 行)

```rust
// Migu 使用 RSA 签名: sign = rsa私钥.sign(message)
// 新版 API 需要 header: {"uid": "", "appId": "music"}
```

- [ ] 实现 `MiguSigner::sign()`
- [ ] 添加必要的请求头
- [ ] 验证签名后的 API 调用可用

---

## 三、Phase 49: NeoTrix 生态接线 (Ecosystem Wiring)

### 目标
将媒体源系统接入 NeoTrix 核心模块，而非实现完整生态集成。

### 49.1 NT-MEMORY KB 接线
**文件**: `core/kb_bridge.rs` (新建, ≤50 行)

```rust
/// 媒体搜索结果 → KB 节点存储
pub async fn store_search_result(result: &SearchResult, kb: &KnowledgeBase) -> Result<()> {
    // 每个 MediaItem → KB node (namespace: "media")
    // edges: "found_by" → search query node
}
```

- [ ] 搜索结果落盘 KB
- [ ] 媒体实体关联

### 49.2 NT-SHIELD 安全接线
**文件**: `core/security_bridge.rs` (新建, ≤40 行)

```rust
/// API Key 安全存储 (从环境变量读取，不硬编码)
pub fn get_api_key(source: &str) -> Option<String> {
    std::env::var(&format!("NEOTRIX_MEDIA_{}_API_KEY", source.to_uppercase())).ok()
}
```

- [ ] API Key 环境变量读取
- [ ] 移除硬编码密钥

### 49.3 NT-MIND 进化信号
**文件**: `core/evolution_bridge.rs` (新建, ≤30 行)

```rust
/// 源健康度变化 → 进化信号
pub fn emit_source_health_signal(source: &str, healthy: bool) {
    log::info!("[evolution] source {} health: {}", source, healthy);
}
```

- [ ] 源健康度信号

---

## 四、Phase 50: 搜索质量提升 (Search Quality)

### 目标
提升搜索结果的准确性和完整性。

### 50.1 多源搜索聚合
**文件**: `core/search_aggregator.rs` (新建, ≤60 行)

```rust
/// 并发搜索多个源，合并去重排序
pub async fn aggregate_search(query: &str, sources: &[Box<dyn MediaSource>]) -> Vec<MediaItem> {
    // 并发搜索 → 按相关性排序 → 去重 → 返回 top N
}
```

- [ ] 并发搜索
- [ ] 结果去重
- [ ] 相关性排序

### 50.2 搜索结果缓存
**文件**: `core/search_cache.rs` (新建, ≤40 行)

```rust
/// 搜索结果缓存 (LRU, TTL=1h)
pub struct SearchCache {
    cache: lru::LruCache<String, (SearchResult, std::time::Instant)>,
}
```

- [ ] LRU 缓存
- [ ] TTL 过期
- [ ] 缓存命中率统计

---

## 五、Phase 51: 播放体验优化 (Playback UX)

### 目标
提升播放体验的稳定性。

### 51.1 播放链接自动重试
**文件**: `core/playback_retry.rs` (新建, ≤40 行)

```rust
/// 播放链接失效时自动切换音质/源
pub async fn get_play_url_with_fallback(item: &MediaItem, quality: Quality) -> Result<ViewSource> {
    // 尝试目标音质 → 失败则降级 → 最终返回错误
}
```

- [ ] 音质降级重试
- [ ] 多源备用

### 51.2 播放历史记录
**文件**: `core/playback_history.rs` (新建, ≤30 行)

```rust
/// 播放历史记录
pub struct PlaybackHistory {
    history: Vec<PlaybackRecord>,
}
```

- [ ] 播放记录
- [ ] 历史查询

---

## 六、Phase 52: 测试覆盖 (Test Coverage)

### 目标
为核心功能添加测试。

### 52.1 单元测试
**文件**: `tests/unit/` (新建)

- [ ] `crypto_test.rs` — 加密/解密测试
- [ ] `search_test.rs` — 搜索逻辑测试
- [ ] `cache_test.rs` — 缓存测试

### 52.2 集成测试
**文件**: `tests/integration/` (新建)

- [ ] `source_test.rs` — 真实 API 调用测试 (需要网络)
- [ ] `kb_sync_test.rs` — KB 同步测试

---

## 七、执行计划

### Sprint 1: 加密能力 (Phase 48)
| 任务 | 优先级 | 预估工时 |
|------|--------|----------|
| Kugou eapi 加密 | **Critical** | 2h |
| Netease weapi/eapi 加密 | **Critical** | 3h |
| Migu RSA 签名 | **High** | 1h |

### Sprint 2: 生态接线 (Phase 49)
| 任务 | 优先级 | 预估工时 |
|------|--------|----------|
| KB 接线 | **High** | 1h |
| 安全接线 | **High** | 1h |
| 进化信号 | **Medium** | 0.5h |

### Sprint 3: 搜索质量 (Phase 50)
| 任务 | 优先级 | 预估工时 |
|------|--------|----------|
| 多源聚合 | **High** | 2h |
| 搜索缓存 | **Medium** | 1h |

### Sprint 4: 播放体验 (Phase 51)
| 任务 | 优先级 | 预估工时 |
|------|--------|----------|
| 自动重试 | **High** | 1h |
| 播放历史 | **Medium** | 0.5h |

### Sprint 5: 测试 (Phase 52)
| 任务 | 优先级 | 预估工时 |
|------|--------|----------|
| 单元测试 | **High** | 2h |
| 集成测试 | **Medium** | 1h |

**总预估**: ~14h (3-4 天)

---

## 八、验收标准

### Phase 48 加密
- [ ] Kugou eapi 加密 URL 可用
- [ ] Netease weapi 加密搜索可用
- [ ] Migu 签名 API 调用可用
- [ ] 无硬编码密钥

### Phase 49 生态
- [ ] 搜索结果可存入 KB
- [ ] API Key 从环境变量读取
- [ ] 源健康度信号发射

### Phase 50 搜索
- [ ] 多源聚合搜索结果正确
- [ ] 缓存命中率 > 70%

### Phase 51 播放
- [ ] 链接失效时自动降级重试
- [ ] 播放历史记录正确

### Phase 52 测试
- [ ] 单元测试通过
- [ ] 集成测试通过 (需网络)

---

## 九、文件变更预览

```
nt_world_media_source/
├── core/
│   ├── kb_bridge.rs          # [NEW] KB 接线
│   ├── security_bridge.rs    # [NEW] 安全接线
│   ├── evolution_bridge.rs   # [NEW] 进化信号
│   ├── search_aggregator.rs  # [NEW] 多源聚合
│   ├── search_cache.rs       # [NEW] 搜索缓存
│   ├── playback_retry.rs     # [NEW] 播放重试
│   └── playback_history.rs   # [NEW] 播放历史
├── audio/
│   ├── kugou.rs              # [MOD] +eapi 加密
│   ├── netease.rs            # [MOD] +weapi/eapi 加密
│   └── migu.rs               # [MOD] +RSA 签名
└── tests/
    ├── unit/
    │   ├── crypto_test.rs    # [NEW]
    │   ├── search_test.rs    # [NEW]
    │   └── cache_test.rs     # [NEW]
    └── integration/
        ├── source_test.rs    # [NEW]
        └── kb_sync_test.rs   # [NEW]
```

**新增**: ~12 文件, ~500 行
**修改**: 3 文件, ~170 行
**总计**: ~670 行新代码

---

## 十、与 v5 的差异

| 维度 | v5 计划 | v6 实际 |
|------|---------|---------|
| 新文件数 | 40+ | 12 |
| 新代码行 | 2000+ | ~670 |
| Phase 数 | 11 (37-47) | 5 (48-52) |
| 重点 | 全栈生态集成 | 加密能力 + 核心接线 |
| 复杂度 | 高 (ML/边缘/云/合规) | 低 (加密/缓存/测试) |

**v6 原则**: 每个 Phase ≤200 行，禁止过度工程化。

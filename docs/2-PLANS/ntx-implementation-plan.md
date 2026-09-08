# NTX 存储格式推进方案

> 完整实施计划: 4 阶段, 18 个文件, 预计 32 工时

## 阶段总览

```
Phase 1: 格式骨架 (8h)     → Header + TOC + Frame 读写 + 单文件可打开
Phase 2: 持久化索引 (10h)   → HNSW 持久化 + WAL 崩溃恢复 + Graph Segment
Phase 3: 同步引擎 (8h)     → SQLite↔NTX 双写 + 快照导出 + 导入恢复
Phase 4: 搜索集成 (6h)     → 替换 load_all_embeddings + 冷启动加速 + 基准测试
```

---

## Phase 1: 格式骨架 (8h)

> 目标: 定义 NTX 格式, 实现 Header/TOC/Frame 读写, 单文件可打开关闭

### 文件清单

| 文件 | 行数 | 内容 | 依赖 |
|------|------|------|------|
| `ntx/mod.rs` | ~80 | 公共 API: `NtxFile::create/open/close/commit` | format, wal |
| `ntx/format.rs` | ~300 | Header/TOC/SegmentDescriptor 结构体 + 序列化 | bincode, sha2, crc32fast |
| `ntx/frames.rs` | ~250 | KnowledgeFrame 读写 + 编解码 (raw/zstd/lz4) | zstd, crc32fast |
| `ntx/wal.rs` | ~200 | 嵌入式 WAL: append/checkpoint/recover | format, frames |

### 实现细节

**format.rs**:
```rust
pub const NTX_MAGIC: &[u8; 4] = b"NTX\0";
pub const NTX_VERSION: u16 = 0x0100;

#[repr(C)]
pub struct NtxHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub feature_flags: u8,      // bit0=lex, bit1=vec, bit2=graph, bit3=temporal
    pub compression: u8,        // 0=none, 1=zstd, 2=lz4
    pub footer_offset: u64,
    pub wal_offset: u64,        // 恒定 4096
    pub wal_size: u64,
    pub wal_checkpoint_pos: u64,
    pub wal_sequence: u64,
    pub frame_count: u64,
    pub node_count: u64,
    pub edge_count: u64,
    pub toc_checksum: [u8; 32],
    pub schema_hash: [u8; 32],
    pub _reserved: [u8; 3960],
}

pub struct NtxToc {
    pub magic: [u8; 4],         // "NNTC"
    pub version: u16,
    pub segments: Vec<SegmentDescriptor>,
}

pub struct SegmentDescriptor {
    pub segment_type: SegmentType,
    pub offset: u64,
    pub length: u64,
    pub checksum: [u8; 32],
}

pub enum SegmentType {
    Wal = 0,
    Frames = 1,
    Graph = 2,
    Vec = 3,
    Lex = 4,
    Time = 5,
}
```

**frames.rs**:
```rust
pub struct KnowledgeFrame {
    pub frame_id: u64,
    pub node_id: [u8; 36],      // UUID
    pub frame_type: FrameType,   // node/edge/kv/embed
    pub encoding: Encoding,      // raw/zstd/lz4
    pub payload: Vec<u8>,
    pub checksum: u32,           // CRC32
    pub timestamp: u64,
    pub tags: Option<serde_json::Value>,
}

impl KnowledgeFrame {
    pub fn encode(&self) -> Vec<u8>;
    pub fn decode(data: &[u8]) -> Result<Self>;
    pub fn compress(&self, level: u8) -> Vec<u8>;
    pub fn decompress(&self) -> Result<Vec<u8>>;
}
```

**wal.rs**:
```rust
pub struct EmbeddedWal {
    file: File,
    offset: u64,
    size: u64,
    sequence: u64,
    checkpoint_pos: u64,
}

impl EmbeddedWal {
    pub fn append(&mut self, frame: &KnowledgeFrame) -> Result<()>;
    pub fn checkpoint(&mut self, frames: &mut Vec<KnowledgeFrame>) -> Result<()>;
    pub fn recover(&mut self) -> Result<Vec<KnowledgeFrame>>;
    pub fn stats(&self) -> WalStats;
}
```

### 验收标准

- [ ] `NtxFile::create("test.ntx")` 创建有效 NTX 文件
- [ ] Header + TOC 写入正确, magic/version 校验通过
- [ ] 写入 100 个 KnowledgeFrame, 读回一致
- [ ] WAL append + checkpoint 流程正确
- [ ] 崩溃恢复: 写入 50 帧后模拟崩溃, 恢复后数据完整

---

## Phase 2: 持久化索引 (10h)

> 目标: HNSW 持久化到文件, Graph Segment 构建, Time Index

### 文件清单

| 文件 | 行数 | 内容 | 依赖 |
|------|------|------|------|
| `ntx/vec_segment.rs` | ~350 | 持久化 HNSW: 构建/序列化/mmap 加载 | instant-distance, memmap2 |
| `ntx/graph_segment.rs` | ~250 | 邻接表: 构建/序列化/查询 | bincode |
| `ntx/time_segment.rs` | ~150 | 时间索引: 构建/查询 | - |

### 实现细节

**vec_segment.rs**:
```rust
pub struct VecSegment {
    dim: usize,
    vectors: Vec<Vec<f32>>,
    node_ids: Vec<[u8; 36]>,
    hnsw: Option<HnswMap<FloatVec, String>>,
}

impl VecSegment {
    /// 从 SQLite embeddings 表构建
    pub fn from_embeddings(conn: &Connection) -> Result<Self>;
    
    /// 序列化到字节
    pub fn serialize(&self) -> Vec<u8>;
    
    /// 从字节反序列化 (mmap 加载)
    pub fn deserialize(data: &[u8]) -> Result<Self>;
    
    /// 持久化到 NTX 文件
    pub fn write_to(&self, file: &mut File, offset: u64) -> Result<SegmentDescriptor>;
    
    /// 从 NTX 文件加载 (mmap)
    pub fn load_from(file: &File, descriptor: &SegmentDescriptor) -> Result<Self>;
    
    /// 向量搜索
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<(String, f64)>;
    
    /// 增量添加向量
    pub fn insert(&mut self, node_id: [u8; 36], vector: Vec<f32>);
}
```

**graph_segment.rs**:
```rust
pub struct GraphSegment {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

pub struct GraphNode {
    pub node_id: [u8; 36],
    pub edge_start: u32,
    pub edge_count: u32,
}

pub struct GraphEdge {
    pub target_id: [u8; 36],
    pub relation: u32,       // RelationType 枚举
    pub weight: f32,
}

impl GraphSegment {
    /// 从 SQLite edges 表构建
    pub fn from_edges(conn: &Connection) -> Result<Self>;
    
    /// 序列化到字节
    pub fn serialize(&self) -> Vec<u8>;
    
    /// 从字节反序列化
    pub fn deserialize(data: &[u8]) -> Result<Self>;
    
    /// 查询邻居 (O(1))
    pub fn neighbors(&self, node_id: &[u8; 36], depth: usize) -> Vec<(&GraphNode, &GraphEdge)>;
    
    /// BFS 子图提取
    pub fn subgraph(&self, center: &[u8; 36], max_depth: usize) -> GraphSegment;
}
```

**time_segment.rs**:
```rust
pub struct TimeSegment {
    entries: Vec<TimeEntry>,
}

pub struct TimeEntry {
    pub frame_id: u64,
    pub timestamp: u64,
    pub offset: u64,        // Knowledge Frame 字节偏移
}

impl TimeSegment {
    pub fn from_frames(frames: &[KnowledgeFrame]) -> Self;
    pub fn serialize(&self) -> Vec<u8>;
    pub fn deserialize(data: &[u8]) -> Result<Self>;
    pub fn query_range(&self, since: u64, until: u64) -> Vec<&TimeEntry>;
    pub fn latest(&self, n: usize) -> Vec<&TimeEntry>;
}
```

### 验收标准

- [ ] 390K 向量构建 HNSW + 序列化 < 30s
- [ ] mmap 加载 Vec Segment < 100ms
- [ ] HNSW 搜索精度: recall@10 > 0.95 (vs 全量扫描)
- [ ] Graph Segment 邻居查询 O(1)
- [ ] Time Index 范围查询正确

---

## Phase 3: 同步引擎 (8h)

> 目标: SQLite↔NTX 双写, 快照导出, 导入恢复

### 文件清单

| 文件 | 行数 | 内容 | 依赖 |
|------|------|------|------|
| `ntx/sync.rs` | ~400 | 双写/快照/导入/增量同步 | ntx/*, rusqlite |

### 实现细节

```rust
pub struct NtxSync {
    ntx: NtxFile,
    conn: Connection,
    sync_mode: SyncMode,
}

pub enum SyncMode {
    /// 运行时: SQLite 为真相源, NTX 为索引层
    DualWrite,
    /// 导出: SQLite → NTX 单向快照
    Snapshot,
    /// 导入: NTX → SQLite 单向恢复
    Import,
}

impl NtxSync {
    /// 双写模式: 写入 SQLite + NTX WAL
    pub fn write_dual(&mut self, node: &KnowledgeNode) -> Result<()>;
    
    /// 快照模式: SQLite 全量 → NTX 单文件
    pub fn snapshot(&mut self) -> Result<NtxFile>;
    
    /// 导入模式: NTX 单文件 → SQLite
    pub fn import(&mut self) -> Result<ImportStats>;
    
    /// 增量同步: NTX WAL 中未同步的帧 → SQLite
    pub fn sync_incremental(&mut self) -> Result<SyncStats>;
    
    /// 向量索引同步: SQLite embeddings → NTX Vec Segment
    pub fn sync_vectors(&mut self) -> Result<()>;
    
    /// 图谱同步: SQLite edges → NTX Graph Segment
    pub fn sync_graph(&mut self) -> Result<()>;
}

pub struct ImportStats {
    pub nodes_imported: usize,
    pub edges_imported: usize,
    pub embeddings_imported: usize,
    pub kv_imported: usize,
    pub duration_ms: u64,
}

pub struct SyncStats {
    pub frames_synced: usize,
    pub bytes_synced: u64,
    pub duration_ms: u64,
}
```

### 集成点

在 `KnowledgeBase` 中新增:
```rust
impl KnowledgeBase {
    /// 打开 NTX 同步 (如果 .ntx 文件存在)
    pub fn with_ntx_sync(&mut self) -> Result<()>;
    
    /// 快照导出
    pub fn snapshot_to_ntx(&self, path: &Path) -> Result<NtxFile>;
    
    /// 从 NTX 导入
    pub fn import_from_ntx(&mut self, path: &Path) -> Result<ImportStats>;
    
    /// NTX 搜索 (持久化 HNSW)
    pub fn search_ntx(&self, query: &[f32], top_k: usize) -> Vec<(String, f64)>;
}
```

### 验收标准

- [ ] 双写模式: SQLite 和 NTX 数据一致
- [ ] 快照导出: 390K 节点导出 < 60s
- [ ] 导入恢复: .ntx 文件导入 SQLite < 60s
- [ ] 增量同步: WAL 中新帧正确追加

---

## Phase 4: 搜索集成 (6h)

> 目标: 替换 load_all_embeddings, 冷启动加速, 基准测试

### 文件清单

| 文件 | 行数 | 内容 | 依赖 |
|------|------|------|------|
| `ntx/search_bridge.rs` | ~200 | NTX 搜索桥接: 替换内存向量加载 | ntx/vec_segment |
| `ntx/benchmark.rs` | ~150 | 基准测试: 冷启动/搜索延迟/压缩率 | criterion |

### 实现细节

**search_bridge.rs**:
```rust
/// 替换 nt_memory_search.rs 中的 load_all_embeddings()
pub struct NtxSearchBridge {
    vec_segment: VecSegment,    // mmap 加载, 零拷贝
    hnsw_ready: bool,
}

impl NtxSearchBridge {
    /// 冷启动: mmap 加载 Vec Segment (< 100ms)
    pub fn open(ntx_path: &Path) -> Result<Self>;
    
    /// 向量搜索: 直接查询持久化 HNSW
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<(String, f64)>;
    
    /// 替换 semantic_search 的全量加载
    pub fn embedding_search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
}
```

**集成到 nt_memory_search.rs**:
```rust
// 修改 hybrid_search Tier 4
// 旧: load_all_embeddings() → O(N) 内存
// 新: NtxSearchBridge::search() → O(log N) mmap

pub fn hybrid_search(...) {
    // ...
    // Tier 4: Embedding Cosine Rerank
    if let Some(bridge) = &self.ntx_bridge {
        // mmap HNSW 搜索, 无全量加载
        let emb_results = bridge.search(&query_embedding, limit * 2);
        // ... 融合
    } else {
        // 回退: 旧的全量加载路径
        let all_embeddings = load_all_embeddings(&conn)?;
        // ...
    }
}
```

### 验收标准

- [ ] 冷启动时间: 390K 向量从 5s → < 200ms
- [ ] 搜索延迟: P50 < 5ms, P99 < 20ms
- [ ] 压缩率: .ntx 文件比 SQLite 小 30-50%
- [ ] 搜索质量: recall@10 > 0.95 (vs 旧路径)

---

## 依赖引入清单

| Crate | 版本 | 用途 | 引入阶段 |
|-------|------|------|----------|
| `sha2` | 0.10 | SHA-256 校验 | Phase 1 |
| `memmap2` | 0.9 | mmap 向量索引 | Phase 2 |
| `bincode` | 1.3 | 二进制序列化 | Phase 1 |
| `tempfile` | 3 | 临时文件 (已有) | - |

## 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| mmap 平台兼容性 | macOS/Linux 已支持, Windows 需额外处理 | `memmap2` 跨平台 |
| HNSW 增量更新复杂 | 全量重建可接受 (定期) | Phase 4 优化 |
| FTS5 快照兼容性 | FTS5 数据库文件可内嵌 | Phase 1 验证 |
| 写入放大 | 双写增加 I/O | WAL 批量提交 |

## 与旧系统兼容

| 场景 | 策略 |
|------|------|
| 无 .ntx 文件 | 纯 SQLite 模式 (当前行为) |
| 有 .ntx 文件 | NTX 为索引层, SQLite 为事务层 |
| .ntx 损坏 | 自动回退 SQLite, 重建 NTX |
| 版本不兼容 | Header.version 检查, 提示升级 |

# NTX 单文件存储格式架构设计

> 借鉴 Memvid MV2, 解决 NeoTrix KB 10 大痛点

## 1. 设计目标

| 目标 | 度量 | 当前差距 | 状态 |
|------|------|----------|------|
| 持久化向量索引 | 冷启动 0 重建 | 当前 HNSW 每次重启重建 (390K 向量) | ✅ 已实现 |
| 崩溃恢复 | 嵌入式 WAL, 零外部文件 | 当前依赖 SQLite WAL (.wal/.shm) | ✅ 已实现 |
| 单文件便携 | `.ntx` 可分享/迁移 | 当前 SQLite + 多表 + 外部索引 | ✅ 已实现 |
| 追加只读 | 时间旅行查询 | 当前原地更新, 无版本历史 | ✅ 已实现 |
| 压缩存储 | Zstd 按帧压缩 | 当前无压缩, 浪费 30-50% 空间 | ✅ 已实现 |
| WAL 压缩 | Zstd/LZ4 压缩 | 当前 WAL 无压缩 | ✅ 已实现 |
| mmap 加载 | 零拷贝加载 | 当前传统加载 | ✅ 已实现 |
| 消除 FTS5 内容复制 | 存储减半 | 当前 FTS5 复制 title+summary+content | ✅ 已实现 |
| 并发安全 | RwLock + 文件锁 | 当前无同步机制 | ✅ 已实现 |
| 版本兼容性 | 版本检查 | 当前无版本检查 | ✅ 已实现 |

## 2. NTX 文件格式规范 (v1.0)

```
┌─────────────────────────────────────────────────────────────┐
│                        .ntx FILE                            │
├─────────────────────────────────────────────────────────────┤
│ Header                 │ 4 KB                               │
├─────────────────────────────────────────────────────────────┤
│ Embedded WAL           │ 1-64 MB (容量自适应)                │
├─────────────────────────────────────────────────────────────┤
│ Knowledge Frames       │ 追加只读知识帧 (压缩)               │
├─────────────────────────────────────────────────────────────┤
│ Graph Segment          │ 知识图谱邻接表 (压缩)               │
├─────────────────────────────────────────────────────────────┤
│ Vec Index Segment      │ 持久化 HNSW 向量索引                │
├─────────────────────────────────────────────────────────────┤
│ Lex Segment            │ 全文索引 (Tantivy 或 FTS5 快照)     │
├─────────────────────────────────────────────────────────────┤
│ Time Index Segment     │ 时间旅行索引                        │
├─────────────────────────────────────────────────────────────┤
│ TOC (Footer)           │ 段目录 + 校验和                     │
└─────────────────────────────────────────────────────────────┘
```

### 2.1 Header (4096 bytes)

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 4 | `magic` | `NTX\0` (0x4E 0x54 0x58 0x00) |
| 4 | 2 | `version` | 格式版本 (1.0 = 0x0100) |
| 6 | 1 | `feature_flags` | 位图: bit0=lex, bit1=vec, bit2=graph, bit3=temporal |
| 7 | 1 | `compression` | 0=none, 1=zstd, 2=lz4 |
| 8 | 8 | `footer_offset` | TOC 字节偏移 |
| 16 | 8 | `wal_offset` | WAL 字节偏移 (恒定 4096) |
| 24 | 8 | `wal_size` | WAL 区域大小 |
| 32 | 8 | `wal_checkpoint_pos` | 最后检查点序列号 |
| 40 | 8 | `wal_sequence` | 当前 WAL 序列号 |
| 48 | 8 | `frame_count` | 知识帧总数 |
| 56 | 8 | `node_count` | 图谱节点总数 |
| 64 | 8 | `edge_count` | 图谱边总数 |
| 72 | 32 | `toc_checksum` | SHA-256 of TOC |
| 104 | 32 | `schema_hash` | SQLite schema 哈希 (兼容性校验) |
| 136 | 3960 | reserved | 零填充, 未来扩展 |

### 2.2 Knowledge Frame (知识帧)

每条知识 = 一个不可变帧, 追加写入, 永不修改:

```
┌──────────────────────────────────────┐
│ frame_id      │ 8 bytes (u64 LE)     │  单调递增
│ node_id       │ 36 bytes (UUID)      │  关联 SQLite nodes.id
│ frame_type    │ 1 byte               │  0=node, 1=edge, 2=kv, 3=embed
│ encoding      │ 1 byte               │  0=raw, 1=zstd, 2=lz4
│ payload_len   │ 4 bytes (u32 LE)     │  压缩后长度
│ payload       │ variable             │  压缩后的帧数据
│ checksum      │ 4 bytes (CRC32)      │  帧完整性校验
│ timestamp     │ 8 bytes (u64 LE)     │  Unix 秒
│ tags_len      │ 2 bytes (u16 LE)     │  标签 JSON 长度
│ tags          │ variable             │  JSON 标签 (可选)
└──────────────────────────────────────┘
```

**帧类型 (frame_type)**:
- `0x00` NodeFrame: 序列化的 KnowledgeNode (JSON)
- `0x01` EdgeFrame: 序列化的 KnowledgeEdge (JSON)
- `0x02` KvFrame: key-value 对 (namespace + key + value)
- `0x03` EmbedFrame: 向量 BLOB (node_id + dimension + f32 array)

### 2.3 Graph Segment (图谱段)

邻接表格式, 支持 O(1) 邻居查询:

```
┌──────────────────────────────────────┐
│ magic         │ "NTGR"               │
│ node_count    │ 4 bytes (u32 LE)     │
│ edge_count    │ 4 bytes (u32 LE)     │
│ node_index    │ [node_count] NodeEntry│
│ edge_data     │ [edge_count] EdgeEntry│
│ checksum      │ 32 bytes (SHA-256)   │
└──────────────────────────────────────┘

NodeEntry:
  node_id    │ 36 bytes (UUID)
  edge_start │ 4 bytes (u32 LE)  → edge_data 索引
  edge_count │ 4 bytes (u32 LE)  → 边数量

EdgeEntry:
  target_id  │ 36 bytes (UUID)
  relation   │ 4 bytes (u32)     → relation_type 枚举
  weight     │ 4 bytes (f32 LE)
```

### 2.4 Vec Index Segment (向量索引段)

持久化 HNSW, 直接 mmap 加载, 零重建:

```
┌──────────────────────────────────────┐
│ magic         │ "NTVH"               │
│ version       │ 2 bytes              │
│ dimension     │ 4 bytes (u32 LE)     │
│ vector_count  │ 4 bytes (u32 LE)     │
│ hnsw_params   │ 12 bytes            │  (M, ef_construction, max_level)
│ vectors       │ [vector_count * dim * 4 bytes]  │  f32 LE
│ node_ids      │ [vector_count * 36 bytes]        │  UUID
│ hnsw_layers   │ variable            │  HNSW 层级结构
│ checksum      │ 32 bytes (SHA-256)  │
└──────────────────────────────────────┘
```

### 2.5 Lex Segment (全文索引段)

两种模式 (可选):
- **模式 A**: Tantivy 段快照 (推荐, 需要 `tantivy` 依赖)
- **模式 B**: FTS5 数据库文件内嵌 (兼容当前实现)

```
模式 B (嵌入式 FTS5):
┌──────────────────────────────────────┐
│ magic         │ "NTLX"               │
│ fts_db_size   │ 8 bytes (u64 LE)     │
│ fts_db_data   │ [fts_db_size bytes]  │  SQLite FTS5 数据库文件
│ checksum      │ 32 bytes (SHA-256)   │
└──────────────────────────────────────┘
```

### 2.6 Time Index Segment (时间索引段)

```
┌──────────────────────────────────────┐
│ magic         │ "NTTI"               │
│ entry_count   │ 4 bytes (u32 LE)     │
│ entries       │ [entry_count] TimeEntry│
│ checksum      │ 32 bytes (SHA-256)   │
└──────────────────────────────────────┘

TimeEntry:
  frame_id    │ 8 bytes (u64)
  timestamp   │ 8 bytes (u64)
  offset      │ 8 bytes (u64)  → Knowledge Frame 字节偏移
```

### 2.7 TOC (Footer)

```
┌──────────────────────────────────────┐
│ magic         │ "NNTC"               │
│ version       │ 2 bytes              │
│ segment_count │ 4 bytes (u32 LE)     │
│ segments[]    │ SegmentDescriptor[]  │
│ checksum      │ 32 bytes (SHA-256)   │
└──────────────────────────────────────┘

SegmentDescriptor:
  segment_type │ 1 byte   (0=wal, 1=frames, 2=graph, 3=vec, 4=lex, 5=time)
  offset       │ 8 bytes  (u64 LE)
  length       │ 8 bytes  (u64 LE)
  checksum     │ 32 bytes (SHA-256)
```

## 3. 嵌入式 WAL 崩溃恢复

### 3.1 WAL 条目格式

```
┌──────────────────────────────────────┐
│ sequence    │ 8 bytes (u64 LE)       │
│ entry_type  │ 1 byte                 │  0x01=append, 0x02=update, 0x03=delete
│ payload_len │ 4 bytes (u32 LE)       │
│ payload     │ variable               │  KnowledgeFrame 序列化
│ checksum    │ 4 bytes (CRC32)        │
└──────────────────────────────────────┘
```

### 3.2 检查点策略

| 条件 | 动作 |
|------|------|
| WAL 占用 ≥ 75% | 触发检查点: WAL → Data Frames |
| 每 1000 条目 | 强制检查点 |
| `commit()` 调用 | 立即检查点 + fsync |
| 崩溃恢复 | 重放 `sequence > wal_checkpoint_pos` 的条目 |

### 3.3 恢复流程

```
1. 读取 Header → 获取 wal_checkpoint_pos
2. 读取 WAL 区域 → 过滤 sequence > wal_checkpoint_pos 的条目
3. 按 sequence 排序 → 重放到 Data Frames
4. 更新 Header: wal_checkpoint_pos = max(sequence)
5. 重写 TOC (段偏移更新)
```

## 4. 并发控制

### 4.1 锁机制

```rust
pub struct NtxFile {
    // ... 现有字段
    lock: RwLock<()>,        // 读写锁: 读操作用 read(), 写操作用 write()
    wal_lock: Mutex<()>,     // WAL 锁: 保护 WAL 顺序写入
}
```

### 4.2 锁使用模式

| 操作 | 锁类型 | 说明 |
|------|--------|------|
| `put_frame()` | `wal_lock.lock()` | WAL 顺序写入保护 |
| `put_frames()` | `wal_lock.lock()` | 批量写入保护 |
| `commit()` | `lock.write()` | 写操作保护 |
| `frames()` | 无锁 | 只读访问 |
| `vec_segment()` | 无锁 | 只读访问 |

### 4.3 文件锁

```rust
// 创建时获取独占锁
file.lock_exclusive()?;

// 只读打开时获取共享锁
file.lock_shared()?;

// Drop 时自动释放
impl Drop for NtxFile {
    fn drop(&mut self) {
        if let Err(e) = self.file.unlock() {
            eprintln!("[NTX] 释放文件锁失败: {}", e);
        }
    }
}
```

### 4.4 版本兼容性

```rust
// 打开时检查版本
if header.version < 0x0100 || header.version > 0x0101 {
    return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("NTX 版本不兼容: 期望 0x0100-0x0101, 实际 0x{:04X}", header.version),
    ));
}
```

## 4. 持久化向量索引

### 4.1 构建流程

```
1. 从 SQLite embeddings 表加载所有向量
2. 构建 HNSW 索引 (instant-distance)
3. 序列化到 .ntx 文件 Vec Index Segment
4. 更新 TOC
```

### 4.2 查询流程

```
1. mmap 加载 Vec Index Segment (零拷贝)
2. 查询 HNSW → top-k 候选
3. 返回 (node_id, distance) 对
```

### 4.3 增量更新

```
新向量写入 → SQLite embeddings 表 + WAL (append EmbedFrame)
定期重建 → 增量合并新向量到 HNSW (不重建全量)
```

## 5. SQLite ↔ NTX 同步机制

### 5.1 双写模式 (运行时)

```
写入路径:
  write_memory_entry()
    ├── 1. SQLite INSERT/UPDATE (事务)
    ├── 2. NTX WAL append (帧)
    └── 3. 标记脏页

读取路径:
  search()
    ├── SQLite FTS5/BM25 (全文)
    ├── NTX Vec Index (向量, mmap)
    └── 融合结果
```

### 5.2 快照模式 (导出/分享)

```
snapshot_to_ntx():
  1. SQLite BEGIN IMMEDIATE
  2. 遍历 nodes/edges/embeddings 表
  3. 逐帧写入 NTX Data Frames
  4. 构建 Graph Segment
  5. 构建 Vec Index Segment
  6. 构建 Lex Segment (FTS5 快照)
  7. 构建 Time Index
  8. 写入 TOC + Header
  9. SQLite COMMIT
```

### 5.3 导入模式 (恢复)

```
import_from_ntx():
  1. 读取 TOC → 定位各段
  2. 遍历 Data Frames → SQLite INSERT
  3. 重建 Graph Cache
  4. 重建 HNSW Index
  5. 重建 FTS5 Index
```

## 6. 缺陷修补清单

| # | 当前痛点 | NTX 解决方案 | 优先级 |
|---|---------|-------------|--------|
| 1 | HNSW 每次启动重建 | 持久化 Vec Index Segment, mmap 加载 | 🔴 P0 |
| 2 | FTS5 内容复制 (2× 存储) | Lex Segment 嵌入 FTS5 数据库文件 | 🔴 P0 |
| 3 | 无崩溃恢复 (外部 WAL) | 嵌入式 WAL, 检查点策略 | 🔴 P0 |
| 4 | 无时间旅行 | Time Index + 追加只读帧 | 🟡 P1 |
| 5 | 无压缩 | Zstd 按帧压缩 (30-50% 节省) | 🟡 P1 |
| 6 | 不可移植 | 单文件 .ntx, 零依赖打开 | 🟡 P1 |
| 7 | PQ codebook 每次查询从 DB 加载 | 缓存在内存 + 持久化到 NTX | 🟢 P2 |
| 8 | load_all_embeddings 全量加载 | mmap Vec Index, 按需读取 | 🟢 P2 |
| 9 | 写入锁争用 (单 Mutex) | WAL 追加不阻塞读取 | 🟢 P2 |
| 10 | schema 迁移手工编码 | NTX schema_hash 校验 + 版本兼容 | 🟢 P3 |

## 7. 依赖矩阵

| 依赖 | 用途 | 已有? | 需引入? |
|------|------|-------|---------|
| `zstd` | 帧压缩 | ✅ 已有 | - |
| `crc32fast` | 帧校验 | ✅ 已有 | - |
| `sha2` | 段校验 | ✅ 已有 | - |
| `memmap2` | mmap 加载 | ✅ 已有 | - |
| `instant-distance` | HNSW | ✅ 已有 | - |
| `bincode` | 二进制序列化 | ✅ 已有 | - |
| `tempfile` | 临时文件 | ✅ 已有 | - |
| `fs2` | 文件锁 | ✅ 已有 | - |

## 8. 文件结构

```
neotrix-core/src/l1_action/nt_memory/nt_memory_kb/
├── ntx/
│   ├── mod.rs              # NTX 公共 API + 并发控制
│   ├── format.rs           # NTX 格式定义 (Header, TOC, Segment)
│   ├── wal.rs              # 嵌入式 WAL 实现
│   ├── frames.rs           # Knowledge Frame 读写
│   ├── graph_segment.rs    # Graph Segment 构建/查询
│   ├── vec_segment.rs      # Vec Index Segment (持久化 HNSW)
│   ├── lex_segment.rs      # Lex Segment (FTS5 快照)
│   ├── time_segment.rs     # Time Index Segment
│   └── sync.rs             # SQLite ↔ NTX 同步
```

## 9. 向后兼容策略

| 场景 | 策略 |
|------|------|
| 旧 SQLite KB 存在 | `snapshot_to_ntx()` 一次性迁移 |
| NTX 文件不存在 | 回退到纯 SQLite 模式 (当前行为) |
| NTX 版本不兼容 | Header.version 检查, 提示升级 |
| 混合读写 | SQLite 为真相源, NTX 为索引层 |

# NTX 缺陷修复 + 架构补齐方案

> 基于 11 文件深度审查, 修复 6 个 P0 + 22 个 P1 缺陷, 补齐设计文档缺失项。

## 审查摘要

| 等级 | 数量 | 示例 |
|------|------|------|
| P0 致命 | 6 | search_bridge 占位未实现、帧解码丢失 uncompressed_len、close 吞错误 |
| P1 重要 | 22 | 段无 magic/checksum、O(N²) 查找、TOC 不验证、双文件描述符 |
| P2 次要 | 12 | 版本检查、压缩回退静默、TOCTOU 竞态 |

---

## Phase A: P0 致命修复 (阻塞发布)

### A1. frames.rs — `decode_bytes()` 丢失 `uncompressed_len`

**问题**: `decode_bytes()` 设置 `uncompressed_len: 0`, 导致所有 Zstd/Lz4 帧从磁盘加载后无法解压。

**修复**:
- `encode_bytes()` 增加 `uncompressed_len` 字段 (4 bytes, 放在 `checksum` 之前)
- `decode_bytes()` 读取该字段
- 格式版本 bump → `NTX_VERSION = 0x0101`

**文件**: `ntx/frames.rs:119, 179`

### A2. search_bridge.rs — TODO 占位未实现

**问题**: `open()` 中 `vec_segment` / `graph_segment` 始终为 `None`, 搜索永远返回空。

**修复**:
- `open()` 调用 `NtxFile::open_read_only()` → 直接读取 `ntx.vec_segment()` / `ntx.graph_segment()`
- 删除 `None // 占位` 注释, 替换为实际引用

**文件**: `ntx/search_bridge.rs:36-52`

### A3. mod.rs — `close()` 吞掉 commit 错误

**问题**: `close()` 返回 `Ok(())` 而非 `commit()` 结果。

**修复**:
```rust
pub fn close(mut self) -> std::io::Result<()> {
    self.commit()  // 直接返回, 不再忽略
}
```

**文件**: `ntx/mod.rs:327-330`

### A4. 缺失 `lex_segment.rs`

**问题**: 设计文档列出此文件, `SegmentType::Lex` 枚举存在, 但无实现。

**修复**:
- 创建 `ntx/lex_segment.rs` — 嵌入 FTS5 数据库文件快照
- 实现 `LexSegment::new(db_bytes)`, `write_to()`, `read_from()`
- 在 `mod.rs` 注册模块

**文件**: 新建 `ntx/lex_segment.rs`

### A5. sync.rs — `import_to_sqlite()` 空操作

**问题**: 方法名暗示"从 NTX 恢复到 SQLite", 但实际只返回元数据, 不写入任何数据。

**修复**:
- 实现 `import_to_sqlite(&self, conn: &Connection)` — 读取 NTX 帧, 批量 INSERT INTO nodes/edges
- 添加事务包装

**文件**: `ntx/sync.rs:142-154`

### A6. KnowledgeBase 无 NTX 集成

**问题**: NTX 模块与 KB 完全脱节, 无字段、无初始化、无双写。

**修复**:
- `KnowledgeBase` 增加 `ntx_index: Option<NtxIndexManager>` 字段
- `KnowledgeBase::open()` 检测 `.ntx` 文件, 存在则加载
- `KnowledgeBase::insert_node()` 双写 NTX
- `KnowledgeBase::search_diverse()` 优先从 NTX 向量搜索

**文件**: `nt_memory_kb/mod.rs`

---

## Phase B: P1 重要修复 (质量门)

### B1. 段 magic bytes + SHA-256 校验

| 段 | Magic | 位置 |
|----|-------|------|
| Vec | `NTVH` | `vec_segment.rs` |
| Graph | `NTGR` | `graph_segment.rs` |
| Time | `NTTI` | `time_segment.rs` |
| Lex | `NTLX` | `lex_segment.rs` |

**修复**: 每个 `write_to()` 开头写入 4 字节 magic + 32 字节 SHA-256; `read_from()` 验证。

### B2. wal.rs — `try_clone()` 双文件描述符

**问题**: WAL 和 NtxFile 各持有独立 `File` 句柄, 可能交错写入。

**修复**:
- `EmbeddedWal` 不持有 `File`, 只持有 offset/size 元数据
- WAL 操作通过 `NtxFile.file` 统一 I/O
- `EmbeddedWal` 改为方法集 (接受 `&mut File` 参数)

### B3. graph_segment.rs — O(N²) 查找

**修复**:
- 增加 `HashMap<[u8; 36], usize>` 索引 (node_id → index)
- `add_edge()` / `bfs()` / `neighbors()` 使用索引 O(1) 查找

### B4. time_segment.rs — 每次 insert 排序

**修复**:
- `insert()` 不排序, 标记 `dirty`
- 新增 `sort()` 方法, 在 `write_to()` 前自动调用
- `range()` 使用二分查找

### B5. format.rs — TOC 验证

**修复**:
- `NtxToc::read_from()` 读取后验证 TOC magic (`NNTC`)
- 检查 `footer_offset` 在文件大小范围内
- `NtxHeader::from_bytes()` 校验版本兼容性

### B6. vec_segment.rs — 暴力搜索 vs HNSW

**修复**:
- 暂保留暴力搜索 (正确性优先)
- TODO: 集成 `instant-distance` crate 的持久化 HNSW (Phase C)

### B7. ntx_integration.rs — `bytes_to_uuid()` 输出 72 字符

**修复**:
- `bytes_to_uuid()` 输出标准 36 字符 UUID 格式 (带连字符)
- 修复 `uuid_to_bytes()` 反向解析

### B8. ntx_integration.rs — 无文件缓存

**修复**:
- 增加 `cached_ntx: Option<NtxFile>` 字段
- 首次搜索时打开并缓存
- `full_sync()` 后自动失效缓存

### B9. 去重 `uuid_to_bytes()`

**修复**:
- 提取到 `ntx/format.rs` 作为公共工具函数
- `sync.rs`, `ntx_integration.rs` 统一引用

### B10. mod.rs — 检查点 O(N²) 去重

**修复**:
- 使用 `HashSet<u64>` 跟踪已见 frame_id
- 去重复杂度从 O(N²) 降至 O(N)

### B11. mod.rs — `open_read_only()` 实际可写

**修复**:
- `open_read_only()` 使用 `OpenOptions::new().read(true).write(false)`
- 增加 `readonly: bool` 字段到 `NtxFile`

### B12. mod.rs — `write_frames_segment()` 不校验帧校验和

**修复**: 写入前验证 `frame.verify()`, 失败则返回错误。

### B13. mod.rs — `load_frames_segment()` 静默丢弃损坏帧

**修复**: 解码失败时记录错误, 返回 `Err` 而非跳过。

### B14. sync.rs — `snapshot_from_sqlite()` 不构建 VecSegment

**修复**: 在 `full_sync()` 中构建 VecSegment 并 `put_vec_segment()`.

### B15. sync.rs — `incremental_sync()` 不更新段

**修复**: 增量同步时追加到内存 VecSegment/GraphSegment.

### B16. Drop impl 静默丢弃错误

**修复**: `Drop` 中使用 `eprintln!` 记录 commit 失败。

---

## Phase C: P2 优化 (性能/健壮性)

| # | 修复 | 文件 |
|---|------|------|
| C1 | 版本兼容性检查 | `format.rs` |
| C2 | 压缩回退日志 | `frames.rs` |
| C3 | `assert_eq!` → `Result` | `vec_segment.rs` |
| C4 | `find_by_id()` HashMap 索引 | `vec_segment.rs` |
| C5 | `open_or_create()` TOCTOU | `mod.rs` |
| C6 | WAL 清零分块写入 | `wal.rs` |
| C7 | 空帧校验 | `wal.rs` |
| C8 | 删除死代码 `bytes_to_f32_vec` | `sync.rs` |

---

## 文件变更清单

| 文件 | 变更类型 | 优先级 |
|------|----------|--------|
| `ntx/frames.rs` | 修复 decode_bytes | P0 |
| `ntx/search_bridge.rs` | 实现段加载 | P0 |
| `ntx/mod.rs` | close/Drop/checkpoint 修复 | P0 |
| `ntx/lex_segment.rs` | 新建 | P0 |
| `ntx/sync.rs` | 实现 import_to_sqlite | P0 |
| `nt_memory_kb/mod.rs` | 集成 NTX | P0 |
| `ntx/format.rs` | magic/checksum/uuid 工具 | P1 |
| `ntx/vec_segment.rs` | magic/checksum | P1 |
| `ntx/graph_segment.rs` | magic/checksum/HashMap 索引 | P1 |
| `ntx/time_segment.rs` | magic/checksum/二分查找 | P1 |
| `ntx/wal.rs` | 移除 try_clone | P1 |
| `ntx/ntx_integration.rs` | 缓存/uuid 修复 | P1 |
| `ntx/benchmark.rs` | 小修 | P2 |

---

## 执行顺序

```
Phase A (P0): ~4h
  A1 → A2 → A3 → A4 → A5 → A6 (串行, 有依赖)

Phase B (P1): ~6h
  B1-B9 可并行 (无依赖)
  B10-B16 可并行 (依赖 A 完成)

Phase C (P2): ~2h
  C1-C8 全部并行
```

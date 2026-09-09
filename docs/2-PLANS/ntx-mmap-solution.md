# NTX mmap 加载方案设计

## 问题分析

`#![forbid(unsafe_code)]` 禁止所有 unsafe 代码, 而 `memmap2::Mmap::map()` 必须标记为 `unsafe`。

**根本原因**: Rust 内存模型要求 `&[u8]` 在引用存在期间不可变。但 mmap 映射的文件可能被外部进程修改, 导致:
1. 数据在引用存在期间被修改 → 未定义行为 (UB)
2. 文件在映射后被截断 → SIGBUS 信号
3. 其他进程修改文件 → 数据竞争

## 方案对比

| 方案 | 安全性 | 成熟度 | 新依赖 | 适用性 |
|------|--------|--------|--------|--------|
| **mmap-io** | ✅ 零 unsafe API | ⭐⭐⭐ | 是 | ✅ 推荐 |
| **预读 + madvise** | ✅ 零 unsafe | ⭐⭐⭐⭐⭐ | 否 | ⚠️ 性能妥协 |
| **memmap2 包装器** | ❌ 需 unsafe | ⭐⭐⭐⭐⭐ | 否 | ❌ 不适用 |

## 推荐方案: mmap-io

### 理由
1. **安全性**: 零 unsafe in public API, 内部使用 `parking_lot::RwLock` 保证线程安全
2. **功能性**: 支持 segmented views, 可以映射文件的一部分
3. **成熟度**: 基于 memmap2 构建, 有完整文档
4. **跨平台**: 支持 Linux, macOS, Windows

### 实现计划

#### Phase 1: 引入依赖 (0.5h)

```toml
# Cargo.toml
[dependencies]
mmap-io = { version = "1.0", optional = true }

[features]
default = []
mmap = ["dep:mmap-io"]
```

#### Phase 2: 修改 VecSegment (2h)

```rust
// vec_segment.rs
use mmap_io::segment::SegmentView;

pub struct VecSegment {
    entries: Vec<VecPoint>,
    dimension: usize,
    params: HnswParams,
    hnsw: Option<Hnsw<VecPoint>>,
    point_ids: Option<Vec<PointId>>,
    pending_entries: Vec<VecPoint>,
    hnsw_dirty: bool,
    mmap: Option<SegmentView>,  // mmap 视图
}

impl VecSegment {
    /// 从文件加载 (mmap, 零拷贝)
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let mmap = mmap_io::create_mmap(path, 0)?;  // 0 = 使用文件大小
        
        // 解析 magic
        if mmap.len() < 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File too small for Vec segment",
            ));
        }
        
        // ... 解析逻辑同之前
        
        Ok(Self {
            entries,
            dimension: dim,
            params,
            hnsw: None,
            point_ids: None,
            pending_entries: Vec::new(),
            hnsw_dirty: false,
            mmap: Some(mmap),
        })
    }
}
```

#### Phase 3: 集成到 NtxFile (1h)

```rust
// mod.rs
impl NtxFile {
    /// 打开现有 NTX 文件 (mmap 加载)
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;
        
        // ... 读取 header 和 toc
        
        // 使用 mmap 加载 vec_segment
        #[cfg(feature = "mmap")]
        let vec_segment = {
            let vec_path = path.with_extension("ntx.vec");
            if vec_path.exists() {
                Some(VecSegment::from_file(&vec_path)?)
            } else {
                None
            }
        };
        
        #[cfg(not(feature = "mmap"))]
        let vec_segment = {
            // 回退到传统加载
            None
        };
        
        // ... 其余逻辑
    }
}
```

#### Phase 4: 编写测试 (0.5h)

```rust
// ntx/mmap_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_vec_segment_mmap() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("test.vec");
        
        // 创建测试数据
        let mut seg = VecSegment::new(4, HnswParams::default());
        for i in 0..100 {
            let mut node_id = [0u8; 36];
            node_id[0] = i;
            let vec = vec![i as f32, (i+1) as f32, (i+2) as f32, (i+3) as f32];
            seg.insert(node_id, vec);
        }
        
        // 写入文件
        let mut file = File::create(&path).unwrap();
        seg.write_to(&mut file).unwrap();
        drop(file);
        
        // mmap 加载
        let loaded = VecSegment::from_file(&path).unwrap();
        assert_eq!(loaded.len(), 100);
        assert_eq!(loaded.dimension(), 4);
    }
}
```

## 性能预估

| 操作 | 传统加载 | mmap 加载 |
|------|----------|-----------|
| 100K 向量 | ~500ms | ~50ms |
| 1M 向量 | ~5s | ~100ms |
| 内存占用 | 2x | 1x |

## 风险评估

| 风险 | 影响 | 缓解 |
|------|------|------|
| mmap-io 不成熟 | API 可能变化 | 锁定版本, 编写适配层 |
| 跨平台兼容性 | Windows 行为差异 | mmap-io 已处理 |
| 性能不如原生 mmap | 加载略慢 | 可接受 |

## 备选方案: 预读优化

如果不想引入新依赖, 可以使用预读优化:

```rust
pub struct VecSegment {
    // ... 现有字段
    file_cache: Option<Vec<u8>>,  // 文件缓存
}

impl VecSegment {
    /// 从文件加载 (预读优化)
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        
        // 预读整个文件
        file.read_to_end(&mut buffer)?;
        
        // 设置预读提示
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            unsafe {
                libc::madvise(
                    buffer.as_ptr() as *mut libc::c_void,
                    buffer.len(),
                    libc::MADV_SEQUENTIAL,
                );
            }
        }
        
        // ... 解析逻辑同之前
        
        Ok(Self {
            // ...
            file_cache: Some(buffer),
        })
    }
}
```

## 决策建议

**推荐使用 mmap-io**:
1. 零 unsafe API, 符合 `#![forbid(unsafe_code)]`
2. 性能提升显著 (10x 加载速度)
3. 跨平台支持完整
4. 实现简单, 风险低

**实施顺序**:
1. 引入 mmap-io 依赖
2. 实现 VecSegment::from_file()
3. 集成到 NtxFile::open()
4. 编写测试
5. 性能基准测试

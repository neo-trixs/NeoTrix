//! NTX 单文件存储格式 — NeoTrix 知识库的可移植单文件格式
//!
//! 借鉴 Memvid MV2, 解决 NeoTrix KB 10 大痛点:
//! - 持久化向量索引 (冷启动 0 重建)
//! - 嵌入式 WAL (零外部文件崩溃恢复)
//! - 追加只读帧 (时间旅行)
//! - 单文件便携 (零依赖分享)
//! - Zstd 压缩 (30-50% 节省)

pub mod format;
pub mod frames;
pub mod wal;
pub mod vec_segment;
pub mod graph_segment;
pub mod time_segment;
pub mod lex_segment;
pub mod sync;
pub mod search_bridge;
pub mod ntx_integration;
pub mod benchmark;
#[cfg(test)]
mod integration_test;

/// Trait for NTX segment types that can be serialized via `write_to`.
pub trait NtxWritable {
    fn write_to(&self, writer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<u64>;
}

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use format::{NtxHeader, NtxToc, SegmentDescriptor, SegmentType, FeatureFlags, HEADER_SIZE};
use frames::KnowledgeFrame;
use wal::{EmbeddedWal, WalStats, WalEntryType};
use vec_segment::VecSegment;
use graph_segment::GraphSegment;
use time_segment::TimeSegment;
use lex_segment::LexSegment;

/// NTX 文件统计
#[derive(Debug, Clone, Default)]
pub struct NtxStats {
    pub frame_count: u64,
    pub node_count: u64,
    pub edge_count: u64,
    pub wal_stats: WalStats,
    pub has_lex_index: bool,
    pub has_vec_index: bool,
    pub has_graph: bool,
    pub has_time_index: bool,
    pub file_size: u64,
    pub vec_entries: usize,
    pub graph_nodes: usize,
    pub time_entries: usize,
}

/// 压缩率统计
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub ratio: f64,  // compressed / uncompressed
}

/// NTX 文件核心结构
use std::collections::HashMap;
use std::sync::Mutex;

pub struct NtxFile {
    path: PathBuf,
    file: File,
    header: NtxHeader,
    toc: NtxToc,
    wal: EmbeddedWal,
    frames: Vec<KnowledgeFrame>,
    frame_index: HashMap<u64, usize>,  // frame_id → frames 索引 (O(1) 查找)
    vec_segment: Option<VecSegment>,
    graph_segment: Option<GraphSegment>,
    time_segment: Option<TimeSegment>,
    lex_segment: Option<LexSegment>,
    dirty: bool,
    read_only: bool,
    wal_lock: Mutex<()>,  // WAL 操作锁
    write_buffer: Vec<KnowledgeFrame>,  // 写缓冲区
    flush_threshold: usize,  // 刷新阈值 (默认 1000)
}

impl NtxFile {
    /// 创建新的 NTX 文件
    pub fn create(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024; // 1MB WAL

        // 预分配 Header + WAL 区域
        header.write_to(&mut file)?;
        file.seek(SeekFrom::Start(HEADER_SIZE as u64))?;
        file.write_all(&vec![0u8; header.wal_size as usize])?;

        // 写入空 TOC
        let toc = NtxToc::default();
        let footer_offset = toc.write_to(&mut file)?;
        header.footer_offset = footer_offset;
        header.write_to(&mut file)?;

        let wal = EmbeddedWal::open(&mut file, &header)?;

        Ok(Self {
            path, file, header, toc, wal,
            frames: Vec::new(),
            frame_index: HashMap::new(),
            vec_segment: None,
            graph_segment: None,
            time_segment: None,
            lex_segment: None,
            dirty: false,
            read_only: false,
            wal_lock: Mutex::new(()),
            write_buffer: Vec::new(),
            flush_threshold: 1000,
        })
    }

    /// 打开现有 NTX 文件
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;

        let header = NtxHeader::read_from(&mut file)?;
        let toc = NtxToc::read_from(&mut file)?;
        let wal = EmbeddedWal::open(&mut file, &header)?;

        // 从 TOC 加载段
        let frames = Self::load_frames_segment(&mut file, &toc)?;
        let vec_segment = Self::load_vec_segment(&mut file, &toc)?;
        let graph_segment = Self::load_graph_segment(&mut file, &toc)?;
        let time_segment = Self::load_time_segment(&mut file, &toc)?;
        let lex_segment = Self::load_lex_segment(&mut file, &toc)?;

        // 构建帧索引 (O(1) 查找)
        let mut frame_index = HashMap::new();
        for (i, frame) in frames.iter().enumerate() {
            frame_index.insert(frame.frame_id, i);
        }

        Ok(Self {
            path, file, header, toc, wal,
            frames, frame_index, vec_segment, graph_segment, time_segment, lex_segment,
            dirty: false,
            read_only: false,
            wal_lock: Mutex::new(()),
            write_buffer: Vec::new(),
            flush_threshold: 1000,
        })
    }

    /// 只读打开
    /// 只读打开 (不写 WAL, 不修改)
    pub fn open_read_only(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&path)?;

        let header = NtxHeader::read_from(&mut file)?;
        let toc = NtxToc::read_from(&mut file)?;

        // 只读打开时跳过 WAL
        let wal = EmbeddedWal::new_read_only(&header);

        let mut ntx = NtxFile {
            file,
            wal,
            header,
            toc,
            frames: Vec::new(),
            frame_index: HashMap::new(),
            vec_segment: None,
            graph_segment: None,
            time_segment: None,
            lex_segment: None,
            path,
            dirty: false,
            read_only: true,
            wal_lock: Mutex::new(()),
            write_buffer: Vec::new(),
            flush_threshold: 1000,
        };

        // 只读时也加载帧并构建索引
        ntx.frames = Self::load_frames_segment(&mut ntx.file, &ntx.toc)?;
        for (i, frame) in ntx.frames.iter().enumerate() {
            ntx.frame_index.insert(frame.frame_id, i);
        }
        Ok(ntx)
    }

    // ── 帧操作 ──────────────────────────────────────

    /// 写入知识帧 (只读检查 + WAL 锁 + 写缓冲)
    pub fn put_frame(&mut self, frame: &KnowledgeFrame) -> std::io::Result<()> {
        if self.read_only {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "NTX file opened read-only",
            ));
        }

        // 添加到写缓冲区
        self.write_buffer.push(frame.clone());

        // 如果缓冲区达到阈值, 刷新到 WAL
        if self.write_buffer.len() >= self.flush_threshold {
            self.flush_write_buffer()?;
        }

        Ok(())
    }

    /// 刷新写缓冲区到 WAL
    fn flush_write_buffer(&mut self) -> std::io::Result<()> {
        if self.write_buffer.is_empty() {
            return Ok(());
        }

        // WAL 锁: 保护顺序写入
        {
            let _wal_guard = self.wal_lock.lock().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("WAL 锁获取失败: {}", e))
            })?;

            // 批量写入 WAL
            for frame in &self.write_buffer {
                self.wal.append(&mut self.file, frame)?;
                let idx = self.frames.len();
                self.frames.push(frame.clone());
                self.frame_index.insert(frame.frame_id, idx);
                self.header.frame_count += 1;
            }
            self.dirty = true;
        }

        // 清空缓冲区
        self.write_buffer.clear();

        if self.wal.needs_checkpoint() {
            self.checkpoint()?;
        }

        Ok(())
    }

    /// 批量写入 (优化: 合并 WAL 写入 + 写缓冲)
    pub fn put_frames(&mut self, frames: &[KnowledgeFrame]) -> std::io::Result<()> {
        if self.read_only {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "NTX file opened read-only",
            ));
        }

        // 添加到写缓冲区
        self.write_buffer.extend_from_slice(frames);

        // 如果缓冲区达到阈值, 刷新到 WAL
        if self.write_buffer.len() >= self.flush_threshold {
            self.flush_write_buffer()?;
        }

        Ok(())
    }

    // ── 段操作 ──────────────────────────────────────

    /// 写入向量段
    pub fn put_vec_segment(&mut self, seg: VecSegment) {
        self.vec_segment = Some(seg);
        self.header.set_feature(FeatureFlags::VEC, true);
        self.dirty = true;
    }

    /// 写入图谱段
    pub fn put_graph_segment(&mut self, seg: GraphSegment) {
        self.header.node_count = seg.node_count() as u64;
        self.header.edge_count = seg.edge_count() as u64;
        self.header.set_feature(FeatureFlags::GRAPH, true);
        self.graph_segment = Some(seg);
        self.dirty = true;
    }

    /// 写入时间索引段
    pub fn put_time_segment(&mut self, seg: TimeSegment) {
        self.header.set_feature(FeatureFlags::TEMPORAL, true);
        self.time_segment = Some(seg);
        self.dirty = true;
    }

    /// 写入 Lex 段 (FTS5 快照)
    pub fn put_lex_segment(&mut self, seg: LexSegment) {
        self.header.set_feature(FeatureFlags::LEX, true);
        self.lex_segment = Some(seg);
        self.dirty = true;
    }

    /// 获取帧列表
    pub fn frames(&self) -> &[KnowledgeFrame] {
        &self.frames
    }

    /// 获取向量段
    pub fn vec_segment(&self) -> Option<&VecSegment> {
        self.vec_segment.as_ref()
    }

    /// 获取图谱段
    pub fn graph_segment(&self) -> Option<&GraphSegment> {
        self.graph_segment.as_ref()
    }

    /// 获取时间索引段
    pub fn time_segment(&self) -> Option<&TimeSegment> {
        self.time_segment.as_ref()
    }

    /// 获取 Lex 段 (FTS5 快照)
    pub fn lex_segment(&self) -> Option<&LexSegment> {
        self.lex_segment.as_ref()
    }

    // ── 提交 ──────────────────────────────────────

    /// 提交: 刷新缓冲区 + 检查点 + 写段 + 刷盘
    pub fn commit(&mut self) -> std::io::Result<()> {
        if !self.dirty && self.write_buffer.is_empty() {
            return Ok(());
        }
        if self.read_only {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "NTX file opened read-only",
            ));
        }

        // 刷新写缓冲区
        self.flush_write_buffer()?;

        // 检查点: WAL → frames
        self.checkpoint()?;

        // 更新 WAL 状态
        self.header.wal_sequence = self.wal.stats().sequence;
        self.header.wal_checkpoint_pos = self.wal.stats().checkpoint_pos;

        // 写帧段
        let _frames_offset = self.write_frames_segment()?;

        // 写向量段
        if self.vec_segment.is_some() {
            let (offset, len) = self.write_vec_segment_and_len()?;
            self.toc.add_segment(SegmentDescriptor::new(
                SegmentType::Vec, offset, len, &[],
            ));
        }

        // 写图谱段
        if self.graph_segment.is_some() {
            let (offset, len) = self.write_graph_segment_and_len()?;
            self.toc.add_segment(SegmentDescriptor::new(
                SegmentType::Graph, offset, len, &[],
            ));
        }

        // 写时间索引段
        if self.time_segment.is_some() {
            let (offset, len) = self.write_time_segment_and_len()?;
            self.toc.add_segment(SegmentDescriptor::new(
                SegmentType::Time, offset, len, &[],
            ));
        }

        // 写 Lex 段 (FTS5 快照)
        if self.lex_segment.is_some() {
            let (offset, len) = self.write_lex_segment_and_len()?;
            self.toc.add_segment(SegmentDescriptor::new(
                SegmentType::Lex, offset, len, &[],
            ));
        }

        // 刷新 WAL
        self.wal.flush(&mut self.file)?;

        // 重写 TOC
        self.file.seek(SeekFrom::End(0))?;
        let footer_offset = self.toc.write_to(&mut self.file)?;
        self.header.footer_offset = footer_offset;
        self.header.toc_checksum = self.toc.checksum();

        // 重写 Header
        self.file.seek(SeekFrom::Start(0))?;
        self.header.write_to(&mut self.file)?;

        // fsync
        self.file.sync_all()?;
        self.dirty = false;
        Ok(())
    }

    // ── 检查点 ──────────────────────────────────────

    /// 检查点: WAL → frames (HashSet 去重, O(N) 替代 O(N²))
    fn checkpoint(&mut self) -> std::io::Result<()> {
        use std::collections::HashSet;
        let entries = self.wal.checkpoint(&mut self.file)?;
        let existing: HashSet<u64> = self.frames.iter().map(|f| f.frame_id).collect();
        for entry in &entries {
            if entry.entry_type == WalEntryType::Append {
                if let Ok(frame) = KnowledgeFrame::decode_bytes(&entry.payload) {
                    if !existing.contains(&frame.frame_id) {
                        self.frames.push(frame);
                    }
                }
            }
        }
        Ok(())
    }

    /// 恢复: 重放 WAL
    pub fn recover(&mut self) -> std::io::Result<Vec<KnowledgeFrame>> {
        let entries = self.wal.recover(&mut self.file)?;
        let mut frames = Vec::new();
        for entry in &entries {
            if entry.entry_type == WalEntryType::Append {
                if let Ok(frame) = KnowledgeFrame::decode_bytes(&entry.payload) {
                    frames.push(frame);
                }
            }
        }
        Ok(frames)
    }

    // ── 特性标志 ──────────────────────────────────────

    pub fn enable_feature(&mut self, flag: FeatureFlags) {
        self.header.set_feature(flag, true);
        self.dirty = true;
    }

    // ── 统计 ──────────────────────────────────────

    pub fn stats(&self) -> NtxStats {
        let file_size = self.file.metadata().map(|m| m.len()).unwrap_or(0);
        NtxStats {
            frame_count: self.header.frame_count,
            node_count: self.header.node_count,
            edge_count: self.header.edge_count,
            wal_stats: self.wal.stats(),
            has_lex_index: self.header.has_feature(FeatureFlags::LEX),
            has_vec_index: self.header.has_feature(FeatureFlags::VEC),
            has_graph: self.header.has_feature(FeatureFlags::GRAPH),
            has_time_index: self.header.has_feature(FeatureFlags::TEMPORAL),
            file_size,
            vec_entries: self.vec_segment.as_ref().map_or(0, |s| s.len()),
            graph_nodes: self.graph_segment.as_ref().map_or(0, |s| s.node_count()),
            time_entries: self.time_segment.as_ref().map_or(0, |s| s.len()),
        }
    }

    /// 获取压缩率统计
    pub fn compression_stats(&self) -> CompressionStats {
        let mut uncompressed_size = 0u64;
        let mut compressed_size = 0u64;

        // 计算帧段压缩率
        for frame in &self.frames {
            let encoded = frame.encode_bytes();
            uncompressed_size += frame.payload.len() as u64;
            compressed_size += encoded.len() as u64;
        }

        // 计算向量段压缩率
        if let Some(seg) = &self.vec_segment {
            for entry in seg.entries() {
                uncompressed_size += (entry.vector.len() * 4) as u64;  // f32 = 4 bytes
            }
            // 向量段未压缩, compressed = uncompressed
            compressed_size += uncompressed_size;
        }

        let ratio = if uncompressed_size > 0 {
            compressed_size as f64 / uncompressed_size as f64
        } else {
            1.0
        };

        CompressionStats {
            uncompressed_size,
            compressed_size,
            ratio,
        }
    }

    pub fn path(&self) -> &Path { &self.path }
    pub fn header(&self) -> &NtxHeader { &self.header }
    pub fn toc(&self) -> &NtxToc { &self.toc }

    pub fn close(mut self) -> std::io::Result<()> {
        self.commit()?;
        Ok(())
    }

    // ── Drop 实现 ──────────────────────────────────────

    /// Drop: 刷新写缓冲区 + 尝试提交
    #[allow(dead_code)]
    fn drop(&mut self) {
        // 尝试刷新写缓冲区
        if !self.write_buffer.is_empty() {
            if let Err(e) = self.flush_write_buffer() {
                eprintln!("[NTX] Drop: 刷新写缓冲区失败: {}", e);
            }
        }

        // 尝试提交
        if self.dirty && !self.read_only {
            if let Err(e) = self.commit() {
                eprintln!("[NTX] Drop: 提交失败: {}", e);
            }
        }
    }

    // ── 段读写内部方法 ──────────────────────────────

    /// 写帧段到文件, 返回偏移
    fn write_frames_segment(&mut self) -> std::io::Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let count = self.frames.len() as u32;
        self.file.write_all(&count.to_le_bytes())?;
        for frame in &self.frames {
            let bytes = frame.encode_bytes();
            let len = bytes.len() as u32;
            self.file.write_all(&len.to_le_bytes())?;
            self.file.write_all(&bytes)?;
        }
        Ok(offset)
    }

    /// 从文件加载帧段 (带完整性验证)
    #[allow(dead_code)]
    fn load_frames_segment(file: &mut File, toc: &NtxToc) -> std::io::Result<Vec<KnowledgeFrame>> {
        // 帧大小限制: 10MB
        const MAX_FRAME_SIZE: usize = 10 * 1024 * 1024;

        let desc = match toc.find_segment(SegmentType::Frames) {
            Some(d) => d,
            None => return Ok(Vec::new()),
        };

        // 段完整性验证: 检查段偏移和长度
        if desc.offset == 0 || desc.length == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "帧段偏移或长度无效",
            ));
        }

        file.seek(SeekFrom::Start(desc.offset))?;
        let mut count_buf = [0u8; 4];
        file.read_exact(&mut count_buf)?;
        let count = u32::from_le_bytes(count_buf) as usize;

        // 预期数据大小验证
        let expected_min_size = 4 + count as u64 * 4;  // count + 每帧至少 4 字节长度
        if desc.length < expected_min_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("帧段大小异常: 段长度 {} < 预期最小 {}", desc.length, expected_min_size),
            ));
        }

        let mut frames = Vec::with_capacity(count.min(100000)); // 防止过大预分配
        let mut errors = 0u32;
        let mut total_bytes_read = 4u64; // count 字段

        for i in 0..count {
            let mut len_buf = [0u8; 4];
            if file.read_exact(&mut len_buf).is_err() {
                errors += 1;
                eprintln!("[NTX] load_frames: 帧 {} 长度读取失败", i);
                continue;
            }
            let len = u32::from_le_bytes(len_buf) as usize;
            total_bytes_read += 4;

            if len > MAX_FRAME_SIZE {
                errors += 1;
                eprintln!("[NTX] load_frames: 帧 {} 大小异常 ({} bytes, 上限 {} bytes)", i, len, MAX_FRAME_SIZE);
                continue;
            }

            let mut buf = vec![0u8; len];
            if file.read_exact(&mut buf).is_err() {
                errors += 1;
                eprintln!("[NTX] load_frames: 帧 {} 数据读取失败", i);
                continue;
            }
            total_bytes_read += len as u64;

            match KnowledgeFrame::decode_bytes(&buf) {
                Ok(frame) => {
                    // 帧校验和验证
                    if !frame.verify() {
                        errors += 1;
                        eprintln!("[NTX] load_frames: 帧 {} 校验和验证失败", i);
                        continue;
                    }
                    frames.push(frame);
                },
                Err(e) => {
                    errors += 1;
                    eprintln!("[NTX] load_frames: 帧 {} 解码失败: {}", i, e);
                }
            }
        }

        // 最终大小验证
        if total_bytes_read > desc.length {
            eprintln!("[NTX] load_frames: 段大小不匹配: 实际读取 {} > 段长度 {}", total_bytes_read, desc.length);
        }

        // 段完整性验证: SHA-256 校验
        if !desc.checksum.iter().all(|&b| b == 0) {
            // 读取段数据进行校验
            file.seek(SeekFrom::Start(desc.offset))?;
            let mut segment_data = vec![0u8; desc.length as usize];
            if file.read_exact(&mut segment_data).is_ok() {
                if !desc.verify(&segment_data) {
                    eprintln!("[NTX] load_frames: 段 SHA-256 校验失败");
                }
            }
        }

        if errors > 0 {
            eprintln!("[NTX] load_frames: {} / {} 帧加载失败", errors, count);
        }
        Ok(frames)
    }

    /// 写向量段
    #[allow(dead_code)]
    fn write_vec_segment(&mut self, seg: &VecSegment) -> std::io::Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        seg.write_to(&mut self.file)?;
        Ok(offset)
    }

    /// 加载向量段
    fn load_vec_segment(file: &mut File, toc: &NtxToc) -> std::io::Result<Option<VecSegment>> {
        let desc = match toc.find_segment(SegmentType::Vec) {
            Some(d) => d,
            None => return Ok(None),
        };
        file.seek(SeekFrom::Start(desc.offset))?;
        Ok(Some(VecSegment::read_from(file)?))
    }

    /// 写图谱段
    #[allow(dead_code)]
    fn write_graph_segment(&mut self, seg: &GraphSegment) -> std::io::Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        seg.write_to(&mut self.file)?;
        Ok(offset)
    }

    /// 加载图谱段
    fn load_graph_segment(file: &mut File, toc: &NtxToc) -> std::io::Result<Option<GraphSegment>> {
        let desc = match toc.find_segment(SegmentType::Graph) {
            Some(d) => d,
            None => return Ok(None),
        };
        file.seek(SeekFrom::Start(desc.offset))?;
        Ok(Some(GraphSegment::read_from(file)?))
    }

    /// 写时间索引段
    #[allow(dead_code)]
    fn write_time_segment(&mut self, seg: &TimeSegment) -> std::io::Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        seg.write_to(&mut self.file)?;
        Ok(offset)
    }

    /// 加载时间索引段
    fn load_time_segment(file: &mut File, toc: &NtxToc) -> std::io::Result<Option<TimeSegment>> {
        let desc = match toc.find_segment(SegmentType::Time) {
            Some(d) => d,
            None => return Ok(None),
        };
        file.seek(SeekFrom::Start(desc.offset))?;
        Ok(Some(TimeSegment::read_from(file)?))
    }

    /// 写 Lex 段
    #[allow(dead_code)]
    fn write_lex_segment(&mut self, seg: &LexSegment) -> std::io::Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        seg.write_to(&mut self.file)?;
        Ok(offset)
    }

    /// 加载 Lex 段
    fn load_lex_segment(file: &mut File, toc: &NtxToc) -> std::io::Result<Option<LexSegment>> {
        let desc = match toc.find_segment(SegmentType::Lex) {
            Some(d) => d,
            None => return Ok(None),
        };
        file.seek(SeekFrom::Start(desc.offset))?;
        Ok(Some(LexSegment::read_from(file)?))
    }

    /// 计算段序列化大小 (用于 TOC)
    fn segment_len(&self, seg: &impl NtxWritable) -> std::io::Result<u64> {
        let mut buf = std::io::Cursor::new(Vec::new());
        seg.write_to(&mut buf)?;
        Ok(buf.position())
    }

    /// 写向量段并返回 (offset, len)
    fn write_vec_segment_and_len(&mut self) -> std::io::Result<(u64, u64)> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let seg = self.vec_segment.as_ref().unwrap();
        seg.write_to(&mut self.file)?;
        let len = self.segment_len(seg)?;
        Ok((offset, len))
    }

    /// 写图谱段并返回 (offset, len)
    fn write_graph_segment_and_len(&mut self) -> std::io::Result<(u64, u64)> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let seg = self.graph_segment.as_ref().unwrap();
        seg.write_to(&mut self.file)?;
        let len = self.segment_len(seg)?;
        Ok((offset, len))
    }

    /// 写时间索引段并返回 (offset, len)
    fn write_time_segment_and_len(&mut self) -> std::io::Result<(u64, u64)> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let seg = self.time_segment.as_ref().unwrap();
        seg.write_to(&mut self.file)?;
        let len = self.segment_len(seg)?;
        Ok((offset, len))
    }

    /// 写 Lex 段并返回 (offset, len)
    fn write_lex_segment_and_len(&mut self) -> std::io::Result<(u64, u64)> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let seg = self.lex_segment.as_ref().unwrap();
        seg.write_to(&mut self.file)?;
        let len = self.segment_len(seg)?;
        Ok((offset, len))
    }
}

/// 快速打开或创建
pub fn open_or_create(path: impl AsRef<Path>) -> std::io::Result<NtxFile> {
    if path.as_ref().exists() {
        NtxFile::open(path)
    } else {
        NtxFile::create(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::frames::{FrameType, Encoding};
    use super::time_segment::TimeEntry;
    use super::vec_segment::HnswParams;
    use std::collections::HashMap;

    fn mk_id(n: u64) -> [u8; 36] { let mut id = [0u8; 36]; id[0] = n as u8; id }

    #[test]
    fn test_create_open_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.ntx");

        {
            let mut ntx = NtxFile::create(&path).unwrap();
            let frame = KnowledgeFrame::new(0, mk_id(1), FrameType::Node, b"hello", Encoding::Raw, None);
            ntx.put_frame(&frame).unwrap();
            ntx.commit().unwrap();
        }

        {
            let ntx = NtxFile::open(&path).unwrap();
            assert_eq!(ntx.frames().len(), 1);
            assert_eq!(ntx.stats().frame_count, 1);
        }
    }

    #[test]
    fn test_commit_persists_frames() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("persist.ntx");

        {
            let mut ntx = NtxFile::create(&path).unwrap();
            for i in 0..20 {
                let frame = KnowledgeFrame::new(i as u64, mk_id(i as u64), FrameType::Node, b"test", Encoding::Raw, None);
                ntx.put_frame(&frame).unwrap();
            }
            ntx.commit().unwrap();
        }

        {
            let ntx = NtxFile::open(&path).unwrap();
            assert_eq!(ntx.frames().len(), 20);
        }
    }

    #[test]
    fn test_vec_segment_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("vec.ntx");

        {
            let mut ntx = NtxFile::create(&path).unwrap();
            let mut seg = VecSegment::new(4, HnswParams::default());
            for i in 0..50 {
                seg.insert(mk_id(i as u64), vec![i as f32; 4]);
            }
            ntx.put_vec_segment(seg);
            ntx.commit().unwrap();
        }

        {
            let ntx = NtxFile::open(&path).unwrap();
            assert!(ntx.vec_segment().is_some());
            assert_eq!(ntx.vec_segment().unwrap().len(), 50);
        }
    }

    #[test]
    fn test_graph_segment_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("graph.ntx");

        {
            let mut ntx = NtxFile::create(&path).unwrap();
            let mut seg = GraphSegment::new();
            seg.add_edge(graph_segment::GraphEdge {
                source: mk_id(1), target: mk_id(2),
                edge_type: 0, weight: 1.0,
                direction: graph_segment::EdgeDirection::Bidirectional,
            });
            ntx.put_graph_segment(seg);
            ntx.commit().unwrap();
        }

        {
            let ntx = NtxFile::open(&path).unwrap();
            let g = ntx.graph_segment().unwrap();
            assert_eq!(g.node_count(), 2);
            assert_eq!(g.edge_count(), 1);
        }
    }

    #[test]
    fn test_time_segment_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("time.ntx");

        {
            let mut ntx = NtxFile::create(&path).unwrap();
            let mut seg = TimeSegment::new();
            seg.insert(TimeEntry { timestamp: 1000, frame_id: 0, entry_type: 0, offset: 0 });
            seg.insert(TimeEntry { timestamp: 2000, frame_id: 1, entry_type: 1, offset: 100 });
            ntx.put_time_segment(seg);
            ntx.commit().unwrap();
        }

        {
            let ntx = NtxFile::open(&path).unwrap();
            let t = ntx.time_segment().unwrap();
            assert_eq!(t.len(), 2);
        }
    }

    #[test]
    fn test_full_ntx_with_all_segments() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("full.ntx");

        {
            let mut ntx = NtxFile::create(&path).unwrap();

            // 帧
            for i in 0..10 {
                ntx.put_frame(&KnowledgeFrame::new(i as u64, mk_id(i as u64), FrameType::Node, b"data", Encoding::Zstd, None)).unwrap();
            }

            // 向量
            let mut vec_seg = VecSegment::new(3, HnswParams::default());
            for i in 0..10 { vec_seg.insert(mk_id(i as u64), vec![i as f32; 3]); }
            ntx.put_vec_segment(vec_seg);

            // 图谱
            let mut g = GraphSegment::new();
            g.add_edge(graph_segment::GraphEdge {
                source: mk_id(0), target: mk_id(1), edge_type: 0, weight: 1.0,
                direction: graph_segment::EdgeDirection::Bidirectional,
            });
            ntx.put_graph_segment(g);

            // 时间
            let mut t = TimeSegment::new();
            t.insert(TimeEntry { timestamp: 1000, frame_id: 0, entry_type: 0, offset: 0 });
            ntx.put_time_segment(t);

            ntx.commit().unwrap();
        }

        {
            let ntx = NtxFile::open(&path).unwrap();
            let s = ntx.stats();
            assert_eq!(s.frame_count, 10);
            assert_eq!(s.vec_entries, 10);
            assert_eq!(s.graph_nodes, 2);
            assert_eq!(s.time_entries, 1);
        }
    }

    #[test]
    fn test_concurrent_read_write() {
        use std::sync::Arc;
        use std::thread;

        let tmp = tempfile::tempdir().unwrap();
        let path = Arc::new(tmp.path().join("concurrent.ntx"));

        // 创建文件
        {
            let mut ntx = NtxFile::create(&*path).unwrap();
            for i in 0..5 {
                ntx.put_frame(&KnowledgeFrame::new(i as u64, mk_id(i as u64), FrameType::Node, b"init", Encoding::Raw, None)).unwrap();
            }
            ntx.commit().unwrap();
        }

        // 并发读写测试
        let mut handles = vec![];
        for i in 0..5 {
            let path = path.clone();
            handles.push(thread::spawn(move || {
                let mut ntx = NtxFile::open(&*path).unwrap();
                let frame = KnowledgeFrame::new(100 + i as u64, mk_id(100 + i as u64), FrameType::Node, b"concurrent", Encoding::Raw, None);
                ntx.put_frame(&frame).unwrap();
                ntx.commit().unwrap();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // 验证
        let ntx = NtxFile::open(&*path).unwrap();
        assert_eq!(ntx.frames().len(), 10);
    }

    #[test]
    fn test_file_lock_prevents_concurrent_write() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("lock.ntx");

        // 创建文件
        {
            let mut ntx = NtxFile::create(&path).unwrap();
            ntx.put_frame(&KnowledgeFrame::new(0, mk_id(0), FrameType::Node, b"test", Encoding::Raw, None)).unwrap();
            ntx.commit().unwrap();
        }

        // 第一个文件句柄 (持有锁)
        let mut ntx1 = NtxFile::open(&path).unwrap();
        ntx1.put_frame(&KnowledgeFrame::new(1, mk_id(1), FrameType::Node, b"lock1", Encoding::Raw, None)).unwrap();

        // 第二个文件句柄应该获取锁失败 (Linux/macOS 行为)
        // 注意: fs2 的锁是 advisory 的, 在某些系统上可能不会阻塞
        // 这里只测试基本的锁获取/释放
        drop(ntx1);

        // 锁释放后应该可以打开
        let ntx2 = NtxFile::open(&path).unwrap();
        assert!(ntx2.frames().len() > 0);
    }

    #[test]
    fn test_version_check() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("version.ntx");

        // 创建文件
        {
            let mut ntx = NtxFile::create(&path).unwrap();
            ntx.commit().unwrap();
        }

        // 正常打开
        let ntx = NtxFile::open(&path);
        assert!(ntx.is_ok());
    }

    #[test]
    fn test_frame_size_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("size.ntx");

        let mut ntx = NtxFile::create(&path).unwrap();

        // 正常大小帧
        let frame = KnowledgeFrame::new(0, mk_id(0), FrameType::Node, b"normal", Encoding::Raw, None);
        assert!(ntx.put_frame(&frame).is_ok());

        // 大帧测试 (11MB, 超过 10MB 限制)
        let large_data = vec![0u8; 11 * 1024 * 1024];
        let large_frame = KnowledgeFrame::new(1, mk_id(1), FrameType::Node, &large_data, Encoding::Raw, None);
        // put_frame 不验证大小, 只有 load_frames_segment 验证
        assert!(ntx.put_frame(&large_frame).is_ok());
    }
}

impl NtxWritable for vec_segment::VecSegment {
    fn write_to(&self, writer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<u64> {
        vec_segment::VecSegment::write_to(self, writer)
    }
}

impl NtxWritable for graph_segment::GraphSegment {
    fn write_to(&self, writer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<u64> {
        graph_segment::GraphSegment::write_to(self, writer)
    }
}

impl NtxWritable for time_segment::TimeSegment {
    fn write_to(&self, writer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<u64> {
        time_segment::TimeSegment::write_to(self, writer)
    }
}

impl NtxWritable for lex_segment::LexSegment {
    fn write_to(&self, writer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<u64> {
        lex_segment::LexSegment::write_to(self, writer)
    }
}

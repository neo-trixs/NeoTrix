use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::nt_core_consciousness::memory_lattice::MemoryLattice;

/// MemoryArchiver — 自归档引擎。
///
/// 模拟人类记忆的冷却曲线：
/// - SpreadingActivation < 0.1 的 episodic 记忆 → 序列化到 `docs/sessions/archive/`
/// - 从热存储中删除已归档条目
/// - 归档文件名按时间分片：`YYYY-MM-DD-HHMMSS-hash.md`
///
/// 这完成了 AGENTS.md 历史日志块从手动维护 → 自动"遗忘"到冷存储的迁移。
pub struct MemoryArchiver {
    /// 归档根目录
    pub archive_root: PathBuf,
    /// 归档间隔（cycle 数）
    pub interval: u64,
    /// 上次归档的 cycle
    pub last_archived: u64,
    /// 归档次数
    pub archive_count: u64,
    /// 激活阈值 — 低于此值的 episodic 可归档
    pub activation_threshold: f64,
    /// 最大每批归档数
    pub batch_size: usize,
    /// 是否启用
    pub enabled: bool,
}

impl Default for MemoryArchiver {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryArchiver {
    pub fn new() -> Self {
        Self {
            archive_root: PathBuf::from("docs/sessions/archive"),
            interval: 100,
            last_archived: 0,
            archive_count: 0,
            activation_threshold: 0.1,
            batch_size: 20,
            enabled: true,
        }
    }

    pub fn with_interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_archive_root(mut self, path: PathBuf) -> Self {
        self.archive_root = path;
        self
    }

    pub fn with_activation_threshold(mut self, threshold: f64) -> Self {
        self.activation_threshold = threshold;
        self
    }

    /// 从 MemoryLattice 的 episodic 层归档冷记忆。
    ///
    /// 1. 检查是否该运行
    /// 2. 从 lattice.episodic 找到低激活条目
    /// 3. 序列化到 archive_root
    /// 4. 从热存储移除
    /// 返回归档的条目数
    pub fn archive_cold_memories(
        &mut self,
        cycle: u64,
        lattice: &mut MemoryLattice,
    ) -> usize {
        if !self.enabled || cycle < self.last_archived + self.interval {
            return 0;
        }
        self.last_archived = cycle;

        let mut candidates: Vec<(usize, f64)> = Vec::new();

        for (i, entry) in lattice.episodic.iter().enumerate() {
            if entry.confidence > 0.3 {
                continue;
            }
            // 用 confidence * 0.5 作为激活水平的代理
            let activation = entry.confidence * 0.5;
            if activation < self.activation_threshold {
                candidates.push((i, activation));
            }
        }

        if candidates.is_empty() {
            return 0;
        }

        // 按激活水平升序排序（最冷的优先归档）
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let to_archive: Vec<_> = candidates
            .into_iter()
            .take(self.batch_size)
            .collect();

        // 确保归档目录存在
        let _ = fs::create_dir_all(&self.archive_root);

        let archived_count = to_archive.len();
        let mut archived_ids: Vec<usize> = Vec::with_capacity(archived_count);

        for (idx, activation) in &to_archive {
            if let Some(entry) = lattice.episodic.get(*idx) {
                // 生成归档文件名: YYYY-MM-DD-HHMMSS-hash.md
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let hash = simple_hash(&entry.content);
                let date = chrono_format(timestamp);
                let filename = format!("{}-{:x}.md", date, hash);
                let filepath = self.archive_root.join(&filename);

                // 序列化为 markdown
                let content = format!(
                    "---\narchived_at: {}\ncycle: {}\nactivation: {:.4}\nconfidence: {:.4}\nlayer: {}\n---\n\n{}",
                    date,
                    cycle,
                    activation,
                    entry.confidence,
                    entry.layer.name(),
                    entry.content,
                );

                match fs::File::create(&filepath) {
                    Ok(mut f) => {
                        let _ = f.write_all(content.as_bytes());
                        log::info!(
                            "memory_archiver: archived cycle={} act={:.4} to {}",
                            cycle, activation, filename
                        );
                    }
                    Err(e) => {
                        log::warn!("memory_archiver: write failed for {}: {}", filename, e);
                        continue;
                    }
                }
            }
            archived_ids.push(*idx);
        }

        // 从热存储移除（逆序以保持索引正确）
        archived_ids.sort_unstable_by(|a, b| b.cmp(a));
        for idx in archived_ids {
            lattice.episodic.remove(idx);
        }

        self.archive_count += archived_count as u64;
        log::info!(
            "memory_archiver: archived {} cold memories (batch), total_archived={}",
            archived_count,
            self.archive_count,
        );

        archived_count
    }

    /// 检查是否需要归档
    pub fn should_archive(&self, cycle: u64) -> bool {
        self.enabled && cycle > 0 && cycle >= self.last_archived + self.interval
    }

    /// 统计报告
    pub fn stats(&self) -> String {
        format!(
            "memory_archiver: archived={} last_cycle={} interval={} threshold={:.2} batch={} enabled={}",
            self.archive_count,
            self.last_archived,
            self.interval,
            self.activation_threshold,
            self.batch_size,
            self.enabled,
        )
    }
}

/// 简单的字符串哈希（非加密，仅用于文件名去重）
fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

/// 格式化时间戳为 YYYY-MM-DD-HHMMSS
fn chrono_format(ts: u64) -> String {
    // 粗略的秒→日期转换（够用即可）
    let days = ts / 86400;
    let remaining = ts % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // 从 1970-01-01 开始计算年/月/日
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let (m, day) = month_day(d as u64 + 1, is_leap(y));

    format!("{:04}-{:02}-{:02}-{:02}{:02}{:02}", y, m, day, hours, minutes, seconds)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn month_day(day_of_year: u64, leap: bool) -> (u32, u32) {
    let month_days = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut remaining = day_of_year;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining <= md as u64 {
            return (i as u32 + 1, remaining as u32);
        }
        remaining -= md as u64;
    }
    (12, 31)
}



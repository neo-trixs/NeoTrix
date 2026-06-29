use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::{Record, SegmentReader, SegmentWriter, SegmentType, RT_VSA_VECTOR, VSA_BYTES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionLevel {
    Minor,
    Major,
}

#[derive(Debug, Clone)]
pub struct CompactionStats {
    pub segments_before: usize,
    pub segments_after: usize,
    pub records_before: usize,
    pub records_after: usize,
    pub tombstones_removed: usize,
    pub level: CompactionLevel,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct SegmentInfo {
    pub path: PathBuf,
    pub segment_id: u64,
    pub record_count: usize,
    pub tombstone_count: usize,
    pub live_bytes: u64,
    pub total_bytes: u64,
    pub tombstone_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct CompactionScheduler {
    pub minor_ratio: f64,
    pub major_ratio: f64,
    pub min_segments: usize,
    pub last_compaction: Option<CompactionStats>,
}

impl CompactionScheduler {
    pub fn new(minor: f64, major: f64, min_seg: usize) -> Self {
        Self { minor_ratio: minor, major_ratio: major, min_segments: min_seg, last_compaction: None }
    }

    pub fn should_compact(&self, segments: &[SegmentInfo]) -> Option<CompactionLevel> {
        if segments.is_empty() { return None; }
        let total_records: usize = segments.iter().map(|s| s.record_count).sum();
        let total_tombstones: usize = segments.iter().map(|s| s.tombstone_count).sum();
        if total_records == 0 { return None; }
        let ratio = total_tombstones as f64 / total_records as f64;
        if ratio > self.major_ratio { return Some(CompactionLevel::Major); }
        if ratio > self.minor_ratio && segments.len() >= self.min_segments { return Some(CompactionLevel::Minor); }
        None
    }
}

fn segment_id_from_path(path: &Path) -> u64 {
    path.file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| u64::from_str_radix(s, 16).ok())
        .unwrap_or(0)
}

pub fn analyze_segment(path: &Path) -> io::Result<SegmentInfo> {
    let reader = SegmentReader::open(path)?;
    let records = reader.records();
    let record_count = records.len();
    let tombstone_count = records.iter().filter(|r| r.tombstone).count();
    let total_bytes = std::fs::metadata(path)?.len();
    let live_bytes: u64 = records.iter().filter(|r| !r.tombstone).map(|r| r.encode().len() as u64 + 4).sum();
    let segment_id = segment_id_from_path(path);
    let tombstone_ratio = if record_count > 0 { tombstone_count as f64 / record_count as f64 } else { 0.0 };
    Ok(SegmentInfo { path: path.to_path_buf(), segment_id, record_count, tombstone_count, live_bytes, total_bytes, tombstone_ratio })
}

pub fn compact_minor(paths: &[PathBuf], output_dir: &Path) -> io::Result<CompactionStats> {
    let start = Instant::now();
    let segments_before = paths.len();
    let mut all_records = Vec::new();
    let mut records_before = 0;
    for p in paths {
        let reader = SegmentReader::open(p)?;
        let recs = reader.records();
        records_before += recs.len();
        all_records.extend(recs);
    }
    let tombstones_removed = all_records.iter().filter(|r| r.tombstone).count();
    all_records.retain(|r| !r.tombstone);
    let mut dedup: HashMap<String, Record> = HashMap::new();
    for r in all_records {
        dedup.entry(r.key.clone()).and_modify(|existing| { if r.timestamp > existing.timestamp { *existing = r.clone(); } }).or_insert(r);
    }
    let records_after = dedup.len();
    let max_id = paths.iter().map(|p| segment_id_from_path(p)).max().unwrap_or(0);
    let output_path = output_dir.join(format!("{:016x}.nts", max_id + 1));
    let mut writer = SegmentWriter::create(&output_path, SegmentType::Data)?;
    for record in dedup.values() { writer.append(record)?; }
    writer.finalize()?;
    for p in paths { let _ = std::fs::remove_file(p); }
    Ok(CompactionStats { segments_before, segments_after: 1, records_before, records_after, tombstones_removed, level: CompactionLevel::Minor, duration_ms: start.elapsed().as_millis() as u64 })
}

pub fn compact_major(paths: &[PathBuf], output_dir: &Path) -> io::Result<CompactionStats> {
    let start = Instant::now();
    let segments_before = paths.len();
    let mut all_records = Vec::new();
    let mut records_before = 0;
    for p in paths {
        let reader = SegmentReader::open(p)?;
        let recs = reader.records();
        records_before += recs.len();
        all_records.extend(recs);
    }
    let tombstones_removed = all_records.iter().filter(|r| r.tombstone).count();
    all_records.retain(|r| !r.tombstone);
    let mut dedup: HashMap<String, Record> = HashMap::new();
    for r in all_records {
        dedup.entry(r.key.clone()).and_modify(|existing| { if r.timestamp > existing.timestamp { *existing = r.clone(); } }).or_insert(r);
    }
    let records_after = dedup.len();
    let max_id = paths.iter().map(|p| segment_id_from_path(p)).max().unwrap_or(0);
    let output_path = output_dir.join(format!("{:016x}.nts", max_id + 1));
    let mut writer = SegmentWriter::create(&output_path, SegmentType::Data)?;
    for record in dedup.values() { writer.append(record)?; }
    writer.finalize()?;
    let index_path = output_dir.join(format!("{:016x}.idx", max_id + 1));
    let new_seg = analyze_segment(&output_path)?;
    rebuild_vsa_index(&[new_seg], &index_path)?;
    for p in paths { let _ = std::fs::remove_file(p); }
    Ok(CompactionStats { segments_before, segments_after: 1, records_before, records_after, tombstones_removed, level: CompactionLevel::Major, duration_ms: start.elapsed().as_millis() as u64 })
}

pub fn rebuild_vsa_index(segments: &[SegmentInfo], output_path: &Path) -> io::Result<usize> {
    let mut count = 0;
    let mut writer = SegmentWriter::create(output_path, SegmentType::Index)?;
    for seg in segments {
        let reader = SegmentReader::open(&seg.path)?;
        for rec in reader.records() {
            if rec.record_type == RT_VSA_VECTOR && !rec.tombstone && rec.data.len() == VSA_BYTES {
                writer.append(&rec)?;
                count += 1;
            }
        }
    }
    writer.finalize()?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Record, SegmentWriter, SegmentType, VsaTag, RT_KNOWLEDGE_NODE, RT_VSA_VECTOR};

    fn test_dir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("nt_segstore_compaction_{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn make_tombstone(key: &str) -> Record { Record::tombstone(VsaTag::WorldKnowledge, RT_KNOWLEDGE_NODE, key) }
    fn make_live_record(key: &str, data: &[u8]) -> Record { Record::new(VsaTag::SelfThought, RT_KNOWLEDGE_NODE, key, data.to_vec()) }
    fn _make_vsa_record(key: &str, data: &[u8]) -> Record { Record::new(VsaTag::SelfMemory, RT_VSA_VECTOR, key, data.to_vec()) }

    fn make_test_segment(dir: &Path, records: Vec<Record>, name: &str) -> PathBuf {
        let path = dir.join(name);
        let mut writer = SegmentWriter::create(&path, SegmentType::Data).unwrap();
        for r in &records { writer.append(r).unwrap(); }
        writer.finalize().unwrap();
        path
    }

    #[test]
    fn test_scheduler_defaults() {
        let s = CompactionScheduler::new(0.30, 0.60, 3);
        assert!((s.minor_ratio - 0.30).abs() < 1e-9);
        assert!((s.major_ratio - 0.60).abs() < 1e-9);
        assert_eq!(s.min_segments, 3);
    }

    #[test]
    fn test_should_compact_minor() {
        let s = CompactionScheduler::new(0.30, 0.60, 3);
        let dir = test_dir("minor");
        let seg1 = make_test_segment(&dir, vec![make_live_record("a", b"a1"), make_tombstone("x")], "0000000000000001.nts");
        let seg2 = make_test_segment(&dir, vec![make_live_record("b", b"b1"), make_tombstone("y")], "0000000000000002.nts");
        let seg3 = make_test_segment(&dir, vec![make_live_record("c", b"c1"), make_tombstone("z")], "0000000000000003.nts");
        let infos: Vec<SegmentInfo> = vec![&seg1, &seg2, &seg3].into_iter().map(|p| analyze_segment(p).unwrap()).collect();
        assert_eq!(s.should_compact(&infos), Some(CompactionLevel::Minor));
    }

    #[test]
    fn test_compact_minor_removes_tombstones() {
        let dir = test_dir("compact_remove");
        let s1 = make_test_segment(&dir, vec![make_live_record("a", b"a1"), make_tombstone("x")], "0000000000000001.nts");
        let s2 = make_test_segment(&dir, vec![make_live_record("b", b"b1"), make_tombstone("y")], "0000000000000002.nts");
        let stats = compact_minor(&[s1, s2], &dir).unwrap();
        assert_eq!(stats.tombstones_removed, 2);
        assert_eq!(stats.records_after, 2);
    }

    #[test]
    fn test_compact_minor_deduplicates() {
        let dir = test_dir("compact_dedup");
        let s1 = make_test_segment(&dir, vec![make_live_record("dup", b"old_value"), make_live_record("unique", b"keep")], "0000000000000001.nts");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let s2 = make_test_segment(&dir, vec![make_live_record("dup", b"new_value")], "0000000000000002.nts");
        let stats = compact_minor(&[s1, s2], &dir).unwrap();
        assert_eq!(stats.records_before, 3);
        assert_eq!(stats.records_after, 2);
    }
}

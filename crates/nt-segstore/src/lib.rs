pub mod compaction;
pub mod null_drift;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const STORAGE_MAGIC: &[u8; 8] = b"NTSSEG2\0";
pub const STORAGE_VERSION: u32 = 2;
pub const VSA_DIM: usize = 4096;
pub const VSA_BYTES: usize = VSA_DIM / 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VsaTag {
    SelfThought = 0x01,
    SelfMemory = 0x02,
    SelfPlan = 0x03,
    SelfGoal = 0x04,
    SelfEmotion = 0x05,
    WorldInput = 0x11,
    WorldSensor = 0x12,
    WorldWeb = 0x13,
    WorldKnowledge = 0x14,
}

impl VsaTag {
    pub fn to_byte(self) -> u8 { self as u8 }
    pub fn from_byte(b: u8) -> Option<Self> {
        Some(match b {
            0x01 => Self::SelfThought,
            0x02 => Self::SelfMemory,
            0x03 => Self::SelfPlan,
            0x04 => Self::SelfGoal,
            0x05 => Self::SelfEmotion,
            0x11 => Self::WorldInput,
            0x12 => Self::WorldSensor,
            0x13 => Self::WorldWeb,
            0x14 => Self::WorldKnowledge,
            _ => return None,
        })
    }
    pub fn is_self(self) -> bool { (self as u8) < 0x10 }
    pub fn is_world(self) -> bool { (self as u8) >= 0x10 }
}

pub const RT_KNOWLEDGE_NODE: u16 = 0x01;
pub const RT_KNOWLEDGE_EDGE: u16 = 0x02;
pub const RT_EVIDENCE: u16 = 0x03;
pub const RT_CONSCIOUSNESS_STATE: u16 = 0x04;
pub const RT_CAPABILITY: u16 = 0x05;
pub const RT_AGENT_GENOTYPE: u16 = 0x06;
pub const RT_SESSION: u16 = 0x07;
pub const RT_BENCHMARK: u16 = 0x08;
pub const RT_VSA_VECTOR: u16 = 0x09;
pub const RT_EXPERIENCE: u16 = 0x0A;
pub const RT_CHECKPOINT: u16 = 0x0B;
pub const RT_VSA_E8: u16 = 0x0C;

pub fn record_type_name(t: u16) -> &'static str {
    match t {
        0x01 => "knowledge_node",
        0x02 => "knowledge_edge",
        0x03 => "evidence",
        0x04 => "consciousness_state",
        0x05 => "capability",
        0x06 => "agent_genotype",
        0x07 => "session",
        0x08 => "benchmark",
        0x09 => "vsa_vector",
        0x0A => "experience",
        0x0B => "checkpoint",
        0x0C => "vsa_e8",
        _ => "unknown",
    }
}

fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

// ── Credit Meter (P3: soft rate limiting) ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditBudget {
    pub max_writes: u64,
    pub max_vsa_vectors: usize,
    pub max_disk_bytes: u64,
}

impl Default for CreditBudget {
    fn default() -> Self {
        Self {
            max_writes: 1_000_000,
            max_vsa_vectors: 100_000,
            max_disk_bytes: 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditMeter {
    pub used_writes: u64,
    pub used_vsa_vectors: usize,
    pub budget: CreditBudget,
}

impl CreditMeter {
    pub fn new(budget: CreditBudget) -> Self {
        Self { used_writes: 0, used_vsa_vectors: 0, budget }
    }

    pub fn deduct_write(&mut self) -> Result<(), CreditExhausted> {
        if self.used_writes >= self.budget.max_writes {
            return Err(CreditExhausted("write"));
        }
        self.used_writes += 1;
        Ok(())
    }

    pub fn deduct_vsa(&mut self) -> Result<(), CreditExhausted> {
        if self.used_vsa_vectors >= self.budget.max_vsa_vectors {
            return Err(CreditExhausted("vsa_vector"));
        }
        self.used_vsa_vectors += 1;
        Ok(())
    }

    pub fn utilization(&self) -> f64 {
        let w = self.used_writes as f64 / self.budget.max_writes.max(1) as f64;
        let v = self.used_vsa_vectors as f64 / self.budget.max_vsa_vectors.max(1) as f64;
        w.max(v)
    }

    pub fn is_exhausted(&self) -> bool {
        self.used_writes >= self.budget.max_writes
            || self.used_vsa_vectors >= self.budget.max_vsa_vectors
    }

    pub fn degrade_message(&self) -> Option<String> {
        let util = self.utilization();
        if util >= 0.95 {
            Some(format!("credit:critical util={:.1}%", util * 100.0))
        } else if util >= 0.80 {
            Some(format!("credit:degraded util={:.1}%", util * 100.0))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreditExhausted(&'static str);

impl std::fmt::Display for CreditExhausted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "credit exhausted: {}", self.0)
    }
}

impl std::error::Error for CreditExhausted {}

// ── Record ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub tag: VsaTag,
    pub record_type: u16,
    pub tombstone: bool,
    pub key: String,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

impl Record {
    pub fn new(tag: VsaTag, record_type: u16, key: &str, data: Vec<u8>) -> Self {
        Self { tag, record_type, tombstone: false, key: key.to_string(), data, timestamp: now_nanos() }
    }

    pub fn tombstone(tag: VsaTag, record_type: u16, key: &str) -> Self {
        Self { tag, record_type, tombstone: true, key: key.to_string(), data: Vec::new(), timestamp: now_nanos() }
    }

    pub fn encode(&self) -> Vec<u8> {
        let key_bytes = self.key.as_bytes();
        let mut buf = Vec::with_capacity(17 + key_bytes.len() + self.data.len());
        buf.push(self.tag.to_byte());
        buf.extend_from_slice(&self.record_type.to_le_bytes());
        buf.push(if self.tombstone { 1 } else { 0 });
        buf.extend_from_slice(&(key_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(key_bytes);
        buf.extend_from_slice(&self.data);
        buf
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 17 { return None; }
        let mut c = Cursor::new(data);
        let mut tag_byte = [0u8; 1]; c.read_exact(&mut tag_byte).map_err(|e| { log::warn!("[segstore] decode tag_byte: {e}"); e }).ok()?;
        let tag = VsaTag::from_byte(tag_byte[0])?;
        let mut rt = [0u8; 2]; c.read_exact(&mut rt).map_err(|e| { log::warn!("[segstore] decode record_type: {e}"); e }).ok()?;
        let record_type = u16::from_le_bytes(rt);
        let mut ts = [0u8; 1]; c.read_exact(&mut ts).map_err(|e| { log::warn!("[segstore] decode tombstone: {e}"); e }).ok()?;
        let tombstone = ts[0] != 0;
        let mut kl = [0u8; 2]; c.read_exact(&mut kl).map_err(|e| { log::warn!("[segstore] decode key_len: {e}"); e }).ok()?;
        let key_len = u16::from_le_bytes(kl) as usize;
        let mut dl = [0u8; 4]; c.read_exact(&mut dl).map_err(|e| { log::warn!("[segstore] decode data_len: {e}"); e }).ok()?;
        let data_len = u32::from_le_bytes(dl) as usize;
        let mut stamp = [0u8; 8]; c.read_exact(&mut stamp).map_err(|e| { log::warn!("[segstore] decode timestamp: {e}"); e }).ok()?;
        let timestamp = u64::from_le_bytes(stamp);
        let mut key = vec![0u8; key_len]; c.read_exact(&mut key).map_err(|e| { log::warn!("[segstore] decode key: {e}"); e }).ok()?;
        let mut rec_data = vec![0u8; data_len]; c.read_exact(&mut rec_data).map_err(|e| { log::warn!("[segstore] decode rec_data: {e}"); e }).ok()?;
        Some(Self {
            tag,
            record_type,
            tombstone,
            key: String::from_utf8(key).map_err(|e| { log::warn!("[segstore] decode key utf8: {e}"); e }).ok()?,
            data: rec_data,
            timestamp,
        })
    }
}

// ── Segment ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    Data,
    Index,
    Checkpoint,
}

pub struct SegmentHeader {
    pub version: u32,
    pub seg_type: SegmentType,
    pub record_count: u32,
    pub data_offset: u32,
}

pub struct SegmentWriter {
    file: std::fs::File,
    path: PathBuf,
    record_count: u32,
    data_offset: u32,
}

impl SegmentWriter {
    pub fn create(path: impl AsRef<Path>, seg_type: SegmentType) -> std::io::Result<Self> {
        let mut file = std::fs::File::create(path.as_ref())?;
        let header_len = 24u32;
        file.write_all(STORAGE_MAGIC)?;
        file.write_all(&STORAGE_VERSION.to_le_bytes())?;
        file.write_all(&(seg_type as u32).to_le_bytes())?;
        file.write_all(&0u32.to_le_bytes())?;
        file.write_all(&header_len.to_le_bytes())?;
        Ok(Self { file, path: path.as_ref().to_path_buf(), record_count: 0, data_offset: header_len })
    }

    pub fn append(&mut self, record: &Record) -> std::io::Result<()> {
        let encoded = record.encode();
        let len = encoded.len() as u32;
        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(&encoded)?;
        self.record_count += 1;
        Ok(())
    }

    pub fn finalize(mut self) -> std::io::Result<()> {
        let file_len = self.file.metadata()?.len() as u32;
        let crc = crc32(&[]);
        self.file.write_all(&crc.to_le_bytes())?;
        self.file.write_all(&file_len.to_le_bytes())?;
        self.file.flush()?;
        let mut f = std::fs::File::open(&self.path)?;
        let mut header = [0u8; 12];
        f.read_exact(&mut header)?;
        drop(f);
        let mut f = std::fs::File::options().write(true).open(&self.path)?;
        f.write_all(STORAGE_MAGIC)?;
        f.write_all(&STORAGE_VERSION.to_le_bytes())?;
        f.write_all(&(SegmentType::Data as u32).to_le_bytes())?;
        f.write_all(&self.record_count.to_le_bytes())?;
        f.write_all(&self.data_offset.to_le_bytes())?;
        f.flush()?;
        Ok(())
    }

    pub fn record_count(&self) -> u32 { self.record_count }
}

pub struct SegmentReader {
    data: Vec<u8>,
    path: PathBuf,
}

impl SegmentReader {
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let data = std::fs::read(path.as_ref())?;
        Ok(Self { data, path: path.as_ref().to_path_buf() })
    }

    pub fn header(&self) -> Option<SegmentHeader> {
        if self.data.len() < 24 { return None; }
        let magic = &self.data[0..8];
        if magic != STORAGE_MAGIC { return None; }
        let version = u32::from_le_bytes(self.data[8..12].try_into().inspect_err(|&e| { log::warn!("[segstore] header version: {e:?}"); }).ok()?);
        let st = u32::from_le_bytes(self.data[12..16].try_into().inspect_err(|&e| { log::warn!("[segstore] header seg_type: {e:?}"); }).ok()?);
        let seg_type = match st { 0 => SegmentType::Data, 1 => SegmentType::Index, 2 => SegmentType::Checkpoint, _ => return None };
        let record_count = u32::from_le_bytes(self.data[16..20].try_into().inspect_err(|&e| { log::warn!("[segstore] header record_count: {e:?}"); }).ok()?);
        let data_offset = u32::from_le_bytes(self.data[20..24].try_into().inspect_err(|&e| { log::warn!("[segstore] header data_offset: {e:?}"); }).ok()?);
        Some(SegmentHeader { version, seg_type, record_count, data_offset })
    }

    pub fn records(&self) -> Vec<Record> {
        let header = match self.header() {
            Some(h) => h,
            None => return Vec::new(),
        };
        let mut pos = header.data_offset as usize;
        let mut results = Vec::with_capacity(header.record_count as usize);
        while pos + 4 <= self.data.len() - 8 {
            let len = u32::from_le_bytes(match self.data[pos..pos+4].try_into() { Ok(l) => l, Err(_) => break });
            pos += 4;
            if len == 0 || pos + len as usize > self.data.len() - 8 { break; }
            if let Some(rec) = Record::decode(&self.data[pos..pos + len as usize]) {
                results.push(rec);
            }
            pos += len as usize;
        }
        results
    }

    pub fn find_by_key(&self, key: &str) -> Option<Record> {
        self.records().into_iter().find(|r| r.key == key && !r.tombstone)
    }

    pub fn find_by_type(&self, rt: u16) -> Vec<Record> {
        self.records().into_iter().filter(|r| r.record_type == rt && !r.tombstone).collect()
    }

    pub fn record_count(&self) -> usize {
        self.header().map(|h| h.record_count as usize).unwrap_or(0)
    }

    pub fn path(&self) -> &Path { &self.path }
}

// ── VsaIndex ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsaIndexEntry {
    pub key: String,
    pub vector: Vec<u8>,
}

pub struct VsaIndex {
    entries: Vec<VsaIndexEntry>,
    centroids: Vec<Vec<u8>>,
    partitions: Vec<Vec<usize>>,
    num_partitions: usize,
}

impl VsaIndex {
    pub fn new(num_partitions: usize) -> Self {
        Self { entries: Vec::new(), centroids: Vec::new(), partitions: Vec::new(), num_partitions }
    }

    pub fn insert(&mut self, key: &str, vector: &[u8]) {
        if vector.len() != VSA_BYTES { return; }
        if self.entries.iter().any(|e| e.key == key) { return; }
        self.entries.push(VsaIndexEntry { key: key.to_string(), vector: vector.to_vec() });
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.retain(|e| e.key != key);
    }

    pub fn build_index(&mut self) {
        if self.entries.len() < 2 { return; }
        let k = self.num_partitions.min(self.entries.len());
        let vectors: Vec<Vec<u8>> = self.entries.iter().map(|e| e.vector.clone()).collect();
        self.centroids = select_centroids(&vectors, k);
        if self.centroids.is_empty() { return; }
        self.partitions = vec![Vec::new(); self.centroids.len()];
        for (i, entry) in self.entries.iter().enumerate() {
            let mut best = 0;
            let mut best_dist = u64::MAX;
            for (j, c) in self.centroids.iter().enumerate() {
                let d = hamming_distance(&entry.vector, c);
                if d < best_dist { best_dist = d; best = j; }
            }
            self.partitions[best].push(i);
        }
    }

    pub fn search(&self, query: &[u8], k: usize) -> Vec<(String, f64)> {
        if self.entries.is_empty() { return Vec::new(); }
        if !self.centroids.is_empty() {
            let mut best_c = 0;
            let mut best_d = u64::MAX;
            for (j, c) in self.centroids.iter().enumerate() {
                let d = hamming_distance(query, c);
                if d < best_d { best_d = d; best_c = j; }
            }
            let mut results: Vec<(String, f64)> = self.partitions.get(best_c).map(|part| {
                part.iter().map(|&i| {
                    let e = &self.entries[i];
                    let sim = cosine_similarity(query, &e.vector);
                    (e.key.clone(), sim)
                }).collect()
            }).unwrap_or_default();
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            results.truncate(k);
            return results;
        }
        let mut results: Vec<(String, f64)> = self.entries.iter().map(|e| {
            let sim = cosine_similarity(query, &e.vector);
            (e.key.clone(), sim)
        }).collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

// ── StorageEngine ────────────────────────────────────────────────

pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub max_segment_bytes: u64,
    pub num_partitions: usize,
    pub auto_compact: bool,
    pub credit_budget: CreditBudget,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(".neotrix/storage"),
            max_segment_bytes: 64 * 1024 * 1024,
            num_partitions: 16,
            auto_compact: true,
            credit_budget: CreditBudget::default(),
        }
    }
}

impl StorageConfig {
    pub fn with_credit(mut self, budget: CreditBudget) -> Self {
        self.credit_budget = budget;
        self
    }
}

pub struct StorageEngine {
    config: StorageConfig,
    writer: Mutex<Option<SegmentWriter>>,
    reader: Mutex<SegmentReader>,
    index: Mutex<VsaIndex>,
    records: Mutex<HashMap<String, Record>>,
    segment_id: u64,
    credit: Mutex<CreditMeter>,
}

impl StorageEngine {
    pub fn new(config: StorageConfig) -> std::io::Result<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        let seg_id = 0u64;
        let seg_path = config.data_dir.join(format!("{:016x}.nts", seg_id));
        let writer = SegmentWriter::create(&seg_path, SegmentType::Data)?;
        let reader = SegmentReader::open(&seg_path)?;
        Ok(Self {
            index: Mutex::new(VsaIndex::new(config.num_partitions)),
            writer: Mutex::new(Some(writer)),
            reader: Mutex::new(reader),
            records: Mutex::new(HashMap::new()),
            credit: Mutex::new(CreditMeter::new(config.credit_budget.clone())),
            config,
            segment_id: seg_id,
        })
    }

    pub fn credit_meter(&self) -> CreditMeter {
        self.credit.lock().map(|c| c.clone()).unwrap_or_default()
    }

    pub fn credit_ok(&self) -> bool {
        self.credit.lock().map(|c| !c.is_exhausted()).unwrap_or(true)
    }

    pub fn degrade_message(&self) -> Option<String> {
        self.credit.lock().ok().and_then(|c| c.degrade_message())
    }

    pub fn put(&mut self, record: Record) -> std::io::Result<()> {
        {
            let mut credit = self.credit.lock().map_err(|e| {
                std::io::Error::other(e.to_string())
            })?;
            if credit.deduct_write().is_err() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ResourceBusy,
                    format!("credit exhausted: write (degraded mode, util={:.1}%)", credit.utilization() * 100.0),
                ));
            }
        }
        {
            if let Ok(mut cache) = self.records.lock() {
                cache.insert(record.key.clone(), record.clone());
            }
        }
        let mut w = self.writer.lock().map_err(|e| std::io::Error::other(e.to_string()))?;
        let mut writer = w.take().unwrap();
        writer.append(&record)?;
        let needs_rotation = (writer.record_count() as u64 * 1024) >= self.config.max_segment_bytes;
        if needs_rotation {
            writer.finalize()?;
            self.segment_id += 1;
            let seg_path = self.config.data_dir.join(format!("{:016x}.nts", self.segment_id));
            *w = Some(SegmentWriter::create(&seg_path, SegmentType::Data)?);
        } else {
            *w = Some(writer);
        }
        Ok(())
    }

    pub fn put_vsa(&mut self, key: &str, tag: VsaTag, vector: &[u8]) -> std::io::Result<()> {
        {
            let mut credit = self.credit.lock().map_err(|e| {
                std::io::Error::other(e.to_string())
            })?;
            credit.deduct_write().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::ResourceBusy, "credit exhausted: write")
            })?;
            credit.deduct_vsa().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::ResourceBusy, "credit exhausted: vsa_vector")
            })?;
        }
        let record = Record::new(tag, RT_VSA_VECTOR, key, vector.to_vec());
        self.put(record)?;
        if let Ok(mut idx) = self.index.lock() {
            idx.insert(key, vector);
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Record> {
        if let Ok(cache) = self.records.lock() {
            if let Some(r) = cache.get(key) {
                if !r.tombstone { return Some(r.clone()); }
            }
        }
        self.reader.lock().ok().and_then(|r| r.find_by_key(key))
    }

    pub fn search_vsa(&self, query: &[u8], k: usize) -> Vec<(String, f64)> {
        self.index.lock().map(|idx| idx.search(query, k)).unwrap_or_default()
    }

    pub fn find_by_type(&self, rt: u16) -> Vec<Record> {
        let mut results = Vec::new();
        if let Ok(cache) = self.records.lock() {
            for r in cache.values() {
                if r.record_type == rt && !r.tombstone {
                    results.push(r.clone());
                }
            }
        }
        if let Ok(reader) = self.reader.lock() {
            for r in reader.find_by_type(rt) {
                if !results.iter().any(|x: &Record| x.key == r.key) {
                    results.push(r);
                }
            }
        }
        results
    }

    pub fn rebuild_index(&self) {
        if let Ok(mut idx) = self.index.lock() {
            let records = self.reader.lock().ok().map(|r| r.find_by_type(RT_VSA_VECTOR)).unwrap_or_default();
            for rec in &records {
                idx.insert(&rec.key, &rec.data);
            }
            idx.build_index();
        }
    }

    pub fn delete(&self, key: &str) -> std::io::Result<()> {
        if let Ok(mut idx) = self.index.lock() {
            idx.remove(key);
        }
        if let Ok(mut cache) = self.records.lock() {
            if let Some(r) = cache.get_mut(key) {
                r.tombstone = true;
            }
        }
        Ok(())
    }

    pub fn stats(&self) -> StoreStats {
        let rec_count = self.records.lock().map(|c| c.len()).unwrap_or(0)
            + self.reader.lock().map(|r| r.record_count()).unwrap_or(0);
        let idx_count = self.index.lock().map(|i| i.len()).unwrap_or(0);
        StoreStats {
            record_count: rec_count,
            indexed_vectors: idx_count,
            segment_count: self.segment_id + 1,
            credit_utilization: self.credit.lock().map(|c| c.utilization()).unwrap_or(0.0),
            credit_exhausted: self.credit.lock().map(|c| c.is_exhausted()).unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoreStats {
    pub record_count: usize,
    pub indexed_vectors: usize,
    pub segment_count: u64,
    pub credit_utilization: f64,
    pub credit_exhausted: bool,
}

impl Default for CreditMeter {
    fn default() -> Self {
        Self::new(CreditBudget::default())
    }
}

// ── Distance Functions ───────────────────────────────────────────

pub fn hamming_distance(a: &[u8], b: &[u8]) -> u64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones() as u64).sum()
}

pub fn cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
    let hd = hamming_distance(a, b);
    let dim = (a.len().min(b.len()) * 8) as f64;
    if dim == 0.0 { return 0.0; }
    1.0 - 2.0 * hd as f64 / dim
}

fn select_centroids(vectors: &[Vec<u8>], k: usize) -> Vec<Vec<u8>> {
    if vectors.is_empty() || k == 0 { return Vec::new(); }
    let k = k.min(vectors.len());
    let mut rng = fast_rng(42);
    let mut centroids = Vec::with_capacity(k);
    centroids.push(vectors[(rng() as usize) % vectors.len()].clone());
    let mut min_dists = vec![u64::MAX; vectors.len()];
    for _ in 1..k {
        let last = centroids.last().unwrap();
        let mut total: u64 = 0;
        for (i, v) in vectors.iter().enumerate() {
            let d = hamming_distance(v, last);
            min_dists[i] = min_dists[i].min(d);
            total += min_dists[i];
        }
        if total == 0 { centroids.push(vectors[(rng() as usize) % vectors.len()].clone()); continue; }
        let threshold = (rng() as f64 / u64::MAX as f64) * total as f64;
        let mut cumulative = 0u64;
        let mut chosen = 0;
        for (i, &d) in min_dists.iter().enumerate() {
            cumulative += d;
            if cumulative as f64 >= threshold { chosen = i; break; }
        }
        centroids.push(vectors[chosen].clone());
    }
    centroids
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn fast_rng(seed: u64) -> impl FnMut() -> u64 {
    let mut state = seed;
    move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        state >> 33
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("nt_segstore_test_{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn test_record_roundtrip() {
        let data = b"hello vsa world".to_vec();
        let r = Record::new(VsaTag::SelfThought, RT_KNOWLEDGE_NODE, "test-key", data.clone());
        let encoded = r.encode();
        let decoded = Record::decode(&encoded).unwrap();
        assert_eq!(r.tag, decoded.tag);
        assert_eq!(r.record_type, decoded.record_type);
        assert_eq!(r.key, decoded.key);
        assert_eq!(r.data, decoded.data);
        assert_eq!(r.tombstone, decoded.tombstone);
    }

    #[test]
    fn test_tombstone_record() {
        let r = Record::tombstone(VsaTag::WorldKnowledge, RT_EVIDENCE, "obsolete-evidence");
        let encoded = r.encode();
        let decoded = Record::decode(&encoded).unwrap();
        assert!(decoded.tombstone);
        assert!(decoded.data.is_empty());
        assert_eq!(decoded.key, "obsolete-evidence");
    }

    #[test]
    fn test_segment_write_read() {
        let dir = test_dir("segment");
        let path = dir.join("test.nts");
        let mut writer = SegmentWriter::create(&path, SegmentType::Data).unwrap();
        let r1 = Record::new(VsaTag::SelfMemory, RT_SESSION, "sess-1", b"mem1".to_vec());
        let r2 = Record::new(VsaTag::WorldInput, RT_KNOWLEDGE_NODE, "node-42", b"data42".to_vec());
        writer.append(&r1).unwrap();
        writer.append(&r2).unwrap();
        writer.finalize().unwrap();

        let reader = SegmentReader::open(&path).unwrap();
        let header = reader.header().unwrap();
        assert_eq!(header.version, STORAGE_VERSION);
        assert_eq!(header.record_count, 2);
        let records = reader.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].key, "sess-1");
        assert_eq!(records[1].key, "node-42");
    }

    #[test]
    fn test_storage_engine_put_get() {
        let dir = test_dir("engine");
        let cfg = StorageConfig { data_dir: dir.clone(), ..Default::default() };
        let mut engine = StorageEngine::new(cfg).unwrap();
        let r = Record::new(VsaTag::WorldKnowledge, RT_KNOWLEDGE_NODE, "k1", b"value1".to_vec());
        engine.put(r).unwrap();
        let got = engine.get("k1");
        assert!(got.is_some());
        assert_eq!(got.unwrap().data, b"value1");
    }

    #[test]
    fn test_credit_meter_deduct() {
        let budget = CreditBudget { max_writes: 3, max_vsa_vectors: 2, max_disk_bytes: 1024 };
        let mut meter = CreditMeter::new(budget);
        assert!(meter.deduct_write().is_ok());
        assert!(meter.deduct_write().is_ok());
        assert!(meter.deduct_write().is_ok());
        assert!(meter.deduct_write().is_err());
        assert!(meter.is_exhausted());
    }

    #[test]
    fn test_credit_degrade_message() {
        let budget = CreditBudget { max_writes: 100, max_vsa_vectors: 100, max_disk_bytes: 1024 };
        let mut meter = CreditMeter::new(budget);
        assert!(meter.degrade_message().is_none());
        for _ in 0..80 {
            let _ = meter.deduct_write();
        }
        assert_eq!(meter.degrade_message(), Some("credit:degraded util=80.0%".to_string()));
    }

    #[test]
    fn test_storage_engine_credit_hard_stops() {
        let dir = test_dir("credit_stop");
        let budget = CreditBudget { max_writes: 2, max_vsa_vectors: 100, max_disk_bytes: 1024 };
        let cfg = StorageConfig { data_dir: dir.clone(), credit_budget: budget, ..Default::default() };
        let mut engine = StorageEngine::new(cfg).unwrap();

        assert!(engine.put(Record::new(VsaTag::SelfThought, RT_SESSION, "a", vec![0; 8])).is_ok());
        assert!(engine.put(Record::new(VsaTag::SelfThought, RT_SESSION, "b", vec![0; 8])).is_ok());
        assert!(engine.put(Record::new(VsaTag::SelfThought, RT_SESSION, "c", vec![0; 8])).is_err());
        assert!(engine.credit_ok() == false);
    }

    #[test]
    fn test_storage_engine_stats_credit() {
        let dir = test_dir("stats_credit");
        let budget = CreditBudget { max_writes: 5, max_vsa_vectors: 10, max_disk_bytes: 1024 };
        let cfg = StorageConfig { data_dir: dir.clone(), credit_budget: budget, ..Default::default() };
        let mut engine = StorageEngine::new(cfg).unwrap();
        for i in 0..3 {
            engine.put(Record::new(VsaTag::SelfThought, RT_SESSION, &format!("k{}", i), vec![i as u8; 8])).unwrap();
        }
        let s = engine.stats();
        assert_eq!(s.record_count, 3);
        assert!(s.credit_utilization > 0.0);
    }

    #[test]
    fn test_vsa_index_insert_search() {
        let mut idx = VsaIndex::new(4);
        let v1 = vec![0u8; VSA_BYTES];
        let mut v2 = vec![0u8; VSA_BYTES];
        v2[0] = 0xFF;
        idx.insert("vec-zero", &v1);
        idx.insert("vec-partial", &v2);
        idx.build_index();
        let results = idx.search(&v1, 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "vec-zero");
        assert!(results[0].1 > 0.99);
    }

    #[test]
    fn test_vsa_tag_self_world() {
        assert!(VsaTag::SelfThought.is_self());
        assert!(VsaTag::SelfPlan.is_self());
        assert!(!VsaTag::SelfThought.is_world());
        assert!(VsaTag::WorldInput.is_world());
        assert!(!VsaTag::WorldInput.is_self());
    }
}

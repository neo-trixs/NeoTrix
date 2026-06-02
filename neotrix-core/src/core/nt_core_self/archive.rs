use super::silicon_self::SiliconSelfModel;

pub struct AttentionSnapshot {
    pub dominant_domain: String,
    pub active_count: usize,
    pub avg_activation: f64,
}

pub struct SiliconSnapshot {
    pub iteration: usize,
    pub timestamp_id: usize,
    pub label: String,
    pub attention_snapshot: AttentionSnapshot,
    pub strategy_count: usize,
    pub trace_count: usize,
    pub capabilities: Vec<(String, f64)>,
}

impl SiliconSnapshot {
    pub fn to_json_line(&self) -> String {
        let escaped_label = self.label.replace('\\', "\\\\").replace('"', "\\\"");
        let caps: Vec<String> = self.capabilities.iter()
            .map(|(k, v)| format!("[\"{}\",{}]", k, v))
            .collect();
        format!(
            r#"{{"iteration":{},"timestamp_id":{},"label":"{}","trace_count":{},"strategy_count":{},"attention":{{"dominant":"{}","active":{},"avg_activation":{}}},"capabilities":[{}]}}"#,
            self.iteration,
            self.timestamp_id,
            escaped_label,
            self.trace_count,
            self.strategy_count,
            self.attention_snapshot.dominant_domain,
            self.attention_snapshot.active_count,
            self.attention_snapshot.avg_activation,
            caps.join(","),
        )
    }

    pub fn from_json_line(line: &str) -> Option<Self> {
        let line = line.trim();
        if !(line.starts_with('{') && line.ends_with('}')) {
            return None;
        }

        fn kv_after<'a>(s: &'a str, key: &str) -> Option<&'a str> {
            let pat = format!("\"{}\":", key);
            let pos = s.find(&pat)?;
            Some(&s[pos + pat.len()..])
        }

        fn parse_int(s: &str) -> Option<(usize, &str)> {
            let start = s.find(|c: char| c.is_ascii_digit())?;
            let tail = &s[start..];
            let end = tail.find(|c: char| !c.is_ascii_digit())?;
            let val: usize = tail[..end].parse().ok()?;
            Some((val, &tail[end..]))
        }

        fn parse_f64_from_str(s: &str) -> Option<(f64, &str)> {
            let start = s.find(|c: char| c.is_ascii_digit() || c == '-')?;
            let tail = &s[start..];
            let end = tail.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E' && c != '+')?;
            let val: f64 = tail[..end].parse().ok()?;
            Some((val, &tail[end..]))
        }

        fn parse_quoted(s: &str) -> Option<String> {
            let start = s.find('"')?;
            let tail = &s[start + 1..];
            let end = tail.find('"')?;
            Some(tail[..end].to_string())
        }

        let iteration: usize = parse_int(kv_after(line, "iteration")?)?.0;
        let timestamp_id: usize = parse_int(kv_after(line, "timestamp_id")?)?.0;
        let label = parse_quoted(kv_after(line, "label")?)?;
        let trace_count: usize = parse_int(kv_after(line, "trace_count")?)?.0;
        let strategy_count: usize = parse_int(kv_after(line, "strategy_count")?)?.0;

        let attn_raw = kv_after(line, "attention")?;
        let dominant = parse_quoted(kv_after(attn_raw, "dominant")?)?;
        let active_count: usize = parse_int(kv_after(attn_raw, "active")?)?.0;
        let avg_activation: f64 = parse_f64_from_str(kv_after(attn_raw, "avg_activation")?)?.0;

        let attention_snapshot = AttentionSnapshot {
            dominant_domain: dominant,
            active_count,
            avg_activation,
        };

        let mut capabilities = Vec::new();
        if let Some(caps_raw) = kv_after(line, "capabilities") {
            let inner = caps_raw
                .strip_prefix('[')
                .and_then(|s| {
                    let mut depth = 1u32;
                    for (i, b) in s.bytes().enumerate() {
                        if b == b'[' {
                            depth += 1;
                        } else if b == b']' {
                            depth -= 1;
                            if depth == 0 {
                                return Some(&s[..i]);
                            }
                        }
                    }
                    None
                })
                .unwrap_or("");

            let bytes = inner.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'[' {
                    let arr_start = i + 1;
                    if let Some(q_start) = inner[arr_start..].find('"') {
                        let name_s = arr_start + q_start + 1;
                        if let Some(q_end) = inner[name_s..].find('"') {
                            let name = inner[name_s..name_s + q_end].to_string();
                            let after_name = name_s + q_end;
                            if let Some((val, _)) = parse_f64_from_str(&inner[after_name..]) {
                                capabilities.push((name, val));
                            }
                        }
                    }
                    while i < bytes.len() && bytes[i] != b']' {
                        i += 1;
                    }
                }
                i += 1;
            }
        }

        Some(SiliconSnapshot {
            iteration,
            timestamp_id,
            label,
            attention_snapshot,
            strategy_count,
            trace_count,
            capabilities,
        })
    }

    pub fn from_model(iteration: usize, label: &str, model: &SiliconSelfModel) -> Self {
        let dominant_domain = model.attention_manager.dominant_domain()
            .map(|d| d.label().to_string())
            .unwrap_or_else(|| "none".to_string());
        let active_count = model.attention_manager.active_heads().len();
        let avg_activation = if model.attention_manager.heads.is_empty() {
            0.0
        } else {
            model.attention_manager.heads.iter().map(|h| h.activation).sum::<f64>()
                / model.attention_manager.heads.len() as f64
        };

        let capabilities: Vec<(String, f64)> = model.identity.capabilities.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        SiliconSnapshot {
            iteration,
            timestamp_id: iteration,
            label: label.to_string(),
            attention_snapshot: AttentionSnapshot {
                dominant_domain,
                active_count,
                avg_activation,
            },
            strategy_count: model.strategy_registry.strategies.len(),
            trace_count: model.thinking_traces.len(),
            capabilities,
        }
    }

    pub fn diff_capabilities(&self, other: &SiliconSnapshot) -> Vec<(String, f64, f64)> {
        let mut result = Vec::new();
        for (name, level) in &self.capabilities {
            let other_level = other.capabilities.iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            if (level - other_level).abs() > 0.01 {
                result.push((name.clone(), *level, other_level));
            }
        }
        for (name, level) in &other.capabilities {
            if !self.capabilities.iter().any(|(n, _)| n == name) {
                result.push((name.clone(), 0.0, *level));
            }
        }
        result
    }
}

pub struct SiliconArchive {
    pub snapshots: Vec<SiliconSnapshot>,
    pub max_snapshots: usize,
    pub archive_count: usize,
}

impl Default for SiliconArchive {
    fn default() -> Self {
        Self::new()
    }
}

impl SiliconArchive {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: 50,
            archive_count: 0,
        }
    }

    pub fn with_max(max: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: max,
            archive_count: 0,
        }
    }

    pub fn snapshot(&mut self, label: &str, model: &SiliconSelfModel) -> usize {
        let id = self.archive_count;
        let snap = SiliconSnapshot::from_model(id, label, model);
        self.snapshots.push(snap);
        self.archive_count += 1;

        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }

        id
    }

    pub fn latest(&self) -> Option<&SiliconSnapshot> {
        self.snapshots.last()
    }

    pub fn by_iteration(&self, iteration: usize) -> Option<&SiliconSnapshot> {
        self.snapshots.iter().rev().find(|s| s.iteration == iteration)
    }

    pub fn diff_range(&self, from_id: usize, to_id: usize) -> Vec<(String, f64, f64)> {
        let from = self.snapshots.iter().find(|s| s.timestamp_id == from_id);
        let to = self.snapshots.iter().find(|s| s.timestamp_id == to_id);
        match (from, to) {
            (Some(f), Some(t)) => f.diff_capabilities(t),
            _ => Vec::new(),
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "SiliconArchive | snapshots={}/{} | latest_iter={} | archive_count={}",
            self.snapshots.len(),
            self.max_snapshots,
            self.latest().map(|s| s.iteration).unwrap_or(0),
            self.archive_count,
        )
    }

    pub fn backtrack(&self, steps_back: usize) -> Option<&SiliconSnapshot> {
        if steps_back == 0 || steps_back >= self.snapshots.len() {
            return None;
        }
        let idx = self.snapshots.len() - 1 - steps_back;
        self.snapshots.get(idx)
    }

    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        for snap in &self.snapshots {
            writeln!(file, "{}", snap.to_json_line())?;
        }
        Ok(())
    }

    pub fn load_from(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut archive = SiliconArchive::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(snap) = SiliconSnapshot::from_json_line(line) {
                archive.archive_count = archive.archive_count.max(snap.timestamp_id + 1);
                archive.snapshots.push(snap);
            }
        }
        while archive.snapshots.len() > archive.max_snapshots {
            archive.snapshots.remove(0);
        }
        Ok(archive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::silicon_self::SiliconSelfModel;

    #[test]
    fn test_archive_new() {
        let a = SiliconArchive::new();
        assert!(a.snapshots.is_empty());
        assert_eq!(a.max_snapshots, 50);
        assert_eq!(a.archive_count, 0);
    }

    #[test]
    fn test_snapshot_creates_from_model() {
        let mut a = SiliconArchive::new();
        let model = SiliconSelfModel::new();
        let id = a.snapshot("initial", &model);
        assert_eq!(id, 0);
        assert_eq!(a.snapshots.len(), 1);
        assert_eq!(a.snapshots[0].label, "initial");
        assert_eq!(a.snapshots[0].trace_count, 0);
    }

    #[test]
    fn test_latest_after_snapshots() {
        let mut a = SiliconArchive::new();
        let model = SiliconSelfModel::new();
        a.snapshot("first", &model);
        a.snapshot("second", &model);
        let latest = a.latest().expect("value should be ok in test");
        assert_eq!(latest.label, "second");
        assert_eq!(latest.timestamp_id, 1);
    }

    #[test]
    fn test_max_snapshots_enforced() {
        let mut a = SiliconArchive::with_max(3);
        let model = SiliconSelfModel::new();
        a.snapshot("s0", &model);
        a.snapshot("s1", &model);
        a.snapshot("s2", &model);
        a.snapshot("s3", &model);
        assert_eq!(a.snapshots.len(), 3);
        assert_eq!(a.snapshots[0].label, "s1");
        assert_eq!(a.snapshots[2].label, "s3");
    }

    #[test]
    fn test_capability_diff_detects_change() {
        let mut model = SiliconSelfModel::new();
        let mut a = SiliconArchive::new();
        a.snapshot("before", &model);
        model.identity.update_capability("testing", 0.95);
        a.snapshot("after", &model);
        let diffs = a.diff_range(0, 1);
        assert!(diffs.iter().any(|(name, _, _)| name == "testing"));
    }

    #[test]
    fn test_backtrack_returns_correct() {
        let mut a = SiliconArchive::new();
        let model = SiliconSelfModel::new();
        a.snapshot("s0", &model);
        a.snapshot("s1", &model);
        a.snapshot("s2", &model);
        let back = a.backtrack(1).expect("value should be ok in test");
        assert_eq!(back.label, "s1");
        assert!(a.backtrack(3).is_none());
    }
}

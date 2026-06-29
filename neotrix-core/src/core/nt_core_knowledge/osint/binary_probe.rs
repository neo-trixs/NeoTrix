use super::intelligence_probe::{IntelligenceProbe, ProbeFinding, ProbeResult, ProbeSeverity};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

fn get_file_type(path: &Path) -> String {
    let output = Command::new("file").arg("-b").arg(path).output().ok();
    match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "file command unavailable".into(),
    }
}

fn compute_hashes(path: &Path) -> HashMap<String, String> {
    let mut hashes = HashMap::new();
    if let Ok(data) = fs::read(path) {
        hashes.insert("size_bytes".into(), data.len().to_string());
        if let Ok(output) = Command::new("shasum")
            .arg("-a")
            .arg("256")
            .arg(path)
            .output()
        {
            if output.status.success() {
                let line = String::from_utf8_lossy(&output.stdout);
                if let Some(hash) = line.split_whitespace().next() {
                    hashes.insert("sha256".into(), hash.to_string());
                }
            }
        }
        if let Ok(output) = Command::new("md5").arg("-q").arg(path).output() {
            if output.status.success() {
                let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                hashes.insert("md5".into(), hash);
            }
        }
    }
    hashes
}

fn extract_strings(path: &Path, min_len: usize) -> Vec<String> {
    let output = Command::new("strings")
        .arg("-n")
        .arg(min_len.to_string())
        .arg(path)
        .output()
        .ok();
    match output {
        Some(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let mut strings: Vec<String> = text
                .lines()
                .filter(|l| l.len() >= min_len)
                .take(200)
                .map(|l| l.to_string())
                .collect();
            strings.sort();
            strings.dedup();
            strings
        }
        _ => Vec::new(),
    }
}

fn classify_binary_type(file_type: &str) -> Vec<&'static str> {
    let mut tags = Vec::new();
    let lower = file_type.to_lowercase();
    if lower.contains("elf") {
        tags.push("ELF");
    }
    if lower.contains("pe32")
        || lower.contains("pe ")
        || (lower.contains("executable") && lower.contains("windows"))
    {
        tags.push("PE");
    }
    if lower.contains("mach-o") || lower.contains("mach object") {
        tags.push("Mach-O");
    }
    if lower.contains("ar archive") {
        tags.push("StaticLibrary");
    }
    if lower.contains("shared object") || lower.contains("shared library") || lower.contains("dll")
    {
        tags.push("SharedLibrary");
    }
    if lower.contains("executable") && !lower.contains("shared") {
        tags.push("Executable");
    }
    if lower.contains("not stripped") {
        tags.push("NotStripped");
    }
    if lower.contains("stripped") {
        tags.push("Stripped");
    }
    if lower.contains("dynamically linked") {
        tags.push("DynamicallyLinked");
    }
    if lower.contains("statically linked") {
        tags.push("StaticallyLinked");
    }
    if lower.contains("position independent") {
        tags.push("PIE");
    }
    if lower.contains("32-bit") {
        tags.push("32bit");
    }
    if lower.contains("64-bit") {
        tags.push("64bit");
    }
    if lower.contains("arm") && !lower.contains("aarch64") {
        tags.push("ARM");
    }
    if lower.contains("aarch64") {
        tags.push("AArch64");
    }
    if lower.contains("x86") && !lower.contains("x86-64") && !lower.contains("amd64") {
        tags.push("x86");
    }
    if lower.contains("x86-64") || lower.contains("amd64") {
        tags.push("x86_64");
    }
    if lower.contains("mips") {
        tags.push("MIPS");
    }
    if lower.contains("packed")
        || lower.contains("upx")
        || lower.contains("aspack")
        || lower.contains("themida")
    {
        tags.push("Packed");
    }
    if lower.contains("data") && !lower.contains("executable") && !lower.contains("shared object") {
        tags.push("DataFile");
    }
    tags
}

fn looks_like_ip(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    for p in &parts {
        if p.is_empty() || p.len() > 3 {
            return false;
        }
        if !p.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    true
}

fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://")
}

fn looks_like_domain(s: &str) -> bool {
    let s = s.trim_start_matches("www.");
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    let tld = parts.last().unwrap_or(&"");
    let valid: &[&str] = &[
        "com", "net", "org", "io", "gov", "edu", "mil", "xyz", "ai", "app", "dev", "info", "co",
        "uk", "de", "cn", "jp", "fr", "ru", "br", "au", "ca", "in", "it", "nl", "es", "kr",
    ];
    valid.contains(tld)
}

fn looks_like_registry_path(s: &str) -> bool {
    let u = s.to_uppercase();
    u.starts_with("HK") || u.starts_with("HKEY")
}

fn detect_iocs(strings: &[String]) -> Vec<String> {
    let mut iocs = Vec::new();
    for s in strings {
        if looks_like_ip(s) {
            iocs.push(format!("IP: {}", s));
        }
        if looks_like_url(s) {
            iocs.push(format!("URL: {}", s));
        }
        if looks_like_domain(s) && !looks_like_url(s) {
            iocs.push(format!("Domain: {}", s));
        }
        if looks_like_registry_path(s) {
            iocs.push(format!("Registry: {}", s));
        }
    }
    iocs.sort();
    iocs.dedup();
    iocs.truncate(50);
    iocs
}

pub struct BinaryAnalysisProbe;

impl BinaryAnalysisProbe {
    pub fn new() -> Self {
        Self
    }
}

impl IntelligenceProbe for BinaryAnalysisProbe {
    fn name(&self) -> &str {
        "binary_analysis"
    }

    fn description(&self) -> &str {
        "Binary file analysis: file type identification, hash computation, string extraction, IOC detection"
    }

    fn probe(&self, target: &str, _timeout: u64) -> ProbeResult {
        let start = Instant::now();
        let path = Path::new(target);
        let mut result = ProbeResult::new("binary_analysis", target);

        if !path.exists() {
            result.findings.push(
                ProbeFinding::new("error", "File not found", "filesystem")
                    .with_severity(ProbeSeverity::Critical)
                    .with_confidence(1.0),
            );
            result.error = Some("File not found".into());
            result.duration_ms = start.elapsed().as_millis() as u64;
            return result;
        }

        let file_type = get_file_type(path);
        let binary_tags = classify_binary_type(&file_type);
        result.findings.push(
            ProbeFinding::new("file_type", &file_type, "file")
                .with_confidence(0.9)
                .with_severity(ProbeSeverity::Info),
        );
        if !binary_tags.is_empty() {
            result.findings.push(
                ProbeFinding::new(
                    "binary_classification",
                    &binary_tags.join(", "),
                    "classification",
                )
                .with_confidence(0.85)
                .with_severity(ProbeSeverity::Info),
            );
        }

        let hashes = compute_hashes(path);
        for (key, value) in &hashes {
            result.findings.push(
                ProbeFinding::new(key, value, "hash")
                    .with_confidence(1.0)
                    .with_severity(ProbeSeverity::Info),
            );
        }

        let strings = extract_strings(path, 6);
        if !strings.is_empty() {
            let interesting: Vec<&str> = strings
                .iter()
                .filter(|s| {
                    let lower = s.to_lowercase();
                    lower.contains("http")
                        || lower.contains("cmd")
                        || lower.contains("powershell")
                        || lower.contains("shell")
                        || lower.contains("exec")
                        || lower.contains("decrypt")
                        || lower.contains("encrypt")
                        || lower.contains("key")
                        || lower.contains("token")
                        || lower.contains("secret")
                        || lower.contains("password")
                        || lower.contains("admin")
                        || lower.contains("debug")
                        || lower.contains("backdoor")
                        || lower.contains("inject")
                })
                .map(|s| s.as_str())
                .collect();
            result.findings.push(
                ProbeFinding::new("strings_count", &strings.len().to_string(), "strings")
                    .with_confidence(0.9)
                    .with_severity(ProbeSeverity::Info),
            );
            if !interesting.is_empty() {
                result.findings.push(
                    ProbeFinding::new("interesting_strings", &interesting.join(" | "), "strings")
                        .with_confidence(0.75)
                        .with_severity(ProbeSeverity::Medium),
                );
            }
            let iocs = detect_iocs(&strings);
            if !iocs.is_empty() {
                let sev = if iocs
                    .iter()
                    .any(|i| i.starts_with("IP:") || i.starts_with("URL:"))
                {
                    ProbeSeverity::High
                } else {
                    ProbeSeverity::Medium
                };
                result.findings.push(
                    ProbeFinding::new("extracted_iocs", &iocs.join("; "), "ioc")
                        .with_confidence(0.6)
                        .with_severity(sev),
                );
            }
        }

        result.success = true;
        result.duration_ms = start.elapsed().as_millis() as u64;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_classify_binary_type_elf() {
        let tags = classify_binary_type("ELF 64-bit LSB executable, x86-64, dynamically linked");
        assert!(tags.contains(&"ELF"));
        assert!(tags.contains(&"Executable"));
        assert!(tags.contains(&"64bit"));
        assert!(tags.contains(&"x86_64"));
        assert!(tags.contains(&"DynamicallyLinked"));
    }

    #[test]
    fn test_classify_binary_type_macho() {
        let tags = classify_binary_type("Mach-O 64-bit executable x86_64");
        assert!(tags.contains(&"Mach-O"));
        assert!(tags.contains(&"Executable"));
    }

    #[test]
    fn test_classify_binary_type_pe() {
        let tags = classify_binary_type("PE32 executable (GUI) Intel 80386, for MS Windows");
        assert!(tags.contains(&"PE"));
        assert!(tags.contains(&"Executable"));
    }

    #[test]
    fn test_classify_binary_type_data() {
        let tags = classify_binary_type("data");
        assert!(tags.contains(&"DataFile"));
    }

    #[test]
    fn test_detect_iocs_empty() {
        let strings = vec!["hello".into(), "world".into()];
        let iocs = detect_iocs(&strings);
        assert!(iocs.is_empty());
    }

    #[test]
    fn test_detect_iocs_ip() {
        let strings = vec!["connect to 192.168.1.1".into()];
        let iocs = detect_iocs(&strings);
        assert!(iocs.iter().any(|i| i.contains("192.168.1.1")));
    }

    #[test]
    fn test_detect_iocs_url() {
        let strings = vec!["fetch https://evil.com/payload".into()];
        let iocs = detect_iocs(&strings);
        assert!(iocs.iter().any(|i| i.contains("https://evil.com")));
    }

    #[test]
    fn test_binary_probe_file_not_found() {
        let probe = BinaryAnalysisProbe::new();
        let result = probe.probe("/nonexistent/file.exe", 10);
        assert!(!result.success);
    }

    #[test]
    fn test_binary_probe_analysis() {
        let mut tmp = NamedTempFile::new().expect("create temp file");
        tmp.write_all(b"Hello world test http://example.com").ok();
        let path = tmp.path().to_str().unwrap().to_string();
        let probe = BinaryAnalysisProbe::new();
        let result = probe.probe(&path, 10);
        assert!(result.success);
        assert!(result.findings.iter().any(|f| f.key == "size_bytes"));
    }

    #[test]
    fn test_probe_name() {
        let probe = BinaryAnalysisProbe::new();
        assert_eq!(probe.name(), "binary_analysis");
    }
}

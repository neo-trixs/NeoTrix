//! UnifiedModelPool — 统一模型池，聚合本地 GGUF + 云端免费 API
//!
//! 架构:
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │  UnifiedModelPool                                   │
//! │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
//! │  │ LocalGguf    │  │ CloudFree    │  │ Local     │ │
//! │  │ Source       │  │ Source       │  │ Endpoint  │ │
//! │  │ (GGUF files) │  │ (API catalogs)│  │ (running) │ │
//! │  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘ │
//! │         └─────────────────┼─────────────────┘       │
//! │                           ▼                         │
//! │                  UnifiedModelEntry                  │
//! │  (id, name, category, source, size_gb, tier, ...)   │
//! └─────────────────────────────────────────────────────┘
//! ```

use std::path::{Path, PathBuf};

use crate::l1_action::nt_io::nt_io_provider::catalog::provider_catalog::ProviderCategory;
use crate::l1_action::nt_io::nt_io_provider::common::factory::LlmProviderType;

// ═══════════════════════════════════════════════════════════
// UnifiedModelEntry — 统一模型表示
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct UnifiedModelEntry {
    pub id: String,
    pub display_name: String,
    pub category: ProviderCategory,
    pub source: String,
    pub base_url: String,
    pub model_id: String,
    pub size_gb: f64,
    pub tier: String,
    pub is_free: bool,
    pub requires_api_key: bool,
    pub api_key_env: Option<String>,
    pub provider_type: LlmProviderType,
    pub mmproj_path: Option<PathBuf>,
}

impl UnifiedModelEntry {
    pub fn local_gguf(path: PathBuf, size_gb: f64) -> Self {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        Self {
            id: format!("local/{}", name),
            display_name: name.clone(),
            category: ProviderCategory::Local,
            source: "gguf".into(),
            base_url: "http://localhost:8080/v1".into(),
            model_id: name,
            size_gb,
            tier: classify_size_tier(size_gb),
            is_free: true,
            requires_api_key: false,
            api_key_env: None,
            provider_type: LlmProviderType::OpenAI,
            mmproj_path: None,
        }
    }

    pub fn cloud_free(
        provider: &str,
        model_id: &str,
        display_name: &str,
        base_url: &str,
        tier: &str,
        requires_api_key: bool,
        api_key_env: Option<&str>,
        provider_type: LlmProviderType,
    ) -> Self {
        Self {
            id: format!("{}/{}", provider, model_id),
            display_name: display_name.to_string(),
            category: ProviderCategory::Cloud,
            source: "cloud_free".into(),
            base_url: base_url.to_string(),
            model_id: model_id.to_string(),
            size_gb: 0.0,
            tier: tier.to_string(),
            is_free: true,
            requires_api_key,
            api_key_env: api_key_env.map(|s| s.to_string()),
            provider_type,
            mmproj_path: None,
        }
    }
}

fn classify_size_tier(size_gb: f64) -> String {
    if size_gb < 2.0 { "t0-cheap".into() }
    else if size_gb < 5.0 { "t1-standard".into() }
    else if size_gb < 10.0 { "t2-balanced".into() }
    else { "t3-powerful".into() }
}

// ═══════════════════════════════════════════════════════════
// ModelSource trait — 插件接口
// ═══════════════════════════════════════════════════════════

/// 模型源插件 trait — 所有模型发现后端实现此接口。
pub trait ModelSource: Send + Sync {
    /// 源名称 (如 "gguf", "openrouter", "groq")
    fn name(&self) -> &str;

    /// 源分类
    fn category(&self) -> ProviderCategory;

    /// 发现可用模型
    fn discover(&self) -> Vec<UnifiedModelEntry>;

    /// 健康检查 (可选, 默认 true)
    fn is_available(&self) -> bool { true }
}

// ═══════════════════════════════════════════════════════════
// LocalGgufSource — 本地 GGUF 文件扫描
// ═══════════════════════════════════════════════════════════

pub struct LocalGgufSource {
    scan_dirs: Vec<PathBuf>,
    max_size_gb: f64,
}

impl LocalGgufSource {
    pub fn new(scan_dirs: Vec<PathBuf>, max_size_gb: f64) -> Self {
        Self { scan_dirs, max_size_gb }
    }

    pub fn default_m5() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/neo".into());
        Self::new(
            vec![
                PathBuf::from("/Users/neo/Downloads/neotrix/models"),
                PathBuf::from(&home).join(".cache/neotrix/models"),
                PathBuf::from(&home).join(".ollama/models"),
            ],
            5.0, // M5 16GB: 最大 5GB 模型
        )
    }

    fn scan_dir(&self, dir: &Path) -> Vec<UnifiedModelEntry> {
        let mut entries = Vec::new();
        if !dir.exists() { return entries; }

        if let Ok(read_dir) = std::fs::read_dir(dir) {
            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    // 递归扫描子目录 (如 mmproj 文件)
                    if let Ok(sub) = std::fs::read_dir(&path) {
                        for f in sub.filter_map(|e| e.ok()) {
                            if f.path().extension().map(|e| e == "gguf").unwrap_or(false) {
                                if let Some(model) = self.try_parse_gguf(f.path()) {
                                    entries.push(model);
                                }
                            }
                        }
                    }
                } else if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                    if let Some(model) = self.try_parse_gguf(path) {
                        entries.push(model);
                    }
                }
            }
        }
        entries
    }

    fn try_parse_gguf(&self, path: PathBuf) -> Option<UnifiedModelEntry> {
        let metadata = std::fs::metadata(&path).ok()?;
        let size_gb = metadata.len() as f64 / 1024.0 / 1024.0 / 1024.0;

        // 跳过超过阈值的模型
        if size_gb > self.max_size_gb {
            return None;
        }

        // 跳过 mmproj 文件 (多模态投影层, 不是独立模型)
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if name.starts_with("mmproj") {
            return None;
        }

        let mut entry = UnifiedModelEntry::local_gguf(path.clone(), size_gb);

        // 查找对应的 mmproj 文件
        if let Some(parent) = path.parent() {
            if let Ok(read_dir) = std::fs::read_dir(parent) {
                for f in read_dir.filter_map(|e| e.ok()) {
                    let fp = f.path();
                    if fp.extension().map(|e| e == "gguf").unwrap_or(false) {
                        let fn_name = fp.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        if fn_name.starts_with("mmproj") && fn_name.contains(&entry.model_id[..entry.model_id.len().min(10)]) {
                            entry.mmproj_path = Some(fp);
                            break;
                        }
                    }
                }
            }
        }

        Some(entry)
    }
}

impl ModelSource for LocalGgufSource {
    fn name(&self) -> &str { "gguf" }

    fn category(&self) -> ProviderCategory { ProviderCategory::Local }

    fn discover(&self) -> Vec<UnifiedModelEntry> {
        let mut all = Vec::new();
        for dir in &self.scan_dirs {
            all.extend(self.scan_dir(dir));
        }
        // 按大小降序
        all.sort_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap_or(std::cmp::Ordering::Equal));
        all
    }
}

// ═══════════════════════════════════════════════════════════
// CloudFreeSource — 云端免费 API 聚合
// ═══════════════════════════════════════════════════════════

pub struct CloudFreeSource;

impl CloudFreeSource {
    pub fn new() -> Self { Self }
}

impl ModelSource for CloudFreeSource {
    fn name(&self) -> &str { "cloud_free" }

    fn category(&self) -> ProviderCategory { ProviderCategory::Cloud }

    fn discover(&self) -> Vec<UnifiedModelEntry> {
        let mut entries = Vec::new();

        // Groq 免费模型
        let groq_base = "https://api.groq.com/openai/v1";
        entries.extend([
            UnifiedModelEntry::cloud_free("groq", "llama-4-scout-17b-16e-instruct", "Llama 4 Scout 17B (Groq)", groq_base, "t3-powerful", true, Some("GROQ_API_KEY"), LlmProviderType::FreeApi),
            UnifiedModelEntry::cloud_free("groq", "gemma2-9b-it", "Gemma 2 9B IT (Groq)", groq_base, "t1-standard", true, Some("GROQ_API_KEY"), LlmProviderType::FreeApi),
            UnifiedModelEntry::cloud_free("groq", "llama-3.3-70b-versatile", "Llama 3.3 70B (Groq)", groq_base, "t3-powerful", true, Some("GROQ_API_KEY"), LlmProviderType::FreeApi),
            UnifiedModelEntry::cloud_free("groq", "deepseek-r1-distill-llama-70b", "DeepSeek R1 Distill 70B (Groq)", groq_base, "t4-frontier", true, Some("GROQ_API_KEY"), LlmProviderType::FreeApi),
        ]);

        // LLM7 keyless
        entries.push(UnifiedModelEntry::cloud_free(
            "llm7", "codestral-latest", "Codestral Latest (LLM7)",
            "https://api.llm7.io/v1", "t2-capable", false, None, LlmProviderType::Llm7,
        ));

        // ApiAirforce keyless
        let airforce_base = "https://api.airforce/v1";
        entries.extend([
            UnifiedModelEntry::cloud_free("api-airforce", "grok-4.1-mini:free", "Grok 4.1 Mini (ApiAirforce)", airforce_base, "t4-frontier", false, None, LlmProviderType::ApiAirforce),
            UnifiedModelEntry::cloud_free("api-airforce", "deepseek-v3.2:free", "DeepSeek V3.2 (ApiAirforce)", airforce_base, "t4-frontier", false, None, LlmProviderType::ApiAirforce),
            UnifiedModelEntry::cloud_free("api-airforce", "gemma3-270m:free", "Gemma 3 270M (ApiAirforce)", airforce_base, "t0-cheap", false, None, LlmProviderType::ApiAirforce),
        ]);

        // OpenCode Zen 免费
        entries.extend([
            UnifiedModelEntry::cloud_free("opencode-zen", "mimo-v2.5-free", "MiMo V2.5 Free (OpenCode)", "https://opencode.ai/zen/v1", "t4-frontier", false, None, LlmProviderType::OpenCodeZen),
            UnifiedModelEntry::cloud_free("opencode-zen", "deepseek-v4-flash-free", "DeepSeek V4 Flash Free (OpenCode)", "https://opencode.ai/zen/v1", "t3-powerful", false, None, LlmProviderType::OpenCodeZen),
        ]);

        entries
    }
}

// ═══════════════════════════════════════════════════════════
// LocalEndpointSource — 本地运行端点探测
// ═══════════════════════════════════════════════════════════

pub struct LocalEndpointSource {
    endpoints: Vec<(String, String, String)>, // (id, url, name)
}

impl LocalEndpointSource {
    pub fn new() -> Self {
        Self {
            endpoints: vec![
                ("llama-cpp".into(), "http://localhost:8080/v1".into(), "llama.cpp".into()),
                ("ollama".into(), "http://localhost:11434/v1".into(), "Ollama".into()),
            ],
        }
    }

    fn probe_endpoint(url: &str) -> bool {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .ok()
            .and_then(|c| c.get(url).send().ok())
            .map(|r| r.status().is_success() || r.status().as_u16() == 404 || r.status().as_u16() == 405)
            .unwrap_or(false)
    }
}

impl ModelSource for LocalEndpointSource {
    fn name(&self) -> &str { "local_endpoint" }

    fn category(&self) -> ProviderCategory { ProviderCategory::Local }

    fn discover(&self) -> Vec<UnifiedModelEntry> {
        let mut entries = Vec::new();
        for (id, url, name) in &self.endpoints {
            if Self::probe_endpoint(url) {
                entries.push(UnifiedModelEntry {
                    id: format!("{}/default", id),
                    display_name: format!("{} (Local)", name),
                    category: ProviderCategory::Local,
                    source: "local_endpoint".into(),
                    base_url: url.clone(),
                    model_id: "default".into(),
                    size_gb: 0.0,
                    tier: "t1-standard".into(),
                    is_free: true,
                    requires_api_key: false,
                    api_key_env: None,
                    provider_type: LlmProviderType::OpenAI,
                    mmproj_path: None,
                });
            }
        }
        entries
    }
}

// ═══════════════════════════════════════════════════════════
// UnifiedModelPool — 统一模型池
// ═══════════════════════════════════════════════════════════

pub struct UnifiedModelPool {
    sources: Vec<Box<dyn ModelSource>>,
    cache: std::sync::RwLock<Vec<UnifiedModelEntry>>,
}

impl UnifiedModelPool {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            cache: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// 创建默认池 (包含所有内置源)
    pub fn default_pool() -> Self {
        let mut pool = Self::new();
        pool.add_source(Box::new(LocalGgufSource::default_m5()));
        pool.add_source(Box::new(CloudFreeSource::new()));
        pool.add_source(Box::new(LocalEndpointSource::new()));
        pool
    }

    /// 添加模型源插件
    pub fn add_source(&mut self, source: Box<dyn ModelSource>) {
        self.sources.push(source);
    }

    /// 刷新所有源, 返回合并后的模型列表
    pub fn refresh(&self) -> Vec<UnifiedModelEntry> {
        let mut all = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for source in &self.sources {
            for entry in source.discover() {
                // 去重: 同一 id 只保留第一个 (源优先级由 add_source 顺序决定)
                if seen.insert(entry.id.clone()) {
                    all.push(entry);
                }
            }
        }

        // 写入缓存
        if let Ok(mut cache) = self.cache.write() {
            *cache = all.clone();
        }

        all
    }

    /// 获取缓存的模型列表 (不触发刷新)
    pub fn list(&self) -> Vec<UnifiedModelEntry> {
        match self.cache.read() {
            Ok(guard) => guard.clone(),
            Err(_) => Vec::new(),
        }
    }

    /// 按分类获取模型
    pub fn list_by_category(&self, category: ProviderCategory) -> Vec<UnifiedModelEntry> {
        self.list().into_iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// 按 tier 获取模型
    pub fn list_by_tier(&self, tier: &str) -> Vec<UnifiedModelEntry> {
        self.list().into_iter()
            .filter(|e| e.tier == tier)
            .collect()
    }

    /// 获取所有免费模型 (本地 + keyless 云端)
    pub fn free_models(&self) -> Vec<UnifiedModelEntry> {
        self.list().into_iter()
            .filter(|e| e.is_free)
            .collect()
    }

    /// 按 ID 查找模型
    pub fn find_by_id(&self, id: &str) -> Option<UnifiedModelEntry> {
        self.list().into_iter().find(|e| e.id == id)
    }

    /// 格式化显示所有模型
    pub fn format_list(&self) -> String {
        let models = self.list();
        let mut output = format!("╭─ Unified Model Pool ({}) ──────────────────────╮\n", models.len());

        let mut by_category: std::collections::HashMap<String, Vec<&UnifiedModelEntry>> = std::collections::HashMap::new();
        for m in &models {
            let key = match m.category {
                ProviderCategory::Local => "Local (本地)",
                ProviderCategory::Cloud => "Cloud (云端)",
                ProviderCategory::Proxy => "Proxy (代理)",
            };
            by_category.entry(key.into()).or_default().push(m);
        }

        for (cat, entries) in &by_category {
            output.push_str(&format!("  {}:\n", cat));
            for m in entries {
                let free_tag = if m.is_free { "🆓" } else { "🔑" };
                let size = if m.size_gb > 0.0 { format!(" ({:.1}GB)", m.size_gb) } else { String::new() };
                output.push_str(&format!("    {} {}/{}{}\n", free_tag, m.source, m.model_id, size));
            }
        }

        output.push_str("╰─────────────────────────────────────────────────╯\n");
        output
    }
}

impl Default for UnifiedModelPool {
    fn default() -> Self {
        Self::default_pool()
    }
}

// ═══════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_gguf_source_discovers_models() {
        let source = LocalGgufSource::default_m5();
        let models = source.discover();
        // 应该能找到 Gemma 4 E2B
        assert!(models.iter().any(|m| m.model_id.contains("Gemma-4-E2B")),
            "should discover Gemma 4 E2B model");
    }

    #[test]
    fn test_unified_pool_default() {
        let pool = UnifiedModelPool::default_pool();
        let models = pool.refresh();
        // 至少有本地 GGUF 模型
        assert!(!models.is_empty(), "pool should have at least one model");
    }

    #[test]
    fn test_size_tier_classification() {
        assert_eq!(classify_size_tier(1.0), "t0-cheap");
        assert_eq!(classify_size_tier(3.0), "t1-standard");
        assert_eq!(classify_size_tier(7.0), "t2-balanced");
        assert_eq!(classify_size_tier(12.0), "t3-powerful");
    }
}

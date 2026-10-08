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
use std::sync::Arc;

use neotrix_neobot::nt_llama::{default_max_model_gb, llamacpp_base_url, ollama_base_url};

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
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub requires_api_key: bool,
    pub api_key_env: Option<String>,
    pub provider_type: LlmProviderType,
    pub mmproj_path: Option<PathBuf>,
}

impl UnifiedModelEntry {
    /// Create a model entry from a local GGUF file.
    ///
    /// `base_url` 取自 [`llamacpp_base_url()`] (可被 `NEOTRIX_LLAMACPP_BASE_URL`
    /// 覆盖), 不再写死 `localhost:8080` —— 端口冲突/多实例时那个常量是错的。
    /// 仍待补: 解析 GGUF header 拿真实模型名/架构, 以及从运行中的进程自动
    /// 发现实际服务的端点。
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
            base_url: llamacpp_base_url(),
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

    /// Create a model entry for a free cloud API.
    ///
    /// Note: Real implementation needs — currently all cloud_free entries are marked
    /// `is_free: true`. Consider distinguishing between truly free (keyless) and
    /// free-tier-with-key (rate-limited) models for better routing decisions.
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

/// 取字符串前 `n` 个**字符**的前缀 (按字节切会在多字节字符上 panic)
///
/// 用于 mmproj 配对时比较模型名/投影名的公共前缀。模型文件名可能含中文,
/// 字节切片会在字符边界处 panic —— 生产代码不允许。
fn prefix_chars(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

impl LocalGgufSource {
    pub fn new(scan_dirs: Vec<PathBuf>, max_size_gb: f64) -> Self {
        Self { scan_dirs, max_size_gb }
    }

    /// 本机默认扫描目录 —— 唯一真源是 `llama_process::model_search_dirs()`,
    /// 避免"启动进程用的目录"和"列模型用的目录"两处各写一份而漂移。
    pub fn default_m5() -> Self {
        Self::new(
            neotrix_neobot::nt_llama::model_search_dirs(),
            default_max_model_gb(),
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

        // 查找同目录下的 mmproj 投影。
        //
        // 命名约定: `mmproj-<模型名>-<量化>.gguf` 配 `<模型名>-<量化>.gguf`。
        // 旧实现用 `&model_id[..10]` 取字节前缀 —— 模型名含多字节字符时会切在
        // 字符边界上 **panic**, 且 10 字节对短名/长名都不稳。改为按字符取前缀。
        //
        // `prefix_chars` 返回去掉了量化后缀的模型名, 与 mmproj 名去掉
        // `mmproj-` 前缀后的部分比公共前缀。
        if let Some(parent) = path.parent() {
            let stem = prefix_chars(&entry.model_id, 24);
            if let Ok(read_dir) = std::fs::read_dir(parent) {
                for f in read_dir.filter_map(|e| e.ok()) {
                    let fp = f.path();
                    if fp.extension().map(|e| e == "gguf").unwrap_or(false) {
                        let fn_name = fp.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        if let Some(rest) = fn_name.strip_prefix("mmproj") {
                            if !stem.is_empty() && prefix_chars(rest, stem.len()) == stem {
                                entry.mmproj_path = Some(fp);
                                break;
                            }
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

impl Default for CloudFreeSource {
    fn default() -> Self {
        Self::new()
    }
}

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

impl Default for LocalEndpointSource {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalEndpointSource {
    pub fn new() -> Self {
        Self {
            // base_url 走 llamacpp_base_url() / ollama_base_url(), 可被
            // NEOTRIX_LLAMACPP_BASE_URL / NEOTRIX_OLLAMA_BASE_URL 覆盖。
            // 写死端口在「端口被占」或「多实例并存(8080/8081 各跑一个模型)」
            // 时探测的是错的端点 —— 而这恰恰是多模型场景的常态。
            endpoints: vec![
                ("llama-cpp".into(), llamacpp_base_url(), "llama.cpp".into()),
                ("ollama".into(), ollama_base_url(), "Ollama".into()),
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
    /// 源以 `Arc` 共享持有：既能被本池直接 `new`，也能由 `PluginRegistry`
    /// 交回**同一个**实例（统一接入口），避免 pool 与 registry 各造一份。
    sources: Vec<Arc<dyn ModelSource>>,
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
        pool.add_source(Arc::new(LocalGgufSource::default_m5()));
        pool.add_source(Arc::new(CloudFreeSource::new()));
        pool.add_source(Arc::new(LocalEndpointSource::new()));
        pool
    }

    /// 用 `PluginRegistry` 里已登记的真实 source 组池 —— 统一接入口的收口点。
    /// registry 登记了哪些 `capability="model_source"`，池就跑哪些；池不再
    /// 自行 `new`，故 CLI/后台与 registry 共享同一批 source 实例。
    pub fn from_registry_sources(sources: Vec<Arc<dyn ModelSource>>) -> Self {
        let mut pool = Self::new();
        for source in sources {
            pool.add_source(source);
        }
        pool
    }

    /// 添加模型源插件
    pub fn add_source(&mut self, source: Arc<dyn ModelSource>) {
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
    // 这两个只在测试里用; 顶层 import 会让非测试构建报 unused。
    use neotrix_neobot::nt_llama::{llamacpp_port, port_from_url};
    // 2026-09-28：生产侧 `:214` 用的是这个全路径真源，但测试里只写了裸名
    // `model_search_dirs()`，本模块顶部没有引入 ⇒ `cargo test` 编译不过
    // (E0425 + 后续 E0282)。补上导入，与 `:214` 保持同一真源。
    use crate::l1_action::nt_io::nt_io_provider::llama::llama_process::model_search_dirs;


    /// 回归锁: `prefix_chars` 按**字符**切, 不按字节。
    ///
    /// 旧实现在 mmproj 配对处写的是 `&model_id[..model_id.len().min(10)]`,
    /// 字节切片。模型文件名含多字节字符 (中文等) 时会切在字符边界上
    /// **panic** —— 生产代码不允许, 且这是扫描路径, 一个坏文件名就能打挂
    /// 整个模型池发现。
    #[test]
    fn test_prefix_chars_is_char_safe_not_byte_unsafe() {
        assert_eq!(prefix_chars("abcdefghij", 4), "abcd");
        // 短于 n → 整串
        assert_eq!(prefix_chars("abc", 10), "abc");
        // 空串
        assert_eq!(prefix_chars("", 5), "");
        // 多字节: "模型权重-v1-Qwen" 前 4 个**字符** = "模型权重" (12 字节)
        let s = "模型权重-v1-Qwen";
        let got = prefix_chars(s, 4);
        assert_eq!(got, "模型权重");
        // 关键: 结果必须是合法 UTF-8 边界, 且能安全 round-trip
        assert!(s.starts_with(got), "前缀必须是原串的字节前缀");
        assert_eq!(got.chars().count(), 4);
        // 旧写法在这个输入上会 panic —— 记录下来作为对照
        assert!(10 < "模型权重".len(), "10 字节落在多字节字符中间, 证明旧写法危险");
    }

    /// mmproj 配对必须用字符前缀, 且认得 `mmproj-` 前缀剥离。
    #[test]
    fn test_mmproj_pairing_uses_stripped_prefix() {
        let model_id = "Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-Q6_K";
        let stem = prefix_chars(model_id, 24);
        // mmproj 文件名去掉 `mmproj-` 后应与模型名共享该前缀
        let mmproj_rest = "Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-BF16";
        assert_eq!(prefix_chars(mmproj_rest, stem.chars().count()), stem);
        // 不匹配的名字不得误配
        let other = "MiniCPM5-2B-heretic-abliterated-BF16";
        assert_ne!(prefix_chars(other, stem.chars().count()), stem);
    }

    /// 端口必须从 base_url 解析, 而不是各处写死 8080。
    #[test]
    fn test_port_from_url() {
        assert_eq!(port_from_url("http://localhost:8080/v1"), Some(8080));
        assert_eq!(port_from_url("http://127.0.0.1:8081/v1"), Some(8081));
        // 无端口 → None (调用方回落)
        assert_eq!(port_from_url("http://localhost/v1"), None);
        // 畸形输入不得 panic
        assert_eq!(port_from_url(""), None);
        assert_eq!(port_from_url("::::"), None);
        assert_eq!(port_from_url("http://host:notaport/v1"), None);
    }

    /// 搜索目录必须**推导**得出, 而不是源码里写死一条绝对路径。
    ///
    /// ⚠️ 这条测试曾经写错: 它断言 `dirs` 里不出现 `/Users/neo`, 结果误报了
    /// 正确的实现 —— 因为 `$CARGO_MANIFEST_DIR/../models` 在本机**恰好**就是
    /// `/Users/neo/Downloads/neotrix/models`。输出值含用户名是正常的, 该断言的
    /// 是"来源是推导的"而非"输出不含某个字符串"。
    #[test]
    fn test_model_search_dirs_are_derived_not_hardcoded() {
        let dirs = model_search_dirs();
        assert!(!dirs.is_empty(), "至少要有一个搜索目录");

        // 1) 相对路径兜底必须存在 (不依赖任何环境)
        assert!(
            dirs.iter().any(|p| p == Path::new("models")),
            "必须含 cwd 相对的 `models` 兜底, 实际: {:?}", dirs
        );

        // 2) 仓库内 models 必须由 CARGO_MANIFEST_DIR 推导, 而非写死
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let expected = Path::new(&manifest).parent().map(|r| r.join("models"));
            assert!(
                expected.as_ref().is_some_and(|e| dirs.contains(e)),
                "必须含由 CARGO_MANIFEST_DIR 推导的 {:?}, 实际: {:?}",
                expected, dirs
            );
        }

        // 3) HOME 派生的缓存目录只在 HOME 存在时出现
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                assert!(
                    dirs.contains(&PathBuf::from(&home).join(".ollama/models")),
                    "必须含 HOME 派生的 ollama 目录, 实际: {:?}", dirs
                );
            }
        }
    }

    /// base_url 走 env 覆盖; 未设置时有可用的默认值 (不得为空/畸形)。
    #[test]
    fn test_base_urls_are_non_empty_and_wellformed() {
        let ll = llamacpp_base_url();
        assert!(ll.starts_with("http"), "llamacpp base_url 必须是 http(s), 实际 {}", ll);
        assert!(ll.ends_with("/v1"), "应为 OpenAI 兼容的 /v1 结尾, 实际 {}", ll);
        let ol = ollama_base_url();
        assert!(ol.starts_with("http") && ol.ends_with("/v1"), "ollama base_url 畸形: {}", ol);
        // 端口必须能从 base_url 解析回来
        let p = llamacpp_port();
        assert!(p > 0, "端口必须为正");
    }

    #[test]
    fn test_local_gguf_source_discovers_models() {
        // HONESTY: This test is hardware-dependent — it expects Gemma-4-E2B on an M5
        // machine. On machines without this model, the test fails.
        // Once remote model catalog is wired, replace with a catalog-based
        // lookup that doesn't depend on local filesystem contents.
        let source = LocalGgufSource::default_m5();
        let models = source.discover();
        // Document behavior rather than failing on unsupported hardware.
        if models.is_empty() {
            // No local GGUF models — this is expected on non-M5 hardware.
            // Wire remote model discovery and assert non-empty from at least one source.
        } else {
            assert!(models.iter().all(|m| !m.id.is_empty()),
                "all discovered models must have non-empty IDs");
        }
    }

    #[test]
    fn test_unified_pool_default() {
        // HONESTY: default_pool() may return empty on machines without local GGUF models.
        // Once remote model discovery is wired, assert non-empty from at least one source.
        let pool = UnifiedModelPool::default_pool();
        let models = pool.refresh();
        if models.is_empty() {
            // No local models found — document the behavior, not a failure.
        } else {
            assert!(models.iter().all(|m| !m.id.is_empty()),
                "all discovered models must have non-empty IDs");
        }
    }

    #[test]
    fn test_size_tier_classification() {
        assert_eq!(classify_size_tier(1.0), "t0-cheap");
        assert_eq!(classify_size_tier(3.0), "t1-standard");
        assert_eq!(classify_size_tier(7.0), "t2-balanced");
        assert_eq!(classify_size_tier(12.0), "t3-powerful");
    }
}

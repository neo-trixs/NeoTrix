//! `nt_provider` — 第三方模型端点注册表 + 模型池聚合.
//!
//! 对接 neotrix 模型池的方式：neotrix serve 即 OpenAI 兼容端点
//! （默认 `http://127.0.0.1:3000/v1`），在此注册为一个 provider 即可
//! （`neobot provider preset neotrix` 一键下沉）。
//!
//! 安全律（沿 nt_http_engine）：key 只从环境变量读，库里只存
//! **变量名**（`key_env`），永不落明文密钥；空 `key_env` = 免 key
//! （本地 Ollama/LM Studio 位）。

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;

/// 第三方端点（`providers` 表 1:1）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub base_url: String,
    pub key_env: String,
    pub model: String,
    pub enabled: bool,
}

/// neotrix 晶体核心默认模型名（桌面端默认选中；serve 端按其池子解析，
/// 若池子无此 id 则在模型面板里点选实际 id 即可）。
pub const NEOTRIX_CORE_MODEL: &str = "neotrix-crystal";

/// 知名端点预设：(name, base_url, key_env, model)。
/// key_env 是环境变量名（密钥永不落库）；model 空=跑时指定。
pub const PRESETS: &[(&str, &str, &str, &str)] = &[
    ("neotrix", "http://127.0.0.1:3000/v1", "NEOBOT_API_KEY", NEOTRIX_CORE_MODEL),
    ("ollama", "http://127.0.0.1:11434/v1", "", ""),
    ("lmstudio", "http://127.0.0.1:1234/v1", "", ""),
    ("deepseek", "https://api.deepseek.com/v1", "DEEPSEEK_API_KEY", "deepseek-chat"),
    ("openai", "https://api.openai.com/v1", "OPENAI_API_KEY", "gpt-4o-mini"),
    // 注：anthropic 原生接口非 OpenAI 兼容（x-api-key + /messages），本引擎只说
    // OpenAI 协议，故不收录 anthropic 预设（可用其兼容网关自建端点接入）。
    ("gemini", "https://generativelanguage.googleapis.com/v1beta/openai", "GEMINI_API_KEY", ""),
    ("qwen", "https://dashscope.aliyuncs.com/compatible-mode/v1", "QWEN_API_KEY", "qwen-plus"),
    ("moonshot", "https://api.moonshot.cn/v1", "MOONSHOT_API_KEY", "moonshot-v1-8k"),
    ("zhipu", "https://open.bigmodel.cn/api/paas/v4", "ZHIPU_API_KEY", "glm-4-flash"),
];

/// 聚合后的池子行（带来源）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolModel {
    pub provider: String,
    pub id: String,
    pub owner: String,
}

impl Provider {
    /// `key_env` 必须是合法环境变量名（`[A-Za-z_][A-Za-z0-9_]*`），否则极可能是
    /// 把明文 key 粘进了该字段（2026-09-26 nvapi 事故：key 落库 + `provider list` 明文回显，
    /// 而库文件是 644）。命中即视为密钥泄露形态。
    pub fn looks_like_secret(s: &str) -> bool {
        let t = s.trim();
        if t.is_empty() || t.len() > 128 {
            return !t.is_empty();
        }
        let mut chars = t.chars();
        let first_ok = chars
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false);
        // 合法变量名才不是 secret；其余一律按泄露形态处理。
        !(first_ok && chars.all(|c| c.is_ascii_alphanumeric() || c == '_'))
    }

    pub fn validate(&self) -> Result<(), NtBotError> {
        if self.name.trim().is_empty() {
            return Err(NtBotError::Invalid("provider name is empty".to_owned()));
        }
        if self.name.trim().len() > 64 {
            return Err(NtBotError::Invalid("provider name exceeds 64 chars".to_owned()));
        }
        let base = self.base_url.trim().trim_end_matches('/');
        if !(base.starts_with("http://") || base.starts_with("https://")) {
            return Err(NtBotError::Invalid(format!(
                "base_url must start with http(s)://, got '{}'",
                self.base_url
            )));
        }
        // 入库门禁：key_env 存明文 key 直接拒绝（nvapi 事故复发门）。
        // 修法：export 名字=<key> 后重填变量名。
        if Self::looks_like_secret(&self.key_env) {
            return Err(NtBotError::Invalid(
                "key_env looks like a plaintext key, not a variable name (refused: key 永不落盘)；export <NAME>=<key> 后填变量名".to_owned(),
            ));
        }
        Ok(())
    }

    /// 环境读 key（空 key_env = 免 key，返回空串）。
    pub fn read_key(&self) -> String {
        if self.key_env.trim().is_empty() {
            return String::new();
        }
        std::env::var(self.key_env.trim()).unwrap_or_default()
    }

    fn models_url(&self) -> String {
        format!("{}/models", self.base_url.trim().trim_end_matches('/'))
    }

    /// 模型 id 入池校验（P0 审计 F3）：id 来自远端 `/models`，
    /// 非法字符（引号/尖括号/空白等）直接丢弃，不让脏串流进前端属性与请求路径。
    /// fallback 行（已配置 model 但 /models 不可达）复用同一门。
    pub fn valid_model_id(id: &str) -> bool {
        const MAX_LEN: usize = 128;
        !id.is_empty()
            && id.len() <= MAX_LEN
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':' | '/'))
    }

    /// 拉该端点模型列表（`GET /v1/models`，owner 缺失回落 id 前缀）。
    pub fn list_models(&self, timeout_secs: u64) -> Result<Vec<PoolModel>, NtBotError> {
        let timeout = Duration::from_secs(timeout_secs.clamp(1, 15));
        let mut request = ureq::get(&self.models_url()).timeout(timeout);
        let key = self.read_key();
        if !key.trim().is_empty() {
            request = request.set("Authorization", &format!("Bearer {}", key.trim()));
        }
        let value = request
            .call()
            .map_err(|err| NtBotError::Engine {
                engine: format!("pool:{}", self.name),
                reason: format!("{err}"),
            })?
            .into_json::<serde_json::Value>()
            .map_err(|err| NtBotError::Engine {
                engine: format!("pool:{}", self.name),
                reason: format!("bad /models json: {err}"),
            })?;
        let mut out = Vec::new();
        if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
            for entry in data {
                let Some(id) = entry.get("id").and_then(|v| v.as_str()) else {
                    continue;
                };
                if !Self::valid_model_id(id) {
                    continue;
                }
                let owner = entry
                    .get("owned_by")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        id.split_once('/')
                            .map(|(prefix, _)| prefix.to_owned())
                            .unwrap_or_else(|| self.name.clone())
                    });
                out.push(PoolModel {
                    provider: self.name.clone(),
                    id: id.to_owned(),
                    owner,
                });
            }
        }
        Ok(out)
    }

    /// 已配置 model 的 fallback 行（`/models` 不可达但配了 model 时不断档）。
    /// CLI `models` + 桌面 `neobot_models` 同律共用；`valid_model_id` 同门防脏串。
    pub fn fallback_row(&self) -> Option<PoolModel> {
        let id = self.model.trim().to_owned();
        if id.is_empty() || !Self::valid_model_id(&id) {
            return None;
        }
        Some(PoolModel {
            provider: self.name.clone(),
            id,
            owner: self.name.clone(),
        })
    }
}

/// 池子聚合（对话三端同律：`neobot models` CLI + 桌面 `neobot_models` +
/// `neotrix dialog models`）：启用中端点逐个 `/models` 拉取，失败则 fallback 行；
/// 按 (provider, id) 排序。禁用的端点不出池。
/// 返回（模型行，不可达端点名）：调用方按需提示（CLI 打印，桌面静默）。
pub fn pool_models(store: &crate::NeobotStore) -> (Vec<PoolModel>, Vec<String>) {
    let mut out = Vec::new();
    let mut unreachable = Vec::new();
    let Ok(providers) = store.list_providers() else {
        return (out, unreachable);
    };
    for provider in providers {
        if !provider.enabled {
            continue;
        }
        match provider.list_models(10) {
            Ok(models) => out.extend(models),
            Err(_) => {
                unreachable.push(provider.name.clone());
                if let Some(row) = provider.fallback_row() {
                    if !out.iter().any(|m| m.provider == row.provider && m.id == row.id) {
                        out.push(row);
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| (&a.provider, &a.id).cmp(&(&b.provider, &b.id)));
    (out, unreachable)
}

impl Provider {
    /// 按此端点组装 HTTP 引擎（`run --provider` 用；model 优先级：
    /// 显式覆盖 > provider.model > NEOBOT_MODEL）。
    /// 缺省三层全空时按缺省拒绝律直接报错，不猜模型。
    pub fn http_engine(
        &self,
        model_override: Option<&str>,
    ) -> Result<crate::nt_http_engine::HttpEngine, NtBotError> {
        let model = model_override
            .map(str::trim)
            .filter(|m| !m.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                if self.model.trim().is_empty() {
                    None
                } else {
                    Some(self.model.trim().to_owned())
                }
            })
            .or_else(|| {
                let env = std::env::var("NEOBOT_MODEL").unwrap_or_default();
                let trimmed = env.trim().to_owned();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
            .ok_or_else(|| {
                NtBotError::Invalid(format!(
                    "provider '{}' has no model (set one, --model, or NEOBOT_MODEL)",
                    self.name
                ))
            })?;
        let config = crate::nt_http_engine::HttpEngineConfig {
            base_url: self.base_url.trim().trim_end_matches('/').to_owned(),
            model,
            timeout_secs: crate::nt_http_engine::DEFAULT_TIMEOUT_SECS,
        };
        config.validate()?;
        crate::nt_http_engine::HttpEngine::new(config, self.read_key())
    }
}

#[cfg(test)]
mod tests {
    use super::{PRESETS, Provider};

    fn provider(name: &str, base: &str) -> Provider {
        Provider {
            name: name.to_owned(),
            base_url: base.to_owned(),
            key_env: String::new(),
            model: String::new(),
            enabled: true,
        }
    }

    #[test]
    fn validation_rejects_bad_input() {
        assert!(provider("", "http://x/v1").validate().is_err());
        assert!(provider("a", "ftp://x").validate().is_err());
        assert!(provider("ok", "https://x/v1/").validate().is_ok());
    }

    #[test]
    fn model_id_filter_blocks_xss_shapes() {
        // 审计 F3：远端 id 含引号/尖括号/空白即脏串。
        for bad in [
            "",
            "m\" onmouseover=\"x",
            "m'id",
            "<script>",
            "a b",
            "a\nb",
            "m;rm -rf /",
            &"x".repeat(129),
        ] {
            assert!(!Provider::valid_model_id(bad), "must reject: {bad:?}");
        }
        for good in ["gpt-4o", "org/model:v1", "neotrix-crystal", "a.b_c:d/e"] {
            assert!(Provider::valid_model_id(good), "must accept: {good}");
        }
    }

    #[test]
    fn presets_are_valid() {
        for (name, base, _, _) in PRESETS {
            assert!(provider(name, base).validate().is_ok(), "{name}");
        }
    }

    #[test]
    fn empty_key_env_means_keyless() {
        assert_eq!(provider("o", "http://x/v1").read_key(), "");
    }

    #[test]
    fn fallback_row_needs_configured_valid_model() {
        // 空 model → 无行；脏串 → 无行；正常 → 行（provider 作 source/owner）。
        assert!(provider("p", "https://x/v1").fallback_row().is_none());
        let mut dirty = provider("p", "https://x/v1");
        dirty.model = "a b".to_owned();
        assert!(dirty.fallback_row().is_none());
        let mut ok = provider("p", "https://x/v1");
        ok.model = "m-1".to_owned();
        let row = ok.fallback_row().unwrap();
        assert_eq!(row.id, "m-1");
        assert_eq!(row.provider, "p");
    }

    #[test]
    fn secret_shaped_key_env_is_rejected() {
        // 2026-09-26 nvapi 事故：明文 key 进 key_env 必须入库即拒。
        assert!(!Provider::looks_like_secret(""));
        assert!(!Provider::looks_like_secret("NVAPI_KEY"));
        assert!(!Provider::looks_like_secret("_k1"));
        assert!(Provider::looks_like_secret("nvapi-abc123XYZ"));
        assert!(Provider::looks_like_secret("sk-foo"));
        assert!(Provider::looks_like_secret("has space"));
        assert!(Provider::looks_like_secret(&"x".repeat(129)));
        let mut p = provider("n", "https://x/v1");
        p.key_env = "nvapi-abc123".to_owned();
        assert!(p.validate().is_err());
        p.key_env = "NVAPI_KEY".to_owned();
        assert!(p.validate().is_ok());
    }
}

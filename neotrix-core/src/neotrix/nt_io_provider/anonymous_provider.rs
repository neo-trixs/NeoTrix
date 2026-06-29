use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rand::Rng;

use super::identity_council::{CallPlan, IdentityCouncil};
use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};
use crate::neotrix::nt_io_http_factory::{TlsFingerprint, TlsVariant};

/// An anonymous LLM provider that uses IdentityCouncil to rotate:
///   - API keys (round-robin across pools)
///   - TLS fingerprints (5 browser variants)
///   - TLS protocol variants (HTTP/2 vs HTTP/1.1, cert verification)
///   - Timing jitter (randomized pre-send delay)
///
/// Every call produces a unique {key, proxy, fingerprint, timing} identity.
/// Every outcome is reported back to the council for heat tracking.
pub struct AnonymousLlmProvider {
    council: Arc<IdentityCouncil>,
    base_url: String,
    model: String,
    provider_name: String,
    fallback_api_key: Option<String>,
}

impl AnonymousLlmProvider {
    pub fn new(
        council: Arc<IdentityCouncil>,
        base_url: &str,
        model: &str,
        provider_name: &str,
        fallback_api_key: Option<String>,
    ) -> Self {
        Self {
            council,
            base_url: base_url.to_string(),
            model: model.to_string(),
            provider_name: provider_name.to_string(),
            fallback_api_key,
        }
    }

    fn get_plan(&self) -> CallPlan {
        self.council
            .plan_call(&self.provider_name, &self.model)
            .unwrap_or_else(|| self.fallback_plan())
    }

    fn fallback_plan(&self) -> CallPlan {
        let mut rng = rand::thread_rng();
        let fps = [
            TlsFingerprint::Chrome120,
            TlsFingerprint::Firefox120,
            TlsFingerprint::Chrome116,
            TlsFingerprint::Safari17,
            TlsFingerprint::Edge120,
        ];
        let variants = [
            TlsVariant::ModernH2,
            TlsVariant::LegacyHttp11,
            TlsVariant::StrictVerify,
            TlsVariant::LegacyStrict,
        ];
        let jitter_base = 300u64;
        let jitter_range = 200u64;
        let offset = rng.gen_range(0..=jitter_range * 2);
        let jitter = (jitter_base + offset).saturating_sub(jitter_range);
        CallPlan {
            api_key: self.fallback_api_key.clone().unwrap_or_default(),
            proxy_url: None,
            tls_fingerprint: fps[rng.gen_range(0..5)],
            tls_variant: variants[rng.gen_range(0..4)],
            jitter_pre_send_ms: jitter,
            timeout_secs: 60,
        }
    }

    fn build_client(&self, plan: &CallPlan) -> Result<reqwest::Client, LlmError> {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(plan.timeout_secs))
            .connect_timeout(Duration::from_secs(10));

        builder = match plan.tls_variant {
            TlsVariant::ModernH2 => builder.danger_accept_invalid_certs(true),
            TlsVariant::LegacyHttp11 => {
                builder.http1_only().danger_accept_invalid_certs(true)
            }
            TlsVariant::StrictVerify => builder,
            TlsVariant::LegacyStrict => builder.http1_only(),
        };

        if let Some(ref proxy_url) = plan.proxy_url {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        builder
            .build()
            .map_err(|e| LlmError::Network(format!("client build: {}", e)))
    }

    fn build_json_body(&self, request: &LlmRequest) -> serde_json::Value {
        serde_json::json!({
            "model": self.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
        })
    }
}

#[async_trait]
impl LlmProvider for AnonymousLlmProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let plan = self.get_plan();

        if plan.jitter_pre_send_ms > 0 {
            let delay = Duration::from_millis(plan.jitter_pre_send_ms);
            tokio::time::sleep(delay).await;
        }

        let client = self.build_client(&plan)?;
        let url = format!("{}/chat/completions", self.base_url);
        let start = std::time::Instant::now();
        let body = self.build_json_body(request);

        let mut req_builder = client
            .post(&url)
            .header("Content-Type", "application/json");
        if !plan.api_key.is_empty() {
            let bearer = format!("Bearer {}", plan.api_key);
            req_builder = req_builder.header("Authorization", bearer);
        }

        let response = req_builder
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let status = response.status().as_u16();
        let latency = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            self.council
                .report_outcome(&self.provider_name, &plan, false, status, latency);
            return match status {
                429 => Err(LlmError::RateLimit(error_text)),
                401 | 403 => Err(LlmError::Authentication(error_text)),
                500..=599 => Err(LlmError::Server(format!("HTTP {}: {}", status, error_text))),
                _ => Err(LlmError::Unknown(format!("HTTP {}: {}", status, error_text))),
            };
        }

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

        let content = resp_json["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let usage = resp_json
            .get("usage")
            .map(|u| Usage {
                prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            })
            .unwrap_or_default();

        let finish_reason = resp_json["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("finish_reason"))
            .and_then(|f| f.as_str())
            .map(|f| match f {
                "stop" => FinishReason::Stop,
                "length" => FinishReason::Length,
                "tool_calls" => FinishReason::Tool,
                "content_filter" => FinishReason::ContentFilter,
                _ => FinishReason::Unknown,
            })
            .unwrap_or(FinishReason::Unknown);

        self.council
            .report_outcome(&self.provider_name, &plan, true, status, latency);

        Ok(LlmResponse {
            content,
            model: request.model.clone(),
            usage,
            finish_reason,
        })
    }

    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let plan = self.get_plan();

        if plan.jitter_pre_send_ms > 0 {
            tokio::time::sleep(Duration::from_millis(plan.jitter_pre_send_ms)).await;
        }

        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(plan.timeout_secs * 2))
            .connect_timeout(Duration::from_secs(10));

        builder = match plan.tls_variant {
            TlsVariant::ModernH2 => builder.danger_accept_invalid_certs(true),
            TlsVariant::LegacyHttp11 => {
                builder.http1_only().danger_accept_invalid_certs(true)
            }
            TlsVariant::StrictVerify => builder,
            TlsVariant::LegacyStrict => builder.http1_only(),
        };

        if let Some(ref proxy_url) = plan.proxy_url {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        let client = builder
            .build()
            .map_err(|e| LlmError::Network(format!("client build: {}", e)))?;

        let url = format!("{}/chat/completions", self.base_url);
        let start = std::time::Instant::now();
        let mut body = self.build_json_body(request);
        body["stream"] = serde_json::json!(true);

        let mut req_builder = client
            .post(&url)
            .header("Content-Type", "application/json");
        if !plan.api_key.is_empty() {
            let bearer = format!("Bearer {}", plan.api_key);
            req_builder = req_builder.header("Authorization", bearer);
        }

        let response = req_builder
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let status = response.status().as_u16();
        let latency = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            self.council
                .report_outcome(&self.provider_name, &plan, false, status, latency);
            return match status {
                429 => Err(LlmError::RateLimit(error_text)),
                401 | 403 => Err(LlmError::Authentication(error_text)),
                _ => Err(LlmError::Server(format!("HTTP {}: {}", status, error_text))),
            };
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let provider_name = self.provider_name.clone();
        let plan_clone = plan.clone();
        let council_clone = self.council.clone();
        let model_name = request.model.clone();

        tokio::spawn(async move {
            use futures_util::StreamExt;

            let mut full_content = String::new();
            let mut stream = response.bytes_stream();
            let mut buf = String::new();
            let mut had_data = false;

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx
                            .send(Err(LlmError::Network(e.to_string())))
                            .await;
                        break;
                    }
                };

                let text = String::from_utf8_lossy(&chunk);
                buf.push_str(&text);

                while let Some(line_end) = buf.find('\n') {
                    let line = buf[..line_end].trim().to_string();
                    buf = buf[line_end + 1..].to_string();

                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(val) =
                            serde_json::from_str::<serde_json::Value>(data)
                        {
                            if let Some(delta) = val["choices"]
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|v| v.get("delta"))
                                .and_then(|d| d.get("content"))
                                .and_then(|c| c.as_str())
                            {
                                had_data = true;
                                full_content.push_str(delta);
                                let _ = tx
                                    .send(Ok(LlmResponse {
                                        content: delta.to_string(),
                                        model: model_name.clone(),
                                        usage: Usage::default(),
                                        finish_reason: FinishReason::Stop,
                                    }))
                                    .await;
                            }
                        }
                    }
                }
            }

            let success = had_data || !full_content.is_empty();
            council_clone.report_outcome(
                &provider_name,
                &plan_clone,
                success,
                status,
                latency,
            );
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn test_council() -> Arc<IdentityCouncil> {
        Arc::new(IdentityCouncil::new())
    }

    #[test]
    fn test_fallback_plan_no_crash() {
        let council = test_council();
        let provider = AnonymousLlmProvider::new(
            council.clone(),
            "https://api.example.com/v1",
            "test-model",
            "nonexistent",
            Some("sk-fallback".into()),
        );
        let plan = provider.fallback_plan();
        assert_eq!(plan.api_key, "sk-fallback");
        assert!(plan.jitter_pre_send_ms <= 500);
    }

    #[test]
    fn test_fallback_plan_no_key() {
        let council = test_council();
        let provider = AnonymousLlmProvider::new(
            council.clone(),
            "https://api.example.com/v1",
            "test-model",
            "nonexistent",
            None,
        );
        let plan = provider.get_plan();
        // With no key pool and no fallback, api_key should be empty
        assert_eq!(plan.api_key, "");
    }

    #[test]
    fn test_build_json_body() {
        let council = test_council();
        let provider = AnonymousLlmProvider::new(
            council.clone(),
            "https://api.example.com/v1",
            "test-model",
            "test",
            None,
        );
        let req = LlmRequest::new("test-model", "hello");
        let body = provider.build_json_body(&req);
        assert_eq!(body["model"], "test-model");
        assert_eq!(body["temperature"], 0.7);
        assert_eq!(body["max_tokens"], 4096);
    }

    #[test]
    fn test_build_client_sanity() {
        let council = test_council();
        let provider = AnonymousLlmProvider::new(
            council.clone(),
            "https://api.example.com/v1",
            "test-model",
            "test",
            None,
        );
        let plan = CallPlan {
            api_key: "sk-test".into(),
            proxy_url: None,
            tls_fingerprint: TlsFingerprint::Chrome120,
            tls_variant: TlsVariant::ModernH2,
            jitter_pre_send_ms: 0,
            timeout_secs: 30,
        };
        let client = provider.build_client(&plan);
        assert!(client.is_ok());
    }
}

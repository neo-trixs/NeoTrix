use std::collections::HashMap;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::agent::tool::lifecycle::*;

// ─── MiniMax API Types ───────────────────────────────────────────────────────

#[derive(Serialize)]
struct MiniMaxT2IRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MiniMaxT2IResponse {
    #[serde(default)]
    data: Vec<MiniMaxImageData>,
    #[serde(default)]
    #[allow(dead_code)]
    base_resp: Option<MiniMaxBaseResp>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MiniMaxT2IResponseV2 {
    #[serde(default)]
    images: Vec<MiniMaxImageData>,
    #[serde(default)]
    #[allow(dead_code)]
    base_resp: Option<MiniMaxBaseResp>,
}

#[derive(Deserialize)]
struct MiniMaxImageData {
    #[serde(default)]
    url: String,
    #[serde(default)]
    image_url: String,
    #[serde(default)]
    b64: String,
    #[serde(default)]
    base64: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MiniMaxBaseResp {
    #[serde(default)]
    #[allow(dead_code)]
    status_code: i32,
    #[serde(default)]
    #[allow(dead_code)]
    status_msg: String,
}

// ─── API Call ────────────────────────────────────────────────────────────────

fn call_minimax_t2i(
    prompt: &str,
    width: u32,
    height: u32,
    n: u32,
    seed: Option<u64>,
) -> Result<String, String> {
    let api_host = std::env::var("MINIMAX_API_HOST")
        .unwrap_or_else(|_| "https://api.minimaxi.com".to_string());
    let api_key = std::env::var("MINIMAX_API_KEY").map_err(|_| {
        "MINIMAX_API_KEY not set. Set it via export MINIMAX_API_KEY=sk-...".to_string()
    })?;

    let url = format!("{}/v1/image/generation", api_host);
    let size = format!("{}x{}", width, height);

    let body = MiniMaxT2IRequest {
        model: "image-01".into(),
        prompt: prompt.to_string(),
        n: Some(n.min(4)),
        image_size: Some(size),
        seed,
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .map_err(|e| format!("MiniMax API request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("MiniMax API returned {status}: {text}"));
    }

    let text = resp
        .text()
        .map_err(|e| format!("Failed to read response: {e}"))?;

    // Try V1 format first: { data: [{ url, base64 }] }
    if let Ok(v1) = serde_json::from_str::<MiniMaxT2IResponse>(&text) {
        if let Some(img) = v1.data.into_iter().next() {
            let b64 = resolve_image_data(img, &text)?;
            return Ok(b64);
        }
    }

    // Try V2 format: { images: [{ image_url, base64 }] }
    if let Ok(v2) = serde_json::from_str::<MiniMaxT2IResponseV2>(&text) {
        if let Some(img) = v2.images.into_iter().next() {
            let b64 = resolve_image_data_v2(img, &text)?;
            return Ok(b64);
        }
    }

    Err(format!(
        "MiniMax API: unexpected response format: {text:.500}"
    ))
}

fn resolve_image_data(img: MiniMaxImageData, _raw: &str) -> Result<String, String> {
    if !img.base64.is_empty() {
        return Ok(img.base64);
    }
    if !img.b64.is_empty() {
        return Ok(img.b64);
    }
    if !img.url.is_empty() {
        return fetch_image_b64(&img.url);
    }
    if !img.image_url.is_empty() {
        return fetch_image_b64(&img.image_url);
    }
    Err("No image data in response".into())
}

fn resolve_image_data_v2(img: MiniMaxImageData, _raw: &str) -> Result<String, String> {
    if !img.base64.is_empty() {
        return Ok(img.base64);
    }
    if !img.b64.is_empty() {
        return Ok(img.b64);
    }
    if !img.image_url.is_empty() {
        return fetch_image_b64(&img.image_url);
    }
    if !img.url.is_empty() {
        return fetch_image_b64(&img.url);
    }
    Err("No image data in response".into())
}

fn fetch_image_b64(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let resp = client
        .get(url)
        .send()
        .map_err(|e| format!("Failed to fetch image from {url}: {e}"))?;

    let bytes = resp
        .bytes()
        .map_err(|e| format!("Failed to read image bytes: {e}"))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

// ─── Tool Implementation ─────────────────────────────────────────────────────

pub struct MiniMaxT2ITool {
    manifest: ToolManifest,
}

impl MiniMaxT2ITool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "minimax_t2i".into(),
                name: "MiniMax Text-to-Image".into(),
                version: "0.1.0".into(),
                permissions: vec![],
                mcp: None,
                min_runtime: "0.1.0".into(),
                description: "Generate photorealistic images via MiniMax image-01 API. "
                    .to_string() + "Parameters: prompt, width (max 2048), height (max 2048), n (1-4), seed (optional). "
                    + "Requires MINIMAX_API_KEY and MINIMAX_API_HOST environment variables. "
                    + "Output is PNG (base64-encoded).",
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for MiniMaxT2ITool {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTool for MiniMaxT2ITool {
    fn id(&self) -> &str {
        &self.manifest.id
    }
    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> {
        Ok(())
    }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value =
            serde_json::from_str(&ctx.input).map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e.to_string(),
            })?;

        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("abstract art");
        let width = args
            .get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024)
            .min(2048)
            .max(64) as u32;
        let height = args
            .get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024)
            .min(2048)
            .max(64) as u32;
        let n = args
            .get("n")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
            .min(4)
            .max(1) as u32;
        let seed = args.get("seed").and_then(|v| v.as_u64());

        let b64 =
            call_minimax_t2i(prompt, width, height, n, seed).map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e,
            })?;

        let result = serde_json::json!({
            "tool": "minimax_t2i",
            "format": "png",
            "width": width,
            "height": height,
            "model": "image-01",
            "prompt": prompt,
            "data": b64,
        });

        let mut meta = HashMap::new();
        meta.insert("format".into(), "png".into());
        meta.insert("width".into(), width.to_string());
        meta.insert("height".into(), height.to_string());
        meta.insert("model".into(), "image-01".into());
        meta.insert("prompt".into(), prompt.into());
        Ok(ToolOutput {
            result: serde_json::to_string_pretty(&result).unwrap_or_default(),
            metadata: meta,
        })
    }

    fn stop(&mut self) -> Result<(), ToolError> {
        Ok(())
    }
}

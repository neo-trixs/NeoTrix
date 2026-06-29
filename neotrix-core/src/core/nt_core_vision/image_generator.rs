use base64::Engine;
use std::collections::HashMap;

fn decode_b64(data_str: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(data_str)
        .map_err(|e| format!("Base64 decode failed: {}", e))
}

/// Text-to-Image generation backend. Abstraction over DALL-E, Stable Diffusion, ComfyUI, etc.
pub trait ImageGenerator: Send + Sync {
    fn generate(&self, prompt: &str, params: &GenParams) -> Result<GeneratedImage, String>;
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
}

/// Generation parameters (from xiaohu-ip-studio)
#[derive(Debug, Clone)]
pub struct GenParams {
    pub width: u32,
    pub height: u32,
    pub style_dna: Option<String>,
    pub character_ref: Option<CharacterRef>,
    pub negative_prompt: Option<String>,
    pub quality: GenQuality,
    pub seed: Option<u64>,
}

impl Default for GenParams {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            style_dna: None,
            character_ref: None,
            negative_prompt: None,
            quality: GenQuality::Standard,
            seed: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GenQuality {
    Draft,
    Standard,
    High,
}

#[derive(Debug, Clone)]
pub struct GeneratedImage {
    pub data: Vec<u8>,
    pub mime: String,
    pub seed: u64,
    pub model: String,
}

/// Reference image for character consistency
#[derive(Debug, Clone)]
pub struct CharacterRef {
    pub name: String,
    pub reference_b64: String,
    pub mime: String,
}

/// OpenAI DALL-E 3 implementation
pub struct DallE3Generator {
    api_key: String,
    model: String,
}

impl DallE3Generator {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: "dall-e-3".to_string(),
        }
    }

    fn build_request_body(
        &self,
        prompt: &str,
        params: &GenParams,
    ) -> Result<HashMap<String, serde_json::Value>, String> {
        let size = format!("{}x{}", params.width, params.height);
        let quality_str = match params.quality {
            GenQuality::Draft => "standard",
            GenQuality::Standard => "standard",
            GenQuality::High => "hd",
        };

        let mut body = HashMap::new();
        body.insert(
            "model".to_string(),
            serde_json::Value::String(self.model.clone()),
        );
        body.insert(
            "prompt".to_string(),
            serde_json::Value::String(prompt.to_string()),
        );
        body.insert(
            "n".to_string(),
            serde_json::Value::Number(serde_json::Number::from(1)),
        );
        body.insert("size".to_string(), serde_json::Value::String(size));
        body.insert(
            "quality".to_string(),
            serde_json::Value::String(quality_str.to_string()),
        );

        let mut final_prompt = prompt.to_string();
        if let Some(ref dna_id) = params.style_dna {
            final_prompt = format!("{}, style: {}", final_prompt, dna_id);
        }
        if let Some(ref cref) = params.character_ref {
            final_prompt = format!("{}, character: {}", final_prompt, cref.name);
        }
        body.insert(
            "prompt".to_string(),
            serde_json::Value::String(final_prompt),
        );

        if let Some(seed) = params.seed {
            body.insert(
                "seed".to_string(),
                serde_json::Value::Number(serde_json::Number::from(seed)),
            );
        }

        Ok(body)
    }
}

impl ImageGenerator for DallE3Generator {
    fn generate(&self, prompt: &str, params: &GenParams) -> Result<GeneratedImage, String> {
        let body = self.build_request_body(prompt, params)?;

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://api.openai.com/v1/images/generations")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("DALL-E request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(format!("DALL-E API error ({}): {}", status, text));
        }

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("DALL-E parse failed: {}", e))?;

        let url = json["data"][0]["url"]
            .as_str()
            .ok_or_else(|| "DALL-E response missing url".to_string())?
            .to_string();

        let img_resp = client
            .get(&url)
            .send()
            .map_err(|e| format!("DALL-E image download failed: {}", e))?;

        let data = img_resp
            .bytes()
            .map_err(|e| format!("DALL-E image read failed: {}", e))?
            .to_vec();

        let seed = params.seed.unwrap_or(0);

        Ok(GeneratedImage {
            data,
            mime: "image/png".to_string(),
            seed,
            model: self.model.clone(),
        })
    }

    fn name(&self) -> &str {
        "dall-e-3"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// ComfyUI/Stable Diffusion API backend
pub struct ComfyUIGenerator {
    endpoint: String,
    api_key: Option<String>,
}

impl ComfyUIGenerator {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }
}

impl ImageGenerator for ComfyUIGenerator {
    fn generate(&self, prompt: &str, params: &GenParams) -> Result<GeneratedImage, String> {
        let width = params.width.max(64).min(2048);
        let height = params.height.max(64).min(2048);
        let steps = match params.quality {
            GenQuality::Draft => 12,
            GenQuality::Standard => 24,
            GenQuality::High => 40,
        };

        let mut payload = HashMap::new();
        payload.insert(
            "prompt".to_string(),
            serde_json::Value::String(prompt.to_string()),
        );
        payload.insert(
            "width".to_string(),
            serde_json::Value::Number(serde_json::Number::from(width)),
        );
        payload.insert(
            "height".to_string(),
            serde_json::Value::Number(serde_json::Number::from(height)),
        );
        payload.insert(
            "steps".to_string(),
            serde_json::Value::Number(serde_json::Number::from(steps)),
        );

        if let Some(seed) = params.seed {
            payload.insert(
                "seed".to_string(),
                serde_json::Value::Number(serde_json::Number::from(seed)),
            );
        }
        if let Some(ref np) = params.negative_prompt {
            payload.insert(
                "negative_prompt".to_string(),
                serde_json::Value::String(np.clone()),
            );
        }

        let client = reqwest::blocking::Client::new();
        let mut req = client
            .post(format!("{}/generate", self.endpoint))
            .json(&payload);

        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req
            .send()
            .map_err(|e| format!("ComfyUI request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(format!("ComfyUI error ({}): {}", status, text));
        }

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("ComfyUI parse failed: {}", e))?;

        let data_str = json["image"]
            .as_str()
            .or_else(|| json["images"][0]["data"].as_str())
            .ok_or_else(|| "ComfyUI response missing image".to_string())?;

        let data = decode_b64(data_str)?;

        let seed = json["seed"].as_u64().or(params.seed).unwrap_or(0);

        Ok(GeneratedImage {
            data,
            mime: "image/png".to_string(),
            seed,
            model: "comfyui".to_string(),
        })
    }

    fn name(&self) -> &str {
        "comfyui"
    }

    fn is_available(&self) -> bool {
        !self.endpoint.is_empty()
    }
}

/// Mock generator for testing
pub struct MockImageGenerator;

impl ImageGenerator for MockImageGenerator {
    fn generate(&self, prompt: &str, params: &GenParams) -> Result<GeneratedImage, String> {
        let data = format!(
            "mock:{}:{}x{}:{:?}",
            prompt, params.width, params.height, params.quality
        )
        .into_bytes();
        Ok(GeneratedImage {
            data,
            mime: "image/png".to_string(),
            seed: params.seed.unwrap_or(42),
            model: "mock".to_string(),
        })
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_generator() {
        let gen = MockImageGenerator;
        assert!(gen.is_available());
        assert_eq!(gen.name(), "mock");

        let params = GenParams::default();
        let result = gen.generate("a cat", &params).unwrap();
        assert_eq!(result.model, "mock");
        assert_eq!(result.seed, 42);
        assert!(!result.data.is_empty());
    }

    #[test]
    fn test_gen_params_defaults() {
        let p = GenParams::default();
        assert_eq!(p.width, 1024);
        assert_eq!(p.height, 1024);
        assert!(p.style_dna.is_none());
        assert!(p.character_ref.is_none());
        assert!(p.seed.is_none());
    }

    #[test]
    fn test_gen_params_custom() {
        let p = GenParams {
            width: 512,
            height: 768,
            quality: GenQuality::High,
            seed: Some(12345),
            style_dna: Some("ghibli".to_string()),
            character_ref: None,
            negative_prompt: Some("ugly".to_string()),
        };
        assert_eq!(p.width, 512);
        assert_eq!(p.height, 768);
        assert_eq!(p.seed, Some(12345));
    }

    #[test]
    fn test_character_ref_construction() {
        let cr = CharacterRef {
            name: "hero".to_string(),
            reference_b64: "AAAA".to_string(),
            mime: "image/png".to_string(),
        };
        assert_eq!(cr.name, "hero");
        assert_eq!(cr.mime, "image/png");
    }

    #[test]
    fn test_comfyui_not_available_with_empty_endpoint() {
        let gen = ComfyUIGenerator::new("");
        assert!(!gen.is_available());
        assert_eq!(gen.name(), "comfyui");
    }

    #[test]
    fn test_dalle_not_available_with_empty_key() {
        let gen = DallE3Generator::new("");
        assert!(!gen.is_available());
        assert_eq!(gen.name(), "dall-e-3");
    }

    #[test]
    fn test_quality_variants() {
        let qs = vec![GenQuality::Draft, GenQuality::Standard, GenQuality::High];
        assert_eq!(qs.len(), 3);
    }
}

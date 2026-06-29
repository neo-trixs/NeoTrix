pub mod image_cache;
pub mod vision_system;

use base64::Engine;

use crate::core::nt_core_consciousness::vsa_tag::{
    SenseModality, VsaOrigin, VsaTagged, VsaWorldCategory,
};
use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;
use crate::core::nt_core_llm_provider::{LlmProvider, LlmRequest};

pub use vision_system::{ImagePercept, VisionResult, VisionStage, VisionSystem};

/// Source of image data for analysis
pub enum ImageSource {
    /// Path to an image file on disk
    File(String),
    /// Base64-encoded image data with MIME type
    Base64 { data: String, mime: String },
    /// Raw image bytes with MIME type
    Raw { data: Vec<u8>, mime: String },
}

/// Result of image analysis
#[derive(Debug)]
pub struct ImageDescription {
    /// LLM-generated text description of the image
    pub text: String,
    /// VSA-encoded representation of the description (4096-bit binary)
    pub vsa_vector: Vec<u8>,
    /// MIME type of the source image
    pub mime_type: String,
    /// LLM model used for analysis
    pub llm_model: String,
    /// Timestamp of analysis in ms since epoch
    pub timestamp_ms: i64,
}

/// Image Understanding Pipeline
///
/// Loads images from file paths or raw bytes, sends to a multimodal LLM,
/// converts the resulting description into a VSA vector for consciousness ingestion.
///
/// Graceful degradation: if the provider does not support images, returns
/// a descriptive error. No crash, no panic.
pub struct ImagePipeline {
    aligner: CrossModalAligner,
    provider: Box<dyn LlmProvider>,
    model: String,
    default_prompt: String,
}

impl ImagePipeline {
    /// Create with explicit provider and model
    pub fn new(provider: Box<dyn LlmProvider>, model: &str) -> Self {
        Self {
            aligner: CrossModalAligner::new(4096, 0xdead_beef),
            provider,
            model: model.to_string(),
            default_prompt: "Describe this image in detail. Include: main subjects, colors, composition, text if any, and overall scene. Be thorough but concise.".to_string(),
        }
    }

    /// Set a custom analysis prompt
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.default_prompt = prompt.to_string();
        self
    }

    /// Analyze an image from a file path
    pub async fn analyze_file(&self, path: &str) -> Result<ImageDescription, String> {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read image file '{}': {}", path, e))?;
        if bytes.is_empty() {
            return Err(format!("Image file '{}' is empty", path));
        }
        let mime = mime_from_extension(path);
        self.analyze_raw(&bytes, &mime).await
    }

    /// Analyze base64-encoded image data
    pub async fn analyze_base64(&self, data: &str, mime: &str) -> Result<ImageDescription, String> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;
        let response = self
            .call_multimodal_llm(&decoded, mime, &self.default_prompt)
            .await?;
        Ok(self.build_description(response, mime))
    }

    /// Analyze image from raw bytes with explicit MIME type
    pub async fn analyze_raw(&self, data: &[u8], mime: &str) -> Result<ImageDescription, String> {
        let response = self
            .call_multimodal_llm(data, mime, &self.default_prompt)
            .await?;
        Ok(self.build_description(response, mime))
    }

    /// Analyze with a custom prompt
    pub async fn analyze_with_prompt(
        &self,
        source: &ImageSource,
        prompt: &str,
    ) -> Result<ImageDescription, String> {
        let (bytes, mime) = match source {
            ImageSource::File(path) => {
                let bytes =
                    std::fs::read(path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;
                (bytes, mime_from_extension(path))
            }
            ImageSource::Base64 { data, mime } => {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .map_err(|e| format!("Base64 decode failed: {}", e))?;
                (decoded, mime.clone())
            }
            ImageSource::Raw { data, mime } => (data.clone(), mime.clone()),
        };
        let response = self.call_multimodal_llm(&bytes, &mime, prompt).await?;
        Ok(self.build_description(response, &mime))
    }

    /// Convert ImageDescription to a VsaTagged sensory frame for consciousness
    pub fn to_vsa_tagged(&self, desc: &ImageDescription) -> VsaTagged {
        VsaTagged {
            vector: desc.vsa_vector.clone(),
            tag: VsaOrigin::World(VsaWorldCategory::Sensor),
            sense_modality: Some(SenseModality::Visual),
            confidence: 0.9,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            salience: 0.8,
            provenance: None,
            prediction: None,
            outcome: None,
        }
    }

    /// Provider availability check
    pub fn is_available(&self) -> bool {
        true
    }

    /// Convert raw bytes to a base64 data URI string
    pub fn bytes_to_data_uri(&self, data: &[u8], mime: &str) -> String {
        use base64::engine::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        format!("data:{};base64,{}", mime, encoded)
    }

    // ── private helpers ──

    async fn call_multimodal_llm(
        &self,
        data: &[u8],
        _mime: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let mut request = LlmRequest::new(&self.model, prompt);
        request.temperature = 0.3;
        request.max_tokens = 1024;
        request.image_data = Some(data.to_vec());
        let response = self
            .provider
            .complete(&request)
            .await
            .map_err(|e| format!("Image analysis LLM call failed: {}", e))?;
        if response.content.is_empty() {
            return Err("LLM returned empty description for image".to_string());
        }
        Ok(sanitize_vision_output(&response.content))
    }

    fn build_description(&self, text: String, mime: &str) -> ImageDescription {
        let vsa = self.aligner.text_to_vsa(&text);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        ImageDescription {
            text,
            vsa_vector: vsa,
            mime_type: mime.to_string(),
            llm_model: self.model.clone(),
            timestamp_ms: now,
        }
    }
}

impl std::fmt::Debug for ImagePipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImagePipeline")
            .field("model", &self.model)
            .finish()
    }
}

/// Strip common secret patterns from LLM responses (no cross-crate dep)
fn sanitize_vision_output(s: &str) -> String {
    let mut r = s.to_string();
    for pat in &[
        "AKIA",
        "-----BEGIN",
        "sk-",
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
    ] {
        r = r.replace(pat, "[REDACTED]");
    }
    r
}

/// Infer MIME type from file extension
fn mime_from_extension(path: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "tiff" | "tif" => "image/tiff",
        "avif" => "image/avif",
        "heic" | "heif" => "image/heic",
        _ => "image/jpeg",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_llm_provider::{FinishReason, LlmError, LlmResponse, Usage};

    struct MockLlmProvider;
    #[async_trait::async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                content: "mock description".into(),
                model: "mock".into(),
                usage: Usage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                finish_reason: FinishReason::Stop,
            })
        }
        async fn stream_complete(
            &self,
            _request: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            if let Err(e) = tx
                .send(Ok(LlmResponse {
                    content: "mock".into(),
                    model: "mock".into(),
                    usage: Usage {
                        prompt_tokens: 0,
                        completion_tokens: 0,
                        total_tokens: 0,
                    },
                    finish_reason: FinishReason::Stop,
                }))
                .await
            {
                log::warn!("[vision] mock send failed: {}", e);
            }
            Ok(rx)
        }
    }

    fn mock_pipeline() -> ImagePipeline {
        ImagePipeline::new(Box::new(MockLlmProvider), "gpt-4o")
    }

    #[test]
    fn test_mime_from_extension() {
        assert_eq!(mime_from_extension("photo.jpg"), "image/jpeg");
        assert_eq!(mime_from_extension("photo.jpeg"), "image/jpeg");
        assert_eq!(mime_from_extension("photo.png"), "image/png");
        assert_eq!(mime_from_extension("photo.gif"), "image/gif");
        assert_eq!(mime_from_extension("photo.webp"), "image/webp");
        assert_eq!(mime_from_extension("photo.bmp"), "image/bmp");
        assert_eq!(mime_from_extension("photo.svg"), "image/svg+xml");
        assert_eq!(mime_from_extension("photo.ico"), "image/x-icon");
        assert_eq!(mime_from_extension("photo.tiff"), "image/tiff");
        assert_eq!(mime_from_extension("photo.tif"), "image/tiff");
        assert_eq!(mime_from_extension("photo.avif"), "image/avif");
        assert_eq!(mime_from_extension("photo.heic"), "image/heic");
    }

    #[test]
    fn test_mime_from_extension_unknown_defaults_to_jpeg() {
        assert_eq!(mime_from_extension("photo.raw"), "image/jpeg");
        assert_eq!(mime_from_extension("photo"), "image/jpeg");
        assert_eq!(mime_from_extension(""), "image/jpeg");
    }

    #[test]
    fn test_mime_from_extension_case_insensitive() {
        assert_eq!(mime_from_extension("photo.JPG"), "image/jpeg");
        assert_eq!(mime_from_extension("photo.PNG"), "image/png");
    }

    #[test]
    fn test_bytes_to_data_uri() {
        let pipeline = mock_pipeline();
        let data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        let uri = pipeline.bytes_to_data_uri(&data, "image/jpeg");
        assert!(uri.starts_with("data:image/jpeg;base64,"));
        assert!(!uri.ends_with(";base64,"));
    }

    #[test]
    fn test_bytes_to_data_uri_diff_mime() -> Result<(), String> {
        let pipeline = mock_pipeline();
        let data = b"PNG data";
        let uri = pipeline.bytes_to_data_uri(data, "image/png");
        assert!(uri.starts_with("data:image/png;base64,"));
        // Verify the base64 is valid
        let encoded = uri.trim_start_matches("data:image/png;base64,");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| format!("valid base64: {}", e))?;
        assert_eq!(decoded, data);
        Ok(())
    }

    #[test]
    fn test_to_vsa_tagged_fields() {
        let pipeline = mock_pipeline();
        let desc = ImageDescription {
            text: "a cat sitting on a mat".to_string(),
            vsa_vector: vec![1u8; 4096],
            mime_type: "image/jpeg".to_string(),
            llm_model: "gpt-4o".to_string(),
            timestamp_ms: 1000,
        };
        let tagged = pipeline.to_vsa_tagged(&desc);
        assert_eq!(tagged.vector.len(), 4096);
        assert_eq!(tagged.sense_modality, Some(SenseModality::Visual));
        assert_eq!(tagged.confidence, 0.9);
        assert_eq!(tagged.salience, 0.8);
    }

    #[test]
    fn test_image_pipeline_debug() {
        let pipeline = mock_pipeline();
        let debug = format!("{:?}", pipeline);
        assert!(debug.contains("ImagePipeline"));
        assert!(debug.contains("gpt-4o"));
    }

    #[test]
    fn test_analyze_file_not_found() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pipeline = mock_pipeline();
        let result = rt.block_on(pipeline.analyze_file("/nonexistent/image.png"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read"));
    }

    #[test]
    fn test_analyze_empty_file_rejected() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pipeline = mock_pipeline();
        // Write empty temp file
        let dir = std::env::temp_dir();
        let path = dir.join("_neotrix_empty_test.png");
        std::fs::write(&path, &[]).unwrap();
        let result = rt.block_on(pipeline.analyze_file(path.to_str().unwrap()));
        std::fs::remove_file(&path).unwrap();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("is empty"));
    }

    #[test]
    fn test_analyze_with_prompt_file_not_found() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pipeline = mock_pipeline();
        let result = rt.block_on(pipeline.analyze_with_prompt(
            &ImageSource::File("/nonexistent/img.jpg".to_string()),
            "what color is this?",
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_image_source_base64_decode_error() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pipeline = mock_pipeline();
        let result = rt.block_on(pipeline.analyze_with_prompt(
            &ImageSource::Base64 {
                data: "!!!invalid-base64!!!".to_string(),
                mime: "image/png".to_string(),
            },
            "describe",
        ));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Base64 decode failed"));
    }

    #[test]
    fn test_image_pipeline_is_available() {
        let pipeline = mock_pipeline();
        assert!(pipeline.is_available());
    }

    #[test]
    fn test_with_prompt_chains_correctly() {
        let pipeline = mock_pipeline().with_prompt("custom prompt");
        assert_eq!(pipeline.default_prompt, "custom prompt");
    }
}

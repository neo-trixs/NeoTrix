use super::types::*;
use super::engine::MediaSource;

pub struct LxScriptSource;

impl LxScriptSource {
    pub fn new() -> Self {
        Self
    }
}

impl MediaSource for LxScriptSource {
    fn id(&self) -> &str {
        "lx_script"
    }

    fn name(&self) -> &str {
        "LX Music Script"
    }

    fn media_type(&self) -> MediaType {
        MediaType::Audio
    }

    fn search(
        &self,
        query: &str,
        page: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            Ok(SearchResult {
                data: vec![],
                total: 0,
                source: "lx_script".into(),
                page,
            })
        })
    }
}

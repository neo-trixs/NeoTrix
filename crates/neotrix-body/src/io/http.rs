//! # HTTP Device Driver
//!
//! HTTP client with stealth capabilities (fingerprint rotation, proxy chain).

/// HTTP request methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
        }
    }
}

/// HTTP request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timeout_ms: u64,
    pub use_stealth: bool,
}

impl Default for HttpRequest {
    fn default() -> Self {
        Self {
            method: HttpMethod::Get,
            url: String::new(),
            headers: Vec::new(),
            body: None,
            timeout_ms: 10000,
            use_stealth: false,
        }
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub latency_ms: u64,
}

/// HTTP driver trait
pub trait HttpDriver: std::fmt::Debug + Send + Sync {
    fn request(&self, req: HttpRequest) -> Result<HttpResponse, String>;
}

/// Mock HTTP driver
#[derive(Debug, Clone)]
pub struct MockHttpDriver;

impl HttpDriver for MockHttpDriver {
    fn request(&self, req: HttpRequest) -> Result<HttpResponse, String> {
        Ok(HttpResponse {
            status: 200,
            headers: vec![("content-type".into(), "text/plain".into())],
            body: format!("Mock response to {}", req.url),
            latency_ms: 50,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_http_driver() {
        let driver = MockHttpDriver;
        let req = HttpRequest {
            url: "https://example.com".into(),
            ..Default::default()
        };
        let resp = driver.request(req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_http_method_str() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
    }
}

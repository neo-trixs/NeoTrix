//! # Filesystem Device Driver
//!
//! Abstract filesystem access. Supports local and remote (mock) backends.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsOperation {
    Read,
    Write,
    List,
    Delete,
    Metadata,
}

#[derive(Debug, Clone)]
pub struct FsRequest {
    pub operation: FsOperation,
    pub path: String,
    pub content: Option<String>,
    pub max_bytes: Option<u64>,
}

impl Default for FsRequest {
    fn default() -> Self {
        Self {
            operation: FsOperation::Read,
            path: String::new(),
            content: None,
            max_bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FsResponse {
    pub success: bool,
    pub content: Option<String>,
    pub entries: Option<Vec<String>>,
    pub size_bytes: Option<u64>,
    pub error: Option<String>,
}

pub trait FsDriver: std::fmt::Debug + Send + Sync {
    fn execute(&self, request: FsRequest) -> FsResponse;
}

#[derive(Debug, Clone)]
pub struct MockFsDriver;

impl FsDriver for MockFsDriver {
    fn execute(&self, request: FsRequest) -> FsResponse {
        match request.operation {
            FsOperation::Read => FsResponse {
                success: true,
                content: Some(format!("mock content of {}", request.path)),
                entries: None,
                size_bytes: Some(0),
                error: None,
            },
            FsOperation::Write => FsResponse {
                success: true,
                content: None,
                entries: None,
                size_bytes: None,
                error: None,
            },
            FsOperation::List => FsResponse {
                success: true,
                content: None,
                entries: Some(vec!["file1.txt".into(), "file2.txt".into()]),
                size_bytes: None,
                error: None,
            },
            _ => FsResponse {
                success: false,
                content: None,
                entries: None,
                size_bytes: None,
                error: Some("unsupported".into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_fs_driver_read() {
        let driver = MockFsDriver;
        let req = FsRequest {
            operation: FsOperation::Read,
            path: "/tmp/test.txt".into(),
            ..Default::default()
        };
        let resp = driver.execute(req);
        assert!(resp.success);
        assert!(resp.content.unwrap().contains("mock content"));
    }

    #[test]
    fn test_mock_fs_driver_list() {
        let driver = MockFsDriver;
        let req = FsRequest {
            operation: FsOperation::List,
            path: "/tmp".into(),
            ..Default::default()
        };
        let resp = driver.execute(req);
        assert!(resp.success);
        assert_eq!(resp.entries.unwrap().len(), 2);
    }
}

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use futures::StreamExt;

use super::{CloudResult, CloudRuntime};

#[async_trait]
pub trait CloudSandboxProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn execute(
        &self,
        session_id: &str,
        code: &str,
        runtime: CloudRuntime,
    ) -> Result<CloudResult, String>;

    async fn upload_file(
        &self,
        session_id: &str,
        path: &str,
        data: Vec<u8>,
    ) -> Result<(), String>;

    async fn download_result(&self, session_id: &str) -> Result<CloudResult, String>;

    fn stream_logs(&self, session_id: &str) -> BoxStream<'static, String>;

    async fn cancel(&self, session_id: &str) -> Result<(), String>;
}

pub struct NoopProvider;

#[async_trait]
impl CloudSandboxProvider for NoopProvider {
    fn name(&self) -> &'static str {
        "noop"
    }

    async fn execute(
        &self,
        _session_id: &str,
        _code: &str,
        _runtime: CloudRuntime,
    ) -> Result<CloudResult, String> {
        Err("No sandbox provider configured — install Docker or set NEOTRIX_CLOUD_ENDPOINT".to_string())
    }

    async fn upload_file(
        &self,
        _session_id: &str,
        _path: &str,
        _data: Vec<u8>,
    ) -> Result<(), String> {
        Err("No sandbox provider configured".to_string())
    }

    async fn download_result(&self, _session_id: &str) -> Result<CloudResult, String> {
        Err("No sandbox provider configured".to_string())
    }

    fn stream_logs(&self, _session_id: &str) -> BoxStream<'static, String> {
        stream::once(async { "No sandbox provider configured".to_string() }).boxed()
    }

    async fn cancel(&self, _session_id: &str) -> Result<(), String> {
        Err("No sandbox provider configured".to_string())
    }
}

use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;

use super::provider::CloudSandboxProvider;
use super::{CloudResult, CloudRuntime, ResourceUsage};

pub struct RemoteApiProvider {
    endpoint: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl RemoteApiProvider {
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();
        Self {
            endpoint,
            api_key,
            client,
        }
    }

    fn auth_header(&self) -> Option<(&str, &str)> {
        self.api_key.as_ref().map(|k| ("Authorization", k.as_str()))
    }
}

#[async_trait]
impl CloudSandboxProvider for RemoteApiProvider {
    fn name(&self) -> &'static str {
        "remote-api"
    }

    async fn execute(
        &self,
        session_id: &str,
        code: &str,
        runtime: CloudRuntime,
    ) -> Result<CloudResult, String> {
        let url = format!("{}/api/v1/sandbox/{}/execute", self.endpoint, session_id);
        let mut req = self.client.post(&url).json(&serde_json::json!({
            "code": code,
            "runtime": format!("{:?}", runtime),
        }));

        if let Some((k, v)) = self.auth_header() {
            req = req.header(k, v);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("parse response: {}", e))?;

        if !status.is_success() {
            let msg = body["error"].as_str().unwrap_or("unknown error");
            return Err(format!("Remote API error ({}): {}", status, msg));
        }

        Ok(CloudResult {
            stdout: body["stdout"].as_str().unwrap_or("").to_string(),
            stderr: body["stderr"].as_str().unwrap_or("").to_string(),
            exit_code: body["exit_code"].as_i64().unwrap_or(-1) as i32,
            execution_time: Duration::from_secs_f64(body["execution_time"].as_f64().unwrap_or(0.0)),
            resource_usage: ResourceUsage {
                cpu_time: body["resource_usage"]["cpu_time"].as_f64().unwrap_or(0.0),
                memory_mb: body["resource_usage"]["memory_mb"].as_f64().unwrap_or(0.0),
                network_kb: body["resource_usage"]["network_kb"].as_f64().unwrap_or(0.0),
            },
        })
    }

    async fn upload_file(&self, session_id: &str, path: &str, data: Vec<u8>) -> Result<(), String> {
        let url = format!("{}/api/v1/sandbox/{}/upload", self.endpoint, session_id);
        let mut req = self
            .client
            .post(&url)
            .multipart(reqwest::multipart::Form::new().part(
                "file",
                reqwest::multipart::Part::bytes(data).file_name(path.to_string()),
            ));

        if let Some((k, v)) = self.auth_header() {
            req = req.header(k, v);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("upload failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = match resp.json().await {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("[remote-sandbox] failed to parse upload error response: {e}");
                    serde_json::Value::Null
                }
            };
            let msg = body["error"].as_str().unwrap_or("upload failed");
            return Err(msg.to_string());
        }
        Ok(())
    }

    async fn download_result(&self, session_id: &str) -> Result<CloudResult, String> {
        let url = format!("{}/api/v1/sandbox/{}/result", self.endpoint, session_id);
        let mut req = self.client.get(&url);

        if let Some((k, v)) = self.auth_header() {
            req = req.header(k, v);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("download result: {}", e))?;
        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("parse result: {}", e))?;

        if !status.is_success() {
            return Err(body["error"]
                .as_str()
                .unwrap_or("download failed")
                .to_string());
        }

        Ok(CloudResult {
            stdout: body["stdout"].as_str().unwrap_or("").to_string(),
            stderr: body["stderr"].as_str().unwrap_or("").to_string(),
            exit_code: body["exit_code"].as_i64().unwrap_or(-1) as i32,
            execution_time: Duration::from_secs_f64(body["execution_time"].as_f64().unwrap_or(0.0)),
            resource_usage: ResourceUsage {
                cpu_time: body["resource_usage"]["cpu_time"].as_f64().unwrap_or(0.0),
                memory_mb: body["resource_usage"]["memory_mb"].as_f64().unwrap_or(0.0),
                network_kb: body["resource_usage"]["network_kb"].as_f64().unwrap_or(0.0),
            },
        })
    }

    fn stream_logs(&self, session_id: &str) -> BoxStream<'static, String> {
        let url = format!("{}/api/v1/sandbox/{}/logs", self.endpoint, session_id);
        let client = self.client.clone();
        let api_key = self.api_key.clone();

        let (mut tx, rx) = futures::channel::mpsc::channel(128);

        tokio::spawn(async move {
            let mut req = client.get(&url);
            if let Some(ref k) = api_key {
                req = req.header("Authorization", k);
            }
            match req.send().await {
                Ok(resp) => {
                    if let Ok(body) = resp.text().await {
                        for line in body.lines() {
                            if let Err(e) = tx.try_send(line.to_string()) {
                                log::warn!("remote.rs: try_send failed: {e}");
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Err(e) = tx.try_send(format!("[error] {}", e)) {
                        log::warn!("remote.rs: try_send failed: {e}");
                    }
                }
            }
        });

        rx.boxed()
    }

    async fn cancel(&self, session_id: &str) -> Result<(), String> {
        let url = format!("{}/api/v1/sandbox/{}/cancel", self.endpoint, session_id);
        let mut req = self.client.post(&url);

        if let Some((k, v)) = self.auth_header() {
            req = req.header(k, v);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("cancel failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = match resp.json().await {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("[remote-sandbox] failed to parse cancel error response: {e}");
                    serde_json::Value::Null
                }
            };
            return Err(body["error"]
                .as_str()
                .unwrap_or("cancel failed")
                .to_string());
        }
        Ok(())
    }
}

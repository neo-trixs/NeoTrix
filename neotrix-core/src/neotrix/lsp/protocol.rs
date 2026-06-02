use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

pub async fn write_message(
    writer: &mut BufWriter<tokio::process::ChildStdin>,
    msg: &JsonRpcMessage,
) -> std::io::Result<()> {
    let body = serde_json::to_string(msg)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes()).await?;
    writer.write_all(body.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_message(
    reader: &mut BufReader<tokio::process::ChildStdout>,
) -> Result<JsonRpcMessage, String> {
    let length = read_content_length(reader).await?;
    let mut buf = vec![0u8; length];
    let mut read = 0;
    while read < length {
        let n = reader
            .read(&mut buf[read..])
            .await
            .map_err(|e| format!("read body: {}", e))?;
        if n == 0 {
            return Err("unexpected EOF in body".into());
        }
        read += n;
    }
    serde_json::from_slice::<JsonRpcMessage>(&buf)
        .map_err(|e| format!("parse json: {}", e))
}

async fn read_content_length(
    reader: &mut BufReader<tokio::process::ChildStdout>,
) -> Result<usize, String> {
    let mut length = None;
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| format!("read header: {}", e))?;
        if line.is_empty() {
            return Err("unexpected EOF in headers".into());
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            length = Some(
                len_str
                    .parse::<usize>()
                    .map_err(|e| format!("bad Content-Length: {}", e))?,
            );
        }
    }
    length.ok_or_else(|| "missing Content-Length header".into())
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

impl IpcError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recoverable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse<T: Serialize = ()> {
    pub ok: bool,
    pub error: Option<IpcError>,
    pub data: Option<T>,
}

impl<T: Serialize> IpcResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            error: None,
            data: Some(data),
        }
    }
    pub fn error(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(IpcError::new(code, msg)),
            data: None,
        }
    }
}

pub fn ok<T: Serialize>(data: T) -> IpcResponse<T> {
    IpcResponse::success(data)
}

pub fn err<T: Serialize>(code: impl Into<String>, msg: impl Into<String>) -> IpcResponse<T> {
    IpcResponse::error(code, msg)
}

/// Convert an `AppError` directly into an `IpcResponse`.
pub fn from_error<T: Serialize>(e: crate::app_error::AppError) -> IpcResponse<T> {
    IpcResponse {
        ok: false,
        error: Some(e.into()),
        data: None,
    }
}

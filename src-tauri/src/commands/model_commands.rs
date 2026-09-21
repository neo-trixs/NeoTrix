#![forbid(unsafe_code)]

//! # Model Manager Commands — 模型管理统一接口
//!
//! 提供模型下载、验证、列表等操作的 Tauri 命令。

use crate::desktop::model_manager::{ModelFormat, ModelManager, ModelMetadata, ModelSource};
use crate::ipc::{self, IpcResponse};
use std::sync::Arc;
use tauri::command;
use tauri::State;
use tokio::sync::RwLock;

// ========== 模型管理状态 ==========

pub type ModelState = Arc<RwLock<ModelManager>>;

pub fn new_model_state() -> ModelState {
    Arc::new(RwLock::new(ModelManager::new()))
}

// ========== Commands ==========

/// 列出所有本地模型
#[command]
pub async fn model_list_local(state: State<'_, ModelState>) -> Result<IpcResponse<Vec<ModelMetadata>>, String> {
    let manager = state.read().await;
    Ok(ipc::ok(manager.list_models().into_iter().cloned().collect()))
}

/// 获取模型详情
#[command]
pub async fn model_get_metadata(
    state: State<'_, ModelState>,
    model_id: String,
) -> Result<IpcResponse<Option<ModelMetadata>>, String> {
    let manager = state.read().await;
    Ok(ipc::ok(manager.get_model(&model_id).cloned()))
}

/// 删除本地模型
#[command]
pub async fn model_delete_local(state: State<'_, ModelState>, model_id: String) -> Result<IpcResponse<()>, String> {
    let mut manager = state.write().await;
    match manager.delete_model(&model_id).await {
        Ok(()) => Ok(ipc::ok(())),
        Err(e) => Ok(ipc::err("MODEL_DELETE_FAILED", format!("{e}"))),
    }
}

/// 验证模型完整性
#[command]
pub async fn model_validate(
    state: State<'_, ModelState>,
    model_id: String,
) -> Result<IpcResponse<serde_json::Value>, String> {
    let manager = state.read().await;
    match manager.validate_model(&model_id).await {
        Ok(result) => Ok(ipc::ok(serde_json::json!({
            "model_id": result.model_id,
            "sha256_valid": result.sha256_valid,
            "model_json_valid": result.model_json_valid,
            "format_valid": result.format_valid,
            "file_size_matches": result.file_size_matches,
            "overall_valid": result.overall_valid,
        }))),
        Err(e) => Ok(ipc::err("MODEL_VALIDATE_FAILED", e)),
    }
}

/// 扫描本地模型目录
#[command]
pub async fn model_scan_local(state: State<'_, ModelState>) -> Result<IpcResponse<Vec<ModelMetadata>>, String> {
    let mut manager = state.write().await;
    match manager.scan_local_models().await {
        Ok(()) => Ok(ipc::ok(manager.list_models().into_iter().cloned().collect())),
        Err(e) => Ok(ipc::err("MODEL_SCAN_FAILED", e)),
    }
}

/// 获取模型管理器统计信息
#[command]
pub async fn model_stats(state: State<'_, ModelState>) -> Result<IpcResponse<serde_json::Value>, String> {
    let manager = state.read().await;
    let models = manager.list_models();
    let total_size: u64 = models.iter().map(|m| m.file_size).sum();

    Ok(ipc::ok(serde_json::json!({
        "total_models": models.len(),
        "total_size_bytes": total_size,
        "total_size_mb": total_size / 1024 / 1024,
        "by_format": {
            "gguf": models.iter().filter(|m| m.format == ModelFormat::GGUF).count(),
            "onnx": models.iter().filter(|m| m.format == ModelFormat::ONNX).count(),
            "safetensors": models.iter().filter(|m| m.format == ModelFormat::Safetensors).count(),
        },
    })))
}

/// 搜索模型（按名称）
#[command]
pub async fn model_search(
    state: State<'_, ModelState>,
    query: String,
) -> Result<IpcResponse<Vec<ModelMetadata>>, String> {
    let manager = state.read().await;
    let query_lower = query.to_lowercase();
    Ok(ipc::ok(
        manager
            .list_models()
            .into_iter()
            .filter(|m| {
                m.id.to_lowercase().contains(&query_lower)
                    || m.display_name.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect(),
    ))
}

// ========== 测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_source_display() {
        assert_eq!(ModelSource::HuggingFace.display_name(), "Hugging Face");
        assert_eq!(ModelSource::Ollama.display_name(), "Ollama");
    }

    #[test]
    fn test_model_format_display() {
        assert_eq!(ModelFormat::GGUF.to_string(), "gguf");
        assert_eq!(ModelFormat::ONNX.to_string(), "onnx");
    }
}

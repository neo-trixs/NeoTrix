//! WhatsApp CLI 命令
//!
//! 使用 wa-rs 库通过 QR 码或配对码登录个人 WhatsApp 账户

use crate::error::WhatsAppError;
use crate::WhatsAppResult;
use tauri::State;

// WhatsApp 会话管理器 (通过 db_path 标识)
struct WhatsAppSessionManager {
    db_path: String,
}

impl WhatsAppSessionManager {
    fn new(db_path: &str) -> Self {
        WhatsAppSessionManager { db_path: db_path.to_string() }
    }

    fn init_db(&self) -> Result<(), String> {
        // 简化的数据库初始化
        // 实际实现会由 neotrix-core 的 SessionManager 处理
        Ok(())
    }
}

/// 使用 QR 码登录 WhatsApp
#[tauri::command]
pub async fn cmd_whatsapp_login_qr(db_path: String) -> Result<WhatsAppResult<()>, String> {
    let manager = WhatsAppSessionManager::new(&db_path);
    manager.init_db().map_err(|e| WhatsAppError::Storage(e))?;

    let bot = crate::unified::layers::action::nt_io::nt_io_whatsapp::WhatsAppBot::new(&db_path);
    bot.login_with_qr().await.map_err(|e| WhatsAppError::Connection(e))?;

    Ok(Ok(()))
}

/// 使用配对码登录 WhatsApp
#[tauri::command]
pub async fn cmd_whatsapp_login_pair(
    phone_number: String,
    custom_code: Option<String>,
    db_path: String,
) -> Result<WhatsAppResult<String>, String> {
    let manager = WhatsAppSessionManager::new(&db_path);
    manager.init_db().map_err(|e| WhatsAppError::Storage(e))?;

    let bot = crate::unified::layers::action::nt_io::nt_io_whatsapp::WhatsAppBot::new(&db_path);
    let result = bot.login_with_pair_code(&phone_number, custom_code.as_deref()).await
        .map_err(|e| WhatsAppError::Connection(e))?;

    Ok(Ok(result))
}
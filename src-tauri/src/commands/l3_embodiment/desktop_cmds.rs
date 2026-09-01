//! Desktop 辅助命令 — 剪贴板 / 图像生成 / 应用切换 / 深度审查
//!
//! 补齐前端 lib/api.ts 调用但后端缺失的命令 (审计发现 8 个运行时缺失项)。

use serde::Serialize;
use tauri::Window;

/// 窗口最小化
#[tauri::command]
pub fn window_minimize(window: Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

/// 窗口最大化/还原
#[tauri::command]
pub fn window_maximize(window: Window) -> Result<(), String> {
    if window.is_maximized().map_err(|e| e.to_string())? {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

/// 窗口关闭
#[tauri::command]
pub fn window_close(window: Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

/// 窗口是否最大化
#[tauri::command]
pub fn window_is_maximized(window: Window) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}

/// 读取剪贴板文本 (macOS/Windows/Linux 通用, 经 arboard)
#[tauri::command]
pub fn read_clipboard() -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("clipboard init failed: {}", e))?;
    clipboard.get_text().map_err(|e| format!("clipboard read failed: {}", e))
}

/// 写入剪贴板文本
#[tauri::command]
pub fn write_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("clipboard init failed: {}", e))?;
    clipboard.set_text(text).map_err(|e| format!("clipboard write failed: {}", e))
}

/// 图像生成 — 经由 nt_io_provider 网关的图像模型。
/// 当前 provider 层尚未暴露图像端点, 返回明确的未实现错误 (避免前端静默失败)。
#[tauri::command]
pub async fn image_generate(prompt: String, options: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let _ = (prompt, options);
    Err("image_generate: 图像生成端点在 provider 层尚未接入 (ImageGen 路由需 nt_io_provider 图像模型支持)".to_string())
}

/// 前台应用切换 — macOS System Events
#[tauri::command]
pub fn switch_app(app_name: String) -> Result<(), String> {
    let output = std::process::Command::new("osascript")
        .args([
            "-e",
            &format!(r#"tell application "{}" to activate"#, app_name),
        ])
        .output()
        .map_err(|e| format!("Failed to activate app {}: {}", app_name, e))?;
    if !output.status.success() {
        return Err(format!("activate {} failed: {}", app_name, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(())
}

/// 深度代码审查 — 调用 neotrix-core 的审查能力
/// 当前为本地启发式扫描 + 结构化报告 (与 review_cmds 保持一致)。
#[derive(Serialize, Clone)]
pub struct UltraReviewResult {
    pub target: String,
    pub findings: Vec<UltraFinding>,
    pub score: f64,
    pub summary: String,
}

#[derive(Serialize, Clone)]
pub struct UltraFinding {
    pub severity: String,
    pub title: String,
    pub detail: String,
    pub line: Option<usize>,
}

#[tauri::command]
pub async fn ultra_review(config: Option<serde_json::Value>) -> Result<UltraReviewResult, String> {
    let target = config
        .as_ref()
        .and_then(|c| c.get("target"))
        .and_then(|t| t.as_str())
        .unwrap_or(".")
        .to_string();

    let mut findings = Vec::new();
    let mut score = 100.0;

    // 扫描目录下的 Rust 源码: 检查 unwrap/panic/todo 生产密度
    fn scan_dir(path: &std::path::Path, findings: &mut Vec<UltraFinding>, score: &mut f64) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    scan_dir(&p, findings, score);
                    continue;
                }
                if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                    continue;
                }
                if p.to_string_lossy().contains("tests") || p.to_string_lossy().contains("target") {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&p) {
                    for (i, line) in content.lines().enumerate() {
                        let l = i + 1;
                        if line.contains("unwrap()") {
                            findings.push(UltraFinding {
                                severity: "warning".into(),
                                title: "production unwrap()".into(),
                                detail: line.trim().to_string(),
                                line: Some(l),
                            });
                            *score -= 0.5;
                        }
                        if line.contains("todo!()") || line.contains("unimplemented!()") {
                            findings.push(UltraFinding {
                                severity: "warning".into(),
                                title: "unimplemented stub".into(),
                                detail: line.trim().to_string(),
                                line: Some(l),
                            });
                            *score -= 1.0;
                        }
                    }
                }
            }
        }
    }

    scan_dir(std::path::Path::new(&target), &mut findings, &mut score);

    Ok(UltraReviewResult {
        target,
        findings: findings.iter().take(50).cloned().collect(),
        score: score.max(0.0),
        summary: format!("扫描完成: {} 个发现, 健康度 {:.1}", findings.len(), score.max(0.0)),
    })
}

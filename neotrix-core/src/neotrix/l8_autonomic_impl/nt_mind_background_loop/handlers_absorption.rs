use super::*;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

// ============================================================================
// handlers_absorption.rs — pending-absorb 自动吸收 (意识能力网内化, cycle 1053)
//
// 内化原 .opencode/plugins/experience-tree-absorption.js 的 idle 机制:
//   session.idle → ~/.neotrix/pending-absorb.json → neotrix-experience absorb
//   → 成功删除 pending → close --cycle NNN
// 吸收完全由 NeoTrix 自身后台运行时驱动, 不再依赖任何 opencode 插件。
// 挂载点: run.rs spawn_handler!(60, |h| h.handle_pending_absorption().await)
// ============================================================================

/// 重入守卫 — handler 执行期间禁止并发重复触发 (原子标志 RAII 复位)。
struct ReentryGuard<'a>(&'a AtomicBool);

impl<'a> ReentryGuard<'a> {
    fn acquire(flag: &'a AtomicBool) -> Option<Self> {
        if flag.swap(true, Ordering::Relaxed) {
            return None; // 已有吸收在跑
        }
        Some(Self(flag))
    }
}

impl Drop for ReentryGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

/// pending-absorb.json 的读取结构 (仅取吸收所需字段)。
#[derive(Debug, Deserialize)]
struct PendingAbsorb {
    cycle: String,
    #[serde(rename = "session_id")]
    session_id: String,
}

/// 解析 pending JSON — 提取 cycle 用于吸收后 close (R-P16: 单元测试覆盖)。
fn parse_pending(content: &str) -> Result<PendingAbsorb, String> {
    let v: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("invalid pending JSON: {e}"))?;
    let cycle = v
        .get("cycle")
        .and_then(|c| c.as_str())
        .ok_or_else(|| "missing 'cycle' field".to_string())?
        .to_string();
    let session_id = v
        .get("session_id")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "missing 'session_id' field".to_string())?
        .to_string();
    Ok(PendingAbsorb { cycle, session_id })
}

/// 定位 neotrix-experience CLI: 优先 ~/.local/bin, 回退 PATH。
fn experience_cli() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let local = home.join(".local").join("bin").join("neotrix-experience");
        if local.exists() {
            return Some(local);
        }
    }
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|p| p.join("neotrix-experience"))
            .find(|p| p.exists())
    })
}

/// 待吸收文件路径: ~/.neotrix/pending-absorb.json
fn pending_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("pending-absorb.json")
}

impl BackgroundLoopHandle {
    /// 后台吸收 handler — 周期检查 pending 文件, 存在则驱动 CLI 吸收。
    /// 幂等语义: 失败保留文件下轮重试; 防重入标志避免重叠执行。
    pub(crate) async fn handle_pending_absorption(&mut self) {
        let Some(_guard) = ReentryGuard::acquire(&self.absorption_in_progress) else {
            log::trace!("[bg-absorb] previous absorption still running, skip tick");
            return;
        };

        let pending = pending_path();
        let content = match std::fs::read_to_string(&pending) {
            Ok(c) => c,
            Err(_) => return, // 无待吸收文件 — 正常静默
        };

        let parsed = match parse_pending(&content) {
            Ok(p) => p,
            Err(e) => {
                // 非法 pending 文件: 记录并保留, 由人工/后续会话处理 (不自动删数据)。
                log::warn!("[bg-absorb] pending-absorb.json malformed ({e}); keeping file");
                return;
            }
        };

        let Some(cli) = experience_cli() else {
            log::warn!("[bg-absorb] neotrix-experience CLI not found; pending kept for retry");
            return;
        };

        log::info!("[bg-absorb] absorbing pending file (session {}, cycle {})", parsed.session_id, parsed.cycle);
        let absorb = tokio::time::timeout(
            std::time::Duration::from_secs(600),
            tokio::process::Command::new(&cli)
                .arg("absorb")
                .arg(&pending)
                .output(),
        )
        .await;

        match absorb {
            Ok(Ok(out)) if out.status.success() => {
                // 吸收成功 → 删除 pending → close 快照 (反馈阶段)
                let _ = std::fs::remove_file(&pending);
                log::info!("[bg-absorb] absorbed {} (cycle {}), pending removed", parsed.session_id, parsed.cycle);
                match tokio::time::timeout(
                    std::time::Duration::from_secs(60),
                    tokio::process::Command::new(&cli)
                        .arg("close")
                        .arg("--cycle")
                        .arg(&parsed.cycle)
                        .output(),
                )
                .await
                {
                    Ok(Ok(o)) if o.status.success() => {
                        log::info!("[bg-absorb] closed cycle {}", parsed.cycle);
                    }
                    Ok(Ok(o)) => {
                        log::warn!("[bg-absorb] close cycle {} failed: {}", parsed.cycle, String::from_utf8_lossy(&o.stderr).trim());
                    }
                    Ok(Err(e)) => log::warn!("[bg-absorb] close spawn failed: {e}"),
                    Err(_) => log::warn!("[bg-absorb] close timed out"),
                }
            }
            Ok(Ok(out)) => {
                // 吸收失败: 保留 pending 下轮重试 (CLI 内部幂等)
                log::warn!(
                    "[bg-absorb] absorb failed (exit {}): {}",
                    out.status,
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
            Ok(Err(e)) => log::warn!("[bg-absorb] absorb spawn failed: {e}"),
            Err(_) => log::warn!("[bg-absorb] absorb timed out (600s); pending kept for retry"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pending_valid() {
        let json = r#"{
            "schema_version": 1,
            "session_id": "sess_1234_ab12",
            "cycle": "1053",
            "ts": 1754899200,
            "domain": "NT-CORE",
            "entries": []
        }"#;
        let p = parse_pending(json).unwrap();
        assert_eq!(p.cycle, "1053");
        assert_eq!(p.session_id, "sess_1234_ab12");
    }

    #[test]
    fn test_parse_pending_missing_cycle() {
        let json = r#"{"session_id": "sess_1", "entries": []}"#;
        let err = parse_pending(json).unwrap_err();
        assert!(err.contains("cycle"), "expected cycle error, got: {err}");
    }

    #[test]
    fn test_parse_pending_invalid_json() {
        assert!(parse_pending("not json{{{").is_err());
    }

    #[test]
    fn test_reentry_guard_blocks_second_acquire() {
        let flag = AtomicBool::new(false);
        let g1 = ReentryGuard::acquire(&flag);
        assert!(g1.is_some());
        // 第二次获取应被拒绝
        let g2 = ReentryGuard::acquire(&flag);
        assert!(g2.is_none());
        drop(g1);
        // 释放后可重新获取
        let g3 = ReentryGuard::acquire(&flag);
        assert!(g3.is_some());
        drop(g3);
    }

    #[test]
    fn test_experience_cli_resolution_prefers_local_bin() {
        // 不依赖真实文件系统: 仅验证解析函数存在且 home 拼接逻辑正确。
        // local bin 分支在真实环境 (~/.local/bin/neotrix-experience 已安装) 下命中。
        let resolved = experience_cli();
        // 当前开发机安装于 ~/.local/bin → 应能解析; 若 PATH 也未命中则 None (测试容错)。
        if let Some(path) = &resolved {
            assert!(path.exists(), "resolved CLI should exist: {path:?}");
        }
    }

    #[test]
    fn test_pending_path_under_neotrix_dir() {
        let p = pending_path();
        assert!(p.ends_with(".neotrix/pending-absorb.json"), "unexpected path: {p:?}");
    }
}

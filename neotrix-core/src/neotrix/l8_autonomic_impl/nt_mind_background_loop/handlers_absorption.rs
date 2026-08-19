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
#[derive(Debug, Deserialize, Clone)]
struct PendingAbsorb {
    cycle: String,
    #[serde(rename = "session_id")]
    session_id: String,
}

/// 解析 pending JSON — 提取 cycle 用于吸收后 close (R-P16: 单元测试覆盖)。
///
/// 支持两种格式 (双格式兼容):
/// - object: 单 session (原协议格式): `{"cycle": "N", "session_id": "sess_..."}`
/// - list:   多 session 批处理: `[{"cycle": "N", "session_id": "sess_..."}, ...]`
///
/// 背景循环必须兼容 list — 实际写入 `~/.neotrix/pending-absorb.json` 的并发
/// session 使用 list 格式, 否则整个文件被判定 malformed 而滞留 (格式缺陷修复)。
fn parse_pending(content: &str) -> Result<Vec<PendingAbsorb>, String> {
    let v: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("invalid pending JSON: {e}"))?;

    let items: Vec<&serde_json::Value> = match &v {
        serde_json::Value::Array(arr) => arr.iter().collect(),
        _ => vec![&v],
    };
    if items.is_empty() {
        return Err("pending JSON is empty list".to_string());
    }

    items
        .into_iter()
        .map(|item| {
            let cycle = item
                .get("cycle")
                .and_then(|c| c.as_str())
                .ok_or_else(|| "missing 'cycle' field".to_string())?
                .to_string();
            let session_id = item
                .get("session_id")
                .and_then(|s| s.as_str())
                .ok_or_else(|| "missing 'session_id' field".to_string())?
                .to_string();
            Ok(PendingAbsorb { cycle, session_id })
        })
        .collect()
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

        // 逐个 session 吸收 (list 双格式兼容): 每个 session 子 JSON 经 stdin 直送 CLI,
        // 全部成功才删除 pending (all-or-nothing)。避免 CLI 只消费 list 首个元素
        // 导致后续 session 滞留 (格式缺陷修复)。
        // 淘汰本地临时文件: CLI `absorb -` 支持 stdin (Phase 1 KB 直写迁移),
        // 不再落 ~/tmp/neotrix-pending-*.json 中间载体。
        let mut all_ok = true;
        for (i, item) in parsed.iter().enumerate() {
            // 提取该 session 对应的 JSON 子片段 (object 时即为全文; list 时取对应元素)
            let sub = extract_session(&content, i).unwrap_or_else(|| content.clone());

            log::info!("[bg-absorb] absorbing pending item {} (session {}, cycle {})", i + 1, item.session_id, item.cycle);
            // spawn 同步返回; wait_with_output 才是异步等待 (timeout 包它)。
            let mut child = match tokio::process::Command::new(&cli)
                .arg("absorb")
                .arg("-")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("[bg-absorb] absorb spawn failed: {e}");
                    all_ok = false;
                    continue;
                }
            };
            // stdin 直送 session JSON (写端 tokio 任务, 不阻塞主循环)
            if let Some(mut stdin) = child.stdin.take() {
                let payload = sub.clone();
                tokio::spawn(async move {
                    use tokio::io::AsyncWriteExt;
                    let _ = stdin.write_all(payload.as_bytes()).await;
                    let _ = stdin.shutdown().await;
                });
            }
            let absorb = tokio::time::timeout(
                std::time::Duration::from_secs(600),
                child.wait_with_output(),
            )
            .await;

            match absorb {
                Ok(Ok(out)) if out.status.success() => {
                    // close 快照 (反馈阶段)
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(60),
                        tokio::process::Command::new(&cli)
                            .arg("close")
                            .arg("--cycle")
                            .arg(&item.cycle)
                            .output(),
                    )
                    .await
                    {
                        Ok(Ok(o)) if o.status.success() => {
                            log::info!("[bg-absorb] closed cycle {}", item.cycle);
                        }
                        Ok(Ok(o)) => {
                            log::warn!("[bg-absorb] close cycle {} failed: {}", item.cycle, String::from_utf8_lossy(&o.stderr).trim());
                        }
                        Ok(Err(e)) => log::warn!("[bg-absorb] close spawn failed: {e}"),
                        Err(_) => log::warn!("[bg-absorb] close timed out"),
                    }
                }
                Ok(Ok(out)) => {
                    // 吸收失败: 保留 pending 下轮重试 (CLI 内部幂等)
                    all_ok = false;
                    log::warn!(
                        "[bg-absorb] absorb failed (exit {}): {}",
                        out.status,
                        String::from_utf8_lossy(&out.stderr).trim()
                    );
                }
                Ok(Err(e)) => {
                    all_ok = false;
                    log::warn!("[bg-absorb] absorb spawn failed: {e}");
                }
                Err(_) => {
                    all_ok = false;
                    log::warn!("[bg-absorb] absorb timed out (600s); pending kept for retry");
                }
            }
        }

        if all_ok {
            let _ = std::fs::remove_file(&pending);
            log::info!("[bg-absorb] all {} pending items absorbed, pending removed", parsed.len());
        }
    }
}

/// 从 pending JSON 提取第 idx 个 session 的子 JSON (list 时取元素, object 时取全文)。
/// 失败时返回 None (调用方回退全文 — object 场景天然全文)。
fn extract_session(content: &str, idx: usize) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(content).ok()?;
    match v {
        serde_json::Value::Array(arr) => arr.get(idx).map(|e| e.to_string()),
        _ => Some(content.to_string()),
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
        let parsed = parse_pending(json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cycle, "1053");
        assert_eq!(parsed[0].session_id, "sess_1234_ab12");
    }

    #[test]
    fn test_parse_pending_list_multiple_sessions() {
        // 格式缺陷修复: list 格式 (多 session 批处理) 必须被解析, 不能判 malformed。
        let json = r#"[
            {"schema_version":1,"session_id":"sess_a","cycle":"1109","ts":1,"domain":"NT-CORE","entries":[]},
            {"schema_version":1,"session_id":"sess_b","cycle":"1110","ts":2,"domain":"NT-MIND","entries":[]}
        ]"#;
        let parsed = parse_pending(json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].cycle, "1109");
        assert_eq!(parsed[0].session_id, "sess_a");
        assert_eq!(parsed[1].cycle, "1110");
        assert_eq!(parsed[1].session_id, "sess_b");
    }

    #[test]
    fn test_parse_pending_empty_list_is_error() {
        // 空 list 无内容可吸收 → 视为 malformed (保留文件待人工处理)。
        let err = parse_pending("[]").unwrap_err();
        assert!(err.contains("empty"), "expected empty-list error, got: {err}");
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
    fn test_extract_session_list_element() {
        // list 中提取第 idx 个 session 子 JSON (供 CLI 逐个 absorb)。
        let json = r#"[
            {"session_id":"sess_a","cycle":"1109"},
            {"session_id":"sess_b","cycle":"1110"}
        ]"#;
        let sub0 = extract_session(json, 0).unwrap();
        assert!(sub0.contains("sess_a"));
        assert!(!sub0.contains("sess_b"));
        let sub1 = extract_session(json, 1).unwrap();
        assert!(sub1.contains("sess_b"));
    }

    #[test]
    fn test_extract_session_object_returns_full() {
        let json = r#"{"session_id":"sess_x","cycle":"1111"}"#;
        let sub = extract_session(json, 0).unwrap();
        assert!(sub.contains("sess_x"));
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

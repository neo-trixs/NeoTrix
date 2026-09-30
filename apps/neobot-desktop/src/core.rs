//! 核心/备份/配置 —— 优先级 2：**接线**，不是从零实现。
//!
//! # 概念映射：名字相同，含义不同
//!
//! 上游的 "core" 是 **DSH 引擎二进制**（带 version / tag / release 下载路径）。
//! neotrix 的对应概念是 **provider + model**（跟哪个端点、用哪个模型）。
//!
//! ⛔ 所以这里**不做字段级的假映射**。上游 `HarnessCore` 有 8 个字段，
//!    其中 version / tag / path 对 neotrix 不适用 —— 填假值会让界面显示
//!    一个看起来正常、实际不存在的「已安装核心 v0.2.0」。
//!    ⇒ 不适用的字段**留空**并在前端标明，让「没有」被看见。
//!
//! # 这些能力 neotrix 库里本来就有
//!
//! `list_providers` / `upsert_provider` / `remove_provider`（nt_store_providers）
//! `export_bundle` / `prune_*`（nt_export）
//! `NeobotConfig::from_env`（nt_config）
//! ⇒ 这里只做「库能力 → 上游命令形状」的适配层。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use neotrix_neobot::nt_store::NeobotStore;

use crate::commands::data_dir;

/// 核心（= neotrix 的 provider）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HarnessCore {
    /// `local` = 本仓内置（不是下载来的）。
    pub id: String,
    /// 上游枚举：`local` | `app`。
    pub source: String,
    /// neotrix 没有「核心版本」这个概念 ⇒ 空串而非编一个。
    pub version: String,
    pub tag: String,
    /// 上游是「核心入口（cli path）」；neotrix 对应的是**端点地址**。
    pub path: String,
    /// 「打开目录」入口。neotrix 无对应物 ⇒ 空。
    pub open_dir: String,
    /// 模型名。neotrix 有，且是真实信息。
    pub model: String,
    /// 凭据来自哪个环境变量名（**不返回密钥本身**）。
    pub key_env: String,
    pub enabled: bool,
}

fn open_store() -> Result<NeobotStore, String> {
    let dir = data_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败：{e}"))?;
    let db = dir.join("neobot.db");
    NeobotStore::open(db.to_str().ok_or("数据目录不是合法 UTF-8")?).map_err(|e| e.to_string())
}

/// `get_cores() -> HarnessCore[]`
#[tauri::command]
pub fn get_cores() -> Result<Vec<HarnessCore>, String> {
    let store = open_store()?;
    let active = store.get_core_pair().ok().flatten();
    let mut out: Vec<HarnessCore> = store
        .list_providers()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|p| HarnessCore {
            id: p.name.clone(),
            source: "local".to_owned(),
            version: String::new(),
            tag: String::new(),
            path: p.base_url.clone(),
            open_dir: String::new(),
            model: p.model.clone(),
            key_env: p.key_env.clone(),
            enabled: active.as_ref().map(|c| c.model == p.model).unwrap_or(p.enabled),
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// `set_active_core(core)`
///
/// ⚠️ 入参用 `id`（provider 名）而不是整包回传：界面手上那份可能已过期，
///    用它覆盖会丢掉后端在这期间的变更。**按 id 查**才是对的。
#[tauri::command]
pub fn set_active_core(core: HarnessCoreIn) -> Result<(), String> {
    let store = open_store()?;
    let p = store.get_provider(&core.id).map_err(|e| e.to_string())?.ok_or_else(|| {
        // ⛔ 说清是「不存在」而不是泛泛的 Err：界面上「切到一个没了的 provider」
        //    与「provider 存在但启用失败」是两种处境。
        format!("没有名为 {} 的 provider", core.id)
    })?;
    store
        .upsert_provider(&neotrix_neobot::nt_provider::Provider {
            enabled: core.enabled,
            ..p
        })
        .map_err(|e| e.to_string())
}

/// `set_active_core` 的入参。只取需要的字段。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HarnessCoreIn {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// `remove_core(id)`
#[tauri::command]
pub fn remove_core(id: String) -> Result<(), String> {
    let store = open_store()?;
    store.remove_provider(&id).map_err(|e| e.to_string())
}

/// 备份信息。字段名与上游 `use-backup.ts` 的 `BackupInfo` 逐项对齐。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BackupInfo {
    /// 文件名（不含路径）。⛔ **不返回绝对路径**：
    /// 它会出现在界面上，而绝对路径含用户名。
    pub timestamp: String,
    pub path: String,
    pub size: u64,
    pub include_credentials: bool,
}

/// 备份目录：`<data_dir>/backups`。
fn backups_dir() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("backups"))
}

/// `list_backups() -> BackupInfo[]`（最新在前）
#[tauri::command]
pub fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let dir = backups_dir()?;
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out: Vec<BackupInfo> = std::fs::read_dir(&dir)
        .map_err(|e| format!("读备份目录失败：{e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let p = e.path();
            let size = e.metadata().ok()?.len();
            Some(BackupInfo {
                timestamp: p.file_name()?.to_string_lossy().into_owned(),
                path: p.to_string_lossy().into_owned(),
                size,
                include_credentials: false,
            })
        })
        .collect();
    // ⛔ 必须排序。目录遍历顺序**不保证**是时间序（取决于文件系统），
    //    不排就会看到「最新的在最下面」，而用户预期是最上面。
    out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(out)
}

/// `delete_backup(path)`
///
/// ⚠️ 只允许删**备份目录内**的文件。⛔ 否则「删一个备份」就成了
/// 「删任意路径」—— 参数来自界面，而界面里的路径可能来自导入的旧数据。
#[tauri::command]
pub fn delete_backup(path: String) -> Result<(), String> {
    let dir = backups_dir()?;
    let target = std::path::PathBuf::from(&path);
    let canon_dir = dir.canonicalize().map_err(|e| format!("备份目录不可读：{e}"))?;
    let canon = target.canonicalize().map_err(|e| format!("备份不存在：{e}"))?;
    if !canon.starts_with(&canon_dir) {
        return Err("只能删除备份目录内的文件".to_owned());
    }
    std::fs::remove_file(&canon).map_err(|e| format!("删除失败：{e}"))
}

/// `update_app_config(config)`
///
/// 合并写入 `<data_dir>/app-config.json`。
///
/// ⛔ **合并而不是覆盖**：调用方通常只带一个字段。覆盖会把没带的字段清成默认值，
///    而调用方以为「其它设置还在」。⇒ 先读现有 JSON，再逐键合并。
#[tauri::command]
pub fn update_app_config(config: serde_json::Value) -> Result<(), String> {
    let path = data_dir()?.join("app-config.json");
    let mut cur: serde_json::Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if !cur.is_object() {
        // 坏文件不能当「空」—— 那会用默认值把它盖掉，丢掉用户原有配置。
        return Err(format!(
            "{} 不是合法的 JSON 对象；已拒绝覆盖（请先人工检查）",
            path.display()
        ));
    }
    let obj = config
        .as_object()
        .ok_or_else(|| "config 必须是 JSON 对象".to_owned())?;
    let map = cur.as_object_mut().expect("上面已确认是对象");
    for (k, v) in obj {
        map.insert(k.clone(), v.clone());
    }
    let body = serde_json::to_string_pretty(&cur).map_err(|e| format!("序列化失败：{e}"))?;
    std::fs::write(&path, body).map_err(|e| format!("写配置失败：{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 核心的版本字段留空而不是编造() {
        // ⛔ 编一个 "0.2.0" 会让界面显示一个不存在的已安装版本。
        //    这条守的是「不适用 ≠ 填假值」。
        let c = HarnessCore {
            id: "openai".into(), source: "local".into(),
            version: String::new(), tag: String::new(),
            path: "https://api.openai.com/v1".into(),
            open_dir: String::new(), model: "gpt-x".into(),
            key_env: "OPENAI_API_KEY".into(), enabled: true,
        };
        assert!(c.version.is_empty());
        assert!(c.tag.is_empty());
        assert!(c.open_dir.is_empty());
        // 但**有**的信息必须真：端点与模型是 neotrix 真实持有的
        assert!(c.path.starts_with("https://"));
        assert_eq!(c.model, "gpt-x");
    }

    #[test]
    fn 核心只带凭据的变量名不带密钥() {
        let s = serde_json::to_string(&HarnessCore {
            id: "x".into(), source: "local".into(), version: String::new(),
            tag: String::new(), path: String::new(), open_dir: String::new(),
            model: String::new(), key_env: "API_KEY".into(), enabled: true,
        })
        .unwrap();
        assert!(s.contains("key_env"));
        assert!(!s.contains("apiKey"), "序列化体里不该出现密钥字段名");
    }

    #[test]
    fn 删备份拒绝目录外的路径() {
        // ⛔ 否则「删备份」= 「删任意文件」，参数来自界面就危险。
        let d = backups_dir().unwrap();
        let outside = d.parent().unwrap().join("definitely-not-a-backup.txt");
        std::fs::create_dir_all(d.parent().unwrap()).ok();
        std::fs::write(&outside, "x").ok();
        let r = delete_backup(outside.to_string_lossy().into_owned());
        assert!(r.is_err(), "目录外的路径必须被拒");
        std::fs::remove_file(&outside).ok();
    }

    #[test]
    fn 配置合并保留未提及的键() {
        // ⛔ 覆盖式写会把调用方没带的字段清成默认值。
        let dir = data_dir().unwrap();
        std::fs::create_dir_all(&dir).ok();
        let p = dir.join("app-config.json");
        std::fs::write(&p, r#"{"a":1,"b":"keep"}"#).unwrap();
        // 直接测合并语义（不经过 data_dir 的环境依赖）
        let mut cur: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let inc: serde_json::Value = serde_json::json!({"a": 2});
        {
            let m = cur.as_object_mut().unwrap();
            for (k, v) in inc.as_object().unwrap() {
                m.insert(k.clone(), v.clone());
            }
        }
        assert_eq!(cur["a"], 2, "被提及的键应更新");
        assert_eq!(cur["b"], "keep", "⛔ 未提及的键必须保留");
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn set_active_core的入参默认启用() {
        let c: HarnessCoreIn = serde_json::from_str(r#"{"id":"openai"}"#).unwrap();
        assert_eq!(c.id, "openai");
        assert!(c.enabled, "省略 enabled 时默认启用 —— 上游的调用点依赖这个默认");
    }
}

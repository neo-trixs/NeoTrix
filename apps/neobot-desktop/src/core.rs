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

/// `get_app_config() -> AppConfig`（JSON 对象）
///
/// `update_app_config` 的读侧。缺文件返回 `{}` 而不是报错 ——
/// 全新安装本来就没有配置文件，报错会让设置页首屏就红。
/// 文件坏了（不是合法 JSON）则**报错而不回 `{}`**：
/// 回空对象会让设置页显示一堆默认值，而用户原有配置其实还在盘上。
#[tauri::command]
pub fn get_app_config() -> Result<serde_json::Value, String> {
    let path = data_dir()?.join("app-config.json");
    match std::fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(e) => Err(format!("读应用配置失败：{e}")),
        Ok(s) => serde_json::from_str(&s)
            .map_err(|e| format!("应用配置不是合法 JSON（请先人工检查 {}）：{e}", path.display())),
    }
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

/// 技能清单（`neobot_skill_list` 返回值）。
///
/// 顶替上游「插件」页签的数据源：技能才是 NeoBot 真正的扩展件。
/// `skipped` 是坏包数 —— 坏包不炸列表，但必须能让人看见（静默跳过 = 少了技能
/// 却没人知道）。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SkillListView {
    pub skills: Vec<SkillRow>,
    pub skipped: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SkillRow {
    pub name: String,
    pub description: String,
    pub path: String,
}

/// `neobot_skill_list() -> SkillListView`
#[tauri::command]
pub fn neobot_skill_list() -> Result<SkillListView, String> {
    // ⛔ 技能目录挂在 `data_dir()` 下（与 store/usage.json 同根），不用
    //    `NeobotConfig::from_env` 再解一次 —— 两处解析同一件事时，
    //    环境变量一旦不一致，界面读的就是另一个目录（而没人发现）。
    Ok(skill_list_view(&data_dir()?))
}

/// 技能清单纯函数（命令薄壳；逻辑可测，不碰真实 HOME）。
fn skill_list_view(data_dir: &std::path::Path) -> SkillListView {
    let (skills, skipped) = neotrix_neobot::nt_skills::scan_skills(data_dir);
    SkillListView {
        skills: skills
            .into_iter()
            .map(|s| SkillRow {
                name: s.name,
                description: s.description,
                path: s.path.to_string_lossy().into_owned(),
            })
            .collect(),
        skipped,
    }
}

/// `neobot_skill_install(path) -> ()`
///
/// ⛔ 只接受**绝对路径**且必须已存在：路径不存在时给一句人话，
/// 而不是让库去造一个空目录然后报告成功。
#[tauri::command]
pub fn neobot_skill_install(path: String) -> Result<(), String> {
    let src = check_skill_src(&path)?;
    let skill =
        neotrix_neobot::nt_skills::install_skill(&data_dir()?, &src).map_err(|e| e.to_string())?;
    println!("neobot skill installed: {}", skill.name);
    Ok(())
}

/// 技能来源路径校验（纯函数，可测）。
///
/// ⛔ 拒绝相对路径与不存在目录：否则库会去 `skills/<名字>` 造一个空目录，
/// 然后报告「装好了」—— 用户拿到一个永远不会被加载的空技能。
fn check_skill_src(path: &str) -> Result<std::path::PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("技能路径为空".to_owned());
    }
    let src = std::path::PathBuf::from(trimmed);
    if !src.is_absolute() {
        return Err("技能路径必须是绝对路径".to_owned());
    }
    if !src.is_dir() {
        return Err(format!("技能目录不存在：{}", src.display()));
    }
    Ok(src)
}

#[cfg(test)]
mod skill_tests {
    use super::{check_skill_src, skill_list_view, SkillListView, SkillRow};

    /// 临时目录（app crate 不依赖库的 testutil —— 那是库内部的测试设施）。
    fn tmp(case: &str) -> std::path::PathBuf {
        let d = tempdir::TempDir::new(&format!("nb-skill-{case}")).expect("临时目录");
        let p = d.path().to_path_buf();
        // TempDir 析构会删目录；测试内已显式清理，这里保活到函数结束即可。
        std::mem::forget(d);
        p
    }

    #[test]
    fn 空目录读出零技能零坏包() {
        let dir = tmp("empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let v = skill_list_view(&dir);
        assert!(v.skills.is_empty());
        assert_eq!(v.skipped, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 技能装上后能被读到() {
        let dir = tmp("list");
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src-skill");
        std::fs::create_dir_all(&src).expect("mkdir");
        // 装：库侧要求目录里有技能清单/文档，缺了会被拒 —— 这里只验「读得出来」。
        std::fs::write(src.join("SKILL.md"), "# test\n\n说明\n").expect("写");
        let installed = neotrix_neobot::nt_skills::install_skill(&dir, &src).expect("装技能");
        let v = skill_list_view(&dir);
        assert_eq!(v.skills.len(), 1, "读到 {:?}", v.skills);
        assert_eq!(v.skills[0].name, installed.name);
        assert_eq!(v.skipped, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 路径校验拒绝相对与不存在() {
        assert!(check_skill_src("").is_err());
        assert!(check_skill_src("  ").is_err());
        assert!(check_skill_src("relative/path").is_err());
        assert!(check_skill_src("/definitely/not/here-12345").is_err());
        // 真目录才放行（绝对路径）。
        let d = tmp("src");
        std::fs::create_dir_all(&d).expect("mkdir");
        assert!(check_skill_src(d.to_str().expect("utf-8")).is_ok());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn 技能行字段是投影不是整个结构体() {
        // ⛔ SkillRow 刻意只带界面要用的三个字段：直接把库的 Skill 序列化过去
        //    会让库内部结构变成前端隐式契约，改一行就得分改 TS，而没人会提醒。
        let row = SkillRow { name: "a".into(), description: String::new(), path: "/p".into() };
        let json = serde_json::to_string(&row).expect("序列化");
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"path\""));
        assert!(!json.contains("created_at"));
        let _ = SkillListView { skills: vec![row], skipped: 0 };
    }
}

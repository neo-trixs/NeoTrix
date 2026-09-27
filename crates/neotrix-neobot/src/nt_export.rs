//! `nt_export` — 一键备份（审计 P1-4，local-first #5 长期保存）。
//!
//! 打包单 zip：`neobot.db`（先 WAL 检查点落盘，保证自包含）+
//! `attachments/`（只取文件名，防 zip-slip 与家目录泄漏）+
//! `MEMORY.md` + `config.json`（仅 key_env 变量名，无 secret）+
//! `manifest.json`（计数 + 时间 + 版本）。
//! 恢复 = 解压后放回 `~/.neobot/`（换机搬运即此一文件）。

use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::nt_error::NtBotError;
use crate::nt_store::NeobotStore;

/// 导出报告（CLI 直接打印）。
#[derive(Debug)]
pub struct ExportReport {
    pub path: PathBuf,
    pub db_bytes: u64,
    pub files: usize,
    pub tasks: i64,
    pub convos: i64,
    pub ledger_rows: i64,
    pub at: String,
}

/// 导出 `data_dir` 下全部资产为单 zip。
pub fn export_bundle(data_dir: &Path, out: &Path) -> Result<ExportReport, NtBotError> {
    let db_path = data_dir.join("neobot.db");
    if !db_path.is_file() {
        return Err(NtBotError::Invalid(format!(
            "no database at {} (run `neobot init` first)",
            db_path.display()
        )));
    }
    let store = NeobotStore::open(db_path.to_str().ok_or_else(|| {
        NtBotError::Invalid("data dir is not valid UTF-8".to_owned())
    })?)?;
    store.checkpoint()?;
    let (tasks, convos, ledger_rows) = store.export_counts()?;
    drop(store);

    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| NtBotError::Io(format!("mkdir {}: {e}", parent.display())))?;
        }
    }
    let file = std::fs::File::create(out)
        .map_err(|e| NtBotError::Io(format!("create {}: {e}", out.display())))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut files = 0usize;

    let add_bytes = |name: &str, bytes: &[u8], zip: &mut zip::ZipWriter<std::fs::File>| {
        zip.start_file(name, options)
            .map_err(|e| NtBotError::Io(format!("zip entry {name}: {e}")))?;
        zip.write_all(bytes)
            .map_err(|e| NtBotError::Io(format!("zip write {name}: {e}")))?;
        Ok::<(), NtBotError>(())
    };
    let mut add_file = |name: &str, path: &Path, zip: &mut zip::ZipWriter<std::fs::File>| {        match std::fs::read(path) {
            Ok(bytes) => {
                add_bytes(name, &bytes, zip)?;
                files += 1;
            }
            Err(_) => {
                // 附件本体缺失不挡导出（行在库里，恢复时可见缺件）。
            }
        }
        Ok::<(), NtBotError>(())
    };

    let db_bytes = std::fs::metadata(&db_path)
        .map_err(|e| NtBotError::Io(format!("stat db: {e}")))?
        .len();
    add_file("neobot.db", &db_path, &mut zip)?;
    add_file("MEMORY.md", &data_dir.join("MEMORY.md"), &mut zip)?;
    add_file("config.json", &data_dir.join("config.json"), &mut zip)?;

    let attach_dir = data_dir.join("attachments");
    if let Ok(entries) = std::fs::read_dir(&attach_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            // 只取文件名进包：防 zip-slip，兼防家目录绝对路径泄漏。
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            add_file(&format!("attachments/{name}"), &path, &mut zip)?;
        }
    }

    let at = chrono::Utc::now().to_rfc3339();
    let manifest = serde_json::json!({
        "app": "neobot",
        "version": env!("CARGO_PKG_VERSION"),
        "at": at,
        "tasks": tasks,
        "conversations": convos,
        "ledger_rows": ledger_rows,
        "files": files,
    });
    add_bytes(
        "manifest.json",
        serde_json::to_string_pretty(&manifest)
            .map_err(|e| NtBotError::Codec(e.to_string()))?
            .as_bytes(),
        &mut zip,
    )?;
    zip.finish()
        .map_err(|e| NtBotError::Io(format!("zip finish: {e}")))?;
    Ok(ExportReport {
        path: out.to_path_buf(),
        db_bytes,
        files,
        tasks,
        convos,
        ledger_rows,
        at,
    })
}

#[cfg(test)]
mod tests {
    use super::export_bundle;

    #[test]
    fn export_roundtrip_manifest_counts() {
        let dir = std::env::temp_dir().join("neobot-export-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("attachments")).expect("mkdir");
        // 最小资产：库（1 会话）+ 记忆 + 配置 + 1 附件。
        let db = dir.join("neobot.db");
        let store = crate::nt_store::NeobotStore::open(db.to_str().expect("utf8")).expect("open");
        store.create_conversation("group", "小队", &[]).expect("convo");
        drop(store);
        std::fs::write(dir.join("MEMORY.md"), "remember this").expect("mem");
        std::fs::write(dir.join("config.json"), "{}").expect("cfg");
        std::fs::write(dir.join("attachments").join("a.png"), b"fakepng").expect("att");

        let out = dir.join("backup.zip");
        let report = export_bundle(&dir, &out).expect("export");
        assert_eq!(report.convos, 1);
        assert!(report.files >= 4, "files={}", report.files);

        let file = std::fs::File::open(&out).expect("open zip");
        let mut archive = zip::ZipArchive::new(file).expect("read zip");
        let names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_owned()))
            .collect();
        for want in ["neobot.db", "MEMORY.md", "config.json", "attachments/a.png", "manifest.json"] {
            assert!(names.iter().any(|n| n == want), "missing {want}: {names:?}");
        }
        let mut manifest = String::new();
        std::io::Read::read_to_string(
            &mut archive.by_name("manifest.json").expect("manifest"),
            &mut manifest,
        )
        .expect("read manifest");
        let value: serde_json::Value = serde_json::from_str(&manifest).expect("json");
        assert_eq!(value["conversations"], 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_missing_db_fails_helpfully() {
        let dir = std::env::temp_dir().join("neobot-export-empty-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let err = export_bundle(&dir, &dir.join("b.zip")).expect_err("must fail");
        assert!(err.to_string().contains("neobot init"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

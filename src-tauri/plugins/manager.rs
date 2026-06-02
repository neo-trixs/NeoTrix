// NeoTrix 插件管理器
// 参考 osaurus PluginManager.swift 架构
// 实现插件发现、加载、卸载和沙箱隔离

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use chrono::Utc;
use rusqlite::Connection;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub abi_version: u8,
    pub main: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub abi_version: u8,
    pub installed_at: String,
    pub status: PluginStatus,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginStatus {
    Installed,
    Loaded,
    Error(String),
}

#[derive(Debug)]
pub struct PluginHandle {
    pub info: PluginInfo,
    #[allow(dead_code)]
    pub library: Option<libloading::Library>,
}

pub struct PluginManager {
    plugin_dir: PathBuf,
    plugins: Vec<PluginHandle>,
}

impl PluginManager {
    /// 创建新的插件管理器实例
    pub fn new() -> Self {
        let plugin_dir = Self::get_plugin_dir();
        Self::ensure_plugin_dir(&plugin_dir);

        PluginManager {
            plugin_dir,
            plugins: Vec::new(),
        }
    }

    /// 获取插件目录路径 (~/.neotrix/plugins/)
    fn get_plugin_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("plugins")
    }

    /// 确保插件目录存在
    fn ensure_plugin_dir(dir: &Path) {
        if !dir.exists() {
            fs::create_dir_all(dir).unwrap_or_else(|e| {
                eprintln!("Failed to create plugin directory: {}", e);
            });
        }
    }

    /// 发现所有已安装的插件
    pub fn discover(&mut self) -> Vec<PluginInfo> {
        self.plugins.clear();
        let mut discovered = Vec::new();

        if !self.plugin_dir.exists() {
            return discovered;
        }

        let entries = fs::read_dir(&self.plugin_dir).unwrap_or_else(|e| {
            eprintln!("Failed to read plugin directory: {}", e);
            return fs::read_dir(".").unwrap();
        });

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(info) = self.load_plugin_info(&path) {
                    discovered.push(info.clone());
                    self.plugins.push(PluginHandle {
                        info,
                        library: None,
                    });
                }
            }
        }

        discovered
    }

    /// 从指定路径加载插件信息
    fn load_plugin_info(&self, plugin_path: &Path) -> Option<PluginInfo> {
        let manifest_path = plugin_path.join("manifest.json");
        if !manifest_path.exists() {
            return None;
        }

        let manifest_content = fs::read_to_string(&manifest_path)
            .inspect_err(|e| log::warn!("[plugin] read manifest: {}", e))
            .ok()?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .inspect_err(|e| log::warn!("[plugin] parse manifest: {}", e))
            .ok()?;

        let installed_at = self.get_installation_time(plugin_path);

        Some(PluginInfo {
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            author: manifest.author,
            abi_version: manifest.abi_version,
            installed_at,
            status: PluginStatus::Installed,
            permissions: manifest.permissions,
        })
    }

    /// 获取插件安装时间
    fn get_installation_time(&self, plugin_path: &Path) -> String {
        fs::metadata(plugin_path)
            .and_then(|m| m.modified())
            .map(|t| {
                let datetime: chrono::DateTime<Utc> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            })
            .unwrap_or_else(|_| "Unknown".to_string())
    }

    /// 加载指定插件
    pub fn load(&mut self, name: &str) -> Result<(), String> {
        let plugin_path = self.plugin_dir.join(name);
        if !plugin_path.exists() {
            return Err(format!("Plugin '{}' not found", name));
        }

        let manifest_path = plugin_path.join("manifest.json");
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Invalid manifest: {}", e))?;

        if manifest.abi_version != 1 {
            return Err(format!("Unsupported ABI version: {}", manifest.abi_version));
        }

        let library_name = if cfg!(target_os = "macos") {
            format!("{}.dylib", manifest.main)
        } else if cfg!(target_os = "linux") {
            format!("{}.so", manifest.main)
        } else if cfg!(target_os = "windows") {
            format!("{}.dll", manifest.main)
        } else {
            return Err("Unsupported platform".to_string());
        };

        let library_path = plugin_path.join(&library_name);
        if !library_path.exists() {
            return Err(format!("Plugin library not found: {:?}", library_path));
        }

        unsafe {
            let library = libloading::Library::new(&library_path)
                .map_err(|e| format!("Failed to load library: {}", e))?;

            self.update_plugin_status(name, PluginStatus::Loaded);

            if let Some(handle) = self.plugins.iter_mut().find(|p| p.info.name == name) {
                handle.library = Some(library);
            } else {
                let info = PluginInfo {
                    name: manifest.name,
                    version: manifest.version,
                    description: manifest.description,
                    author: manifest.author,
                    abi_version: manifest.abi_version,
                    installed_at: self.get_installation_time(&plugin_path),
                    status: PluginStatus::Loaded,
                    permissions: manifest.permissions,
                };
                self.plugins.push(PluginHandle {
                    info,
                    library: Some(library),
                });
            }
        }

        self.init_plugin_sandbox(name)?;
        Ok(())
    }

    /// 卸载指定插件
    pub fn unload(&mut self, name: &str) -> Result<(), String> {
        if let Some(index) = self.plugins.iter().position(|p| p.info.name == name) {
            let handle = self.plugins.remove(index);
            drop(handle.library);
            self.update_plugin_status(name, PluginStatus::Installed);
            Ok(())
        } else {
            Err(format!("Plugin '{}' not loaded", name))
        }
    }

    /// 更新插件状态
    fn update_plugin_status(&mut self, name: &str, status: PluginStatus) {
        if let Some(handle) = self.plugins.iter_mut().find(|p| p.info.name == name) {
            handle.info.status = status;
        }
    }

    /// 初始化插件沙箱（独立 SQLite 数据库）
    fn init_plugin_sandbox(&self, name: &str) -> Result<(), String> {
        let plugin_data_dir = self.plugin_dir.join(name);
        let db_path = plugin_data_dir.join("data.sqlite");

        if !db_path.exists() {
            let conn = Connection::open(&db_path)
                .map_err(|e| format!("Failed to create sandbox database: {}", e))?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS plugin_data (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )
            .map_err(|e| format!("Failed to create sandbox table: {}", e))?;

            conn.execute(
                "INSERT INTO plugin_data (key, value, updated_at) VALUES (?, ?, ?)",
                ("__init__", "sandbox_initialized", &Utc::now().to_rfc3339()),
            )
            .map_err(|e| format!("Failed to initialize sandbox: {}", e))?;
        }

        Ok(())
    }

    /// 从 ZIP 文件安装插件
    pub fn install_from_zip(&mut self, zip_path: &str) -> Result<PluginInfo, String> {
        let zip_path_buf = PathBuf::from(zip_path);
        if !zip_path_buf.exists() {
            return Err(format!("ZIP file not found: {}", zip_path));
        }

        let file = fs::File::open(&zip_path_buf)
            .map_err(|e| format!("Failed to open ZIP: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

        let manifest = self.extract_and_validate_manifest(&mut archive)?;
        let plugin_dir = self.plugin_dir.join(&manifest.name);

        if plugin_dir.exists() {
            return Err(format!("Plugin '{}' already installed", manifest.name));
        }

        fs::create_dir_all(&plugin_dir)
            .map_err(|e| format!("Failed to create plugin directory: {}", e))?;

        self.extract_zip_contents(&mut archive, &plugin_dir)?;

        let info = PluginInfo {
            name: manifest.name.clone(),
            version: manifest.version,
            description: manifest.description,
            author: manifest.author,
            abi_version: manifest.abi_version,
            installed_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            status: PluginStatus::Installed,
            permissions: manifest.permissions,
        };

        self.plugins.push(PluginHandle {
            info: info.clone(),
            library: None,
        });

        Ok(info)
    }

    /// 从 ZIP 中提取并验证 manifest.json
    fn extract_and_validate_manifest(
        &self,
        archive: &mut zip::ZipArchive<fs::File>,
    ) -> Result<PluginManifest, String> {
        let manifest_file = archive
            .by_name("manifest.json")
            .map_err(|e| format!("manifest.json not found in ZIP: {}", e))?;

        let manifest: PluginManifest = serde_json::from_reader(manifest_file)
            .map_err(|e| format!("Invalid manifest.json: {}", e))?;

        if manifest.abi_version == 0 || manifest.abi_version > 1 {
            return Err(format!("Unsupported ABI version: {}", manifest.abi_version));
        }

        Ok(manifest)
    }

    /// 提取 ZIP 内容到插件目录
    fn extract_zip_contents(
        &self,
        archive: &mut zip::ZipArchive<fs::File>,
        target_dir: &Path,
    ) -> Result<(), String> {
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = target_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                }

                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    /// 卸载插件（从磁盘删除）
    pub fn uninstall(&mut self, name: &str) -> Result<(), String> {
        self.unload(name)?;

        let plugin_dir = self.plugin_dir.join(name);
        if plugin_dir.exists() {
            fs::remove_dir_all(&plugin_dir)
                .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;
        }

        Ok(())
    }

    /// 列出所有已加载的插件
    pub fn list_loaded_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins
            .iter()
            .filter(|p| matches!(p.info.status, PluginStatus::Loaded))
            .map(|p| &p.info)
            .collect()
    }

    /// 获取指定插件信息
    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins
            .iter()
            .find(|p| p.info.name == name)
            .map(|p| &p.info)
    }

    /// 写入插件数据到沙箱数据库
    pub fn write_plugin_data(
        &self,
        name: &str,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        let db_path = self.plugin_dir.join(name).join("data.sqlite");
        if !db_path.exists() {
            return Err(format!("Plugin '{}' sandbox not initialized", name));
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open sandbox database: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO plugin_data (key, value, updated_at) VALUES (?, ?, ?)",
            (key, value, Utc::now().to_rfc3339()),
        )
        .map_err(|e| format!("Failed to write plugin data: {}", e))?;

        Ok(())
    }

    /// 从沙箱数据库读取插件数据
    pub fn read_plugin_data(&self, name: &str, key: &str) -> Result<Option<String>, String> {
        let db_path = self.plugin_dir.join(name).join("data.sqlite");
        if !db_path.exists() {
            return Err(format!("Plugin '{}' sandbox not initialized", name));
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open sandbox database: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT value FROM plugin_data WHERE key = ?")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let result = stmt.query_row((key,), |row| row.get(0));

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to read plugin data: {}", e)),
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.plugin_dir.to_string_lossy().contains(".neotrix/plugins"));
    }

    #[test]
    fn test_plugin_info_serialization() {
        let info = PluginInfo {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            abi_version: 1,
            installed_at: "2024-01-01".to_string(),
            status: PluginStatus::Installed,
            permissions: vec!["read".to_string()],
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-plugin"));
    }
}

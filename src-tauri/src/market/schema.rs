//! Unified Plugin Schema — plugin.toml 解析
//!
//! 统一插件清单格式，支持 DSH 市场、GitHub、npm 等多源。

use serde::{Deserialize, Serialize};
use std::path::Path;

/// 插件 ID
pub type PluginId = String;

/// 插件来源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginSourceType {
    #[serde(rename = "dsh-market")]
    DshMarket,
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "npm")]
    Npm,
    #[serde(rename = "pypi")]
    PyPI,
    #[serde(rename = "local")]
    Local,
}

impl std::fmt::Display for PluginSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DshMarket => write!(f, "dsh-market"),
            Self::GitHub => write!(f, "github"),
            Self::Npm => write!(f, "npm"),
            Self::PyPI => write!(f, "pypi"),
            Self::Local => write!(f, "local"),
        }
    }
}

/// 插件运行时类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginRuntimeType {
    #[serde(rename = "wasm")]
    Wasm,
    #[serde(rename = "native")]
    Native,
    #[serde(rename = "domain")]
    Domain,
    #[serde(rename = "script")]
    Script,
}

/// 插件分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PluginCategory {
    #[serde(rename = "im-channel")]
    ImChannel,
    #[serde(rename = "tool")]
    Tool,
    #[serde(rename = "memory")]
    Memory,
    #[serde(rename = "perception")]
    Perception,
    #[serde(rename = "action")]
    Action,
    #[serde(rename = "cognition")]
    Cognition,
    #[serde(rename = "security")]
    Security,
    #[serde(rename = "ui")]
    Ui,
    #[serde(rename = "other")]
    #[default]
    Other,
}

impl std::fmt::Display for PluginCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImChannel => write!(f, "im-channel"),
            Self::Tool => write!(f, "tool"),
            Self::Memory => write!(f, "memory"),
            Self::Perception => write!(f, "perception"),
            Self::Action => write!(f, "action"),
            Self::Cognition => write!(f, "cognition"),
            Self::Security => write!(f, "security"),
            Self::Ui => write!(f, "ui"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// 插件权限
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginPermissions {
    /// 网络访问白名单
    #[serde(default)]
    pub network: Vec<String>,
    /// 文件系统白名单
    #[serde(default)]
    pub filesystem: Vec<String>,
    /// 能力声明
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// 环境变量
    #[serde(default)]
    pub env: Vec<String>,
}

/// 插件来源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSource {
    /// 来源类型
    #[serde(rename = "type")]
    pub source_type: PluginSourceType,
    /// 仓库/包名 (如 "xmanrui/dsh-im")
    #[serde(default)]
    pub repo: Option<String>,
    /// 资产文件名 (如 "plugin.wasm")
    #[serde(default)]
    pub asset: Option<String>,
    /// 下载 URL (直接指定)
    #[serde(default)]
    pub url: Option<String>,
    /// 版本约束
    #[serde(default)]
    pub version_constraint: Option<String>,
}

/// 插件运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRuntime {
    /// 运行时类型
    #[serde(rename = "type")]
    pub runtime_type: PluginRuntimeType,
    /// 最大内存 (MB)
    #[serde(default = "default_max_memory")]
    pub max_memory_mb: u32,
    /// 最大 CPU 时间 (ms)
    #[serde(default = "default_max_cpu")]
    pub max_cpu_ms: u32,
    /// 超时 (秒)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
    /// 每分钟最大调用次数
    #[serde(default)]
    pub max_calls_per_minute: Option<u32>,
}

fn default_max_memory() -> u32 { 128 }
fn default_max_cpu() -> u32 { 1000 }
fn default_timeout() -> u32 { 30 }

/// 插件依赖
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginDependencies {
    /// 必需依赖
    #[serde(default)]
    pub requires: Vec<String>,
    /// 冲突插件
    #[serde(default)]
    pub conflicts: Vec<String>,
    /// 可选依赖
    #[serde(default)]
    pub optional: Vec<String>,
}

/// 统一插件清单 (plugin.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// 插件元数据
    pub plugin: PluginMeta,
    /// 来源配置
    pub source: PluginSource,
    /// 权限声明
    #[serde(default)]
    pub permissions: PluginPermissions,
    /// 运行时配置
    pub runtime: PluginRuntime,
    /// 依赖声明
    #[serde(default)]
    pub dependencies: PluginDependencies,
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    /// 插件 ID (唯一标识)
    pub id: PluginId,
    /// 显示名称
    pub name: String,
    /// 版本号 (semver)
    pub version: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 许可证
    #[serde(default)]
    pub license: Option<String>,
    /// 分类
    #[serde(default)]
    pub category: PluginCategory,
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
    /// 最低运行时版本
    #[serde(default = "default_min_runtime")]
    pub min_runtime: String,
    /// 图标 URL
    #[serde(default)]
    pub icon: Option<String>,
    /// 主页 URL
    #[serde(default)]
    pub homepage: Option<String>,
    /// 仓库 URL
    #[serde(default)]
    pub repository: Option<String>,
}

fn default_min_runtime() -> String { "0.21.0".into() }

impl PluginManifest {
    /// 从 TOML 文件解析
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Read plugin manifest: {e}"))?;
        Self::from_toml(&content)
    }

    /// 从 TOML 字符串解析
    pub fn from_toml(content: &str) -> Result<Self, String> {
        toml::from_str(content)
            .map_err(|e| format!("Parse plugin manifest: {e}"))
    }

    /// 转换为 TOML 字符串
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self)
            .map_err(|e| format!("Serialize plugin manifest: {e}"))
    }

    /// 从 JSON 文件解析 (兼容旧格式)
    pub fn from_json_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Read plugin manifest: {e}"))?;
        Self::from_json(&content)
    }

    /// 从 JSON 字符串解析
    pub fn from_json(content: &str) -> Result<Self, String> {
        let json: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| format!("Parse JSON: {e}"))?;

        // 转换为统一格式
        let manifest = serde_json::json!({
            "plugin": {
                "id": json["name"].as_str().unwrap_or("unknown"),
                "name": json["name"].as_str().unwrap_or("Unknown"),
                "version": json["version"].as_str().unwrap_or("0.0.0"),
                "description": json["description"].as_str().unwrap_or(""),
                "author": json["author"].as_str().unwrap_or("Unknown"),
                "license": json["license"].as_str(),
                "category": "other",
                "tags": json["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()).unwrap_or_default(),
                "min_runtime": "0.21.0",
            },
            "source": {
                "type": "local",
                "repo": null,
                "asset": json["main"].as_str(),
                "url": null,
            },
            "permissions": {
                "network": [],
                "filesystem": [],
                "capabilities": json["permissions"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()).unwrap_or_default(),
            },
            "runtime": {
                "type": "native",
                "max_memory_mb": 128,
                "max_cpu_ms": 1000,
                "timeout_secs": 30,
            },
            "dependencies": {
                "requires": [],
                "conflicts": [],
            },
        });

        serde_json::from_value(manifest)
            .map_err(|e| format!("Convert JSON manifest: {e}"))
    }
}

/// 已安装插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    /// 插件清单
    pub manifest: PluginManifest,
    /// 安装路径
    pub install_path: String,
    /// 安装时间
    pub installed_at: u64,
    /// 启用状态
    pub enabled: bool,
    /// 加载状态
    pub loaded: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 市场条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEntry {
    /// 插件 ID
    pub id: PluginId,
    /// 显示名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 分类
    pub category: String,
    /// 标签
    pub tags: Vec<String>,
    /// 下载次数
    pub downloads: u64,
    /// 评分 (0-5)
    pub rating: f32,
    /// 来源类型
    pub source_type: String,
    /// 来源仓库
    pub source_repo: Option<String>,
    /// 图标 URL
    pub icon: Option<String>,
    /// 主页 URL
    pub homepage: Option<String>,
    /// 最新版本
    pub latest_version: String,
}

/// 插件资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAsset {
    /// 资产名称
    pub name: String,
    /// 下载 URL
    pub url: String,
    /// 文件大小 (bytes)
    pub size: u64,
    /// SHA256 哈希
    pub sha256: Option<String>,
    /// 目标平台
    pub platform: Option<String>,
}

/// 市场搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSearchResult {
    /// 搜索结果
    pub entries: Vec<MarketEntry>,
    /// 总数
    pub total: usize,
    /// 页码
    pub page: usize,
    /// 每页数量
    pub per_page: usize,
    /// 来源
    pub source: String,
}

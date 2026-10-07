//! 桌宠内容 —— 宠物清单 / 资源包导入 / 精灵图供给。
//!
//! # 与上游的关系
//!
//! 协议与校验语义取自 `dsh-harness-desktop 0.19.1` 的
//! `src-tauri/src/bridge/pet.rs`（MIT）：包上限、manifest 规则、
//! 网格推断、错误码字面量（`PET_*`）全部对齐 —— 前端按这些字面量
//! 分支，改一个词就是改一次前端行为。
//!
//! # 本文件只做纯函数
//!
//! 全部函数收显式 `root: &Path`（`<data_dir>/pets/<source>`），
//! 不读环境、不碰 Tauri —— `data_dir` 的拼法是 app 壳的事
//! （`apps/neobot-desktop`），库里拼路径等于把「桌面建的 CLI 看不见」
//! 那类 bug 再造一次。调用方见 app 侧 `pet.rs`。
//!
//! # 安全边界（与上游同）
//!
//! ① 包 32MB / 条目 512 / 解后 128MB 三道 cap；② zip 内链接条目直接拒
//! （`enclosed_name` 防 zip-slip 只是第一层）；③ manifest id 限
//! `1..=64 ascii [a-z0-9-_]`；④ 相对路径拒绝 `\`、`:`、NUL、绝对路径与 `..`。
//! 任何一条放宽，导入包就从「换皮肤」变成「写任意文件」。

use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};

/// 导入包上限（压缩后）。与上游同。
pub const PET_PACKAGE_MAX_BYTES: usize = 32 * 1024 * 1024;
/// 导入包条目上限。与上游同。
pub const PET_PACKAGE_MAX_ENTRIES: usize = 512;
/// 导入包解后上限。与上游同。
pub const PET_PACKAGE_MAX_UNCOMPRESSED_BYTES: u64 = 128 * 1024 * 1024;
/// `pet.json` 上限。与上游同。
pub const PET_MANIFEST_MAX_BYTES: u64 = 64 * 1024;
/// 精灵图上限。与上游同。
pub const PET_SPRITESHEET_MAX_BYTES: u64 = 8 * 1024 * 1024;
const PET_SPRITE_V1: u8 = 1;
const PET_SPRITE_V2: u8 = 2;
const PET_SPRITE_COLUMNS: u8 = 8;
const PET_SPRITE_V1_ROWS: u8 = 9;
const PET_SPRITE_V2_ROWS: u8 = 11;
const PET_SPRITESHEET_MAX_DIMENSION: u32 = 16_384;
const PET_SPRITESHEET_MAX_PIXELS: u64 = 64 * 1024 * 1024;

/// 宠物来源（目录名即取值）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetSource {
    Chat,
    Codex,
}

impl PetSource {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "chat" => Ok(Self::Chat),
            "codex" => Ok(Self::Codex),
            _ => Err("PET_SOURCE_INVALID: source must be chat|codex".to_owned()),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Codex => "codex",
        }
    }
}

/// `pet.json` 受支持字段（camelCase；与上游同）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetManifest {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sprite_version_number: Option<u8>,
    pub spritesheet_path: String,
}

/// 列表项（来源限定 id，避免 chat/codex 同名覆盖；与上游同形）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PetListItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub source: String,
}

/// 精灵图资产（data URL 直渲染；与上游同形）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PetAsset {
    pub id: String,
    pub spritesheet: String,
    pub sprite_version_number: u8,
    pub columns: u8,
    pub rows: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpriteGrid {
    version: u8,
    columns: u8,
    rows: u8,
}

pub fn valid_manifest_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn validate_manifest_id(id: &str) -> Result<(), String> {
    if valid_manifest_id(id) {
        Ok(())
    } else {
        Err("PET_ID_INVALID: manifest id must be 1..=64 ascii letters/digits/-/_".to_owned())
    }
}

pub fn qualified_id(source: PetSource, manifest_id: &str) -> String {
    format!("{}:{manifest_id}", source.as_str())
}

/// `source:manifest_id` 解析（`get_pet_asset` 的 id 形态；与上游同）。
pub fn parse_qualified_id(id: &str) -> Result<(PetSource, &str), String> {
    let (source, manifest_id) = id
        .split_once(':')
        .ok_or_else(|| "PET_ID_INVALID: filesystem pet id must be source-qualified".to_owned())?;
    let source = PetSource::parse(source)?;
    validate_manifest_id(manifest_id)?;
    Ok((source, manifest_id))
}

/// 相对路径守卫（与上游 `safe_relative_path` 同规则）。
fn safe_relative_path(value: &str) -> Result<PathBuf, String> {
    if value.is_empty() || value.contains('\0') || value.contains('\\') || value.contains(':') {
        return Err("PET_PATH_INVALID: path must be a plain relative path".to_owned());
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err("PET_PATH_INVALID: absolute paths are not allowed".to_owned());
    }
    for comp in path.components() {
        match comp {
            Component::Normal(_) => {}
            _ => return Err("PET_PATH_INVALID: .. and prefixes are not allowed".to_owned()),
        }
    }
    Ok(path.to_path_buf())
}

/// 跟随符号链接后仍必须留在目录内（与上游 `contained_file` 同）。
fn contained_file(directory: &Path, relative: &str) -> Result<PathBuf, String> {
    let target = directory.join(safe_relative_path(relative)?);
    // 链接不存在时 canonicalize 报错 ⇒ 按「不在目录内」处理，不区分。
    let canon_dir = directory.canonicalize().unwrap_or_else(|_| directory.to_path_buf());
    let canon_target = target.canonicalize().map_err(|_| {
        "PET_PATH_INVALID: target does not exist inside pet directory".to_owned()
    })?;
    if !canon_target.starts_with(&canon_dir) {
        return Err("PET_PATH_INVALID: target escapes pet directory".to_owned());
    }
    Ok(canon_target)
}

fn read_bounded_file(path: &Path, cap: u64, code: &str) -> Result<Vec<u8>, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("{code}: {e}"))?;
    let mut buf = Vec::new();
    // take +1：读满 cap+1 即超限（与「正好 cap」区分）。
    let n = f
        .take(cap + 1)
        .read_to_end(&mut buf)
        .map_err(|e| format!("{code}: {e}"))?;
    if n as u64 > cap {
        return Err(format!("{code}: file exceeds {cap} bytes"));
    }
    Ok(buf)
}

fn parse_manifest_bytes(bytes: &[u8]) -> Result<PetManifest, String> {
    let manifest: PetManifest = serde_json::from_slice(bytes)
        .map_err(|e| format!("PET_MANIFEST_INVALID: invalid pet.json: {e}"))?;
    validate_manifest_id(&manifest.id)?;
    if let Some(version) = manifest.sprite_version_number {
        if version != PET_SPRITE_V1 && version != PET_SPRITE_V2 {
            return Err(format!(
                "PET_SPRITE_VERSION_UNSUPPORTED: spriteVersionNumber must be {PET_SPRITE_V1} or {PET_SPRITE_V2}"
            ));
        }
    }
    safe_relative_path(&manifest.spritesheet_path)?;
    Ok(manifest)
}

fn read_manifest(directory: &Path) -> Result<PetManifest, String> {
    let bytes = read_bounded_file(
        &directory.join("pet.json"),
        PET_MANIFEST_MAX_BYTES,
        "PET_MANIFEST_READ_FAILED",
    )?;
    parse_manifest_bytes(&bytes)
}

/// 图集尺寸 sniff（PNG IHDR + WebP VP8X/VP8L/VP8；与上游同规则）。
///
/// ⛔ 只读文件头，不解码像素 —— 8MB 的图集解码一次只为量尺寸是浪费，
/// 且解码器是比头解析大得多的攻击面。
/// 从字节流读大端 `u32`（PNG IHDR 的宽/高）。
///
/// ⚠️ 刻意用 `get(..)` 而非 `[..]` 索引：**切片越界本身就是 panic**，
/// `try_into().unwrap()` 是**第二层** panic ⇒ 两层都去掉。
fn be32_at(bytes: &[u8], at: usize) -> Option<u32> {
    bytes
        .get(at..at + 4)
        .and_then(|s| s.try_into().ok())
        .map(u32::from_be_bytes)
}

fn spritesheet_dimensions(
    bytes: &[u8],
    declared: Option<u8>,
) -> Result<(&'static str, SpriteGrid), String> {
    let (mime, width, height) = if bytes.len() >= 24 && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        if bytes.get(12..16) != Some(b"IHDR".as_slice()) {
            return Err("PET_ASSET_FORMAT_INVALID: PNG is missing IHDR".to_owned());
        }
        (
            "image/png",
            // ⚠️ 原为 `bytes[16..20].try_into().unwrap()`：
            //   ⛔ **切片本身也会 panic**（越界），`unwrap` 只是第二层 panic。
            //   ⇒ 用 `get(..)` + `ok_or(..)?`：越界返回 `Err`，与函数签名一致。
            be32_at(bytes, 16).ok_or("PET_ASSET_FORMAT_INVALID: PNG 宽字段越界")?,
            be32_at(bytes, 20).ok_or("PET_ASSET_FORMAT_INVALID: PNG 高字段越界")?,
        )
    } else if bytes.len() >= 30 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        let chunk = &bytes[12..16];
        let (width, height) = match chunk {
            b"VP8X" if bytes.len() >= 30 => (
                1 + u32::from(bytes[24]) + (u32::from(bytes[25]) << 8) + (u32::from(bytes[26]) << 16),
                1 + u32::from(bytes[27]) + (u32::from(bytes[28]) << 8) + (u32::from(bytes[29]) << 16),
            ),
            b"VP8L" if bytes.len() >= 25 && bytes[20] == 0x2f => (
                1 + u32::from(bytes[21]) + ((u32::from(bytes[22]) & 0x3f) << 8),
                1 + (u32::from(bytes[22]) >> 6)
                    + (u32::from(bytes[23]) << 2)
                    + ((u32::from(bytes[24]) & 0x0f) << 10),
            ),
            b"VP8 " if bytes.len() >= 30 && bytes[23..26] == [0x9d, 0x01, 0x2a] => (
                u32::from(u16::from_le_bytes([bytes[26], bytes[27]]) & 0x3fff),
                u32::from(u16::from_le_bytes([bytes[28], bytes[29]]) & 0x3fff),
            ),
            _ => {
                return Err(
                    "PET_ASSET_FORMAT_INVALID: unsupported or malformed WebP header".to_owned(),
                )
            }
        };
        ("image/webp", width, height)
    } else {
        return Err("PET_ASSET_FORMAT_INVALID: spritesheet must be PNG or WebP".to_owned());
    };

    let pixels = u64::from(width) * u64::from(height);
    if width == 0
        || height == 0
        || width > PET_SPRITESHEET_MAX_DIMENSION
        || height > PET_SPRITESHEET_MAX_DIMENSION
        || pixels > PET_SPRITESHEET_MAX_PIXELS
    {
        return Err(format!(
            "PET_ASSET_DIMENSIONS_INVALID: spritesheet dimensions exceed {PET_SPRITESHEET_MAX_DIMENSION}px or {PET_SPRITESHEET_MAX_PIXELS} pixels"
        ));
    }
    Ok((mime, sprite_grid(declared, width, height)?))
}

/// 由图集高度定网格（声明只在两布局都整除时消歧；与上游同）。
fn sprite_grid(declared: Option<u8>, width: u32, height: u32) -> Result<SpriteGrid, String> {
    if !width.is_multiple_of(u32::from(PET_SPRITE_COLUMNS)) {
        return Err(format!(
            "PET_ASSET_DIMENSIONS_INVALID: spritesheet width must be divisible by {PET_SPRITE_COLUMNS} columns"
        ));
    }
    let v1 = height.is_multiple_of(u32::from(PET_SPRITE_V1_ROWS));
    let v2 = height.is_multiple_of(u32::from(PET_SPRITE_V2_ROWS));
    let version = match (v1, v2) {
        (false, false) => {
            return Err(format!(
                "PET_ASSET_DIMENSIONS_INVALID: spritesheet height must be divisible by {PET_SPRITE_V1_ROWS} (v1) or {PET_SPRITE_V2_ROWS} (v2) rows"
            ))
        }
        (true, false) => PET_SPRITE_V1,
        (false, true) => PET_SPRITE_V2,
        (true, true) => declared.unwrap_or(PET_SPRITE_V2),
    };
    Ok(SpriteGrid {
        version,
        columns: PET_SPRITE_COLUMNS,
        rows: if version == PET_SPRITE_V1 {
            PET_SPRITE_V1_ROWS
        } else {
            PET_SPRITE_V2_ROWS
        },
    })
}

fn read_spritesheet(
    directory: &Path,
    relative: &str,
    declared: Option<u8>,
) -> Result<(String, SpriteGrid), String> {
    let path = contained_file(directory, relative)?;
    let bytes = read_bounded_file(&path, PET_SPRITESHEET_MAX_BYTES, "PET_ASSET_READ_FAILED")?;
    let (mime, grid) = spritesheet_dimensions(&bytes, declared)?;
    Ok((
        format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)),
        grid,
    ))
}

fn immediate_pet_directories(root: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("PET_ROOT_READ_FAILED: {e}")),
    };
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("PET_ROOT_READ_FAILED: {e}"))?;
        let path = entry.path();
        // ⛔ 跟随链接判定：链接进来的目录不算宠物目录（与上游 `path_exists_including_symlink` 同纪律）。
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir && !path.is_symlink() {
            out.push(path);
        }
    }
    Ok(out)
}

fn manifest_to_list_item(source: PetSource, directory: &Path, manifest: PetManifest) -> PetListItem {
    let name = manifest
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(&manifest.id)
        .to_owned();
    let description = manifest
        .description
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());
    // ⛔ 缩略图失败即无缩略图，不拦整条列表 —— 一张坏图不该让十个宠物消失。
    let thumbnail = read_spritesheet(directory, &manifest.spritesheet_path, manifest.sprite_version_number)
        .map(|(url, _)| url)
        .ok();
    PetListItem {
        id: qualified_id(source, &manifest.id),
        name,
        description,
        thumbnail,
        source: source.as_str().to_owned(),
    }
}

/// 列出某来源的宠物（坏 manifest 的目录跳过；与上游 `list_pets` 同策略）。
///
/// ⛔ 跳过不是吞错：`read_manifest` 失败的目录不是宠物（缺 pet.json），
///
/// 列出来才是错 —— 界面会显示一个点不开的东西。
pub fn list_pets(root: &Path, source: PetSource) -> Result<Vec<PetListItem>, String> {
    let mut items = Vec::new();
    let mut ids = std::collections::HashSet::new();
    for directory in immediate_pet_directories(root)? {
        let Ok(manifest) = read_manifest(&directory) else {
            continue;
        };
        let id = qualified_id(source, &manifest.id);
        if ids.insert(id) {
            items.push(manifest_to_list_item(source, &directory, manifest));
        }
    }
    items.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(items)
}

/// 按限定 id 找宠物目录（与上游 `find_pet_directory` 同）。
pub fn find_pet_directory(root: &Path, manifest_id: &str) -> Result<Option<(PathBuf, PetManifest)>, String> {
    for directory in immediate_pet_directories(root)? {
        let Ok(manifest) = read_manifest(&directory) else {
            continue;
        };
        if manifest.id == manifest_id {
            return Ok(Some((directory, manifest)));
        }
    }
    Ok(None)
}

/// 读精灵图资产（data URL 直渲染；与上游 `get_pet_asset` 同）。
pub fn pet_asset(root: &Path, source: PetSource, manifest_id: &str) -> Result<PetAsset, String> {
    let (directory, manifest) = find_pet_directory(root, manifest_id)?.ok_or_else(|| {
        format!("PET_NOT_FOUND: pet {} was not found", qualified_id(source, manifest_id))
    })?;
    let (spritesheet, grid) = read_spritesheet(
        &directory,
        &manifest.spritesheet_path,
        manifest.sprite_version_number,
    )?;
    Ok(PetAsset {
        id: qualified_id(source, &manifest.id),
        spritesheet,
        sprite_version_number: grid.version,
        columns: grid.columns,
        rows: grid.rows,
    })
}

/// 导入资源包（base64 zip；与上游 `import_pet` 同规则）。
///
/// 包内必须有且仅有一个 `pet.json`（包装目录不限层）；同 id 已存在即拒；
/// 暂存目录失败即清，不留 `.staging` 垃圾。
pub fn import_pet(root: &Path, name: &str, data: &str) -> Result<PetListItem, String> {
    // 前端协议仍带文件名，但它是不可信展示数据（Unicode/空格），只用于报错文案。
    let _ = name;
    let encoded_limit = PET_PACKAGE_MAX_BYTES.div_ceil(3) * 4;
    if data.len() > encoded_limit {
        return Err(format!(
            "PET_PACKAGE_TOO_LARGE: pet package must not exceed {PET_PACKAGE_MAX_BYTES} compressed bytes"
        ));
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data.as_bytes())
        .map_err(|e| format!("PET_PACKAGE_DECODE_FAILED: invalid base64 payload: {e}"))?;
    if bytes.len() > PET_PACKAGE_MAX_BYTES {
        return Err(format!(
            "PET_PACKAGE_TOO_LARGE: pet package must not exceed {PET_PACKAGE_MAX_BYTES} compressed bytes"
        ));
    }

    let staging_root = root.join(".staging");
    std::fs::create_dir_all(&staging_root)
        .map_err(|e| format!("PET_STAGING_FAILED: {e}"))?;
    let staging = staging_root.join(uuid::Uuid::new_v4().to_string());
    let result = import_pet_staged(root, &staging, &bytes);
    // ⛔ 暂存无论成败都清：失败留下的半包下次会被当成宠物目录扫描。
    let _ = std::fs::remove_dir_all(&staging);
    result
}

fn import_pet_staged(root: &Path, staging: &Path, bytes: &[u8]) -> Result<PetListItem, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("PET_ARCHIVE_INVALID: {e}"))?;
    if archive.len() > PET_PACKAGE_MAX_ENTRIES {
        return Err(format!(
            "PET_ARCHIVE_TOO_MANY_ENTRIES: at most {PET_PACKAGE_MAX_ENTRIES} entries"
        ));
    }
    let mut total_uncompressed: u64 = 0;
    let mut entries: Vec<(PathBuf, bool)> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("PET_ARCHIVE_INVALID: {e}"))?;
        // ⛔ 链接条目直接拒（`enclosed_name` 只防 `..`，不防链接本身）。
        let enclosed = file
            .enclosed_name()
            .ok_or_else(|| "PET_ARCHIVE_LINK_FORBIDDEN: archive links are not allowed".to_owned())?;
        if let Some(mode) = file.unix_mode() {
            let file_type = mode & 0o170000;
            if file_type != 0 && file_type != 0o040000 && file_type != 0o100000 {
                return Err("PET_ARCHIVE_LINK_FORBIDDEN: archive links are not allowed".to_owned());
            }
        }
        let is_dir = file.is_dir();
        let rel = enclosed.to_path_buf();
        if is_dir {
            std::fs::create_dir_all(staging.join(&rel))
                .map_err(|e| format!("PET_STAGING_FAILED: {e}"))?;
        } else {
            total_uncompressed = total_uncompressed.saturating_add(file.size());
            if total_uncompressed > PET_PACKAGE_MAX_UNCOMPRESSED_BYTES {
                return Err(format!(
                    "PET_ARCHIVE_TOO_LARGE: uncompressed payload exceeds {PET_PACKAGE_MAX_UNCOMPRESSED_BYTES} bytes"
                ));
            }
            if let Some(parent) = staging.join(&rel).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("PET_STAGING_FAILED: {e}"))?;
            }
            // take +1：与 read_bounded_file 同一手法，区分「正好 cap」与超限。
            let mut out = std::fs::File::create(staging.join(&rel))
                .map_err(|e| format!("PET_STAGING_FAILED: {e}"))?;
            let mut limited = file.take(PET_PACKAGE_MAX_UNCOMPRESSED_BYTES + 1);
            let mut buf = [0u8; 8192];
            let mut written: u64 = 0;
            loop {
                let n = limited.read(&mut buf).map_err(|e| format!("PET_ARCHIVE_INVALID: {e}"))?;
                if n == 0 {
                    break;
                }
                written += n as u64;
                if written > PET_PACKAGE_MAX_UNCOMPRESSED_BYTES {
                    return Err(format!(
                        "PET_ARCHIVE_TOO_LARGE: uncompressed payload exceeds {PET_PACKAGE_MAX_UNCOMPRESSED_BYTES} bytes"
                    ));
                }
                out.write_all(&buf[..n])
                    .map_err(|e| format!("PET_STAGING_FAILED: {e}"))?;
            }
        }
        entries.push((rel, is_dir));
    }

    // 包内有且仅有一个 pet.json（包装目录不限；元数据条目忽略；与上游同）。
    let mut manifests = Vec::new();
    for (rel, is_dir) in &entries {
        if *is_dir || ignored_archive_entry(rel) {
            continue;
        }
        if rel.file_name().and_then(|v| v.to_str()) == Some("pet.json") {
            manifests.push(rel.clone());
        }
    }
    if manifests.len() != 1 {
        return Err("PET_MANIFEST_INVALID: package must contain exactly one pet.json".to_owned());
    }
    let manifest_rel = &manifests[0];
    let manifest_dir = staging.join(manifest_rel.parent().unwrap_or_else(|| Path::new("")));
    let manifest_bytes = read_bounded_file(
        &manifest_dir.join("pet.json"),
        PET_MANIFEST_MAX_BYTES,
        "PET_MANIFEST_READ_FAILED",
    )?;
    let manifest = parse_manifest_bytes(&manifest_bytes)?;
    // 精灵图必须在包里可读 —— 读不出来等于导入一个点不开的宠物。
    let _ = read_spritesheet(
        &manifest_dir,
        &manifest.spritesheet_path,
        manifest.sprite_version_number,
    )?;
    let target = root.join(&manifest.id);
    if target.exists() || target.is_symlink() {
        return Err(format!("PET_ALREADY_IMPORTED: pet {} already exists", manifest.id));
    }
    if find_pet_directory(root, &manifest.id)?.is_some() {
        return Err(format!("PET_ALREADY_IMPORTED: pet {} already exists", manifest.id));
    }
    std::fs::rename(&manifest_dir, &target).map_err(|e| format!("PET_IMPORT_FAILED: {e}"))?;
    Ok(PetListItem {
        id: format!("codex:{}", manifest.id),
        name: manifest.display_name.unwrap_or_else(|| manifest.id.clone()),
        description: manifest.description,
        thumbnail: None,
        source: "codex".to_owned(),
    })
}

/// 归档里与宠物无关的条目（与上游 `ignored_archive_entry` 同）。
fn ignored_archive_entry(path: &Path) -> bool {
    path.components().any(|component| {
        let Component::Normal(part) = component else {
            return true;
        };
        let name = part.to_string_lossy();
        matches!(
            name.as_ref(),
            "__MACOSX" | ".DS_Store" | "Thumbs.db" | "desktop.ini" | ".staging"
        ) || name.starts_with("._")
    })
}

/// 内置海豚播种：`pets/codex/neobot-dolphin/` 缺席即写 pet.json + 精灵图。
///
/// ⛔ 种子只写一次：已存在（用户删了重装/自己改过）绝不覆盖 ——
/// 覆盖等于把用户的修改回滚掉，而调用方以为只是「确保存在」.
///
/// ⛔ 静态单帧特例：种子图是方形图标，不是 8 列动画表，
/// 落盘时不走动画网格校验（方图过不了 8×9/11 整除）。
/// 这是对上游校验的有意偏离：上游只有动画表，没有静态图；
/// 硬把方图塞进网格会切出碎帧。1×1 在组件里能否渲染**待目检**
/// （与 WKWebView 真机验证同一缺口），但落盘形状可测。
pub fn seed_dolphin(root: &Path, png_bytes: &[u8]) -> Result<bool, String> {
    let target = root.join("neobot-dolphin");
    if target.exists() || target.is_symlink() {
        return Ok(false);
    }
    // PNG 头先验：非 PNG/坏头直接拒，不写半个种子。
    if png_bytes.len() < 24 || !png_bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("PET_SEED_FAILED: seed image is not a PNG".to_owned());
    }
    let staging = root.join(".staging-seed");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| format!("PET_SEED_FAILED: {e}"))?;
    let result = (|| -> Result<bool, String> {
        std::fs::write(staging.join("spritesheet.png"), png_bytes)
            .map_err(|e| format!("PET_SEED_FAILED: {e}"))?;
        let manifest = PetManifest {
            id: "neobot-dolphin".to_owned(),
            display_name: Some("海豚".to_owned()),
            description: Some("NeoBot 内置".to_owned()),
            sprite_version_number: Some(PET_SPRITE_V1),
            spritesheet_path: "spritesheet.png".to_owned(),
        };
        let body = serde_json::to_string_pretty(&pet_manifest_json(&manifest))
            .map_err(|e| format!("PET_SEED_FAILED: {e}"))?;
        std::fs::write(staging.join("pet.json"), body)
            .map_err(|e| format!("PET_SEED_FAILED: {e}"))?;
        // 回读只验 manifest 合法（id/路径），不验动画网格 —— 静态单帧无网格可言。
        let _ = read_manifest(&staging)?;
        std::fs::rename(&staging, &target).map_err(|e| format!("PET_SEED_FAILED: {e}"))?;
        Ok(true)
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    result
}

/// 种子资产的静态网格（1×1：整张即一帧；与 `seed_dolphin` 落盘配套）。
///
/// ⛔ 这不是推断出来的，是声明：种子图没有动画网格可言。
/// 调用方（app 侧 `get_pet_asset`）对种子 id 走这条，不走 `pet_asset` 的网格校验.
pub fn seed_dolphin_grid() -> (u8, u8, u8) {
    (PET_SPRITE_V1, 1, 1)
}

/// `PetManifest` 的 camelCase JSON 形（种子写盘用；读侧由 serde 担）。
fn pet_manifest_json(m: &PetManifest) -> serde_json::Value {
    serde_json::json!({
        "id": m.id,
        "displayName": m.display_name,
        "description": m.description,
        "spriteVersionNumber": m.sprite_version_number,
        "spritesheetPath": m.spritesheet_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn tmp_root() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "nt-pet-test-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("建临时目录");
        p
    }

    /// 最小 PNG（只要 IHDR 头；本模块只读头，不解码像素）。
    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut v = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
        v.extend_from_slice(&13u32.to_be_bytes());
        v.extend_from_slice(b"IHDR");
        v.extend_from_slice(&width.to_be_bytes());
        v.extend_from_slice(&height.to_be_bytes());
        v.extend_from_slice(&[8, 2, 0, 0, 0]);
        v
    }

    fn write_pet(dir: &Path, id: &str, width: u32, height: u32) {
        std::fs::create_dir_all(dir).expect("建宠物目录");
        std::fs::write(
            dir.join("pet.json"),
            serde_json::json!({
                "id": id,
                "displayName": id,
                "spriteVersionNumber": 1,
                "spritesheetPath": "spritesheet.png",
            })
            .to_string(),
        )
        .expect("写 manifest");
        std::fs::write(dir.join("spritesheet.png"), png_bytes(width, height)).expect("写图");
    }

    #[test]
    fn manifest_id字符集与上游一致() {
        assert!(valid_manifest_id("neobot-dolphin"));
        assert!(valid_manifest_id("a"));
        assert!(!valid_manifest_id(""));
        assert!(!valid_manifest_id("a/b"));
        assert!(!valid_manifest_id("a:b"));
        assert!(!valid_manifest_id("有中文"));
        assert!(!valid_manifest_id(&"x".repeat(65)));
    }

    #[test]
    fn 限定id解析() {
        assert_eq!(parse_qualified_id("codex:abc").expect("解析").0, PetSource::Codex);
        assert!(parse_qualified_id("abc").is_err());
        assert!(parse_qualified_id("ssh:abc").is_err());
    }

    #[test]
    fn 网格按高度消歧() {
        // 宽 8 整除；高 99 = 9×11，两布局都整除 ⇒ 缺省 v2。
        let g = sprite_grid(None, 8, 99).expect("网格");
        assert_eq!((g.version, g.columns, g.rows), (2, 8, 11));
        // 高 90 = 9×10 ⇒ v1。
        let g = sprite_grid(None, 8, 90).expect("网格");
        assert_eq!((g.version, g.columns, g.rows), (1, 8, 9));
        // 宽不整除 ⇒ 拒。
        assert!(sprite_grid(None, 7, 90).is_err());
        // 高都不整除 ⇒ 拒。
        assert!(sprite_grid(None, 8, 91).is_err());
    }

    #[test]
    fn 列表跳过坏目录但不报错() {
        let root = tmp_root();
        write_pet(&root.join("good"), "good", 8, 90);
        std::fs::create_dir_all(root.join("empty")).expect("空目录");
        std::fs::write(root.join("junk.txt"), "x").expect("杂文件");
        let items = list_pets(&root, PetSource::Codex).expect("列表");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "codex:good");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn 资产读出data_url与网格() {
        let root = tmp_root();
        write_pet(&root.join("good"), "good", 8, 90);
        let asset = pet_asset(&root, PetSource::Codex, "good").expect("资产");
        assert_eq!(asset.id, "codex:good");
        assert_eq!((asset.sprite_version_number, asset.columns, asset.rows), (1, 8, 9));
        assert!(asset.spritesheet.starts_with("data:image/png;base64,"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn 越界路径被拒() {
        assert!(safe_relative_path("../x").is_err());
        assert!(safe_relative_path("/etc/passwd").is_err());
        assert!(safe_relative_path("a\\b").is_err());
        assert!(safe_relative_path("a:b").is_err());
        assert!(safe_relative_path("ok/dir").is_ok());
    }

    #[test]
    fn 种子只写一次且可被列表读到() {
        let root = tmp_root();
        // 非 PNG 直接拒。
        assert!(seed_dolphin(&root, b"nope").is_err());
        let png = png_bytes(32, 32);
        assert_eq!(seed_dolphin(&root, &png).expect("播种"), true);
        // 第二次是 Ok(false)，不是覆盖。
        assert_eq!(seed_dolphin(&root, &png).expect("重播"), false);
        let items = list_pets(&root, PetSource::Codex).expect("列表");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "codex:neobot-dolphin");
        assert_eq!(items[0].name, "海豚");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn 导入包往返_装得进列得出读得出() {
        use std::io::Write as _;
        let root = tmp_root();
        // 现场造一个合法包（Stored 免压缩；pet.json 在根）。
        let manifest = serde_json::json!({
            "id": "roundtrip",
            "displayName": "往返",
            "spriteVersionNumber": 1,
            "spritesheetPath": "sheet.png",
        })
        .to_string();
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut w = zip::ZipWriter::new(&mut buf);
            w.start_file("pet.json", zip::write::SimpleFileOptions::default())
                .expect("写 manifest");
            w.write_all(manifest.as_bytes()).expect("写内容");
            w.start_file("sheet.png", zip::write::SimpleFileOptions::default())
                .expect("写图");
            w.write_all(&png_bytes(8, 90)).expect("写像素");
            w.finish().expect("封包");
        }
        let data = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());
        let item = import_pet(&root, "x.zip", &data).expect("导入");
        assert_eq!(item.id, "codex:roundtrip");
        // 重导同 id 被拒。
        assert!(import_pet(&root, "x.zip", &data).is_err());
        let items = list_pets(&root, PetSource::Codex).expect("列表");
        assert_eq!(items.len(), 1);
        let asset = pet_asset(&root, PetSource::Codex, "roundtrip").expect("资产");
        assert_eq!((asset.sprite_version_number, asset.columns, asset.rows), (1, 8, 9));
        assert!(asset.spritesheet.starts_with("data:image/png;base64,"));
        let _ = std::fs::remove_dir_all(&root);
    }
}

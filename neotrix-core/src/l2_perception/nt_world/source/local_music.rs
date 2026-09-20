//! 本地音乐管理模块 — 本地音频文件扫描、索引与搜索
//!
//! 整合 go-music-dl 的本地音乐管理能力:
//! - 扫描本地音频文件 (mp3/flac/m4a/ogg/wav/wma/aac)
//! - 元数据提取 (标题/歌手/专辑/时长)
//! - 封面/歌词读取
//! - SQLite 索引加速搜索

use crate::l2_perception::nt_world::source::types::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// 支持的音频格式
pub const SUPPORTED_FORMATS: &[&str] = &[
    "mp3", "flac", "m4a", "ogg", "wav", "wma", "aac", "opus",
];

/// 本地音乐条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMusicEntry {
    pub path: String,
    pub file_name: String,
    pub file_size: u64,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<Duration>,
    pub format: String,
    pub cover_path: Option<String>,
    pub lyric_path: Option<String>,
    pub last_modified: i64,
}

impl LocalMusicEntry {
    /// 转换为 MediaItem
    pub fn to_media_item(&self) -> MediaItem {
        MediaItem {
            id: format!("local:{}", self.path),
            title: self.title.clone().unwrap_or_else(|| {
                Path::new(&self.file_name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            }),
            artist: self.artist.clone().unwrap_or_else(|| "Unknown".into()),
            album: self.album.clone().unwrap_or_default(),
            duration: self.duration,
            cover_url: self.cover_path.clone().map(|p| format!("file://{}", p)),
            media_type: MediaType::Audio,
            qualities: vec![Quality::Standard],
        }
    }
}

/// 本地音乐扫描器
pub struct LocalMusicScanner {
    root_dir: PathBuf,
}

impl LocalMusicScanner {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// 扫描目录下的所有音频文件
    pub fn scan(&self) -> Result<Vec<LocalMusicEntry>, String> {
        let mut entries = Vec::new();
        self.scan_recursive(&self.root_dir, &mut entries)?;
        Ok(entries)
    }

    /// 递归扫描
    fn scan_recursive(&self, dir: &Path, entries: &mut Vec<LocalMusicEntry>) -> Result<(), String> {
        if !dir.exists() {
            return Ok(());
        }

        let read_dir = std::fs::read_dir(dir).map_err(|e| format!("Read dir failed: {}", e))?;

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_dir() {
                self.scan_recursive(&path, entries)?;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if SUPPORTED_FORMATS.contains(&ext.to_lowercase().as_str()) {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        let file_name = path.file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();

                        // 查找同名封面
                        let cover_path = self.find_cover(&path);
                        // 查找同名歌词
                        let lyric_path = self.find_lyric(&path);

                        entries.push(LocalMusicEntry {
                            path: path.to_string_lossy().to_string(),
                            file_name,
                            file_size: metadata.len(),
                            title: None, // 需要 ffprobe 或 tag 解析
                            artist: None,
                            album: None,
                            duration: None,
                            format: ext.to_lowercase(),
                            cover_path,
                            lyric_path,
                            last_modified: metadata.modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// 查找同名封面图片
    fn find_cover(&self, audio_path: &Path) -> Option<String> {
        let stem = audio_path.file_stem()?;
        let parent = audio_path.parent()?;

        let cover_extensions = ["jpg", "jpeg", "png", "webp", "bmp"];
        for ext in cover_extensions {
            let cover_path = parent.join(format!("{}.{}", stem.to_str()?, ext));
            if cover_path.exists() {
                return Some(cover_path.to_string_lossy().to_string());
            }
        }
        None
    }

    /// 查找同名歌词文件
    fn find_lyric(&self, audio_path: &Path) -> Option<String> {
        let stem = audio_path.file_stem()?;
        let parent = audio_path.parent()?;

        let lyric_extensions = ["lrc", "txt", "lyric"];
        for ext in lyric_extensions {
            let lyric_path = parent.join(format!("{}.{}", stem.to_str()?, ext));
            if lyric_path.exists() {
                return Some(lyric_path.to_string_lossy().to_string());
            }
        }
        None
    }

    /// 搜索本地音乐
    pub fn search<'a>(&self, query: &str, entries: &'a [LocalMusicEntry]) -> Vec<&'a LocalMusicEntry> {
        let query_lower = query.to_lowercase();
        entries.iter()
            .filter(|e| {
                let title_match = e.title.as_ref()
                    .map(|t| t.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);
                let file_match = e.file_name.to_lowercase().contains(&query_lower);
                let artist_match = e.artist.as_ref()
                    .map(|a| a.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);
                title_match || file_match || artist_match
            })
            .collect()
    }
}

/// 本地音乐索引 (SQLite)
pub struct LocalMusicIndex {
    db_path: PathBuf,
}

impl LocalMusicIndex {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// 初始化数据库表
    pub fn init(&self) -> Result<(), String> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Open DB failed: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS local_music (
                path TEXT PRIMARY KEY,
                file_name TEXT,
                file_size INTEGER,
                title TEXT,
                artist TEXT,
                album TEXT,
                duration INTEGER,
                format TEXT,
                cover_path TEXT,
                lyric_path TEXT,
                last_modified INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_local_music_title ON local_music(title);
            CREATE INDEX IF NOT EXISTS idx_local_music_artist ON local_music(artist);"
        ).map_err(|e| format!("Create table failed: {}", e))?;

        Ok(())
    }

    /// 批量插入/更新
    pub fn upsert_batch(&self, entries: &[LocalMusicEntry]) -> Result<usize, String> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Open DB failed: {}", e))?;

        let mut count = 0;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO local_music 
             (path, file_name, file_size, title, artist, album, duration, format, cover_path, lyric_path, last_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        ).map_err(|e| format!("Prepare failed: {}", e))?;

        for entry in entries {
            let duration_secs = entry.duration.map(|d| d.as_secs() as i64).unwrap_or(0);
            stmt.execute(rusqlite::params![
                entry.path,
                entry.file_name,
                entry.file_size,
                entry.title,
                entry.artist,
                entry.album,
                duration_secs,
                entry.format,
                entry.cover_path,
                entry.lyric_path,
                entry.last_modified,
            ]).map_err(|e| format!("Insert failed: {}", e))?;
            count += 1;
        }

        Ok(count)
    }

    /// 搜索
    pub fn search(&self, query: &str) -> Result<Vec<LocalMusicEntry>, String> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Open DB failed: {}", e))?;

        let query_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT path, file_name, file_size, title, artist, album, duration, format, cover_path, lyric_path, last_modified
             FROM local_music 
             WHERE title LIKE ?1 OR artist LIKE ?1 OR file_name LIKE ?1
             LIMIT 100"
        ).map_err(|e| format!("Prepare failed: {}", e))?;

        let entries = stmt.query_map(rusqlite::params![query_pattern], |row| {
            let duration_secs: i64 = row.get(6)?;
            Ok(LocalMusicEntry {
                path: row.get(0)?,
                file_name: row.get(1)?,
                file_size: row.get(2)?,
                title: row.get(3)?,
                artist: row.get(4)?,
                album: row.get(5)?,
                duration: if duration_secs > 0 { Some(Duration::from_secs(duration_secs as u64)) } else { None },
                format: row.get(7)?,
                cover_path: row.get(8)?,
                lyric_path: row.get(9)?,
                last_modified: row.get(10)?,
            })
        }).map_err(|e| format!("Query failed: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

        Ok(entries)
    }

    /// 删除记录
    pub fn remove(&self, path: &str) -> Result<bool, String> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Open DB failed: {}", e))?;

        let affected = conn.execute("DELETE FROM local_music WHERE path = ?1", rusqlite::params![path])
            .map_err(|e| format!("Delete failed: {}", e))?;

        Ok(affected > 0)
    }

    /// 获取总数
    pub fn count(&self) -> Result<usize, String> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Open DB failed: {}", e))?;

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM local_music", [], |row| row.get(0))
            .map_err(|e| format!("Count failed: {}", e))?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_formats() {
        assert!(SUPPORTED_FORMATS.contains(&"mp3"));
        assert!(SUPPORTED_FORMATS.contains(&"flac"));
        assert!(SUPPORTED_FORMATS.contains(&"m4a"));
    }

    #[test]
    fn test_local_music_entry_to_media_item() {
        let entry = LocalMusicEntry {
            path: "/music/test.mp3".into(),
            file_name: "test.mp3".into(),
            file_size: 1024,
            title: Some("Test Song".into()),
            artist: Some("Test Artist".into()),
            album: Some("Test Album".into()),
            duration: Some(Duration::from_secs(180)),
            format: "mp3".into(),
            cover_path: None,
            lyric_path: None,
            last_modified: 0,
        };

        let item = entry.to_media_item();
        assert_eq!(item.id, "local:/music/test.mp3");
        assert_eq!(item.title, "Test Song");
        assert_eq!(item.artist, "Test Artist");
    }
}

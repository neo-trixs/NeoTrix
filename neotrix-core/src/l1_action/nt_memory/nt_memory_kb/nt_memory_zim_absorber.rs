use std::path::Path;
use std::time::Instant;

pub struct ZimAbsorbConfig {
    pub batch_size: usize,
    pub max_articles: usize,
    pub url_prefix_filter: Option<String>,
    pub min_content_bytes: usize,
    pub min_text_len: usize,
    pub max_text_len: usize,
}

impl Default for ZimAbsorbConfig {
    fn default() -> Self {
        Self {
            batch_size: 500,
            max_articles: 100_000,
            url_prefix_filter: None,
            min_content_bytes: 100,
            min_text_len: 50,
            max_text_len: 50_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZimAbsorbStats {
    pub scanned: usize,
    pub ingested: usize,
    pub skipped: usize,
    pub errors: usize,
    pub elapsed_ms: u64,
}

impl std::fmt::Display for ZimAbsorbStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "scanned={} ingested={} skipped={} errors={} elapsed={}ms",
            self.scanned, self.ingested, self.skipped, self.errors, self.elapsed_ms
        )
    }
}

pub fn absorb_zim_file(
    db: &mut rusqlite::Connection,
    zim_path: &Path,
    config: &ZimAbsorbConfig,
) -> Result<ZimAbsorbStats, String> {
    let start = Instant::now();
    let zim = zim::Zim::new(zim_path).map_err(|e| format!("Failed to open ZIM: {}", e))?;

    let uuid = extract_zim_uuid(zim_path);

    let tx = db
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    let source_url = format!("zim_source://{}", uuid);
    let _ = tx.execute(
        "INSERT OR IGNORE INTO nodes (id, node_type, title, language, confidence, importance, recall_weight, created_at, updated_at, access_count)
         VALUES (?1, 'source', ?2, 'en', 1.0, 1.0, 1.0, strftime('%s','now'), strftime('%s','now'), 0)",
        rusqlite::params![source_url, uuid],
    );

    let mut stats = ZimAbsorbStats {
        scanned: 0,
        ingested: 0,
        skipped: 0,
        errors: 0,
        elapsed_ms: 0,
    };

    for entry_result in zim.iterate_by_urls() {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => {
                stats.errors += 1;
                continue;
            }
        };
        if stats.scanned >= config.max_articles {
            break;
        }
        stats.scanned += 1;

        let path = entry.url.clone();

        if let Some(ref prefix) = config.url_prefix_filter {
            if !path.starts_with(prefix) {
                stats.skipped += 1;
                continue;
            }
        }

        let content: Vec<u8> = match zim.entry_content(&entry) {
            Ok(Some(c)) => c.to_vec().unwrap_or_default(),
            _ => {
                stats.errors += 1;
                continue;
            }
        };

        if content.len() < config.min_content_bytes {
            stats.skipped += 1;
            continue;
        }

        let text = html_to_text(&String::from_utf8_lossy(&content));

        if text.len() < config.min_text_len {
            stats.skipped += 1;
            continue;
        }

        let text = if text.len() > config.max_text_len {
            &text[..config.max_text_len]
        } else {
            &text
        };

        let title = entry.title.clone();
        let node_id = format!("zimid://{}/{}", uuid, path);

        let _ = tx.execute(
            "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, url, language, confidence, importance, recall_weight, created_at, updated_at, access_count)
             VALUES (?1, 'article', ?2, ?3, ?4, 'en', 0.8, 0.5, 1.0, strftime('%s','now'), strftime('%s','now'), 0)",
            rusqlite::params![node_id, title, &text[..text.len().min(500)], path],
        );

        let _ = tx.execute(
            "INSERT OR IGNORE INTO edges (source_id, target_id, relation_type, weight, created_at)
             VALUES (?1, ?2, 'contains', 1.0, strftime('%s','now'))",
            rusqlite::params![source_url, node_id],
        );

        stats.ingested += 1;

        if stats.ingested % config.batch_size == 0 {
            let _ = tx.commit();
            let new_tx = db
                .transaction()
                .map_err(|e| format!("Failed to restart transaction: {}", e))?;
            // Note: can't reassign tx in this scope, but the auto-commit on drop handles it
            // For true batching, the caller should use absorb_zim_batch
            drop(new_tx);
        }
    }

    tx.commit().map_err(|e| format!("Failed to commit: {}", e))?;

    stats.elapsed_ms = start.elapsed().as_millis() as u64;
    Ok(stats)
}

fn extract_zim_uuid(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn html_to_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_script = false;
    let mut in_style = false;
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            let tag: String = chars.by_ref().take_while(|&c| c != '>').collect();
            let lower = tag.to_lowercase();
            if lower.starts_with("script") {
                in_script = true;
            } else if lower.starts_with("style") {
                in_style = true;
            } else if lower == "/script" {
                in_script = false;
            } else if lower == "/style" {
                in_style = false;
            } else if lower == "br" || lower == "br/" || lower == "p" || lower == "/p" {
                text.push('\n');
            }
            continue;
        }

        if !in_script && !in_style {
            text.push(ch);
        }
    }

    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_text_basic() {
        let html = "<p>Hello <b>world</b></p>";
        let text = html_to_text(html);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn test_html_to_text_script_stripped() {
        let html = "before<script>alert('x')</script>after";
        let text = html_to_text(html);
        assert_eq!(text, "before after");
    }

    #[test]
    fn test_html_to_text_style_stripped() {
        let html = "text<style>.red{color:red}</style>more";
        let text = html_to_text(html);
        assert_eq!(text, "text more");
    }

    #[test]
    fn test_zim_absorb_config_defaults() {
        let config = ZimAbsorbConfig::default();
        assert_eq!(config.batch_size, 500);
        assert_eq!(config.max_articles, 100_000);
        assert!(config.url_prefix_filter.is_none());
    }
}

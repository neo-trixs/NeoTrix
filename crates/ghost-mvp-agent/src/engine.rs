use std::process::Command;

use crate::config::Config;

/// Run the analyze.sh script on a GitHub repo.
/// Returns (patterns_found, capabilities_proposed, content_angles, report_path)
pub async fn analyze_repo(
    config: &Config,
    repo_url: &str,
    _client: &reqwest::Client,
) -> Result<(Vec<String>, Vec<String>, Vec<String>, Option<String>), String> {
    let skills_dir = config
        .skills_dir
        .as_ref()
        .ok_or("skills_dir not configured")?;
    let analyze_script = skills_dir.join("scripts/analyze.sh");
    if !analyze_script.exists() {
        return Err(format!("analyze.sh not found at {}", analyze_script.display()));
    }

    let output_dir = std::env::current_dir()
        .unwrap_or_default()
        .join("analysis");
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("mkdir: {e}"))?;

    let output = Command::new("bash")
        .arg(&analyze_script)
        .arg(repo_url)
        .arg("--output")
        .arg(output_dir.to_str().unwrap_or("analysis"))
        .output()
        .map_err(|e| format!("failed to run analyze.sh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("analyze.sh failed: {stderr}"));
    }

    // Extract repo name from URL for the report path
    let repo_name = repo_url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("unknown");

    let report_path = output_dir.join(repo_name).join("04-synthesis.md");

    let (patterns, capabilities, angles) = if report_path.exists() {
        let content = std::fs::read_to_string(&report_path).unwrap_or_default();
        let patterns: Vec<String> = content
            .lines()
            .filter(|l| l.contains("|") && l.contains("TBD"))
            .map(|l| l.to_string())
            .collect();
        let capabilities = vec!["synthesized from analysis".to_string()];
        let angles = vec!["generated from patterns".to_string()];
        (patterns, capabilities, angles)
    } else {
        (vec![], vec![], vec![])
    };

    Ok((patterns, capabilities, angles, Some(report_path.to_string_lossy().to_string())))
}

/// Placeholder for LLM content generation.
/// In production, this calls an LLM API with persona + analysis context.
pub async fn generate_content(
    _config: &Config,
    topic: &str,
    platform: &str,
    _client: &reqwest::Client,
) -> Result<String, String> {
    Ok(format!(
        "[ghost-mvp] Generated content for {platform} on topic: {topic}\n\
         (LLM integration pending — edit engine.rs to wire your provider)"
    ))
}

/// Publish content to a platform.
pub async fn publish_to_platform(
    config: &Config,
    platform: &str,
    content: &str,
    client: &reqwest::Client,
) -> Result<String, String> {
    match platform {
        "x" | "bluesky" | "linkedin" => {
            if let Some(key) = &config.letmepost_api_key {
                let resp = client
                    .post("https://api.letmepost.dev/v1/posts")
                    .header("Authorization", format!("Bearer {key}"))
                    .json(&serde_json::json!({
                        "targets": [{"platform": platform}],
                        "text": content
                    }))
                    .send()
                    .await
                    .map_err(|e| format!("letmepost request failed: {e}"))?;
                let body = resp.text().await.unwrap_or_default();
                Ok(body)
            } else {
                Ok(format!("[dry-run] would publish to {platform}:\n{content}"))
            }
        }
        "zhihu" | "xhs" | "bilibili" | "weibo" | "douyin" => {
            if config.hui_mei_enabled {
                let output = Command::new("huimei")
                    .arg("publish")
                    .arg("--platform")
                    .arg(platform)
                    .arg("--title")
                    .arg("content title")
                    .arg("--desc")
                    .arg(content)
                    .output()
                    .map_err(|e| format!("huimei failed: {e}"))?;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(stdout)
            } else {
                Ok(format!("[dry-run] would publish to {platform} via HuiMei:\n{content}"))
            }
        }
        "dev.to" => {
            if let Some(key) = &config.devto_api_key {
                let resp = client
                    .post("https://dev.to/api/articles")
                    .header("api-key", key)
                    .json(&serde_json::json!({
                        "article": {
                            "title": "ghost-mvp generated",
                            "body_markdown": content,
                            "published": true,
                            "tags": ["ai", "opensource"]
                        }
                    }))
                    .send()
                    .await
                    .map_err(|e| format!("dev.to request failed: {e}"))?;
                let body = resp.text().await.unwrap_or_default();
                Ok(body)
            } else {
                Ok(format!("[dry-run] would publish to dev.to:\n{content}"))
            }
        }
        _ => Err(format!("unsupported platform: {platform}")),
    }
}

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEntry {
    pub platform: String,
    pub username: String,
    pub url: String,
    pub exists: bool,
    pub name_display: Option<String>,
    pub bio: Option<String>,
    pub follower_count: Option<u64>,
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonFindings {
    pub username: Option<String>,
    pub email: Option<String>,
    pub profiles: Vec<ProfileEntry>,
}

impl std::fmt::Display for PersonFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── Person / Username OSINT ──")?;
        if let Some(ref u) = self.username { writeln!(f, "    Username:   {}", u)?; }
        if let Some(ref e) = self.email { writeln!(f, "    Email:      {}", e)?; }
        writeln!(f, "    Profiles:   {}", self.profiles.len())?;
        for profile in &self.profiles {
            writeln!(f, "      {:<15} {}", profile.platform, profile.url)?;
            if let Some(ref bio) = profile.bio {
                let short: String = bio.chars().take(80).collect();
                writeln!(f, "      {short}")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PlatformCheck {
    name: &'static str,
    url_fn: fn(username: &str) -> String,
}

const PLATFORMS: &[PlatformCheck] = &[
    PlatformCheck { name: "GitHub", url_fn: |u| format!("https://github.com/{u}") },
    PlatformCheck { name: "Twitter/X", url_fn: |u| format!("https://twitter.com/{u}") },
    PlatformCheck { name: "LinkedIn", url_fn: |u| format!("https://linkedin.com/in/{u}") },
    PlatformCheck { name: "Reddit", url_fn: |u| format!("https://reddit.com/user/{u}") },
    PlatformCheck { name: "HackerNews", url_fn: |u| format!("https://news.ycombinator.com/user?id={u}") },
    PlatformCheck { name: "StackOverflow", url_fn: |u| format!("https://stackoverflow.com/users?user={u}") },
    PlatformCheck { name: "Medium", url_fn: |u| format!("https://medium.com/@{u}") },
    PlatformCheck { name: "Dev.to", url_fn: |u| format!("https://dev.to/{u}") },
    PlatformCheck { name: "Keybase", url_fn: |u| format!("https://keybase.io/{u}") },
    PlatformCheck { name: "BitBucket", url_fn: |u| format!("https://bitbucket.org/{u}") },
    PlatformCheck { name: "GitLab", url_fn: |u| format!("https://gitlab.com/{u}") },
    PlatformCheck { name: "YouTube", url_fn: |u| format!("https://youtube.com/@{u}") },
    PlatformCheck { name: "Telegram", url_fn: |u| format!("https://t.me/{u}") },
    PlatformCheck { name: "Instagram", url_fn: |u| format!("https://instagram.com/{u}") },
    PlatformCheck { name: "Pinterest", url_fn: |u| format!("https://pinterest.com/{u}") },
    PlatformCheck { name: "TikTok", url_fn: |u| format!("https://tiktok.com/@{u}") },
    PlatformCheck { name: "Twitch", url_fn: |u| format!("https://twitch.tv/{u}") },
    PlatformCheck { name: "Discord", url_fn: |u| format!("https://discord.com/users/{u}") },
    PlatformCheck { name: "ProductHunt", url_fn: |u| format!("https://producthunt.com/@{u}") },
    PlatformCheck { name: "AngelList/Wellfound", url_fn: |u| format!("https://wellfound.com/u/{u}") },
    PlatformCheck { name: "Crunchbase", url_fn: |u| format!("https://crunchbase.com/person/{u}") },
];

async fn check_platform(username: &str, platform: &PlatformCheck, client: &Client) -> ProfileEntry {
    let url = (platform.url_fn)(username);
    let exists = match client.get(&url).timeout(Duration::from_secs(5)).send().await {
        Ok(resp) => resp.status().as_u16() < 400,
        Err(_) => false,
    };
    ProfileEntry {
        platform: platform.name.to_string(),
        username: username.to_string(),
        url,
        exists,
        name_display: None,
        bio: None,
        follower_count: None,
        source: "profile-check".to_string(),
    }
}

pub async fn investigate(target: &OsintTarget, client: &Client, config: &OsintConfig) -> Result<PersonFindings, String> {
    let username = target.username.as_deref().or_else(|| {
        target.email.as_ref().and_then(|e| e.split('@').next())
    }).ok_or("no username or email specified")?;

    let mut findings = PersonFindings {
        username: Some(username.to_string()),
        email: target.email.clone(),
        ..Default::default()
    };

    // Check all platforms
    let mut handles = Vec::new();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(config.concurrency.max(5)));

    for platform in PLATFORMS {
        let s = semaphore.clone();
        let u = username.to_string();
        let c = client.clone();
        handles.push(tokio::spawn(async move {
            let _permit = s.acquire().await.expect("semaphore is never closed");
            check_platform(&u, platform, &c).await
        }));
    }

    for handle in handles {
        if let Ok(profile) = handle.await {
            findings.profiles.push(profile);
        }
    }

    // Remove non-existent profiles
    findings.profiles.retain(|p| p.exists);

    // Sort by platform name
    findings.profiles.sort_by(|a, b| a.platform.cmp(&b.platform));

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_checks_count() {
        assert!(PLATFORMS.len() >= 15);
    }

    #[test]
    fn test_platform_urls() {
        for platform in PLATFORMS {
            let url = (platform.url_fn)("testuser");
            assert!(url.contains("testuser"), "platform {} url missing username: {}", platform.name, url);
        }
    }

    #[test]
    fn test_github_url() {
        let url = (PLATFORMS[0].url_fn)("octocat");
        assert_eq!(url, "https://github.com/octocat");
    }

    #[test]
    fn test_person_findings_default() {
        let f = PersonFindings::default();
        assert!(f.profiles.is_empty());
    }

    #[test]
    fn test_person_findings_display_empty() {
        let f = PersonFindings::default();
        let s = format!("{f}");
        assert!(s.contains("Person"));
    }

    #[test]
    fn test_profile_entry_new() {
        let p = ProfileEntry {
            platform: "GitHub".to_string(),
            username: "test".to_string(),
            url: "https://github.com/test".to_string(),
            exists: true,
            name_display: None,
            bio: None,
            follower_count: None,
            source: "profile-check".to_string(),
        };
        assert!(p.exists);
    }
}

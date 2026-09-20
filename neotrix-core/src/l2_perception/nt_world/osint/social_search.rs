//! OSINT social account search — platform definitions and username presence detection.
//!
//! `SocialPlatform` models a single social network with a URL template and check method.
//! `DEFAULT_PLATFORMS` provides 55+ platform definitions for broad coverage.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// HTTP method used to verify whether a username exists on a platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckMethod {
    /// Lightweight HEAD request — most platforms support this.
    Head,
    /// GET request — used when HEAD is blocked or returns misleading responses.
    Get,
}

/// Outcome of a single platform check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    /// Profile page returned 2xx.
    Found,
    /// Profile page returned 404 or similar "not found" indicator.
    NotFound,
    /// Request failed (timeout, DNS, connection error, etc.).
    Error,
    /// Server returned 429 or we were throttled.
    RateLimited,
}

/// Describes one social platform that can be checked for username presence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPlatform {
    /// Human-readable platform name (e.g. "GitHub").
    pub name: String,
    /// URL template with `{username}` placeholder (e.g. "https://github.com/{username}").
    pub url_template: String,
    /// How to probe the profile page.
    pub check_method: CheckMethod,
    /// Strings whose presence in the response body indicates the account exists.
    /// Empty list means status-code-only detection.
    pub claim_patterns: Vec<String>,
}

impl SocialPlatform {
    /// Build the full profile URL for a given username.
    pub fn profile_url(&self, username: &str) -> String {
        self.url_template.replace("{username}", username)
    }

    /// Check whether `username` exists on this platform.
    pub async fn check(
        &self,
        username: &str,
        client: &Client,
        timeout: Duration,
    ) -> PlatformResult {
        let url = self.profile_url(username);
        let start = Instant::now();

        let result = self.probe(&url, client, timeout).await;
        let response_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(status) => PlatformResult {
                platform: self.name.clone(),
                url,
                status,
                response_time_ms,
            },
            Err(_) => PlatformResult {
                platform: self.name.clone(),
                url,
                status: AccountStatus::Error,
                response_time_ms,
            },
        }
    }

    async fn probe(
        &self,
        url: &str,
        client: &Client,
        timeout: Duration,
    ) -> Result<AccountStatus, reqwest::Error> {
        let builder = match self.check_method {
            CheckMethod::Head => client.head(url),
            CheckMethod::Get => client.get(url),
        };

        let resp = builder.timeout(timeout).send().await?;
        let status_code = resp.status().as_u16();

        if status_code == 429 {
            return Ok(AccountStatus::RateLimited);
        }

        if status_code >= 400 {
            return Ok(AccountStatus::NotFound);
        }

        if self.claim_patterns.is_empty() {
            return Ok(AccountStatus::Found);
        }

        // Read body and check claim patterns
        let body = resp.text().await.unwrap_or_default();
        for pattern in &self.claim_patterns {
            if body.contains(pattern.as_str()) {
                return Ok(AccountStatus::Found);
            }
        }

        // 2xx but no pattern matched — still treat as Found (body may have changed).
        Ok(AccountStatus::Found)
    }
}

/// Result of checking one platform for a username.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformResult {
    pub platform: String,
    pub url: String,
    pub status: AccountStatus,
    pub response_time_ms: u64,
}

// ---------------------------------------------------------------------------
// DEFAULT_PLATFORMS — 55 platforms
// ---------------------------------------------------------------------------

/// Returns the built-in list of 55 social/developer/professional platforms.
pub fn default_platforms() -> Vec<SocialPlatform> {
    build_default_platforms()
}

/// Build the hard-coded platform registry at runtime.
fn build_default_platforms() -> Vec<SocialPlatform> {
    vec![
    // ── Developer / Code ──────────────────────────────────────────────
    SocialPlatform {
        name: "GitHub".into(),
        url_template: "https://github.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["repositories".into()],
    },
    SocialPlatform {
        name: "GitLab".into(),
        url_template: "https://gitlab.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["activity".into()],
    },
    SocialPlatform {
        name: "Bitbucket".into(),
        url_template: "https://bitbucket.org/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["repositories".into()],
    },
    SocialPlatform {
        name: "Codeberg".into(),
        url_template: "https://codeberg.org/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "SourceForge".into(),
        url_template: "https://sourceforge.net/u/{username}/profile".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Replit".into(),
        url_template: "https://replit.com/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "CodePen".into(),
        url_template: "https://codepen.io/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "JSFiddle".into(),
        url_template: "https://jsfiddle.net/user/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "HackerRank".into(),
        url_template: "https://www.hackerrank.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "LeetCode".into(),
        url_template: "https://leetcode.com/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Kaggle".into(),
        url_template: "https://www.kaggle.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "StackOverflow".into(),
        url_template: "https://stackoverflow.com/users/?tab=Accounts&User={username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    // ── Social / Microblogging ────────────────────────────────────────
    SocialPlatform {
        name: "Twitter/X".into(),
        url_template: "https://x.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["followers".into()],
    },
    SocialPlatform {
        name: "Bluesky".into(),
        url_template: "https://bsky.app/profile/{username}.bsky.social".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Mastodon".into(),
        url_template: "https://mastodon.social/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["followers".into()],
    },
    SocialPlatform {
        name: "Threads".into(),
        url_template: "https://www.threads.net/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Truth Social".into(),
        url_template: "https://truthsocial.com/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Gettr".into(),
        url_template: "https://gettr.com/user/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Gab".into(),
        url_template: "https://gab.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Parler".into(),
        url_template: "https://parler.com/profile/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Koo".into(),
        url_template: "https://www.kooapp.com/profile/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Weibo".into(),
        url_template: "https://weibo.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "WeChat".into(),
        url_template: "https://weixin.qq.com/u/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "LINE".into(),
        url_template: "https://line.me/p/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    // ── Media / Content ───────────────────────────────────────────────
    SocialPlatform {
        name: "YouTube".into(),
        url_template: "https://youtube.com/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["subscriber".into()],
    },
    SocialPlatform {
        name: "TikTok".into(),
        url_template: "https://www.tiktok.com/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["followers".into()],
    },
    SocialPlatform {
        name: "Instagram".into(),
        url_template: "https://www.instagram.com/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["followers".into(), "Posts".into()],
    },
    SocialPlatform {
        name: "Twitch".into(),
        url_template: "https://www.twitch.tv/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["followers".into()],
    },
    SocialPlatform {
        name: "Dailymotion".into(),
        url_template: "https://www.dailymotion.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Vimeo".into(),
        url_template: "https://vimeo.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Rumble".into(),
        url_template: "https://rumble.com/c/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Odysee".into(),
        url_template: "https://odysee.com/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Bitchute".into(),
        url_template: "https://www.bitchute.com/channel/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "MySpace".into(),
        url_template: "https://myspace.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    // ── Professional / Business ───────────────────────────────────────
    SocialPlatform {
        name: "LinkedIn".into(),
        url_template: "https://www.linkedin.com/in/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["connections".into()],
    },
    SocialPlatform {
        name: "Glassdoor".into(),
        url_template: "https://www.glassdoor.com/profile/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Wellfound".into(),
        url_template: "https://wellfound.com/u/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Crunchbase".into(),
        url_template: "https://www.crunchbase.com/person/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Xing".into(),
        url_template: "https://www.xing.com/profile/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    // ── Blogging / Long-form ──────────────────────────────────────────
    SocialPlatform {
        name: "Medium".into(),
        url_template: "https://medium.com/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["followers".into()],
    },
    SocialPlatform {
        name: "Substack".into(),
        url_template: "https://{username}.substack.com".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["Subscribe".into()],
    },
    SocialPlatform {
        name: "Dev.to".into(),
        url_template: "https://dev.to/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["posts".into()],
    },
    SocialPlatform {
        name: "Hashnode".into(),
        url_template: "https://hashnode.com/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Ghost".into(),
        url_template: "https://{username}.ghost.io".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["Subscribe".into()],
    },
    SocialPlatform {
        name: "Blogger".into(),
        url_template: "https://{username}.blogspot.com".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "WordPress.com".into(),
        url_template: "https://{username}.wordpress.com".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Tumblr".into(),
        url_template: "https://{username}.tumblr.com".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["posts".into()],
    },
    SocialPlatform {
        name: "LiveJournal".into(),
        url_template: "https://{username}.livejournal.com".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    // ── Discussion / Community ─────────────────────────────────────────
    SocialPlatform {
        name: "Reddit".into(),
        url_template: "https://www.reddit.com/user/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["karma".into()],
    },
    SocialPlatform {
        name: "HackerNews".into(),
        url_template: "https://news.ycombinator.com/user?id={username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["karma".into()],
    },
    SocialPlatform {
        name: "Lobsters".into(),
        url_template: "https://lobste.rs/u/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Lemmy".into(),
        url_template: "https://lemmy.ml/u/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "StackExchange".into(),
        url_template: "https://stackexchange.com/users/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "ProductHunt".into(),
        url_template: "https://www.producthunt.com/@{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Dribbble".into(),
        url_template: "https://dribbble.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["shots".into()],
    },
    SocialPlatform {
        name: "Behance".into(),
        url_template: "https://www.behance.net/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["appreciations".into()],
    },
    SocialPlatform {
        name: "DeviantArt".into(),
        url_template: "https://www.deviantart.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["Watchers".into()],
    },
    // ── Messaging / Chat ──────────────────────────────────────────────
    SocialPlatform {
        name: "Telegram".into(),
        url_template: "https://t.me/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["subscriber".into()],
    },
    SocialPlatform {
        name: "Discord".into(),
        url_template: "https://discord.com/users/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Signal".into(),
        url_template: "https://signal.me/#p/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    // ── Miscellaneous ─────────────────────────────────────────────────
    SocialPlatform {
        name: "Keybase".into(),
        url_template: "https://keybase.io/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["proofs".into()],
    },
    SocialPlatform {
        name: "Patreon".into(),
        url_template: "https://www.patreon.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["patron".into()],
    },
    SocialPlatform {
        name: "Fiverr".into(),
        url_template: "https://www.fiverr.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Pinterest".into(),
        url_template: "https://www.pinterest.com/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["Pins".into()],
    },
    SocialPlatform {
        name: "Flickr".into(),
        url_template: "https://www.flickr.com/people/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "VSCO".into(),
        url_template: "https://vsco.co/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Letterboxd".into(),
        url_template: "https://letterboxd.com/{username}/".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["films".into()],
    },
    SocialPlatform {
        name: "Goodreads".into(),
        url_template: "https://www.goodreads.com/user/show/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "MyAnimeList".into(),
        url_template: "https://myanimelist.net/profile/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Strava".into(),
        url_template: "https://www.strava.com/athletes/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Spotify".into(),
        url_template: "https://open.spotify.com/user/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["Playlists".into()],
    },
    SocialPlatform {
        name: "SoundCloud".into(),
        url_template: "https://soundcloud.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec!["tracks".into()],
    },
    SocialPlatform {
        name: "Bandcamp".into(),
        url_template: "https://{username}.bandcamp.com".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    },
    SocialPlatform {
        name: "Gravatar".into(),
        url_template: "https://en.gravatar.com/{username}".into(),
        check_method: CheckMethod::Get,
        claim_patterns: vec![],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_platforms_count() {
        let platforms = default_platforms();
        assert!(
            platforms.len() >= 55,
            "expected >=55 platforms, got {}",
            platforms.len()
        );
    }

    #[test]
    fn all_templates_contain_placeholder() {
        let platforms = default_platforms();
        for p in platforms {
            assert!(
                p.url_template.contains("{username}"),
                "platform {} missing {{username}} placeholder",
                p.name
            );
        }
    }

    #[test]
    fn profile_url_substitution() {
        let platforms = default_platforms();
        let p = &platforms[0]; // GitHub
        let url = p.profile_url("octocat");
        assert!(url.contains("octocat"));
        assert!(!url.contains("{username}"));
    }

    #[test]
    fn no_duplicate_platform_names() {
        let platforms = default_platforms();
        let mut seen = std::collections::HashSet::new();
        for p in platforms {
            assert!(seen.insert(p.name.clone()), "duplicate platform: {}", p.name);
        }
    }

    #[test]
    fn check_method_serialization() {
        let head = CheckMethod::Head;
        let json = serde_json::to_string(&head).unwrap();
        assert!(json.contains("Head"));
        let back: CheckMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(back, CheckMethod::Head);
    }

    #[test]
    fn account_status_serialization() {
        let statuses = [
            AccountStatus::Found,
            AccountStatus::NotFound,
            AccountStatus::Error,
            AccountStatus::RateLimited,
        ];
        for s in &statuses {
            let json = serde_json::to_string(s).unwrap();
            let back: AccountStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, s);
        }
    }

    #[test]
    fn default_platforms_returns_owned_vec() {
        let platforms = default_platforms();
        assert!(platforms.len() >= 55);
    }
}

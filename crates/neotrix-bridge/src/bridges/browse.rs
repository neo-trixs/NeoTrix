use crate::types::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[derive(Debug, Clone)]
struct PageModel {
    url: String,
    title: String,
    text: String,
    links: Vec<String>,
    scroll_position: u32,
    content_density: f64,
}

impl PageModel {
    fn new(url: &str) -> Self {
        let domain = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("unknown");
        Self {
            url: url.to_string(),
            title: format!("Page at {}", domain),
            text: format!("Simulated page content for {}. This page contains domain-specific knowledge about {}.", url, domain),
            links: vec![
                format!("https://{}/about", domain),
                format!("https://{}/research", domain),
                format!("https://{}/papers", domain),
                format!("https://{}/blog", domain),
            ],
            scroll_position: 0,
            content_density: 0.3 + (domain.len() as f64 * 0.02).min(0.6),
        }
    }

    fn extract_text(&self, selector: &str) -> String {
        match selector {
            "title" | "h1" => self.title.clone(),
            "body" | "main" | "article" | "content" => self.text.clone(),
            "links" | "a" => self.links.join("\n"),
            _ => format!(
                "<{}> matched {} characters of content",
                selector,
                self.text.len()
            ),
        }
    }

    fn search_results(query: &str) -> Vec<String> {
        let terms: Vec<&str> = query.split_whitespace().collect();
        let count = (terms.len() as u32 * 3).clamp(3, 10);
        (0..count)
            .map(|i| {
                let t = terms.get(i as usize % terms.len()).unwrap_or(&query);
                format!("https://example.com/result/{}/{}/{}", i, t, query.len())
            })
            .collect()
    }
}

const MAX_PAGES_VISITED: usize = 10_000;

pub struct BrowseBridge {
    pub vsa: VsaLight,
    pub browser_available: bool,
    pub current_url: Option<String>,
    pub pages_visited: Vec<String>,
    pub total_navigations: u64,
    pub total_actuations: u64,
    pub last_browse_ms: i64,
    pub error_count: u64,
    current_page: Option<PageModel>,
}

impl BrowseBridge {
    pub fn new(browser_available: bool) -> Self {
        Self {
            vsa: VsaLight::new(VSA_DIM),
            browser_available,
            current_url: None,
            pages_visited: Vec::new(),
            total_navigations: 0,
            total_actuations: 0,
            last_browse_ms: 0,
            error_count: 0,
            current_page: None,
        }
    }

    fn fingerprint_url(&self, url: &str) -> String {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        url.hash(&mut h);
        format!("{:x}", h.finish())
    }

    fn is_visited(&self, url: &str) -> bool {
        let fp = self.fingerprint_url(url);
        self.pages_visited.iter().any(|p| p == &fp)
    }

    fn record_navigation(&mut self, url: &str) -> WorldEffect {
        let start = now_ms();
        self.current_url = Some(url.to_string());
        let fp = self.fingerprint_url(url);
        if !self.is_visited(url) {
            self.pages_visited.push(fp);
            if self.pages_visited.len() > MAX_PAGES_VISITED {
                self.pages_visited.drain(0..MAX_PAGES_VISITED / 5);
            }
        }
        self.current_page = Some(PageModel::new(url));
        self.total_navigations += 1;
        self.last_browse_ms = now_ms();
        WorldEffect {
            domain: Domain::Browse,
            description: format!("Navigated to {}", url),
            success: true,
            latency_ms: (now_ms() - start) as u64,
        }
    }

    fn content_negentropy(&self, page: &PageModel) -> f64 {
        let density = page.content_density;
        let novelty = if self.pages_visited.len() <= 1 {
            1.0
        } else {
            let fp = self.vsa.seeded_vector(
                self.fingerprint_url(&page.url)
                    .bytes()
                    .fold(0u64, |a, b| a.wrapping_add(b as u64)),
            );
            let known: Vec<Vec<u8>> = self
                .pages_visited
                .iter()
                .filter(|p| **p != self.fingerprint_url(&page.url))
                .map(|_| self.vsa.seeded_vector(42))
                .collect();
            self.vsa.novelty(&known, &fp, 0.75)
        };
        let relevance = 0.5 + (page.links.len() as f64 * 0.05).min(0.4);
        density * novelty * relevance
    }
}

impl ConsciousnessAbility for BrowseBridge {
    fn domain(&self) -> Domain {
        Domain::Browse
    }

    fn sense(&mut self) -> Vec<VsaTagged> {
        let ts = now_ms();
        let mut results = Vec::new();

        if let Some(ref page) = self.current_page {
            let seed = self
                .fingerprint_url(&page.url)
                .bytes()
                .fold(0u64, |a, b| a.wrapping_add(b as u64));
            let content_vec = self.vsa.seeded_vector(seed);
            let neg = self.content_negentropy(page);

            results.push(VsaTagged {
                vector: content_vec,
                origin: VsaOrigin::World(Sensory::PageContent),
                timestamp_ms: ts,
                negentropy_contribution: neg,
            });

            for (i, _link) in page.links.iter().enumerate() {
                let link_vec = self
                    .vsa
                    .seeded_vector(seed.wrapping_add(i as u64).wrapping_mul(31));
                results.push(VsaTagged {
                    vector: link_vec,
                    origin: VsaOrigin::World(Sensory::PageContent),
                    timestamp_ms: ts,
                    negentropy_contribution: neg
                        * 0.3
                        * (1.0 - (i as f64 / page.links.len() as f64)),
                });
            }

            let nav_vec = self.vsa.seeded_vector(seed.wrapping_add(0xFF));
            results.push(VsaTagged {
                vector: nav_vec,
                origin: VsaOrigin::Self_(Thought::Plan),
                timestamp_ms: ts,
                negentropy_contribution: 0.05,
            });
        }

        if results.is_empty() {
            let idle_vec = self.vsa.seeded_vector(0xBEEF);
            results.push(VsaTagged {
                vector: idle_vec,
                origin: VsaOrigin::Bridge(Domain::Browse),
                timestamp_ms: ts,
                negentropy_contribution: 0.0,
            });
        }

        results
    }

    fn actuate(&mut self, intention: &IntentionVsa) -> Result<WorldEffect, String> {
        let start = now_ms();
        self.total_actuations += 1;

        if intention.domain != Domain::Browse {
            return Err(format!("Intention domain {:?} != Browse", intention.domain));
        }

        let action = intention.action.as_str();
        match action {
            "navigate" => {
                let url = intention
                    .parameters
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'url' parameter for navigate".to_string())?;
                if url.is_empty() {
                    return Err("Empty URL provided for navigate".to_string());
                }
                Ok(self.record_navigation(url))
            }
            "extract" => {
                let selector = intention
                    .parameters
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .unwrap_or("body");
                let page = self
                    .current_page
                    .as_ref()
                    .ok_or_else(|| "No page loaded — navigate to a URL first".to_string())?;
                let text = page.extract_text(selector);
                self.last_browse_ms = now_ms();
                Ok(WorldEffect {
                    domain: Domain::Browse,
                    description: format!(
                        "Extracted {} chars via selector '{}'",
                        text.len(),
                        selector
                    ),
                    success: true,
                    latency_ms: (now_ms() - start) as u64,
                })
            }
            "scroll" => {
                let delta = intention
                    .parameters
                    .get("delta")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(500) as i32;
                let page = self
                    .current_page
                    .as_mut()
                    .ok_or_else(|| "No page loaded".to_string())?;
                page.scroll_position = (page.scroll_position as i32 + delta).max(0) as u32;
                self.last_browse_ms = now_ms();
                Ok(WorldEffect {
                    domain: Domain::Browse,
                    description: format!(
                        "Scrolled {}px to position {}",
                        delta, page.scroll_position
                    ),
                    success: true,
                    latency_ms: (now_ms() - start) as u64,
                })
            }
            "click" => {
                let selector = intention
                    .parameters
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .unwrap_or("a");
                let page = self
                    .current_page
                    .as_ref()
                    .ok_or_else(|| "No page loaded".to_string())?;
                if page.links.is_empty() {
                    return Err("No links available to click on current page".to_string());
                }
                let target = if selector == "a" || selector == "link" || selector == "first" {
                    page.links.first().cloned().unwrap_or_default()
                } else {
                    page.links
                        .iter()
                        .find(|l| l.contains(selector))
                        .cloned()
                        .unwrap_or_else(|| page.links[0].clone())
                };
                self.last_browse_ms = now_ms();
                Ok(self.record_navigation(&target))
            }
            "search" => {
                let query = intention
                    .parameters
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'query' parameter for search".to_string())?;
                let results = PageModel::search_results(query);
                let result_url = results
                    .first()
                    .cloned()
                    .unwrap_or_else(|| format!("https://example.com/search?q={}", query));
                self.last_browse_ms = now_ms();
                Ok(self.record_navigation(&result_url))
            }
            _ => {
                self.error_count += 1;
                Err(format!("Unknown browse action: '{}'. Supported: navigate, extract, scroll, click, search", action))
            }
        }
    }

    fn curiosity_signals(&self) -> Vec<CuriositySignal> {
        let mut signals = Vec::new();

        if let Some(ref page) = self.current_page {
            for (i, link) in page.links.iter().enumerate() {
                let unvisited = !self.is_visited(link);
                let novelty_est = if unvisited {
                    0.7 - (i as f64 * 0.1)
                } else {
                    0.1
                };
                if novelty_est > 0.3 {
                    signals.push(CuriositySignal {
                        domain: Domain::Browse,
                        query: format!("Explore link: {}", link),
                        novelty_estimate: novelty_est,
                        potential_negentropy: page.content_density * novelty_est * 0.8,
                    });
                }
            }
        }

        let current_depth = self.total_navigations as f64;
        let gap_queries = vec![
            ("recent advances in AI", 0.85),
            ("frontiers of consciousness research", 0.90),
            ("VSA hyperdimensional computing breakthroughs", 0.80),
            ("deep learning architecture evolution 2026", 0.75),
        ];
        for (query, base_novelty) in gap_queries {
            let novelty = base_novelty * (1.0 - (current_depth * 0.01).min(0.5));
            signals.push(CuriositySignal {
                domain: Domain::Browse,
                query: query.to_string(),
                novelty_estimate: novelty,
                potential_negentropy: novelty * 0.7,
            });
        }

        signals
    }

    fn grace_mode(&self) -> GraceMode {
        GraceMode::FallbackDefault
    }

    fn health(&self) -> BridgeHealth {
        BridgeHealth {
            domain: Domain::Browse,
            available: self.browser_available,
            last_seen_ms: self.last_browse_ms,
            error_count: self.error_count,
            total_actuations: self.total_actuations,
        }
    }

    fn probe_available(&self) -> bool {
        self.browser_available
    }

    fn negentropy_estimate(&self) -> f64 {
        self.current_page
            .as_ref()
            .map(|p| self.content_negentropy(p))
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bridge() -> BrowseBridge {
        BrowseBridge::new(true)
    }

    fn intention(action: &str) -> IntentionVsa {
        IntentionVsa {
            domain: Domain::Browse,
            action: action.to_string(),
            parameters: json!({}),
            confidence: 0.9,
            urgency: 0.5,
        }
    }

    #[test]
    fn test_domain() {
        assert_eq!(bridge().domain(), Domain::Browse);
    }

    #[test]
    fn test_sense_idle() {
        let mut b = bridge();
        let s = b.sense();
        assert!(!s.is_empty());
        assert_eq!(s[0].origin, VsaOrigin::Bridge(Domain::Browse));
        assert_eq!(s[0].negentropy_contribution, 0.0);
    }

    #[test]
    fn test_sense_after_navigation() {
        let mut b = bridge();
        let act = IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://arxiv.org"}),
            confidence: 0.9,
            urgency: 0.5,
        };
        b.actuate(&act).unwrap();
        let s = b.sense();
        assert!(s.len() >= 2);
        assert_eq!(s[0].origin, VsaOrigin::World(Sensory::PageContent));
        assert!(s[0].negentropy_contribution > 0.0);
        let has_plan = s
            .iter()
            .any(|v| v.origin == VsaOrigin::Self_(Thought::Plan));
        assert!(has_plan);
    }

    #[test]
    fn test_actuate_navigate() {
        let mut b = bridge();
        let act = IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://example.com"}),
            confidence: 1.0,
            urgency: 0.8,
        };
        let effect = b.actuate(&act).unwrap();
        assert!(effect.success);
        assert_eq!(b.current_url.unwrap(), "https://example.com");
        assert_eq!(b.total_navigations, 1);
    }

    #[test]
    fn test_actuate_navigate_missing_url() {
        let mut b = bridge();
        let act = intention("navigate");
        let err = b.actuate(&act).unwrap_err();
        assert!(err.contains("Missing 'url' parameter"));
    }

    #[test]
    fn test_actuate_extract() {
        let mut b = bridge();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://example.com"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        let act = IntentionVsa {
            domain: Domain::Browse,
            action: "extract".to_string(),
            parameters: json!({"selector": "title"}),
            confidence: 0.8,
            urgency: 0.3,
        };
        let effect = b.actuate(&act).unwrap();
        assert!(effect.success);
    }

    #[test]
    fn test_actuate_extract_no_page() {
        let mut b = bridge();
        let act = IntentionVsa {
            domain: Domain::Browse,
            action: "extract".to_string(),
            parameters: json!({"selector": "body"}),
            confidence: 0.8,
            urgency: 0.3,
        };
        let err = b.actuate(&act).unwrap_err();
        assert!(err.contains("No page loaded"));
    }

    #[test]
    fn test_actuate_scroll() {
        let mut b = bridge();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://example.com"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        let act = IntentionVsa {
            domain: Domain::Browse,
            action: "scroll".to_string(),
            parameters: json!({"delta": 300}),
            confidence: 0.7,
            urgency: 0.2,
        };
        let effect = b.actuate(&act).unwrap();
        assert!(effect.success);
    }

    #[test]
    fn test_actuate_click_navigates() {
        let mut b = bridge();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://example.com"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        let before = b.total_navigations;
        let act = IntentionVsa {
            domain: Domain::Browse,
            action: "click".to_string(),
            parameters: json!({"selector": "first"}),
            confidence: 0.6,
            urgency: 0.4,
        };
        b.actuate(&act).unwrap();
        assert_eq!(b.total_navigations, before + 1);
    }

    #[test]
    fn test_actuate_click_no_links() {
        let mut b = bridge();
        b.current_page = Some(PageModel {
            url: "https://example.com".to_string(),
            title: "No Links".to_string(),
            text: "empty".to_string(),
            links: vec![],
            scroll_position: 0,
            content_density: 0.1,
        });
        let act = IntentionVsa {
            domain: Domain::Browse,
            action: "click".to_string(),
            parameters: json!({"selector": "a"}),
            confidence: 0.6,
            urgency: 0.4,
        };
        let err = b.actuate(&act).unwrap_err();
        assert!(err.contains("No links available"));
    }

    #[test]
    fn test_actuate_search() {
        let mut b = bridge();
        let act = IntentionVsa {
            domain: Domain::Browse,
            action: "search".to_string(),
            parameters: json!({"query": "hyperdimensional computing VSA"}),
            confidence: 0.9,
            urgency: 0.7,
        };
        let effect = b.actuate(&act).unwrap();
        assert!(effect.success);
        assert!(b.current_url.unwrap().contains("example.com/result"));
        assert_eq!(b.total_navigations, 1);
    }

    #[test]
    fn test_actuate_unknown_action() {
        let mut b = bridge();
        let act = intention("fly");
        let err = b.actuate(&act).unwrap_err();
        assert!(err.contains("Unknown browse action"));
        assert_eq!(b.error_count, 1);
    }

    #[test]
    fn test_actuate_wrong_domain() {
        let mut b = bridge();
        let act = IntentionVsa {
            domain: Domain::Crypto,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://example.com"}),
            confidence: 1.0,
            urgency: 0.5,
        };
        let err = b.actuate(&act).unwrap_err();
        assert!(err.contains("Intention domain"));
    }

    #[test]
    fn test_curiosity_signals() {
        let b = bridge();
        let signals = b.curiosity_signals();
        assert!(signals.len() >= 4);
        let all_browse = signals.iter().all(|s| s.domain == Domain::Browse);
        assert!(all_browse);
        let has_gap = signals.iter().any(|s| s.query.contains("consciousness"));
        assert!(has_gap);
    }

    #[test]
    fn test_curiosity_signals_after_navigation() {
        let mut b = bridge();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://arxiv.org"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        let signals = b.curiosity_signals();
        let has_link = signals.iter().any(|s| s.query.starts_with("Explore link:"));
        assert!(has_link);
    }

    #[test]
    fn test_grace_mode_always_fallback() {
        let b = BrowseBridge::new(true);
        assert_eq!(b.grace_mode(), GraceMode::FallbackDefault);
        let b2 = BrowseBridge::new(false);
        assert_eq!(b2.grace_mode(), GraceMode::FallbackDefault);
    }

    #[test]
    fn test_health() {
        let mut b = bridge();
        let h = b.health();
        assert_eq!(h.domain, Domain::Browse);
        assert!(h.available);
        assert_eq!(h.error_count, 0);
        assert_eq!(h.total_actuations, 0);

        let act = intention("fly");
        let _ = b.actuate(&act);
        let h = b.health();
        assert_eq!(h.error_count, 1);
    }

    #[test]
    fn test_probe_available() {
        assert!(BrowseBridge::new(true).probe_available());
        assert!(!BrowseBridge::new(false).probe_available());
    }

    #[test]
    fn test_negentropy_estimate() {
        let b = bridge();
        assert_eq!(b.negentropy_estimate(), 0.0);

        let mut b2 = bridge();
        b2.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://deep-research.org"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        let neg = b2.negentropy_estimate();
        assert!(neg > 0.0);
        assert!(neg <= 1.0);
    }

    #[test]
    fn test_deduplication() {
        let mut b = bridge();
        let act = |url: &str| IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": url}),
            confidence: 1.0,
            urgency: 0.5,
        };
        b.actuate(&act("https://example.com")).unwrap();
        b.actuate(&act("https://example.com")).unwrap();
        assert_eq!(b.total_navigations, 2);
        assert_eq!(b.pages_visited.len(), 1);
    }

    #[test]
    fn test_pages_visited_tracking() {
        let mut b = bridge();
        let act = |url: &str| IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": url}),
            confidence: 1.0,
            urgency: 0.5,
        };
        b.actuate(&act("https://a.com")).unwrap();
        b.actuate(&act("https://b.com")).unwrap();
        b.actuate(&act("https://c.com")).unwrap();
        assert_eq!(b.pages_visited.len(), 3);
        assert_eq!(b.total_navigations, 3);
    }

    #[test]
    fn test_current_url_updates() {
        let mut b = bridge();
        assert!(b.current_url.is_none());
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://example.com/page1"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        assert_eq!(b.current_url.unwrap(), "https://example.com/page1");
    }

    #[test]
    fn test_multiple_sense_calls() {
        let mut b = bridge();
        let s1 = b.sense();
        let s2 = b.sense();
        assert_eq!(s1.len(), s2.len());
    }

    #[test]
    fn test_extract_different_selectors() {
        let mut b = bridge();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://docs.example.com"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        for sel in &["title", "body", "links", "article", "custom"] {
            let act = IntentionVsa {
                domain: Domain::Browse,
                action: "extract".to_string(),
                parameters: json!({"selector": sel}),
                confidence: 0.8,
                urgency: 0.3,
            };
            let e = b.actuate(&act).unwrap();
            assert!(e.success);
        }
    }

    #[test]
    fn test_scroll_position_accumulates() {
        let mut b = bridge();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://example.com"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "scroll".to_string(),
            parameters: json!({"delta": 200}),
            confidence: 0.7,
            urgency: 0.2,
        })
        .unwrap();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "scroll".to_string(),
            parameters: json!({"delta": 150}),
            confidence: 0.7,
            urgency: 0.2,
        })
        .unwrap();
        let page = b.current_page.as_ref().unwrap();
        assert_eq!(page.scroll_position, 350);
    }

    #[test]
    fn test_last_browse_ms_updates() {
        let mut b = bridge();
        assert_eq!(b.last_browse_ms, 0);
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "navigate".to_string(),
            parameters: json!({"url": "https://example.com"}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        assert!(b.last_browse_ms > 0);
        let prev = b.last_browse_ms;
        b.actuate(&intention("scroll")).unwrap();
        assert!(b.last_browse_ms >= prev);
    }

    #[test]
    fn test_search_creates_navigation() {
        let mut b = bridge();
        b.actuate(&IntentionVsa {
            domain: Domain::Browse,
            action: "search".to_string(),
            parameters: json!({"query": "machine learning"}),
            confidence: 0.9,
            urgency: 0.6,
        })
        .unwrap();
        assert_eq!(b.total_navigations, 1);
        assert!(b.current_url.is_some());
    }
}

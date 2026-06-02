use std::collections::{HashMap, HashSet, VecDeque};

use super::config::CrawlTopic;

#[derive(Debug, Clone)]
pub struct UrlEntry {
    pub url: String,
    pub domain: String,
    pub depth: u32,
    pub priority: u32,
    pub topic: Option<CrawlTopic>,
}

pub struct DualQueueFrontier {
    front_queues: Vec<VecDeque<UrlEntry>>,
    back_queues: HashMap<String, VecDeque<UrlEntry>>,
    seen: HashSet<String>,
    domain_pushed_count: HashMap<String, usize>,
    domain_visited_count: HashMap<String, usize>,
    domain_last_access: HashMap<String, u64>,
    max_per_domain: usize,
    max_priority_tiers: usize,
    total_pushed: u64,
    total_popped: u64,
    total_skipped: u64,
}

impl DualQueueFrontier {
    pub fn new(max_per_domain: usize) -> Self {
        DualQueueFrontier {
            front_queues: vec![VecDeque::new(); 5],
            back_queues: HashMap::new(),
            seen: HashSet::new(),
            domain_pushed_count: HashMap::new(),
            domain_visited_count: HashMap::new(),
            domain_last_access: HashMap::new(),
            max_per_domain,
            max_priority_tiers: 5,
            total_pushed: 0,
            total_popped: 0,
            total_skipped: 0,
        }
    }

    pub fn push(&mut self, entry: UrlEntry) -> bool {
        if self.seen.contains(&entry.url) {
            self.total_skipped += 1;
            return false;
        }

        let pushed_count = self.domain_pushed_count.get(&entry.domain).copied().unwrap_or(0);
        if pushed_count >= self.max_per_domain {
            self.total_skipped += 1;
            return false;
        }

        self.seen.insert(entry.url.clone());
        self.total_pushed += 1;
        self.domain_pushed_count.entry(entry.domain.clone()).and_modify(|c| *c += 1).or_insert(1);

        let priority_tier = (entry.priority as usize).min(self.max_priority_tiers - 1);
        self.front_queues[priority_tier].push_back(entry.clone());

        self.back_queues
            .entry(entry.domain.clone())
            .or_default()
            .push_back(entry);

        true
    }

    pub fn push_seeds(&mut self, seeds: Vec<UrlEntry>) {
        for seed in seeds {
            self.push(seed);
        }
    }

    pub fn pop(&mut self, now_secs: u64, min_interval_ms: u64) -> Option<UrlEntry> {
        for tier in (0..self.max_priority_tiers).rev() {
            while let Some(entry) = self.front_queues[tier].pop_front() {
                if self.seen.contains(&entry.url) {
                    let last_access = self.domain_last_access.get(&entry.domain).copied().unwrap_or(0);
                    if now_secs * 1000 - last_access < min_interval_ms {
                        self.front_queues[tier].push_back(entry);
                        break;
                    }
                } else {
                    self.total_skipped += 1;
                    continue;
                }

                self.domain_visited_count
                    .entry(entry.domain.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
                self.domain_last_access.insert(entry.domain.clone(), now_secs * 1000);
                self.total_popped += 1;
                return Some(entry);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.front_queues.iter().map(|q| q.len()).sum::<usize>()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn total_seen(&self) -> usize {
        self.seen.len()
    }

    pub fn stats(&self) -> FrontierStats {
        FrontierStats {
            total_pushed: self.total_pushed,
            total_popped: self.total_popped,
            total_skipped: self.total_skipped,
            queue_size: self.len(),
            seen_urls: self.seen.len(),
            active_domains: self.back_queues.len(),
        }
    }

    pub fn domain_visited_count(&self, domain: &str) -> usize {
        self.domain_visited_count.get(domain).copied().unwrap_or(0)
    }
}

pub struct FrontierStats {
    pub total_pushed: u64,
    pub total_popped: u64,
    pub total_skipped: u64,
    pub queue_size: usize,
    pub seen_urls: usize,
    pub active_domains: usize,
}

impl std::fmt::Display for FrontierStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Frontier: pushed={} popped={} skipped={} queue={} seen={} domains={}",
            self.total_pushed, self.total_popped, self.total_skipped,
            self.queue_size, self.seen_urls, self.active_domains,
        )
    }
}

pub fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let mut links = Vec::new();
    let base_domain = extract_domain(base_url);

    for fragment in html.split("<a ") {
        if let Some(href_start) = fragment.find("href=\"") {
            let start = href_start + 6;
            if let Some(end) = fragment[start..].find('"') {
                let href = &fragment[start..start + end];
                if href.starts_with("http")
                    && extract_domain(href) != base_domain {
                        links.push(href.to_string());
                    }
            }
        }
    }

    links.truncate(50);
    links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut frontier = DualQueueFrontier::new(100);
        assert!(frontier.is_empty());

        let entry = UrlEntry {
            url: "https://example.com/page1".into(),
            domain: "example.com".into(),
            depth: 1,
            priority: 3,
            topic: None,
        };
        assert!(frontier.push(entry));
        assert!(!frontier.is_empty());
        assert_eq!(frontier.len(), 1);

        let popped = frontier.pop(1000, 0);
        assert!(popped.is_some());
        assert_eq!(popped.expect("popped should be ok in test").url, "https://example.com/page1");
        assert!(frontier.is_empty());
    }

    #[test]
    fn test_dedup() {
        let mut frontier = DualQueueFrontier::new(100);
        let entry = UrlEntry {
            url: "https://example.com/dup".into(),
            domain: "example.com".into(),
            depth: 1,
            priority: 1,
            topic: None,
        };
        assert!(frontier.push(entry.clone()));
        assert!(!frontier.push(entry));
        assert_eq!(frontier.total_skipped, 1);
    }

    #[test]
    fn test_max_per_domain() {
        let mut frontier = DualQueueFrontier::new(2);
        for i in 0..5 {
            let entry = UrlEntry {
                url: format!("https://example.com/page{}", i),
                domain: "example.com".into(),
                depth: 1,
                priority: 1,
                topic: None,
            };
            frontier.push(entry);
        }
        assert_eq!(frontier.total_pushed, 2);
        assert_eq!(frontier.total_skipped, 3);
    }

    #[test]
    fn test_priority_tiers() {
        let mut frontier = DualQueueFrontier::new(100);
        let low = UrlEntry {
            url: "https://low.com".into(),
            domain: "low.com".into(),
            depth: 2,
            priority: 0,
            topic: None,
        };
        let high = UrlEntry {
            url: "https://high.com".into(),
            domain: "high.com".into(),
            depth: 1,
            priority: 4,
            topic: None,
        };
        frontier.push(low);
        frontier.push(high);

        let first = frontier.pop(1000, 0);
        assert_eq!(first.expect("first should be ok in test").url, "https://high.com");
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/path"), "example.com");
        assert_eq!(extract_domain("http://sub.example.com"), "sub.example.com");
        assert_eq!(extract_domain("no-scheme"), "no-scheme");
    }

    #[test]
    fn test_extract_links() {
        let html = r#"<html><a href="https://other.com/page">link</a><a href="https://same.com">same</a></html>"#;
        let links = extract_links(html, "https://same.com/page");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], "https://other.com/page");
    }

    #[test]
    fn test_stats_display() {
        let mut frontier = DualQueueFrontier::new(100);
        let entry = UrlEntry {
            url: "https://test.com".into(),
            domain: "test.com".into(),
            depth: 0,
            priority: 1,
            topic: None,
        };
        frontier.push(entry);
        let stats_str = format!("{}", frontier.stats());
        assert!(stats_str.contains("pushed=1"));
    }
}

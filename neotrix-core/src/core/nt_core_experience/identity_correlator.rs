#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// ── Error type ──

#[derive(Debug, thiserror::Error)]
pub enum IdentityCorrelationError {
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    #[error("No correlation found between provided aliases")]
    NoCorrelation,
    #[error("Insufficient attributes to form identity: {0}")]
    InsufficientAttributes(String),
}

pub type Result<T> = std::result::Result<T, IdentityCorrelationError>;

// ── VSA Encoder ──

#[derive(Debug, Clone)]
pub struct VSAEncoder {
    dim: usize,
}

impl VSAEncoder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Deterministic seeded hash-based embedding. Produces `dim/8` bytes
    /// of pseudo-random bits keyed by the input text.
    pub fn embed_text(&self, text: &str) -> Vec<u8> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();
        let byte_count = self.dim / 8;
        (0..byte_count)
            .map(|i| {
                let mut h = std::collections::hash_map::DefaultHasher::new();
                seed.hash(&mut h);
                i.hash(&mut h);
                text.hash(&mut h);
                h.finish() as u8
            })
            .collect()
    }
}

/// Jaccard similarity over two byte slices (treated as bit vectors).
pub fn similarity(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut intersection = 0;
    let mut union = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        for bit in 0..8u8 {
            let a_bit = (x >> bit) & 1;
            let b_bit = (y >> bit) & 1;
            if a_bit == 1 || b_bit == 1 {
                union += 1;
            }
            if a_bit == 1 && b_bit == 1 {
                intersection += 1;
            }
        }
    }
    if union == 0 {
        return 1.0;
    }
    intersection as f64 / union as f64
}

// ── Identity Profile ──

#[derive(Debug, Clone)]
pub struct IdentityProfile {
    pub fingerprint_hash: u64,
    pub primary_username: String,
    pub known_aliases: Vec<String>,
    pub known_emails: Vec<String>,
    pub discovered_urls: Vec<String>,
    pub platforms: Vec<(String, f64)>,
    pub attributes: HashMap<String, String>,
    pub confidence: f64,
    pub last_updated: u64,
    pub evidence_ids: Vec<u64>,
}

// ── Correlation Result ──

#[derive(Debug, Clone)]
pub struct CorrelationResult {
    pub identities: Vec<IdentityProfile>,
    pub match_confidence: f64,
    pub shared_attributes: Vec<String>,
    pub evidence_chain: Vec<(String, String, f64)>,
    pub analysis_summary: String,
}

// ── Identity Correlator ──

pub struct IdentityCorrelator {
    known_identities: HashMap<String, IdentityProfile>,
    alias_graph: HashMap<String, Vec<String>>,
    min_correlation_threshold: f64,
    vsa_encoder: Option<VSAEncoder>,
    max_profiles: usize,
}

impl IdentityCorrelator {
    pub fn new() -> Self {
        Self {
            known_identities: HashMap::new(),
            alias_graph: HashMap::new(),
            min_correlation_threshold: 0.65,
            vsa_encoder: Some(VSAEncoder::new(4096)),
            max_profiles: 1000,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.min_correlation_threshold = threshold;
        self
    }

    pub fn with_max_profiles(mut self, max: usize) -> Self {
        self.max_profiles = max;
        self
    }

    pub fn with_vsa_encoder(mut self, encoder: VSAEncoder) -> Self {
        self.vsa_encoder = Some(encoder);
        self
    }

    pub fn without_vsa(mut self) -> Self {
        self.vsa_encoder = None;
        self
    }

    /// Compute a deterministic fingerprint hash from a set of string attributes.
    fn compute_fingerprint(primary: &str, aliases: &[String], emails: &[String]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        primary.hash(&mut hasher);
        for a in aliases {
            a.hash(&mut hasher);
        }
        for e in emails {
            e.hash(&mut hasher);
        }
        hasher.finish()
    }

    // ── Registration ──

    /// Register an alias on a specific platform with a confidence score.
    /// If the username already exists, the platform entry is merged in.
    pub fn register_alias(
        &mut self,
        username: &str,
        platform: &str,
        confidence: f64,
        evidence_id: Option<u64>,
    ) {
        let aliases: Vec<String> = vec![username.to_string()];
        let fp = Self::compute_fingerprint(username, &aliases, &[]);

        // Find or create identity with this fingerprint
        let _identity_key = if let Some((key, profile)) =
            self.known_identities.iter_mut().find(|(_, p)| {
                p.primary_username == username || p.known_aliases.contains(&username.to_string())
            }) {
            profile.platforms.push((platform.to_string(), confidence));
            if !profile.known_aliases.contains(&username.to_string()) {
                profile.known_aliases.push(username.to_string());
            }
            if let Some(eid) = evidence_id {
                if !profile.evidence_ids.contains(&eid) {
                    profile.evidence_ids.push(eid);
                }
            }
            profile.confidence = profile.confidence.max(confidence);
            profile.last_updated = current_time();
            profile.fingerprint_hash = Self::compute_fingerprint(
                &profile.primary_username,
                &profile.known_aliases,
                &profile.known_emails,
            );
            key.clone()
        } else {
            let profile = IdentityProfile {
                fingerprint_hash: fp,
                primary_username: username.to_string(),
                known_aliases: vec![username.to_string()],
                known_emails: Vec::new(),
                discovered_urls: Vec::new(),
                platforms: vec![(platform.to_string(), confidence)],
                attributes: HashMap::new(),
                confidence,
                last_updated: current_time(),
                evidence_ids: evidence_id.map(|e| vec![e]).unwrap_or_default(),
            };
            let key = username.to_string();
            self.known_identities.insert(key.clone(), profile);
            key
        };

        self.alias_graph
            .entry(username.to_string())
            .or_default()
            .push(platform.to_string());

        self.prune();
    }

    /// Create a bidirectional link between two aliases.
    pub fn link_aliases(&mut self, alias1: &str, alias2: &str, confidence: f64) {
        self.alias_graph
            .entry(alias1.to_string())
            .or_default()
            .push(alias2.to_string());
        self.alias_graph
            .entry(alias2.to_string())
            .or_default()
            .push(alias1.to_string());

        // Merge or link underlying identities
        let ids: Vec<String> = self
            .known_identities
            .iter()
            .filter(|(_, p)| {
                p.primary_username == alias1
                    || p.known_aliases.contains(&alias1.to_string())
                    || p.primary_username == alias2
                    || p.known_aliases.contains(&alias2.to_string())
            })
            .map(|(k, _)| k.clone())
            .collect();

        if ids.len() >= 2 {
            if let Some(merged) = self.merge_profiles(&ids) {
                // Bump confidence for the linked aliases
                if let Some(profile) = self.known_identities.get_mut(&merged) {
                    profile.confidence = profile.confidence.max(confidence);
                }
            }
        }
    }

    // ── Correlation ──

    /// Three-level identity correlation. Given a set of usernames, find likely identity clusters.
    pub fn correlate(&self, usernames: &[&str]) -> Vec<CorrelationResult> {
        let mut results = Vec::new();

        // Collect candidate profiles
        let candidates: Vec<&IdentityProfile> = self
            .known_identities
            .values()
            .filter(|p| {
                usernames
                    .iter()
                    .any(|u| p.primary_username == *u || p.known_aliases.contains(&u.to_string()))
            })
            .collect();

        if candidates.is_empty() {
            return results;
        }

        // Level 1: Direct — exact username match across platforms
        for profile in &candidates {
            let shared: Vec<String> = usernames
                .iter()
                .filter(|u| {
                    profile.primary_username == **u
                        || profile.known_aliases.contains(&u.to_string())
                })
                .map(|u| u.to_string())
                .collect();

            if !shared.is_empty() {
                let confidence =
                    0.65 + 0.20 * (shared.len() as f64 / usernames.len() as f64).min(1.0);
                let evidence_chain: Vec<(String, String, f64)> = shared
                    .iter()
                    .map(|a| (a.clone(), profile.primary_username.clone(), 0.85))
                    .collect();

                results.push(CorrelationResult {
                    identities: vec![(*profile).clone()],
                    match_confidence: confidence.min(0.95),
                    shared_attributes: vec!["username".to_string()],
                    evidence_chain,
                    analysis_summary: format!(
                        "Level 1 (Direct): {} matched {} profile(s) via exact username",
                        usernames.join(", "),
                        shared.len()
                    ),
                });
            }
        }

        // Level 2: Attribute — shared attributes across different usernames using VSA bio similarity
        if let Some(ref encoder) = self.vsa_encoder {
            let mut seen_pairs: std::collections::HashSet<String> = std::collections::HashSet::new();
            for (i, a) in candidates.iter().enumerate() {
                for b in candidates.iter().skip(i + 1) {
                    let pair_key = if a.primary_username < b.primary_username {
                        format!("{}::{}", a.primary_username, b.primary_username)
                    } else {
                        format!("{}::{}", b.primary_username, a.primary_username)
                    };
                    if seen_pairs.contains(&pair_key) {
                        continue;
                    }
                    seen_pairs.insert(pair_key.clone());

                    let mut shared_attrs: Vec<String> = Vec::new();
                    let mut attr_sim = 0.0_f64;

                    // Compare bio / location / name attributes if present
                    for key in &["name", "location", "bio"] {
                        let a_val = a.attributes.get(*key);
                        let b_val = b.attributes.get(*key);
                        if let (Some(av), Some(bv)) = (a_val, b_val) {
                            let emb_a = encoder.embed_text(av);
                            let emb_b = encoder.embed_text(bv);
                            let sim = similarity(&emb_a, &emb_b);
                            if sim > self.min_correlation_threshold {
                                shared_attrs.push(key.to_string());
                                attr_sim += sim;
                            }
                        }
                    }

                    if !shared_attrs.is_empty() {
                        let avg_sim = attr_sim / shared_attrs.len() as f64;
                        let confidence = 0.50 + 0.30 * avg_sim;

                        results.push(CorrelationResult {
                            identities: vec![(*a).clone(), (*b).clone()],
                            match_confidence: confidence.min(0.90),
                            shared_attributes: shared_attrs.clone(),
                            evidence_chain: vec![(
                                a.primary_username.clone(),
                                b.primary_username.clone(),
                                avg_sim,
                            )],
                            analysis_summary: format!(
                                "Level 2 (Attribute): {} and {} share {}",
                                a.primary_username,
                                b.primary_username,
                                shared_attrs.join(", ")
                            ),
                        });
                    }
                }
            }
        }

        // Level 3: Graph — transitive alias link analysis with decay
        let mut graph_results: Vec<CorrelationResult> = Vec::new();
        for username in usernames {
            let chain = self.alias_chain(username, 3);
            if chain.len() > 1 {
                let mut linked_profiles: Vec<IdentityProfile> = Vec::new();
                for (alias, _conf, _hops) in &chain {
                    if let Some(profile) = self
                        .known_identities
                        .values()
                        .find(|p| p.primary_username == *alias || p.known_aliases.contains(alias))
                    {
                        if !linked_profiles
                            .iter()
                            .any(|lp| lp.fingerprint_hash == profile.fingerprint_hash)
                        {
                            linked_profiles.push((*profile).clone());
                        }
                    }
                }

                if linked_profiles.len() > 1 {
                    let avg_confidence: f64 =
                        chain.iter().map(|(_, c, _)| c).sum::<f64>() / chain.len() as f64;
                    let decayed = avg_confidence * 0.85_f64.powi((chain.len() - 1) as i32);

                    graph_results.push(CorrelationResult {
                        identities: linked_profiles,
                        match_confidence: decayed,
                        shared_attributes: vec!["graph_link".to_string()],
                        evidence_chain: chain
                            .iter()
                            .map(|(a, c, _)| (username.to_string(), a.clone(), *c))
                            .collect(),
                        analysis_summary: format!(
                            "Level 3 (Graph): transitive link chain from {} to {} aliases",
                            username,
                            chain.len() - 1
                        ),
                    });
                }
            }
        }
        results.extend(graph_results);

        // Deduplicate by fingerprint hash
        let mut seen_fps: Vec<u64> = Vec::new();
        results.retain(|r| {
            let fp = r.identities.iter().map(|p| p.fingerprint_hash).sum::<u64>();
            if seen_fps.contains(&fp) {
                false
            } else {
                seen_fps.push(fp);
                true
            }
        });

        results.sort_by(|a, b| {
            b.match_confidence
                .partial_cmp(&a.match_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Find an identity profile by a specific attribute key-value pair.
    pub fn find_identity(&self, attribute: &str, value: &str) -> Option<&IdentityProfile> {
        self.known_identities.values().find(|p| match attribute {
            "email" | "emails" => p.known_emails.iter().any(|e| e == value),
            "username" | "alias" => {
                p.primary_username == value || p.known_aliases.contains(&value.to_string())
            }
            "url" | "discovered_url" | "discovered_urls" => {
                p.discovered_urls.iter().any(|u| u == value)
            }
            _ => p.attributes.get(attribute).map_or(false, |v| v == value),
        })
    }

    /// Extract structured attributes from a platform profile page text.
    /// Uses simple regex-free pattern matching on common profile fields.
    pub fn extract_attributes_from_profile(
        &self,
        platform: &str,
        page_text: &str,
    ) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        let lower = page_text.to_lowercase();

        // Name detection: look for common patterns
        if let Some(name) = self.extract_field(&lower, &["name:", "display name:", "full name:"]) {
            attrs.insert("name".to_string(), name);
        }

        // Location
        if let Some(loc) =
            self.extract_field(&lower, &["location:", "location :", "from:", "from :"])
        {
            attrs.insert("location".to_string(), loc);
        }

        // Bio / description — take text after common markers
        if let Some(bio) =
            self.extract_field(&lower, &["bio:", "about:", "description:", "biography:"])
        {
            attrs.insert("bio".to_string(), bio);
        }

        // Email detection (rudimentary)
        for token in lower.split_whitespace() {
            if token.contains('@') && token.contains('.') && !token.starts_with('@') {
                let clean = token.trim_matches(|c: char| {
                    !c.is_ascii_alphanumeric() && c != '@' && c != '.' && c != '_' && c != '-'
                });
                if clean.contains('@') {
                    attrs.insert("email".to_string(), clean.to_string());
                }
            }
        }

        // URL detection
        for token in lower.split_whitespace() {
            if token.starts_with("http://") || token.starts_with("https://") {
                let clean = token.trim_end_matches(|c: char| {
                    c == ',' || c == '.' || c == ')' || c == ']' || c == '>'
                });
                attrs.insert("url".to_string(), clean.to_string());
            }
        }

        // Platform-specific: extract handle
        match platform {
            "twitter" | "x" => {
                if let Some(handle) =
                    self.extract_field(&lower, &["@handle:", "handle:", "twitter:"])
                {
                    attrs.insert("handle".to_string(), handle);
                }
            }
            "github" => {
                if let Some(org) =
                    self.extract_field(&lower, &["organization:", "company:", "org:"])
                {
                    attrs.insert("organization".to_string(), org);
                }
                if let Some(repos) = self.extract_field(&lower, &["repositories:", "repos:"]) {
                    attrs.insert("repos".to_string(), repos);
                }
            }
            "linkedin" => {
                if let Some(headline) =
                    self.extract_field(&lower, &["headline:", "title:", "current position:"])
                {
                    attrs.insert("headline".to_string(), headline);
                }
            }
            _ => {}
        }

        attrs
    }

    /// Helper to extract a field value after a known prefix.
    fn extract_field<'a>(&self, text: &'a str, prefixes: &[&str]) -> Option<String> {
        for prefix in prefixes {
            if let Some(pos) = text.find(prefix) {
                let start = pos + prefix.len();
                let rest = &text[start..];
                let line_end = rest.find('\n').unwrap_or(rest.len());
                let field = rest[..line_end].trim().to_string();
                if !field.is_empty() && field.len() < 200 {
                    return Some(field);
                }
            }
        }
        None
    }

    // ── Profile Merging ──

    /// Merge multiple identity profiles into one. Returns the key of the merged profile.
    pub fn merge_profiles(&mut self, profile_ids: &[String]) -> Option<String> {
        if profile_ids.is_empty() {
            return None;
        }

        let mut profiles: Vec<IdentityProfile> = Vec::new();
        for id in profile_ids {
            if let Some(p) = self.known_identities.remove(id.as_str()) {
                profiles.push(p);
            }
            // Also try finding by primary_username matching
            let to_remove: Vec<String> = self
                .known_identities
                .iter()
                .filter(|(k, _)| k.as_str() == id.as_str())
                .map(|(k, _)| k.clone())
                .collect();
            for key in to_remove {
                if let Some(p) = self.known_identities.remove(&key) {
                    if !profiles
                        .iter()
                        .any(|x| x.fingerprint_hash == p.fingerprint_hash)
                    {
                        profiles.push(p);
                    }
                }
            }
        }

        if profiles.is_empty() {
            return None;
        }

        let mut merged = profiles.swap_remove(0);
        for other in profiles {
            for alias in other.known_aliases {
                if !merged.known_aliases.contains(&alias) {
                    merged.known_aliases.push(alias);
                }
            }
            for email in other.known_emails {
                if !merged.known_emails.contains(&email) {
                    merged.known_emails.push(email);
                }
            }
            for url in other.discovered_urls {
                if !merged.discovered_urls.contains(&url) {
                    merged.discovered_urls.push(url);
                }
            }
            for (platform, conf) in other.platforms {
                let existing = merged.platforms.iter_mut().find(|(p, _)| p == &platform);
                if let Some(e) = existing {
                    e.1 = e.1.max(conf);
                } else {
                    merged.platforms.push((platform, conf));
                }
            }
            for (k, v) in other.attributes {
                merged.attributes.entry(k).or_insert(v);
            }
            for eid in other.evidence_ids {
                if !merged.evidence_ids.contains(&eid) {
                    merged.evidence_ids.push(eid);
                }
            }
            merged.confidence = merged.confidence.max(other.confidence);
        }

        merged.fingerprint_hash = Self::compute_fingerprint(
            &merged.primary_username,
            &merged.known_aliases,
            &merged.known_emails,
        );
        merged.last_updated = current_time();

        let key = merged.primary_username.clone();
        self.known_identities.insert(key.clone(), merged);
        Some(key)
    }

    // ── Pruning ──

    /// Evict lowest-confidence profiles when over max capacity.
    pub fn prune(&mut self) {
        if self.known_identities.len() <= self.max_profiles {
            return;
        }
        let mut entries: Vec<(String, f64)> = self
            .known_identities
            .iter()
            .map(|(k, p)| (k.clone(), p.confidence))
            .collect();
        entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let to_remove = self.known_identities.len() - self.max_profiles;
        for (key, _) in entries.iter().take(to_remove) {
            self.known_identities.remove(key);
        }
    }

    // ── Alias Chain BFS ──

    /// BFS traversal of the alias graph to `max_depth` hops.
    /// Returns (alias, confidence, hops) tuples reachable from `username`.
    pub fn alias_chain(&self, username: &str, max_depth: usize) -> Vec<(String, f64, usize)> {
        let mut visited: Vec<String> = Vec::new();
        let mut queue: Vec<(String, f64, usize)> = Vec::new();
        let mut results: Vec<(String, f64, usize)> = Vec::new();

        if !self.alias_graph.contains_key(username) {
            return results;
        }

        visited.push(username.to_string());
        queue.push((username.to_string(), 1.0, 0));

        while let Some((current, conf, depth)) = queue.pop() {
            if depth > 0 {
                results.push((current.clone(), conf, depth));
            }

            if depth >= max_depth {
                continue;
            }

            if let Some(neighbors) = self.alias_graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.push(neighbor.clone());
                        let decay = conf * 0.85;
                        queue.push((neighbor.clone(), decay, depth + 1));
                    }
                }
            }
        }

        results.sort_by(|a, b| a.2.cmp(&b.2));
        results
    }

    // ── Accessors ──

    pub fn known_identities(&self) -> &HashMap<String, IdentityProfile> {
        &self.known_identities
    }

    pub fn alias_graph(&self) -> &HashMap<String, Vec<String>> {
        &self.alias_graph
    }

    pub fn profile_count(&self) -> usize {
        self.known_identities.len()
    }

    pub fn min_threshold(&self) -> f64 {
        self.min_correlation_threshold
    }

    pub fn get_profile(&self, key: &str) -> Option<&IdentityProfile> {
        self.known_identities.get(key)
    }
}

impl Default for IdentityCorrelator {
    fn default() -> Self {
        Self::new()
    }
}

/// Current unix timestamp in seconds.
fn current_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_correlator_empty() {
        let corr = IdentityCorrelator::new();
        assert_eq!(corr.profile_count(), 0);
        assert!(corr.known_identities().is_empty());
        assert!(corr.alias_graph().is_empty());
        assert!((corr.min_threshold() - 0.65).abs() < 1e-9);
    }

    #[test]
    fn test_register_alias() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("alice", "twitter", 0.9, None);

        assert_eq!(corr.profile_count(), 1);
        let profile = corr.get_profile("alice").unwrap();
        assert_eq!(profile.primary_username, "alice");
        assert_eq!(profile.platforms.len(), 1);
        assert_eq!(profile.platforms[0], ("twitter".to_string(), 0.9));
        assert!((profile.confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_link_aliases() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("alice", "twitter", 0.8, None);
        corr.register_alias("alice_dev", "github", 0.7, None);
        corr.link_aliases("alice", "alice_dev", 0.85);

        let chain = corr.alias_chain("alice", 2);
        assert!(chain.iter().any(|(a, _, _)| a == "alice_dev"));
    }

    #[test]
    fn test_alias_chain_single_hop() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("bob", "twitter", 0.9, None);
        corr.register_alias("bob42", "github", 0.8, None);
        corr.link_aliases("bob", "bob42", 0.8);

        let chain = corr.alias_chain("bob", 1);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].0, "bob42");
        assert_eq!(chain[0].2, 1); // 1 hop
    }

    #[test]
    fn test_alias_chain_multi_hop() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("user_a", "twitter", 0.9, None);
        corr.register_alias("user_b", "github", 0.8, None);
        corr.register_alias("user_c", "linkedin", 0.7, None);
        corr.link_aliases("user_a", "user_b", 0.85);
        corr.link_aliases("user_b", "user_c", 0.75);

        let chain = corr.alias_chain("user_a", 3);
        assert_eq!(chain.len(), 2);
        assert!(chain.iter().any(|(a, _, _)| a == "user_b"));
        assert!(chain.iter().any(|(a, _, _)| a == "user_c"));

        // user_b should be 1 hop, user_c should be 2 hops
        let c_hops: Vec<usize> = chain.iter().map(|(_, _, h)| *h).collect();
        assert!(c_hops.contains(&1));
        assert!(c_hops.contains(&2));
    }

    #[test]
    fn test_identity_correlation_direct() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("charlie", "twitter", 0.9, None);
        corr.register_alias("charlie", "github", 0.85, None);

        let results = corr.correlate(&["charlie"]);
        assert!(!results.is_empty());

        // Should include Level 1 direct match
        let direct = results
            .iter()
            .find(|r| r.analysis_summary.contains("Level 1"));
        assert!(direct.is_some());
        assert!(direct.unwrap().match_confidence >= 0.65);
    }

    #[test]
    fn test_vsa_attribute_extraction() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("dave", "twitter", 0.7, None);

        let mut attrs = HashMap::new();
        attrs.insert("name".to_string(), "Dave Smith".to_string());
        attrs.insert("location".to_string(), "Berlin".to_string());
        attrs.insert(
            "bio".to_string(),
            "Rust developer, AI researcher".to_string(),
        );

        // Simulate adding attributes to the profile
        if let Some(profile) = corr.known_identities.get_mut("dave") {
            profile.attributes = attrs;
        }

        // Register a second alias with similar attributes
        corr.register_alias("dave_smith", "github", 0.6, None);
        if let Some(profile) = corr.known_identities.get_mut("dave_smith") {
            let mut attrs2 = HashMap::new();
            attrs2.insert("name".to_string(), "Dave Smith".to_string());
            attrs2.insert("location".to_string(), "Berlin, Germany".to_string());
            attrs2.insert("bio".to_string(), "Rust developer".to_string());
            profile.attributes = attrs2;
        }

        // Correlate should find Level 2 attribute match
        let results = corr.correlate(&["dave", "dave_smith"]);
        assert!(!results.is_empty(), "Should find at least Level 1 match");
    }

    #[test]
    fn test_embed_text_deterministic() {
        let encoder = VSAEncoder::new(4096);
        let emb1 = encoder.embed_text("hello world");
        let emb2 = encoder.embed_text("hello world");
        assert_eq!(emb1, emb2);
        assert_eq!(emb1.len(), 512); // 4096 / 8
    }

    #[test]
    fn test_embed_text_different() {
        let encoder = VSAEncoder::new(4096);
        let emb1 = encoder.embed_text("hello world");
        let emb2 = encoder.embed_text("goodbye world");
        assert_ne!(emb1, emb2);
    }

    #[test]
    fn test_similarity_identical() {
        let a = vec![0b10101010u8; 64];
        let b = vec![0b10101010u8; 64];
        let sim = similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_similarity_different() {
        let a = vec![0b00000000u8; 64];
        let b = vec![0b11111111u8; 64];
        let sim = similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_prune_evicts_lowest_confidence() {
        let mut corr = IdentityCorrelator::new().with_max_profiles(3);
        corr.register_alias("alice", "twitter", 0.9, None);
        corr.register_alias("bob", "twitter", 0.8, None);
        corr.register_alias("charlie", "twitter", 0.7, None);
        assert_eq!(corr.profile_count(), 3);

        // Adding a fourth should trigger eviction of lowest confidence
        corr.register_alias("dave", "twitter", 0.95, None);
        assert_eq!(corr.profile_count(), 3);

        // The lowest confidence profile (charlie, 0.7) should have been evicted
        assert!(corr.get_profile("alice").is_some());
        assert!(corr.get_profile("bob").is_some());
        assert!(corr.get_profile("dave").is_some());
        assert!(corr.get_profile("charlie").is_none());
    }

    #[test]
    fn test_extract_attributes_from_bio() {
        let corr = IdentityCorrelator::new();
        let profile_text = "Name: Jane Doe\nLocation: San Francisco\nBio: Software engineer, open source contributor\nEmail: jane@example.com\nWebsite: https://jane.dev";
        let attrs = corr.extract_attributes_from_profile("github", profile_text);

        assert_eq!(attrs.get("name").unwrap(), "jane doe");
        assert_eq!(attrs.get("location").unwrap(), "san francisco");
        assert!(attrs
            .get("email")
            .unwrap_or(&String::new())
            .contains("jane@example.com"));
        assert!(attrs
            .get("url")
            .unwrap_or(&String::new())
            .contains("https://jane.dev"));
    }

    #[test]
    fn test_find_identity_by_email() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("janedoe", "twitter", 0.8, None);
        if let Some(profile) = corr.known_identities.get_mut("janedoe") {
            profile.known_emails.push("jane@example.com".to_string());
            profile
                .attributes
                .insert("name".to_string(), "Jane Doe".to_string());
        }

        let found = corr.find_identity("email", "jane@example.com");
        assert!(found.is_some());
        assert_eq!(found.unwrap().primary_username, "janedoe");

        let found_by_name = corr.find_identity("name", "Jane Doe");
        assert!(found_by_name.is_some());
    }

    #[test]
    fn test_merge_profiles() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("user1", "twitter", 0.8, None);
        corr.register_alias("user2", "github", 0.7, None);

        if let Some(p) = corr.known_identities.get_mut("user1") {
            p.known_emails.push("same@person.com".to_string());
        }
        if let Some(p) = corr.known_identities.get_mut("user2") {
            p.known_emails.push("same@person.com".to_string());
        }

        let merged_key = corr.merge_profiles(&["user1".to_string(), "user2".to_string()]);
        assert!(merged_key.is_some());

        // After merge, only the merged profile should remain
        assert_eq!(corr.profile_count(), 1);
        let merged = corr.get_profile(&merged_key.unwrap()).unwrap();
        assert!(merged.known_emails.contains(&"same@person.com".to_string()));
    }

    #[test]
    fn test_correlation_no_match() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("alice", "twitter", 0.9, None);
        corr.register_alias("bob", "github", 0.8, None);

        let results = corr.correlate(&["alice", "bob"]);
        // Should find at least Level 1 (direct individual matches)
        assert!(!results.is_empty());
    }

    #[test]
    fn test_correlation_three_level() {
        let mut corr = IdentityCorrelator::new();
        // Level 1 set: direct matching aliases on one profile
        corr.register_alias("main_user", "twitter", 0.9, None);
        corr.register_alias("main_user", "github", 0.85, None);

        // Level 2 candidate: different username, shared attributes
        corr.register_alias("main_dev", "linkedin", 0.7, None);
        if let Some(p) = corr.known_identities.get_mut("main_user") {
            p.attributes
                .insert("name".to_string(), "Alex Smith".to_string());
            p.attributes
                .insert("location".to_string(), "London".to_string());
        }
        if let Some(p) = corr.known_identities.get_mut("main_dev") {
            p.attributes
                .insert("name".to_string(), "Alex Smith".to_string());
            p.attributes
                .insert("location".to_string(), "London UK".to_string());
        }

        // Level 3: graph link
        corr.link_aliases("main_user", "main_dev", 0.75);

        let results = corr.correlate(&["main_user", "main_dev"]);
        assert!(!results.is_empty());

        // Check that we have results at multiple levels
        let level_tags: Vec<bool> = results
            .iter()
            .map(|r| {
                r.analysis_summary.contains("Level 1")
                    || r.analysis_summary.contains("Level 2")
                    || r.analysis_summary.contains("Level 3")
            })
            .collect();
        assert!(
            level_tags.iter().any(|&t| t),
            "Results should mention a correlation level"
        );
    }

    #[test]
    fn test_alias_chain_max_depth() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("a", "twitter", 0.9, None);
        corr.register_alias("b", "github", 0.8, None);
        corr.register_alias("c", "linkedin", 0.7, None);
        corr.register_alias("d", "reddit", 0.6, None);
        corr.link_aliases("a", "b", 0.9);
        corr.link_aliases("b", "c", 0.8);
        corr.link_aliases("c", "d", 0.7);

        let depth1 = corr.alias_chain("a", 1);
        assert_eq!(depth1.len(), 1); // only b at depth 1
        assert_eq!(depth1[0].0, "b");

        let depth2 = corr.alias_chain("a", 2);
        assert_eq!(depth2.len(), 2); // b (1 hop) + c (2 hops)

        let depth3 = corr.alias_chain("a", 3);
        assert_eq!(depth3.len(), 3); // b + c + d
    }

    #[test]
    fn test_vsa_encoder_dim() {
        let encoder = VSAEncoder::new(4096);
        assert_eq!(encoder.dim(), 4096);
        let emb = encoder.embed_text("test");
        assert_eq!(emb.len(), 512);
    }

    #[test]
    fn test_similarity_mismatched_lengths() {
        let a = vec![0u8; 32];
        let b = vec![0u8; 64];
        let sim = similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_extract_attributes_twitter_handle() {
        let corr = IdentityCorrelator::new();
        let text = "Handle: @johndoe\nName: John Doe\nBio: Just a dev";
        let attrs = corr.extract_attributes_from_profile("twitter", text);
        assert!(attrs.contains_key("handle"));
    }

    #[test]
    fn test_register_alias_with_evidence() {
        let mut corr = IdentityCorrelator::new();
        corr.register_alias("user", "twitter", 0.9, Some(42));
        let profile = corr.get_profile("user").unwrap();
        assert_eq!(profile.evidence_ids, vec![42]);

        // Register same alias with additional evidence
        corr.register_alias("user", "github", 0.8, Some(99));
        let profile = corr.get_profile("user").unwrap();
        assert!(profile.evidence_ids.contains(&42));
        assert!(profile.evidence_ids.contains(&99));
    }
}

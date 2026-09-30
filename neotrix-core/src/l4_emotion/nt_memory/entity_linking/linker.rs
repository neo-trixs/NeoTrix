#![forbid(unsafe_code)]

//! Entity linker: merges duplicate mentions into unified entity records.
//!
//! After extraction, the linker groups entities that refer to the same
//! real-world thing (fuzzy name matching + type agreement), then produces
//! deduplicated entities with stable IDs and consolidated mention lists.

use std::collections::HashMap;

use super::entity::{Entity, EntityType};

/// Configuration for the entity linker.
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    /// Maximum edit distance (Levenshtein) for fuzzy name matching.
    pub max_edit_distance: usize,
    /// Minimum overlap ratio (0.0..1.0) for name matching.
    pub min_name_overlap: f64,
    /// Prefix for generated stable IDs.
    pub id_prefix: String,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            max_edit_distance: 2,
            min_name_overlap: 0.6,
            id_prefix: "ent".to_string(),
        }
    }
}

/// Entity linker that merges same-entity mentions.
pub struct EntityLinker {
    config: LinkerConfig,
    next_id: u64,
}

impl EntityLinker {
    pub fn new(config: LinkerConfig) -> Self {
        Self { config, next_id: 1 }
    }

    pub fn with_default_config() -> Self {
        Self::new(LinkerConfig::default())
    }

    /// Link a list of extracted entities: group, merge, and assign stable IDs.
    pub fn link(&mut self, entities: Vec<Entity>) -> Vec<Entity> {
        if entities.is_empty() {
            return Vec::new();
        }

        // Group by entity type first, then merge within each group
        let mut by_type: HashMap<EntityType, Vec<Entity>> = HashMap::new();
        for e in entities {
            by_type.entry(e.entity_type).or_default().push(e);
        }

        let mut linked = Vec::new();
        for (etype, group) in by_type {
            let merged = self.merge_group(group, etype);
            linked.extend(merged);
        }

        // Assign stable IDs
        for e in &mut linked {
            if e.id.is_empty() {
                e.id = self.next_id();
            }
        }

        linked
    }

    /// Merge entities within a type group.
    fn merge_group(&self, mut entities: Vec<Entity>, _etype: EntityType) -> Vec<Entity> {
        let mut merged: Vec<Entity> = Vec::new();

        // Sort by mention count descending to use the most-mentioned as the base
        entities.sort_by(|a, b| b.mentions.len().cmp(&a.mentions.len()));

        for entity in entities {
            // Try to find an existing merged entity to append to
            let target = merged
                .iter_mut()
                .find(|m| self.names_match(&m.name, &entity.name));

            match target {
                Some(target) => {
                    // Merge mentions
                    for mention in entity.mentions {
                        // 2026-09-27 修复: 去重键含 offset → 同一实体在文中不同
                        // 位置各留一条, 合并后提及数虚高 (同文两个 "Alice" 不合并)。
                        // 提及列表的语义是"出现过哪些提及", 按 surface 去重。
                        let already_present = target
                            .mentions
                            .iter()
                            .any(|existing| existing.surface == mention.surface);
                        if !already_present {
                            target.mentions.push(mention);
                        }
                    }
                    // Update temporal window
                    if entity.first_seen < target.first_seen {
                        target.first_seen = entity.first_seen;
                    }
                    if entity.last_seen > target.last_seen {
                        target.last_seen = entity.last_seen;
                    }
                    // Prefer longer name as canonical
                    if entity.name.len() > target.name.len() {
                        target.name = entity.name;
                    }
                }
                None => {
                    merged.push(entity);
                }
            }
        }

        merged
    }

    /// Check if two names refer to the same entity.
    fn names_match(&self, a: &str, b: &str) -> bool {
        // Exact match
        if a.eq_ignore_ascii_case(b) {
            return true;
        }

        // Substring containment (e.g., "MIT" matches "Massachusetts Institute of Technology")
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
            // Only if the shorter name is at least 3 chars to avoid false positives
            if a.len().min(b.len()) >= 3 {
                return true;
            }
        }

        // 2026-09-27 补缩写判定: "MIT" 不是 "Massachusetts Institute of Technology"
        // 的子串, 相似度也仅 0.06 → 两者永不合并 (linker_merges_substring_matches
        // 实锤)。全大写短形式按"各词首字母"匹配长形式, 这是缩写消歧的标准做法。
        if acronym_matches(a, b) || acronym_matches(b, a) {
            return true;
        }

        // Levenshtein distance
        let dist = levenshtein(&a_lower, &b_lower);
        // max_len must be in the SAME unit as `dist` (chars). It used to be
        // `a.len()` = bytes, which silently made the ratio unit-mixed once
        // `levenshtein` became char-based — and it was already wrong for
        // mixed-width strings before that. Same reasoning as the distance.
        let max_len = a_lower.chars().count().max(b_lower.chars().count());
        if max_len == 0 {
            return true;
        }
        let similarity = 1.0 - (dist as f64 / max_len as f64);
        similarity >= self.config.min_name_overlap
    }

    /// Generate a stable, deterministic ID.
    fn next_id(&mut self) -> String {
        let id = format!("{}/{:06}", self.config.id_prefix, self.next_id);
        self.next_id += 1;
        id
    }
}

/// 缩写匹配: `acronym` 是否为 `full` 各实词首字母的缩写。
///
/// 判据 (保守, 避免误合并): acronym 全 ASCII 字母、长度 2..=6、全大写;
/// full 至少 2 个词; 逐词首字母 (跳过 "of/the/and" 等连接词) 与 acronym 顺序一致。
fn acronym_matches(acronym: &str, full: &str) -> bool {
    let acr: Vec<char> = acronym.chars().collect();
    if acr.len() < 2 || acr.len() > 6 {
        return false;
    }
    if !acr.iter().all(|c| c.is_ascii_alphabetic() && c.is_ascii_uppercase()) {
        return false;
    }
    const STOP: [&str; 5] = ["of", "the", "and", "for", "de"];
    let initials: Vec<char> = full
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|w| !w.is_empty())
        .filter(|w| !STOP.contains(&w.to_lowercase().as_str()))
        .filter_map(|w| w.chars().next())
        .collect();
    initials.len() >= 2 && initials.len() == acr.len() && initials == acr
}

/// Compute Levenshtein edit distance between two strings, in **characters**.
///
/// 2026-09-30 改为按 `char` 而非 `byte`（行为对位实测抓到，见
/// `.neotrix/parity/levenshtein.vectors.json` 与 strsim@0.11.1 的 oracle）：
/// 旧实现用 `a.len()`/`as_bytes()`，即**按 UTF-8 字节**计数，于是
/// 「中文」vs「中化」得 3（一个汉字=3 字节，每字节都不同）而不是 1。
/// 纯 CJK 字符串的 `dist/max_len` 比值恰好被约掉，但**中英混排**不会：
/// `"a中b"` vs `"ab"` 旧实现算 3/5 → 相似度 0.4，按字符应是 1/3 → 0.667
/// ⇒ 阈值 0.6 一类的配置会**系统性漏合并**只差一个汉字的实体。
/// 参考实现同仓已有字符版（`nt_act_code/semantic_entropy.rs` 的
/// `levenshtein_distance`），本函数此前是**同名的第二份、且语义不同**。
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev = vec![0usize; b_len + 1];
    let mut curr = vec![0usize; b_len + 1];

    for j in 0..=b_len {
        prev[j] = j;
    }

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (curr[j - 1] + 1).min(prev[j] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

/// Merge two entities by consolidating their mentions and temporal windows.
pub fn merge_entities(mut a: Entity, b: Entity) -> Entity {
    for mention in b.mentions {
        // 2026-09-27 同上: 自由函数版按 surface 去重
        let already_present = a.mentions.iter().any(|existing| existing.surface == mention.surface);
        if !already_present {
            a.mentions.push(mention);
        }
    }

    if b.first_seen < a.first_seen {
        a.first_seen = b.first_seen;
    }
    if b.last_seen > a.last_seen {
        a.last_seen = b.last_seen;
    }

    // Prefer longer name
    if b.name.len() > a.name.len() {
        a.name = b.name;
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> i64 {
        1_700_000_000
    }

    #[test]
    fn linker_merges_exact_duplicates() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("Alice", EntityType::Person, 0, ts());
        let mut e2 = Entity::new("Alice", EntityType::Person, 50, ts() + 100);
        e2.add_mention("Alice Smith", 60, ts() + 100);

        let linked = linker.link(vec![e1, e2]);
        assert_eq!(linked.len(), 1);
        assert_eq!(linked[0].mentions.len(), 2);
        assert_eq!(linked[0].first_seen, ts());
        assert_eq!(linked[0].last_seen, ts() + 100);
    }

    #[test]
    fn linker_merges_substring_matches() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("MIT", EntityType::Org, 0, ts());
        let e2 = Entity::new(
            "Massachusetts Institute of Technology",
            EntityType::Org,
            10,
            ts(),
        );

        let linked = linker.link(vec![e1, e2]);
        assert_eq!(linked.len(), 1);
    }

    #[test]
    fn linker_does_not_merge_different_types() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("Apple", EntityType::Org, 0, ts());
        let e2 = Entity::new("Apple", EntityType::Concept, 10, ts());

        let linked = linker.link(vec![e1, e2]);
        assert_eq!(linked.len(), 2);
    }

    #[test]
    fn linker_assigns_stable_ids() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("Alice", EntityType::Person, 0, ts());
        let e2 = Entity::new("Bob", EntityType::Person, 10, ts());

        let linked = linker.link(vec![e1, e2]);
        assert!(linked[0].id.starts_with("ent/"));
        assert!(linked[1].id.starts_with("ent/"));
        assert_ne!(linked[0].id, linked[1].id);
    }

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("abc", "abd"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    /// Regression: this whole test module was ASCII-only, which is exactly why a
    /// **byte**-based implementation survived here (a CJK character is 3 UTF-8
    /// bytes, so "中文"/"中化" scored 3 instead of 1). Found by behavioural
    /// parity against strsim@0.11.1 — see `.neotrix/parity/levenshtein.vectors.json`.
    #[test]
    fn levenshtein_counts_chars_not_bytes() {
        assert_eq!(levenshtein("中文", "中文"), 0);
        assert_eq!(levenshtein("中文", "中化"), 1);
        assert_eq!(levenshtein("知识库", "知识"), 1);
        // Mixed width: one CJK char is one edit, not three.
        assert_eq!(levenshtein("a中b", "ab"), 1);
        assert_eq!(levenshtein("ab", "a中b"), 1);
    }

    /// The similarity ratio must use the same unit as the distance. Before the
    /// 2026-09-30 fix it was `dist(chars) / len(bytes)`.
    ///
    /// Pair choice matters and cost me one wrong test first: "知识库"/"知识库务"
    /// is a **substring** pair, so `names_match` returns true at the containment
    /// branch and never reaches the Levenshtein ratio at all. A pair that differs
    /// in the *last* character and is not contained is required.
    ///
    /// Threshold 0.7 discriminates: char semantics ⇒ 1 - 1/3 = 0.667 (no merge);
    /// byte denominator ⇒ 1 - 1/9 = 0.889 (merge). Verified by reverting the fix
    /// and watching this test fail.
    #[test]
    fn similarity_ratio_uses_chars_not_bytes() {
        let mut linker = EntityLinker::new(LinkerConfig {
            min_name_overlap: 0.7,
            ..LinkerConfig::default()
        });
        let e1 = Entity::new("知识库", EntityType::Concept, 0, ts());
        let e2 = Entity::new("知识阁", EntityType::Concept, 10, ts());

        let linked = linker.link(vec![e1, e2]);
        assert_eq!(
            linked.len(),
            2,
            "'知识库' vs '知识阁' is 1 edit of 3 chars ⇒ similarity 0.667 < 0.7 ⇒ \
             must NOT merge. If this merges, the denominator is bytes again."
        );
    }

    /// Same pair, default threshold: 0.667 ≥ 0.6 ⇒ merges. Proves the test above
    /// pins semantics rather than switching CJK linking off.
    #[test]
    fn should_link_cjk_names_differing_by_one_char() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("知识库", EntityType::Concept, 0, ts());
        let e2 = Entity::new("知识阁", EntityType::Concept, 10, ts());

        let linked = linker.link(vec![e1, e2]);
        assert_eq!(linked.len(), 1, "default 0.6 threshold should merge these");
    }

    #[test]
    fn merge_entities_combines_mentions() {
        let a = Entity::new("Google", EntityType::Org, 0, 100);
        let b = Entity::new("Google Inc", EntityType::Org, 50, 200);
        let merged = merge_entities(a.clone(), b.clone());
        assert_eq!(merged.mentions.len(), 2);
        assert_eq!(merged.first_seen, 100);
        assert_eq!(merged.last_seen, 200);
        assert_eq!(merged.name, "Google Inc");
    }

    #[test]
    fn linker_empty_input() {
        let mut linker = EntityLinker::with_default_config();
        let linked = linker.link(vec![]);
        assert!(linked.is_empty());
    }

    #[test]
    fn custom_id_prefix() {
        let config = LinkerConfig {
            id_prefix: "my".into(),
            ..Default::default()
        };
        let mut linker = EntityLinker::new(config);
        let e = Entity::new("Alice", EntityType::Person, 0, ts());
        let linked = linker.link(vec![e]);
        assert!(linked[0].id.starts_with("my/"));
    }

    #[test]
    fn merge_entities_no_new_duplicates() {
        let mut a = Entity::new("Foo", EntityType::Org, 0, 100);
        a.add_mention("Foo Inc", 50, 200);
        let mut b = Entity::new("Foo", EntityType::Org, 10, 150);
        b.add_mention("Foo Inc", 55, 250);
        let merged = merge_entities(a, b);
        let unique_surfaces: Vec<&str> =
            merged.mentions.iter().map(|m| m.surface.as_str()).collect();
        assert_eq!(merged.mentions.len(), 2);
        assert!(unique_surfaces.contains(&"Foo"));
        assert!(unique_surfaces.contains(&"Foo Inc"));
    }
}

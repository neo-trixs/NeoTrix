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
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
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
                        // ⚠️ 2026-10-02：下面这段去重规则的**语义存在未裁决冲突**，
                        // 两种读法都被测试断言着，详见
                        // `docs/architecture/MENTION-SEMANTICS-CONFLICT-2026-10-02.md`：
                        //
                        // · **当前实现（读法 A）**：`mentions` = 不同 **surface 形式**的集合。
                        //   被 `linker_merges_exact_duplicates`(断言 2) 与
                        //   `merge_entities_no_new_duplicates`(断言 2) 钉住。
                        // · **另一读法 B**：集成测试 `entity_linking_dedup_across_mentions`
                        //   期望 `mentions.len() >= 2`（两个 "Alice Smith" 在 offset 0/35
                        //   应记两次）⇒ `mentions` = **出现次数**。
                        //
                        // ⚠️ 2026-09-27 那次改动把注释写成「去重键含 offset 是问题」，
                        // 紧接着又按 surface 去重 —— **那段注释自相矛盾**，
                        // 会让下一个 agent 按错误读法理解代码。
                        //
                        // 倾向 B 的理由：`Mention` 带 `offset` 字段，其自述为
                        // 「在源文本中的偏移」⇒ 该字段**只为区分同 surface 的不同次出现**
                        // 而存在；按 surface 去重等于丢弃它携带的信息。
                        // ⛔ 但改动需同时重写上面 2 条绿测，故**不在此处单方面裁决**。
                        let already_present = target
                            .mentions
                            .iter()
                            .any(|existing| existing.surface == mention.surface && existing.offset == mention.offset);
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
        // 2026-10-02：与 `merge_group` **同一判据**，去重键为 (surface, offset)。
        // ⚠️ 这条规则此前在**两处各有一份副本**（此处 + `merge_group` 内），
        // 改一处漏一处 —— 上一次改 `merge_group` 时它仍是旧的 surface 去重，
        // 实测表现为「同一规则两种行为」。⇒ 现已两处一致；
        // 若日后要再改判据，**必须同时改这两处**（或把它们收敛成单一函数）。
        let already_present = a
            .mentions
            .iter()
            .any(|existing| existing.surface == mention.surface && existing.offset == mention.offset);
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
        // 2026-10-02 语义裁决：`mentions` = **出现次数**。
        // 原断言 `== 2`（surface 去重：{Alice} ∪ {Alice, Alice Smith}）
        // 在 occurrence 语义下应为 3 —— Alice 出现在 offset 0 与 50，
        // 是**两次不同的出现**（这正是 `Mention.offset` 存在的理由）。
        assert_eq!(
            linked[0].mentions.len(),
            3,
            "occurrence 语义：Alice@0、Alice@50、Alice Smith@60 共 3 次"
        );
        // 「出现过哪些不同写法」仍可得 —— 由派生方法提供，未丢失。
        let surfaces = linked[0].unique_surfaces();
        assert_eq!(surfaces.len(), 2, "不同 surface 形式仍是 2 种");
        assert!(surfaces.contains(&"Alice"));
        assert!(surfaces.contains(&"Alice Smith"));
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
        // 2026-10-02：本测试的**原始意图**是「合并不产生重复写法」。
        // occurrence 语义下 `mentions` 保留全部出现（4 次），
        // 而「不重复的写法」改由派生方法 `unique_surfaces()` 提供
        // ⇒ **意图完全保留，且不再需要在 merge 时销毁数据**。
        assert_eq!(merged.mentions.len(), 4, "Foo@0、FooInc@50、Foo@10、FooInc@55");
        let unique_surfaces = merged.unique_surfaces();
        assert_eq!(
            unique_surfaces.len(),
            2,
            "不同 surface 形式去重后仍为 2"
        );
        assert!(unique_surfaces.contains(&"Foo"));
        assert!(unique_surfaces.contains(&"Foo Inc"));
    }
}

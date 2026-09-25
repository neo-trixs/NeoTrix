//! nt_extractor — 启发式抽取管线: 切句/实体/关系 helpers + ExtractionConfig/GraphExtractor.
//! 从 `nt_memory_graphrag/mod.rs` 纯搬移, 行为零变更.

use std::collections::HashMap;

use super::nt_store::GraphRagStore;
use super::nt_types::{generate_id, now_nanos, EntityNode, RelationEdge};

// ─── Heuristic Extraction Helpers ────────────────────────────────────

pub(crate) fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        current.push(c);
        if matches!(c, '.' | '!' | '?') {
            // Check if this is likely an abbreviation by looking at what follows
            let is_abbreviation = if c == '.' {
                // If followed by another letter (no space), it's an abbreviation
                if i + 1 < len && chars[i + 1].is_alphabetic() {
                    true
                } else if i + 2 < len && chars[i + 1] == ' ' && chars[i + 2].is_lowercase() {
                    // "word. more" — the period ends a sentence
                    false
                } else {
                    false
                }
            } else {
                false
            };

            if !is_abbreviation {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() && trimmed.len() > 2 {
                    sentences.push(trimmed);
                }
                current = String::new();
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() && trimmed.len() > 2 {
        sentences.push(trimmed);
    }

    sentences
}

/// P1-12 稳定切片重建 (吸收 GEOFlow 模式):
/// LLM 只规划边界, 切片从原文稳定重建 — 不依赖 LLM 每次输出漂移。
/// 实现: 按段落 (空行) + 标题 (Markdown #) 规划边界, 从原文逐段重建切片,
/// 保证同一文档多次切片结果一致 (确定性)。
/// GEOFlow 原文: "LLM only plans boundaries; slices are stably rebuilt
/// from the original text"。
///
/// 返回 (切片列表, 边界规划)。边界规划可被 LLM 覆盖 (外部传入), 但默认确定性。
pub fn stable_slice_document(text: &str, max_chars: usize) -> (Vec<String>, Vec<usize>) {
    // 1) 边界规划: 段落边界 (空行) + 标题边界 (Markdown # 开头)
    let mut boundaries: Vec<usize> = Vec::new();
    let mut current_len = 0usize;
    for (idx, line) in text.lines().enumerate() {
        let line_len = line.chars().count() + 1; // +1 换行
        let is_heading = line.trim_start().starts_with('#');
        let is_para_break = line.trim().is_empty();
        if (is_heading || is_para_break) && idx > 0 && current_len > 0 {
            boundaries.push(idx);
            current_len = 0;
        }
        current_len += line_len;
        if current_len >= max_chars {
            boundaries.push(idx);
            current_len = 0;
        }
    }

    // 2) 原文重建: 按边界切分, 保证切片内容与原文逐字一致
    let lines: Vec<&str> = text.lines().collect();
    let mut slices = Vec::new();
    let mut start = 0usize;
    for &b in &boundaries {
        if b > start {
            let slice = lines[start..b].join("\n");
            if !slice.trim().is_empty() {
                slices.push(slice);
            }
        }
        start = b;
    }
    if start < lines.len() {
        let slice = lines[start..].join("\n");
        if !slice.trim().is_empty() {
            slices.push(slice);
        }
    }

    (slices, boundaries)
}

pub(crate) fn extract_capitalized_terms(sentence: &str, _source_id: &str) -> Vec<String> {
    let mut entities = Vec::new();
    let words: Vec<&str> = sentence.split_whitespace().collect();
    let mut i = 0;

    while i < words.len() {
        let clean = words[i]
            .trim_start_matches(|c: char| !c.is_alphanumeric())
            .trim_end_matches(|c: char| !c.is_alphanumeric());

        if clean.is_empty() || clean.len() < 2 {
            i += 1;
            continue;
        }

        let first_char = clean.chars().next().unwrap_or(' ');
        if first_char.is_uppercase() || first_char.is_ascii_digit() {
            let mut term_parts: Vec<String> = Vec::new();
            let mut j = i;

            while j < words.len() {
                let w = words[j]
                    .trim_start_matches(|c: char| !c.is_alphanumeric())
                    .trim_end_matches(|c: char| !c.is_alphanumeric());

                if w.is_empty() || w.len() < 2 {
                    break;
                }
                let wfc = w.chars().next().unwrap_or(' ');
                if !wfc.is_uppercase() && !wfc.is_ascii_digit() {
                    break;
                }
                // Stop at connecting words that are not proper nouns
                let wlower = w.to_lowercase();
                if !term_parts.is_empty()
                    && matches!(
                        wlower.as_str(),
                        "the"
                            | "a"
                            | "an"
                            | "and"
                            | "or"
                            | "but"
                            | "in"
                            | "on"
                            | "at"
                            | "for"
                            | "with"
                            | "by"
                            | "to"
                            | "of"
                            | "is"
                            | "are"
                            | "was"
                            | "were"
                    )
                {
                    break;
                }
                term_parts.push(w.to_string());
                j += 1;
            }

            if !term_parts.is_empty() {
                let term = term_parts.join(" ");
                // Filter out common non-entity single capitalized words
                let tlower = term.to_lowercase();
                if !matches!(
                    tlower.as_str(),
                    "this"
                        | "that"
                        | "these"
                        | "those"
                        | "they"
                        | "what"
                        | "which"
                        | "when"
                        | "where"
                        | "why"
                        | "how"
                        | "there"
                        | "here"
                        | "then"
                        | "than"
                        | "thus"
                        | "hence"
                        | "very"
                        | "just"
                        | "also"
                        | "only"
                        | "more"
                        | "most"
                        | "some"
                        | "any"
                        | "each"
                        | "every"
                        | "both"
                        | "such"
                        | "because"
                        | "while"
                        | "although"
                        | "however"
                        | "therefore"
                        | "moreover"
                        | "furthermore"
                        | "nevertheless"
                        | "nonetheless"
                        | "accordingly"
                        | "consequently"
                        | "additionally"
                ) && term.len() > 1
                {
                    entities.push(term);
                }
            }

            i = j;
        } else {
            i += 1;
        }
    }

    entities
}

pub(crate) fn infer_entity_type(name: &str) -> String {
    let lower = name.to_lowercase();

    if lower.ends_with("inc")
        || lower.ends_with("corp")
        || lower.ends_with("ltd")
        || lower.ends_with("llc")
        || lower.ends_with("company")
        || lower.ends_with("corporation")
        || lower.ends_with("foundation")
        || lower.ends_with("institute")
        || lower.ends_with("organization")
        || lower.ends_with("association")
        || lower.ends_with("group")
        || lower.ends_with("laboratories")
        || lower.ends_with("lab")
        || lower.ends_with("limited")
        || lower.contains("university")
        || lower.contains("college")
        || lower.contains("school")
        || lower.contains("department of")
    {
        return "Organization".to_string();
    }

    if lower.starts_with("dr ")
        || lower.starts_with("prof ")
        || lower.starts_with("mr ")
        || lower.starts_with("ms ")
        || lower.starts_with("mrs ")
        || lower.starts_with("sir ")
        || lower.starts_with("lord ")
    {
        return "Person".to_string();
    }

    if lower.contains("system")
        || lower.contains("framework")
        || lower.contains("tool")
        || lower.contains("language")
        || lower.contains("platform")
        || lower.contains("software")
        || lower.contains("algorithm")
        || lower.contains("database")
        || lower.contains("protocol")
        || lower.contains("engine")
        || lower.contains("runtime")
        || lower.contains("library")
        || lower.contains("api")
        || lower.contains("sdk")
        || lower.contains("kernel")
        || lower.contains("module")
        || lower.contains("network")
        || lower.contains("model")
        || lower.contains("transformer")
        || lower.contains("architecture")
    {
        return "Technology".to_string();
    }

    if lower.ends_with("city")
        || lower.ends_with("ville")
        || lower.ends_with("burg")
        || lower.ends_with("town")
        || lower.ends_with("shire")
        || lower.ends_with("land")
        || lower.ends_with("stan")
        || lower.ends_with("valley")
        || lower.ends_with("beach")
        || lower.ends_with("bay")
        || lower.ends_with("county")
        || lower.ends_with("province")
        || lower.ends_with("state")
        || lower.ends_with("kingdom")
        || lower.contains("republic of")
        || lower.contains("city of")
    {
        return "Location".to_string();
    }

    if lower.contains("conference")
        || lower.contains("summit")
        || lower.contains("workshop")
        || lower.contains("symposium")
        || lower.contains("hackathon")
        || lower.contains("competition")
        || lower.contains("challenge")
        || lower.contains("tournament")
        || lower.contains("exhibition")
        || lower.contains("convention")
    {
        return "Event".to_string();
    }

    "Concept".to_string()
}

pub(crate) fn estimate_entity_confidence(name: &str, sentence: &str) -> f64 {
    let lower = name.to_lowercase();
    let mut confidence: f64 = 0.7;

    // Longer, more specific names get higher confidence
    let word_count = name.split_whitespace().count();
    if word_count >= 3 {
        confidence += 0.15;
    } else if word_count >= 2 {
        confidence += 0.05;
    }

    // Type keywords boost confidence
    if lower.ends_with("inc")
        || lower.ends_with("corp")
        || lower.ends_with("ltd")
        || lower.contains("university")
    {
        confidence += 0.15;
    }

    // If the entity appears multiple times in the sentence, higher confidence
    let count = sentence.to_lowercase().matches(&lower).count();
    if count > 1 {
        confidence += 0.1;
    }

    // Single capitalized word that's common → lower confidence
    if word_count == 1 {
        let common_single = [
            "hello",
            "world",
            "this",
            "that",
            "these",
            "those",
            "today",
            "tomorrow",
            "yesterday",
            "now",
            "here",
            "there",
        ];
        if common_single.contains(&lower.as_str()) {
            confidence -= 0.3;
        }
        // Very short words
        if name.len() <= 3 {
            confidence -= 0.2;
        }
    }

    (confidence.max(0.1f64)).min(1.0f64)
}

pub(crate) fn detect_relation(e1: &str, e2: &str, sentence: &str) -> Option<(&'static str, f64)> {
    let s_lower = sentence.to_lowercase();
    let e1_lower = e1.to_lowercase();
    let e2_lower = e2.to_lowercase();

    // Find positions in the lowercased sentence
    let pos1 = s_lower.find(&e1_lower)?;
    let pos2 = s_lower.find(&e2_lower)?;

    let between = if pos1 < pos2 {
        &s_lower[pos1 + e1_lower.len()..pos2]
    } else {
        &s_lower[pos2 + e2_lower.len()..pos1]
    };

    let between = between.trim();

    let rel_type = if between.contains("works at")
        || between.contains("employed by")
        || between.contains("ceo of")
        || between.contains("cfo of")
        || between.contains("cto of")
        || between.contains("employee of")
        || between.contains("founder of")
        || between.contains("chairman of")
        || between.contains("president of")
        || between.contains("director of")
        || between.contains("manager of")
        || between.contains("led by")
        || between.contains("run by")
        || between.contains("staff of")
        || between.contains("team at")
    {
        "works_at"
    } else if between.contains("developed")
        || between.contains("created")
        || between.contains("built")
        || between.contains("designed")
        || between.contains("invented")
        || between.contains("wrote")
        || between.contains("authored")
        || between.contains("published")
        || between.contains("produced")
        || between.contains("engineered")
        || between.contains("founded")
        || between.contains("established")
        || between.contains("launched")
        || between.contains("introduced")
        || between.contains("released")
        || between.contains("originated from")
        || between.contains("created by")
        || between.contains("developed by")
        || between.contains("built by")
        || between.contains("designed by")
        || between.contains("authored by")
    {
        "developed_by"
    } else if between.contains("part of")
        || between.contains("component of")
        || between.contains("belongs to")
        || between.contains("member of")
        || between.contains("subsidiary of")
        || between.contains("division of")
        || between.contains("unit of")
        || between.contains("segment of")
        || between.contains("element of")
        || between.contains("subset of")
        || between.contains("included in")
        || between.contains("within")
    {
        "part_of"
    } else if between.contains("located in")
        || between.contains("based in")
        || between.contains("headquartered")
        || between.contains("situated in")
        || between.contains("founded in")
        || between.contains("established in")
        || between.contains("head office in")
    {
        "located_in"
    } else if between.contains("uses")
        || between.contains("utilizes")
        || between.contains("integrates")
        || between.contains("runs on")
        || between.contains("built on")
        || between.contains("powered by")
        || between.contains("driven by")
        || between.contains("supports")
        || between.contains("compatible with")
        || between.contains("implemented with")
        || between.contains("implemented in")
        || between.contains("written in")
        || between.contains("built with")
        || between.contains("based on")
        || between.contains("relies on")
        || between.contains("depends on")
        || between.contains("leveraging")
        || between.contains("powered by")
    {
        "used_by"
    } else {
        "related_to"
    };

    // Distance = number of words between the entities
    let distance = between.split_whitespace().count().max(1) as f64;

    Some((rel_type, distance))
}

// ─── Extraction Pipeline ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub model: String,
    pub max_entities_per_chunk: usize,
    pub confidence_threshold: f64,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        ExtractionConfig {
            model: "heuristic".to_string(),
            max_entities_per_chunk: 50,
            confidence_threshold: 0.3,
        }
    }
}

pub struct GraphExtractor {
    config: ExtractionConfig,
}

impl GraphExtractor {
    pub fn new(config: ExtractionConfig) -> Self {
        GraphExtractor { config }
    }

    pub fn config(&self) -> &ExtractionConfig {
        &self.config
    }

    pub fn extract(
        &self,
        text: &str,
        source_id: &str,
    ) -> Result<(Vec<EntityNode>, Vec<RelationEdge>), String> {
        let sentences = split_sentences(text);
        let mut entities_map: HashMap<String, EntityNode> = HashMap::new();
        let mut relations: Vec<RelationEdge> = Vec::new();
        let mut sentence_entity_names: Vec<Vec<String>> = Vec::new();

        for sentence in &sentences {
            if sentence.len() < 3 {
                sentence_entity_names.push(Vec::new());
                continue;
            }
            let names = extract_capitalized_terms(sentence, "extractor");
            let filtered: Vec<String> = names
                .into_iter()
                .filter(|n| {
                    let conf = estimate_entity_confidence(n, sentence);
                    conf >= self.config.confidence_threshold
                })
                .collect();
            sentence_entity_names.push(filtered.clone());

            for name in filtered {
                let key = name.to_lowercase();
                if !entities_map.contains_key(&key) {
                    if entities_map.len() >= self.config.max_entities_per_chunk {
                        break;
                    }
                    let etype = infer_entity_type(&name);
                    entities_map.insert(
                        key,
                        EntityNode {
                            id: generate_id(),
                            name: name.clone(),
                            entity_type: etype,
                            source_node_id: source_id.to_string(),
                            confidence: self.config.confidence_threshold,
                            properties: HashMap::new(),
                            created_at: now_nanos(),
                        },
                    );
                }
            }
        }

        for (s_idx, sentence) in sentences.iter().enumerate() {
            let names = &sentence_entity_names[s_idx];
            if names.len() < 2 {
                continue;
            }
            for i in 0..names.len() {
                for j in (i + 1)..names.len() {
                    if let Some((rel_type, distance)) =
                        detect_relation(&names[i], &names[j], sentence)
                    {
                        let e1_lower = names[i].to_lowercase();
                        let e2_lower = names[j].to_lowercase();
                        if let (Some(e1), Some(e2)) =
                            (entities_map.get(&e1_lower), entities_map.get(&e2_lower))
                        {
                            let weight = (1.0 / distance.max(1.0))
                                * self.config.confidence_threshold.max(0.5);
                            relations.push(RelationEdge {
                                id: generate_id(),
                                source_entity: e1.id.clone(),
                                target_entity: e2.id.clone(),
                                relation_type: rel_type.to_string(),
                                weight: (weight.max(0.0)).min(1.0),
                                evidence: String::new(),
                                confidence: self.config.confidence_threshold,
                                created_at: now_nanos(),
                            });
                        }
                    }
                }
            }
        }

        let entities: Vec<EntityNode> = entities_map.into_values().collect();
        Ok((entities, relations))
    }

    pub fn extract_and_store(
        &self,
        text: &str,
        store: &mut GraphRagStore,
        source_id: &str,
    ) -> Result<(), String> {
        let (entities, relations) = self.extract(text, source_id)?;

        let mut entity_id_map: HashMap<String, String> = HashMap::new();
        for entity in &entities {
            let key = entity.name.to_lowercase();
            let existing = store
                .graph()
                .entities
                .values()
                .find(|e| e.name.to_lowercase() == key);
            let store_id = if let Some(existing) = existing {
                existing.id.clone()
            } else {
                store.add_entity(entity.clone());
                entity.id.clone()
            };
            entity_id_map.insert(key, store_id);
        }

        for relation in &relations {
            let source_key = entities
                .iter()
                .find(|e| e.id == relation.source_entity)
                .map(|e| e.name.to_lowercase());
            let target_key = entities
                .iter()
                .find(|e| e.id == relation.target_entity)
                .map(|e| e.name.to_lowercase());
            if let (Some(src_key), Some(tgt_key)) = (source_key, target_key) {
                if let (Some(sid), Some(tid)) =
                    (entity_id_map.get(&src_key), entity_id_map.get(&tgt_key))
                {
                    store.add_relation(RelationEdge {
                        id: generate_id(),
                        source_entity: sid.clone(),
                        target_entity: tid.clone(),
                        relation_type: relation.relation_type.clone(),
                        weight: relation.weight,
                        evidence: String::new(),
                        confidence: relation.weight,
                        created_at: now_nanos(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn merge_entities(entities: &[EntityNode]) -> Vec<EntityNode> {
        let mut seen: HashMap<String, EntityNode> = HashMap::new();
        for entity in entities {
            let key = entity.name.to_lowercase();
            seen.entry(key).or_insert_with(|| entity.clone());
        }
        seen.into_values().collect()
    }

    pub fn generate_prompt(&self, text: &str) -> String {
        format!(
            r#"Extract entities and relations from the following text.

Entities should be: name, type (Person/Organization/Location/Technology/Concept/Event), and optional properties.
Relations should be: source → relation_type → target.

Return as JSON:
{{
  "entities": [{{"name": "...", "type": "...", "properties": {{...}}}}],
  "relations": [{{"source": "...", "target": "...", "type": "..."}}]
}}

Text:
{}

Max entities: {}"#,
            text, self.config.max_entities_per_chunk
        )
    }
}

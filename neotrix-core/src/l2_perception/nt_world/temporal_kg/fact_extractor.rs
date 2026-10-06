use chrono::NaiveDateTime;
use regex::Regex;

use super::entity::Entity;
use super::relation::TemporalRelation;

pub struct FactExtractor;

impl FactExtractor {
    pub fn extract_facts(text: &str, entities: &[Entity]) -> Vec<TemporalRelation> {
        let mut facts = Vec::new();
        let date_re =
            Regex::new(r"(?i)\b(\d{4}-\d{2}-\d{2}(?:\s+\d{2}:\d{2}(?::\d{2})?)?)\b").unwrap();

        for cap in date_re.captures_iter(text) {
            let date_str = cap.get(1).unwrap().as_str().trim();
            let parsed = parse_datetime(date_str);
            if parsed.is_none() {
                continue;
            }
            let t = parsed.unwrap();

            let prefix = &text[..cap.get(0).unwrap().start()];
            let suffix = &text[cap.get(0).unwrap().end()..];

            let context = format!("{}{}", prefix, suffix);
            if let Some((subject, object, rel)) = extract_triple(&context, entities) {
                facts.push(TemporalRelation::new(subject, object, rel, t));
            }
        }

        facts
    }
}

fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    if let Ok(t) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(t);
    }
    if let Ok(t) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return Some(t);
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d.and_hms_opt(0, 0, 0).unwrap());
    }
    None
}

fn extract_triple(context: &str, entities: &[Entity]) -> Option<(String, String, String)> {
    let rel_patterns: Vec<(&str, &str)> = vec![
        (
            r"(?i)(\w+)\s+(?:works at|joined|employed by)\s+(\w+)",
            "works_at",
        ),
        (
            r"(?i)(\w+)\s+(?:created|founded|launched)\s+(\w+)",
            "created",
        ),
        (r"(?i)(\w+)\s+(?:knows|met|met with)\s+(\w+)", "knows"),
        (r"(?i)(\w+)\s+(?:visited|traveled to)\s+(\w+)", "visited"),
        (r"(?i)(\w+)\s+(?:owns|possesses)\s+(\w+)", "owns"),
        (r"(?i)(\w+)\s+(?:manages|leads|directs)\s+(\w+)", "manages"),
    ];

    for (pattern, rel_type) in &rel_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(context) {
                let subj = caps.get(1).map(|m| m.as_str().to_string())?;
                let obj = caps.get(2).map(|m| m.as_str().to_string())?;

                let subj_id = find_entity_id(&subj, entities)
                    .unwrap_or_else(|| format!("entity_{}", subj.to_lowercase()));
                let obj_id = find_entity_id(&obj, entities)
                    .unwrap_or_else(|| format!("entity_{}", obj.to_lowercase()));

                return Some((subj_id, obj_id, rel_type.to_string()));
            }
        }
    }

    let words: Vec<&str> = context.split_whitespace().collect();
    if words.len() < 2 {
        return None;
    }

    let mut found_subject: Option<&Entity> = None;
    let mut found_object: Option<&Entity> = None;

    for word in &words {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();
        if cleaned.len() < 2 {
            continue;
        }

        if let Some(e) = entities.iter().find(|e| e.name == cleaned) {
            if found_subject.is_none() {
                found_subject = Some(e);
            } else if found_object.is_none() {
                found_object = Some(e);
            }
        }
    }

    if let (Some(subj), Some(obj)) = (found_subject, found_object) {
        let rel = infer_relation_type(context);
        return Some((subj.id.clone(), obj.id.clone(), rel));
    }

    None
}

fn find_entity_id(name: &str, entities: &[Entity]) -> Option<String> {
    entities
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .map(|e| e.id.clone())
}

fn infer_relation_type(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains("works") || lower.contains("employ") || lower.contains("join") {
        "works_at".to_string()
    } else if lower.contains("creat") || lower.contains("found") || lower.contains("launch") {
        "created".to_string()
    } else if lower.contains("know") || lower.contains("met") {
        "knows".to_string()
    } else if lower.contains("visit") || lower.contains("travel") {
        "visited".to_string()
    } else if lower.contains("own") || lower.contains("possess") {
        "owns".to_string()
    } else if lower.contains("manage") || lower.contains("lead") || lower.contains("direct") {
        "manages".to_string()
    } else {
        "related_to".to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::entity::EntityType;
    use super::*;

    fn dt(year: i32, month: u32, day: u32) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year,month,day)
            .and_then(|d| d.and_hms_opt(0,0,0))
            .expect("测试时间戳构造：日期/时间应合法")
    }

    fn make_entities() -> Vec<Entity> {
        vec![
            Entity {
                id: "e1".into(),
                name: "Alice".into(),
                entity_type: EntityType::Person,
                properties: HashMap::new(),
            },
            Entity {
                id: "e2".into(),
                name: "Google".into(),
                entity_type: EntityType::Organization,
                properties: HashMap::new(),
            },
        ]
    }

    #[test]
    fn test_extract_date_pattern() {
        let entities = make_entities();
        let text = "Alice joined Google on 2024-03-15";
        let facts = FactExtractor::extract_facts(text, &entities);
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].valid_from, dt(2024, 3, 15));
        assert_eq!(facts[0].relation_type, "works_at");
    }

    #[test]
    fn test_extract_datetime_pattern() {
        let entities = make_entities();
        let text = "Alice met Google on 2024-06-15 14:30";
        let facts = FactExtractor::extract_facts(text, &entities);
        assert_eq!(facts.len(), 1);
        assert_eq!(
            facts[0].valid_from,
            chrono::NaiveDate::from_ymd_opt(2024,6,15)
            .and_then(|d| d.and_hms_opt(14,30,0))
            .expect("测试时间戳构造：日期/时间应合法")
        );
    }

    #[test]
    fn test_extract_multiple_facts() {
        let entities = make_entities();
        let text = "Alice created Google on 2024-01-01. Alice visited Google on 2024-06-01.";
        let facts = FactExtractor::extract_facts(text, &entities);
        assert!(facts.len() >= 2);
    }

    #[test]
    fn test_no_date_yields_no_facts() {
        let entities = make_entities();
        let text = "Alice joined Google";
        let facts = FactExtractor::extract_facts(text, &entities);
        assert!(facts.is_empty());
    }

    #[test]
    fn test_infer_relation_type() {
        assert_eq!(infer_relation_type("Alice works at Google"), "works_at");
        assert_eq!(infer_relation_type("Alice created Widget"), "created");
        assert_eq!(infer_relation_type("Alice knows Bob"), "knows");
        assert_eq!(infer_relation_type("Alice visited Paris"), "visited");
        assert_eq!(infer_relation_type("Alice owns a car"), "owns");
        assert_eq!(infer_relation_type("Alice manages the team"), "manages");
        assert_eq!(infer_relation_type("random text"), "related_to");
    }

    #[test]
    fn test_parse_datetime_formats() {
        assert!(parse_datetime("2024-01-01").is_some());
        assert!(parse_datetime("2024-01-01 12:30").is_some());
        assert!(parse_datetime("2024-01-01 12:30:45").is_some());
        assert!(parse_datetime("not-a-date").is_none());
    }

    #[test]
    fn test_extract_triple_with_explicit_pattern() {
        let entities = make_entities();
        let text = "Alice works at Google";
        let result = extract_triple(text, &entities);
        assert!(result.is_some());
        let (s, o, r) = result.unwrap();
        assert_eq!(s, "e1");
        assert_eq!(o, "e2");
        assert_eq!(r, "works_at");
    }
}

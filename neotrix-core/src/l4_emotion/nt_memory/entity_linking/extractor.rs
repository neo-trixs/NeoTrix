#![forbid(unsafe_code)]

//! Rule-based entity extraction from text.
//!
//! Uses regex patterns to identify Person, Org, Concept, Event, and Place
//! mentions. This is a lightweight first pass — the linker handles dedup.

use regex::Regex;
use std::sync::LazyLock;

use super::entity::{Entity, EntityType};

/// Known organization suffixes (English).
static ORG_SUFFIXES: &[&str] = &[
    "Inc",
    "Corp",
    "Corporation",
    "LLC",
    "Ltd",
    "Limited",
    "Co",
    "Company",
    "Institute",
    "Foundation",
    "Association",
    "University",
    "College",
    "Laboratory",
    "Lab",
    "Group",
    "Agency",
    "Bureau",
    "Department",
    "Ministry",
    "Council",
    "Committee",
    "Commission",
    "Authority",
    "Bank",
    "Fund",
    "Society",
    "Center",
    "Centre",
];

/// Known event keywords.
static EVENT_KEYWORDS: &[&str] = &[
    "conference",
    "summit",
    "workshop",
    "symposium",
    "meeting",
    "festival",
    "expo",
    "exhibition",
    "ceremony",
    "launch",
    "webinar",
    "hackathon",
    "retreat",
    "convention",
];

/// Named person: capitalized first + last name, or titles.
static RE_PERSON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:(?:Mr|Mrs|Ms|Dr|Prof|Sir|Dame)\s+)?([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)"#).unwrap()
});

/// Organization: word(s) followed by known suffix.
static RE_ORG: LazyLock<Regex> = LazyLock::new(|| {
    let suffixes = ORG_SUFFIXES.join("|");
    Regex::new(&format!(
        r#"([A-Z][A-Za-z]*(?:\s+[A-Z][A-Za-z]*)*\s+(?:{}))"#,
        suffixes
    ))
    .unwrap()
});

/// Date patterns: YYYY-MM-DD or Month DD, YYYY.
static RE_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\b(\d{4}-\d{2}-\d{2})\b|(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2},?\s+\d{4}"#)
        .unwrap()
});

/// Concept: quoted phrases or technical terms (ALL_CAPS words).
static RE_CONCEPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""([^"]{2,50})"|\b([A-Z]{2,}(?:-[A-Z]{2,})*)\b"#).unwrap());

/// Known place suffixes.
static PLACE_SUFFIXES: &[&str] = &[
    "City",
    "Town",
    "Village",
    "County",
    "State",
    "Province",
    "Street",
    "Avenue",
    "Boulevard",
    "Road",
    "Drive",
    "Park",
    "Square",
    "Plaza",
    "Bridge",
];

/// Place pattern.
static RE_PLACE: LazyLock<Regex> = LazyLock::new(|| {
    let suffixes = PLACE_SUFFIXES.join("|");
    Regex::new(&format!(
        r#"([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\s+(?:{}))"#,
        suffixes
    ))
    .unwrap()
});

/// Extract entities from text using rule-based regex patterns.
pub fn extract_entities(text: &str) -> Vec<Entity> {
    let now = epoch_now();
    let mut entities = Vec::new();

    // Organizations (check before persons to avoid false positives)
    for cap in RE_ORG.find_iter(text) {
        let surface = cap.as_str().to_string();
        let offset = cap.start();
        entities.push(Entity::new(surface, EntityType::Org, offset, now));
    }

    // Persons
    for cap in RE_PERSON.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            let surface = m.as_str().to_string();
            let offset = m.start();
            // Skip if already captured as an org
            if entities
                .iter()
                .any(|e| e.entity_type == EntityType::Org && e.name == surface)
            {
                continue;
            }
            entities.push(Entity::new(surface, EntityType::Person, offset, now));
        }
    }

    // Events
    for cap in RE_DATE.find_iter(text) {
        let surface = cap.as_str().to_string();
        let offset = cap.start();
        entities.push(Entity::new(surface, EntityType::Event, offset, now));
    }

    // Concepts
    for cap in RE_CONCEPT.captures_iter(text) {
        let surface = cap
            .get(1)
            .or_else(|| cap.get(2))
            .map(|m| m.as_str().to_string());
        if let Some(s) = surface {
            let offset = cap.get(1).or_else(|| cap.get(2)).unwrap().start();
            // Skip single all-caps words shorter than 3 chars
            if s.len() < 3 {
                continue;
            }
            entities.push(Entity::new(s, EntityType::Concept, offset, now));
        }
    }

    // Places
    for cap in RE_PLACE.find_iter(text) {
        let surface = cap.as_str().to_string();
        let offset = cap.start();
        // 2026-09-27 修复: 去重只看 name, 于是同一字符串先前被 Person/Org 认领后
        // Place 分支被静默吞掉 ("Central Park" 既不是人也不是地点)。去重须带类型。
        if !entities
            .iter()
            .any(|e| e.name == surface && e.entity_type == EntityType::Place)
        {
            entities.push(Entity::new(surface, EntityType::Place, offset, now));
        }
    }

    entities
}

/// Returns the current Unix timestamp in seconds.
fn epoch_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_persons() {
        let text = "Alice Smith met with Bob Johnson.";
        let entities = extract_entities(text);
        let persons: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Person)
            .collect();
        assert!(!persons.is_empty(), "expected at least one person");
        let names: Vec<&str> = persons.iter().map(|e| e.name.as_str()).collect();
        assert!(
            names.contains(&"Alice Smith"),
            "expected Alice Smith in {:?}",
            names
        );
        assert!(
            names.contains(&"Bob Johnson"),
            "expected Bob Johnson in {:?}",
            names
        );
    }

    #[test]
    fn extracts_orgs() {
        let text = "She works at MIT and Google Inc.";
        let entities = extract_entities(text);
        let orgs: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Org)
            .collect();
        assert!(!orgs.is_empty());
        let names: Vec<&str> = orgs.iter().map(|e| e.name.as_str()).collect();
        assert!(
            names.iter().any(|n| n.contains("Google Inc")),
            "expected Google Inc in {:?}",
            names
        );
    }

    #[test]
    fn extracts_dates_as_events() {
        let text = "The event is on 2025-03-15.";
        let entities = extract_entities(text);
        let events: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Event)
            .collect();
        assert!(!events.is_empty(), "expected at least one event (date)");
    }

    #[test]
    fn extracts_concepts() {
        let text = "We discussed \"machine learning\" and used NLP.";
        let entities = extract_entities(text);
        let concepts: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Concept)
            .collect();
        assert!(!concepts.is_empty());
        let names: Vec<&str> = concepts.iter().map(|e| e.name.as_str()).collect();
        assert!(
            names.contains(&"machine learning"),
            "expected 'machine learning' in {:?}",
            names
        );
        assert!(names.contains(&"NLP"), "expected NLP in {:?}", names);
    }

    #[test]
    fn extracts_places() {
        let text = "The meeting is at Central Park.";
        let entities = extract_entities(text);
        let places: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Place)
            .collect();
        assert!(!places.is_empty(), "expected at least one place");
    }

    #[test]
    fn empty_text_returns_empty() {
        let entities = extract_entities("");
        assert!(entities.is_empty());
    }

    #[test]
    fn no_duplicates_from_org_vs_person() {
        let text = "Alice Smith joined Google Inc.";
        let entities = extract_entities(text);
        // Google Inc should not also appear as a person
        let google_as_person = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Person && e.name.contains("Google"))
            .count();
        assert_eq!(google_as_person, 0);
    }

    #[test]
    fn titles_extracted_as_person() {
        let text = "Dr. Alice Smith presented.";
        let entities = extract_entities(text);
        let persons: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Person)
            .collect();
        assert!(
            !persons.is_empty(),
            "expected at least one person with title"
        );
    }

    #[test]
    fn text_with_no_entities_returns_empty() {
        let text = "the and is";
        let entities = extract_entities(text);
        assert!(entities.is_empty());
    }
}

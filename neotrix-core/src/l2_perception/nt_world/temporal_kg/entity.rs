use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Organization,
    Concept,
    Event,
    Place,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub properties: HashMap<String, String>,
}

pub fn extract_entities(text: &str) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for word in text.split_whitespace() {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();

        if cleaned.is_empty() || cleaned.len() < 2 {
            continue;
        }

        let first_char = cleaned.chars().next().unwrap();
        if !first_char.is_uppercase() {
            continue;
        }

        if matches!(
            cleaned.as_str(),
            "The" | "This" | "That" | "And" | "For" | "Not" | "Use" | "Run" | "Add"
        ) {
            continue;
        }

        if seen.insert(cleaned.clone()) {
            let entity_type = classify_entity(&cleaned);
            entities.push(Entity {
                id: format!("entity_{}", cleaned.to_lowercase()),
                name: cleaned,
                entity_type,
                properties: HashMap::new(),
            });
        }
    }

    entities
}

fn classify_entity(name: &str) -> EntityType {
    let lower = name.to_lowercase();
    if lower.ends_with("inc")
        || lower.ends_with("corp")
        || lower.ends_with("llc")
        || lower.ends_with("co")
    {
        EntityType::Organization
    } else if lower.starts_with("http") || lower.contains('.') {
        EntityType::Concept
    } else {
        EntityType::Concept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let e = Entity {
            id: "e1".into(),
            name: "Alice".into(),
            entity_type: EntityType::Person,
            properties: HashMap::from([("role".into(), "engineer".into())]),
        };
        assert_eq!(e.id, "e1");
        assert_eq!(e.name, "Alice");
        assert_eq!(e.properties.get("role").unwrap(), "engineer");
    }

    #[test]
    fn test_extract_entities_basic() {
        let text = "Alice visited Google headquarters in MountainView.";
        let entities = extract_entities(text);
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Google"));
        assert!(names.contains(&"MountainView"));
    }

    #[test]
    fn test_extract_entities_dedup() {
        let text = "Alice met Alice again.";
        let entities = extract_entities(text);
        let alice_count = entities.iter().filter(|e| e.name == "Alice").count();
        assert_eq!(alice_count, 1);
    }

    #[test]
    fn test_classify_entity() {
        assert_eq!(classify_entity("AcmeInc"), EntityType::Organization);
        assert_eq!(classify_entity("RustLang"), EntityType::Concept);
    }

    #[test]
    fn test_empty_text() {
        let entities = extract_entities("");
        assert!(entities.is_empty());
    }

    #[test]
    fn test_skips_lowercase() {
        let entities = extract_entities("hello world foo bar");
        assert!(entities.is_empty());
    }
}

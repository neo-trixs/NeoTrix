#![forbid(unsafe_code)]

//! Entity types and the core `Entity` struct for entity linking.
//!
//! Entities are ADD-only (R-P117): once extracted, an entity record is never
//! mutated except by the linker which appends mention records.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Entity type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Org,
    Concept,
    Event,
    Place,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Org => "org",
            Self::Concept => "concept",
            Self::Event => "event",
            Self::Place => "place",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "person" => Some(Self::Person),
            "org" => Some(Self::Org),
            "concept" => Some(Self::Concept),
            "event" => Some(Self::Event),
            "place" => Some(Self::Place),
            _ => None,
        }
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single mention of an entity in source text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    /// The surface form as it appeared in text.
    pub surface: String,
    /// Offset within the source text (byte index).
    pub offset: usize,
    /// Timestamp when this mention was observed (Unix seconds).
    pub observed_at: i64,
}

/// A resolved entity with merged mentions and stable identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Stable unique identifier assigned by the linker.
    pub id: String,
    /// Canonical name (longest or most common mention).
    pub name: String,
    /// Classified entity type.
    pub entity_type: EntityType,
    /// All observed surface forms.
    pub mentions: Vec<Mention>,
    /// Timestamp of the first observed mention.
    pub first_seen: i64,
    /// Timestamp of the most recent mention.
    pub last_seen: i64,
}

impl Entity {
    /// Create a new entity from a single mention.
    pub fn new(
        name: impl Into<String>,
        entity_type: EntityType,
        offset: usize,
        timestamp: i64,
    ) -> Self {
        let name = name.into();
        Self {
            id: String::new(), // assigned by linker/index
            name: name.clone(),
            entity_type,
            mentions: vec![Mention {
                surface: name,
                offset,
                observed_at: timestamp,
            }],
            first_seen: timestamp,
            last_seen: timestamp,
        }
    }

    /// Add a new mention. Updates `last_seen` but preserves `first_seen`.
    pub fn add_mention(&mut self, surface: impl Into<String>, offset: usize, timestamp: i64) {
        self.mentions.push(Mention {
            surface: surface.into(),
            offset,
            observed_at: timestamp,
        });
        if timestamp > self.last_seen {
            self.last_seen = timestamp;
        }
    }

    /// Return the number of observed mentions.
    pub fn mention_count(&self) -> usize {
        self.mentions.len()
    }

    /// Return the canonical surface form (longest mention, ties broken by frequency).
    pub fn canonical_form(&self) -> &str {
        self.mentions
            .iter()
            .max_by_key(|m| m.surface.len())
            .map(|m| m.surface.as_str())
            .unwrap_or(&self.name)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({}, {} mentions)",
            &self.id,
            self.name,
            self.entity_type,
            self.mentions.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> i64 {
        1_700_000_000
    }

    #[test]
    fn entity_new_initializes_correctly() {
        let e = Entity::new("Alice", EntityType::Person, 0, ts());
        assert_eq!(e.name, "Alice");
        assert_eq!(e.entity_type, EntityType::Person);
        assert_eq!(e.mentions.len(), 1);
        assert_eq!(e.first_seen, ts());
        assert_eq!(e.last_seen, ts());
        assert!(e.id.is_empty());
    }

    #[test]
    fn add_mention_updates_last_seen() {
        let mut e = Entity::new("Acme", EntityType::Org, 0, 100);
        e.add_mention("Acme Corp", 50, 200);
        assert_eq!(e.mentions.len(), 2);
        assert_eq!(e.first_seen, 100);
        assert_eq!(e.last_seen, 200);
    }

    #[test]
    fn canonical_form_returns_longest_mention() {
        let mut e = Entity::new("MIT", EntityType::Org, 0, 100);
        e.add_mention("Massachusetts Institute of Technology", 10, 100);
        assert_eq!(e.canonical_form(), "Massachusetts Institute of Technology");
    }

    #[test]
    fn entity_type_roundtrip() {
        for et in [
            EntityType::Person,
            EntityType::Org,
            EntityType::Concept,
            EntityType::Event,
            EntityType::Place,
        ] {
            let s = et.as_str();
            assert_eq!(EntityType::from_str(s), Some(et));
        }
        assert_eq!(EntityType::from_str("unknown"), None);
    }

    #[test]
    fn display_impl() {
        let e = Entity::new("Bob", EntityType::Person, 0, ts());
        let s = e.to_string();
        assert!(s.contains("Bob"));
        assert!(s.contains("person"));
    }

    #[test]
    fn mention_count_starts_at_one() {
        let e = Entity::new("X", EntityType::Concept, 0, ts());
        assert_eq!(e.mention_count(), 1);
    }

    #[test]
    fn add_mention_preserves_first_seen() {
        let mut e = Entity::new("Y", EntityType::Place, 0, 100);
        e.add_mention("Y City", 10, 500);
        assert_eq!(e.first_seen, 100);
        assert_eq!(e.last_seen, 500);
        assert_eq!(e.mention_count(), 2);
    }

    #[test]
    fn add_mention_does_not_regress_last_seen() {
        let mut e = Entity::new("Z", EntityType::Event, 0, 300);
        e.add_mention("Z Expo", 5, 100);
        assert_eq!(e.last_seen, 300);
    }

    #[test]
    fn entity_type_from_str_case_insensitive() {
        assert_eq!(EntityType::from_str("PERSON"), Some(EntityType::Person));
        assert_eq!(EntityType::from_str("Org"), Some(EntityType::Org));
        assert_eq!(EntityType::from_str("CONCEPT"), Some(EntityType::Concept));
    }

    #[test]
    fn canonical_form_single_mention() {
        let e = Entity::new("Only", EntityType::Concept, 0, ts());
        assert_eq!(e.canonical_form(), "Only");
    }
}

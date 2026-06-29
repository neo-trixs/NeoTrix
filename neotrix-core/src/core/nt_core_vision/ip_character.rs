use std::collections::HashMap;

/// IP character spec — inspired by xiaohu-ip-studio
#[derive(Debug, Clone)]
pub struct IpCharacter {
    pub id: String,
    pub name: String,
    pub appearance: CharacterAppearance,
    pub personality: Vec<String>,
    pub archetype: Archetype,
    pub reference_prompt: String,
}

#[derive(Debug, Clone)]
pub struct CharacterAppearance {
    pub age_group: AgeGroup,
    pub hair: HairStyle,
    pub outfit_vibe: OutfitVibe,
    pub color_palette: Vec<String>,
    pub notable_features: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum AgeGroup {
    Child,
    Teen,
    YoungAdult,
    Adult,
    Elder,
}

#[derive(Debug, Clone)]
pub enum HairStyle {
    Short,
    Long,
    Ponytail,
    Bun,
    Curly,
    Bald,
    Mohawk,
}

#[derive(Debug, Clone)]
pub enum OutfitVibe {
    Casual,
    Formal,
    Techwear,
    Fantasy,
    Sporty,
    Traditional,
    Cyberpunk,
}

#[derive(Debug, Clone)]
pub enum Archetype {
    Sage,
    Hero,
    Trickster,
    Caregiver,
    Explorer,
    Rebel,
    Creator,
    Ruler,
    Innocent,
    Everyman,
}

/// Character registry — manages IP character collection
#[derive(Debug, Clone)]
pub struct CharacterRegistry {
    characters: HashMap<String, IpCharacter>,
}

impl CharacterRegistry {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
        }
    }

    pub fn register(&mut self, c: IpCharacter) {
        self.characters.insert(c.id.clone(), c);
    }

    pub fn get(&self, id: &str) -> Option<&IpCharacter> {
        self.characters.get(id)
    }

    pub fn list_by_archetype(&self, archetype: Archetype) -> Vec<&IpCharacter> {
        self.characters
            .values()
            .filter(|c| std::mem::discriminant(&c.archetype) == std::mem::discriminant(&archetype))
            .collect()
    }

    pub fn all_ids(&self) -> Vec<String> {
        self.characters.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.characters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }

    /// Seed with 6+ default characters covering different archetypes/age/appearance
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();

        reg.register(IpCharacter {
            id: "sage_li".to_string(),
            name: "Sage Li".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::Elder,
                hair: HairStyle::Bun,
                outfit_vibe: OutfitVibe::Traditional,
                color_palette: vec!["#2F4F4F".to_string(), "#8B7355".to_string(), "#FFF8DC".to_string()],
                notable_features: vec!["long beard".to_string(), "calm eyes".to_string(), "calligraphy brush".to_string()],
            },
            personality: vec!["wise".to_string(), "patient".to_string(), "contemplative".to_string()],
            archetype: Archetype::Sage,
            reference_prompt: "An elderly sage with a long white beard, wearing traditional scholar robes, holding a brush, with deep calm eyes.".to_string(),
        });

        reg.register(IpCharacter {
            id: "nova_hero".to_string(),
            name: "Nova".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::YoungAdult,
                hair: HairStyle::Long,
                outfit_vibe: OutfitVibe::Techwear,
                color_palette: vec!["#1A1A2E".to_string(), "#E94560".to_string(), "#0F3460".to_string()],
                notable_features: vec!["cybernetic arm".to_string(), "neon visor".to_string(), "energy blade".to_string()],
            },
            personality: vec!["brave".to_string(), "impulsive".to_string(), "loyal".to_string()],
            archetype: Archetype::Hero,
            reference_prompt: "A young hero with long cyberpunk-style hair, wearing high-tech armor with a neon visor and cybernetic arm.".to_string(),
        });

        reg.register(IpCharacter {
            id: "pixel_trickster".to_string(),
            name: "Pixel".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::Teen,
                hair: HairStyle::Curly,
                outfit_vibe: OutfitVibe::Casual,
                color_palette: vec!["#FF6B35".to_string(), "#FFE66D".to_string(), "#2EC4B6".to_string()],
                notable_features: vec!["mischievous grin".to_string(), "freckles".to_string(), "glowing sneakers".to_string()],
            },
            personality: vec!["playful".to_string(), "witty".to_string(), "unpredictable".to_string()],
            archetype: Archetype::Trickster,
            reference_prompt: "A teen with curly hair and freckles, wearing colorful casual clothes with glowing sneakers, grinning mischievously.".to_string(),
        });

        reg.register(IpCharacter {
            id: "aether_caregiver".to_string(),
            name: "Aether".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::Adult,
                hair: HairStyle::Long,
                outfit_vibe: OutfitVibe::Fantasy,
                color_palette: vec!["#B8E6C8".to_string(), "#7EC8E3".to_string(), "#F0E6D3".to_string()],
                notable_features: vec!["glowing hands".to_string(), "flower crown".to_string(), "flowing robes".to_string()],
            },
            personality: vec!["nurturing".to_string(), "gentle".to_string(), "selfless".to_string()],
            archetype: Archetype::Caregiver,
            reference_prompt: "An adult figure with long flowing hair and a flower crown, wearing ethereal fantasy robes with softly glowing hands.".to_string(),
        });

        reg.register(IpCharacter {
            id: "vex_rebel".to_string(),
            name: "Vex".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::YoungAdult,
                hair: HairStyle::Mohawk,
                outfit_vibe: OutfitVibe::Cyberpunk,
                color_palette: vec!["#0D0D0D".to_string(), "#FF0055".to_string(), "#00FFFF".to_string()],
                notable_features: vec!["face tattoo".to_string(), "pierced eyebrow".to_string(), "leather jacket".to_string()],
            },
            personality: vec!["defiant".to_string(), "fierce".to_string(), "independent".to_string()],
            archetype: Archetype::Rebel,
            reference_prompt: "A young rebel with a mohawk and face tattoos, wearing a worn leather jacket with cyberpunk neon accents.".to_string(),
        });

        reg.register(IpCharacter {
            id: "luna_explorer".to_string(),
            name: "Luna".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::Child,
                hair: HairStyle::Ponytail,
                outfit_vibe: OutfitVibe::Sporty,
                color_palette: vec!["#4ECDC4".to_string(), "#FF6B6B".to_string(), "#FFE66D".to_string()],
                notable_features: vec!["backpack".to_string(), "compass necklace".to_string(), "muddy boots".to_string()],
            },
            personality: vec!["curious".to_string(), "energetic".to_string(), "optimistic".to_string()],
            archetype: Archetype::Explorer,
            reference_prompt: "A child explorer with a ponytail and backpack, wearing sporty clothes and muddy boots, holding a compass.".to_string(),
        });

        reg.register(IpCharacter {
            id: "orion_creator".to_string(),
            name: "Orion".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::Adult,
                hair: HairStyle::Short,
                outfit_vibe: OutfitVibe::Casual,
                color_palette: vec!["#2C3E50".to_string(), "#E74C3C".to_string(), "#ECF0F1".to_string()],
                notable_features: vec!["paint-stained hands".to_string(), "glasses".to_string(), "tool belt".to_string()],
            },
            personality: vec!["inventive".to_string(), "perfectionist".to_string(), "passionate".to_string()],
            archetype: Archetype::Creator,
            reference_prompt: "An adult with short hair and glasses, paint-stained hands, wearing a casual apron with a tool belt.".to_string(),
        });

        reg.register(IpCharacter {
            id: "iris_innocent".to_string(),
            name: "Iris".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::Child,
                hair: HairStyle::Long,
                outfit_vibe: OutfitVibe::Fantasy,
                color_palette: vec!["#F8B4D9".to_string(), "#D4A5F5".to_string(), "#FFF0F5".to_string()],
                notable_features: vec!["big round eyes".to_string(), "ribbon".to_string(), "fairy wings".to_string()],
            },
            personality: vec!["pure-hearted".to_string(), "hopeful".to_string(), "imaginative".to_string()],
            archetype: Archetype::Innocent,
            reference_prompt: "A child with big round eyes and long hair tied with a ribbon, wearing a fantasy dress with delicate fairy wings.".to_string(),
        });

        reg
    }

    /// Generate a character description / prompt for use in image gen
    pub fn to_generation_prompt(&self, id: &str, scene: &str) -> Result<String, String> {
        let character = self
            .get(id)
            .ok_or_else(|| format!("Character '{}' not found", id))?;

        Ok(format!(
            "{}. Scene: {}. Personality: {}. Art style: consistent character illustration, IP character design, {}-themed.",
            character.reference_prompt,
            scene,
            character.personality.join(", "),
            match character.archetype {
                Archetype::Sage => "wise and contemplative",
                Archetype::Hero => "dynamic and heroic",
                Archetype::Trickster => "playful and whimsical",
                Archetype::Caregiver => "gentle and nurturing",
                Archetype::Explorer => "adventurous and curious",
                Archetype::Rebel => "edgy and defiant",
                Archetype::Creator => "artisan and inventive",
                Archetype::Ruler => "regal and commanding",
                Archetype::Innocent => "pure and dreamy",
                Archetype::Everyman => "relatable and grounded",
            },
        ))
    }
}

impl Default for CharacterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_characters_count() {
        let reg = CharacterRegistry::with_defaults();
        assert_eq!(reg.len(), 8);
    }

    #[test]
    fn test_get_character() {
        let reg = CharacterRegistry::with_defaults();
        let c = reg.get("nova_hero").unwrap();
        assert_eq!(c.name, "Nova");
        assert!(c.personality.contains(&"brave".to_string()));
    }

    #[test]
    fn test_get_missing_character() {
        let reg = CharacterRegistry::new();
        assert!(reg.get("unknown").is_none());
    }

    #[test]
    fn test_list_by_archetype() {
        let reg = CharacterRegistry::with_defaults();
        let sages = reg.list_by_archetype(Archetype::Sage);
        assert_eq!(sages.len(), 1);
        assert_eq!(sages[0].name, "Sage Li");
    }

    #[test]
    fn test_to_generation_prompt() {
        let reg = CharacterRegistry::with_defaults();
        let prompt = reg
            .to_generation_prompt("nova_hero", "fighting a dragon")
            .unwrap();
        assert!(prompt.contains("Nova"));
        assert!(prompt.contains("dragon"));
        assert!(prompt.contains("heroic"));
    }

    #[test]
    fn test_to_generation_prompt_missing() {
        let reg = CharacterRegistry::new();
        let result = reg.to_generation_prompt("ghost", "any scene");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_ids() {
        let reg = CharacterRegistry::with_defaults();
        let ids = reg.all_ids();
        assert!(ids.contains(&"sage_li".to_string()));
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn test_appearance_fields() {
        let reg = CharacterRegistry::with_defaults();
        let c = reg.get("pixel_trickster").unwrap();
        assert_eq!(c.appearance.hair, HairStyle::Curly);
        assert_eq!(c.appearance.age_group, AgeGroup::Teen);
        assert_eq!(c.appearance.outfit_vibe, OutfitVibe::Casual);
        assert!(!c.appearance.color_palette.is_empty());
    }

    #[test]
    fn test_register_custom() {
        let mut reg = CharacterRegistry::new();
        reg.register(IpCharacter {
            id: "custom".to_string(),
            name: "Custom".to_string(),
            appearance: CharacterAppearance {
                age_group: AgeGroup::Adult,
                hair: HairStyle::Bald,
                outfit_vibe: OutfitVibe::Formal,
                color_palette: vec![],
                notable_features: vec![],
            },
            personality: vec![],
            archetype: Archetype::Everyman,
            reference_prompt: "A custom character.".to_string(),
        });
        assert_eq!(reg.len(), 1);
        assert!(reg.get("custom").is_some());
    }
}

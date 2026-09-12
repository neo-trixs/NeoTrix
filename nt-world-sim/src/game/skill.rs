use crate::core::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType { Awareness, Focus, Creativity, Empathy, Logic }

#[derive(Debug, Clone)]
pub struct Skill {
    pub skill_type: SkillType,
    pub level: u32,
    pub xp: u32,
    pub xp_to_next: u32,
    pub profession: Option<Profession>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profession {
    InnerPeace,
    DeepVision,
    Precision,
    Endurance,
    Artistry,
    Innovation,
    Healing,
    Communion,
    Analysis,
    Synthesis,
}

impl Skill {
    pub fn new(skill_type: SkillType) -> Self {
        Self {
            skill_type,
            level: 1,
            xp: 0,
            xp_to_next: 100,
            profession: None,
        }
    }

    pub fn add_xp(&mut self, amount: u32) -> Vec<SkillEvent> {
        let mut events = Vec::new();
        self.xp += amount;

        while self.xp >= self.xp_to_next && self.level < 10 {
            self.xp -= self.xp_to_next;
            self.level += 1;
            self.xp_to_next = self.xp_for_level(self.level);
            events.push(SkillEvent::LevelUp { skill: self.skill_type, level: self.level });

            if self.level == 5 || self.level == 10 {
                events.push(SkillEvent::ProfessionChoice { skill: self.skill_type, level: self.level });
            }
        }

        events
    }

    fn xp_for_level(&self, level: u32) -> u32 {
        match level {
            1 => 100, 2 => 280, 3 => 500, 4 => 800, 5 => 1200,
            6 => 1700, 7 => 2300, 8 => 3000, 9 => 3800, _ => 5000,
        }
    }

    pub fn choose_profession(&mut self, profession: Profession) -> bool {
        if (self.level == 5 || self.level == 10) && self.profession.is_none() {
            self.profession = Some(profession);
            true
        } else {
            false
        }
    }

    pub fn has_profession(&self, profession: Profession) -> bool {
        self.profession == Some(profession)
    }
}

#[derive(Debug, Clone)]
pub enum SkillEvent {
    LevelUp { skill: SkillType, level: u32 },
    ProfessionChoice { skill: SkillType, level: u32 },
}

pub struct SkillSystem {
    pub skills: HashMap<SkillType, Skill>,
}

impl Resource for SkillSystem {}

impl SkillSystem {
    pub fn new() -> Self {
        let mut skills = HashMap::new();
        for st in [SkillType::Awareness, SkillType::Focus, SkillType::Creativity,
                   SkillType::Empathy, SkillType::Logic] {
            skills.insert(st, Skill::new(st));
        }
        Self { skills }
    }

    pub fn add_xp(&mut self, skill_type: SkillType, amount: u32) -> Vec<SkillEvent> {
        if let Some(skill) = self.skills.get_mut(&skill_type) {
            skill.add_xp(amount)
        } else {
            Vec::new()
        }
    }

    pub fn get_level(&self, skill_type: SkillType) -> u32 {
        self.skills.get(&skill_type).map(|s| s.level).unwrap_or(1)
    }

    pub fn get_total_level(&self) -> u32 {
        self.skills.values().map(|s| s.level).sum()
    }

    pub fn get_skill(&self, skill_type: &SkillType) -> Option<&Skill> {
        self.skills.get(skill_type)
    }

    pub fn all_skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }
}

impl Default for SkillSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new(SkillType::Awareness);
        assert_eq!(skill.level, 1);
        assert_eq!(skill.xp, 0);
        assert!(skill.profession.is_none());
    }

    #[test]
    fn test_xp_gain_no_level_up() {
        let mut skill = Skill::new(SkillType::Focus);
        let events = skill.add_xp(50);
        assert!(events.is_empty());
        assert_eq!(skill.xp, 50);
        assert_eq!(skill.level, 1);
    }

    #[test]
    fn test_level_up() {
        let mut skill = Skill::new(SkillType::Logic);
        let events = skill.add_xp(100);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SkillEvent::LevelUp { skill: SkillType::Logic, level: 2 }));
        assert_eq!(skill.level, 2);
        assert_eq!(skill.xp, 0);
    }

    #[test]
    fn test_profession_choice_at_level_5() {
        let mut skill = Skill::new(SkillType::Creativity);
        skill.add_xp(1700); // 100+280+500+800 = 1680 needed for level 5
        assert_eq!(skill.level, 5);
        assert!(skill.choose_profession(Profession::Artistry));
        assert!(skill.has_profession(Profession::Artistry));
    }

    #[test]
    fn test_profession_choice_rejected_before_level_5() {
        let mut skill = Skill::new(SkillType::Empathy);
        assert!(!skill.choose_profession(Profession::Healing));
    }

    #[test]
    fn test_skill_system() {
        let mut system = SkillSystem::new();
        let events = system.add_xp(SkillType::Awareness, 100);
        assert_eq!(events.len(), 1);
        assert_eq!(system.get_level(SkillType::Awareness), 2);
    }

    #[test]
    fn test_total_level() {
        let mut system = SkillSystem::new();
        system.add_xp(SkillType::Focus, 300);
        system.add_xp(SkillType::Logic, 200);
        let total = system.get_total_level();
        assert!(total > 5);
    }

    #[test]
    fn test_xp_carryover() {
        let mut skill = Skill::new(SkillType::Focus);
        let events = skill.add_xp(150);
        assert_eq!(skill.level, 2);
        assert_eq!(skill.xp, 50);
        assert!(!events.is_empty());
    }
}

use super::dialogue::DialogueTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcRole {
    Merchant,
    Guide,
    Companion,
    Mentor,
    Challenge,
}

#[derive(Debug, Clone)]
pub struct Npc {
    pub name: String,
    pub role: NpcRole,
    pub resonance: u32,
    pub max_resonance: u32,
    pub gifts_this_week: u32,
    pub liked_items: Vec<u32>,
    pub disliked_items: Vec<u32>,
    pub loved_items: Vec<u32>,
    pub schedule: Vec<NpcScheduleEntry>,
}

#[derive(Debug, Clone)]
pub struct NpcScheduleEntry {
    pub hour: u32,
    pub location: String,
    pub dialogue: Option<DialogueTree>,
}

impl Npc {
    pub fn new(name: &str, role: NpcRole) -> Self {
        Self {
            name: name.to_string(),
            role,
            resonance: 0,
            max_resonance: 2500,
            gifts_this_week: 0,
            liked_items: vec![],
            disliked_items: vec![],
            loved_items: vec![],
            schedule: vec![],
        }
    }

    pub fn give_gift(&mut self, item_id: u32, _base_points: u32, quality_mult: f32, is_birthday: bool) -> i32 {
        if self.gifts_this_week >= 2 {
            return 0;
        }

        let preference = if self.loved_items.contains(&item_id) {
            80
        } else if self.liked_items.contains(&item_id) {
            45
        } else if self.disliked_items.contains(&item_id) {
            -20
        } else {
            20
        };

        let mut points = (preference as f32 * quality_mult) as i32;
        if is_birthday {
            points *= 8;
        }

        self.resonance = (self.resonance as i32 + points).max(0) as u32;
        self.gifts_this_week += 1;

        points
    }

    pub fn resonance_hearts(&self) -> u32 {
        self.resonance / 250
    }

    pub fn reset_weekly_gifts(&mut self) {
        self.gifts_this_week = 0;
    }

    pub fn relationship_level(&self) -> &str {
        let hearts = self.resonance_hearts();
        match hearts {
            0 => "Stranger",
            1..=3 => "Acquaintance",
            4..=6 => "Friend",
            7..=9 => "Close Friend",
            _ => "Soul Resonant",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_gift() {
        let mut npc = Npc::new("Sage", NpcRole::Mentor);
        npc.loved_items = vec![101];

        let points = npc.give_gift(101, 80, 1.0, false);
        assert!(points > 0);
        assert_eq!(npc.gifts_this_week, 1);
    }

    #[test]
    fn test_npc_resonance() {
        let mut npc = Npc::new("Guide", NpcRole::Guide);
        npc.resonance = 500;
        assert_eq!(npc.resonance_hearts(), 2);
    }
}

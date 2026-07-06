use super::SubsystemId;

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub timestamp: i64,
    pub mood: [f64; 6],
    pub persona: [f64; 5],
    pub social: [f64; 3],
    pub reflection: [f64; 2],
    pub conversation: [f64; 2],
    pub behavioral: f64,
    pub law: f64,
}

impl SystemSnapshot {
    pub fn subsystem_vec(&self, id: SubsystemId) -> Vec<f64> {
        match id {
            SubsystemId::Mood => self.mood.to_vec(),
            SubsystemId::Persona => self.persona.to_vec(),
            SubsystemId::SocialMemory => self.social.to_vec(),
            SubsystemId::Reflection => self.reflection.to_vec(),
            SubsystemId::Conversation => self.conversation.to_vec(),
            SubsystemId::Behavioral => vec![self.behavioral],
            SubsystemId::LawKeeper => vec![self.law],
        }
    }

    pub fn flatten(&self) -> Vec<f64> {
        let mut v = Vec::with_capacity(20);
        v.extend_from_slice(&self.mood);
        v.extend_from_slice(&self.persona);
        v.extend_from_slice(&self.social);
        v.extend_from_slice(&self.reflection);
        v.extend_from_slice(&self.conversation);
        v.push(self.behavioral);
        v.push(self.law);
        v
    }
}

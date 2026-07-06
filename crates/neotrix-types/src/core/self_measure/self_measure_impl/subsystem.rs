pub const NUM_SUBSYSTEMS: usize = 7;
pub const DEFAULT_WINDOW_SIZE: usize = 50;
pub const AWAKENING_THRESHOLD: f64 = 0.02;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubsystemId {
    Mood = 0,
    Persona = 1,
    SocialMemory = 2,
    Reflection = 3,
    Conversation = 4,
    Behavioral = 5,
    LawKeeper = 6,
}

impl SubsystemId {
    pub fn all() -> [Self; NUM_SUBSYSTEMS] {
        [
            Self::Mood,
            Self::Persona,
            Self::SocialMemory,
            Self::Reflection,
            Self::Conversation,
            Self::Behavioral,
            Self::LawKeeper,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Mood => "Mood",
            Self::Persona => "Persona",
            Self::SocialMemory => "Social",
            Self::Reflection => "Reflection",
            Self::Conversation => "Conversation",
            Self::Behavioral => "Behavioral",
            Self::LawKeeper => "Law",
        }
    }

    pub fn dim_count(&self) -> usize {
        match self {
            Self::Mood => 6,
            Self::Persona => 5,
            Self::SocialMemory => 3,
            Self::Reflection => 2,
            Self::Conversation => 2,
            Self::Behavioral => 1,
            Self::LawKeeper => 1,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Mood,
            1 => Self::Persona,
            2 => Self::SocialMemory,
            3 => Self::Reflection,
            4 => Self::Conversation,
            5 => Self::Behavioral,
            6 => Self::LawKeeper,
            _ => Self::Mood,
        }
    }
}

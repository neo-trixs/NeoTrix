use std::fmt;

#[derive(Debug)]
pub enum GameError {
    Io(String),
    Serde(String),
    Game(String),
    Asset(String),
    Physics(String),
    Audio(String),
    Save(String),
    Load(String),
    InvalidState(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO: {}", e),
            Self::Serde(e) => write!(f, "Serde: {}", e),
            Self::Game(e) => write!(f, "Game: {}", e),
            Self::Asset(e) => write!(f, "Asset: {}", e),
            Self::Physics(e) => write!(f, "Physics: {}", e),
            Self::Audio(e) => write!(f, "Audio: {}", e),
            Self::Save(e) => write!(f, "Save: {}", e),
            Self::Load(e) => write!(f, "Load: {}", e),
            Self::InvalidState(e) => write!(f, "State: {}", e),
        }
    }
}

impl std::error::Error for GameError {}

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> Self { Self::Io(e.to_string()) }
}

impl From<serde_json::Error> for GameError {
    fn from(e: serde_json::Error) -> Self { Self::Serde(e.to_string()) }
}

pub type GameResult<T> = Result<T, GameError>;

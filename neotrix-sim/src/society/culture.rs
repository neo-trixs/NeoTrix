// Culture - Meme/norm propagation in agent society
// Absorbed from Project Sid's cultural transmission patterns

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meme {
    pub id: String,
    pub content: String,
    pub originator: String,
    pub creation_tick: u64,
    pub spread_count: u32,
    pub adherence: HashMap<String, f32>,  // agent_id -> adherence level
}

pub struct Culture {
    pub memes: Vec<Meme>,
    pub norms: HashMap<String, f32>,  // norm_name -> compliance_level
    pub next_meme_id: usize,
}

impl Culture {
    pub fn new() -> Self {
        Self { memes: Vec::new(), norms: HashMap::new(), next_meme_id: 0 }
    }

    pub fn create_meme(&mut self, content: &str, originator: &str, tick: u64) -> &Meme {
        self.next_meme_id += 1;
        self.memes.push(Meme {
            id: format!("meme_{}", self.next_meme_id),
            content: content.to_string(),
            originator: originator.to_string(),
            creation_tick: tick,
            spread_count: 1,
            adherence: HashMap::new(),
        });
        self.memes.last().unwrap()
    }

    pub fn spread_meme(&mut self, meme_id: &str, agent_id: &str, adherence: f32) -> bool {
        if let Some(meme) = self.memes.iter_mut().find(|m| m.id == meme_id) {
            meme.adherence.insert(agent_id.to_string(), adherence);
            meme.spread_count += 1;
            true
        } else {
            false
        }
    }

    pub fn set_norm(&mut self, norm: &str, compliance: f32) {
        self.norms.insert(norm.to_string(), compliance.clamp(0.0, 1.0));
    }

    pub fn compliance_with(&self, norm: &str) -> f32 {
        self.norms.get(norm).copied().unwrap_or(0.5)
    }

    pub fn popular_memes(&self, top_n: usize) -> Vec<&Meme> {
        let mut sorted = self.memes.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.spread_count.cmp(&a.spread_count));
        sorted.into_iter().take(top_n).collect()
    }

    pub fn agent_memes(&self, agent_id: &str) -> Vec<&Meme> {
        self.memes.iter()
            .filter(|m| m.adherence.contains_key(agent_id))
            .collect()
    }
}

impl Default for Culture {
    fn default() -> Self { Self::new() }
}

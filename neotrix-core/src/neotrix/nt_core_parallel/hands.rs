//! Hands 模块 - 7 Agent执行手

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 手类型 - 7种执行手
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandType {
    Browser,
    Terminal,
    FileSystem,
    CodeEditor,
    Database,
    API,
    Network,
}

/// 执行手
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hand {
    pub hand_type: HandType,
    pub busy: bool,
    pub task_id: Option<String>,
}

/// 七臂控制器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandsController {
    hands: HashMap<HandType, Hand>,
    max_concurrent: usize,
}

impl HandsController {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            hands: HashMap::new(),
            max_concurrent,
        }
    }

    pub fn register_hand(&mut self, hand_type: HandType) {
        self.hands.insert(
            hand_type,
            Hand {
                hand_type,
                busy: false,
                task_id: None,
            },
        );
    }

    pub fn acquire(&mut self, hand_type: HandType) -> Option<&mut Hand> {
        if self.hands.values().filter(|h| h.busy).count() >= self.max_concurrent {
            return None;
        }
        let hand = self.hands.get_mut(&hand_type)?;
        if hand.busy {
            return None;
        }
        hand.busy = true;
        Some(hand)
    }

    pub fn release(&mut self, hand_type: HandType) {
        if let Some(hand) = self.hands.get_mut(&hand_type) {
            hand.busy = false;
            hand.task_id = None;
        }
    }

    pub fn idle_hands(&self) -> Vec<HandType> {
        self.hands
            .values()
            .filter(|h| !h.busy)
            .map(|h| h.hand_type)
            .collect()
    }
}

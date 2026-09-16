#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestState {
    NotStarted,
    Active,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub enum ObjectiveType {
    Kill {
        target: String,
        count: u32,
    },
    Collect {
        item_id: u32,
        count: u32,
    },
    TalkTo {
        npc_id: u64,
    },
    ReachLocation {
        x: f64,
        y: f64,
        radius: f64,
    },
    Escort {
        npc_id: u64,
        destination: (f64, f64),
    },
}

#[derive(Debug, Clone)]
pub struct QuestObjective {
    pub description: String,
    pub objective_type: ObjectiveType,
    pub progress: u32,
    pub required: u32,
    pub completed: bool,
}

impl QuestObjective {
    pub fn kill(description: &str, target: &str, count: u32) -> Self {
        Self {
            description: description.to_string(),
            objective_type: ObjectiveType::Kill {
                target: target.to_string(),
                count,
            },
            progress: 0,
            required: count,
            completed: false,
        }
    }

    pub fn collect(description: &str, item_id: u32, count: u32) -> Self {
        Self {
            description: description.to_string(),
            objective_type: ObjectiveType::Collect { item_id, count },
            progress: 0,
            required: count,
            completed: false,
        }
    }

    pub fn talk_to(description: &str, npc_id: u64) -> Self {
        Self {
            description: description.to_string(),
            objective_type: ObjectiveType::TalkTo { npc_id },
            progress: 0,
            required: 1,
            completed: false,
        }
    }

    pub fn update_progress(&mut self, amount: u32) {
        self.progress = (self.progress + amount).min(self.required);
        self.completed = self.progress >= self.required;
    }

    pub fn is_complete(&self) -> bool {
        self.completed
    }
}

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub state: QuestState,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<QuestReward>,
    pub prerequisites: Vec<u32>,
    pub level_requirement: u32,
}

#[derive(Debug, Clone)]
pub struct QuestReward {
    pub reward_type: RewardType,
    pub amount: u32,
}

#[derive(Debug, Clone)]
pub enum RewardType {
    Experience,
    Gold,
    Item { item_id: u32 },
    Reputation { faction: String, amount: i32 },
}

impl Quest {
    pub fn new(id: u32, name: &str, description: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            state: QuestState::NotStarted,
            objectives: Vec::new(),
            rewards: Vec::new(),
            prerequisites: Vec::new(),
            level_requirement: 1,
        }
    }

    pub fn with_objective(mut self, obj: QuestObjective) -> Self {
        self.objectives.push(obj);
        self
    }

    pub fn with_reward(mut self, reward: QuestReward) -> Self {
        self.rewards.push(reward);
        self
    }

    pub fn with_prerequisite(mut self, quest_id: u32) -> Self {
        self.prerequisites.push(quest_id);
        self
    }

    pub fn with_level_requirement(mut self, level: u32) -> Self {
        self.level_requirement = level;
        self
    }

    pub fn start(&mut self) -> bool {
        if self.state == QuestState::NotStarted {
            self.state = QuestState::Active;
            true
        } else {
            false
        }
    }

    pub fn update_objective(&mut self, index: usize, amount: u32) {
        if let Some(obj) = self.objectives.get_mut(index) {
            obj.update_progress(amount);
        }
    }

    pub fn check_completion(&mut self) {
        if self.objectives.iter().all(|o| o.is_complete()) {
            self.state = QuestState::Completed;
        }
    }

    pub fn fail(&mut self) {
        self.state = QuestState::Failed;
    }

    pub fn is_complete(&self) -> bool {
        self.state == QuestState::Completed
    }

    pub fn progress_summary(&self) -> String {
        self.objectives
            .iter()
            .map(|o| format!("{}: {}/{}", o.description, o.progress, o.required))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

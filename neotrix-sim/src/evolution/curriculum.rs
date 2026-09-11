pub struct CurriculumStage {
    pub name: String,
    pub difficulty: f32,
    pub pass_threshold: f32,
}

pub struct Curriculum {
    pub stages: Vec<CurriculumStage>,
    pub current: usize,
    pub performance_history: Vec<f32>,
}

impl Curriculum {
    pub fn new() -> Self {
        let stages = vec![
            CurriculumStage {
                name: "Tutorial".into(),
                difficulty: 0.2,
                pass_threshold: 0.3,
            },
            CurriculumStage {
                name: "Easy".into(),
                difficulty: 0.4,
                pass_threshold: 0.5,
            },
            CurriculumStage {
                name: "Normal".into(),
                difficulty: 0.6,
                pass_threshold: 0.6,
            },
            CurriculumStage {
                name: "Hard".into(),
                difficulty: 0.8,
                pass_threshold: 0.7,
            },
            CurriculumStage {
                name: "Expert".into(),
                difficulty: 1.0,
                pass_threshold: 0.8,
            },
        ];
        Curriculum {
            stages,
            current: 0,
            performance_history: Vec::new(),
        }
    }

    pub fn current_difficulty(&self) -> f32 {
        self.stages
            .get(self.current)
            .map(|s| s.difficulty)
            .unwrap_or(1.0)
    }

    pub fn record_performance(&mut self, score: f32) {
        self.performance_history.push(score);
        if self.performance_history.len() >= 10 {
            let avg: f32 = self.performance_history.iter().sum::<f32>()
                / self.performance_history.len() as f32;
            let threshold = self
                .stages
                .get(self.current)
                .map(|s| s.pass_threshold)
                .unwrap_or(0.8);
            if avg >= threshold && self.current < self.stages.len() - 1 {
                self.current += 1;
            }
            self.performance_history.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_tutorial() {
        let curr = Curriculum::new();
        assert_eq!(curr.current, 0);
        assert_eq!(curr.stages.len(), 5);
    }

    #[test]
    fn current_difficulty_returns_stage_value() {
        let curr = Curriculum::new();
        assert!((curr.current_difficulty() - 0.2).abs() < 0.001);
    }

    #[test]
    fn advance_after_10_good_scores() {
        let mut curr = Curriculum::new();
        for _ in 0..10 {
            curr.record_performance(1.0);
        }
        assert_eq!(curr.current, 1);
    }

    #[test]
    fn no_advance_with_bad_scores() {
        let mut curr = Curriculum::new();
        for _ in 0..10 {
            curr.record_performance(0.0);
        }
        assert_eq!(curr.current, 0);
    }
}

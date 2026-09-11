pub struct Objective {
    pub name: String,
    pub maximize: bool,
}

pub struct ParetoPoint {
    pub agent_id: u32,
    pub scores: Vec<f32>,
}

pub struct MultiObjectiveOptimizer {
    pub objectives: Vec<Objective>,
    pub pareto_front: Vec<ParetoPoint>,
}

impl MultiObjectiveOptimizer {
    pub fn new(objectives: Vec<Objective>) -> Self {
        MultiObjectiveOptimizer {
            objectives,
            pareto_front: Vec::new(),
        }
    }

    fn dominates(&self, a: &ParetoPoint, b: &ParetoPoint) -> bool {
        a.scores
            .iter()
            .zip(b.scores.iter())
            .all(|(x, y)| x >= y)
            && a.scores
                .iter()
                .zip(b.scores.iter())
                .any(|(x, y)| x > y)
    }

    pub fn update_front(&mut self, points: Vec<ParetoPoint>) {
        let mut combined: Vec<ParetoPoint> = std::mem::take(&mut self.pareto_front);
        combined.extend(points);
        let dominated: Vec<u32> = combined.iter()
            .filter(|p| combined.iter().any(|other| other.agent_id != p.agent_id && self.dominates(other, p)))
            .map(|p| p.agent_id)
            .collect();
        combined.retain(|p| !dominated.contains(&p.agent_id));
        self.pareto_front = combined;
    }

    pub fn front_size(&self) -> usize {
        self.pareto_front.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominates_detects_strict_dominance() {
        let moo = MultiObjectiveOptimizer::new(vec![
            Objective { name: "a".into(), maximize: true },
            Objective { name: "b".into(), maximize: true },
        ]);
        let p1 = ParetoPoint { agent_id: 1, scores: vec![0.5, 0.5] };
        let p2 = ParetoPoint { agent_id: 2, scores: vec![0.8, 0.8] };
        assert!(moo.dominates(&p2, &p1));
        assert!(!moo.dominates(&p1, &p2));
    }

    #[test]
    fn update_front_removes_dominated() {
        let mut moo = MultiObjectiveOptimizer::new(vec![
            Objective { name: "a".into(), maximize: true },
        ]);
        let points = vec![
            ParetoPoint { agent_id: 1, scores: vec![0.3] },
            ParetoPoint { agent_id: 2, scores: vec![0.9] },
            ParetoPoint { agent_id: 3, scores: vec![0.6] },
        ];
        moo.update_front(points);
        assert_eq!(moo.front_size(), 1);
        assert_eq!(moo.pareto_front[0].agent_id, 2);
    }

    #[test]
    fn front_size_empty_initially() {
        let moo = MultiObjectiveOptimizer::new(vec![]);
        assert_eq!(moo.front_size(), 0);
    }
}

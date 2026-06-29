/// DGM-H: Darwin Gödel Machine Hyperagents
/// Self-referential agents where meta agent modifies task agent AND itself.
/// Reference: Meta AI 2026 — arXiv:2603.19461, ICLR 2026

/// A DGM-H agent: task logic + meta logic in one editable program
#[derive(Debug, Clone)]
pub struct HyperAgent {
    pub id: String,
    pub generation: u64,
    pub code: String,
    pub task_performance: f64,
    pub meta_performance: f64,
    pub parent_id: Option<String>,
    pub birth_cycle: u64,
    pub archive_entry: Option<ArchiveEntry>,
}

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub ancestor_id: String,
    pub lineage: Vec<String>,
    pub improvement_ratio: f64,
}

/// The DGM-H archive: stores all generated agents
#[derive(Debug)]
pub struct HyperAgentArchive {
    pub agents: Vec<HyperAgent>,
    pub max_size: usize,
}

impl HyperAgentArchive {
    pub fn new() -> Self {
        HyperAgentArchive {
            agents: Vec::with_capacity(100),
            max_size: 1000,
        }
    }

    pub fn add(&mut self, agent: HyperAgent) {
        if self.agents.len() >= self.max_size {
            if let Some(idx) = self
                .agents
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.task_performance
                        .partial_cmp(&b.task_performance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
            {
                self.agents.remove(idx);
            }
        }
        self.agents.push(agent);
    }

    pub fn top_k(&self, k: usize) -> Vec<&HyperAgent> {
        let mut sorted: Vec<_> = self.agents.iter().collect();
        sorted.sort_by(|a, b| {
            b.task_performance
                .partial_cmp(&a.task_performance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(k);
        sorted
    }

    pub fn size(&self) -> usize {
        self.agents.len()
    }
}

/// Meta agent that modifies both task and meta logic
#[derive(Debug)]
pub struct MetaAgent {
    pub prompt_template: String,
    pub edit_history: Vec<EditRecord>,
    pub performance_tracking: bool,
    pub cross_domain_transfer: bool,
}

#[derive(Debug)]
pub struct EditRecord {
    pub target: String,
    pub edit_type: EditType,
    pub before_hash: u64,
    pub after_hash: u64,
    pub performance_delta: f64,
    pub cycle: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditType {
    TaskLogic,
    MetaLogic,
    PromptTemplate,
    SearchStrategy,
    Evaluation,
}

/// DGM-H Orchestrator: runs the self-improvement loop
#[derive(Debug)]
pub struct DgmhOrchestrator {
    pub archive: HyperAgentArchive,
    pub meta: MetaAgent,
    pub cycle: u64,
    pub enabled: bool,
    pub improvement_rate: f64,
    pub generation: u64,
}

impl DgmhOrchestrator {
    pub fn new(seed_agent: HyperAgent) -> Self {
        let mut archive = HyperAgentArchive::new();
        archive.add(seed_agent);
        DgmhOrchestrator {
            archive,
            meta: MetaAgent {
                prompt_template: String::new(),
                edit_history: Vec::new(),
                performance_tracking: true,
                cross_domain_transfer: true,
            },
            cycle: 0,
            enabled: true,
            improvement_rate: 0.0,
            generation: 0,
        }
    }

    /// One iteration of the DGM-H loop
    pub fn tick(&mut self, task_performance: f64) -> Option<HyperAgent> {
        if !self.enabled {
            return None;
        }
        self.cycle += 1;

        let parent = self.get_parent()?;

        let mut child = self.mutate_agent(&parent);

        if self.cycle % 5 == 0 {
            self.meta_improve();
        }

        child.task_performance = task_performance;

        if task_performance > 0.0 {
            self.archive.add(child.clone());
            self.generation += 1;
        }

        Some(child)
    }

    fn get_parent(&self) -> Option<HyperAgent> {
        if self.archive.agents.is_empty() {
            return None;
        }
        let top = self
            .archive
            .top_k(std::cmp::max(1, self.archive.size() / 5));
        let idx = (self.cycle as usize) % top.len();
        top.get(idx).cloned().cloned()
    }

    fn mutate_agent(&self, parent: &HyperAgent) -> HyperAgent {
        HyperAgent {
            id: format!("h{}-g{}", self.generation + 1, parent.generation),
            generation: parent.generation + 1,
            code: parent.code.clone(),
            task_performance: 0.0,
            meta_performance: parent.meta_performance,
            parent_id: Some(parent.id.clone()),
            birth_cycle: self.cycle,
            archive_entry: Some(ArchiveEntry {
                ancestor_id: parent.id.clone(),
                lineage: vec![parent.id.clone()],
                improvement_ratio: 0.0,
            }),
        }
    }

    fn meta_improve(&mut self) {
        if self.cycle % 10 == 0 {
            self.meta.performance_tracking = true;
        }
        if self.cycle % 20 == 0 {
            self.meta.cross_domain_transfer = true;
        }
    }

    pub fn improvement_rate(&self) -> f64 {
        if self.archive.agents.len() < 2 {
            return 0.0;
        }
        let recent: Vec<_> = self.archive.top_k(10);
        if recent.len() < 2 {
            return 0.0;
        }
        (recent[0].task_performance - recent.last().unwrap().task_performance).abs()
    }

    pub fn generation_count(&self) -> u64 {
        self.generation
    }

    pub fn report(&self) -> String {
        format!(
            "DGM-H | Cycle={} Gen={} Archive={} TopPerf={:.3} ImprRate={:.3}",
            self.cycle,
            self.generation,
            self.archive.size(),
            self.archive
                .top_k(1)
                .first()
                .map(|a| a.task_performance)
                .unwrap_or(0.0),
            self.improvement_rate(),
        )
    }
}

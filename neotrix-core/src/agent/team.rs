use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::core::CapabilityVector;
use crate::core::EngineProvider;
use crate::agent::persona::AgentPersona;

#[derive(Debug, Clone)]
pub struct AgentRole {
    pub name: String,
    pub role: String,
    pub goal: String,
    pub backstory: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub agent_name: String,
    pub output: String,
    pub success: bool,
    pub capability_delta: Option<CapabilityVector>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessType {
    Sequential,
    Hierarchical,
    Debate,
}

/// Swarm 执行模式 — HashCortX 4 种 swarm 算法
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwarmMode {
    /// Boss分解 → Worker并行 → Boss综合
    BossTeam,
    /// 所有 Agent 并行投票 → Judge 选最优/合并
    AllVote,
    /// A写 → B改进 → C润色 → D最终
    ChainRefine,
    /// Proposer → Challenger → Resolver 三方辩论
    DevilsAdvocate,
}

pub struct AgentTeam {
    pub name: String,
    pub agents: Vec<AgentRole>,
    pub process: ProcessType,
    pub swarm_mode: Option<SwarmMode>,
    pub results: Vec<AgentResult>,
    pub persona: Option<AgentPersona>,
    engine: Option<Arc<Mutex<dyn EngineProvider>>>,
}

impl AgentTeam {
    pub fn new(name: &str, process: ProcessType) -> Self {
        Self {
            name: name.to_string(),
            agents: Vec::new(),
            process,
            swarm_mode: None,
            results: Vec::new(),
            persona: None,
            engine: None,
        }
    }

    pub fn with_swarm(mut self, mode: SwarmMode) -> Self {
        self.swarm_mode = Some(mode);
        self
    }

    pub fn with_engine(mut self, engine: Arc<Mutex<dyn EngineProvider>>) -> Self {
        self.engine = Some(engine);
        self
    }

    pub fn with_persona(mut self, persona: AgentPersona) -> Self {
        self.persona = Some(persona);
        self
    }

    pub fn add_agent(&mut self, agent: AgentRole) {
        self.agents.push(agent);
    }

    fn reason(&self, prompt: &str) -> String {
        if let Some(ref engine) = self.engine {
            match engine.lock() {
                Ok(mut eng) => eng.reason(prompt).unwrap_or_else(|e| format!("[Engine error: {}]", e)),
                Err(e) => format!("[Lock error: {}]", e),
            }
        } else {
            format!("[No engine] {}", prompt.chars().take(120).collect::<String>())
        }
    }

    /// Boss分解 → Worker并行 → Boss综合
    fn run_swarm_boss_team(&mut self, task: &str) -> Vec<AgentResult> {
        if self.agents.is_empty() {
            return Vec::new();
        }
        let mut results = Vec::new();
        let boss = &self.agents[0];
        let workers: Vec<&AgentRole> = self.agents[1..].iter().collect();

        // Boss 分解
        let decompose_prompt = format!(
            "[Boss {}]\n角色: 团队领导\n任务: {}\n\n请将任务分解为 {} 个子任务，每个子任务分配给一个队员。",
            boss.name, task, workers.len()
        );
        let plan = self.reason(&decompose_prompt);
        results.push(AgentResult {
            agent_name: format!("{} (Boss)", boss.name),
            output: format!("【分解计划】\n{}", plan),
            success: true,
            capability_delta: None,
        });

        // Workers 并行执行
        for worker in workers.iter() {
            let work_prompt = format!(
                "[Worker {}]\n角色: {}\n目标: {}\n领导计划: {}\n\n请执行你分配到的子任务。",
                worker.name, worker.role, worker.goal, plan
            );
            let output = self.reason(&work_prompt);
            results.push(AgentResult {
                agent_name: format!("{} (Worker)", worker.name),
                output,
                success: true,
                capability_delta: None,
            });
        }

        // Boss 综合
        let worker_outputs: Vec<String> = results[1..]
            .iter()
            .map(|r| format!("[{}]: {}", r.agent_name, r.output))
            .collect();
        let synthesize_prompt = format!(
            "[Boss {} 综合]\n任务: {}\n\nWorker 执行结果:\n{}\n\n请综合所有结果给出最终输出。",
            boss.name, task, worker_outputs.join("\n\n")
        );
        let final_output = self.reason(&synthesize_prompt);
        results.push(AgentResult {
            agent_name: format!("{} (Boss)", boss.name),
            output: format!("【最终输出】\n{}", final_output),
            success: true,
            capability_delta: None,
        });

        self.results = results.clone();
        results
    }

    /// 所有 Agent 并行投票 → Judge 选最优/合并
    fn run_swarm_all_vote(&mut self, task: &str) -> Vec<AgentResult> {
        if self.agents.len() < 2 {
            return self.run_sequential(task);
        }
        let mut results = Vec::new();

        for agent in &self.agents {
            let prompt = format!(
                "[Judge 投票]\n角色: {}\n请你从你的专业角度分析以下问题并给出答案:\n\n{}",
                agent.role, task
            );
            let output = self.reason(&prompt);
            results.push(AgentResult {
                agent_name: format!("{} (Voter)", agent.name),
                output,
                success: true,
                capability_delta: None,
            });
        }

        // Judge 选最优/合并
        let votes: Vec<String> = results
            .iter()
            .map(|r| format!("[{}]: {}", r.agent_name, r.output))
            .collect();
        let judge_prompt = format!(
            "[Judge]\n任务: {}\n\n收到的投票:\n{}\n\n请分析所有答案，选择最佳答案或合并多个答案给出最终输出。",
            task, votes.join("\n\n")
        );
        let verdict = self.reason(&judge_prompt);
        results.push(AgentResult {
            agent_name: "Judge".to_string(),
            output: format!("【裁决】\n{}", verdict),
            success: true,
            capability_delta: None,
        });

        self.results = results.clone();
        results
    }

    /// 串行改进链: A写 → B改进 → C润色 → D最终
    fn run_swarm_chain_refine(&mut self, task: &str) -> Vec<AgentResult> {
        if self.agents.is_empty() {
            return Vec::new();
        }
        let mut results = Vec::new();
        let mut current = task.to_string();

        for (i, agent) in self.agents.iter().enumerate() {
            let instruction = if i == 0 {
                format!("请从头开始完成任务: {}", task)
            } else {
                "基于前一步的输出进行改进和优化。提高质量、准确性和清晰度。".to_string()
            };
            let prompt = format!(
                "[Chain Step {}/{} — {}]\n角色: {}\n\nInput:\n{}\n\n{}",
                i + 1,
                self.agents.len(),
                agent.name,
                agent.role,
                current,
                instruction,
            );
            let output = self.reason(&prompt);
            current = output.clone();
            results.push(AgentResult {
                agent_name: format!("{} (Step {})", agent.name, i + 1),
                output,
                success: true,
                capability_delta: None,
            });
        }

        self.results = results.clone();
        results
    }

    /// Proposer → Challenger → Resolver 三方辩论
    fn run_swarm_devils_advocate(&mut self, task: &str) -> Vec<AgentResult> {
        let mut results = Vec::new();
        let _agents_needed = 3;
        let available = self.agents.len();

        let proposer = &self.agents[0];
        let challenger = if available > 1 { &self.agents[1] } else { &self.agents[0] };
        let resolver = if available > 2 { &self.agents[2] } else { &self.agents[0] };

        // Proposer
        let prop_prompt = format!(
            "[Proposer {}]\n角色: {}\n问题: {}\n\n请提出一个深思熟虑的提案或解决方案。给出具体论据。",
            proposer.name, proposer.role, task
        );
        let proposal = self.reason(&prop_prompt);
        results.push(AgentResult {
            agent_name: format!("{} (Proposer)", proposer.name),
            output: format!("【提案】\n{}", proposal),
            success: true,
            capability_delta: None,
        });

        // Challenger
        let chal_prompt = format!(
            "[Challenger {}]\n角色: {} — 批判性思维者\n\n原始提案:\n{}\n\n请严格审查上述提案。指出:\n1. 逻辑漏洞\n2. 假设质疑\n3. 遗漏的替代方案\n4. 潜在风险\n\n给出有建设性的批评。",
            challenger.name, challenger.role, proposal
        );
        let challenge = self.reason(&chal_prompt);
        results.push(AgentResult {
            agent_name: format!("{} (Challenger)", challenger.name),
            output: format!("【质疑】\n{}", challenge),
            success: true,
            capability_delta: None,
        });

        // Resolver
        let res_prompt = format!(
            "[Resolver {}]\n角色: {} — 仲裁者\n\n原始问题: {}\n\n提案:\n{}\n\n质疑:\n{}\n\n请作为仲裁者:\n1. 识别提案和质疑中真正有价值的部分\n2. 解决矛盾\n3. 给出综合的最终方案",
            resolver.name, resolver.role, task, proposal, challenge
        );
        let resolution = self.reason(&res_prompt);
        results.push(AgentResult {
            agent_name: format!("{} (Resolver)", resolver.name),
            output: format!("【仲裁】\n{}", resolution),
            success: true,
            capability_delta: None,
        });

        self.results = results.clone();
        results
    }

    pub fn run_sequential(&mut self, task: &str) -> Vec<AgentResult> {
        let mut context = task.to_string();
        let mut results = Vec::new();
        for agent in &self.agents {
            let prompt = format!(
                "[{}]\n角色: {}\n目标: {}\n故事: {}\n工具: {:?}\n\n上下文:\n{}\n\n请基于你的角色和目标执行任务。",
                agent.name, agent.role, agent.goal, agent.backstory, agent.tools, context
            );
            let output = self.reason(&prompt);
            let result = AgentResult {
                agent_name: agent.name.clone(),
                output,
                success: true,
                capability_delta: None,
            };
            context = format!("{}\n\n[{} 输出]: {}", context, agent.name, result.output);
            results.push(result);
        }
        self.results = results.clone();
        results
    }

    pub fn run_hierarchical(&mut self, task: &str) -> Vec<AgentResult> {
        if self.agents.is_empty() {
            return Vec::new();
        }
        let manager = &self.agents[0];
        let workers: Vec<&AgentRole> = self.agents[1..].iter().collect();
        let mut results = Vec::new();

        let plan_prompt = format!(
            "[Manager {}]\n你的角色: Manager\n目标: 将任务分解为子任务分配给 {} 个 Worker\n任务: {}",
            manager.name, workers.len(), task
        );
        let plan = self.reason(&plan_prompt);
        results.push(AgentResult {
            agent_name: manager.name.clone(),
            output: plan.clone(),
            success: true,
            capability_delta: None,
        });

        for worker in &workers {
            let work_prompt = format!(
                "[{}]\n角色: {}\n目标: {}\nManager 计划:\n{}\n\n请执行分配给你的子任务。",
                worker.name, worker.role, worker.goal, plan
            );
            let output = self.reason(&work_prompt);
            results.push(AgentResult {
                agent_name: worker.name.clone(),
                output,
                success: true,
                capability_delta: None,
            });
        }

        let summary_prompt = format!(
            "[Manager {} 汇总]\n任务: {}\n\nWorker 执行结果:\n{}",
            manager.name, task,
            results[1..].iter().map(|r| format!("[{}]: {}", r.agent_name, r.output)).collect::<Vec<_>>().join("\n")
        );
        let summary = self.reason(&summary_prompt);
        results.push(AgentResult {
            agent_name: manager.name.clone(),
            output: summary,
            success: true,
            capability_delta: None,
        });
        self.results = results.clone();
        results
    }

    pub fn run_debate(&mut self, task: &str) -> Vec<AgentResult> {
        let mut results = Vec::new();
        for (i, agent) in self.agents.iter().enumerate() {
            let stance = if i % 2 == 0 { "赞成" } else { "反对" };
            let prompt = format!(
                "[{}]\n角色: {}\n立场: {}\n\n请你从{}方立场辩论: {}\n请给出具体论点和论据。",
                agent.name, agent.role, stance, stance, task
            );
            let output = self.reason(&prompt);
            results.push(AgentResult {
                agent_name: agent.name.clone(),
                output,
                success: true,
                capability_delta: None,
            });
        }

        let all_args: Vec<String> = results.iter().map(|r| format!("[{}]: {}", r.agent_name, r.output)).collect();
        let review_prompt = format!(
            "[Meta-Reviewer]\n任务: {}\n\n各方观点:\n{}\n\n请作为中立的元评审，综合各方观点给出最终裁决和建议。",
            task, all_args.join("\n\n")
        );
        let verdict = self.reason(&review_prompt);
        results.push(AgentResult {
            agent_name: "Meta-Reviewer".to_string(),
            output: verdict,
            success: true,
            capability_delta: None,
        });
        self.results = results.clone();
        results
    }

    pub fn execute(&mut self, task: &str) -> Vec<AgentResult> {
        if self.agents.is_empty() {
            self.agents.push(AgentRole {
                name: "default".into(),
                role: "General Assistant".into(),
                goal: "Complete the assigned task autonomously".into(),
                backstory: "A versatile AI agent capable of handling various tasks".into(),
                tools: vec!["reason".into()],
            });
        }
        // Swarm 模式优先
        if let Some(mode) = self.swarm_mode {
            return match mode {
                SwarmMode::BossTeam => self.run_swarm_boss_team(task),
                SwarmMode::AllVote => self.run_swarm_all_vote(task),
                SwarmMode::ChainRefine => self.run_swarm_chain_refine(task),
                SwarmMode::DevilsAdvocate => self.run_swarm_devils_advocate(task),
            };
        }
        match self.process {
            ProcessType::Sequential => self.run_sequential(task),
            ProcessType::Hierarchical => self.run_hierarchical(task),
            ProcessType::Debate => self.run_debate(task),
        }
    }

    pub fn summary(&self) -> String {
        let n = self.results.len();
        if n == 0 {
            return "未执行".to_string();
        }
        let successes = self.results.iter().filter(|r| r.success).count();
        format!(
            "AgentTeam '{}': {} agents, {:?} Process, {}/{} 成功",
            self.name, self.agents.len(), self.process, successes, n
        )
    }
}

pub struct Coordinator {
    pub name: String,
    engine: Option<Arc<Mutex<dyn EngineProvider>>>,
    pub routing_table: HashMap<String, Vec<String>>,
}

impl Coordinator {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            engine: None,
            routing_table: HashMap::new(),
        }
    }

    pub fn with_engine(mut self, engine: Arc<Mutex<dyn EngineProvider>>) -> Self {
        self.engine = Some(engine);
        self
    }

    pub fn register_route(&mut self, task_type: &str, agent_names: &[&str]) {
        self.routing_table.insert(
            task_type.to_string(),
            agent_names.iter().map(|s| s.to_string()).collect(),
        );
    }

    pub fn route_task(&self, task: &str) -> Vec<String> {
        for (key, agents) in &self.routing_table {
            if task.to_lowercase().contains(&key.to_lowercase()) {
                return agents.clone();
            }
        }
        Vec::new()
    }

    pub fn resolve_conflict(&self, outputs: &[AgentResult], task: &str) -> AgentResult {
        if outputs.is_empty() {
            return AgentResult {
                agent_name: self.name.clone(),
                output: "No outputs to resolve".to_string(),
                success: false,
                capability_delta: None,
            };
        }
        if outputs.len() == 1 {
            return outputs[0].clone();
        }

        let all = outputs.iter().map(|r| format!("[{}]: {}", r.agent_name, r.output)).collect::<Vec<_>>().join("\n\n");
        if let Some(ref engine) = self.engine {
            let prompt = format!(
                "[Coordinator {} 冲突解决]\n任务: {}\n\n冲突输出:\n{}\n\n请分析差异点并给出统一的解决方案。",
                self.name, task, all
            );
            if let Ok(mut eng) = engine.lock() {
                let resolution = eng.reason(&prompt).unwrap_or_else(|_| "Resolution failed".to_string());
                return AgentResult {
                    agent_name: self.name.clone(),
                    output: resolution,
                    success: true,
                    capability_delta: None,
                };
            }
        }
        AgentResult {
            agent_name: self.name.clone(),
            output: format!("Merged {} outputs:\n{}", outputs.len(), all),
            success: true,
            capability_delta: None,
        }
    }

    pub fn merge_results(&self, results: &[AgentResult], task: &str) -> AgentResult {
        if results.is_empty() {
            return AgentResult {
                agent_name: self.name.clone(),
                output: "No results to merge".to_string(),
                success: false,
                capability_delta: None,
            };
        }
        let all = results.iter().map(|r| format!("[{}]: {}", r.agent_name, r.output)).collect::<Vec<_>>().join("\n\n");
        if let Some(ref engine) = self.engine {
            let prompt = format!(
                "[Coordinator {} 结果合并]\n任务: {}\n\nAgent 结果:\n{}\n\n请将以上结果合并为一个连贯的最终输出。",
                self.name, task, all
            );
            if let Ok(mut eng) = engine.lock() {
                let merged = eng.reason(&prompt).unwrap_or_else(|_| "Merge failed".to_string());
                return AgentResult {
                    agent_name: self.name.clone(),
                    output: merged,
                    success: true,
                    capability_delta: None,
                };
            }
        }
        AgentResult {
            agent_name: self.name.clone(),
            output: format!("Merged from {} agents:\n{}", results.len(), all),
            success: true,
            capability_delta: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(name: &str, role: &str) -> AgentRole {
        AgentRole {
            name: name.to_string(),
            role: role.to_string(),
            goal: format!("作为{}执行任务", role),
            backstory: format!("资深{}专家", role),
            tools: Vec::new(),
        }
    }

    #[test]
    fn test_sequential_execution() {
        let mut team = AgentTeam::new("test", ProcessType::Sequential);
        team.add_agent(make_agent("研究员", "Research Analyst"));
        team.add_agent(make_agent("写手", "Content Writer"));
        let results = team.run_sequential("AI 发展趋势");
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn test_hierarchical_execution() {
        let mut team = AgentTeam::new("test", ProcessType::Hierarchical);
        team.add_agent(make_agent("经理", "Manager"));
        team.add_agent(make_agent("工程师1", "Engineer"));
        team.add_agent(make_agent("工程师2", "Engineer"));
        let results = team.run_hierarchical("开发新功能");
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_debate_execution() {
        let mut team = AgentTeam::new("test", ProcessType::Debate);
        team.add_agent(make_agent("正方", "Proponent"));
        team.add_agent(make_agent("反方", "Opponent"));
        let results = team.run_debate("应该采用 Rust");
        assert_eq!(results.len(), 3);
        assert!(results.last().expect("value should be ok in test").agent_name.contains("Meta"));
    }

    #[test]
    fn test_execute_dispatches_correctly() {
        let mut team = AgentTeam::new("seq", ProcessType::Sequential);
        team.add_agent(make_agent("A", "Role"));
        let r = team.execute("task");
        assert_eq!(r.len(), 1);

        let mut team2 = AgentTeam::new("deb", ProcessType::Debate);
        team2.add_agent(make_agent("X", "Role"));
        team2.add_agent(make_agent("Y", "Role"));
        let r2 = team2.execute("task");
        assert_eq!(r2.len(), 3);
    }

    #[test]
    fn test_summary() {
        let mut team = AgentTeam::new("test-team", ProcessType::Sequential);
        team.add_agent(make_agent("A", "Role"));
        let s = team.summary();
        assert!(s.contains("未执行"));
        team.execute("task");
        let s = team.summary();
        assert!(s.contains("test-team"));
        assert!(s.contains("1/1 成功"));
    }

    #[test]
    fn test_empty_team_hierarchical() {
        let mut team = AgentTeam::new("empty", ProcessType::Hierarchical);
        let results = team.run_hierarchical("task");
        assert!(results.is_empty());
    }

    #[test]
    fn test_swarm_boss_team() {
        let mut team = AgentTeam::new("boss", ProcessType::Sequential);
        team.swarm_mode = Some(SwarmMode::BossTeam);
        team.add_agent(make_agent("经理", "Manager"));
        team.add_agent(make_agent("工程师", "Engineer"));
        team.add_agent(make_agent("设计师", "Designer"));
        let results = team.run_swarm_boss_team("开发登录页面");
        // Boss decompose + 2 workers + Boss synthesize
        assert_eq!(results.len(), 4);
        assert!(results[0].agent_name.contains("Boss"));
        assert!(results[1].agent_name.contains("Worker"));
        assert!(results[2].agent_name.contains("Worker"));
        assert!(results[3].agent_name.contains("Boss"));
        assert!(results[3].output.contains("最终输出"));
    }

    #[test]
    fn test_swarm_all_vote() {
        let mut team = AgentTeam::new("vote", ProcessType::Sequential);
        team.swarm_mode = Some(SwarmMode::AllVote);
        team.add_agent(make_agent("A", "Analyst"));
        team.add_agent(make_agent("B", "Analyst"));
        let results = team.run_swarm_all_vote("最佳编程语言");
        assert_eq!(results.len(), 3);
        assert!(results[0].agent_name.contains("Voter"));
        assert!(results[1].agent_name.contains("Voter"));
        assert_eq!(results[2].agent_name, "Judge");
    }

    #[test]
    fn test_swarm_all_vote_single_agent() {
        let mut team = AgentTeam::new("vote", ProcessType::Sequential);
        team.swarm_mode = Some(SwarmMode::AllVote);
        team.add_agent(make_agent("A", "Analyst"));
        let results = team.run_swarm_all_vote("task");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_swarm_chain_refine() {
        let mut team = AgentTeam::new("chain", ProcessType::Sequential);
        team.swarm_mode = Some(SwarmMode::ChainRefine);
        team.add_agent(make_agent("Writer", "Writer"));
        team.add_agent(make_agent("Editor", "Editor"));
        team.add_agent(make_agent("Reviewer", "Reviewer"));
        let results = team.run_swarm_chain_refine("写一篇关于Rust的文章");
        assert_eq!(results.len(), 3);
        assert!(results[0].agent_name.contains("Step 1"));
        assert!(results[1].agent_name.contains("Step 2"));
        assert!(results[2].agent_name.contains("Step 3"));
    }

    #[test]
    fn test_swarm_devils_advocate() {
        let mut team = AgentTeam::new("debate", ProcessType::Sequential);
        team.swarm_mode = Some(SwarmMode::DevilsAdvocate);
        team.add_agent(make_agent("Proposer", "Proposer"));
        team.add_agent(make_agent("Critic", "Critic"));
        team.add_agent(make_agent("Mediator", "Mediator"));
        let results = team.run_swarm_devils_advocate("应该采用微服务架构");
        assert_eq!(results.len(), 3);
        assert!(results[0].agent_name.contains("Proposer"));
        assert!(results[1].agent_name.contains("Challenger"));
        assert!(results[2].agent_name.contains("Resolver"));
    }

    #[test]
    fn test_swarm_devils_advocate_single_agent_fallback() {
        let mut team = AgentTeam::new("debate", ProcessType::Sequential);
        team.swarm_mode = Some(SwarmMode::DevilsAdvocate);
        team.add_agent(make_agent("Only", "General"));
        let results = team.run_swarm_devils_advocate("task");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_swarm_boss_team_empty() {
        let mut team = AgentTeam::new("empty", ProcessType::Sequential);
        team.swarm_mode = Some(SwarmMode::BossTeam);
        let results = team.run_swarm_boss_team("task");
        assert!(results.is_empty());
    }

    #[test]
    fn test_execute_dispatches_swarm() {
        let mut team = AgentTeam::new("swarm", ProcessType::Sequential);
        team.swarm_mode = Some(SwarmMode::BossTeam);
        team.add_agent(make_agent("Boss", "Manager"));
        team.add_agent(make_agent("Worker", "Worker"));
        let results = team.execute("task");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_coordinator_route() {
        let mut c = Coordinator::new("main");
        c.register_route("design", &["设计师", "UI 专家"]);
        c.register_route("code", &["工程师"]);
        let agents = c.route_task("design a login page");
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"设计师".to_string()));
    }

    #[test]
    fn test_coordinator_merge_empty() {
        let c = Coordinator::new("main");
        let result = c.merge_results(&[], "task");
        assert!(!result.success);
    }

    #[test]
    fn test_coordinator_merge_single_result() {
        let c = Coordinator::new("test");
        let results = vec![AgentResult {
            agent_name: "A".into(),
            output: "single output".into(),
            success: true,
            capability_delta: None,
        }];
        let merged = c.merge_results(&results, "task");
        assert!(merged.success);
        assert!(merged.output.contains("single output"));
    }

    #[test]
    fn test_coordinator_resolve_no_conflict() {
        let c = Coordinator::new("test");
        let results = vec![
            AgentResult { agent_name: "A".into(), output: "agree".into(), success: true, capability_delta: None },
            AgentResult { agent_name: "B".into(), output: "agree".into(), success: true, capability_delta: None },
        ];
        let resolved = c.resolve_conflict(&results, "task");
        assert!(resolved.success);
    }

    #[test]
    fn test_coordinator_route_no_match() {
        let mut c = Coordinator::new("test");
        c.register_route("design", &["designer"]);
        let agents = c.route_task("coding task");
        assert!(agents.is_empty());
    }

    #[test]
    fn test_coordinator_multiple_routes() {
        let mut c = Coordinator::new("test");
        c.register_route("design", &["designer"]);
        c.register_route("code", &["engineer"]);
        assert_eq!(c.route_task("design task").len(), 1);
        assert_eq!(c.route_task("code task").len(), 1);
    }
}

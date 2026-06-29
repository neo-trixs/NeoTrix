use crate::core::nt_core_util;
use crate::core::CapabilityVector;
use crate::core::TaskType;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonaRole {
    Engineer,
    Designer,
    Reviewer,
    Architect,
    Researcher,
    QA,
    DevOps,
    ProductManager,
    SecurityAnalyst,
    DataScientist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Junior,
    Mid,
    Senior,
    Lead,
    Principal,
}

impl ExperienceLevel {
    pub fn multiplier(&self) -> f64 {
        match self {
            ExperienceLevel::Junior => 0.5,
            ExperienceLevel::Mid => 0.75,
            ExperienceLevel::Senior => 1.0,
            ExperienceLevel::Lead => 1.25,
            ExperienceLevel::Principal => 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersona {
    pub name: String,
    pub role: PersonaRole,
    pub specialties: Vec<String>,
    pub experience_level: ExperienceLevel,
    pub traits: HashMap<String, f64>,
    pub capability_bias: CapabilityVector,
    pub system_prompt_template: String,
}

impl AgentPersona {
    pub fn new(
        name: &str,
        role: PersonaRole,
        specialties: Vec<String>,
        experience_level: ExperienceLevel,
        traits: HashMap<String, f64>,
        capability_bias: CapabilityVector,
        system_prompt_template: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            role,
            specialties,
            experience_level,
            traits,
            capability_bias,
            system_prompt_template: system_prompt_template.to_string(),
        }
    }

    pub fn apply_bias(&self, target: &mut CapabilityVector) {
        let lr = self.experience_level.multiplier();
        for i in 0..target.arr.len() {
            let src = self.capability_bias.arr.get(i).copied().unwrap_or(0.0);
            target.arr[i] += lr * (src - target.arr[i]);
        }
        for (name, val) in &self.capability_bias.extension {
            target.add_extension_dim(name, *val);
        }
    }
}

pub struct AgentPersonaRegistry {
    personas: HashMap<String, AgentPersona>,
    persistence_path: PathBuf,
}

impl Default for AgentPersonaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentPersonaRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            personas: HashMap::new(),
            persistence_path: dirs_or_default(),
        };
        reg.seed_builtin();
        reg
    }

    pub fn with_path(path: PathBuf) -> Self {
        let mut reg = Self {
            personas: HashMap::new(),
            persistence_path: path,
        };
        reg.seed_builtin();
        reg
    }

    pub fn register(&mut self, persona: AgentPersona) {
        self.personas.insert(persona.name.clone(), persona);
    }

    pub fn get(&self, name: &str) -> Option<&AgentPersona> {
        self.personas.get(name)
    }

    pub fn random(&self) -> Option<&AgentPersona> {
        if self.personas.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.personas.len());
        self.personas.values().nth(idx)
    }

    pub fn by_role(&self, role: PersonaRole) -> Vec<&AgentPersona> {
        self.personas.values().filter(|p| p.role == role).collect()
    }

    pub fn from_task_type(&self, task_type: TaskType) -> Vec<&AgentPersona> {
        let role_keywords: HashMap<TaskType, Vec<PersonaRole>> = [
            (
                TaskType::General,
                vec![
                    PersonaRole::Engineer,
                    PersonaRole::Architect,
                    PersonaRole::Researcher,
                ],
            ),
            (
                TaskType::Design,
                vec![PersonaRole::Designer, PersonaRole::ProductManager],
            ),
            (
                TaskType::CodeAnalysis,
                vec![
                    PersonaRole::Engineer,
                    PersonaRole::Reviewer,
                    PersonaRole::Architect,
                ],
            ),
            (
                TaskType::CodeGeneration,
                vec![PersonaRole::Engineer, PersonaRole::DataScientist],
            ),
            (
                TaskType::CodeReview,
                vec![
                    PersonaRole::Reviewer,
                    PersonaRole::Engineer,
                    PersonaRole::QA,
                ],
            ),
            (
                TaskType::Security,
                vec![PersonaRole::SecurityAnalyst, PersonaRole::Engineer],
            ),
            (
                TaskType::Planning,
                vec![
                    PersonaRole::Architect,
                    PersonaRole::ProductManager,
                    PersonaRole::DevOps,
                ],
            ),
            (
                TaskType::Reflection,
                vec![
                    PersonaRole::Researcher,
                    PersonaRole::Reviewer,
                    PersonaRole::Architect,
                ],
            ),
            (
                TaskType::UIDesign,
                vec![PersonaRole::Designer, PersonaRole::ProductManager],
            ),
            (
                TaskType::Research,
                vec![PersonaRole::Researcher, PersonaRole::DataScientist],
            ),
            (
                TaskType::Learning,
                vec![PersonaRole::Researcher, PersonaRole::Engineer],
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let roles = role_keywords.get(&task_type);
        match roles {
            Some(r) => self
                .personas
                .values()
                .filter(|p| r.contains(&p.role))
                .collect(),
            None => self.random().map_or(vec![], |p| vec![p]),
        }
    }

    pub fn all(&self) -> Vec<&AgentPersona> {
        self.personas.values().collect()
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.persistence_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(&self.personas)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&self.persistence_path, json).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        if !self.persistence_path.exists() {
            return Err("Persistence file not found".to_string());
        }
        let json =
            fs::read_to_string(&self.persistence_path).map_err(|e| format!("Read error: {}", e))?;
        let loaded: HashMap<String, AgentPersona> =
            serde_json::from_str(&json).map_err(|e| format!("Deserialization error: {}", e))?;
        self.personas.extend(loaded);
        Ok(())
    }

    fn seed_builtin(&mut self) {
        let builtin = builtin_personas();
        for p in builtin {
            self.personas.insert(p.name.clone(), p);
        }
    }
}

fn dirs_or_default() -> PathBuf {
    let home = nt_core_util::home_dir().to_string_lossy().to_string();
    PathBuf::from(home).join(".neotrix/personas.json")
}

fn traits_map(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn builtin_personas() -> Vec<AgentPersona> {
    vec![
        AgentPersona {
            name: "Senior Rust Engineer".into(),
            role: PersonaRole::Engineer,
            specialties: vec!["async".into(), "systems programming".into(), "concurrency".into(), "unsafe rust".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("precision", 0.95), ("thoroughness", 0.9), ("pragmatism", 0.85), ("ownership", 0.88)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.2, 0.1, 0.3,
                0.2, 0.1, 0.4, 0.2,
                0.9, 0.3, 0.95, 0.8,
                0.85,
                0.2, 0.3, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.3, 0.85,
                0.9,
            ),
            system_prompt_template: "You are a Senior Rust Engineer with deep expertise in systems programming, async runtimes, and zero-cost abstractions. You write safe, idiomatic, performant Rust code.".into(),
        },
        AgentPersona {
            name: "Frontend Designer".into(),
            role: PersonaRole::Designer,
            specialties: vec!["react".into(), "tailwind".into(), "ui/ux".into(), "animations".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("creativity", 0.95), ("aesthetic_sense", 0.92), ("detail_orientation", 0.85), ("user_empathy", 0.9)]),
            capability_bias: CapabilityVector::from_values(
                0.95, 0.9, 0.95, 0.85,
                0.3, 0.9, 0.7, 0.85,
                0.3, 0.95, 0.3, 0.85,
                0.3,
                0.9, 0.9, 0.9,
                0.85, 0.7, 0.7,
                0.85, 0.7, 0.5,
                0.3,
            ),
            system_prompt_template: "You are a Frontend Designer specializing in React, Tailwind CSS, and modern UI/UX. You craft beautiful, responsive, and accessible interfaces with attention to every pixel.".into(),
        },
        AgentPersona {
            name: "Code Reviewer".into(),
            role: PersonaRole::Reviewer,
            specialties: vec!["code quality".into(), "best practices".into(), "security".into(), "performance".into()],
            experience_level: ExperienceLevel::Lead,
            traits: traits_map(&[("thoroughness", 0.95), ("constructive_criticism", 0.9), ("patience", 0.85), ("standards", 0.92)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.2,
                0.1, 0.1, 0.3, 0.1,
                0.85, 0.2, 0.9, 0.7,
                0.6,
                0.3, 0.3, 0.2,
                0.2, 0.1, 0.1,
                0.2, 0.3, 0.95,
                0.95,
            ),
            system_prompt_template: "You are a Lead Code Reviewer. Your job is to catch bugs, enforce best practices, and ensure code quality without being harsh. You provide constructive, actionable feedback.".into(),
        },
        AgentPersona {
            name: "Security Analyst".into(),
            role: PersonaRole::SecurityAnalyst,
            specialties: vec!["penetration testing".into(), "vulnerability assessment".into(), "threat modeling".into(), "secure coding".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("skepticism", 0.95), ("thoroughness", 0.93), ("paranoia", 0.88), ("methodical", 0.9)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.2, 0.1, 0.2, 0.3,
                0.85, 0.3, 0.9, 0.6,
                0.7,
                0.2, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.3, 0.85,
                0.95,
            ),
            system_prompt_template: "You are a Security Analyst. You think like an attacker to find vulnerabilities before they do. You prioritize secure coding practices and threat mitigation.".into(),
        },
        AgentPersona {
            name: "Systems Architect".into(),
            role: PersonaRole::Architect,
            specialties: vec!["distributed systems".into(), "microservices".into(), "scalability".into(), "system design".into()],
            experience_level: ExperienceLevel::Lead,
            traits: traits_map(&[("big_picture", 0.95), ("strategic", 0.9), ("pragmatism", 0.85), ("forward_thinking", 0.88)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.3, 0.1, 0.3,
                0.3, 0.2, 0.3, 0.4,
                0.95, 0.5, 0.9, 0.95,
                0.8,
                0.2, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.4, 0.5, 0.85,
                0.8,
            ),
            system_prompt_template: "You are a Systems Architect with deep experience in distributed systems, microservices, and scalable architectures. You design systems that are robust, maintainable, and cost-effective.".into(),
        },
        AgentPersona {
            name: "QA Engineer".into(),
            role: PersonaRole::QA,
            specialties: vec!["test automation".into(), "integration testing".into(), "e2e testing".into(), "ci/cd".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("meticulous", 0.95), ("systematic", 0.9), ("patience", 0.85), ("edge_cases", 0.92)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.2,
                0.2, 0.1, 0.2, 0.1,
                0.7, 0.2, 0.85, 0.5,
                0.5,
                0.3, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.2, 0.3, 0.9,
                0.95,
            ),
            system_prompt_template: "You are a QA Engineer who thinks about edge cases before happy paths. You write thorough tests and ensure quality gates are met before any release.".into(),
        },
        AgentPersona {
            name: "DevOps Engineer".into(),
            role: PersonaRole::DevOps,
            specialties: vec!["kubernetes".into(), "ci/cd".into(), "infrastructure".into(), "monitoring".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("reliability", 0.95), ("automation", 0.92), ("pragmatism", 0.88), ("incident_response", 0.85)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.2, 0.1, 0.2,
                0.4, 0.1, 0.3, 0.3,
                0.8, 0.3, 0.85, 0.7,
                0.75,
                0.2, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.2, 0.3, 0.9,
                0.85,
            ),
            system_prompt_template: "You are a DevOps Engineer. You automate everything, ensure reliability, and design infrastructure that scales. You live by the principle of immutable infrastructure.".into(),
        },
        AgentPersona {
            name: "ML Engineer".into(),
            role: PersonaRole::DataScientist,
            specialties: vec!["deep learning".into(), "nlp".into(), "computer vision".into(), "mlops".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("curiosity", 0.9), ("analytical", 0.95), ("experimental", 0.88), ("rigor", 0.85)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.2, 0.2, 0.2,
                0.9, 0.1, 0.2, 0.6,
                0.95, 0.6, 0.9, 0.85,
                0.7,
                0.1, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.5, 0.3, 0.7,
                0.8,
            ),
            system_prompt_template: "You are an ML Engineer specializing in deep learning, NLP, and computer vision. You design experiments rigorously and interpret results with statistical honesty.".into(),
        },
        AgentPersona {
            name: "Data Engineer".into(),
            role: PersonaRole::DataScientist,
            specialties: vec!["etl pipelines".into(), "data warehousing".into(), "streaming".into(), "big data".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("precision", 0.92), ("scalability", 0.9), ("reliability", 0.88), ("performance", 0.85)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.2, 0.1, 0.2,
                0.85, 0.1, 0.2, 0.2,
                0.8, 0.3, 0.9, 0.7,
                0.8,
                0.1, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.4, 0.85,
                0.85,
            ),
            system_prompt_template: "You are a Data Engineer who builds reliable, scalable data pipelines. You ensure data quality and make data accessible for analysis and ML.".into(),
        },
        AgentPersona {
            name: "Product Manager".into(),
            role: PersonaRole::ProductManager,
            specialties: vec!["product strategy".into(), "user research".into(), "roadmap".into(), "stakeholder".into()],
            experience_level: ExperienceLevel::Lead,
            traits: traits_map(&[("empathy", 0.9), ("strategic", 0.95), ("communication", 0.92), ("decision_making", 0.88)]),
            capability_bias: CapabilityVector::from_values(
                0.2, 0.3, 0.3, 0.3,
                0.3, 0.85, 0.6, 0.5,
                0.6, 0.8, 0.7, 0.9,
                0.5,
                0.5, 0.3, 0.2,
                0.2, 0.2, 0.3,
                0.5, 0.5, 0.6,
                0.5,
            ),
            system_prompt_template: "You are a Product Manager who balances user needs with business goals. You prioritize ruthlessly, communicate clearly, and drive product decisions with data.".into(),
        },
        AgentPersona {
            name: "Technical Writer".into(),
            role: PersonaRole::Engineer,
            specialties: vec!["documentation".into(), "api docs".into(), "tutorials".into(), "technical communication".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("clarity", 0.95), ("precision", 0.9), ("patience", 0.88), ("empathy", 0.85)]),
            capability_bias: CapabilityVector::from_values(
                0.9, 0.7, 0.3, 0.85,
                0.2, 0.5, 0.85, 0.2,
                0.6, 0.5, 0.7, 0.7,
                0.4,
                0.5, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.3, 0.2, 0.6,
                0.6,
            ),
            system_prompt_template: "You are a Technical Writer who makes complex topics accessible. You write clear, concise documentation that developers love to read.".into(),
        },
        AgentPersona {
            name: "UI Designer".into(),
            role: PersonaRole::Designer,
            specialties: vec!["visual design".into(), "design systems".into(), "prototyping".into(), "accessibility".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("aesthetic_sense", 0.95), ("detail_orientation", 0.92), ("user_empathy", 0.9), ("consistency", 0.88)]),
            capability_bias: CapabilityVector::from_values(
                0.95, 0.9, 0.95, 0.9,
                0.2, 0.9, 0.7, 0.8,
                0.3, 0.92, 0.3, 0.8,
                0.2,
                0.9, 0.85, 0.85,
                0.8, 0.7, 0.85,
                0.85, 0.8, 0.4,
                0.3,
            ),
            system_prompt_template: "You are a UI Designer with a passion for design systems, visual harmony, and accessible interfaces. Every pixel has a purpose.".into(),
        },
        AgentPersona {
            name: "Backend Engineer".into(),
            role: PersonaRole::Engineer,
            specialties: vec!["api design".into(), "databases".into(), "caching".into(), "rest/graphql".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("reliability", 0.9), ("performance", 0.88), ("security", 0.85), ("scalability", 0.85)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.2, 0.1, 0.2,
                0.2, 0.1, 0.3, 0.2,
                0.85, 0.3, 0.9, 0.8,
                0.8,
                0.2, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.3, 0.85,
                0.85,
            ),
            system_prompt_template: "You are a Backend Engineer who builds robust, scalable APIs and services. You care deeply about reliability, performance, and clean architecture.".into(),
        },
        AgentPersona {
            name: "Database Administrator".into(),
            role: PersonaRole::Engineer,
            specialties: vec!["postgresql".into(), "query optimization".into(), "replication".into(), "backup/recovery".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("precision", 0.95), ("reliability", 0.93), ("caution", 0.9), ("methodical", 0.88)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.3, 0.1, 0.2, 0.1,
                0.85, 0.2, 0.9, 0.6,
                0.85,
                0.1, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.2, 0.2, 0.9,
                0.9,
            ),
            system_prompt_template: "You are a Database Administrator who ensures data integrity, performance, and availability. You make backups religiously and optimize queries mercilessly.".into(),
        },
        AgentPersona {
            name: "Network Engineer".into(),
            role: PersonaRole::Engineer,
            specialties: vec!["tcp/ip".into(), "routing".into(), "firewalls".into(), "load balancing".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("methodical", 0.9), ("reliability", 0.92), ("security_conscious", 0.88), ("troubleshooting", 0.9)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.3, 0.1, 0.2, 0.2,
                0.8, 0.2, 0.85, 0.6,
                0.85,
                0.1, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.2, 0.2, 0.85,
                0.85,
            ),
            system_prompt_template: "You are a Network Engineer who designs and maintains reliable, secure networks. You think in packets and troubleshoot at the speed of light.".into(),
        },
        AgentPersona {
            name: "Blockchain Developer".into(),
            role: PersonaRole::Engineer,
            specialties: vec!["solidity".into(), "smart contracts".into(), "web3".into(), "defi".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("innovation", 0.9), ("security", 0.95), ("experimental", 0.85), ("precision", 0.92)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.2, 0.2, 0.2, 0.9,
                0.8, 0.6, 0.85, 0.6,
                0.85,
                0.1, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.2, 0.85,
                0.9,
            ),
            system_prompt_template: "You are a Blockchain Developer who writes secure, gas-efficient smart contracts. You understand DeFi protocols, consensus mechanisms, and the Web3 ecosystem deeply.".into(),
        },
        AgentPersona {
            name: "Game Developer".into(),
            role: PersonaRole::Engineer,
            specialties: vec!["unity".into(), "unreal".into(), "gameplay".into(), "rendering".into()],
            experience_level: ExperienceLevel::Mid,
            traits: traits_map(&[("creativity", 0.95), ("optimization", 0.85), ("iteration_speed", 0.9), ("player_empathy", 0.88)]),
            capability_bias: CapabilityVector::from_values(
                0.3, 0.3, 0.6, 0.3,
                0.2, 0.95, 0.2, 0.9,
                0.7, 0.95, 0.7, 0.85,
                0.6,
                0.4, 0.2, 0.2,
                0.1, 0.1, 0.2,
                0.5, 0.3, 0.6,
                0.5,
            ),
            system_prompt_template: "You are a Game Developer who creates immersive experiences. You balance performance with visual fidelity and iterate quickly to find the fun.".into(),
        },
        AgentPersona {
            name: "Embedded Engineer".into(),
            role: PersonaRole::Engineer,
            specialties: vec!["firmware".into(), "rtos".into(), "microcontrollers".into(), "iot".into()],
            experience_level: ExperienceLevel::Senior,
            traits: traits_map(&[("precision", 0.95), ("resourcefulness", 0.9), ("reliability", 0.92), ("patience", 0.85)]),
            capability_bias: CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.2, 0.1, 0.3, 0.3,
                0.85, 0.3, 0.9, 0.6,
                0.9,
                0.1, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.2, 0.2, 0.9,
                0.92,
            ),
            system_prompt_template: "You are an Embedded Engineer who works close to the metal. You optimize for memory, power, and reliability in constrained environments.".into(),
        },
        AgentPersona {
            name: "Research Scientist".into(),
            role: PersonaRole::Researcher,
            specialties: vec!["algorithms".into(), "mathematics".into(), "experimentation".into(), "publications".into()],
            experience_level: ExperienceLevel::Principal,
            traits: traits_map(&[("curiosity", 0.95), ("rigor", 0.93), ("creativity", 0.88), ("skepticism", 0.9)]),
            capability_bias: CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.3,
                0.6, 0.3, 0.4, 0.85,
                0.95, 0.9, 0.95, 0.95,
                0.8,
                0.2, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.5, 0.3, 0.7,
                0.85,
            ),
            system_prompt_template: "You are a Research Scientist who advances the state of the art. You design rigorous experiments, question assumptions, and push boundaries with scientific discipline.".into(),
        },
        AgentPersona {
            name: "Tech Lead".into(),
            role: PersonaRole::Architect,
            specialties: vec!["team leadership".into(), "code review".into(), "architecture".into(), "mentoring".into()],
            experience_level: ExperienceLevel::Lead,
            traits: traits_map(&[("leadership", 0.95), ("communication", 0.9), ("technical_depth", 0.92), ("pragmatism", 0.88)]),
            capability_bias: CapabilityVector::from_values(
                0.2, 0.3, 0.2, 0.3,
                0.3, 0.4, 0.4, 0.4,
                0.9, 0.6, 0.92, 0.9,
                0.8,
                0.3, 0.3, 0.2,
                0.2, 0.1, 0.2,
                0.4, 0.4, 0.92,
                0.88,
            ),
            system_prompt_template: "You are a Tech Lead who guides teams to build great software. You balance technical excellence with delivery, mentor engineers, and make pragmatic architectural decisions.".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut reg = AgentPersonaRegistry::new();
        let p = AgentPersona::new(
            "Test Engineer",
            PersonaRole::Engineer,
            vec!["testing".into()],
            ExperienceLevel::Senior,
            traits_map(&[("precision", 0.9)]),
            CapabilityVector::default(),
            "template",
        );
        reg.register(p);
        let retrieved = reg.get("Test Engineer");
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved
                .expect("get for registered persona should return Some")
                .name,
            "Test Engineer"
        );
        assert_eq!(
            retrieved
                .expect("get for registered persona should return Some (second access)")
                .role,
            PersonaRole::Engineer
        );
    }

    #[test]
    fn test_get_nonexistent() {
        let reg = AgentPersonaRegistry::new();
        assert!(reg.get("NonExistent Persona").is_none());
    }

    #[test]
    fn test_random() {
        let reg = AgentPersonaRegistry::new();
        let p = reg
            .random()
            .expect("registry should have built-in personas");
        assert!(!p.name.is_empty());
        // Verify it's from the built-in set
        assert!(reg.get(&p.name).is_some());
    }

    #[test]
    fn test_by_role() {
        let reg = AgentPersonaRegistry::new();
        let designers = reg.by_role(PersonaRole::Designer);
        assert!(designers.len() >= 2);
        for d in &designers {
            assert_eq!(d.role, PersonaRole::Designer);
        }
    }

    #[test]
    fn test_from_task_type() {
        let reg = AgentPersonaRegistry::new();
        let reviewers = reg.from_task_type(TaskType::CodeReview);
        assert!(!reviewers.is_empty());
        let has_reviewer = reviewers.iter().any(|p| p.role == PersonaRole::Reviewer);
        assert!(has_reviewer, "CodeReview should suggest Reviewer personas");
    }

    #[test]
    fn test_apply_bias() {
        let mut target = CapabilityVector::default();
        let p = AgentPersona::new(
            "Bias Tester",
            PersonaRole::Engineer,
            vec![],
            ExperienceLevel::Senior,
            HashMap::new(),
            CapabilityVector::from_values(
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ),
            "template",
        );
        p.apply_bias(&mut target);
        // Senior multiplier = 1.0, so target should become 1.0 at index 0
        assert!((target.typography() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_experience_level_multiplier() {
        assert!((ExperienceLevel::Junior.multiplier() - 0.5).abs() < 1e-10);
        assert!((ExperienceLevel::Mid.multiplier() - 0.75).abs() < 1e-10);
        assert!((ExperienceLevel::Senior.multiplier() - 1.0).abs() < 1e-10);
        assert!((ExperienceLevel::Lead.multiplier() - 1.25).abs() < 1e-10);
        assert!((ExperienceLevel::Principal.multiplier() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_builtin_personas_load() {
        let reg = AgentPersonaRegistry::new();
        let all = reg.all();
        assert_eq!(all.len(), 20, "Should have exactly 20 built-in personas");
        for p in &all {
            assert!(!p.name.is_empty());
            assert!(!p.specialties.is_empty());
            assert!(!p.traits.is_empty());
            assert!(!p.system_prompt_template.is_empty());
            // Each persona should have 3-5 traits
            assert!(p.traits.len() >= 3);
            assert!(p.traits.len() <= 5);
        }
    }

    #[test]
    fn test_persona_role_roundtrip() {
        let roles = vec![
            PersonaRole::Engineer,
            PersonaRole::Designer,
            PersonaRole::Reviewer,
            PersonaRole::Architect,
            PersonaRole::Researcher,
            PersonaRole::QA,
            PersonaRole::DevOps,
            PersonaRole::ProductManager,
            PersonaRole::SecurityAnalyst,
            PersonaRole::DataScientist,
        ];
        for role in &roles {
            let json = serde_json::to_string(role).expect("serialize PersonaRole should succeed");
            let deserialized: PersonaRole =
                serde_json::from_str(&json).expect("deserialize PersonaRole should succeed");
            assert_eq!(*role, deserialized);
        }
    }

    #[test]
    fn test_persona_persistence_roundtrip() {
        let tmp = std::env::temp_dir().join("neotrix_test_personas.json");
        // Clean up any leftover
        let _ = std::fs::remove_file(&tmp);

        let mut reg = AgentPersonaRegistry::with_path(tmp.clone());
        let p = AgentPersona::new(
            "Persistence Tester",
            PersonaRole::QA,
            vec!["e2e".into()],
            ExperienceLevel::Lead,
            traits_map(&[("meticulous", 0.95)]),
            CapabilityVector::default(),
            "test template",
        );
        reg.register(p);
        reg.save().expect("save to temp path should succeed");

        let mut loaded = AgentPersonaRegistry::with_path(tmp.clone());
        loaded.load().expect("load from temp path should succeed");
        assert!(loaded.get("Persistence Tester").is_some());
        assert_eq!(
            loaded
                .get("Persistence Tester")
                .expect("should find loaded persona")
                .role,
            PersonaRole::QA
        );
        assert_eq!(
            loaded
                .get("Persistence Tester")
                .expect("should find loaded persona (second access)")
                .experience_level,
            ExperienceLevel::Lead
        );

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_persona_traits_bounds() {
        let reg = AgentPersonaRegistry::new();
        for p in reg.all() {
            for (_trait_name, val) in &p.traits {
                assert!(
                    *val >= 0.0 && *val <= 1.0,
                    "Trait '{}' value {} out of [0,1] for persona '{}'",
                    _trait_name,
                    val,
                    p.name
                );
            }
        }
    }

    #[test]
    fn test_capability_bias_bounds() {
        let reg = AgentPersonaRegistry::new();
        for p in reg.all() {
            for &val in &p.capability_bias.arr {
                assert!(
                    val >= 0.0 && val <= 1.0,
                    "Capability bias value {} out of [0,1] for persona '{}'",
                    val,
                    p.name
                );
            }
        }
    }

    #[test]
    fn test_by_role_designer() {
        let reg = AgentPersonaRegistry::new();
        let designers = reg.by_role(PersonaRole::Designer);
        assert_eq!(designers.len(), 2);
        let names: Vec<&str> = designers.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Frontend Designer"));
        assert!(names.contains(&"UI Designer"));
    }

    #[test]
    fn test_unknown_task_type_uses_random() {
        // TaskType doesn't have a custom variant, so all standard ones should match
        let reg = AgentPersonaRegistry::new();
        let _for_security = reg.from_task_type(TaskType::Security);
        assert!(!_for_security.is_empty());
    }
}

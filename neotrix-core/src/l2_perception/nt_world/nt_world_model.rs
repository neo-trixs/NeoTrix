// Stub: nt_world_model

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    General,
    Prediction,
    Classification,
    Generation,
    CodeAnalysis,
    CodeGeneration,
    Security,
    Design,
    UIDesign,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Context {
    pub task_type: TaskType,
    pub input: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Context {
    pub fn from_task_description(desc: &str) -> Self {
        let t = desc.to_lowercase();
        let task_type = if t.contains("design") || t.contains("设计") {
            TaskType::Design
        } else if t.contains("ui") || t.contains("界面") {
            TaskType::UIDesign
        } else if t.contains("code") || t.contains("代码") {
            TaskType::CodeAnalysis
        } else if t.contains("generate") || t.contains("生成") {
            TaskType::CodeGeneration
        } else if t.contains("security") || t.contains("安全") {
            TaskType::Security
        } else if t.contains("predict") || t.contains("预测") {
            TaskType::Prediction
        } else if t.contains("classify") || t.contains("分类") {
            TaskType::Classification
        } else {
            TaskType::General
        };
        Self {
            task_type,
            input: desc.to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

pub struct WorldModel {
    pub num_experts: usize,
}

impl WorldModel {
    pub fn new(num_experts: usize) -> Self {
        Self { num_experts }
    }

    pub fn predict(&self, _context: &Context) -> Result<String, String> {
        Ok("prediction".to_string())
    }

    pub fn update(&mut self, _observation: &str) -> Result<(), String> {
        Ok(())
    }
}

pub struct SimpleWorldModel;

impl SimpleWorldModel {
    pub fn predict(&self, _context: &Context) -> Result<String, String> {
        Ok("prediction".to_string())
    }

    pub fn update(&mut self, _observation: &str) -> Result<(), String> {
        Ok(())
    }
}

pub struct WorldModelV2 {
    pub num_experts: usize,
    pub input_dim: usize,
}

impl WorldModelV2 {
    pub fn new(num_experts: usize, input_dim: usize) -> Self {
        Self { num_experts, input_dim }
    }

    pub fn predict(&self, _context: &Context) -> Result<String, String> {
        Ok("v2 prediction".to_string())
    }
}

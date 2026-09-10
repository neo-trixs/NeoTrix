//! 内容生成器 (Generator)
//! 
//! 内容生成、方案生成、创新生成


use serde::{Deserialize, Serialize};

/// 内容生成器
pub struct Generator {
    /// 生成历史
    pub history: Vec<GenerationRecord>,
    /// 创意库
    pub creativity_pool: Vec<CreativeElement>,
}

/// 创意元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeElement {
    pub id: String,
    pub element_type: ElementType,
    pub content: String,
    pub novelty: f64,
    pub usefulness: f64,
}

/// 元素类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Idea,
    Metaphor,
    Analogy,
    Solution,
    Innovation,
}

/// 生成记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRecord {
    pub id: String,
    pub cycle: u32,
    pub generation_type: GenerationType,
    pub input: String,
    pub output: String,
    pub creativity_score: f64,
    pub timestamp: String,
}

/// 生成类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationType {
    Content,
    Solution,
    Innovation,
    Synthesis,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            creativity_pool: Vec::new(),
        }
    }

    /// 生成内容
    pub fn generate_content(&mut self, cycle: u32, prompt: &str, context: &str) -> String {
        let output = format!("Generated content for: {} (context: {})", prompt, context);
        
        let record = GenerationRecord {
            id: format!("gen_{}", uuid::Uuid::new_v4()),
            cycle,
            generation_type: GenerationType::Content,
            input: prompt.to_string(),
            output: output.clone(),
            creativity_score: 0.7,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        self.history.push(record);
        output
    }

    /// 生成方案
    pub fn generate_solution(&mut self, cycle: u32, problem: &str) -> String {
        let solution = format!("Solution for: {}", problem);
        
        let record = GenerationRecord {
            id: format!("gen_{}", uuid::Uuid::new_v4()),
            cycle,
            generation_type: GenerationType::Solution,
            input: problem.to_string(),
            output: solution.clone(),
            creativity_score: 0.8,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        self.history.push(record);
        solution
    }

    /// 获取统计
    pub fn stats(&self) -> GeneratorStats {
        GeneratorStats {
            total_generations: self.history.len(),
            avg_creativity: if self.history.is_empty() {
                0.0
            } else {
                self.history.iter().map(|r| r.creativity_score).sum::<f64>() / self.history.len() as f64
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorStats {
    pub total_generations: usize,
    pub avg_creativity: f64,
}

impl std::fmt::Display for GeneratorStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Generator: {} generations, avg creativity {:.4}",
            self.total_generations, self.avg_creativity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator() {
        let mut gen = Generator::new();
        let output = gen.generate_content(0, "test", "context");
        assert!(!output.is_empty());
    }
}

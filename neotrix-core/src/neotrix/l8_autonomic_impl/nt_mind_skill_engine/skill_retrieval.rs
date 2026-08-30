/// SkillCorpus 检索层 —— bi-encoder + reranker (2026-08-25 吸收)
///
/// 基于 EverMind-AI/SkillCorpus 吸收:
/// - bi-encoder: Qwen3-Embedding-0.6B (快速检索)
/// - reranker: Qwen3-Reranker-0.6B (精确重排)
/// - 16 类别分类 + 3 维质量 (utility/robustness/safety)
/// - 114k skills 规模, +7.5pp 基准提升

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::neotrix::l8_autonomic_impl::nt_mind_skill_engine::SkillEntry;

/// 技能检索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillQuery {
    pub text: String,
    pub category: Option<String>,
    pub top_k: usize,
    pub min_score: f64,
}

/// 检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub skill_name: String,
    pub score: f64,
    pub category: String,
    pub snippet: String,
}

/// 技能嵌入缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEmbedding {
    pub skill_name: String,
    pub embedding: Vec<f32>,
    pub category: String,
    pub quality_scores: QualityScores,
}

/// 质量分数 (SkillCorpus 3 维度)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityScores {
    pub utility: f64,
    pub robustness: f64,
    pub safety: f64,
    pub overall: f64,
}

/// 技能分类 (SkillCorpus 16 类别)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    CodeGeneration,
    CodeAnalysis,
    Testing,
    Documentation,
    Refactoring,
    Architecture,
    Security,
    Performance,
    DataProcessing,
    MachineLearning,
    DevOps,
    Debugging,
    APIDesign,
    Database,
    Frontend,
    General,
}

impl SkillCategory {
    pub fn all() -> Vec<SkillCategory> {
        vec![
            SkillCategory::CodeGeneration,
            SkillCategory::CodeAnalysis,
            SkillCategory::Testing,
            SkillCategory::Documentation,
            SkillCategory::Refactoring,
            SkillCategory::Architecture,
            SkillCategory::Security,
            SkillCategory::Performance,
            SkillCategory::DataProcessing,
            SkillCategory::MachineLearning,
            SkillCategory::DevOps,
            SkillCategory::Debugging,
            SkillCategory::APIDesign,
            SkillCategory::Database,
            SkillCategory::Frontend,
            SkillCategory::General,
        ]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "code_generation" | "code-generation" => Some(SkillCategory::CodeGeneration),
            "code_analysis" | "code-analysis" => Some(SkillCategory::CodeAnalysis),
            "testing" => Some(SkillCategory::Testing),
            "documentation" => Some(SkillCategory::Documentation),
            "refactoring" => Some(SkillCategory::Refactoring),
            "architecture" => Some(SkillCategory::Architecture),
            "security" => Some(SkillCategory::Security),
            "performance" => Some(SkillCategory::Performance),
            "data_processing" | "data-processing" => Some(SkillCategory::DataProcessing),
            "machine_learning" | "ml" => Some(SkillCategory::MachineLearning),
            "devops" => Some(SkillCategory::DevOps),
            "debugging" => Some(SkillCategory::Debugging),
            "api_design" | "api-design" => Some(SkillCategory::APIDesign),
            "database" => Some(SkillCategory::Database),
            "frontend" => Some(SkillCategory::Frontend),
            _ => Some(SkillCategory::General),
        }
    }
}

/// 技能检索引擎 (bi-encoder + reranker)
pub struct SkillRetriever {
    bi_encoder: Arc<dyn BiEncoder>,
    reranker: Arc<dyn Reranker>,
    embeddings: Arc<Mutex<HashMap<String, SkillEmbedding>>>,
    category_index: Arc<Mutex<HashMap<String, Vec<String>>>>, // category -> skill names
}

/// Bi-encoder trait (快速检索)
pub trait BiEncoder: Send + Sync {
    fn encode(&self, text: &str) -> Result<Vec<f32>, String>;
    fn batch_encode(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String>;
    fn dim(&self) -> usize;
}

/// Reranker trait (精确重排)
pub trait Reranker: Send + Sync {
    fn rerank(&self, query: &str, candidates: &[&str]) -> Result<Vec<f32>, String>;
}

/// 默认实现 (占位, 生产需接入 Qwen3-Embedding-0.6B / Qwen3-Reranker-0.6B)
pub struct PlaceholderBiEncoder;
pub struct PlaceholderReranker;

impl BiEncoder for PlaceholderBiEncoder {
    fn encode(&self, text: &str) -> Result<Vec<f32>, String> {
        // 占位: 随机向量 (实际应接入 Qwen3-Embedding-0.6B)
        let dim = 1024;
        let mut v = vec![0.0f32; dim];
        for (i, c) in text.chars().enumerate() {
            if i < dim { v[i] = (c as u32 as f32) / 1000.0; }
        }
        Ok(v)
    }

    fn batch_encode(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        texts.iter().map(|t| self.encode(t)).collect()
    }

    fn dim(&self) -> usize { 1024 }
}

impl Reranker for PlaceholderReranker {
    fn rerank(&self, query: &str, candidates: &[&str]) -> Result<Vec<f32>, String> {
        // 占位: 基于字符串相似度
        let scores: Vec<f32> = candidates.iter().map(|c| {
            // 简化: 基于词重叠
            let q_words: std::collections::HashSet<_> = query.split_whitespace().collect();
            let c_words: std::collections::HashSet<_> = c.split_whitespace().collect();
            let inter = q_words.intersection(&c_words).count();
            let union = q_words.union(&c_words).count();
            if union == 0 { 0.0 } else { inter as f32 / union as f32 }
        }).collect();
        Ok(scores)
    }
}

impl SkillRetriever {
    pub fn new() -> Self {
        Self {
            bi_encoder: Arc::new(PlaceholderBiEncoder),
            reranker: Arc::new(PlaceholderReranker),
            embeddings: Arc::new(Mutex::new(HashMap::<String, SkillEmbedding>::new())),
            category_index: Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new())),
        }
    }

    pub fn with_bi_encoder(mut self, encoder: Arc<dyn BiEncoder>) -> Self {
        self.bi_encoder = encoder;
        self
    }

    pub fn with_reranker(mut self, reranker: Arc<dyn Reranker>) -> Self {
        self.reranker = reranker;
        self
    }

    /// 索引技能 (计算嵌入 + 分类)
    pub fn index_skill(&self, skill: &SkillEntry) -> Result<(), String> {
        let text = format!("{} {}", skill.name, skill.description);
        let embedding = self.bi_encoder.encode(&text)?;

        let category = SkillCategory::from_str(&skill.category).unwrap_or(SkillCategory::General);
        let category_str = format!("{:?}", category);

        let embedding = SkillEmbedding {
            skill_name: skill.name.clone(),
            embedding,
            category: category_str.clone(),
            quality_scores: QualityScores::default(),
        };

        self.embeddings.lock().unwrap().insert(skill.name.clone(), embedding);
        self.category_index.lock().unwrap()
            .entry(category_str)
            .or_default()
            .push(skill.name.clone());

        Ok(())
    }

    /// 批量索引
    pub fn index_skills(&self, skills: &[SkillEntry]) -> Result<usize, String> {
        let mut count = 0;
        for skill in skills {
            if self.index_skill(skill).is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// 检索技能 (bi-encoder 召回 + reranker 重排)
    pub fn retrieve(&self, query: &SkillQuery) -> Result<Vec<RetrievalResult>, String> {
        let embeddings = self.embeddings.lock().unwrap();
        if embeddings.is_empty() {
            return Ok(Vec::new());
        }

        // 1. Bi-encoder: 计算查询嵌入
        let query_emb = self.bi_encoder.encode(&query.text)?;

        // 2. 向量相似度召回 (余弦相似度)
        let embeddings_guard = self.embeddings.lock().unwrap();
        let mut candidates: Vec<(String, f32)> = embeddings_guard.iter()
            .map(|(name, emb)| {
                let sim = cosine_similarity(&query_emb, &emb.embedding);
                (name.clone(), sim)
            })
            .collect();

        // 按相似度排序
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 应用类别过滤
        if query.category.is_some() {
            candidates.retain(|(name, _)| {
                self.category_index.lock().unwrap()
                    .get(&query.category.clone().unwrap_or_default())
                    .map(|v| v.contains(name))
                    .unwrap_or(true)
            });
        }

        // Top-K 召回
        let top_k = query.top_k.min(candidates.len());
        let top_candidates: Vec<_> = candidates.into_iter().take(top_k).collect();

        // Reranker 重排: 先收集候选文本 (owned), 再释放锁
        let candidate_texts: Vec<String> = top_candidates.iter()
            .map(|(name, _)| embeddings_guard.get(name).map(|e| e.skill_name.clone()).unwrap_or_default())
            .collect();

        drop(embeddings_guard);

        // Reranker 重排
        let rerank_scores = self.reranker.rerank(&query.text, &candidate_texts.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;

        // 合并分数
        let mut results = Vec::new();
        for ((name, bi_score), rerank_score) in top_candidates.into_iter().zip(rerank_scores.into_iter()) {
            let combined = (0.7 * bi_score + 0.3 * rerank_score) as f64;
            if combined >= query.min_score {
                let emb = self.embeddings.lock().unwrap().get(&name).cloned();
                let (category, snippet) = emb
                    .map(|e| (e.category, e.skill_name))
                    .unwrap_or_default();
                results.push(RetrievalResult {
                    skill_name: name.clone(),
                    score: combined,
                    category,
                    snippet,
                });
            }
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    /// 按类别检索
    pub fn retrieve_by_category(&self, category: SkillCategory, top_k: usize) -> Vec<RetrievalResult> {
        let cat_str = format!("{:?}", category);
        let index = self.category_index.lock().unwrap();
        if let Some(names) = index.get(&cat_str) {
            names.iter()
                .take(top_k)
                .filter_map(|name| {
                    self.embeddings.lock().unwrap().get(name).map(|emb| RetrievalResult {
                        skill_name: name.clone(),
                        score: emb.quality_scores.overall,
                        category: format!("{:?}", category),
                        snippet: emb.skill_name.clone(),
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 更新质量分数
    pub fn update_quality(&self, skill_name: &str, scores: QualityScores) {
        if let Some(emb) = self.embeddings.lock().unwrap().get_mut(skill_name) {
            emb.quality_scores = scores;
        }
    }
}

/// 余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}

impl Default for SkillRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_category_from_str() {
        assert_eq!(SkillCategory::from_str("code_generation"), Some(SkillCategory::CodeGeneration));
        assert_eq!(SkillCategory::from_str("testing"), Some(SkillCategory::Testing));
        assert_eq!(SkillCategory::from_str("unknown"), Some(SkillCategory::General));
    }
}

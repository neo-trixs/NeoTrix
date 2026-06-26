use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InsightType {
    RecurringTheme,
    Contradiction,
    KnowledgeGap,
    MetaInsight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionInsight {
    pub id: u64,
    pub content: String,
    pub insight_type: InsightType,
    pub confidence: f64,
    pub source_count: usize,
    pub cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReflector {
    insights: Vec<ReflectionInsight>,
    next_id: u64,
    cycle: u64,
    max_insights: usize,
}

impl Default for MemoryReflector {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryReflector {
    pub fn new() -> Self {
        Self {
            insights: Vec::with_capacity(50),
            next_id: 1,
            cycle: 0,
            max_insights: 50,
        }
    }

    pub fn tick(&mut self) {
        self.cycle += 1;
    }

    pub fn reflect(
        &mut self,
        palace_entries: &[crate::core::nt_core_consciousness::memory_palace::PalaceEntry],
        lattice_layers: &[Vec<crate::core::nt_core_consciousness::memory_lattice::LatticeEntry>],
    ) -> Vec<ReflectionInsight> {
        let mut new_insights = Vec::new();

        // Collect all content strings
        let mut all_content: Vec<String> = Vec::new();
        for entry in palace_entries {
            all_content.push(entry.content.clone());
        }
        for layer in lattice_layers {
            for entry in layer {
                all_content.push(entry.content.clone());
            }
        }

        if all_content.len() < 3 {
            return new_insights;
        }

        // 1. Recurring theme detection: count word frequency across entries
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        for content in &all_content {
            for word in content.split_whitespace() {
                let cleaned = word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();
                if cleaned.len() > 3 {
                    *word_freq.entry(cleaned).or_insert(0) += 1;
                }
            }
        }

        let mut freq_pairs: Vec<(String, usize)> = word_freq.into_iter().collect();
        freq_pairs.sort_by(|a, b| b.1.cmp(&a.1));
        for (word, count) in freq_pairs.iter().take(3) {
            if *count >= 3 && !self.has_similar_insight(word, 0.5) {
                let insight = ReflectionInsight {
                    id: self.next_id,
                    content: format!("Recurring theme: '{}' appears in {} memories", word, count),
                    insight_type: InsightType::RecurringTheme,
                    confidence: (*count as f64 / all_content.len() as f64).min(1.0),
                    source_count: *count,
                    cycle: self.cycle,
                };
                self.next_id += 1;
                new_insights.push(insight);
            }
        }

        // 2. Contradiction detection: entries about same topic with opposite sentiments
        let mut topic_groups: HashMap<String, Vec<&str>> = HashMap::new();
        for content in &all_content {
            let topic = content
                .split_whitespace()
                .take(5)
                .collect::<Vec<_>>()
                .join(" ");
            if topic.len() > 10 {
                topic_groups.entry(topic).or_default().push(content);
            }
        }
        for (_topic, group) in topic_groups.iter() {
            let neg_count = group
                .iter()
                .filter(|c| {
                    c.contains("not")
                        || c.contains("fail")
                        || c.contains("error")
                        || c.contains("wrong")
                        || c.contains("bad")
                })
                .count();
            let pos_count = group
                .iter()
                .filter(|c| {
                    c.contains("success")
                        || c.contains("good")
                        || c.contains("correct")
                        || c.contains("works")
                        || c.contains("improve")
                })
                .count();
            if neg_count > 0 && pos_count > 0 && !self.has_similar_insight("contradiction", 0.3) {
                let insight = ReflectionInsight {
                    id: self.next_id,
                    content: format!(
                        "Detected contradiction: {} positive vs {} negative entries on related topic",
                        pos_count, neg_count
                    ),
                    insight_type: InsightType::Contradiction,
                    confidence: (neg_count.min(pos_count) as f64 / group.len() as f64).min(1.0),
                    source_count: group.len(),
                    cycle: self.cycle,
                };
                self.next_id += 1;
                new_insights.push(insight);
            }
        }

        // 3. Knowledge gap detection: low-confidence entries on unique topics
        let unique_topics: Vec<&str> = all_content
            .iter()
            .filter(|c| {
                let words: Vec<&str> = c.split_whitespace().collect();
                words.len() >= 3 && words.len() <= 10
            })
            .map(|c| c.as_str())
            .collect();
        if unique_topics.len() >= 2 && !self.has_similar_insight("knowledge gap", 0.3) {
            let insight = ReflectionInsight {
                id: self.next_id,
                content: format!(
                    "Knowledge gap: {} isolated entries with limited connections",
                    unique_topics.len()
                ),
                insight_type: InsightType::KnowledgeGap,
                confidence: 0.3,
                source_count: unique_topics.len(),
                cycle: self.cycle,
            };
            self.next_id += 1;
            new_insights.push(insight);
        }

        // Store new insights (bounded)
        for insight in &new_insights {
            if self.insights.len() >= self.max_insights {
                self.insights.remove(0);
            }
            self.insights.push(insight.clone());
        }

        new_insights
    }

    fn has_similar_insight(&self, keyword: &str, _threshold: f64) -> bool {
        let kw = keyword.to_lowercase();
        self.insights
            .iter()
            .any(|i| i.content.to_lowercase().contains(&kw))
    }

    pub fn recent_insights(&self, n: usize) -> Vec<&ReflectionInsight> {
        self.insights.iter().rev().take(n).collect()
    }

    pub fn insight_count(&self) -> usize {
        self.insights.len()
    }

    /// Consolidate low-level insights into higher-level MetaInsights.
    /// Returns consolidated insights with confidence boosted by repetition count.
    pub fn consolidate(&mut self) -> Vec<ReflectionInsight> {
        if self.insights.len() < 3 {
            return Vec::new();
        }

        // Group insights by type
        let mut by_type: HashMap<InsightType, Vec<&ReflectionInsight>> = HashMap::new();
        for insight in &self.insights {
            by_type
                .entry(insight.insight_type.clone())
                .or_default()
                .push(insight);
        }

        let mut consolidated = Vec::new();
        for (itype, group) in by_type {
            let count = group.len();
            let avg_confidence: f64 =
                group.iter().map(|i| i.confidence).sum::<f64>() / count as f64;
            // Promote recurring insights: ≥3 entries of same type → MetaInsight
            if count >= 3 && !self.has_similar_insight(&format!("consolidated_{:?}", itype), 0.7) {
                let content = format!(
                    "Consolidated {:?}: {} instances at avg confidence {:.2}",
                    itype, count, avg_confidence
                );
                let insight = ReflectionInsight {
                    id: self.next_id,
                    content,
                    insight_type: InsightType::MetaInsight,
                    confidence: (avg_confidence + 0.1 * count as f64).min(1.0),
                    source_count: count,
                    cycle: self.cycle,
                };
                self.next_id += 1;
                consolidated.push(insight);
            }
        }

        // Store consolidated insights
        for insight in &consolidated {
            if self.insights.len() >= self.max_insights {
                self.insights.remove(0);
            }
            self.insights.push(insight.clone());
        }

        consolidated
    }

    pub fn diagnostic(&self) -> String {
        format!(
            "mem_reflect:insights={}|next_id={}|cycle={}",
            self.insights.len(),
            self.next_id,
            self.cycle
        )
    }
}

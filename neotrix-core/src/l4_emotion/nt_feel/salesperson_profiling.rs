#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── Helper types ─────────────────────────────────────────────────────────

pub struct ChatRecord {
    pub message: String,
    pub sender: String,
    pub timestamp: i64,
}

pub struct EmailRecord {
    pub subject: String,
    pub body: String,
    pub sender: String,
    pub timestamp: i64,
}

pub struct Customer {
    pub customer_id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct Interaction {
    pub interaction_type: String,
    pub timestamp: i64,
    pub customer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FollowUpFrequency {
    Never,
    Rarely,
    Weekly,
    Daily,
    MultipleDaily,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InsightType {
    TopPerformer,
    NeedsCoaching,
    LanguageExpert,
    CommunicationGap,
    CustomerRetention,
}

// ── Core types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStyle {
    pub formality_level: f32,
    pub avg_message_length: u32,
    pub emoji_usage: f32,
    pub question_ratio: f32,
    pub response_speed: ResponseSpeed,
    pub templates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSpeed {
    pub avg_response_minutes: f32,
    pub fastest_response_minutes: f32,
    pub slowest_response_minutes: f32,
    pub responsiveness_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePreference {
    pub primary_language: String,
    pub secondary_languages: Vec<String>,
    pub multilingual_score: f32,
    pub translation_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPattern {
    pub preferred_channel: String,
    pub active_hours: Vec<u32>,
    pub follow_up_frequency: FollowUpFrequency,
    pub avg_interactions_per_customer: f32,
    pub customer_retention_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopCustomer {
    pub customer_id: String,
    pub customer_name: String,
    pub interaction_count: u32,
    pub total_value: f64,
    pub relationship_duration_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStats {
    pub total_whatsapp_chats: u32,
    pub total_emails: u32,
    pub total_phone_calls: u32,
    pub total_meetings: u32,
    pub unique_customers: u32,
    pub gonghai_customers: u32,
    pub activity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalespersonProfile {
    pub salesperson_id: String,
    pub name: String,
    pub writing_style: WritingStyle,
    pub language_preference: LanguagePreference,
    pub communication_pattern: CommunicationPattern,
    pub top_customers: Vec<TopCustomer>,
    pub activity_stats: ActivityStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileComparison {
    pub salesperson1: String,
    pub salesperson2: String,
    pub similarities: Vec<String>,
    pub differences: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub affected_salespersons: Vec<String>,
    pub priority: Priority,
}

// ── Profiler ─────────────────────────────────────────────────────────────

pub struct SalespersonProfiler {
    profiles: HashMap<String, SalespersonProfile>,
}

impl Default for SalespersonProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl SalespersonProfiler {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    /// Generate a complete salesperson profile from extracted data.
    pub fn generate_profile(
        &mut self,
        salesperson_id: &str,
        name: &str,
        whatsapp_chats: &[ChatRecord],
        emails: &[EmailRecord],
        customers: &[Customer],
    ) -> SalespersonProfile {
        let mut all_messages: Vec<String> = whatsapp_chats
            .iter()
            .filter(|c| c.sender == salesperson_id)
            .map(|c| c.message.clone())
            .collect();

        let email_messages: Vec<String> = emails
            .iter()
            .filter(|e| e.sender == salesperson_id)
            .map(|e| format!("{} {}", e.subject, e.body))
            .collect();

        all_messages.extend(email_messages);

        let writing_style = self.analyze_writing_style(&all_messages);
        let language_preference = self.detect_language_preference(&all_messages);

        let mut interactions: Vec<Interaction> = whatsapp_chats
            .iter()
            .filter(|c| c.sender == salesperson_id)
            .map(|c| Interaction {
                interaction_type: "whatsapp".to_string(),
                timestamp: c.timestamp,
                customer_id: String::new(),
            })
            .collect();

        interactions.extend(emails.iter().filter(|e| e.sender == salesperson_id).map(|e| {
            Interaction {
                interaction_type: "email".to_string(),
                timestamp: e.timestamp,
                customer_id: String::new(),
            }
        }));

        let communication_pattern = self.model_communication_pattern(&interactions);

        let top_customers =
            self.compute_top_customers(whatsapp_chats, emails, customers, salesperson_id);
        let activity_stats =
            self.compute_activity_stats(whatsapp_chats, emails, customers, salesperson_id);

        let profile = SalespersonProfile {
            salesperson_id: salesperson_id.to_string(),
            name: name.to_string(),
            writing_style,
            language_preference,
            communication_pattern,
            top_customers,
            activity_stats,
        };

        self.profiles
            .insert(salesperson_id.to_string(), profile.clone());
        profile
    }

    /// Analyze writing style from a collection of messages.
    pub fn analyze_writing_style(&self, messages: &[String]) -> WritingStyle {
        if messages.is_empty() {
            return WritingStyle {
                formality_level: 0.5,
                avg_message_length: 0,
                emoji_usage: 0.0,
                question_ratio: 0.0,
                response_speed: ResponseSpeed {
                    avg_response_minutes: 0.0,
                    fastest_response_minutes: 0.0,
                    slowest_response_minutes: 0.0,
                    responsiveness_score: 0.0,
                },
                templates: vec![],
            };
        }

        let total_chars: usize = messages.iter().map(|m| m.len()).sum();
        let avg_len = (total_chars / messages.len()) as u32;

        let emoji_count: u32 = messages
            .iter()
            .filter(|m| m.chars().any(|c| is_emoji(c)))
            .count() as u32;
        let emoji_usage = emoji_count as f32 / messages.len() as f32;

        let question_count = messages
            .iter()
            .filter(|m| m.contains('?'))
            .count() as f32;
        let question_ratio = question_count / messages.len() as f32;

        let formality_level = estimate_formality(messages);

        let templates = extract_common_phrases(messages);

        WritingStyle {
            formality_level,
            avg_message_length: avg_len,
            emoji_usage,
            question_ratio,
            response_speed: ResponseSpeed {
                avg_response_minutes: 0.0,
                fastest_response_minutes: 0.0,
                slowest_response_minutes: 0.0,
                responsiveness_score: 0.5,
            },
            templates,
        }
    }

    /// Detect language preference from message content.
    pub fn detect_language_preference(&self, messages: &[String]) -> LanguagePreference {
        if messages.is_empty() {
            return LanguagePreference {
                primary_language: "unknown".to_string(),
                secondary_languages: vec![],
                multilingual_score: 0.0,
                translation_patterns: vec![],
            };
        }

        let mut lang_counts: HashMap<String, u32> = HashMap::new();
        for msg in messages {
            let lang = detect_message_language(msg);
            *lang_counts.entry(lang).or_insert(0) += 1;
        }

        let mut sorted: Vec<(String, u32)> = lang_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        let primary = sorted.first().map(|(l, _)| l.clone()).unwrap_or_default();
        let secondary: Vec<String> = sorted[1..].iter().map(|(l, _)| l.clone()).collect();

        let unique_langs = sorted.len() as f32;
        let multilingual_score = if unique_langs <= 1.0 {
            0.0
        } else {
            (unique_langs / 5.0).min(1.0)
        };

        LanguagePreference {
            primary_language: primary,
            secondary_languages: secondary,
            multilingual_score,
            translation_patterns: vec![],
        }
    }

    /// Model communication patterns from interactions.
    pub fn model_communication_pattern(
        &self,
        interactions: &[Interaction],
    ) -> CommunicationPattern {
        if interactions.is_empty() {
            return CommunicationPattern {
                preferred_channel: "unknown".to_string(),
                active_hours: vec![],
                follow_up_frequency: FollowUpFrequency::Never,
                avg_interactions_per_customer: 0.0,
                customer_retention_rate: 0.0,
            };
        }

        let mut channel_counts: HashMap<String, u32> = HashMap::new();
        let mut hour_counts: HashMap<u32, u32> = HashMap::new();

        for interaction in interactions {
            *channel_counts
                .entry(interaction.interaction_type.clone())
                .or_insert(0) += 1;

            let hour = ((interaction.timestamp % 86400) / 3600) as u32;
            *hour_counts.entry(hour).or_insert(0) += 1;
        }

        let preferred_channel = channel_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(ch, _)| ch.clone())
            .unwrap_or_default();

        let active_hours: Vec<u32> = hour_counts
            .iter()
            .filter(|(_, c)| **c > 1)
            .map(|(h, _)| *h)
            .collect();

        let total_customers: u32 = interactions
            .iter()
            .filter(|i| !i.customer_id.is_empty())
            .map(|i| &i.customer_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;

        let avg_per_customer = if total_customers > 0 {
            interactions.len() as f32 / total_customers as f32
        } else {
            0.0
        };

        let follow_up = classify_follow_up_frequency(interactions);

        CommunicationPattern {
            preferred_channel,
            active_hours,
            follow_up_frequency: follow_up,
            avg_interactions_per_customer: avg_per_customer,
            customer_retention_rate: 0.0,
        }
    }

    /// Get profile by salesperson ID.
    pub fn get_profile(&self, salesperson_id: &str) -> Option<&SalespersonProfile> {
        self.profiles.get(salesperson_id)
    }

    /// Compare two salesperson profiles.
    pub fn compare_profiles(&self, id1: &str, id2: &str) -> Option<ProfileComparison> {
        let p1 = self.profiles.get(id1)?;
        let p2 = self.profiles.get(id2)?;

        let mut similarities = Vec::new();
        let mut differences = Vec::new();

        // Writing style comparison
        let formality_diff = (p1.writing_style.formality_level
            - p2.writing_style.formality_level)
            .abs();
        if formality_diff < 0.15 {
            similarities.push("Similar formality level".to_string());
        } else {
            differences.push(format!(
                "Formality: {} vs {}",
                p1.writing_style.formality_level, p2.writing_style.formality_level
            ));
        }

        if (p1.writing_style.question_ratio - p2.writing_style.question_ratio).abs() < 0.1 {
            similarities.push("Similar question frequency".to_string());
        } else {
            differences.push(format!(
                "Question ratio: {:.2} vs {:.2}",
                p1.writing_style.question_ratio, p2.writing_style.question_ratio
            ));
        }

        // Language comparison
        if p1.language_preference.primary_language == p2.language_preference.primary_language {
            similarities
                .push(format!("Primary language: {}", p1.language_preference.primary_language));
        } else {
            differences.push(format!(
                "Primary language: {} vs {}",
                p1.language_preference.primary_language,
                p2.language_preference.primary_language
            ));
        }

        // Channel comparison
        if p1.communication_pattern.preferred_channel
            == p2.communication_pattern.preferred_channel
        {
            similarities.push(format!(
                "Preferred channel: {}",
                p1.communication_pattern.preferred_channel
            ));
        } else {
            differences.push(format!(
                "Preferred channel: {} vs {}",
                p1.communication_pattern.preferred_channel,
                p2.communication_pattern.preferred_channel
            ));
        }

        // Activity comparison
        let activity_diff = (p1.activity_stats.activity_score
            - p2.activity_stats.activity_score)
            .abs();
        if activity_diff < 10.0 {
            similarities.push("Similar activity levels".to_string());
        } else {
            differences.push(format!(
                "Activity score: {:.1} vs {:.1}",
                p1.activity_stats.activity_score, p2.activity_stats.activity_score
            ));
        }

        let recommendation = if p1.activity_stats.activity_score
            > p2.activity_stats.activity_score
        {
            format!(
                "Consider having {} mentor {} on activity patterns",
                p1.name, p2.name
            )
        } else {
            format!(
                "Consider having {} mentor {} on activity patterns",
                p2.name, p1.name
            )
        };

        Some(ProfileComparison {
            salesperson1: p1.name.clone(),
            salesperson2: p2.name.clone(),
            similarities,
            differences,
            recommendation,
        })
    }

    /// Generate insights across all profiles.
    pub fn generate_insights(&self) -> Vec<ProfileInsight> {
        let mut insights = Vec::new();

        if self.profiles.is_empty() {
            return insights;
        }

        // Find top performer by activity score
        let top = self
            .profiles
            .values()
            .max_by(|a, b| {
                a.activity_stats
                    .activity_score
                    .partial_cmp(&b.activity_stats.activity_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        insights.push(ProfileInsight {
            insight_type: InsightType::TopPerformer,
            description: format!(
                "{} has the highest activity score at {:.1}",
                top.name, top.activity_stats.activity_score
            ),
            affected_salespersons: vec![top.salesperson_id.clone()],
            priority: Priority::High,
        });

        // Identify salespersons with low activity
        for profile in self.profiles.values() {
            if profile.activity_stats.activity_score < 30.0 {
                insights.push(ProfileInsight {
                    insight_type: InsightType::NeedsCoaching,
                    description: format!(
                        "{} has low activity score ({:.1}), may benefit from coaching",
                        profile.name, profile.activity_stats.activity_score
                    ),
                    affected_salespersons: vec![profile.salesperson_id.clone()],
                    priority: Priority::Medium,
                });
            }
        }

        // Identify multilingual experts
        for profile in self.profiles.values() {
            if profile.language_preference.multilingual_score > 0.5 {
                insights.push(ProfileInsight {
                    insight_type: InsightType::LanguageExpert,
                    description: format!(
                        "{} is multilingual (score: {:.2}), can handle diverse customers",
                        profile.name, profile.language_preference.multilingual_score
                    ),
                    affected_salespersons: vec![profile.salesperson_id.clone()],
                    priority: Priority::Medium,
                });
            }
        }

        // Low retention rate
        for profile in self.profiles.values() {
            if profile.communication_pattern.customer_retention_rate < 0.5
                && profile.communication_pattern.customer_retention_rate > 0.0
            {
                insights.push(ProfileInsight {
                    insight_type: InsightType::CustomerRetention,
                    description: format!(
                        "{} has low customer retention ({:.0}%), review follow-up patterns",
                        profile.name,
                        profile.communication_pattern.customer_retention_rate * 100.0
                    ),
                    affected_salespersons: vec![profile.salesperson_id.clone()],
                    priority: Priority::High,
                });
            }
        }

        // Channel diversity gap
        if self.profiles.len() >= 2 {
            let channel_variance = self.channel_diversity_variance();
            if channel_variance < 0.01 {
                let ids: Vec<String> = self.profiles.keys().cloned().collect();
                insights.push(ProfileInsight {
                    insight_type: InsightType::CommunicationGap,
                    description: "All salespersons use the same channel, consider diversifying"
                        .to_string(),
                    affected_salespersons: ids,
                    priority: Priority::Low,
                });
            }
        }

        insights
    }

    // ── Private helpers ──────────────────────────────────────────────

    fn compute_top_customers(
        &self,
        whatsapp_chats: &[ChatRecord],
        emails: &[EmailRecord],
        customers: &[Customer],
        salesperson_id: &str,
    ) -> Vec<TopCustomer> {
        let mut customer_interactions: HashMap<String, u32> = HashMap::new();

        for chat in whatsapp_chats {
            if chat.sender == salesperson_id {
                continue;
            }
            *customer_interactions
                .entry(chat.sender.clone())
                .or_insert(0) += 1;
        }

        for email in emails {
            if email.sender == salesperson_id {
                continue;
            }
            *customer_interactions
                .entry(email.sender.clone())
                .or_insert(0) += 1;
        }

        let mut top: Vec<TopCustomer> = customer_interactions
            .into_iter()
            .map(|(name, count)| {
                let cid = customers
                    .iter()
                    .find(|c| c.name == name)
                    .map(|c| c.customer_id.clone())
                    .unwrap_or_else(|| name.clone());
                TopCustomer {
                    customer_id: cid,
                    customer_name: name,
                    interaction_count: count,
                    total_value: 0.0,
                    relationship_duration_days: 0,
                }
            })
            .collect();

        top.sort_by(|a, b| b.interaction_count.cmp(&a.interaction_count));
        top.truncate(10);
        top
    }

    fn compute_activity_stats(
        &self,
        whatsapp_chats: &[ChatRecord],
        emails: &[EmailRecord],
        customers: &[Customer],
        salesperson_id: &str,
    ) -> ActivityStats {
        let wa_chats = whatsapp_chats
            .iter()
            .filter(|c| c.sender == salesperson_id)
            .count() as u32;
        let email_count = emails
            .iter()
            .filter(|e| e.sender == salesperson_id)
            .count() as u32;

        let unique: u32 = whatsapp_chats
            .iter()
            .filter(|c| c.sender != salesperson_id)
            .map(|c| &c.sender)
            .chain(
                emails
                    .iter()
                    .filter(|e| e.sender != salesperson_id)
                    .map(|e| &e.sender),
            )
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;

        let activity_score =
            ((wa_chats as f32 * 2.0) + (email_count as f32 * 3.0)).min(100.0);

        ActivityStats {
            total_whatsapp_chats: wa_chats,
            total_emails: email_count,
            total_phone_calls: 0,
            total_meetings: 0,
            unique_customers: unique.max(customers.len() as u32),
            gonghai_customers: 0,
            activity_score,
        }
    }

    fn channel_diversity_variance(&self) -> f32 {
        let channels: Vec<&str> = self
            .profiles
            .values()
            .map(|p| p.communication_pattern.preferred_channel.as_str())
            .collect();
        if channels.is_empty() {
            return 0.0;
        }
        let mut counts: HashMap<&str, u32> = HashMap::new();
        for ch in &channels {
            *counts.entry(ch).or_insert(0) += 1;
        }
        let n = channels.len() as f32;
        let mean = n / counts.len() as f32;
        counts
            .values()
            .map(|c| {
                let diff = *c as f32 - mean;
                diff * diff
            })
            .sum::<f32>()
            / n
    }
}

// ── Language detection (heuristic) ───────────────────────────────────────

/// Simple heuristic language detection.
///
/// Uses character range analysis to classify messages into broad language
/// categories. This is a placeholder — real implementation should use a
/// dedicated language detection library (e.g., `lingua` or `whatlang`).
fn detect_message_language(msg: &str) -> String {
    let mut cjk = 0u32;
    let mut latin = 0u32;
    let mut arabic = 0u32;
    let mut cyrillic = 0u32;

    for ch in msg.chars() {
        let cp = ch as u32;
        match cp {
            0x4E00..=0x9FFF | 0x3400..=0x4DBF => cjk += 1,
            0x0041..=0x007A | 0x00C0..=0x024F => latin += 1,
            0x0600..=0x06FF => arabic += 1,
            0x0400..=0x04FF => cyrillic += 1,
            _ => {}
        }
    }

    let total = (cjk + latin + arabic + cyrillic) as f32;
    if total == 0.0 {
        return "unknown".to_string();
    }

    if cjk as f32 / total > 0.3 {
        "zh".to_string()
    } else if arabic as f32 / total > 0.3 {
        "ar".to_string()
    } else if cyrillic as f32 / total > 0.3 {
        "ru".to_string()
    } else {
        "en".to_string()
    }
}

fn is_emoji(c: char) -> bool {
    matches!(c as u32,
        0x1F600..=0x1F64F |
        0x1F300..=0x1F5FF |
        0x1F680..=0x1F6FF |
        0x1F900..=0x1F9FF |
        0x2600..=0x26FF |
        0x2700..=0x27BF
    )
}

fn estimate_formality(messages: &[String]) -> f32 {
    if messages.is_empty() {
        return 0.5;
    }

    let formal_signals = ["dear", "regards", "sincerely", "kindly", "please", "尊敬的", "此致", "敬上"];
    let informal_signals = ["hey", "hi", "lol", "omg", "haha", "哈哈", "嘻嘻"];

    let mut formal_count = 0u32;
    let mut informal_count = 0u32;

    for msg in messages {
        let lower = msg.to_lowercase();
        for signal in &formal_signals {
            if lower.contains(signal) {
                formal_count += 1;
            }
        }
        for signal in &informal_signals {
            if lower.contains(signal) {
                informal_count += 1;
            }
        }
    }

    let total = formal_count + informal_count;
    if total == 0 {
        return 0.5;
    }

    formal_count as f32 / total as f32
}

fn extract_common_phrases(messages: &[String]) -> Vec<String> {
    let mut word_counts: HashMap<String, u32> = HashMap::new();

    for msg in messages {
        let lower = msg.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        for window in words.windows(3) {
            let phrase = window.join(" ");
            if phrase.len() > 8 {
                *word_counts.entry(phrase).or_insert(0) += 1;
            }
        }
    }

    let mut phrases: Vec<(String, u32)> = word_counts.into_iter().collect();
    phrases.sort_by(|a, b| b.1.cmp(&a.1));
    phrases.into_iter().take(5).map(|(p, _)| p).collect()
}

fn classify_follow_up_frequency(interactions: &[Interaction]) -> FollowUpFrequency {
    if interactions.len() <= 1 {
        return FollowUpFrequency::Never;
    }

    let mut sorted = interactions.to_vec();
    sorted.sort_by_key(|i| i.timestamp);

    let mut gaps: Vec<i64> = Vec::new();
    for window in sorted.windows(2) {
        let gap = window[1].timestamp - window[0].timestamp;
        if gap > 0 {
            gaps.push(gap);
        }
    }

    if gaps.is_empty() {
        return FollowUpFrequency::Never;
    }

    let avg_gap_hours = gaps.iter().sum::<i64>() as f32 / gaps.len() as f32 / 3600.0;

    if avg_gap_hours < 24.0 {
        FollowUpFrequency::MultipleDaily
    } else if avg_gap_hours < 168.0 {
        FollowUpFrequency::Daily
    } else if avg_gap_hours < 720.0 {
        FollowUpFrequency::Weekly
    } else {
        FollowUpFrequency::Rarely
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chat(message: &str, sender: &str, ts: i64) -> ChatRecord {
        ChatRecord {
            message: message.to_string(),
            sender: sender.to_string(),
            timestamp: ts,
        }
    }

    fn make_email(subject: &str, body: &str, sender: &str, ts: i64) -> EmailRecord {
        EmailRecord {
            subject: subject.to_string(),
            body: body.to_string(),
            sender: sender.to_string(),
            timestamp: ts,
        }
    }

    #[test]
    fn test_new_profiler_starts_empty() {
        let profiler = SalespersonProfiler::new();
        assert!(profiler.profiles.is_empty());
    }

    #[test]
    fn test_analyze_writing_style_empty() {
        let profiler = SalespersonProfiler::new();
        let style = profiler.analyze_writing_style(&[]);
        assert_eq!(style.avg_message_length, 0);
        assert_eq!(style.emoji_usage, 0.0);
        assert_eq!(style.question_ratio, 0.0);
    }

    #[test]
    fn test_analyze_writing_style_with_messages() {
        let profiler = SalespersonProfiler::new();
        let messages = vec![
            "Hello, how are you?".to_string(),
            "This is a longer formal message with regards".to_string(),
            "hey lol 😂".to_string(),
        ];

        let style = profiler.analyze_writing_style(&messages);
        assert!(style.avg_message_length > 0);
        assert!(style.emoji_usage > 0.0);
        assert!(style.question_ratio > 0.0);
        assert_eq!(style.templates.len(), 3); // 3-word phrases extracted
    }

    #[test]
    fn test_detect_language_preference_empty() {
        let profiler = SalespersonProfiler::new();
        let pref = profiler.detect_language_preference(&[]);
        assert_eq!(pref.primary_language, "unknown");
        assert_eq!(pref.multilingual_score, 0.0);
    }

    #[test]
    fn test_detect_language_preference_chinese() {
        let profiler = SalespersonProfiler::new();
        let messages = vec![
            "你好，今天的订单怎么样？".to_string(),
            "请确认一下发货时间".to_string(),
        ];
        let pref = profiler.detect_language_preference(&messages);
        assert_eq!(pref.primary_language, "zh");
    }

    #[test]
    fn test_detect_language_preference_english() {
        let profiler = SalespersonProfiler::new();
        let messages = vec![
            "Hello, how are you?".to_string(),
            "The order has been confirmed".to_string(),
        ];
        let pref = profiler.detect_language_preference(&messages);
        assert_eq!(pref.primary_language, "en");
    }

    #[test]
    fn test_detect_language_preference_multilingual() {
        let profiler = SalespersonProfiler::new();
        let messages = vec![
            "Hello, how are you?".to_string(),
            "你好，你好吗？".to_string(),
            "The order has been confirmed".to_string(),
            "请确认一下发货时间".to_string(),
        ];
        let pref = profiler.detect_language_preference(&messages);
        assert!(pref.multilingual_score > 0.0);
        assert_eq!(pref.secondary_languages.len(), 1);
    }

    #[test]
    fn test_model_communication_pattern_empty() {
        let profiler = SalespersonProfiler::new();
        let pattern = profiler.model_communication_pattern(&[]);
        assert_eq!(pattern.preferred_channel, "unknown");
    }

    #[test]
    fn test_model_communication_pattern_with_interactions() {
        let profiler = SalespersonProfiler::new();
        let interactions = vec![
            Interaction {
                interaction_type: "whatsapp".to_string(),
                timestamp: 1000,
                customer_id: "c1".to_string(),
            },
            Interaction {
                interaction_type: "whatsapp".to_string(),
                timestamp: 2000,
                customer_id: "c2".to_string(),
            },
            Interaction {
                interaction_type: "email".to_string(),
                timestamp: 3000,
                customer_id: "c1".to_string(),
            },
        ];

        let pattern = profiler.model_communication_pattern(&interactions);
        assert_eq!(pattern.preferred_channel, "whatsapp");
        assert!(pattern.avg_interactions_per_customer > 1.0);
    }

    #[test]
    fn test_generate_profile_creates_and_stores() {
        let mut profiler = SalespersonProfiler::new();

        let chats = vec![
            make_chat("Hello!", "sp1", 1000),
            make_chat("Hey there 😊", "sp1", 2000),
            make_chat("Any questions?", "other", 1500),
        ];

        let emails = vec![make_email("RE: Order", "Thanks for the order", "sp1", 3000)];

        let customers = vec![Customer {
            customer_id: "c1".to_string(),
            name: "other".to_string(),
        }];

        let profile =
            profiler.generate_profile("sp1", "Alice", &chats, &emails, &customers);

        assert_eq!(profile.salesperson_id, "sp1");
        assert_eq!(profile.name, "Alice");
        assert!(profile.writing_style.avg_message_length > 0);
        assert_eq!(profile.activity_stats.total_whatsapp_chats, 2);
        assert_eq!(profile.activity_stats.total_emails, 1);

        // Stored and retrievable
        let stored = profiler.get_profile("sp1").unwrap();
        assert_eq!(stored.salesperson_id, "sp1");
    }

    #[test]
    fn test_get_profile_missing() {
        let profiler = SalespersonProfiler::new();
        assert!(profiler.get_profile("nonexistent").is_none());
    }

    #[test]
    fn test_compare_profiles() {
        let mut profiler = SalespersonProfiler::new();

        let chats1 = vec![make_chat("Hello!", "sp1", 1000)];
        let chats2 = vec![make_chat("Hey 😊", "sp2", 1000)];

        profiler.generate_profile("sp1", "Alice", &chats1, &[], &[]);
        profiler.generate_profile("sp2", "Bob", &chats2, &[], &[]);

        let comparison = profiler.compare_profiles("sp1", "sp2").unwrap();
        assert_eq!(comparison.salesperson1, "Alice");
        assert_eq!(comparison.salesperson2, "Bob");
        assert!(comparison.similarities.len() + comparison.differences.len() > 0);
    }

    #[test]
    fn test_compare_profiles_missing() {
        let profiler = SalespersonProfiler::new();
        assert!(profiler.compare_profiles("a", "b").is_none());
    }

    #[test]
    fn test_generate_insights_empty() {
        let profiler = SalespersonProfiler::new();
        let insights = profiler.generate_insights();
        assert!(insights.is_empty());
    }

    #[test]
    fn test_generate_insights_top_performer() {
        let mut profiler = SalespersonProfiler::new();

        // High activity
        let mut high_activity_chats: Vec<ChatRecord> = (0..30)
            .map(|i| make_chat("msg", "sp1", 1000 + i))
            .collect();
        high_activity_chats.push(make_chat("hey", "customer", 2000));

        let profile1 = profiler.generate_profile(
            "sp1",
            "Alice",
            &high_activity_chats,
            &[],
            &[Customer {
                customer_id: "c1".to_string(),
                name: "customer".to_string(),
            }],
        );
        assert!(profile1.activity_stats.activity_score > 0.0);

        // Low activity
        let low_chats = vec![make_chat("hi", "sp2", 1000)];
        profiler.generate_profile("sp2", "Bob", &low_chats, &[], &[]);

        let insights = profiler.generate_insights();
        assert!(insights.iter().any(|i| i.insight_type == InsightType::TopPerformer));
    }

    #[test]
    fn test_emoji_detection() {
        assert!(is_emoji('😀'));
        assert!(is_emoji('🎉'));
        assert!(!is_emoji('a'));
        assert!(!is_emoji(' '));
    }

    #[test]
    fn test_language_detection_heuristic() {
        assert_eq!(detect_message_language("Hello world"), "en");
        assert_eq!(detect_message_language("你好世界"), "zh");
        assert_eq!(detect_message_language(""), "unknown");
    }

    #[test]
    fn test_formality_estimation() {
        let profiler = SalespersonProfiler::new();
        let formal = profiler.analyze_writing_style(&[
            "Dear Sir, regards".to_string(),
        ]);
        assert!(formal.formality_level > 0.5);

        let informal = profiler.analyze_writing_style(&[
            "hey lol haha".to_string(),
        ]);
        assert!(informal.formality_level < 0.5);
    }

    #[test]
    fn test_follow_up_frequency() {
        let interactions = vec![
            Interaction {
                interaction_type: "whatsapp".to_string(),
                timestamp: 1000,
                customer_id: "c1".to_string(),
            },
            Interaction {
                interaction_type: "whatsapp".to_string(),
                timestamp: 3700, // ~1 hour gap
                customer_id: "c2".to_string(),
            },
            Interaction {
                interaction_type: "whatsapp".to_string(),
                timestamp: 7300, // ~1 hour gap
                customer_id: "c1".to_string(),
            },
        ];

        let profiler = SalespersonProfiler::new();
        let pattern = profiler.model_communication_pattern(&interactions);
        assert_eq!(pattern.follow_up_frequency, FollowUpFrequency::MultipleDaily);
    }

    #[test]
    fn test_top_customers_ranked() {
        let mut profiler = SalespersonProfiler::new();

        let chats = vec![
            make_chat("hi", "alice", 1000),
            make_chat("hi", "alice", 1100),
            make_chat("hi", "alice", 1200),
            make_chat("hi", "bob", 1300),
        ];

        let customers = vec![
            Customer {
                customer_id: "c1".to_string(),
                name: "alice".to_string(),
            },
            Customer {
                customer_id: "c2".to_string(),
                name: "bob".to_string(),
            },
        ];

        let profile = profiler.generate_profile("sp1", "Sales", &chats, &[], &customers);
        assert_eq!(profile.top_customers.len(), 2);
        assert_eq!(profile.top_customers[0].customer_name, "alice");
        assert_eq!(profile.top_customers[0].interaction_count, 3);
    }
}

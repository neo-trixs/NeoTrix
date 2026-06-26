#[derive(Debug, Clone, Default)]
pub struct DisinfoScanReport {
    pub emotional_load: f64,
    pub speed_verification_asymmetry: f64,
    pub source_homology: Vec<String>,
    pub media_integrity_score: f64,
    pub narrative_conflicts: Vec<String>,
    pub overall_suspicion: f64,
    pub details: String,
}

#[derive(Debug, Clone, Default)]
pub struct DisinfoScanner {
    emotional_marker_words: Vec<&'static str>,
    moralizing_words: Vec<&'static str>,
    swearing_words: Vec<&'static str>,
    netspeak_markers: Vec<&'static str>,
    certainty_words: Vec<&'static str>,
    conspiracy_triggers: Vec<&'static str>,
}

impl DisinfoScanner {
    pub fn new() -> Self {
        Self {
            emotional_marker_words: vec![
                "outrage",
                "shocking",
                "unbelievable",
                "appalling",
                "disgusting",
                "horrifying",
                "terrifying",
                "heartbreaking",
                "devastating",
                "mind-blowing",
                "jaw-dropping",
                "you won't believe",
                "must see",
                "spread the word",
                "share if you agree",
            ],
            moralizing_words: vec![
                "evil",
                "pure",
                "corrupt",
                "virtuous",
                "sinful",
                "righteous",
                "immoral",
                "deplorable",
                "heroic",
                "traitor",
                "patriotic",
                "un-american",
                "unpatriotic",
                "treason",
            ],
            swearing_words: vec![
                "damn", "hell", "idiot", "stupid", "moron", "criminal", "scam", "fraud", "liar",
                "cheat",
            ],
            netspeak_markers: vec![
                "!!!",
                "???",
                "lol",
                "smh",
                "omg",
                "wtf",
                "tldr",
                "!!!!",
                "????",
                "!!?!",
                "share this",
                "copy and paste",
                "forward this",
                "type amen",
            ],
            certainty_words: vec![
                "definitely",
                "undoubtedly",
                "absolutely",
                "without question",
                "guaranteed",
                "certainly",
                "indisputably",
                "irrefutably",
                "proven beyond doubt",
                "conclusively",
                "undeniable",
                "of course",
                "obviously",
                "clearly",
            ],
            conspiracy_triggers: vec![
                "they don't want you to know",
                "mainstream media won't tell you",
                "what the government hides",
                "wake up people",
                "do your own research",
                "the truth about",
                "hidden agenda",
                "cover up",
                "controlled opposition",
                "sheeple",
                "deep state",
                "they are hiding",
                "censored",
                "banned",
                "suppressed",
                "secret knowledge",
            ],
        }
    }

    /// 1. Emotional Load Analysis — measures emotional vocabulary density + valence extremity + moralizing language
    pub fn analyze_emotional_load(&self, text: &str) -> f64 {
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        let total = words.len();
        if total == 0 {
            return 0.0;
        }

        let mut emo_hits = 0;
        let mut moral_hits = 0;
        let mut swear_hits = 0;
        let mut net_hits = 0;

        for word in &words {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if self.emotional_marker_words.iter().any(|w| *w == clean) {
                emo_hits += 1;
            }
            if self.moralizing_words.iter().any(|w| *w == clean) {
                moral_hits += 1;
            }
            if self.swearing_words.iter().any(|w| *w == clean) {
                swear_hits += 1;
            }
        }

        // Check for netspeak patterns
        for marker in &self.netspeak_markers {
            if lower.contains(marker) {
                net_hits += 1;
            }
        }

        let density = (emo_hits + moral_hits + swear_hits + net_hits) as f64 / total as f64;
        // Normalize to 0..1 scale
        (density * 5.0).min(1.0)
    }

    /// 2. Speed-Verification Asymmetry — detect urgency markers that bypass verification
    pub fn analyze_urgency(&self, text: &str) -> f64 {
        let lower = text.to_lowercase();
        let mut urgency_score = 0.0;

        let urgency_triggers = [
            "breaking",
            "urgent",
            "immediate",
            "alert",
            "warning",
            "just in",
            "developing story",
            "now",
            "happening right now",
            "minutes ago",
            "moments ago",
            "just released",
            "confirmed",
            "shocking news",
            "must read now",
            "act now",
        ];

        for trigger in &urgency_triggers {
            if lower.contains(trigger) {
                urgency_score += 0.15;
            }
        }

        // Punctuation urgency signals
        let exclamation_count = text.matches('!').count();
        let question_count = text.matches('?').count();
        urgency_score += (exclamation_count as f64) * 0.02;
        urgency_score += (question_count as f64) * 0.01;

        urgency_score.min(1.0)
    }

    /// 3. Source Homology — detect if multiple claims share identical phrasing patterns
    pub fn detect_source_homology(&self, texts: &[&str]) -> Vec<String> {
        let mut similarities = Vec::new();
        if texts.len() < 2 {
            return similarities;
        }

        for i in 0..texts.len() {
            for j in (i + 1)..texts.len() {
                let a = texts[i].to_lowercase();
                let b = texts[j].to_lowercase();
                // Check for shared unique phrases (>5 words identical)
                let a_words: Vec<&str> = a.split_whitespace().collect();
                let b_words: Vec<&str> = b.split_whitespace().collect();
                let shared = a_words.iter().filter(|w| b_words.contains(w)).count();
                let total = a_words.len().max(b_words.len());
                if total > 0 {
                    let ratio = shared as f64 / total as f64;
                    if ratio > 0.7 {
                        similarities.push(format!(
                            "sources {} and {} share {:.0}% phrasing",
                            i,
                            j,
                            ratio * 100.0
                        ));
                    }
                }
            }
        }
        similarities
    }

    /// 4. Media Integrity Score — checks for manipulation indicators
    pub fn analyze_media_integrity(&self, text: &str) -> f64 {
        let lower = text.to_lowercase();
        let mut issues: f64 = 0.0;

        // Deepfake/text pattern indicators
        let manipulation_patterns = [
            "screenshot shows",
            "leaked image",
            "alleged",
            "reportedly",
            "according to sources",
            "unnamed sources",
            "anonymous source",
            "unconfirmed",
            "unverified",
            "rumored",
            "speculation",
        ];

        for pattern in &manipulation_patterns {
            if lower.contains(pattern) {
                issues += 0.12;
            }
        }

        // Conspiracy framing
        for trigger in &self.conspiracy_triggers {
            if lower.contains(trigger) {
                issues += 0.20;
            }
        }

        // Certainty overclaim
        for word in &self.certainty_words {
            if lower.contains(word) {
                issues += 0.08;
            }
        }

        (1.0 - issues.min(1.0)).max(0.0)
    }

    /// 5. Narrative Conflict Detection — simple contradiction pattern detection
    pub fn detect_narrative_conflicts(&self, statements: &[&str]) -> Vec<String> {
        let mut conflicts = Vec::new();
        if statements.len() < 2 {
            return conflicts;
        }

        let pairs: Vec<(&str, &str)> = vec![
            ("increase", "decrease"),
            ("rise", "fall"),
            ("up", "down"),
            ("more", "less"),
            ("higher", "lower"),
            ("positive", "negative"),
            ("support", "oppose"),
            ("win", "lose"),
            ("gain", "loss"),
            ("success", "failure"),
            ("truth", "lie"),
            ("safe", "dangerous"),
            ("good", "bad"),
            ("proven", "disproven"),
        ];

        for i in 0..statements.len() {
            for (a, b) in &pairs {
                let has_a = statements[i].to_lowercase().contains(a);
                let _has_b = statements[i].to_lowercase().contains(b);

                if has_a {
                    for j in 0..statements.len() {
                        if i != j && statements[j].to_lowercase().contains(b) {
                            conflicts.push(format!(
                                "Contradictory framing: '{}' vs '{}' between statements {} and {}",
                                a, b, i, j
                            ));
                        }
                    }
                }
            }
        }

        conflicts.dedup();
        conflicts
    }

    /// Full scan across all 5 dimensions
    pub fn scan_full(&self, text: &str, related_texts: &[&str]) -> DisinfoScanReport {
        let emotional_load = self.analyze_emotional_load(text);
        let urgency = self.analyze_urgency(text);
        let speed_verification_asymmetry = urgency;
        let source_homology = self.detect_source_homology(related_texts);
        let media_integrity_score = self.analyze_media_integrity(text);
        let narrative_conflicts = self.detect_narrative_conflicts(related_texts);

        // Overall suspicion: weighted combination
        let overall = emotional_load * 0.25
            + speed_verification_asymmetry * 0.20
            + (1.0 - media_integrity_score) * 0.25
            + (if source_homology.is_empty() {
                0.0
            } else {
                0.15
            })
            + (if narrative_conflicts.is_empty() {
                0.0
            } else {
                0.15
            });

        let details =
            format!(
            "emo_load={:.2} urgency={:.2} media_int={:.2} homology={} conflicts={} overall={:.2}",
            emotional_load, speed_verification_asymmetry, media_integrity_score,
            source_homology.len(), narrative_conflicts.len(), overall
        );

        DisinfoScanReport {
            emotional_load,
            speed_verification_asymmetry,
            source_homology,
            media_integrity_score,
            narrative_conflicts,
            overall_suspicion: overall,
            details,
        }
    }

    pub fn summary(&self, report: &DisinfoScanReport) -> String {
        if report.overall_suspicion < 0.3 {
            format!(
                "✓ Low disinfo risk ({:.1}%)",
                report.overall_suspicion * 100.0
            )
        } else if report.overall_suspicion < 0.6 {
            format!(
                "⚠ Medium disinfo risk ({:.1}%)",
                report.overall_suspicion * 100.0
            )
        } else {
            format!(
                "✗ High disinfo risk ({:.1}%)",
                report.overall_suspicion * 100.0
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_risk_text() {
        let d = DisinfoScanner::new();
        let r = d.scan_full("The temperature today is 22 degrees Celsius, as reported by the meteorological office.", &[]);
        assert!(r.overall_suspicion < 0.3);
    }

    #[test]
    fn test_high_emotional_load() {
        let d = DisinfoScanner::new();
        let load = d.analyze_emotional_load("This is an OUTRAGE! The corrupt government is hiding the TRUTH from us!!! WAKE UP PEOPLE!!!");
        assert!(load > 0.3);
    }

    #[test]
    fn test_source_homology() {
        let d = DisinfoScanner::new();
        let texts = vec![
            "The government is hiding the truth about the weather",
            "The government is hiding the truth about the economy",
            "Different text entirely about sports",
        ];
        let homology = d.detect_source_homology(&texts);
        assert!(!homology.is_empty());
    }

    #[test]
    fn test_narrative_conflict() {
        let d = DisinfoScanner::new();
        let statements = vec![
            "The economy is showing strong growth this quarter",
            "Reports indicate a significant decrease in economic activity",
        ];
        let conflicts = d.detect_narrative_conflicts(&statements);
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_urgency_detection() {
        let d = DisinfoScanner::new();
        let urgency = d.analyze_urgency("BREAKING NEWS: Urgent alert!!!");
        assert!(urgency > 0.3);
    }
}

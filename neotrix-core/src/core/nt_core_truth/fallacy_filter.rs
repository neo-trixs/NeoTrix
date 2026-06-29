#[derive(Debug, Clone, PartialEq)]
pub enum FallacySeverity {
    Critical, // P0: cognitive manipulation - blocks further processing
    High,     // P1: logical structure - argument invalidated
    Medium,   // P2: evidence relationship - conclusion unsupported
    Low,      // P3: focus diversion - distracts but doesn't invalidate
}

#[derive(Debug, Clone)]
pub struct FallacyPattern {
    pub name: &'static str,
    pub severity: FallacySeverity,
    pub priority: u8, // P0=0, P1=1, P2=2, P3=3
    pub description: &'static str,
    pub triggers: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct FallacyHit {
    pub pattern_name: String,
    pub severity: FallacySeverity,
    pub priority: u8,
    pub description: String,
    pub trigger: String,
    pub position: usize,
    pub confidence: f64,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub struct FallacyReport {
    pub hits: Vec<FallacyHit>,
    pub blocked: bool,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct FallacyFilter {
    pub patterns: Vec<FallacyPattern>,
}

impl Default for FallacyFilter {
    fn default() -> Self {
        Self::new()
    }
}

fn structural_hit(
    fallacy_type: &'static str,
    severity: FallacySeverity,
    priority: u8,
    trigger_text: String,
    position: usize,
    confidence: f64,
    description: &'static str,
    explanation: String,
) -> FallacyHit {
    FallacyHit {
        pattern_name: fallacy_type.to_string(),
        severity,
        priority,
        description: description.to_string(),
        trigger: trigger_text,
        position,
        confidence,
        explanation,
    }
}

impl FallacyFilter {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    fn default_patterns() -> Vec<FallacyPattern> {
        vec![
            // P0 — Cognitive manipulation (most dangerous)
            FallacyPattern {
                name: "appeal_to_emotion",
                severity: FallacySeverity::Critical,
                priority: 0,
                description: "Uses emotional manipulation instead of evidence",
                triggers: vec![
                    "think of the children",
                    "how dare you",
                    "if you cared",
                    "heartbreaking",
                    "sickening",
                    "outrage",
                    "this will make you",
                    "you must feel",
                    "any decent person",
                    "imagine if it was your",
                    "would you want",
                ],
            },
            FallacyPattern {
                name: "appeal_to_fear",
                severity: FallacySeverity::Critical,
                priority: 0,
                description: "Uses fear to bypass rational evaluation",
                triggers: vec![
                    "you should be afraid",
                    "this is terrifying",
                    "nightmare scenario",
                    "inevitable disaster",
                    "we are all at risk",
                    "imminent threat",
                    "will destroy",
                    "end of",
                    "catastrophic consequences",
                ],
            },
            FallacyPattern {
                name: "straw_man",
                severity: FallacySeverity::Critical,
                priority: 0,
                description: "Misrepresents an opposing position to make it easier to attack",
                triggers: vec![
                    "they want us to believe",
                    "opponents claim that",
                    "critics say",
                    "the other side thinks",
                ],
            },
            FallacyPattern {
                name: "loaded_question",
                severity: FallacySeverity::Critical,
                priority: 0,
                description: "Question contains an unjustified presupposition",
                triggers: vec![
                    "have you stopped",
                    "why do you always",
                    "don't you think",
                    "isn't it obvious",
                    "when will you admit",
                ],
            },
            FallacyPattern {
                name: "bandwagon",
                severity: FallacySeverity::Critical,
                priority: 0,
                description: "Argues popularity is evidence of truth",
                triggers: vec![
                    "everyone knows",
                    "millions of people believe",
                    "growing consensus",
                    "most people agree",
                    "widely accepted",
                    "increasingly popular",
                    "join the millions",
                    "everyone is saying",
                ],
            },
            // P1 — Logical structure
            FallacyPattern {
                name: "false_dilemma",
                severity: FallacySeverity::High,
                priority: 1,
                description: "Presents limited options as the only possibilities",
                triggers: vec![
                    "either you're with us or",
                    "there are only two options",
                    "if not this then",
                    "it's either",
                    "the only choice is",
                    "you're either part of the solution or",
                ],
            },
            FallacyPattern {
                name: "circular_reasoning",
                severity: FallacySeverity::High,
                priority: 1,
                description: "Conclusion appears as a premise",
                triggers: vec![
                    "it is true because",
                    "we know it's",
                    "proven by the fact that",
                ],
            },
            FallacyPattern {
                name: "non_sequitur",
                severity: FallacySeverity::High,
                priority: 1,
                description: "Conclusion does not follow from premises",
                triggers: vec!["therefore clearly", "this proves that", "it follows that"],
            },
            // P2 — Evidence relationship
            FallacyPattern {
                name: "hasty_generalization",
                severity: FallacySeverity::Medium,
                priority: 2,
                description: "General conclusion from insufficient evidence",
                triggers: vec![
                    "all politicians",
                    "every single",
                    "always happens",
                    "never works",
                    "everyone",
                    "no one",
                    "always",
                    "never",
                ],
            },
            FallacyPattern {
                name: "slippery_slope",
                severity: FallacySeverity::Medium,
                priority: 2,
                description: "Claims a small step inevitably leads to extreme outcome",
                triggers: vec![
                    "slippery slope",
                    "thin end of the wedge",
                    "first they came for",
                    "next thing you know",
                    "it's only a matter of time before",
                    "will lead to",
                    "opens the door to",
                ],
            },
            FallacyPattern {
                name: "post_hoc_ergo_propter_hoc",
                severity: FallacySeverity::Medium,
                priority: 2,
                description: "Assumes temporal sequence implies causation",
                triggers: vec![
                    "after this happened",
                    "since this event",
                    "ever since",
                    "coincidence that",
                    "right after",
                ],
            },
            FallacyPattern {
                name: "appeal_to_ignorance",
                severity: FallacySeverity::Medium,
                priority: 2,
                description: "Claims something is true because it hasn't been proven false",
                triggers: vec![
                    "no one has proven",
                    "cannot prove it doesn't",
                    "no evidence against",
                    "hasn't been disproven",
                    "not proven false",
                ],
            },
            // P3 — Focus diversion
            FallacyPattern {
                name: "ad_hominem",
                severity: FallacySeverity::Low,
                priority: 3,
                description: "Attacks the person instead of the argument",
                triggers: vec![
                    "you're just",
                    "of course you would say",
                    "typical of",
                    "don't listen to",
                    "what do you expect from",
                ],
            },
            FallacyPattern {
                name: "whataboutism",
                severity: FallacySeverity::Low,
                priority: 3,
                description: "Deflects criticism by raising a different issue",
                triggers: vec![
                    "what about",
                    "but what about",
                    "and yet",
                    "focus on",
                    "don't forget about",
                ],
            },
            FallacyPattern {
                name: "red_herring",
                severity: FallacySeverity::Low,
                priority: 3,
                description: "Introduces irrelevant topic to divert attention",
                triggers: vec!["that reminds me", "speaking of", "more importantly"],
            },
            FallacyPattern {
                name: "appeal_to_authority_irrelevant",
                severity: FallacySeverity::Low,
                priority: 3,
                description: "Appeals to authority outside their domain",
                triggers: vec![
                    "according to celebrity",
                    "famous person said",
                    "millionaire believes",
                    "influencer says",
                ],
            },
        ]
    }

    pub fn scan(&self, text: &str) -> FallacyReport {
        let lower = text.to_lowercase();
        let mut hits = Vec::new();

        for pattern in &self.patterns {
            for trigger in &pattern.triggers {
                if let Some(pos) = lower.find(trigger) {
                    hits.push(FallacyHit {
                        pattern_name: pattern.name.to_string(),
                        severity: pattern.severity.clone(),
                        priority: pattern.priority,
                        description: pattern.description.to_string(),
                        trigger: trigger.to_string(),
                        position: pos,
                        confidence: 1.0,
                        explanation: pattern.description.to_string(),
                    });
                    break;
                }
            }
        }

        // Structural detections beyond keyword matching
        if let Some(hit) = self.detect_circular_reasoning(&lower) {
            hits.push(hit);
        }
        if let Some(hit) = self.detect_straw_man(&lower) {
            hits.push(hit);
        }
        if let Some(hit) = self.detect_false_dilemma(&lower) {
            hits.push(hit);
        }
        if let Some(hit) = self.detect_appeal_to_nature(&lower) {
            hits.push(hit);
        }
        if let Some(hit) = self.detect_false_equivalence(&lower) {
            hits.push(hit);
        }

        hits.sort_by_key(|h| h.position);

        let has_critical = hits.iter().any(|h| h.severity == FallacySeverity::Critical);
        let blocked = has_critical;
        let reasoning = if blocked {
            format!(
                "Blocked: {} critical fallacy(es) detected (P0)",
                hits.iter()
                    .filter(|h| h.severity == FallacySeverity::Critical)
                    .count()
            )
        } else if hits.is_empty() {
            "No fallacies detected".into()
        } else {
            let names: Vec<&str> = hits.iter().map(|h| h.pattern_name.as_str()).collect();
            format!("Warning: fallacies detected: {}", names.join(", "))
        };

        FallacyReport {
            hits,
            blocked,
            reasoning,
        }
    }

    pub fn has_critical_fallacies(&self, text: &str) -> bool {
        self.scan(text).blocked
    }

    pub fn summary(&self, report: &FallacyReport) -> String {
        if report.hits.is_empty() {
            return "\u{2713} No logical fallacies detected".into();
        }
        let by_severity: Vec<_> = report
            .hits
            .iter()
            .map(|h| format!("[{}:{}]", h.severity.clone() as i32, h.pattern_name))
            .collect();
        format!(
            "\u{2717} {} fallacy hits: {}",
            report.hits.len(),
            by_severity.join(" ")
        )
    }

    // ── Structural: Circular Reasoning ──────────────────────────────────

    fn detect_circular_reasoning(&self, lower: &str) -> Option<FallacyHit> {
        // Pattern 1: self-proving assertions with no external evidence
        let self_proof_patterns = [
            ("proves itself", 0.80),
            ("self-evident", 0.75),
            ("self evident", 0.75),
            ("obviously true", 0.65),
            ("by definition true", 0.70),
        ];
        for (pat, conf) in &self_proof_patterns {
            if let Some(pos) = lower.find(pat) {
                return Some(structural_hit(
                    "circular_reasoning",
                    FallacySeverity::High,
                    1,
                    pat.to_string(),
                    pos,
                    *conf,
                    "Conclusion appears as a premise",
                    format!("Self-proof assertion '{}' restates the claim as its own justification without external evidence", pat),
                ));
            }
        }

        // Pattern 2: "X because Y" where Y substantively overlaps with X
        // This detects: "the policy works because it is effective"
        if let Some(because_pos) = lower.find("because") {
            let before = lower[..because_pos].trim();
            let after = lower[because_pos + 7..].trim();

            if !before.is_empty() && !after.is_empty() {
                let before_words: Vec<&str> =
                    before.split_whitespace().filter(|w| w.len() > 3).collect();
                let after_words: Vec<&str> =
                    after.split_whitespace().filter(|w| w.len() > 3).collect();

                if !before_words.is_empty() && !after_words.is_empty() {
                    let overlap = before_words
                        .iter()
                        .filter(|w| after_words.contains(w))
                        .count();
                    let overlap_ratio = overlap as f64 / before_words.len() as f64;

                    // Also check the reverse: how much of the justification is circular
                    let rev_overlap = after_words
                        .iter()
                        .filter(|w| before_words.contains(w))
                        .count();
                    let rev_ratio = rev_overlap as f64 / after_words.len() as f64;

                    if overlap_ratio > 0.45 || rev_ratio > 0.45 {
                        let ctx_start = 0.max(because_pos as isize - 60) as usize;
                        let ctx_end = (because_pos + 7 + after.len()).min(lower.len());
                        let excerpt = &lower[ctx_start..ctx_end];
                        return Some(structural_hit(
                            "circular_reasoning",
                            FallacySeverity::High,
                            1,
                            format!("...{}", excerpt.chars().take(80).collect::<String>()),
                            because_pos,
                            0.55 + (overlap_ratio * 0.30).min(0.40),
                            "Conclusion appears as a premise",
                            format!(
                                "Circular justification: {:.0}% of the claim's significant words are restated in the reason. '{}' relies on '{}' which substantially rephrases the original claim.",
                                overlap_ratio * 100.0,
                                before.chars().take(50).collect::<String>(),
                                after.chars().take(50).collect::<String>(),
                            ),
                        ));
                    }
                }
            }
        }

        // Pattern 3: "it is true because" followed by an assertion that doesn't add independent evidence
        for prefix in &[
            "it is true because",
            "this is true because",
            "that is true because",
        ] {
            if let Some(pos) = lower.find(prefix) {
                let after = lower[pos + prefix.len()..].trim();
                let after_words: Vec<&str> = after.split_whitespace().collect();
                // Short justifications after "it is true because" are likely circular
                if !after.is_empty() && after_words.len() < 20 {
                    if !after.starts_with("of ")
                        && !after.starts_with("studies show")
                        && !after.starts_with("research")
                        && !after.starts_with("evidence")
                    {
                        return Some(structural_hit(
                            "circular_reasoning",
                            FallacySeverity::High,
                            1,
                            format!("{} {}", prefix, after.chars().take(40).collect::<String>()),
                            pos,
                            0.60,
                            "Conclusion appears as a premise",
                            format!("Claim asserted as true with '{}' but the justification '{:.80}' does not provide independent evidence; it restates the claim or relies on bare assertion.", prefix, after),
                        ));
                    }
                }
            }
        }

        // Pattern 4: paired mutual justification — "X because Y" and "Y because X" in the same text
        let because_positions: Vec<usize> =
            lower.match_indices("because").map(|(i, _)| i).collect();
        if because_positions.len() >= 2 {
            for i in 0..because_positions.len() {
                for j in i + 1..because_positions.len() {
                    let claim_a = Self::extract_claim_before(lower, because_positions[i]);
                    let reason_a = Self::extract_reason_after(lower, because_positions[i]);
                    let claim_b = Self::extract_claim_before(lower, because_positions[j]);
                    let reason_b = Self::extract_reason_after(lower, because_positions[j]);

                    if let (Some(ca), Some(ra), Some(cb), Some(rb)) =
                        (claim_a, reason_a, claim_b, reason_b)
                    {
                        let ca_words: Vec<&str> =
                            ca.split_whitespace().filter(|w| w.len() > 3).collect();
                        let rb_words: Vec<&str> =
                            rb.split_whitespace().filter(|w| w.len() > 3).collect();
                        let cb_words: Vec<&str> =
                            cb.split_whitespace().filter(|w| w.len() > 3).collect();
                        let ra_words: Vec<&str> =
                            ra.split_whitespace().filter(|w| w.len() > 3).collect();

                        // Check: claim A overlaps with reason B AND claim B overlaps with reason A
                        let a_in_rb = ca_words.iter().filter(|w| rb_words.contains(w)).count();
                        let b_in_ra = cb_words.iter().filter(|w| ra_words.contains(w)).count();
                        let mutual = (ca_words.len() > 0
                            && a_in_rb as f64 / ca_words.len() as f64 > 0.3)
                            && (cb_words.len() > 0 && b_in_ra as f64 / cb_words.len() as f64 > 0.3);

                        if mutual {
                            return Some(structural_hit(
                                "circular_reasoning",
                                FallacySeverity::High,
                                1,
                                format!("mutual justification at positions {} and {}", because_positions[i], because_positions[j]),
                                because_positions[i],
                                0.75,
                                "Conclusion appears as a premise",
                                "Mutual circular justification detected: claim A is justified by reason B while claim B is justified by reason A, forming a closed logical loop.".into(),
                            ));
                        }
                    }
                }
            }
        }

        None
    }

    fn extract_claim_before(text: &str, because_pos: usize) -> Option<&str> {
        let before = text[..because_pos].trim();
        if before.is_empty() {
            return None;
        }
        // Walk back to sentence boundary or last 100 chars
        let search_start = 0.max(before.len() as isize - 100) as usize;
        let truncated = &before[search_start..];
        if let Some(sentence_end) = truncated.rfind(|c: char| c == '.' || c == '!' || c == '?') {
            Some(truncated[sentence_end + 1..].trim())
        } else {
            Some(truncated.trim())
        }
    }

    fn extract_reason_after(text: &str, because_pos: usize) -> Option<&str> {
        let after = text[because_pos + 7..].trim();
        if after.is_empty() {
            return None;
        }
        // Find end of reason (next sentence boundary or 100 chars max)
        let limit = after.len().min(100);
        let excerpt = &after[..limit];
        if let Some(end) = excerpt.find(|c: char| c == '.' || c == '!' || c == '?') {
            Some(excerpt[..end].trim())
        } else {
            Some(excerpt.trim())
        }
    }

    // ── Structural: Straw Man ───────────────────────────────────────────

    fn detect_straw_man(&self, lower: &str) -> Option<FallacyHit> {
        // Pattern 1: "So you're saying that..." + extreme/absurd conclusion
        for phrase in &[
            "so you're saying that",
            "so you are saying that",
            "so what you're saying is",
        ] {
            if let Some(pos) = lower.find(phrase) {
                let after = lower[pos + phrase.len()..].trim();
                let extreme_words = [
                    "destroy",
                    "abolish",
                    "eliminate",
                    "remove all",
                    "ban everything",
                    "hate",
                    "against all",
                    "want nothing but",
                    "completely get rid of",
                ];
                if after.split_whitespace().count() < 30 {
                    let has_extreme = extreme_words.iter().any(|e| after.contains(e));
                    let has_absurd = after.contains("every single")
                        || after.contains("all of them")
                        || after.contains("every last");
                    if has_extreme || has_absurd {
                        let excerpt = after.chars().take(60).collect::<String>();
                        return Some(structural_hit(
                            "straw_man",
                            FallacySeverity::Critical,
                            0,
                            format!("{} {}", phrase, excerpt),
                            pos,
                            0.70,
                            "Misrepresents an opposing position to make it easier to attack",
                            format!("Straw man detected: attribute an extreme position ('{}') to an opposing view, then attacks the caricature rather than the actual argument.", excerpt),
                        ));
                    }
                }
            }
        }

        // Pattern 2: Extreme characterization of opponents
        let carrier_phrases = [
            ("opponents want", "wants to"),
            ("the other side wants", "wants to"),
            ("they want to", "wants to"),
            ("critics claim", "claims that"),
            ("detractors say", "says that"),
        ];

        for (carrier, _) in &carrier_phrases {
            if let Some(pos) = lower.find(carrier) {
                let after = lower[pos + carrier.len()..].trim();
                let extreme_triggers = [
                    "destroy",
                    "abolish",
                    "dismantle",
                    "eliminate",
                    "get rid of",
                    "ban all",
                    "remove all",
                ];
                for et in &extreme_triggers {
                    if let Some(et_pos) = after.find(et) {
                        let excerpt = after.chars().take(60).collect::<String>();
                        return Some(structural_hit(
                            "straw_man",
                            FallacySeverity::Critical,
                            0,
                            format!("{} {}", carrier, excerpt),
                            pos + et_pos,
                            0.65,
                            "Misrepresents an opposing position to make it easier to attack",
                            format!("Straw man detected: '{}' followed by extreme characterization ('{}'). Opposing position is described in extreme terms that do not accurately represent the actual argument.", carrier, excerpt),
                        ));
                    }
                }
            }
        }

        // Pattern 3: "believes all X should..." — sweeping attribution
        if let Some(pos) = lower.find("believes all") {
            let after = lower[pos + 11..].trim();
            if after.len() < 60 {
                let excerpt = after.chars().take(50).collect::<String>();
                if !excerpt.is_empty() {
                    return Some(structural_hit(
                        "straw_man",
                        FallacySeverity::Critical,
                        0,
                        format!("believes all {}", excerpt),
                        pos,
                        0.60,
                        "Misrepresents an opposing position to make it easier to attack",
                        format!("Straw man detected: attributes a sweeping universal claim ('believes all {}') to an opposing view, overgeneralizing their position.", excerpt),
                    ));
                }
            }
        }

        None
    }

    // ── Structural: False Dilemma ───────────────────────────────────────

    fn detect_false_dilemma(&self, lower: &str) -> Option<FallacyHit> {
        // Pattern 1: "there is no alternative" / "no other choice" / "only option"
        let binary_exclusion = [
            "there is no alternative",
            "there is no other option",
            "there's no alternative",
            "there's no other choice",
            "no other possibility",
            "the only option is",
            "the only alternative is",
        ];
        for pat in &binary_exclusion {
            if let Some(pos) = lower.find(pat) {
                return Some(structural_hit(
                    "false_dilemma",
                    FallacySeverity::High,
                    1,
                    pat.to_string(),
                    pos,
                    0.70,
                    "Presents limited options as the only possibilities",
                    format!("False dilemma: '{}' asserts a single path exists, ignoring the spectrum of possible alternatives.", pat),
                ));
            }
        }

        // Pattern 2: "if not X then Y" — binary outcome framing
        if let Some(pos) = lower.find("if not ") {
            let after = lower[pos + 7..].trim();
            if let Some(then_pos) = after.find(" then ") {
                let alternative = after[..then_pos].trim();
                let consequence = after[then_pos + 5..].trim();
                if !alternative.is_empty()
                    && !consequence.is_empty()
                    && alternative.len() < 60
                    && consequence.len() < 60
                {
                    return Some(structural_hit(
                        "false_dilemma",
                        FallacySeverity::High,
                        1,
                        format!("if not {} then {}", alternative, consequence),
                        pos,
                        0.60,
                        "Presents limited options as the only possibilities",
                        format!("False dilemma: frames choice as a binary 'if not {0} then {1}', excluding third options or middle ground.", alternative, consequence),
                    ));
                }
            }
        }

        // Pattern 3: "choice between X and Y" without acknowledging nuance
        if let Some(pos) = lower.find("choice between") {
            let after = lower[pos + 14..].trim();
            if let Some(and_pos) = after.find(" and ") {
                let first = after[..and_pos].trim();
                if first.len() < 50 {
                    let rest = after[and_pos + 5..].trim();
                    if let Some(end) =
                        rest.find(|c: char| c == '.' || c == ',' || c == '!' || c == '?')
                    {
                        let second = rest[..end].trim();
                        return Some(structural_hit(
                            "false_dilemma",
                            FallacySeverity::High,
                            1,
                            format!("choice between {} and {}", first, second),
                            pos,
                            0.55,
                            "Presents limited options as the only possibilities",
                            format!("False dilemma: presents a binary choice between '{}' and '{}' without acknowledging possible middle positions or third options.", first, second),
                        ));
                    }
                }
            }
        }

        None
    }

    // ── Structural: Appeal to Nature ────────────────────────────────────

    fn detect_appeal_to_nature(&self, lower: &str) -> Option<FallacyHit> {
        // Pattern 1: "natural" implying superiority (combined with positive outcomes)
        let nature_superiority = [
            ("natural remedy", 0.65),
            ("natural cure", 0.65),
            ("all-natural", 0.60),
            ("all natural", 0.60),
            ("naturally better", 0.70),
            ("100% natural", 0.60),
            ("natural solution", 0.55),
            ("natural treatment", 0.55),
        ];
        for (pat, conf) in &nature_superiority {
            if let Some(pos) = lower.find(pat) {
                return Some(structural_hit(
                    "appeal_to_nature",
                    FallacySeverity::Medium,
                    2,
                    pat.to_string(),
                    pos,
                    *conf,
                    "Argues that something is good because it is 'natural' or bad because it is 'unnatural'",
                    format!("Appeal to nature: '{}' implies natural = superior without evidence. Natural origin does not guarantee safety, efficacy, or moral value.", pat),
                ));
            }
        }

        // Pattern 2: "chemical-free" implying superiority
        if let Some(pos) = lower.find("chemical-free") {
            return Some(structural_hit(
                "appeal_to_nature",
                FallacySeverity::Medium,
                2,
                "chemical-free".to_string(),
                pos,
                0.65,
                "Argues that something is good because it is 'natural' or bad because it is 'unnatural'",
                "Appeal to nature: 'chemical-free' is a marketing claim that implies natural superiority. All matter, including safe water and oxygen, consists of chemicals.".into(),
            ));
        }
        if let Some(pos) = lower.find("no chemicals") {
            // Verify this is used as a superiority claim (near "safe", "better", "pure")
            let context_start = 0.max(pos as isize - 40) as usize;
            let context_end = (pos + 12).min(lower.len());
            let context = &lower[context_start..context_end];
            let value_words = ["safe", "better", "pure", "healthy", "natural", "good"];
            if value_words.iter().any(|v| context.contains(v)) {
                return Some(structural_hit(
                    "appeal_to_nature",
                    FallacySeverity::Medium,
                    2,
                    "no chemicals".to_string(),
                    pos,
                    0.60,
                    "Argues that something is good because it is 'natural' or bad because it is 'unnatural'",
                    "Appeal to nature: 'no chemicals' is used as a value claim suggesting natural superiority. Everything physical, including the human body, is made of chemicals.".into(),
                ));
            }
        }

        // Pattern 3: "unnatural" as pejorative / "against nature" implying wrong
        let unnatural_patterns = [
            ("unnatural act", 0.65),
            ("unnatural behavior", 0.60),
            ("against nature", 0.65),
            ("it's unnatural", 0.60),
            ("that's unnatural", 0.55),
        ];
        for (pat, conf) in &unnatural_patterns {
            if let Some(pos) = lower.find(pat) {
                return Some(structural_hit(
                    "appeal_to_nature",
                    FallacySeverity::Medium,
                    2,
                    pat.to_string(),
                    pos,
                    *conf,
                    "Argues that something is good because it is 'natural' or bad because it is 'unnatural'",
                    format!("Appeal to nature: '{}' equates 'unnatural' with 'wrong' or 'harmful'. Naturalness is not a reliable measure of moral or practical value.", pat),
                ));
            }
        }

        // Pattern 4: "it's natural so" / "naturally" as unproven justification
        if let Some(pos) = lower.find("it's natural so") {
            return Some(structural_hit(
                "appeal_to_nature",
                FallacySeverity::Medium,
                2,
                "it's natural so".to_string(),
                pos,
                0.60,
                "Argues that something is good because it is 'natural' or bad because it is 'unnatural'",
                "Appeal to nature: 'it's natural so' uses naturalness as a premise for a conclusion without evidence connecting origin to outcome.".into(),
            ));
        }

        None
    }

    // ── Structural: False Equivalence ───────────────────────────────────

    fn detect_false_equivalence(&self, lower: &str) -> Option<FallacyHit> {
        // Pattern 1: "both sides do it" / "both sides are" + equivalence
        if let Some(pos) = lower.find("both sides") {
            let after = lower[pos + 10..].trim();
            let eq_patterns = [
                "do it",
                "are the same",
                "are equally",
                "are both",
                "have the same",
                "are no different",
            ];
            for eq in &eq_patterns {
                if after.contains(eq) {
                    return Some(structural_hit(
                        "false_equivalence",
                        FallacySeverity::Medium,
                        2,
                        format!("both sides {}", eq),
                        pos,
                        0.65,
                        "Compares two things as equivalent despite disproportionate scale or nature",
                        format!("False equivalence: 'both sides {}' treats two sides as comparable without acknowledging differences in scale, severity, or nature.", eq),
                    ));
                }
            }
        }

        // Pattern 2: "just as bad as" — disproportionate comparison
        if let Some(pos) = lower.find("just as bad as") {
            let before = lower[0.max(pos as isize - 50) as usize..pos].trim();
            let after = lower[pos + 13..].trim();
            let after_short = after.chars().take(50).collect::<String>();
            return Some(structural_hit(
                "false_equivalence",
                FallacySeverity::Medium,
                2,
                format!("...{} just as bad as {}...", before.chars().take(30).collect::<String>(), after_short),
                pos,
                0.60,
                "Compares two things as equivalent despite disproportionate scale or nature",
                format!("False equivalence: compares two things as equally bad ('just as bad as'). This framing equates potentially disproportionate actions or situations without nuanced analysis."),
            ));
        }

        // Pattern 3: "both sides are the same" / "no difference between X and Y"
        let same_patterns = [
            ("are the same", 0.60),
            ("no different from", 0.55),
            ("no difference between", 0.55),
        ];
        for (pat, conf) in &same_patterns {
            if let Some(pos) = lower.find(pat) {
                // Check this isn't a literal/innocent usage by looking for comparative context
                let context_start = 0.max(pos as isize - 60) as usize;
                let context_end = (pos + pat.len() + 50).min(lower.len());
                let context = &lower[context_start..context_end];
                // Look for comparative framing
                let signals = ["compared to", "versus", "vs", "between", "either", "both"];
                if signals.iter().any(|s| context.contains(s)) || pat == &"are the same" {
                    return Some(structural_hit(
                        "false_equivalence",
                        FallacySeverity::Medium,
                        2,
                        pat.to_string(),
                        pos,
                        *conf,
                        "Compares two things as equivalent despite disproportionate scale or nature",
                        format!("False equivalence: '{}' asserts equivalence between compared items without proportionate analysis of their differences.", pat),
                    ));
                }
            }
        }

        // Pattern 4: whataboutism-style deflection with disproportionate comparison
        let whatabout_triggers = [
            "what about when they",
            "but what about the time",
            "what about their",
        ];
        for trigger in &whatabout_triggers {
            if let Some(wa_pos) = lower.find(trigger) {
                // Check that the whatabout is deflecting from a prior criticism
                let before = &lower[0.max(wa_pos as isize - 80) as usize..wa_pos];
                let has_deflection_context = before.contains("criticism")
                    || before.contains("accus")
                    || before.contains("problem")
                    || before.contains("wrong")
                    || before.contains("hypocrite")
                    || before.contains("blame");
                if has_deflection_context {
                    let after = lower[wa_pos + trigger.len()..].trim();
                    let excerpt = after.chars().take(40).collect::<String>();
                    return Some(structural_hit(
                        "false_equivalence",
                        FallacySeverity::Medium,
                        2,
                        format!("{} {}", trigger, excerpt),
                        wa_pos,
                        0.55,
                        "Compares two things as equivalent despite disproportionate scale or nature",
                        format!("Whataboutism (false equivalence): deflects criticism by invoking '{}' to imply both sides are equally culpable, avoiding substantive engagement with the original issue.", trigger),
                    ));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let f = FallacyFilter::new();
        let r = f.scan("The temperature today is 22 degrees Celsius.");
        assert!(!r.blocked);
        assert!(r.hits.is_empty());
    }

    #[test]
    fn test_appeal_to_emotion() {
        let f = FallacyFilter::new();
        let r = f.scan("This heartbreaking tragedy shows we must act now. Think of the children.");
        assert!(r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "appeal_to_emotion"));
    }

    #[test]
    fn test_straw_man() {
        let f = FallacyFilter::new();
        let r = f.scan("My opponents claim that we should do nothing about this problem.");
        assert!(r.blocked);
    }

    #[test]
    fn test_false_dilemma() {
        let f = FallacyFilter::new();
        let r = f.scan("There are only two options: either you're with us or against us.");
        assert!(!r.blocked); // P1, not critical
        assert!(r.hits.iter().any(|h| h.pattern_name == "false_dilemma"));
    }

    #[test]
    fn test_multiple_fallacies() {
        let f = FallacyFilter::new();
        let r = f.scan("Everyone knows this is true. And if you disagree, you're just wrong. It's only a matter of time before disaster strikes.");
        assert!(r.blocked); // bandwagon is critical
        assert!(r.hits.len() >= 1);
    }

    #[test]
    fn test_summary() {
        let f = FallacyFilter::new();
        let r = f.scan("clean text");
        let s = f.summary(&r);
        assert!(s.contains("\u{2713}"));
    }

    // ── Structural: Circular Reasoning ──────────────────────────────────

    #[test]
    fn test_circular_self_evident() {
        let f = FallacyFilter::new();
        let r = f.scan("This proposal is self-evident and requires no further justification.");
        assert!(!r.blocked); // P1, not critical
        assert!(r
            .hits
            .iter()
            .any(|h| h.pattern_name == "circular_reasoning"));
        let hit = r
            .hits
            .iter()
            .find(|h| h.pattern_name == "circular_reasoning")
            .unwrap();
        assert!(hit.confidence > 0.0);
        assert!(!hit.explanation.is_empty());
    }

    #[test]
    fn test_circular_because_word_overlap() {
        let f = FallacyFilter::new();
        let r = f.scan(
            "The regulation works because the regulation is effective at achieving its goals.",
        );
        assert!(r
            .hits
            .iter()
            .any(|h| h.pattern_name == "circular_reasoning"));
    }

    #[test]
    fn test_circular_it_is_true_because() {
        let f = FallacyFilter::new();
        let r = f.scan("It is true because that's just how it is and everyone knows it.");
        assert!(r
            .hits
            .iter()
            .any(|h| h.pattern_name == "circular_reasoning"));
    }

    #[test]
    fn test_circular_mutual_justification() {
        let f = FallacyFilter::new();
        let r = f.scan("The policy is fair because it was democratically approved. It was democratically approved because it is fair.");
        assert!(r
            .hits
            .iter()
            .any(|h| h.pattern_name == "circular_reasoning"));
    }

    // ── Structural: Straw Man ───────────────────────────────────────────

    #[test]
    fn test_straw_man_so_youre_saying() {
        let f = FallacyFilter::new();
        let r = f.scan("So you're saying that we should destroy the entire system and start over.");
        assert!(r.blocked); // structural straw man is P0 critical
        assert!(r.hits.iter().any(|h| h.pattern_name == "straw_man"));
    }

    #[test]
    fn test_straw_man_extreme_opponents() {
        let f = FallacyFilter::new();
        let r = f
            .scan("My opponents want to completely destroy the economy with their reckless plans.");
        assert!(r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "straw_man"));
    }

    #[test]
    fn test_straw_man_believes_all() {
        let f = FallacyFilter::new();
        let r =
            f.scan("He believes all immigrants should be removed from the country immediately.");
        assert!(r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "straw_man"));
    }

    // ── Structural: False Dilemma ───────────────────────────────────────

    #[test]
    fn test_false_dilemma_no_alternative() {
        let f = FallacyFilter::new();
        let r = f.scan("There is no alternative to this approach, we must accept it.");
        assert!(!r.blocked); // P1
        assert!(r.hits.iter().any(|h| h.pattern_name == "false_dilemma"));
    }

    #[test]
    fn test_false_dilemma_if_not_then() {
        let f = FallacyFilter::new();
        let r = f.scan("If not this policy, then we face total economic collapse.");
        assert!(!r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "false_dilemma"));
    }

    #[test]
    fn test_false_dilemma_choice_between() {
        let f = FallacyFilter::new();
        let r = f.scan("The choice between unlimited growth and total collapse is clear.");
        assert!(!r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "false_dilemma"));
    }

    // ── Structural: Appeal to Nature ────────────────────────────────────

    #[test]
    fn test_appeal_to_nature_natural_remedy() {
        let f = FallacyFilter::new();
        let r = f.scan("Try this natural remedy, it's chemical-free and better for you.");
        assert!(!r.blocked); // P2
        assert!(r.hits.iter().any(|h| h.pattern_name == "appeal_to_nature"));
    }

    #[test]
    fn test_appeal_to_nature_unnatural() {
        let f = FallacyFilter::new();
        let r = f.scan("This practice is unnatural and therefore morally wrong.");
        assert!(!r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "appeal_to_nature"));
    }

    #[test]
    fn test_appeal_to_nature_all_natural() {
        let f = FallacyFilter::new();
        let r = f.scan("Our product is 100% natural, so it must be safe.");
        assert!(!r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "appeal_to_nature"));
    }

    // ── Structural: False Equivalence ───────────────────────────────────

    #[test]
    fn test_false_equivalence_both_sides() {
        let f = FallacyFilter::new();
        let r = f.scan("Both sides do it, so you can't criticize one without the other.");
        assert!(!r.blocked); // P2
        assert!(r.hits.iter().any(|h| h.pattern_name == "false_equivalence"));
    }

    #[test]
    fn test_false_equivalence_just_as_bad() {
        let f = FallacyFilter::new();
        let r = f.scan("A parking ticket is just as bad as committing fraud.");
        assert!(!r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "false_equivalence"));
    }

    #[test]
    fn test_false_equivalence_are_the_same() {
        let f = FallacyFilter::new();
        let r = f.scan("Both candidates are the same, there is no real difference between them.");
        assert!(!r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "false_equivalence"));
    }

    #[test]
    fn test_false_equivalence_whatabout() {
        let f = FallacyFilter::new();
        let r = f.scan("You criticize this country's human rights record, but what about when they do the same things?");
        assert!(!r.blocked);
        assert!(r.hits.iter().any(|h| h.pattern_name == "false_equivalence"));
    }

    // ── Structural: Confidence and Explanation Fields ───────────────────

    #[test]
    fn test_structural_hit_has_confidence() {
        let f = FallacyFilter::new();
        let r = f.scan("There is no alternative to this course of action.");
        for hit in &r.hits {
            assert!(hit.confidence >= 0.0 && hit.confidence <= 1.0);
            assert!(!hit.explanation.is_empty());
        }
    }

    #[test]
    fn test_keyword_hit_has_default_confidence() {
        let f = FallacyFilter::new();
        let r = f.scan("This is terrifying and everyone knows it.");
        for hit in &r.hits {
            assert!(hit.confidence >= 0.0 && hit.confidence <= 1.0);
        }
    }

    #[test]
    fn test_no_false_positive_on_legitimate_because() {
        let f = FallacyFilter::new();
        let r =
            f.scan("The sky appears blue because of Rayleigh scattering of shorter wavelengths.");
        // This should NOT trigger circular reasoning — the reason adds new information
        let circular = r
            .hits
            .iter()
            .filter(|h| h.pattern_name == "circular_reasoning")
            .count();
        assert!(
            circular == 0,
            "Legitimate causal explanation should not trigger circular reasoning"
        );
    }

    #[test]
    fn test_no_false_positive_natural_as_neutral() {
        let f = FallacyFilter::new();
        let r = f.scan("Natural gas is a fossil fuel used for heating.");
        let nature_hits = r
            .hits
            .iter()
            .filter(|h| h.pattern_name == "appeal_to_nature")
            .count();
        assert!(
            nature_hits == 0,
            "'Natural gas' as a technical term should not trigger appeal to nature"
        );
    }
}

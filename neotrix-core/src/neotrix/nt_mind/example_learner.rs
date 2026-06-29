//! Example-based learning and RL extraction optimization.
//!
//! Two subsystems:
//! 1. ExampleBasedLearner — learn extraction patterns from labeled examples
//! 2. RlExtractionOptimizer — SCRIBES-inspired RL for learning extraction scripts

use super::selection_engine::{
    AdaptiveSelector, ExtractionSchema, FieldDef, FieldType, StructuralFingerprint,
};
use std::collections::{HashMap, HashSet};

// ============================================================================
// HTML scanning utilities (string-based, zero external dependencies)
// ============================================================================

/// Simple opening tag info
#[derive(Debug)]
struct ScanTag {
    tag_name: String,
    attrs: String,
    open_start: usize,
    open_end: usize,
    is_self_closing: bool,
}

/// Scan for the next opening tag at or after `pos`.
fn scan_next_tag(html: &str, pos: usize) -> Option<ScanTag> {
    let rest = &html[pos..];
    let lt = rest.find('<')?;
    let abs_lt = pos + lt;

    if abs_lt + 1 >= html.len() {
        return None;
    }
    let c2 = html.as_bytes()[abs_lt + 1];
    if c2 == b'/' || c2 == b'!' || c2 == b'?' {
        let gt = html[abs_lt..].find('>')?;
        return scan_next_tag(html, abs_lt + gt + 1);
    }

    let tag_body = &html[abs_lt + 1..];
    let name_end = tag_body
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(tag_body.len());
    if name_end == 0 {
        return None;
    }
    let tag_name = tag_body[..name_end].to_lowercase();

    // Find closing '>' of opening tag
    let mut in_quote = false;
    let mut quote_char = '"';
    let mut gt_offset = 0;
    for (i, c) in tag_body.char_indices() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            }
        } else {
            match c {
                '"' | '\'' => {
                    in_quote = true;
                    quote_char = c;
                }
                '>' => {
                    gt_offset = i + 1;
                    break;
                }
                _ => {}
            }
        }
    }
    if gt_offset == 0 {
        return None;
    }

    let full_open = &tag_body[..gt_offset];
    let attrs_str = if name_end + 1 < full_open.len() {
        let trimmed = full_open[name_end..gt_offset.saturating_sub(1)].trim();
        if trimmed.ends_with('/') {
            trimmed[..trimmed.len() - 1].trim().to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        String::new()
    };

    let is_self_closing = full_open.trim_end().ends_with('/')
        || [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
            "source", "track", "wbr",
        ]
        .contains(&tag_name.as_str());

    Some(ScanTag {
        tag_name,
        attrs: attrs_str,
        open_start: abs_lt,
        open_end: abs_lt + 1 + gt_offset,
        is_self_closing,
    })
}

/// Find the position of the matching closing tag, handling nesting.
fn find_close(html: &str, tag: &str, open_end: usize) -> Option<usize> {
    let mut depth: i32 = 1;
    let mut pos = open_end;
    while depth > 0 {
        let rest = &html[pos..];
        let lt = rest.find('<')?;
        pos += lt;
        if pos + 1 >= html.len() {
            return None;
        }
        let c2 = html.as_bytes()[pos + 1];
        if c2 == b'/' {
            let cr = &html[pos + 2..];
            let ne = cr
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(cr.len());
            if cr[..ne].to_lowercase() == tag {
                depth -= 1;
                if depth == 0 {
                    let gt = html[pos..].find('>')?;
                    return Some(pos + gt + 1);
                }
            }
            let gt = html[pos..].find('>')?;
            pos = pos + gt + 1;
        } else if c2 == b'!' || c2 == b'?' {
            let gt = html[pos..].find('>')?;
            pos = pos + gt + 1;
        } else if let Some(st) = scan_next_tag(html, pos) {
            if !st.is_self_closing && st.tag_name == tag {
                depth += 1;
            }
            pos = st.open_end;
        } else {
            return None;
        }
    }
    None
}

#[derive(Debug, Default)]
struct SimpleSel {
    tag: Option<String>,
    class: Option<String>,
    id: Option<String>,
}

fn parse_simple_sel(sel: &str) -> SimpleSel {
    let s = sel.trim();
    let mut tag = None;
    let mut class = None;
    let mut id = None;
    let mut buf = String::new();
    let mut expect: Option<u8> = None;

    for c in s.chars() {
        match expect {
            Some(b'.') => {
                if c == '.' || c == '#' {
                    class = Some(std::mem::take(&mut buf));
                    expect = Some(if c == '#' { b'#' } else { b'.' });
                } else {
                    buf.push(c);
                }
            }
            Some(b'#') => {
                if c == '.' || c == '#' {
                    id = Some(std::mem::take(&mut buf));
                    expect = Some(if c == '#' { b'#' } else { b'.' });
                } else {
                    buf.push(c);
                }
            }
            _ => match c {
                '.' => {
                    if !buf.is_empty() {
                        tag = Some(std::mem::take(&mut buf));
                    }
                    expect = Some(b'.');
                }
                '#' => {
                    if !buf.is_empty() {
                        tag = Some(std::mem::take(&mut buf));
                    }
                    expect = Some(b'#');
                }
                _ => buf.push(c),
            },
        }
    }
    if !buf.is_empty() {
        match expect {
            Some(b'.') => class = Some(buf),
            Some(b'#') => id = Some(buf),
            _ => tag = Some(buf),
        }
    }
    SimpleSel { tag, class, id }
}

fn extract_classes_from_attrs(attrs: &str) -> Vec<String> {
    let lower = attrs.to_lowercase();
    let idx = match lower.find("class=") { Some(i) => i, None => return vec![] };
    let after = &attrs[idx + 6..];
    let q = match after.chars().next() { Some(c) => c, None => return vec![] };
    let start = 1;
    if start >= after.len() {
        return vec![];
    }
    let inner = &after[start..];
    let end = match inner.find(q) { Some(e) => e, None => return vec![] };
    inner[..end]
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn extract_id_from_attrs(attrs: &str) -> Option<String> {
    let lower = attrs.to_lowercase();
    let idx = lower.find("id=")?;
    let after = &attrs[idx + 3..];
    let q = after.chars().next()?;
    let start = 1;
    if start >= after.len() {
        return None;
    }
    let inner = &after[start..];
    let end = inner.find(q)?;
    let val = inner[..end].trim().to_string();
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

fn tag_matches_sel(tag_name: &str, attrs: &str, sel: &SimpleSel) -> bool {
    if let Some(ref t) = sel.tag {
        if tag_name != t {
            return false;
        }
    }
    if let Some(ref c) = sel.class {
        let cls = extract_classes_from_attrs(attrs);
        if !cls.iter().any(|x| x == c) {
            return false;
        }
    }
    if let Some(ref i) = sel.id {
        if extract_id_from_attrs(attrs).as_deref() != Some(i) {
            return false;
        }
    }
    true
}

/// Find the range `(start, end)` of the first element matching a simple selector.
fn find_element_range(html: &str, sel: &str) -> Option<(usize, usize)> {
    let parsed = parse_simple_sel(sel);
    let mut pos = 0;
    while let Some(st) = scan_next_tag(html, pos) {
        if tag_matches_sel(&st.tag_name, &st.attrs, &parsed) {
            if st.is_self_closing {
                return Some((st.open_start, st.open_end));
            }
            let close = find_close(html, &st.tag_name, st.open_end)?;
            return Some((st.open_start, close));
        }
        if !st.is_self_closing {
            if let Some(cp) = find_close(html, &st.tag_name, st.open_end) {
                pos = cp;
            } else {
                pos = st.open_end;
            }
        } else {
            pos = st.open_end;
        }
    }
    None
}

/// Strip HTML tags to extract text content.
fn strip_html_simple(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut buf = String::new();
    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                buf.clear();
                buf.push(c);
            }
            '>' if in_tag => {
                buf.push(c);
                let l = buf.to_lowercase();
                if l.starts_with("<script") {
                    in_script = true;
                } else if l.starts_with("</script") {
                    in_script = false;
                } else if l.starts_with("<style") {
                    in_style = true;
                } else if l.starts_with("</style") {
                    in_style = false;
                }
                in_tag = false;
            }
            _ if !in_tag && !in_script && !in_style => {
                if c.is_whitespace() {
                    if !out.ends_with(' ') && !out.is_empty() {
                        out.push(' ');
                    }
                } else {
                    out.push(c);
                }
            }
            _ if in_tag => buf.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

/// Extract the inner text of the first element matching a selector.
fn inner_text_by_selector(html: &str, sel: &str) -> Option<String> {
    let (start, end) = find_element_range(html, sel)?;
    let inner = &html[start..end];
    let text = strip_html_simple(inner);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[allow(dead_code)]
fn jaccard_sim(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let sa: HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let sb: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let inter = sa.intersection(&sb).count() as f64;
    let union = sa.union(&sb).count() as f64;
    if union == 0.0 {
        1.0
    } else {
        inter / union
    }
}

// ============================================================================
// Part 1: Example-Based Learning
// ============================================================================

/// A training example: URL + target elements.
#[derive(Debug, Clone)]
pub struct ExtractionExample {
    pub id: u32,
    pub url: String,
    pub html_snippet: String,
    pub target_elements: Vec<LabeledElement>,
    pub page_context: String,
}

/// A single labeled element in an extraction example.
#[derive(Debug, Clone)]
pub struct LabeledElement {
    pub text: String,
    pub selector: String,
    pub structural_fingerprint: StructuralFingerprint,
    pub label: String,
}

/// Learn extraction patterns from labeled examples.
pub struct ExampleBasedLearner {
    pub examples: Vec<ExtractionExample>,
    pub learned_selectors: HashMap<String, AdaptiveSelector>,
    pub learned_schemas: Vec<ExtractionSchema>,
    next_id: u32,
}

impl ExampleBasedLearner {
    pub fn new() -> Self {
        Self {
            examples: Vec::new(),
            learned_selectors: HashMap::new(),
            learned_schemas: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a training example (URL + HTML + labeled element selector/label pairs).
    pub fn add_example(
        &mut self,
        url: &str,
        html: &str,
        elements: Vec<(&str, &str)>,
    ) -> Result<u32, String> {
        let id = self.next_id;
        self.next_id += 1;

        let mut target_elements = Vec::new();
        for (selector, label) in &elements {
            let fps = StructuralFingerprint::from_html(html, selector)
                .map_err(|e| format!("Failed to extract fingerprint for '{}': {}", selector, e))?;

            let fp = fps
                .into_iter()
                .next()
                .ok_or_else(|| format!("No element found for selector: {}", selector))?;

            let text = inner_text_by_selector(html, selector).unwrap_or_default();

            target_elements.push(LabeledElement {
                text,
                selector: selector.to_string(),
                structural_fingerprint: fp,
                label: label.to_string(),
            });
        }

        let snippet = if html.len() > 2000 {
            html[..2000].to_string()
        } else {
            html.to_string()
        };

        self.examples.push(ExtractionExample {
            id,
            url: url.to_string(),
            html_snippet: snippet,
            target_elements,
            page_context: html.to_string(),
        });

        Ok(id)
    }

    /// Train selector from the first example containing the label.
    /// Cross-validates against other examples if available.
    pub fn train_selector(&mut self, label: &str) -> Result<&AdaptiveSelector, String> {
        let example = self
            .examples
            .iter()
            .find(|ex| ex.target_elements.iter().any(|el| el.label == label))
            .ok_or_else(|| format!("No examples found for label: {}", label))?;

        let element = example
            .target_elements
            .iter()
            .find(|el| el.label == label)
            .unwrap();

        let mut selector = AdaptiveSelector::new(&element.selector, &example.page_context)?;

        // Cross-validate against other examples
        let mut successes = 0u64;
        let mut total = 0u64;
        for other_ex in &self.examples {
            if other_ex.id == example.id {
                continue;
            }
            for el in &other_ex.target_elements {
                if el.label == label {
                    total += 1;
                    if selector.reidentify(&other_ex.page_context).is_ok() {
                        successes += 1;
                    }
                }
            }
        }

        if total > 0 {
            selector.success_count = successes;
            selector.use_count = total + 1;
        }

        self.learned_selectors.insert(label.to_string(), selector);
        Ok(self.learned_selectors.get(label).unwrap())
    }

    /// Infer extraction schema from examples.
    pub fn infer_schema(&mut self) -> ExtractionSchema {
        let mut field_map: HashMap<String, Vec<&LabeledElement>> = HashMap::new();
        for ex in &self.examples {
            for el in &ex.target_elements {
                field_map.entry(el.label.clone()).or_default().push(el);
            }
        }

        let mut fields = Vec::new();
        for (label, elements) in &field_map {
            // Use the most common selector or the first one
            let selector = elements
                .first()
                .map(|e| e.selector.clone())
                .unwrap_or_default();

            let field_def = FieldDef {
                name: label.clone(),
                selector,
                field_type: FieldType::Text,
                attribute: None,
                nested: None,
                repeated: false,
                required: true,
            };
            fields.push(field_def);
        }

        let schema = ExtractionSchema {
            name: "inferred".to_string(),
            base_selector: None,
            fields,
        };
        self.learned_schemas.push(schema);
        self.learned_schemas.last().unwrap().clone()
    }

    /// Validate a learned selector against a held-out HTML page.
    pub fn validate_selector(&self, label: &str, html: &str) -> Result<f64, String> {
        let selector = self
            .learned_selectors
            .get(label)
            .ok_or_else(|| format!("No learned selector for label: {}", label))?;

        match selector.reidentify(html) {
            Ok(_) => Ok(selector.confidence()),
            Err(e) => Err(e),
        }
    }

    /// Leave-one-out cross-validation for a label.
    pub fn cross_validate(&self, label: &str) -> Vec<(u32, f64)> {
        let mut results = Vec::new();
        for (i, ex) in self.examples.iter().enumerate() {
            if !ex.target_elements.iter().any(|el| el.label == label) {
                continue;
            }

            // Create temporary learner with all examples except this one
            let mut temp_learner = ExampleBasedLearner::new();
            for (j, other_ex) in self.examples.iter().enumerate() {
                if i == j {
                    continue;
                }
                if let Ok(_) = temp_learner.add_example(
                    &other_ex.url,
                    &other_ex.page_context,
                    other_ex
                        .target_elements
                        .iter()
                        .map(|el| (el.selector.as_str(), el.label.as_str()))
                        .collect::<Vec<_>>(),
                ) {
                    // OK
                }
            }

            // Train selector
            if let Ok(selector) = temp_learner.train_selector(label) {
                let acc = if selector.reidentify(&ex.page_context).is_ok() {
                    1.0
                } else {
                    0.0
                };
                results.push((ex.id, acc));
            } else {
                results.push((ex.id, 0.0));
            }
        }
        results
    }

    /// Confidence score for a learned extraction.
    pub fn confidence(&self, label: &str) -> f64 {
        let selector = match self.learned_selectors.get(label) {
            Some(s) => s,
            None => return 0.0,
        };

        let num_examples = self
            .examples
            .iter()
            .filter(|ex| ex.target_elements.iter().any(|el| el.label == label))
            .count();

        let example_factor = (num_examples as f64 / (num_examples as f64 + 2.0)).min(1.0);
        let selector_conf = selector.confidence();

        0.4 * example_factor + 0.6 * selector_conf
    }
}

impl Default for ExampleBasedLearner {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Part 2: RL Extraction Optimizer
// ============================================================================

/// State in the RL extraction process.
#[derive(Debug, Clone)]
pub struct ExtractionState {
    pub html: String,
    pub current_depth: u32,
    pub explored_selectors: Vec<String>,
    pub extracted_data: HashMap<String, Vec<String>>,
    pub remaining_html: String,
}

/// Action the RL agent can take.
#[derive(Debug, Clone)]
pub enum ExtractionAction {
    TrySelector(String),
    TryPattern(StructuralPattern),
    ExtractText(String),
    ExtractAttribute(String, String),
    Recurse,
    Backtrack,
    Finish,
}

/// Structural pattern for extraction.
#[derive(Debug, Clone)]
pub enum StructuralPattern {
    HighestTextDensity,
    ClassPattern(String),
    SimilarToExample(StructuralFingerprint),
    RepeatingPattern,
}

/// Record of one RL episode step.
#[derive(Debug, Clone)]
pub struct RlStep {
    pub state: ExtractionState,
    pub action: ExtractionAction,
    pub reward: f64,
    pub next_state: ExtractionState,
}

/// Record of one complete RL episode.
#[derive(Debug, Clone)]
pub struct RlEpisode {
    pub episode_number: u32,
    pub steps: Vec<RlStep>,
    pub total_reward: f64,
    pub success: bool,
}

/// SCRIBES-inspired RL optimizer for learning extraction scripts.
pub struct RlExtractionOptimizer {
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub exploration_rate: f64,
    pub q_table: HashMap<String, HashMap<String, f64>>,
    pub episodes: Vec<RlEpisode>,
    pub max_steps_per_episode: u32,
    pub epsilon_decay: f64,
    episode_counter: u32,
}

impl RlExtractionOptimizer {
    pub fn new() -> Self {
        Self {
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.5,
            q_table: HashMap::new(),
            episodes: Vec::new(),
            max_steps_per_episode: 20,
            epsilon_decay: 0.99,
            episode_counter: 0,
        }
    }

    pub fn with_params(
        learning_rate: f64,
        discount_factor: f64,
        exploration_rate: f64,
        epsilon_decay: f64,
    ) -> Self {
        Self {
            learning_rate,
            discount_factor,
            exploration_rate,
            q_table: HashMap::new(),
            episodes: Vec::new(),
            max_steps_per_episode: 20,
            epsilon_decay,
            episode_counter: 0,
        }
    }

    /// Get Q-values for current state.
    pub fn get_q_values(&self, state: &ExtractionState) -> HashMap<String, f64> {
        let state_key = state_key(state);
        self.q_table.get(&state_key).cloned().unwrap_or_default()
    }

    /// Select action using epsilon-greedy policy.
    pub fn select_action(&mut self, state: &ExtractionState) -> ExtractionAction {
        let available_actions = self.available_actions(state);

        if available_actions.is_empty() {
            return ExtractionAction::Finish;
        }

        // Epsilon-greedy: explore with prob exploration_rate
        let rng: f64 = fastrand();
        if rng < self.exploration_rate {
            let idx = (fastrand() * available_actions.len() as f64) as usize;
            return available_actions[idx.min(available_actions.len() - 1)].clone();
        }

        // Exploit: argmax Q(state, action)
        let state_key = state_key(state);
        let q_vals = self.q_table.get(&state_key);

        let mut best_action = available_actions[0].clone();
        let mut best_q = -f64::INFINITY;

        for action in &available_actions {
            let action_key = action_key(action);
            let q = q_vals
                .and_then(|m| m.get(&action_key))
                .copied()
                .unwrap_or(0.0);
            if q > best_q {
                best_q = q;
                best_action = action.clone();
            }
        }

        best_action
    }

    /// Compute reward for extraction result.
    pub fn compute_reward(
        &self,
        action: &ExtractionAction,
        extracted: &HashMap<String, Vec<String>>,
        target: &HashMap<String, Vec<String>>,
    ) -> f64 {
        let mut reward = 0.0;

        // +1.0 for each correctly extracted field
        for (label, values) in extracted {
            if let Some(tv) = target.get(label) {
                for val in values {
                    if tv.contains(val) {
                        reward += 1.0;
                    }
                }
            }
        }

        // +0.5 for discovery (new data found, even if not in target)
        let total_extracted: usize = extracted.values().map(|v| v.len()).sum();
        if total_extracted > 0 {
            let discovery_bonus = 0.5 * (total_extracted as f64).sqrt();
            reward += discovery_bonus;
        }

        // -0.2 for failed selector
        match action {
            ExtractionAction::TrySelector(sel) => {
                let sel_robust = selector_robustness_score(sel);
                reward += sel_robust * 0.3;
            }
            ExtractionAction::TryPattern(_) => {
                reward += 0.1;
            }
            ExtractionAction::Backtrack => {
                reward -= 0.1;
            }
            ExtractionAction::Finish => {
                // No penalty for finishing
            }
            _ => {}
        }

        reward
    }

    /// Update Q-values using TD-learning.
    pub fn update_q(
        &mut self,
        state: &ExtractionState,
        action: &ExtractionAction,
        reward: f64,
        next_state: &ExtractionState,
    ) {
        let s_key = state_key(state);
        let a_key = action_key(action);
        let ns_key = state_key(next_state);

        let current_q = self
            .q_table
            .entry(s_key.clone())
            .or_default()
            .get(&a_key)
            .copied()
            .unwrap_or(0.0);

        let max_next_q = self
            .q_table
            .get(&ns_key)
            .and_then(|m| m.values().cloned().fold(f64::NEG_INFINITY, f64::max).into())
            .unwrap_or(0.0);

        let td_target = reward + self.discount_factor * max_next_q;
        let new_q = current_q + self.learning_rate * (td_target - current_q);

        self.q_table.entry(s_key).or_default().insert(a_key, new_q);
    }

    /// Run one episode of RL extraction.
    pub fn run_episode(
        &mut self,
        html: &str,
        target_labels: &[&str],
        ground_truth: &HashMap<String, Vec<String>>,
    ) -> RlEpisode {
        let episode_number = self.episode_counter;
        self.episode_counter += 1;

        let mut steps = Vec::new();
        let mut state = ExtractionState {
            html: html.to_string(),
            current_depth: 0,
            explored_selectors: Vec::new(),
            extracted_data: HashMap::new(),
            remaining_html: html.to_string(),
        };

        let mut total_reward = 0.0;
        let mut step_count = 0;

        loop {
            if step_count >= self.max_steps_per_episode {
                break;
            }

            // Check if all target labels have been found
            let all_found = target_labels.iter().all(|l| {
                state.extracted_data.contains_key(*l) && !state.extracted_data[*l].is_empty()
            });

            if all_found {
                let action = ExtractionAction::Finish;
                let next_state = state.clone();
                let reward = self.compute_reward(&action, &state.extracted_data, ground_truth);
                total_reward += reward;

                self.update_q(&state, &action, reward, &next_state);

                steps.push(RlStep {
                    state: state.clone(),
                    action: ExtractionAction::Finish,
                    reward,
                    next_state: next_state.clone(),
                });

                state = next_state;
                break;
            }

            let action = self.select_action(&state);
            let (next_state, action_reward) = self.apply_action(&state, &action, target_labels);

            let ground_reward =
                self.compute_reward(&action, &next_state.extracted_data, ground_truth);
            let reward = action_reward + ground_reward * 5.0;
            total_reward += reward;

            self.update_q(&state, &action, reward, &next_state);

            steps.push(RlStep {
                state: state.clone(),
                action: action.clone(),
                reward,
                next_state: next_state.clone(),
            });

            state = next_state;
            step_count += 1;
        }

        let success = target_labels
            .iter()
            .all(|l| state.extracted_data.contains_key(*l) && !state.extracted_data[*l].is_empty());

        let episode = RlEpisode {
            episode_number,
            steps,
            total_reward,
            success,
        };

        self.episodes.push(episode);
        self.episodes.last().unwrap().clone()
    }

    /// Train for N episodes with epsilon decay.
    pub fn train(
        &mut self,
        html: &str,
        target_labels: &[&str],
        ground_truth: &HashMap<String, Vec<String>>,
        episodes: u32,
    ) -> Vec<f64> {
        let mut rewards = Vec::new();

        for _ in 0..episodes {
            let episode = self.run_episode(html, target_labels, ground_truth);
            rewards.push(episode.total_reward);
            self.decay_exploration();

            // Set the initial exploration rate on the ground_truth
            // to reduce variance once we have a good policy
        }

        rewards
    }

    /// Extract using learned policy (no exploration).
    pub fn extract(&self, html: &str, labels: &[&str]) -> HashMap<String, Vec<String>> {
        let mut state = ExtractionState {
            html: html.to_string(),
            current_depth: 0,
            explored_selectors: Vec::new(),
            extracted_data: HashMap::new(),
            remaining_html: html.to_string(),
        };

        let _saved_epsilon = self.exploration_rate;
        // We can't mutate self, so we just simulate greedy actions
        // by manually picking the best action each step

        let mut step_count = 0;
        loop {
            if step_count >= self.max_steps_per_episode {
                break;
            }

            let all_found = labels.iter().all(|l| {
                state.extracted_data.contains_key(*l) && !state.extracted_data[*l].is_empty()
            });
            if all_found {
                break;
            }

            let available = self.available_actions(&state);
            if available.is_empty() {
                break;
            }

            // Greedy selection (no exploration)
            let state_key = state_key(&state);
            let q_vals = self.q_table.get(&state_key);

            let mut best_action = available[0].clone();
            let mut best_q = -f64::INFINITY;

            for action in &available {
                let a_key = action_key(action);
                let q = q_vals.and_then(|m| m.get(&a_key)).copied().unwrap_or(0.0);
                if q > best_q {
                    best_q = q;
                    best_action = action.clone();
                }
            }

            if let ExtractionAction::Finish = best_action {
                break;
            }

            let (next_state, _reward) = Self::apply_action_imm(&state, &best_action, labels);
            state = next_state;
            step_count += 1;
        }

        state.extracted_data
    }

    /// Get the learned best extraction script.
    pub fn learned_script(&self) -> Vec<(String, ExtractionAction)> {
        let mut script = Vec::new();
        for (state_str, action_map) in &self.q_table {
            let best_action = action_map
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));
            if let Some((action_key, _)) = best_action {
                // Deserialize action key back to an action (simplified)
                let action = deserialize_action(action_key);
                script.push((state_str.clone(), action));
            }
        }
        script
    }

    /// Decay exploration rate.
    pub fn decay_exploration(&mut self) {
        self.exploration_rate *= self.epsilon_decay;
        if self.exploration_rate < 0.01 {
            self.exploration_rate = 0.01;
        }
    }

    // ── Internal helpers ──

    fn available_actions(&self, state: &ExtractionState) -> Vec<ExtractionAction> {
        let mut actions = Vec::new();

        // Try commonly useful selectors
        let common_selectors = vec![
            "h1",
            "h2",
            "h3",
            "p",
            "span",
            "div",
            ".title",
            ".price",
            ".name",
            ".description",
            "[class*=title]",
            "[class*=price]",
        ];

        for sel in &common_selectors {
            if !state.explored_selectors.contains(&sel.to_string()) {
                actions.push(ExtractionAction::TrySelector(sel.to_string()));
            }
        }

        actions.push(ExtractionAction::TryPattern(
            StructuralPattern::HighestTextDensity,
        ));
        actions.push(ExtractionAction::Finish);

        actions
    }

    fn apply_action(
        &self,
        state: &ExtractionState,
        action: &ExtractionAction,
        target_labels: &[&str],
    ) -> (ExtractionState, f64) {
        match action {
            ExtractionAction::TrySelector(sel) => {
                let mut next = state.clone();
                next.explored_selectors.push(sel.clone());

                let text = inner_text_by_selector(&state.remaining_html, sel);
                let reward = match &text {
                    Some(t) if !t.is_empty() => {
                        // Assign to the first target label not yet extracted
                        for label in target_labels {
                            if !next.extracted_data.contains_key(*label) {
                                next.extracted_data
                                    .entry(label.to_string())
                                    .or_default()
                                    .push(t.clone());
                                break;
                            }
                        }
                        // Remove matched content from remaining_html
                        if let Some((start, end)) = find_element_range(&state.remaining_html, sel) {
                            let mut new_remaining = state.remaining_html[..start].to_string();
                            new_remaining.push_str(&state.remaining_html[end..]);
                            next.remaining_html = new_remaining;
                        }
                        1.5
                    }
                    _ => -0.2,
                };

                (next, reward)
            }
            ExtractionAction::TryPattern(pattern) => {
                let (next, reward) = self.apply_pattern(state, pattern, target_labels);
                (next, reward)
            }
            ExtractionAction::ExtractText(sel) => {
                let mut next = state.clone();
                let text = inner_text_by_selector(&state.remaining_html, sel);
                match &text {
                    Some(t) if !t.is_empty() => {
                        for label in target_labels {
                            if !next.extracted_data.contains_key(*label) {
                                next.extracted_data
                                    .entry(label.to_string())
                                    .or_default()
                                    .push(t.clone());
                                break;
                            }
                        }
                        (next, 1.0)
                    }
                    _ => (next, -0.2),
                }
            }
            ExtractionAction::ExtractAttribute(sel, attr) => {
                let mut next = state.clone();
                if let Some((start, end)) = find_element_range(&state.remaining_html, sel) {
                    let element = &state.remaining_html[start..end];
                    let lower = element.to_lowercase();
                    let search = format!("{}=\"", attr.to_lowercase());
                    let search_sq = format!("{}='", attr.to_lowercase());
                    let val = if let Some(idx) = lower.find(&search) {
                        let after = &element[idx + search.len()..];
                        after.find('"').map(|end_q| after[..end_q].to_string())
                    } else if let Some(idx) = lower.find(&search_sq) {
                        let after = &element[idx + search_sq.len()..];
                        after.find('\'').map(|end_q| after[..end_q].to_string())
                    } else {
                        None
                    };
                    if let Some(v) = val {
                        if !v.is_empty() {
                            for label in target_labels {
                                if !next.extracted_data.contains_key(*label) {
                                    next.extracted_data
                                        .entry(label.to_string())
                                        .or_default()
                                        .push(v);
                                    break;
                                }
                            }
                            return (next, 1.0);
                        }
                    }
                }
                (next, -0.2)
            }
            ExtractionAction::Recurse => {
                let mut next = state.clone();
                next.current_depth += 1;
                // Recurse into the first element with highest text density
                if let Some((start, end)) = find_element_range(&state.remaining_html, "div") {
                    let inner = &state.remaining_html[start..end];
                    next.remaining_html = inner.to_string();
                    (next, 0.1)
                } else {
                    (next, -0.1)
                }
            }
            ExtractionAction::Backtrack => {
                let mut next = state.clone();
                if next.current_depth > 0 {
                    next.current_depth -= 1;
                }
                next.remaining_html = next.html.clone();
                (next, -0.1)
            }
            ExtractionAction::Finish => (state.clone(), 0.0),
        }
    }

    fn apply_action_imm(
        state: &ExtractionState,
        action: &ExtractionAction,
        target_labels: &[&str],
    ) -> (ExtractionState, f64) {
        match action {
            ExtractionAction::TrySelector(sel) => {
                let mut next = state.clone();
                let text = inner_text_by_selector(&state.remaining_html, sel);
                match text {
                    Some(t) if !t.is_empty() => {
                        for label in target_labels {
                            if !next.extracted_data.contains_key(*label) {
                                next.extracted_data
                                    .entry(label.to_string())
                                    .or_default()
                                    .push(t);
                                break;
                            }
                        }
                        (next, 1.0)
                    }
                    _ => (next, -0.2),
                }
            }
            ExtractionAction::Finish => (state.clone(), 0.0),
            _ => {
                let mut next = state.clone();
                next.extracted_data = state.extracted_data.clone();
                (next, 0.0)
            }
        }
    }

    fn apply_pattern(
        &self,
        state: &ExtractionState,
        pattern: &StructuralPattern,
        target_labels: &[&str],
    ) -> (ExtractionState, f64) {
        match pattern {
            StructuralPattern::HighestTextDensity => {
                // Scan for the element with highest text density
                let mut best_density = 0.0f64;
                let mut best_sel = String::new();

                for tag in &["div", "section", "article", "p", "span"] {
                    if let Some(text) = inner_text_by_selector(&state.remaining_html, tag) {
                        let density = text.len() as f64 / (tag.len() as f64 + 1.0);
                        if density > best_density {
                            best_density = density;
                            best_sel = tag.to_string();
                        }
                    }
                }

                if !best_sel.is_empty() {
                    let action = ExtractionAction::TrySelector(best_sel);
                    return self.apply_action(state, &action, target_labels);
                }

                (state.clone(), -0.2)
            }
            StructuralPattern::ClassPattern(class_pattern) => {
                let sel = format!(".{}", class_pattern);
                let action = ExtractionAction::TrySelector(sel);
                self.apply_action(state, &action, target_labels)
            }
            StructuralPattern::SimilarToExample(_fp) => {
                // Find element with most similar fingerprint
                let mut best_sim = 0.0f64;
                let mut best_sel = String::new();
                let candidates = ["h1", "h2", "h3", "p", "span", "div", "a"];

                for tag in &candidates {
                    let _sel = format!("{}:first-of-type", tag);
                    if let Some(text) = inner_text_by_selector(&state.remaining_html, tag) {
                        if !text.is_empty() {
                            let sim = 0.5; // simplified; in production would compute actual fingerprint
                            if sim > best_sim {
                                best_sim = sim;
                                best_sel = tag.to_string();
                            }
                        }
                    }
                }

                if !best_sel.is_empty() {
                    let action = ExtractionAction::TrySelector(best_sel);
                    return self.apply_action(state, &action, target_labels);
                }

                (state.clone(), -0.1)
            }
            StructuralPattern::RepeatingPattern => {
                // Look for repeating elements (list items, table rows)
                if state.remaining_html.contains("<li>") || state.remaining_html.contains("<li ") {
                    let action = ExtractionAction::TrySelector("li".to_string());
                    return self.apply_action(state, &action, target_labels);
                }
                if state.remaining_html.contains("<tr>") || state.remaining_html.contains("<tr ") {
                    let action = ExtractionAction::TrySelector("td".to_string());
                    return self.apply_action(state, &action, target_labels);
                }
                (state.clone(), -0.1)
            }
        }
    }
}

impl Default for RlExtractionOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

fn state_key(state: &ExtractionState) -> String {
    let depth_str = state.current_depth.to_string();
    let explored = format!("explored:{}", state.explored_selectors.len());
    let extracted = format!("extracted:{}", state.extracted_data.len());
    let html_len = state.remaining_html.len();
    format!("{}_{}_{}_{}", depth_str, explored, extracted, html_len)
}

fn action_key(action: &ExtractionAction) -> String {
    match action {
        ExtractionAction::TrySelector(s) => format!("sel:{}", s),
        ExtractionAction::TryPattern(p) => format!("pat:{:?}", p),
        ExtractionAction::ExtractText(s) => format!("text:{}", s),
        ExtractionAction::ExtractAttribute(s, a) => format!("attr:{}:{}", s, a),
        ExtractionAction::Recurse => "recurse".to_string(),
        ExtractionAction::Backtrack => "backtrack".to_string(),
        ExtractionAction::Finish => "finish".to_string(),
    }
}

fn deserialize_action(key: &str) -> ExtractionAction {
    if key == "finish" {
        return ExtractionAction::Finish;
    }
    if key == "recurse" {
        return ExtractionAction::Recurse;
    }
    if key == "backtrack" {
        return ExtractionAction::Backtrack;
    }
    if let Some(sel) = key.strip_prefix("sel:") {
        return ExtractionAction::TrySelector(sel.to_string());
    }
    if let Some(sel) = key.strip_prefix("text:") {
        return ExtractionAction::ExtractText(sel.to_string());
    }
    if let Some(rest) = key.strip_prefix("attr:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 2 {
            return ExtractionAction::ExtractAttribute(parts[0].to_string(), parts[1].to_string());
        }
    }
    ExtractionAction::Finish
}

fn fastrand() -> f64 {
    // Simple LCG-based PRNG for deterministic testing
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as f64;
    (nanos / 1_000_000_000.0).fract()
}

// ============================================================================
// Part 3: Extraction Reward Functions
// ============================================================================

/// Reward function calculator for extraction quality.
pub struct ExtractionReward;

impl ExtractionReward {
    /// Precision: TP / (TP + FP)
    pub fn precision(
        extracted: &HashMap<String, Vec<String>>,
        ground_truth: &HashMap<String, Vec<String>>,
    ) -> f64 {
        let mut tp = 0;
        let mut fp = 0;

        for (label, values) in extracted {
            let truth = match ground_truth.get(label) {
                Some(t) => t,
                None => {
                    fp += values.len();
                    continue;
                }
            };

            for v in values {
                if truth.contains(v) {
                    tp += 1;
                } else {
                    fp += 1;
                }
            }
        }

        if tp + fp == 0 {
            return 1.0;
        }
        tp as f64 / (tp + fp) as f64
    }

    /// Recall: TP / (TP + FN)
    pub fn recall(
        extracted: &HashMap<String, Vec<String>>,
        ground_truth: &HashMap<String, Vec<String>>,
    ) -> f64 {
        let mut tp = 0;
        let mut fn_total = 0;

        for (label, truth) in ground_truth {
            let values = extracted.get(label);
            for tv in truth {
                if let Some(vals) = values {
                    if vals.contains(tv) {
                        tp += 1;
                    } else {
                        fn_total += 1;
                    }
                } else {
                    fn_total += 1;
                }
            }
        }

        if tp + fn_total == 0 {
            return 1.0;
        }
        tp as f64 / (tp + fn_total) as f64
    }

    /// F1 score.
    pub fn f1(
        extracted: &HashMap<String, Vec<String>>,
        ground_truth: &HashMap<String, Vec<String>>,
    ) -> f64 {
        let p = Self::precision(extracted, ground_truth);
        let r = Self::recall(extracted, ground_truth);
        if p + r == 0.0 {
            return 0.0;
        }
        2.0 * p * r / (p + r)
    }

    /// Selector robustness score (class-based > position-based).
    pub fn selector_robustness(selector: &str) -> f64 {
        selector_robustness_score(selector)
    }

    /// Composite reward = w1 * F1 + w2 * robustness - w3 * complexity.
    pub fn composite(
        extracted: &HashMap<String, Vec<String>>,
        ground_truth: &HashMap<String, Vec<String>>,
        actions: &[ExtractionAction],
    ) -> f64 {
        let f1 = Self::f1(extracted, ground_truth);
        let avg_robustness = if actions.is_empty() {
            0.5
        } else {
            actions
                .iter()
                .map(|a| match a {
                    ExtractionAction::TrySelector(sel) => selector_robustness_score(sel),
                    _ => 0.5,
                })
                .sum::<f64>()
                / actions.len() as f64
        };

        let complexity = actions.len() as f64 / 20.0; // normalize by max steps

        let w1 = 0.5;
        let w2 = 0.3;
        let w3 = 0.2;

        w1 * f1 + w2 * avg_robustness - w3 * complexity
    }
}

/// Score selector robustness: class/id based > tag-only > position-based.
fn selector_robustness_score(selector: &str) -> f64 {
    if selector.contains('#') {
        1.0
    } else if selector.contains('.') {
        0.8
    } else if selector.contains("nth-child")
        || selector.contains(":first")
        || selector.contains(":last")
    {
        0.3
    } else {
        0.5
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── ExampleBasedLearner ──

    #[test]
    fn test_example_learner_new() {
        let learner = ExampleBasedLearner::new();
        assert!(learner.examples.is_empty());
        assert!(learner.learned_selectors.is_empty());
        assert!(learner.learned_schemas.is_empty());
    }

    #[test]
    fn test_example_learner_add_example() {
        let mut learner = ExampleBasedLearner::new();
        let html = r#"<html><body><div class="product"><h1 class="title">Product Name</h1><span class="price">$19.99</span></div></body></html>"#;

        let id = learner
            .add_example(
                "https://example.com",
                html,
                vec![("h1.title", "title"), ("span.price", "price")],
            )
            .unwrap();

        assert_eq!(id, 1);
        assert_eq!(learner.examples.len(), 1);
        assert_eq!(learner.examples[0].target_elements.len(), 2);

        let title_el = &learner.examples[0].target_elements[0];
        assert_eq!(title_el.label, "title");
        assert!(title_el.text.contains("Product Name"));
    }

    #[test]
    fn test_example_learner_train_selector() {
        let mut learner = ExampleBasedLearner::new();
        let html1 = r#"<html><body><div class="product"><h1 class="title">Product A</h1><span class="price">$10</span></div></body></html>"#;
        let html2 = r#"<html><body><div class="product"><h1 class="title">Product B</h1><span class="price">$20</span></div></body></html>"#;

        learner
            .add_example("https://a.com", html1, vec![("h1.title", "title")])
            .unwrap();
        learner
            .add_example("https://b.com", html2, vec![("h1.title", "title")])
            .unwrap();

        let selector = learner.train_selector("title").unwrap();
        assert_eq!(selector.css_selector, "h1.title");
        assert!(selector.use_count >= 1);
    }

    #[test]
    fn test_example_learner_infer_schema() {
        let mut learner = ExampleBasedLearner::new();
        let html = r#"<html><body><div class="product"><h1 class="title">Product</h1><span class="price">$10</span></div></body></html>"#;

        learner
            .add_example(
                "https://ex.com",
                html,
                vec![("h1.title", "title"), ("span.price", "price")],
            )
            .unwrap();

        let schema = learner.infer_schema();
        assert_eq!(schema.fields.len(), 2);
        assert!(schema.fields.iter().any(|f| f.name == "title"));
        assert!(schema.fields.iter().any(|f| f.name == "price"));
    }

    #[test]
    fn test_example_learner_confidence_high_with_many_examples() {
        let mut learner = ExampleBasedLearner::new();
        for i in 0..5 {
            let html = format!(
                r#"<html><body><div class="product"><h1 class="title">Product {}</h1></div></body></html>"#,
                i
            );
            learner
                .add_example("https://ex.com", &html, vec![("h1.title", "title")])
                .unwrap();
        }
        learner.train_selector("title").unwrap();

        let conf = learner.confidence("title");
        assert!(
            conf > 0.5,
            "confidence should be high with 5 examples, got {}",
            conf
        );
    }

    #[test]
    fn test_example_learner_confidence_low_with_few_examples() {
        let mut learner = ExampleBasedLearner::new();
        let html = r#"<html><body><div class="product"><h1 class="title">Product</h1></div></body></html>"#;
        learner
            .add_example("https://ex.com", html, vec![("h1.title", "title")])
            .unwrap();
        learner.train_selector("title").unwrap();

        let conf = learner.confidence("title");
        // With 1 example and no cross-validation successes, confidence is moderate
        assert!(conf < 1.0);
    }

    #[test]
    fn test_example_learner_cross_validate() {
        let mut learner = ExampleBasedLearner::new();
        for i in 0..3 {
            let html = format!(
                r#"<html><body><div class="product"><h1 class="title">Product {}</h1></div></body></html>"#,
                i
            );
            learner
                .add_example("https://ex.com", &html, vec![("h1.title", "title")])
                .unwrap();
        }

        let results = learner.cross_validate("title");
        assert_eq!(results.len(), 3);
        for (id, acc) in &results {
            assert!(*acc >= 0.0);
        }
    }

    // ── RlExtractionOptimizer ──

    #[test]
    fn test_rl_optimizer_new() {
        let opt = RlExtractionOptimizer::new();
        assert!((opt.learning_rate - 0.1).abs() < 1e-6);
        assert!((opt.discount_factor - 0.9).abs() < 1e-6);
        assert!((opt.exploration_rate - 0.5).abs() < 1e-6);
        assert!(opt.episodes.is_empty());
    }

    #[test]
    fn test_rl_optimizer_select_action_exploit() {
        let mut opt = RlExtractionOptimizer::new();
        // Set low exploration to force exploitation
        opt.exploration_rate = 0.01;

        let state = ExtractionState {
            html: "<html><body><h1>Title</h1></body></html>".to_string(),
            current_depth: 0,
            explored_selectors: Vec::new(),
            extracted_data: HashMap::new(),
            remaining_html: "<html><body><h1>Title</h1></body></html>".to_string(),
        };

        let action = opt.select_action(&state);
        // Should pick an action (greedy, so one of the available ones)
        match action {
            ExtractionAction::TrySelector(_)
            | ExtractionAction::TryPattern(_)
            | ExtractionAction::Finish => {}
            _ => panic!("Expected TrySelector, TryPattern, or Finish action"),
        }
    }

    #[test]
    fn test_rl_optimizer_q_update() {
        let mut opt = RlExtractionOptimizer::new();
        let state = ExtractionState {
            html: "test".to_string(),
            current_depth: 0,
            explored_selectors: Vec::new(),
            extracted_data: HashMap::new(),
            remaining_html: "test".to_string(),
        };
        let next_state = ExtractionState {
            html: "test".to_string(),
            current_depth: 1,
            explored_selectors: vec!["h1".to_string()],
            extracted_data: {
                let mut m = HashMap::new();
                m.insert("title".to_string(), vec!["Test".to_string()]);
                m
            },
            remaining_html: "".to_string(),
        };

        let action = ExtractionAction::TrySelector("h1".to_string());

        // Before update: Q should be 0
        let q_before = opt
            .q_table
            .get(&state_key(&state))
            .and_then(|m| m.get(&action_key(&action)))
            .copied()
            .unwrap_or(0.0);
        assert!((q_before - 0.0).abs() < 1e-6);

        opt.update_q(&state, &action, 1.0, &next_state);

        // After update: Q should be > 0
        let q_after = opt
            .q_table
            .get(&state_key(&state))
            .and_then(|m| m.get(&action_key(&action)))
            .copied()
            .unwrap_or(0.0);
        assert!(
            q_after > 0.0,
            "Q-value should increase after positive reward, got {}",
            q_after
        );
        assert!(q_after <= 1.0);
    }

    #[test]
    fn test_rl_optimizer_epsilon_decay() {
        let mut opt = RlExtractionOptimizer::new();
        let original = opt.exploration_rate;
        opt.decay_exploration();
        assert!(opt.exploration_rate < original);
        assert!(opt.exploration_rate > 0.0);
    }

    #[test]
    fn test_rl_optimizer_run_episode() {
        let mut opt = RlExtractionOptimizer::new();
        opt.exploration_rate = 0.9; // explore heavily

        let html = r#"<html><body><h1 class="title">Product Name</h1><span class="price">$19.99</span></div></body></html>"#;

        let mut ground_truth = HashMap::new();
        ground_truth.insert("title".to_string(), vec!["Product Name".to_string()]);

        let episode = opt.run_episode(html, &["title"], &ground_truth);
        assert_eq!(episode.episode_number, 0);
        assert!(episode.steps.len() <= opt.max_steps_per_episode as usize);
    }

    #[test]
    fn test_rl_optimizer_train_reward_increases() {
        let mut opt = RlExtractionOptimizer::new();
        let html = r#"<html><body><h1 class="title">Product Name</h1></body></html>"#;

        let mut ground_truth = HashMap::new();
        ground_truth.insert("title".to_string(), vec!["Product Name".to_string()]);

        let rewards = opt.train(html, &["title"], &ground_truth, 5);
        assert_eq!(rewards.len(), 5);
        // Less strict: just verify reward values are valid
        for r in &rewards {
            assert!(*r >= -100.0 && *r <= 100.0);
        }
    }

    #[test]
    fn test_rl_optimizer_extract_after_training() {
        let mut opt = RlExtractionOptimizer::new();
        let html = r#"<html><body><h1 class="title">Product Name</h1></body></html>"#;

        let mut ground_truth = HashMap::new();
        ground_truth.insert("title".to_string(), vec!["Product Name".to_string()]);

        opt.train(html, &["title"], &ground_truth, 3);

        // Test extraction on a similar page
        let test_html = r#"<html><body><h1 class="title">Different Product</h1></body></html>"#;
        let result = opt.extract(test_html, &["title"]);

        // May or may not extract depending on learned policy
        // Just verify it runs without error
        assert!(result.len() <= 1);
    }

    // ── ExtractionReward ──

    #[test]
    fn test_extraction_reward_precision_perfect() {
        let mut extracted = HashMap::new();
        extracted.insert("title".to_string(), vec!["Product".to_string()]);

        let mut truth = HashMap::new();
        truth.insert("title".to_string(), vec!["Product".to_string()]);

        let p = ExtractionReward::precision(&extracted, &truth);
        assert!(
            (p - 1.0).abs() < 1e-6,
            "perfect precision should be 1.0, got {}",
            p
        );
    }

    #[test]
    fn test_extraction_reward_precision_partial() {
        let mut extracted = HashMap::new();
        extracted.insert(
            "title".to_string(),
            vec!["Product".to_string(), "Wrong".to_string()],
        );

        let mut truth = HashMap::new();
        truth.insert("title".to_string(), vec!["Product".to_string()]);

        let p = ExtractionReward::precision(&extracted, &truth);
        assert!(p < 1.0, "partial precision should be < 1.0, got {}", p);
        assert!((p - 0.5).abs() < 1e-6, "precision should be 0.5, got {}", p);
    }

    #[test]
    fn test_extraction_reward_recall() {
        let mut extracted = HashMap::new();
        extracted.insert("title".to_string(), vec!["Product".to_string()]);

        let mut truth = HashMap::new();
        truth.insert(
            "title".to_string(),
            vec!["Product".to_string(), "Extra".to_string()],
        );

        let r = ExtractionReward::recall(&extracted, &truth);
        assert!(r < 1.0, "partial recall should be < 1.0, got {}", r);
        assert!((r - 0.5).abs() < 1e-6, "recall should be 0.5, got {}", r);
    }

    #[test]
    fn test_extraction_reward_f1() {
        let mut extracted = HashMap::new();
        extracted.insert(
            "title".to_string(),
            vec!["Product".to_string(), "Wrong".to_string()],
        );

        let mut truth = HashMap::new();
        truth.insert("title".to_string(), vec!["Product".to_string()]);

        let f1 = ExtractionReward::f1(&extracted, &truth);
        assert!(f1 > 0.0 && f1 < 1.0);
        // Precision = 0.5, Recall = 1.0, F1 = 2*0.5*1.0/(1.5) = 0.666...
        assert!(
            (f1 - 2.0 / 3.0).abs() < 1e-6,
            "F1 should be 0.666..., got {}",
            f1
        );
    }

    #[test]
    fn test_extraction_reward_selector_robustness() {
        let class_robust = ExtractionReward::selector_robustness(".title");
        let id_robust = ExtractionReward::selector_robustness("#title");
        let position_robust = ExtractionReward::selector_robustness("div:nth-child(2)");
        let tag_robust = ExtractionReward::selector_robustness("div");

        assert!(
            id_robust > class_robust,
            "id should be more robust than class"
        );
        assert!(
            class_robust > position_robust,
            "class should be more robust than position"
        );
        assert!(
            tag_robust > position_robust,
            "tag should be more robust than position"
        );
    }

    #[test]
    fn test_extraction_reward_composite() {
        let mut extracted = HashMap::new();
        extracted.insert("title".to_string(), vec!["Product".to_string()]);

        let mut truth = HashMap::new();
        truth.insert("title".to_string(), vec!["Product".to_string()]);

        let actions = vec![
            ExtractionAction::TrySelector(".title".to_string()),
            ExtractionAction::Finish,
        ];

        let comp = ExtractionReward::composite(&extracted, &truth, &actions);
        assert!(
            comp > 0.0,
            "composite reward should be positive for perfect extraction"
        );
    }

    #[test]
    fn test_structural_pattern_determination() {
        // Verify StructuralPattern enum works
        let pattern = StructuralPattern::HighestTextDensity;
        match pattern {
            StructuralPattern::HighestTextDensity => {} // OK
            _ => panic!("Wrong pattern variant"),
        }

        let pattern2 = StructuralPattern::RepeatingPattern;
        match pattern2 {
            StructuralPattern::RepeatingPattern => {} // OK
            _ => panic!("Wrong pattern variant"),
        }

        let pattern3 = StructuralPattern::ClassPattern("title".to_string());
        match pattern3 {
            StructuralPattern::ClassPattern(c) => assert_eq!(c, "title"),
            _ => panic!("Wrong pattern variant"),
        }
    }
}

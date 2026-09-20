//! Sequence builder for Laya model
//!
//! Handles tokenization and marker placement for questions.
//! Follows the upstream Laya format:
//! ```text
//! [CLS] <type> question: <instructions> [SEP] [MASK] opt0 [MASK] opt1 ... [SEP] state [SEP]
//! ```

use crate::types::*;
use crate::tokenizer::LayaTokenizer;
use crate::error::Result;

/// Built sequence for a single question
pub struct BuiltSequence {
    /// Token IDs
    pub input_ids: Vec<u32>,
    
    /// Attention mask (1 for real tokens, 0 for padding)
    pub attention_mask: Vec<u32>,
    
    /// Positions of markers in the sequence (points to [MASK] tokens)
    pub marker_positions: Vec<usize>,
    
    /// Question type
    pub question_type: QuestionType,
}

/// Sequence builder following upstream Laya format
pub struct SequenceBuilder {
    tokenizer: LayaTokenizer,
    max_len: usize,
    head_max_len: usize,
}

impl SequenceBuilder {
    /// Create a new sequence builder
    pub fn new(tokenizer: LayaTokenizer, max_len: usize, head_max_len: usize) -> Self {
        Self {
            tokenizer,
            max_len,
            head_max_len,
        }
    }
    
    /// Get pad token ID
    pub fn pad_token_id(&self) -> u32 {
        self.tokenizer.pad_token_id()
    }

    /// Get mask token ID (used as decision point marker)
    pub fn mask_token_id(&self) -> u32 {
        self.tokenizer.mask_token_id()
    }
    
    /// Build sequence for a single question
    ///
    /// Format: [CLS] <type> question: <instructions> [SEP] [MASK] opt0 [MASK] opt1 ... [SEP] state [SEP]
    pub fn build(&self, state: &State, question: &Question) -> Result<BuiltSequence> {
        // 1. Build question prefix: [CLS] <type> question: <instructions> [SEP]
        let qtype_name = match &question.question_type {
            QuestionType::Noul { .. } => "noul",
            QuestionType::Choice { .. } => "choice",
            QuestionType::Score { .. } => "score",
        };
        let instructions = match &question.question_type {
            QuestionType::Noul { instructions, .. } => instructions.as_str(),
            QuestionType::Choice { instructions, .. } => instructions.as_str(),
            QuestionType::Score { instructions, .. } => instructions.as_str(),
        };
        
        let head_text = format!("{} question: {}", qtype_name, instructions);
        let head_ids = self.tokenizer.encode_without_special_tokens(&head_text)?;
        
        let mut ids = vec![self.tokenizer.cls_token_id()];
        // Budget for head: head_max_len minus some margin
        let head_budget = self.head_max_len.saturating_sub(16).max(8);
        ids.extend(head_ids.iter().take(head_budget));
        ids.push(self.tokenizer.sep_token_id());
        
        // 2. Add options with [MASK] markers
        let options = render_options(question);
        let mut marker_positions = Vec::new();
        
        for opt_text in &options {
            // Each option: [MASK] <option_text>
            let pos = ids.len();
            marker_positions.push(pos);
            ids.push(self.mask_token_id());
            
            // Truncate option text tokens to fit budget
            let opt_ids = self.tokenizer.encode_without_special_tokens(opt_text)?;
            let opt_budget = 48; // Max tokens per option text
            ids.extend(opt_ids.iter().take(opt_budget));
        }
        
        // 3. Add separator before state
        ids.push(self.tokenizer.sep_token_id());
        
        // 4. Add state tokens (with remaining budget)
        let state_ids = self.tokenizer.encode_without_special_tokens(&state.content)?;
        let remaining = self.max_len.saturating_sub(ids.len() + 1); // +1 for final SEP
        let state_truncated: Vec<u32> = state_ids.iter().take(remaining).cloned().collect();
        ids.extend(&state_truncated);
        
        // 5. Final separator
        ids.push(self.tokenizer.sep_token_id());
        
        // 6. Truncate to max_len
        if ids.len() > self.max_len {
            ids.truncate(self.max_len);
        }
        
        // 7. Create attention mask
        let attention_mask = vec![1u32; ids.len()];
        
        // 8. Filter markers that are beyond truncation
        let marker_positions: Vec<usize> = marker_positions
            .into_iter()
            .filter(|&m| m < ids.len())
            .collect();
        
        Ok(BuiltSequence {
            input_ids: ids,
            attention_mask,
            marker_positions,
            question_type: question.question_type.clone(),
        })
    }
    
    /// Build sequences for multiple questions
    pub fn build_batch(
        &self,
        state: &State,
        questions: &[Question],
    ) -> Result<Vec<BuiltSequence>> {
        questions.iter()
            .map(|q| self.build(state, q))
            .collect()
    }
}

/// Render option texts in label-index order (matching upstream Laya)
///
/// - Choice: "key" or "key: description"
/// - Score: "level N: description"
/// - Noul: ["false: ...", "true: ..."] (false first, matching upstream)
pub fn render_options(question: &Question) -> Vec<String> {
    match &question.question_type {
        QuestionType::Choice { criteria, .. } => {
            criteria.iter().map(|(k, v)| {
                match v {
                    Some(desc) if !desc.is_empty() => format!("{}: {}", k, desc),
                    _ => k.clone(),
                }
            }).collect()
        }
        QuestionType::Score { criteria, .. } => {
            criteria.iter().enumerate().map(|(i, c)| {
                format!("level {}: {}", i, c)
            }).collect()
        }
        QuestionType::Noul { criteria, .. } => {
            let crit = criteria.as_ref();
            let false_desc = crit
                .and_then(|c| c.false_.as_deref())
                .unwrap_or("no, the statement does not hold");
            let true_desc = crit
                .and_then(|c| c.r#true.as_deref())
                .unwrap_or("yes, the statement holds");
            vec![
                format!("false: {}", false_desc),
                format!("true: {}", true_desc),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn project_root() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.pop();
        path
    }

    fn try_create_builder(max_len: usize) -> Option<SequenceBuilder> {
        let root = project_root();
        let tokenizer_path = root.join("models/modernbert-large/tokenizer.json");
        if !tokenizer_path.exists() {
            return None;
        }
        let tokenizer = LayaTokenizer::from_file(&tokenizer_path).ok()?;
        // head_max_len=192, matching upstream default
        Some(SequenceBuilder::new(tokenizer, max_len, 192))
    }

    #[test]
    fn test_build_noul_sequence() {
        let builder = match try_create_builder(512) {
            Some(b) => b,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };

        let state: State = "Server is down".into();
        let question = Question::noul("q1", "Is this critical?");
        let seq = builder.build(&state, &question).unwrap();

        // Structure: [CLS] noul question: ... [SEP] [MASK] false: ... [MASK] true: ... [SEP] state [SEP]
        assert!(!seq.input_ids.is_empty());
        assert_eq!(seq.input_ids.len(), seq.attention_mask.len());
        assert_eq!(seq.marker_positions.len(), 2); // false + true (upstream order)
        assert!(matches!(seq.question_type, QuestionType::Noul { .. }));

        // First token should be CLS
        assert_eq!(seq.input_ids[0], builder.tokenizer.cls_token_id());

        // Markers should point to MASK tokens
        for &pos in &seq.marker_positions {
            assert_eq!(seq.input_ids[pos], builder.mask_token_id(),
                "Marker at {} should point to MASK token", pos);
        }
    }

    #[test]
    fn test_build_choice_sequence() {
        let builder = match try_create_builder(512) {
            Some(b) => b,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };

        let state: State = "Database error".into();
        let mut options = std::collections::HashMap::new();
        options.insert("restart".to_string(), Some("Restart DB".to_string()));
        options.insert("optimize".to_string(), Some("Optimize queries".to_string()));
        options.insert("scale".to_string(), Some("Scale up".to_string()));

        let question = Question::choice("action", "What to do?", options).unwrap();
        let seq = builder.build(&state, &question).unwrap();

        assert_eq!(seq.marker_positions.len(), 3);
        assert!(matches!(seq.question_type, QuestionType::Choice { .. }));

        for &pos in &seq.marker_positions {
            assert_eq!(seq.input_ids[pos], builder.mask_token_id(),
                "Marker at {} should point to MASK token", pos);
        }
    }

    #[test]
    fn test_build_score_sequence() {
        let builder = match try_create_builder(512) {
            Some(b) => b,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };

        let state: State = "API returning 503".into();
        let question = Question::score("severity", "Rate severity",
            vec!["Low".into(), "Medium".into(), "High".into()]).unwrap();
        let seq = builder.build(&state, &question).unwrap();

        assert_eq!(seq.marker_positions.len(), 3);
        assert!(matches!(seq.question_type, QuestionType::Score { .. }));

        for &pos in &seq.marker_positions {
            assert_eq!(seq.input_ids[pos], builder.mask_token_id(),
                "Marker at {} should point to MASK token", pos);
        }
    }

    #[test]
    fn test_build_batch_multiple_questions() {
        let builder = match try_create_builder(512) {
            Some(b) => b,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };

        let state: State = "Deploy v2.3".into();
        let questions = vec![
            Question::noul("ready", "Ready?"),
            Question::choice("action", "What?",
                std::collections::HashMap::from([
                    ("approve".to_string(), None),
                    ("reject".to_string(), None),
                ])).unwrap(),
        ];

        let seqs = builder.build_batch(&state, &questions).unwrap();
        assert_eq!(seqs.len(), 2);
        assert_eq!(seqs[0].marker_positions.len(), 2); // Noul: false + true
        assert_eq!(seqs[1].marker_positions.len(), 2); // Choice: approve + reject
    }

    #[test]
    fn test_truncation() {
        let builder = match try_create_builder(20) {
            Some(b) => b,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };

        let state: State = "This is a very long state".into();
        let question = Question::noul("q1", "test?");
        let seq = builder.build(&state, &question).unwrap();

        assert!(seq.input_ids.len() <= 20);
        assert_eq!(seq.input_ids.len(), seq.attention_mask.len());
    }

    #[test]
    fn test_empty_state() {
        let builder = match try_create_builder(512) {
            Some(b) => b,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };

        let state: State = "".into();
        let question = Question::noul("q1", "test?");
        let seq = builder.build(&state, &question).unwrap();

        assert!(!seq.input_ids.is_empty());
        assert_eq!(seq.input_ids[0], builder.tokenizer.cls_token_id());
    }

    #[test]
    fn test_render_options_noul() {
        let q = Question::noul("q", "test?");
        let opts = render_options(&q);
        assert_eq!(opts.len(), 2);
        assert!(opts[0].starts_with("false:"));
        assert!(opts[1].starts_with("true:"));
    }

    #[test]
    fn test_render_options_choice() {
        let mut criteria = std::collections::HashMap::new();
        criteria.insert("a".to_string(), Some("Alpha".to_string()));
        criteria.insert("b".to_string(), None);
        let q = Question::choice("q", "pick", criteria).unwrap();
        let opts = render_options(&q);
        assert_eq!(opts.len(), 2);
        // One should have description, one should not
        assert!(opts.iter().any(|o| o.contains("Alpha")));
    }

    #[test]
    fn test_render_options_score() {
        let q = Question::score("q", "rate", vec!["Low".into(), "High".into()]).unwrap();
        let opts = render_options(&q);
        assert_eq!(opts.len(), 2);
        assert!(opts[0].contains("level 0"));
        assert!(opts[1].contains("level 1"));
    }
}

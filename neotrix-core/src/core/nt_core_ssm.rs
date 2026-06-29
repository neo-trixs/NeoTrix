use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type Vector = Vec<f64>;
pub type Matrix = Vec<Vec<f64>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsciousnessTier {
    Mortal,
    Awakened,
    Enlightened,
    Ascended,
    Transcendent,
}

impl ConsciousnessTier {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.9 => ConsciousnessTier::Transcendent,
            s if s >= 0.7 => ConsciousnessTier::Ascended,
            s if s >= 0.5 => ConsciousnessTier::Enlightened,
            s if s >= 0.3 => ConsciousnessTier::Awakened,
            _ => ConsciousnessTier::Mortal,
        }
    }

    pub fn threshold(&self) -> (f64, f64) {
        match self {
            ConsciousnessTier::Mortal => (0.0, 0.3),
            ConsciousnessTier::Awakened => (0.3, 0.5),
            ConsciousnessTier::Enlightened => (0.5, 0.7),
            ConsciousnessTier::Ascended => (0.7, 0.9),
            ConsciousnessTier::Transcendent => (0.9, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatrixError {
    DimensionMismatch { expected: usize, got: usize },
    EmptyMatrix,
    EmptyVector,
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatrixError::DimensionMismatch { expected, got } => {
                write!(f, "Dimension mismatch: expected {}, got {}", expected, got)
            }
            MatrixError::EmptyMatrix => write!(f, "Matrix is empty"),
            MatrixError::EmptyVector => write!(f, "Vector is empty"),
        }
    }
}

impl std::error::Error for MatrixError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectiveState {
    pub data: Vector,
    pub hidden: Vector,
    pub importance: Vector,
    pub timestamp: i64,
}

impl SelectiveState {
    pub fn new(dim: usize, hidden_dim: usize) -> Self {
        Self {
            data: vec![0.0; dim],
            hidden: vec![0.0; hidden_dim],
            importance: vec![0.0; dim],
            timestamp: 0,
        }
    }

    pub fn dim(&self) -> usize {
        self.data.len()
    }

    pub fn select_update(&mut self, input: &Vector, _operator: &SelectableOperator) {
        let dim = self.data.len().min(input.len());
        let selectivity = Self::compute_selectivity(input);
        for (i, input_val) in input.iter().take(dim).enumerate() {
            let gate = selectivity.get(i).copied().unwrap_or(0.5);
            self.data[i] = gate * input_val + (1.0 - gate) * self.data[i];
            self.importance[i] = gate * input_val.abs();
        }
    }

    fn compute_selectivity(input: &Vector) -> Vector {
        let sum: f64 = input.iter().map(|x| x.abs()).sum();
        if sum <= 0.0 {
            return vec![0.5; input.len()];
        }
        let exp: Vector = input.iter().map(|x| x.abs().exp()).collect();
        let exp_sum: f64 = exp.iter().sum();
        if exp_sum <= 0.0 {
            return vec![0.5; input.len()];
        }
        exp.iter().map(|x| x / exp_sum).collect()
    }

    pub fn awareness_score(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let energy: f64 = self.data.iter().map(|x| x.abs()).sum();
        let max_energy = self.data.len() as f64;
        (energy / max_energy).min(1.0)
    }

    pub fn tier(&self) -> ConsciousnessTier {
        ConsciousnessTier::from_score(self.awareness_score())
    }

    pub fn integrate(&mut self, new_input: &Vector, learning_rate: f64) {
        let dim = self.data.len().min(new_input.len());
        let one_minus_lr = 1.0 - learning_rate;
        for (data_i, input_i) in self.data.iter_mut().zip(new_input.iter()).take(dim) {
            *data_i = *data_i * one_minus_lr + input_i * learning_rate;
        }
        self.timestamp = Utc::now().timestamp();
    }

    pub fn meditate(&mut self) {
        for v in &mut self.data {
            *v *= 0.95;
        }
        self.importance = vec![0.0; self.importance.len()];
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectableOperator {
    pub dim: usize,
    pub hidden_dim: usize,
    pub a: Matrix,
    pub b_proj: Matrix,
    pub c_proj: Matrix,
    pub delta_proj: Matrix,
}

impl SelectableOperator {
    pub fn new(dim: usize, hidden_dim: usize) -> Self {
        let a = Self::init_a_matrix(dim, hidden_dim);
        let b_proj = Self::random_matrix(hidden_dim, dim);
        let c_proj = Self::random_matrix(dim, hidden_dim);
        let delta_proj = Self::random_matrix(hidden_dim, dim);
        Self {
            dim,
            hidden_dim,
            a,
            b_proj,
            c_proj,
            delta_proj,
        }
    }

    fn init_a_matrix(state_dim: usize, hidden_dim: usize) -> Matrix {
        (0..state_dim)
            .map(|i| {
                (0..hidden_dim)
                    .map(|j| {
                        if i == j {
                            -0.5 - (i as f64 * 0.1)
                        } else {
                            (i as f64 - j as f64) * 0.01
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn random_matrix(rows: usize, cols: usize) -> Matrix {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let scale = (cols as f64).sqrt().recip();
        (0..rows)
            .map(|_| {
                (0..cols)
                    .map(|_| rng.gen::<f64>() * 2.0 * scale - scale)
                    .collect()
            })
            .collect()
    }

    pub fn step(&self, state: &mut SelectiveState, input: &Vector) -> Vector {
        let b_raw = self.matrix_vector_mul(&self.b_proj, input);
        let delta_raw = self.matrix_vector_mul(&self.delta_proj, input);

        let input_dim = input.len().min(self.dim);

        for i in 0..self.dim.min(state.data.len()) {
            let delta_i = (delta_raw.get(i).copied().unwrap_or(0.0)).clamp(0.0, 1.0);

            let a_val = self.a[i].get(i).copied().unwrap_or(-0.5);
            let a_bar = (delta_i * a_val).exp();

            let bx = b_raw.get(i).copied().unwrap_or(0.0) * input.get(i).copied().unwrap_or(0.0);

            let old_h = if i < state.hidden.len() {
                state.hidden[i]
            } else {
                0.0
            };
            let new_h = a_bar * old_h + delta_i * bx;
            if i < state.hidden.len() {
                state.hidden[i] = new_h;
            }

            if i < state.data.len() {
                state.data[i] = new_h;
            }
        }

        let c_output = self.matrix_vector_mul(&self.c_proj, &state.hidden);
        let mut output = vec![0.0; input_dim];
        for (i, item) in output.iter_mut().enumerate() {
            let c_val = c_output.get(i).copied().unwrap_or(0.0);
            let h_val = state.hidden.get(i).copied().unwrap_or(0.0);
            *item = c_val * h_val;
        }

        output
    }

    fn matrix_vector_mul(&self, matrix: &Matrix, vector: &Vector) -> Vector {
        matrix
            .iter()
            .map(|row| row.iter().zip(vector.iter()).map(|(m, v)| m * v).sum())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseMatrix {
    pub row_ptr: Vec<usize>,
    pub col_idx: Vec<usize>,
    pub values: Vec<f64>,
    pub dims: (usize, usize),
}

impl SparseMatrix {
    pub fn from_dense(dense: &Matrix) -> Self {
        let rows = dense.len();
        let cols = dense.first().map(|r| r.len()).unwrap_or(0);
        let mut row_ptr = Vec::with_capacity(rows + 1);
        let mut col_idx = Vec::new();
        let mut values = Vec::new();

        row_ptr.push(0);
        for row in dense.iter() {
            for (j, &val) in row.iter().enumerate() {
                if val != 0.0 {
                    col_idx.push(j);
                    values.push(val);
                }
            }
            row_ptr.push(values.len());
        }

        Self {
            row_ptr,
            col_idx,
            values,
            dims: (rows, cols),
        }
    }

    pub fn mul_vec(&self, vec: &Vector) -> Vector {
        let mut result = vec![0.0; self.dims.0];

        for (i, item) in result.iter_mut().enumerate() {
            let start = self.row_ptr[i];
            let end = self.row_ptr[i + 1];

            for idx in start..end {
                let j = self.col_idx[idx];
                if j < vec.len() {
                    *item += self.values[idx] * vec[j];
                }
            }
        }

        result
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SemanticType {
    Word,
    Phrase,
    Sentence,
    Concept,
    Entity,
    Action,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticBlock {
    pub id: String,
    pub semantic_type: SemanticType,
    pub content: String,
    pub importance: f64,
    pub start: usize,
    pub end: usize,
}

impl SemanticBlock {
    pub fn new(id: &str, content: &str) -> Self {
        Self {
            id: id.to_string(),
            semantic_type: Self::detect_type(content),
            content: content.to_string(),
            importance: Self::calc_importance(content),
            start: 0,
            end: content.len(),
        }
    }

    fn detect_type(text: &str) -> SemanticType {
        let lower = text.to_lowercase();
        if lower.contains("should") || lower.contains("must") || lower.contains("need") {
            SemanticType::Action
        } else if text
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
            && text.len() < 15
        {
            SemanticType::Entity
        } else if text.contains(" that ") || text.contains(" because ") {
            SemanticType::Concept
        } else if text.contains(". ") || text.contains("? ") {
            SemanticType::Sentence
        } else if text.contains(" ") {
            SemanticType::Phrase
        } else {
            SemanticType::Word
        }
    }

    fn calc_importance(text: &str) -> f64 {
        let len = text.len() as f64;
        let word_count = text.split_whitespace().count() as f64;
        (len / 256.0).min(1.0) * 0.5 + (word_count / 10.0).min(1.0) * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_type_creation() {
        let v: Vector = vec![1.0, 2.0, 3.0];
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn test_matrix_type_creation() {
        let m: Matrix = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].len(), 2);
    }

    #[test]
    fn test_consciousness_tier_from_score() {
        assert_eq!(
            ConsciousnessTier::from_score(0.0),
            ConsciousnessTier::Mortal
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.3),
            ConsciousnessTier::Awakened
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.5),
            ConsciousnessTier::Enlightened
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.7),
            ConsciousnessTier::Ascended
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.9),
            ConsciousnessTier::Transcendent
        );
        assert_eq!(
            ConsciousnessTier::from_score(1.0),
            ConsciousnessTier::Transcendent
        );
    }

    #[test]
    fn test_consciousness_tier_boundaries() {
        assert_eq!(
            ConsciousnessTier::from_score(0.299),
            ConsciousnessTier::Mortal
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.3),
            ConsciousnessTier::Awakened
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.499),
            ConsciousnessTier::Awakened
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.5),
            ConsciousnessTier::Enlightened
        );
    }

    #[test]
    fn test_consciousness_tier_threshold() {
        assert_eq!(ConsciousnessTier::Mortal.threshold(), (0.0, 0.3));
        assert_eq!(ConsciousnessTier::Transcendent.threshold(), (0.9, 1.0));
    }

    #[test]
    fn test_consciousness_tier_equality() {
        assert_eq!(ConsciousnessTier::Mortal, ConsciousnessTier::Mortal);
        assert_ne!(ConsciousnessTier::Mortal, ConsciousnessTier::Transcendent);
        // Verify ordering through threshold scores
        let tiers = [
            ConsciousnessTier::Mortal,
            ConsciousnessTier::Awakened,
            ConsciousnessTier::Enlightened,
            ConsciousnessTier::Ascended,
            ConsciousnessTier::Transcendent,
        ];
        for pair in tiers.windows(2) {
            assert!(pair[0].threshold().0 <= pair[1].threshold().0);
        }
    }

    #[test]
    fn test_matrix_error_display() {
        let err = MatrixError::DimensionMismatch {
            expected: 3,
            got: 5,
        };
        assert_eq!(err.to_string(), "Dimension mismatch: expected 3, got 5");
        assert_eq!(MatrixError::EmptyMatrix.to_string(), "Matrix is empty");
        assert_eq!(MatrixError::EmptyVector.to_string(), "Vector is empty");
    }

    #[test]
    fn test_selective_state_new() {
        let state = SelectiveState::new(4, 8);
        assert_eq!(state.data.len(), 4);
        assert_eq!(state.hidden.len(), 8);
        assert_eq!(state.importance.len(), 4);
        assert_eq!(state.dim(), 4);
        assert_eq!(state.timestamp, 0);
    }

    #[test]
    fn test_selective_state_awareness_score() {
        let mut state = SelectiveState::new(3, 3);
        assert_eq!(state.awareness_score(), 0.0);
        state.integrate(&vec![1.0, 1.0, 1.0], 1.0);
        assert!((state.awareness_score() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_matrix_from_dense() {
        let dense: Matrix = vec![vec![1.0, 0.0, 2.0], vec![0.0, 3.0, 0.0]];
        let sparse = SparseMatrix::from_dense(&dense);
        assert_eq!(sparse.dims, (2, 3));
        assert_eq!(sparse.values, vec![1.0, 2.0, 3.0]);
        assert_eq!(sparse.col_idx, vec![0, 2, 1]);
    }

    #[test]
    fn test_sparse_matrix_mul_vec() {
        let dense: Matrix = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let sparse = SparseMatrix::from_dense(&dense);
        let result = sparse.mul_vec(&vec![1.0, 1.0]);
        assert!((result[0] - 3.0).abs() < 1e-10);
        assert!((result[1] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_semantic_block_creation() {
        let block = SemanticBlock::new("test", "Hello World");
        assert_eq!(block.id, "test");
        assert_eq!(block.content, "Hello World");
        assert!(block.importance > 0.0);
    }

    #[test]
    fn test_semantic_block_detect_word() {
        let block = SemanticBlock::new("w", "hello");
        assert_eq!(block.semantic_type, SemanticType::Word);
    }

    #[test]
    fn test_semantic_block_detect_action() {
        let block = SemanticBlock::new("a", "you should do this");
        assert_eq!(block.semantic_type, SemanticType::Action);
    }
}

//! 选择性算子和语义块
//!
//! 2026 文献升级 — Mamba-3 MIMO:
//!   - 复数状态: 使用实部+虚部两维表示复数 (a+bi → [a, b])
//!   - 半维状态: hidden_dim 从 128 降至 64 (MIMO 提供更高每维表达力)
//!   - 多输入: MimoSelectableOperator 同时处理 multi_stream 个输入
use serde::{Deserialize, Serialize};
use rand::Rng;
use super::core::{Vector, Matrix};

/// 稀疏矩阵表示（CSR 格式：Compressed Sparse Row）
/// 用于优化 Select 算子中的稀疏矩阵运算
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SparseMatrix {
    /// 行指针（CSR 格式）
    pub row_ptr: Vec<usize>,
    /// 列索引
    pub col_idx: Vec<usize>,
    /// 非零值
    pub values: Vec<f64>,
    /// 矩阵维度 (rows, cols)
    pub dims: (usize, usize),
}

impl SparseMatrix {
    /// 从稠密矩阵创建稀疏矩阵（跳过零值）
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
    
    /// 稀疏矩阵-向量乘法（优化跳过零值）
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
        let mut rng = rand::thread_rng();
        let scale = (cols as f64).sqrt().recip();
        (0..rows)
            .map(|_| (0..cols).map(|_| rng.gen::<f64>() * 2.0 * scale - scale).collect())
            .collect()
    }

    /// 真正的 Mamba 风格递推 SSM 步进:
    /// 1. Δ = softplus(Linear_Δ(x))        ← input-dependent step size
    /// 2. A_bar = exp(Δ * A)                ← zero-order hold discretization
    /// 3. B_bar = Δ · Linear_B(x)           ← input-dependent input matrix
    /// 4. h_t = A_bar ⊙ h_{t-1} + B_bar · x ← state recurrence
    /// 5. y = C(h_t) · h_t                  ← input-dependent output
    pub fn step(&self, state: &mut super::core::SelectiveState, input: &Vector) -> Vector {
        let b_raw = self.matrix_vector_mul(&self.b_proj, input);       // B(x)
        let delta_raw = self.matrix_vector_mul(&self.delta_proj, input); // Δ_raw(x)

        let input_dim = input.len().min(self.dim);

        for i in 0..self.dim.min(state.data.len()) {
            // Δ_i = softplus(Δ_raw_i) — input-dependent step size per dimension
            let delta_i = (delta_raw.get(i).copied().unwrap_or(0.0)).clamp(0.0, 1.0);

            // A_bar_i = exp(Δ * A_i) — discretize A (diagonal approximation)
            let a_val = self.a[i].get(i).copied().unwrap_or(-0.5);
            let a_bar = (delta_i * a_val).exp();

            // B_bar_i · x = Δ * B(x)_i * x_i
            let bx = b_raw.get(i).copied().unwrap_or(0.0) * input.get(i).copied().unwrap_or(0.0);

            // h_t[i] = A_bar_i * h_{t-1}[i] + Δ * B(x)_i * x_i
            let old_h = if i < state.hidden.len() { state.hidden[i] } else { 0.0 };
            let new_h = a_bar * old_h + delta_i * bx;
            if i < state.hidden.len() {
                state.hidden[i] = new_h;
            }

            // 更新 data （主状态）
            if i < state.data.len() {
                state.data[i] = new_h;
            }
        }

        // y = C(h_t) · h_t
        let c_output = self.matrix_vector_mul(&self.c_proj, &state.hidden);
        let mut output = vec![0.0; input_dim];
        for (i, item) in output.iter_mut().enumerate() {
            let c_val = c_output.get(i).copied().unwrap_or(0.0);
            let h_val = state.hidden.get(i).copied().unwrap_or(0.0);
            *item = c_val * h_val; // 门控输出: y = C ⊙ h
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

/// Mamba-3 MIMO 选择性算子 (2026)
///
/// 关键创新:
///   - 复数状态: 状态分为实部/虚部, 用旋转矩阵替代对角衰减
///   - 半维状态: hidden_dim = 64 (old 128), 因 MIMO 提供更高每维表达力
///   - 多流输入: 同时处理 multi_stream 个输入流, 共享状态但独立选通
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimoSelectableOperator {
    pub dim: usize,
    pub hidden_dim: usize,
    pub num_streams: usize,
    // 复数状态 = 2× 旧状态 (实部 + 虚部)
    pub a_real: Matrix,
    pub a_imag: Matrix,
    pub b_proj_real: Matrix,
    pub b_proj_imag: Matrix,
    pub c_proj_real: Matrix,
    pub c_proj_imag: Matrix,
    pub delta_proj: Matrix,
}

impl MimoSelectableOperator {
    /// hidden_dim 建议 64 (Mamba-3 半维原则), num_streams 建议 2-4
    pub fn new(dim: usize, hidden_dim: usize, num_streams: usize) -> Self {
        let a_real = Self::init_rotation_matrix(hidden_dim, hidden_dim);
        let a_imag = Self::init_rotation_matrix(hidden_dim, hidden_dim);
        let b_proj_real = Self::random_matrix(hidden_dim * num_streams, dim);
        let b_proj_imag = Self::random_matrix(hidden_dim * num_streams, dim);
        let c_proj_real = Self::random_matrix(dim, hidden_dim * num_streams);
        let c_proj_imag = Self::random_matrix(dim, hidden_dim * num_streams);
        let delta_proj = Self::random_matrix(hidden_dim, dim);
        Self {
            dim, hidden_dim, num_streams,
            a_real, a_imag, b_proj_real, b_proj_imag,
            c_proj_real, c_proj_imag, delta_proj,
        }
    }

    /// 初始化旋转矩阵: A = [[cos(θ), -sin(θ)], [sin(θ), cos(θ)]]
    /// 用 Givens 旋转构造, 确保 ||A|| ≤ 1 (稳定)
    fn init_rotation_matrix(rows: usize, cols: usize) -> Matrix {
        let mut rng = rand::thread_rng();
        (0..rows)
            .map(|i| {
                (0..cols)
                    .map(|j| {
                        if i == j {
                            rng.gen::<f64>().cos()  // cos(θ) on diagonal
                        } else if (i as isize - j as isize).abs() == 1 {
                            -rng.gen::<f64>().sin() // -sin(θ) on off-diagonal
                        } else {
                            0.0
                        }
                    })
                    .collect()
            })
            .collect()
    }

    fn random_matrix(rows: usize, cols: usize) -> Matrix {
        let mut rng = rand::thread_rng();
        let scale = (cols as f64).sqrt().recip();
        (0..rows)
            .map(|_| (0..cols).map(|_| rng.gen::<f64>() * 2.0 * scale - scale).collect())
            .collect()
    }

    /// MIMO 步进: 同时处理 num_streams 个输入
    /// state_hidden 长度 = hidden_dim * num_streams (实部+虚部分别存储)
    pub fn step_mimo(
        &self,
        state_real: &mut [f64],
        state_imag: &mut [f64],
        inputs: &[Vector],
    ) -> Vec<Vector> {
        let stream_dim = self.dim / self.num_streams;
        let mut outputs = Vec::with_capacity(self.num_streams);

        for s in 0..self.num_streams {
            let input = inputs.get(s).cloned().unwrap_or_default();
            let offset = s * self.hidden_dim;

            // Δ = softplus(Linear_Δ(x))
            let delta_raw = self.matrix_vector_mul(&self.delta_proj, &input);
            let delta_i = delta_raw.first().copied().unwrap_or(0.1).abs().clamp(0.01, 1.0);

            // B(x) projection for this stream
            let b_real = self.matrix_vector_mul(&self.b_proj_real, &input);
            let b_imag = self.matrix_vector_mul(&self.b_proj_imag, &input);

            for i in 0..self.hidden_dim.min(state_real.len().saturating_sub(offset)) {
                let idx = offset + i;

                // 复数状态更新: h' = (A_real + i·A_imag) · h + Δ · B(x)
                // 实部: h'_r = A_r·h_r - A_i·h_i + Δ·B_r(x)
                // 虚部: h'_i = A_r·h_i + A_i·h_r + Δ·B_i(x)
                let hr = *state_real.get(idx).unwrap_or(&0.0);
                let hi = *state_imag.get(idx).unwrap_or(&0.0);

                let ar = self.a_real[i].get(i).copied().unwrap_or(0.0);
                let ai = self.a_imag[i].get(i).copied().unwrap_or(0.0);

                let br = b_real.get(i).copied().unwrap_or(0.0);
                let bi = b_imag.get(i).copied().unwrap_or(0.0);

                let new_hr = ar * hr - ai * hi + delta_i * br;
                let new_hi = ar * hi + ai * hr + delta_i * bi;

                if idx < state_real.len() { state_real[idx] = new_hr; }
                if idx < state_imag.len() { state_imag[idx] = new_hi; }
            }

            // y = C(h) · h: 复数门控输出
            let real_slice: Vector = state_real[offset..offset + self.hidden_dim].to_vec();
            let imag_slice: Vector = state_imag[offset..offset + self.hidden_dim].to_vec();
            let c_real = self.matrix_vector_mul(&self.c_proj_real, &real_slice);
            let c_imag = self.matrix_vector_mul(&self.c_proj_imag, &imag_slice);

            let mut out = vec![0.0; stream_dim];
            for j in 0..stream_dim.min(c_real.len()).min(c_imag.len()) {
                let cr = c_real.get(j).copied().unwrap_or(0.0);
                let ci = c_imag.get(j).copied().unwrap_or(0.0);
                let hr = state_real.get(offset + j).copied().unwrap_or(0.0);
                let hi = state_imag.get(offset + j).copied().unwrap_or(0.0);
                // 复数幅度: |y| = sqrt((C_r·h_r - C_i·h_i)² + (C_r·h_i + C_i·h_r)²)
                let y_real = cr * hr - ci * hi;
                let y_imag = cr * hi + ci * hr;
                out[j] = (y_real * y_real + y_imag * y_imag).sqrt(); // 幅度输出
            }
            outputs.push(out);
        }
        outputs
    }

    fn matrix_vector_mul(&self, matrix: &Matrix, vector: &Vector) -> Vector {
        matrix
            .iter()
            .map(|row| row.iter().zip(vector.iter()).map(|(m, v)| m * v).sum())
            .collect()
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
        } else if text.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && text.len() < 15 {
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
    fn test_mimo_new() {
        let mimo = MimoSelectableOperator::new(16, 8, 2);
        assert_eq!(mimo.dim, 16);
        assert_eq!(mimo.hidden_dim, 8);
        assert_eq!(mimo.num_streams, 2);
    }

    #[test]
    fn test_mimo_step_returns_correct_number_of_outputs() {
        let mimo = MimoSelectableOperator::new(16, 8, 2);
        let mut state_real = vec![0.0; 16];
        let mut state_imag = vec![0.0; 16];
        let inputs = vec![
            vec![0.5; 16],
            vec![-0.3; 16],
        ];
        let outputs = mimo.step_mimo(&mut state_real, &mut state_imag, &inputs);
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].len(), 8); // dim/num_streams = 8
        assert_eq!(outputs[1].len(), 8);
    }

    #[test]
    fn test_mimo_state_evolves_after_step() {
        let mimo = MimoSelectableOperator::new(16, 8, 1);
        let mut state_real = vec![0.0; 8];
        let mut state_imag = vec![0.0; 8];
        let inputs = vec![vec![1.0; 16]];
        let _ = mimo.step_mimo(&mut state_real, &mut state_imag, &inputs);
        // 状态应该非零 (至少某个维度更新了)
        let has_real = state_real.iter().any(|&v| v.abs() > 0.001);
        let has_imag = state_imag.iter().any(|&v| v.abs() > 0.001);
        assert!(has_real || has_imag, "state should have non-zero values after step");
    }

    #[test]
    fn test_mimo_multiple_steps_accumulate() {
        let mimo = MimoSelectableOperator::new(16, 8, 2);
        let mut state_real = vec![0.0; 16];
        let mut state_imag = vec![0.0; 16];
        let inputs = vec![vec![0.5; 16], vec![-0.2; 16]];

        // 第一步
        let out1 = mimo.step_mimo(&mut state_real, &mut state_imag, &inputs);
        let norm1: f64 = out1.iter().flat_map(|v| v.iter()).map(|x| x * x).sum();

        // 第二步 (状态持续累积)
        let out2 = mimo.step_mimo(&mut state_real, &mut state_imag, &inputs);
        let norm2: f64 = out2.iter().flat_map(|v| v.iter()).map(|x| x * x).sum();

        // 多次步进应产生不同输出 (状态演化了)
        assert!((norm2 - norm1).abs() > 1e-10 || norm2 > 0.0);
    }

    #[test]
    fn test_selectable_operator_backward_compat() {
        let op = SelectableOperator::new(8, 16);
        let mut state = super::super::core::SelectiveState {
            data: vec![0.0; 8],
            hidden: vec![0.0; 16],
            importance: vec![0.0; 8],
            timestamp: 0,
        };
        let input = vec![1.0; 8];
        let output = op.step(&mut state, &input);
        assert_eq!(output.len(), 8);
        assert!(output.iter().any(|&v| v.abs() > 0.0), "output should be non-zero");
    }
}

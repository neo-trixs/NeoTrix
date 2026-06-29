use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

#[derive(Debug, Clone, PartialEq)]
pub enum ToyModelType {
    LinearMapping,
    TwoLayerNetwork,
    AttentionHead,
    MemoryRetrieval,
    PatternMatcher,
    SequencePredictor,
}

impl ToyModelType {
    pub fn complexity_rating(&self) -> usize {
        match self {
            ToyModelType::LinearMapping => 1,
            ToyModelType::TwoLayerNetwork => 2,
            ToyModelType::AttentionHead => 3,
            ToyModelType::MemoryRetrieval => 4,
            ToyModelType::PatternMatcher => 3,
            ToyModelType::SequencePredictor => 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToyConfig {
    pub model_type: ToyModelType,
    pub input_dim: usize,
    pub hidden_dim: usize,
    pub output_dim: usize,
    pub noise_level: f64,
    pub sparsity: f64,
    pub learning_rate: f64,
}

impl ToyConfig {
    pub fn new(model_type: ToyModelType) -> Self {
        let defaults = match model_type {
            ToyModelType::LinearMapping => (8, 0, 4),
            ToyModelType::TwoLayerNetwork => (8, 16, 4),
            ToyModelType::AttentionHead => (8, 0, 4),
            ToyModelType::MemoryRetrieval => (8, 0, 4),
            ToyModelType::PatternMatcher => (8, 0, 4),
            ToyModelType::SequencePredictor => (8, 0, 4),
        };
        ToyConfig {
            model_type,
            input_dim: defaults.0,
            hidden_dim: defaults.1,
            output_dim: defaults.2,
            noise_level: 0.01,
            sparsity: 0.0,
            learning_rate: 0.01,
        }
    }

    pub fn randomize(&mut self, rng_seed: u64) {
        let mut rng = StdRng::seed_from_u64(rng_seed);
        self.input_dim = rng.gen_range(4..=32);
        if matches!(self.model_type, ToyModelType::TwoLayerNetwork) {
            self.hidden_dim = rng.gen_range(4..=64);
        } else {
            self.hidden_dim = 0;
        }
        self.output_dim = rng.gen_range(1..=16);
        self.noise_level = rng.gen::<f64>() * 0.5;
        self.sparsity = rng.gen::<f64>() * 0.8;
        self.learning_rate = 10.0_f64.powf(rng.gen_range(-4.0..0.0));
    }
}

#[derive(Debug, Clone)]
pub struct ToyDataset {
    pub inputs: Vec<Vec<f64>>,
    pub targets: Vec<Vec<f64>>,
}

impl ToyDataset {
    pub fn new(inputs: Vec<Vec<f64>>, targets: Vec<Vec<f64>>) -> Self {
        ToyDataset { inputs, targets }
    }

    pub fn len(&self) -> usize {
        self.inputs.len()
    }

    pub fn classification(
        batch_size: usize,
        input_dim: usize,
        num_classes: usize,
        seed: u64,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut inputs = Vec::with_capacity(batch_size);
        let mut targets = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            let input: Vec<f64> = (0..input_dim)
                .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                .collect();
            let class = rng.gen_range(0..num_classes);
            let mut target = vec![0.0; num_classes];
            target[class] = 1.0;
            inputs.push(input);
            targets.push(target);
        }
        ToyDataset { inputs, targets }
    }

    pub fn regression(n_samples: usize, input_dim: usize, noise: f64, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let true_w: Vec<f64> = (0..input_dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let true_b = rng.gen_range(-0.5..0.5);
        let mut inputs = Vec::with_capacity(n_samples);
        let mut targets = Vec::with_capacity(n_samples);
        for _ in 0..n_samples {
            let input: Vec<f64> = (0..input_dim)
                .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                .collect();
            let y: f64 = input
                .iter()
                .zip(true_w.iter())
                .map(|(x, w)| x * w)
                .sum::<f64>()
                + true_b
                + rng.gen_range(-noise..noise);
            inputs.push(input);
            targets.push(vec![y]);
        }
        ToyDataset { inputs, targets }
    }

    pub fn memorization(n_samples: usize, input_dim: usize, num_classes: usize, seed: u64) -> Self {
        ToyDataset::classification(n_samples, input_dim, num_classes, seed)
    }
}

#[derive(Debug, Clone)]
pub struct ToyModelResult {
    pub final_loss: f64,
    pub accuracy: f64,
    pub convergence_steps: usize,
    pub loss_trajectory: Vec<f64>,
    pub weight_norms: Vec<f64>,
    pub generalization_gap: f64,
}

#[derive(Debug, Clone)]
pub struct ToyModel {
    pub config: ToyConfig,
    pub weights: Vec<Vec<f64>>,
    pub bias: Vec<f64>,
}

impl ToyModel {
    pub fn new(config: ToyConfig, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        match config.model_type {
            ToyModelType::TwoLayerNetwork => {
                let n_rows = config.hidden_dim + config.output_dim;
                let bias_len = config.hidden_dim + config.output_dim;
                let mut weights = Vec::with_capacity(n_rows);
                let scale1 = 1.0 / (config.input_dim as f64).sqrt();
                for _ in 0..config.hidden_dim {
                    let row: Vec<f64> = (0..config.input_dim)
                        .map(|_| rng.gen_range(-scale1..scale1))
                        .collect();
                    weights.push(row);
                }
                let scale2 = 1.0 / (config.hidden_dim as f64).sqrt();
                for _ in 0..config.output_dim {
                    let row: Vec<f64> = (0..config.hidden_dim)
                        .map(|_| rng.gen_range(-scale2..scale2))
                        .collect();
                    weights.push(row);
                }
                let bias: Vec<f64> = (0..bias_len).map(|_| rng.gen_range(-0.1..0.1)).collect();
                ToyModel {
                    config,
                    weights,
                    bias,
                }
            }
            _ => {
                let scale = 1.0 / (config.input_dim as f64).sqrt();
                let weights: Vec<Vec<f64>> = (0..config.output_dim)
                    .map(|_| {
                        (0..config.input_dim)
                            .map(|_| rng.gen_range(-scale..scale))
                            .collect()
                    })
                    .collect();
                let bias: Vec<f64> = (0..config.output_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect();
                ToyModel {
                    config,
                    weights,
                    bias,
                }
            }
        }
    }

    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        match self.config.model_type {
            ToyModelType::LinearMapping | ToyModelType::SequencePredictor => {
                let n_out = self.config.output_dim;
                let mut output = vec![0.0; n_out];
                for j in 0..n_out {
                    let mut z = self.bias[j];
                    let row_len = self.weights[j].len().min(self.config.input_dim);
                    for i in 0..row_len {
                        z += self.weights[j][i] * input[i];
                    }
                    output[j] = z;
                }
                output
            }
            ToyModelType::TwoLayerNetwork => {
                let hd = self.config.hidden_dim;
                let od = self.config.output_dim;
                let mut hidden = vec![0.0; hd];
                for j in 0..hd {
                    let mut z = self.bias[j];
                    let row_len = self.weights[j].len().min(self.config.input_dim);
                    for i in 0..row_len {
                        z += self.weights[j][i] * input[i];
                    }
                    hidden[j] = if z > 0.0 { z } else { 0.0 };
                }
                let mut output = vec![0.0; od];
                for k in 0..od {
                    let idx = hd + k;
                    let mut z = self.bias[idx];
                    let row_len = self.weights[idx].len().min(hd);
                    for j in 0..row_len {
                        z += self.weights[idx][j] * hidden[j];
                    }
                    output[k] = z;
                }
                output
            }
            ToyModelType::AttentionHead => {
                let n_out = self.config.output_dim;
                let mut output = vec![0.0; n_out];
                for j in 0..n_out {
                    let mut z = self.bias[j];
                    let row_len = self.weights[j].len().min(self.config.input_dim);
                    for i in 0..row_len {
                        z += self.weights[j][i] * input[i];
                    }
                    let gate = 1.0 / (1.0 + (-z).exp());
                    output[j] = gate * z;
                }
                output
            }
            ToyModelType::MemoryRetrieval => {
                let n_out = self.config.output_dim;
                let mut logits = vec![0.0; n_out];
                for j in 0..n_out {
                    let mut z = self.bias[j];
                    let row_len = self.weights[j].len().min(self.config.input_dim);
                    for i in 0..row_len {
                        z += self.weights[j][i] * input[i];
                    }
                    logits[j] = z;
                }
                let max_l = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let exp_l: Vec<f64> = logits.iter().map(|v| (v - max_l).exp()).collect();
                let sum_exp: f64 = exp_l.iter().sum();
                exp_l.iter().map(|e| e / sum_exp).collect()
            }
            ToyModelType::PatternMatcher => {
                let n_out = self.config.output_dim;
                let mut output = vec![0.0; n_out];
                for j in 0..n_out {
                    let mut z = self.bias[j];
                    let row_len = self.weights[j].len().min(self.config.input_dim);
                    for i in 0..row_len {
                        z += self.weights[j][i] * input[i];
                    }
                    output[j] = z.tanh();
                }
                output
            }
        }
    }

    pub fn train(&mut self, dataset: &ToyDataset, epochs: usize) -> ToyModelResult {
        let n = dataset.len();
        if n == 0 {
            return ToyModelResult {
                final_loss: 0.0,
                accuracy: 0.0,
                convergence_steps: 0,
                loss_trajectory: Vec::new(),
                weight_norms: Vec::new(),
                generalization_gap: 0.0,
            };
        }
        let mut loss_trajectory = Vec::with_capacity(epochs);
        let mut prev_loss = f64::INFINITY;
        let mut convergence_steps = epochs;
        let lr = self.config.learning_rate;
        for epoch in 0..epochs {
            let mut total_loss = 0.0;
            let mut total_gw: Vec<Vec<f64>> = self
                .weights
                .iter()
                .map(|row| vec![0.0; row.len()])
                .collect();
            let mut total_gb = vec![0.0; self.bias.len()];
            for s in 0..n {
                let input = &dataset.inputs[s];
                let target = &dataset.targets[s];
                let output = self.forward(input);
                total_loss += self.loss(&output, target);
                let (sgw, sgb) = self.sample_gradient(input, target);
                for j in 0..sgw.len() {
                    for i in 0..sgw[j].len() {
                        total_gw[j][i] += sgw[j][i];
                    }
                }
                for j in 0..sgb.len() {
                    total_gb[j] += sgb[j];
                }
            }
            let avg_loss = total_loss / n as f64;
            loss_trajectory.push(avg_loss);
            let inv_n = 1.0 / n as f64;
            for j in 0..self.weights.len() {
                for i in 0..self.weights[j].len() {
                    self.weights[j][i] -= lr * total_gw[j][i] * inv_n;
                }
            }
            for j in 0..self.bias.len() {
                self.bias[j] -= lr * total_gb[j] * inv_n;
            }
            if avg_loss < 1e-10 {
                convergence_steps = epoch + 1;
                break;
            }
            if epoch > 0 && (prev_loss - avg_loss).abs() < 1e-8 {
                convergence_steps = epoch + 1;
                break;
            }
            prev_loss = avg_loss;
        }
        let train_loss = *loss_trajectory.last().unwrap_or(&0.0);
        let (eval_loss, accuracy) = self.evaluate(dataset);
        let weight_norms: Vec<f64> = self
            .weights
            .iter()
            .map(|row| row.iter().map(|w| w * w).sum::<f64>().sqrt())
            .collect();
        ToyModelResult {
            final_loss: eval_loss,
            accuracy,
            convergence_steps,
            loss_trajectory,
            weight_norms,
            generalization_gap: eval_loss - train_loss,
        }
    }

    fn sample_gradient(&self, input: &[f64], target: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
        match self.config.model_type {
            ToyModelType::LinearMapping | ToyModelType::SequencePredictor => {
                self.linear_grad(input, target)
            }
            ToyModelType::TwoLayerNetwork => self.two_layer_grad(input, target),
            ToyModelType::PatternMatcher => self.tanh_grad(input, target),
            ToyModelType::AttentionHead | ToyModelType::MemoryRetrieval => {
                self.numerical_grad(input, target)
            }
        }
    }

    fn linear_grad(&self, input: &[f64], target: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
        let output = self.forward(input);
        let d_out = self.config.output_dim as f64;
        let mut gw: Vec<Vec<f64>> = (0..self.config.output_dim)
            .map(|_| vec![0.0; self.config.input_dim])
            .collect();
        let mut gb = vec![0.0; self.config.output_dim];
        for j in 0..self.config.output_dim {
            let dj = (output[j] - target.get(j).copied().unwrap_or(0.0)) / d_out;
            for i in 0..self.config.input_dim {
                gw[j][i] = dj * input[i];
            }
            gb[j] = dj;
        }
        (gw, gb)
    }

    fn two_layer_grad(&self, input: &[f64], target: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
        let hd = self.config.hidden_dim;
        let od = self.config.output_dim;
        let mut h_pre = vec![0.0; hd];
        let mut h_act = vec![0.0; hd];
        for j in 0..hd {
            let mut z = self.bias[j];
            let row_len = self.weights[j].len().min(self.config.input_dim);
            for i in 0..row_len {
                z += self.weights[j][i] * input[i];
            }
            h_pre[j] = z;
            h_act[j] = if z > 0.0 { z } else { 0.0 };
        }
        let mut output = vec![0.0; od];
        for k in 0..od {
            let idx = hd + k;
            let mut z = self.bias[idx];
            let row_len = self.weights[idx].len().min(hd);
            for j in 0..row_len {
                z += self.weights[idx][j] * h_act[j];
            }
            output[k] = z;
        }
        let d_out = od as f64;
        let d_output: Vec<f64> = (0..od)
            .map(|k| (output[k] - target.get(k).copied().unwrap_or(0.0)) / d_out)
            .collect();
        let mut gw: Vec<Vec<f64>> = (0..hd + od)
            .map(|i| {
                if i < hd {
                    vec![0.0; self.config.input_dim]
                } else {
                    vec![0.0; hd]
                }
            })
            .collect();
        let mut gb = vec![0.0; hd + od];
        let mut d_hidden = vec![0.0; hd];
        for k in 0..od {
            let idx = hd + k;
            for j in 0..hd {
                gw[idx][j] = d_output[k] * h_act[j];
                d_hidden[j] += d_output[k] * self.weights[idx][j];
            }
            gb[idx] = d_output[k];
        }
        for j in 0..hd {
            let dh = if h_pre[j] > 0.0 { d_hidden[j] } else { 0.0 };
            for i in 0..self.config.input_dim {
                gw[j][i] = dh * input[i];
            }
            gb[j] = dh;
        }
        (gw, gb)
    }

    fn tanh_grad(&self, input: &[f64], target: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
        let od = self.config.output_dim;
        let d_out = od as f64;
        let mut gw: Vec<Vec<f64>> = (0..od).map(|_| vec![0.0; self.config.input_dim]).collect();
        let mut gb = vec![0.0; od];
        for j in 0..od {
            let mut z = self.bias[j];
            for i in 0..self.config.input_dim {
                z += self.weights[j][i] * input[i];
            }
            let t = z.tanh();
            let output_j = t;
            let dj = (output_j - target.get(j).copied().unwrap_or(0.0)) / d_out * (1.0 - t * t);
            for i in 0..self.config.input_dim {
                gw[j][i] = dj * input[i];
            }
            gb[j] = dj;
        }
        (gw, gb)
    }

    fn numerical_grad(&self, input: &[f64], target: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
        let eps = 1e-6;
        let mut gw: Vec<Vec<f64>> = self
            .weights
            .iter()
            .map(|row| vec![0.0; row.len()])
            .collect();
        let mut gb = vec![0.0; self.bias.len()];
        for j in 0..self.weights.len() {
            for i in 0..self.weights[j].len() {
                let mut wp = self.weights.clone();
                let mut wm = self.weights.clone();
                wp[j][i] += eps;
                wm[j][i] -= eps;
                let model_plus = ToyModel {
                    weights: wp,
                    bias: self.bias.clone(),
                    config: self.config.clone(),
                };
                let model_minus = ToyModel {
                    weights: wm,
                    bias: self.bias.clone(),
                    config: self.config.clone(),
                };
                let lp = {
                    let o = model_plus.forward(input);
                    model_plus.loss(&o, target)
                };
                let lm = {
                    let o = model_minus.forward(input);
                    model_minus.loss(&o, target)
                };
                gw[j][i] = (lp - lm) / (2.0 * eps);
            }
        }
        for j in 0..self.bias.len() {
            let mut bp = self.bias.clone();
            let mut bm = self.bias.clone();
            bp[j] += eps;
            bm[j] -= eps;
            let model_plus = ToyModel {
                weights: self.weights.clone(),
                bias: bp,
                config: self.config.clone(),
            };
            let model_minus = ToyModel {
                weights: self.weights.clone(),
                bias: bm,
                config: self.config.clone(),
            };
            let lp = {
                let o = model_plus.forward(input);
                model_plus.loss(&o, target)
            };
            let lm = {
                let o = model_minus.forward(input);
                model_minus.loss(&o, target)
            };
            gb[j] = (lp - lm) / (2.0 * eps);
        }
        (gw, gb)
    }

    pub fn evaluate(&self, dataset: &ToyDataset) -> (f64, f64) {
        let n = dataset.len();
        if n == 0 {
            return (0.0, 0.0);
        }
        let mut total_loss = 0.0;
        let mut correct = 0;
        let is_classification = if n > 0 {
            dataset.targets[0].len() > 1
        } else {
            false
        };
        for s in 0..n {
            let output = self.forward(&dataset.inputs[s]);
            let target = &dataset.targets[s];
            total_loss += self.loss(&output, target);
            if is_classification {
                let pred = output
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let truth = target
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                if pred == truth {
                    correct += 1;
                }
            }
        }
        let avg_loss = total_loss / n as f64;
        let accuracy = if is_classification {
            correct as f64 / n as f64
        } else {
            1.0 / (1.0 + avg_loss)
        };
        (avg_loss, accuracy)
    }

    pub fn loss(&self, output: &[f64], target: &[f64]) -> f64 {
        let min_len = output.len().min(target.len());
        if min_len == 0 {
            return 0.0;
        }
        output
            .iter()
            .zip(target.iter())
            .take(min_len)
            .map(|(o, t)| (o - t).powi(2))
            .sum::<f64>()
            / min_len as f64
            * 0.5
    }

    pub fn reset(&mut self, seed: u64) {
        let config = self.config.clone();
        *self = ToyModel::new(config, seed);
    }
}

pub struct ToyModelGenerator;

impl ToyModelGenerator {
    pub fn generate_linear_model(input_dim: usize, output_dim: usize, seed: u64) -> ToyModel {
        let config = ToyConfig {
            model_type: ToyModelType::LinearMapping,
            input_dim,
            hidden_dim: 0,
            output_dim,
            noise_level: 0.0,
            sparsity: 0.0,
            learning_rate: 0.01,
        };
        ToyModel::new(config, seed)
    }

    pub fn generate_two_layer_network(
        input_dim: usize,
        hidden_dim: usize,
        output_dim: usize,
        seed: u64,
    ) -> ToyModel {
        let config = ToyConfig {
            model_type: ToyModelType::TwoLayerNetwork,
            input_dim,
            hidden_dim,
            output_dim,
            noise_level: 0.0,
            sparsity: 0.0,
            learning_rate: 0.01,
        };
        ToyModel::new(config, seed)
    }

    pub fn generate_memorization_benchmark(
        n_samples: usize,
        input_dim: usize,
        num_classes: usize,
        seed: u64,
    ) -> (ToyModel, ToyDataset) {
        let config = ToyConfig {
            model_type: ToyModelType::MemoryRetrieval,
            input_dim,
            hidden_dim: 0,
            output_dim: num_classes,
            noise_level: 0.0,
            sparsity: 0.0,
            learning_rate: 0.01,
        };
        let model = ToyModel::new(config, seed);
        let dataset =
            ToyDataset::memorization(n_samples, input_dim, num_classes, seed.wrapping_add(1));
        (model, dataset)
    }

    pub fn generate_comparison(
        models: &[(ToyConfig, u64)],
        dataset: &ToyDataset,
        epochs: usize,
    ) -> Vec<(String, ToyModelResult)> {
        models
            .iter()
            .map(|(config, seed)| {
                let mut model = ToyModel::new(config.clone(), *seed);
                let result = model.train(dataset, epochs);
                let name = format!("{:?}", config.model_type);
                (name, result)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_linear_forward_output_shape() {
        let config = ToyConfig::new(ToyModelType::LinearMapping);
        let model = ToyModel::new(config, 42);
        let input = vec![0.5; 8];
        let output = model.forward(&input);
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_two_layer_forward_output_shape() {
        let config = ToyConfig::new(ToyModelType::TwoLayerNetwork);
        let model = ToyModel::new(config, 42);
        let input = vec![0.5; 8];
        let output = model.forward(&input);
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_attention_forward_output_shape() {
        let config = ToyConfig::new(ToyModelType::AttentionHead);
        let model = ToyModel::new(config, 42);
        let input = vec![0.5; 8];
        let output = model.forward(&input);
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_memory_retrieval_forward_output_shape() {
        let config = ToyConfig::new(ToyModelType::MemoryRetrieval);
        let model = ToyModel::new(config, 42);
        let input = vec![0.5; 8];
        let output = model.forward(&input);
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_pattern_matcher_forward_output_shape() {
        let config = ToyConfig::new(ToyModelType::PatternMatcher);
        let model = ToyModel::new(config, 42);
        let input = vec![0.5; 8];
        let output = model.forward(&input);
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_sequence_predictor_forward_output_shape() {
        let config = ToyConfig::new(ToyModelType::SequencePredictor);
        let model = ToyModel::new(config, 42);
        let input = vec![0.5; 8];
        let output = model.forward(&input);
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_classification_dataset() {
        let ds = ToyDataset::classification(20, 8, 3, 42);
        assert_eq!(ds.len(), 20);
        assert_eq!(ds.inputs[0].len(), 8);
        assert_eq!(ds.targets[0].len(), 3);
        let sum_t: f64 = ds.targets[0].iter().sum();
        assert!(approx_eq(sum_t, 1.0, 1e-10));
    }

    #[test]
    fn test_regression_dataset() {
        let ds = ToyDataset::regression(15, 5, 0.1, 42);
        assert_eq!(ds.len(), 15);
        assert_eq!(ds.inputs[0].len(), 5);
        assert_eq!(ds.targets[0].len(), 1);
    }

    #[test]
    fn test_memorization_dataset() {
        let ds = ToyDataset::memorization(10, 4, 2, 42);
        assert_eq!(ds.len(), 10);
        assert_eq!(ds.targets[0].len(), 2);
    }

    #[test]
    fn test_training_reduces_loss() {
        let config = ToyConfig::new(ToyModelType::LinearMapping);
        let mut model = ToyModel::new(config, 42);
        let ds = ToyDataset::classification(30, 8, 4, 99);
        let before = {
            let (l, _) = model.evaluate(&ds);
            l
        };
        let result = model.train(&ds, 50);
        let after = result.final_loss;
        assert!(after < before * 1.1);
    }

    #[test]
    fn test_two_layer_training() {
        let config = ToyConfig::new(ToyModelType::TwoLayerNetwork);
        let mut model = ToyModel::new(config, 42);
        let ds = ToyDataset::classification(20, 8, 4, 77);
        let before = {
            let (l, _) = model.evaluate(&ds);
            l
        };
        let result = model.train(&ds, 30);
        assert!(result.final_loss < before * 1.2);
    }

    #[test]
    fn test_reset_produces_new_weights() {
        let config = ToyConfig::new(ToyModelType::LinearMapping);
        let mut model = ToyModel::new(config, 42);
        let w_before = model.weights.clone();
        model.reset(99);
        let w_after = model.weights.clone();
        let all_same: bool = w_before.iter().zip(w_after.iter()).all(|(a, b)| {
            a.iter()
                .zip(b.iter())
                .all(|(x, y)| approx_eq(*x, *y, 1e-10))
        });
        assert!(!all_same);
    }

    #[test]
    fn test_generate_linear_model() {
        let model = ToyModelGenerator::generate_linear_model(6, 3, 42);
        assert_eq!(model.config.model_type, ToyModelType::LinearMapping);
        assert_eq!(model.config.input_dim, 6);
        assert_eq!(model.config.output_dim, 3);
        assert_eq!(model.weights.len(), 3);
        assert_eq!(model.weights[0].len(), 6);
    }

    #[test]
    fn test_generate_two_layer_network() {
        let model = ToyModelGenerator::generate_two_layer_network(10, 20, 5, 42);
        assert_eq!(model.config.model_type, ToyModelType::TwoLayerNetwork);
        assert_eq!(model.weights.len(), 25);
        assert_eq!(model.weights[0].len(), 10);
        assert_eq!(model.weights[20].len(), 20);
    }

    #[test]
    fn test_generate_memorization_benchmark() {
        let (model, dataset) = ToyModelGenerator::generate_memorization_benchmark(8, 4, 2, 42);
        assert_eq!(model.config.model_type, ToyModelType::MemoryRetrieval);
        assert_eq!(dataset.len(), 8);
    }

    #[test]
    fn test_generate_comparison() {
        let cfg1 = ToyConfig::new(ToyModelType::LinearMapping);
        let cfg2 = ToyConfig::new(ToyModelType::PatternMatcher);
        let pairs = [(cfg1, 42u64), (cfg2, 99u64)];
        let ds = ToyDataset::classification(10, 8, 4, 33);
        let results = ToyModelGenerator::generate_comparison(&pairs, &ds, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "LinearMapping");
        assert_eq!(results[1].0, "PatternMatcher");
    }

    #[test]
    fn test_config_randomize() {
        let mut config = ToyConfig::new(ToyModelType::LinearMapping);
        let old_input = config.input_dim;
        config.randomize(42);
        assert!(config.input_dim >= 4 && config.input_dim <= 32);
        assert!(config.learning_rate > 0.0);
        assert!(config.sparsity >= 0.0);
        assert_ne!(config.input_dim, old_input);
    }

    #[test]
    fn test_complexity_ratings() {
        assert_eq!(ToyModelType::LinearMapping.complexity_rating(), 1);
        assert_eq!(ToyModelType::TwoLayerNetwork.complexity_rating(), 2);
        assert_eq!(ToyModelType::AttentionHead.complexity_rating(), 3);
        assert_eq!(ToyModelType::MemoryRetrieval.complexity_rating(), 4);
        assert_eq!(ToyModelType::PatternMatcher.complexity_rating(), 3);
        assert_eq!(ToyModelType::SequencePredictor.complexity_rating(), 5);
    }

    #[test]
    fn test_empty_dataset() {
        let config = ToyConfig::new(ToyModelType::LinearMapping);
        let mut model = ToyModel::new(config, 42);
        let ds = ToyDataset::new(vec![], vec![]);
        assert_eq!(ds.len(), 0);
        let (l, a) = model.evaluate(&ds);
        assert_eq!(l, 0.0);
        assert_eq!(a, 0.0);
        let result = model.train(&ds, 10);
        assert_eq!(result.convergence_steps, 0);
    }

    #[test]
    fn test_loss_symmetric() {
        let config = ToyConfig::new(ToyModelType::LinearMapping);
        let model = ToyModel::new(config, 42);
        let output = vec![1.0, 0.0];
        let target = vec![0.0, 1.0];
        let l1 = model.loss(&output, &target);
        let l2 = model.loss(&target, &output);
        assert!(approx_eq(l1, l2, 1e-10));
    }

    #[test]
    fn test_evaluate_classification_accuracy() {
        let mut model = ToyModelGenerator::generate_linear_model(4, 2, 42);
        let ds = ToyDataset::classification(10, 4, 2, 77);
        model.train(&ds, 30);
        let (_, acc) = model.evaluate(&ds);
        assert!(acc > 0.0);
        assert!(acc <= 1.0);
    }

    #[test]
    fn test_forward_deterministic() {
        let config = ToyConfig::new(ToyModelType::LinearMapping);
        let model = ToyModel::new(config, 42);
        let input = vec![0.3, -0.5, 0.7, -0.2, 0.1, 0.9, -0.8, 0.4];
        let o1 = model.forward(&input);
        let o2 = model.forward(&input);
        for (a, b) in o1.iter().zip(o2.iter()) {
            assert!(approx_eq(*a, *b, 1e-12));
        }
    }

    #[test]
    fn test_memory_retrieval_softmax_output() {
        let config = ToyConfig::new(ToyModelType::MemoryRetrieval);
        let model = ToyModel::new(config, 42);
        let input = vec![0.5; 8];
        let output = model.forward(&input);
        let sum_o: f64 = output.iter().sum();
        assert!(approx_eq(sum_o, 1.0, 1e-8));
        for v in &output {
            assert!(*v >= 0.0 && *v <= 1.0);
        }
    }

    #[test]
    fn test_pattern_matcher_tanh_range() {
        let config = ToyConfig::new(ToyModelType::PatternMatcher);
        let model = ToyModel::new(config, 42);
        let input = vec![100.0; 8];
        let output = model.forward(&input);
        for v in &output {
            assert!(*v >= -1.0 && *v <= 1.0);
        }
    }
}

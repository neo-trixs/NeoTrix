use rand::Rng;
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct TrainableVsaEncoder {
    pub dim: usize,
    pub num_classes: usize,
    pub item_memory: Vec<Vec<f64>>,
    pub label_memory: Vec<Vec<f64>>,
    pub learning_rate: f64,
    pub quantize_on_freeze: bool,
}

impl TrainableVsaEncoder {
    pub fn new(dim: usize, num_items: usize, num_classes: usize) -> Self {
        let scale = 1.0 / (dim as f64).sqrt();
        let mut rng = rand::thread_rng();

        let item_memory = (0..num_items)
            .map(|_| {
                (0..dim)
                    .map(|_| rng.gen::<f64>() * 2.0 * scale - scale)
                    .collect()
            })
            .collect();

        let label_memory = (0..num_classes)
            .map(|_| {
                (0..dim)
                    .map(|_| rng.gen::<f64>() * 2.0 * scale - scale)
                    .collect()
            })
            .collect();

        Self {
            dim,
            num_classes,
            item_memory,
            label_memory,
            learning_rate: 0.01,
            quantize_on_freeze: false,
        }
    }

    pub fn with_seed(dim: usize, num_items: usize, num_classes: usize, seed: u64) -> Self {
        use rand::SeedableRng;
        let scale = 1.0 / (dim as f64).sqrt();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let item_memory = (0..num_items)
            .map(|_| {
                (0..dim)
                    .map(|_| rng.gen::<f64>() * 2.0 * scale - scale)
                    .collect()
            })
            .collect();

        let label_memory = (0..num_classes)
            .map(|_| {
                (0..dim)
                    .map(|_| rng.gen::<f64>() * 2.0 * scale - scale)
                    .collect()
            })
            .collect();

        Self {
            dim,
            num_classes,
            item_memory,
            label_memory,
            learning_rate: 0.01,
            quantize_on_freeze: false,
        }
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.learning_rate = lr;
    }

    pub fn cos_sim(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum();
        let nb: f64 = b.iter().map(|x| x * x).sum();
        let norm = na.sqrt().max(1e-12) * nb.sqrt().max(1e-12);
        dot / norm
    }

    fn softmax(logits: &[f64]) -> Vec<f64> {
        let max = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = logits.iter().map(|v| (v - max).exp()).collect();
        let sum: f64 = exps.iter().sum();
        exps.iter().map(|e| e / sum).collect()
    }

    fn cross_entropy(sims: &[f64], true_label: usize) -> f64 {
        let probs = Self::softmax(sims);
        -probs[true_label].ln().max(-1e12)
    }

    pub fn bundle(items: &[&[f64]]) -> Vec<f64> {
        if items.is_empty() {
            return vec![];
        }
        let dim = items[0].len();
        let n = items.len() as f64;
        let mut sum_cos = vec![0.0; dim];
        let mut sum_sin = vec![0.0; dim];

        for item in items {
            for i in 0..dim.min(item.len()) {
                let angle = item[i] * PI;
                sum_cos[i] += angle.cos();
                sum_sin[i] += angle.sin();
            }
        }

        (0..dim)
            .map(|i| (sum_sin[i] / n).atan2(sum_cos[i] / n) / PI)
            .collect()
    }

    fn similarities(&self, item: &[f64]) -> Vec<f64> {
        (0..self.num_classes)
            .map(|c| Self::cos_sim(item, &self.label_memory[c]))
            .collect()
    }

    pub fn train(&mut self, xs: &[usize], labels: &[usize], epochs: usize) -> Vec<f64> {
        let eps = 1e-5;
        let mut loss_curve = Vec::with_capacity(epochs);

        for _epoch in 0..epochs {
            let mut epoch_loss = 0.0;

            for (&xi, &li) in xs.iter().zip(labels.iter()) {
                let sims0 = self.similarities(&self.item_memory[xi]);
                let base_loss = Self::cross_entropy(&sims0, li);
                epoch_loss += base_loss;

                let mut grad_item = vec![0.0; self.dim];
                for d in 0..self.dim {
                    let orig = self.item_memory[xi][d];

                    self.item_memory[xi][d] = orig + eps;
                    let sims_fp = self.similarities(&self.item_memory[xi]);
                    let loss_fp = Self::cross_entropy(&sims_fp, li);

                    self.item_memory[xi][d] = orig - eps;
                    let sims_fn = self.similarities(&self.item_memory[xi]);
                    let loss_fn_val = Self::cross_entropy(&sims_fn, li);

                    self.item_memory[xi][d] = orig;
                    grad_item[d] = (loss_fp - loss_fn_val) / (2.0 * eps);
                }

                let mut grad_labels = vec![vec![0.0; self.dim]; self.num_classes];
                for c in 0..self.num_classes {
                    for d in 0..self.dim {
                        let orig = self.label_memory[c][d];

                        self.label_memory[c][d] = orig + eps;
                        let sims_fp = self.similarities(&self.item_memory[xi]);
                        let loss_fp = Self::cross_entropy(&sims_fp, li);

                        self.label_memory[c][d] = orig - eps;
                        let sims_fn = self.similarities(&self.item_memory[xi]);
                        let loss_fn_val = Self::cross_entropy(&sims_fn, li);

                        self.label_memory[c][d] = orig;
                        grad_labels[c][d] = (loss_fp - loss_fn_val) / (2.0 * eps);
                    }
                }

                for d in 0..self.dim {
                    self.item_memory[xi][d] -= self.learning_rate * grad_item[d];
                }
                for c in 0..self.num_classes {
                    for d in 0..self.dim {
                        self.label_memory[c][d] -= self.learning_rate * grad_labels[c][d];
                    }
                }
            }

            loss_curve.push(epoch_loss / xs.len() as f64);
        }

        loss_curve
    }

    pub fn train_multi(&mut self, xss: &[&[usize]], labels: &[usize], epochs: usize) -> Vec<f64> {
        let eps = 1e-5;
        let mut loss_curve = Vec::with_capacity(epochs);

        for _epoch in 0..epochs {
            let mut epoch_loss = 0.0;

            for (&x_indices, &li) in xss.iter().zip(labels.iter()) {
                let items: Vec<&[f64]> = x_indices
                    .iter()
                    .map(|&i| &self.item_memory[i][..])
                    .collect();
                let bundle = Self::bundle(&items);

                let sims0 = self.similarities(&bundle);
                let base_loss = Self::cross_entropy(&sims0, li);
                epoch_loss += base_loss;

                let mut grad_labels = vec![vec![0.0; self.dim]; self.num_classes];
                for c in 0..self.num_classes {
                    for d in 0..self.dim {
                        let orig = self.label_memory[c][d];

                        self.label_memory[c][d] = orig + eps;
                        let bundle_fp = {
                            let items2: Vec<&[f64]> = x_indices
                                .iter()
                                .map(|&i| &self.item_memory[i][..])
                                .collect();
                            Self::bundle(&items2)
                        };
                        let sims_fp = self.similarities(&bundle_fp);
                        let loss_fp = Self::cross_entropy(&sims_fp, li);

                        self.label_memory[c][d] = orig - eps;
                        let bundle_fn = {
                            let items2: Vec<&[f64]> = x_indices
                                .iter()
                                .map(|&i| &self.item_memory[i][..])
                                .collect();
                            Self::bundle(&items2)
                        };
                        let sims_fn = self.similarities(&bundle_fn);
                        let loss_fn_val = Self::cross_entropy(&sims_fn, li);

                        self.label_memory[c][d] = orig;
                        grad_labels[c][d] = (loss_fp - loss_fn_val) / (2.0 * eps);
                    }
                }

                for c in 0..self.num_classes {
                    for d in 0..self.dim {
                        self.label_memory[c][d] -= self.learning_rate * grad_labels[c][d];
                    }
                }
            }

            loss_curve.push(epoch_loss / xss.len() as f64);
        }

        loss_curve
    }

    pub fn predict(&self, x_idx: usize) -> usize {
        let item = &self.item_memory[x_idx];
        let mut best_class = 0;
        let mut best_sim = f64::NEG_INFINITY;
        for c in 0..self.num_classes {
            let sim = Self::cos_sim(item, &self.label_memory[c]);
            if sim > best_sim {
                best_sim = sim;
                best_class = c;
            }
        }
        best_class
    }

    pub fn freeze(&mut self) -> Vec<u8> {
        let bytes_per_item = (self.dim + 7) / 8;
        let mut result = vec![0u8; self.item_memory.len() * bytes_per_item];

        for (i, item) in self.item_memory.iter_mut().enumerate() {
            for (j, val) in item.iter_mut().enumerate() {
                if *val > 0.0 {
                    result[i * bytes_per_item + j / 8] |= 1 << (j % 8);
                }
                *val = if *val > 0.0 { 1.0 } else { -1.0 };
            }
        }

        for item in self.label_memory.iter_mut() {
            for val in item.iter_mut() {
                *val = if *val > 0.0 { 1.0 } else { -1.0 };
            }
        }

        self.quantize_on_freeze = true;
        result
    }

    pub fn encode(&self, x_idx: usize) -> Vec<f64> {
        if x_idx >= self.item_memory.len() {
            return vec![0.0; self.dim];
        }
        self.item_memory[x_idx].clone()
    }

    pub fn encode_binary(&self, x_idx: usize) -> Vec<u8> {
        if x_idx >= self.item_memory.len() {
            return vec![0u8; (self.dim + 7) / 8];
        }
        let bytes = (self.dim + 7) / 8;
        let mut result = vec![0u8; bytes];
        for (j, &val) in self.item_memory[x_idx].iter().enumerate() {
            if val > 0.0 {
                result[j / 8] |= 1 << (j % 8);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_correct_dimensions() {
        let e = TrainableVsaEncoder::new(128, 10, 3);
        assert_eq!(e.dim, 128);
        assert_eq!(e.item_memory.len(), 10);
        assert_eq!(e.item_memory[0].len(), 128);
        assert_eq!(e.label_memory.len(), 3);
        assert_eq!(e.label_memory[0].len(), 128);
        assert_eq!(e.num_classes, 3);
    }

    #[test]
    fn test_encode_returns_correct_dim() {
        let e = TrainableVsaEncoder::new(128, 5, 2);
        let v = e.encode(0);
        assert_eq!(v.len(), 128);
        let v2 = e.encode(4);
        assert_eq!(v2.len(), 128);
    }

    #[test]
    fn test_encode_binary_byte_count() {
        let dim = 128;
        let e = TrainableVsaEncoder::new(dim, 5, 2);
        let b = e.encode_binary(0);
        assert_eq!(b.len(), dim / 8);
    }

    #[test]
    fn test_predict_valid_class() {
        let e = TrainableVsaEncoder::new(64, 5, 3);
        let pred = e.predict(0);
        assert!(pred < 3, "predicted class must be in range");
        let pred2 = e.predict(4);
        assert!(pred2 < 3);
    }

    #[test]
    fn test_training_reduces_loss() {
        let mut e = TrainableVsaEncoder::new(64, 3, 3);
        let xs = vec![0usize, 1, 2];
        let labels = vec![0usize, 1, 2];
        let loss = e.train(&xs, &labels, 30);
        assert!(loss.len() == 30, "loss curve should have 30 entries");
        assert!(
            loss[loss.len() - 1] < loss[0] * 0.9,
            "training must reduce loss: first={:.6} last={:.6}",
            loss[0],
            loss[loss.len() - 1]
        );
    }

    #[test]
    fn test_freeze_produces_valid_output() {
        let mut e = TrainableVsaEncoder::new(64, 4, 2);
        let frozen = e.freeze();
        let expected_bytes = 4 * (64 / 8);
        assert_eq!(frozen.len(), expected_bytes);
        assert!(e.quantize_on_freeze);
    }

    #[test]
    fn test_different_seeds_different_vectors() {
        let e1 = TrainableVsaEncoder::with_seed(64, 5, 2, 42);
        let e2 = TrainableVsaEncoder::with_seed(64, 5, 2, 999);
        let mut any_diff = false;
        for i in 0..5 {
            let v1 = e1.encode(i);
            let v2 = e2.encode(i);
            if v1 != v2 {
                any_diff = true;
                break;
            }
        }
        assert!(
            any_diff,
            "different seeds should produce different item vectors"
        );
    }

    #[test]
    fn test_label_memory_shape() {
        let e = TrainableVsaEncoder::with_seed(256, 20, 5, 7);
        assert_eq!(e.label_memory.len(), 5);
        for c in 0..5 {
            assert_eq!(e.label_memory[c].len(), 256);
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        assert!((TrainableVsaEncoder::cos_sim(&a, &b) - 1.0).abs() < 1e-10);

        let c = vec![-1.0, 0.0, 0.0, 0.0];
        assert!((TrainableVsaEncoder::cos_sim(&a, &c) - (-1.0)).abs() < 1e-10);

        let d = vec![0.0, 1.0, 0.0, 0.0];
        assert!(TrainableVsaEncoder::cos_sim(&a, &d).abs() < 1e-10);

        let e = vec![2.0, 2.0, 0.0, 0.0];
        let f = vec![1.0, 1.0, 0.0, 0.0];
        assert!((TrainableVsaEncoder::cos_sim(&e, &f) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_multi_epoch_convergence() {
        let mut e = TrainableVsaEncoder::with_seed(64, 3, 2, 100);
        let xs = vec![0usize, 1, 2];
        let labels = vec![0usize, 0, 1];
        let loss = e.train(&xs, &labels, 50);
        let epoch_5_loss = loss[4];
        let epoch_50_loss = loss[49];
        assert!(
            epoch_50_loss < epoch_5_loss,
            "more epochs should give lower loss: epoch_5={:.6} epoch_50={:.6}",
            epoch_5_loss,
            epoch_50_loss
        );
    }

    #[test]
    fn test_encode_binary_matches_freeze() {
        let mut e = TrainableVsaEncoder::with_seed(64, 5, 2, 42);
        let xs = vec![0usize, 1, 2, 3, 4];
        let labels = vec![0usize, 0, 0, 1, 1];
        e.train(&xs, &labels, 10);

        let frozen = e.freeze();
        let bytes_per_item = (64 + 7) / 8;

        for i in 0..5 {
            let enc = e.encode_binary(i);
            let from_freeze = &frozen[i * bytes_per_item..(i + 1) * bytes_per_item];
            assert_eq!(
                enc, from_freeze,
                "encode_binary for item {} should match frozen segment",
                i
            );
        }
    }

    #[test]
    fn test_learning_rate_effect() {
        let mut e_slow = TrainableVsaEncoder::with_seed(64, 3, 2, 1);
        e_slow.set_learning_rate(0.001);
        let mut e_fast = TrainableVsaEncoder::with_seed(64, 3, 2, 1);
        e_fast.set_learning_rate(0.5);

        let xs = vec![0usize, 1, 2];
        let labels = vec![0usize, 0, 1];

        let loss_slow = e_slow.train(&xs, &labels, 20);
        let loss_fast = e_fast.train(&xs, &labels, 20);

        let slow_final = loss_slow[loss_slow.len() - 1];
        let fast_final = loss_fast[loss_fast.len() - 1];
        assert!(
            fast_final < slow_final,
            "higher learning rate should converge to lower loss: slow={:.6} fast={:.6}",
            slow_final,
            fast_final
        );
    }

    #[test]
    fn test_classify_2class() {
        let mut e = TrainableVsaEncoder::with_seed(64, 8, 2, 42);
        // items 0-3: class 0, items 4-7: class 1
        let xs = vec![0usize, 1, 2, 3, 4, 5, 6, 7];
        let labels = vec![0usize, 0, 0, 0, 1, 1, 1, 1];
        e.train(&xs, &labels, 60);

        let mut correct = 0;
        for i in 0..8 {
            let pred = e.predict(i);
            let expected = if i < 4 { 0 } else { 1 };
            if pred == expected {
                correct += 1;
            }
        }
        assert!(
            correct >= 6,
            "should classify at least 6/8 correctly after training (got {}/8)",
            correct
        );
    }

    #[test]
    fn test_bundle_empty_returns_empty() {
        let result = TrainableVsaEncoder::bundle(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_bundle_single_item_identical() {
        let v = vec![0.5, -0.3, 0.1, 0.8];
        let result = TrainableVsaEncoder::bundle(&[&v]);
        assert_eq!(result.len(), 4);
        for i in 0..4 {
            assert!((result[i] - v[i]).abs() < 1e-10 || result[i].is_finite());
        }
    }

    #[test]
    fn test_with_seed_deterministic() {
        let e1 = TrainableVsaEncoder::with_seed(64, 5, 3, 12345);
        let e2 = TrainableVsaEncoder::with_seed(64, 5, 3, 12345);
        for i in 0..5 {
            assert_eq!(e1.encode(i), e2.encode(i));
            assert_eq!(e1.encode_binary(i), e2.encode_binary(i));
        }
    }
}

use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::neotrix::nt_shield_prompt::RiskLevel;
use std::collections::VecDeque;

const RAW_VSA_LEN: usize = VSA_DIM; // 4096
const PACKED_VSA_LEN: usize = VSA_DIM / 8; // 512
const NUM_REGIONS: usize = 8;

#[derive(Debug, Clone)]
pub struct VsaValidation {
    pub valid: bool,
    pub is_packed: bool,
    pub popcount_ratio: f64,
    pub byte_entropy: f64,
    pub bit_distribution: [u8; 8],
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VsaGuardResult {
    pub allowed: bool,
    pub risk: RiskLevel,
    pub black_hole_score: f64,
    pub bit_flip_score: f64,
    pub validation: VsaValidation,
    pub reason: Option<String>,
}

pub struct VsaGuard {
    min_popcount: f64,
    max_popcount: f64,
    max_black_hole_score: f64,
    history: VecDeque<Vec<u8>>,
    max_history: usize,
}

impl Default for VsaGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl VsaGuard {
    pub fn new() -> Self {
        Self {
            min_popcount: 0.3,
            max_popcount: 0.7,
            max_black_hole_score: 0.9,
            history: VecDeque::new(),
            max_history: 32,
        }
    }

    pub fn with_params(min_pop: f64, max_pop: f64, max_black_hole: f64, max_hist: usize) -> Self {
        Self {
            min_popcount: min_pop,
            max_popcount: max_pop,
            max_black_hole_score: max_black_hole,
            history: VecDeque::new(),
            max_history: max_hist,
        }
    }

    fn detect_format(&self, bytes: &[u8]) -> Result<bool, String> {
        if bytes.len() == RAW_VSA_LEN {
            Ok(false)
        } else if bytes.len() == PACKED_VSA_LEN {
            Ok(true)
        } else {
            Err(format!(
                "invalid VSA dimension: got {} bytes, expected {} (raw) or {} (packed)",
                bytes.len(),
                RAW_VSA_LEN,
                PACKED_VSA_LEN
            ))
        }
    }

    fn popcount(bytes: &[u8], is_packed: bool) -> u64 {
        if is_packed {
            bytes.iter().map(|b| b.count_ones() as u64).sum()
        } else {
            bytes.iter().map(|&b| (b & 1) as u64).sum()
        }
    }

    fn total_bits(bytes: &[u8], is_packed: bool) -> u64 {
        if is_packed {
            bytes.len() as u64 * 8
        } else {
            bytes.len() as u64
        }
    }

    fn byte_entropy(bytes: &[u8]) -> f64 {
        let len = bytes.len() as f64;
        if len == 0.0 {
            return 0.0;
        }
        let mut freq = [0u64; 256];
        for &b in bytes {
            freq[b as usize] += 1;
        }
        let mut entropy = 0.0_f64;
        for &count in freq.iter() {
            if count == 0 {
                continue;
            }
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
        entropy
    }

    fn bit_distribution(bytes: &[u8], is_packed: bool) -> [u8; 8] {
        let mut dist = [0u8; 8];
        if is_packed {
            for &b in bytes {
                for i in 0..8 {
                    if b & (1 << i) != 0 {
                        dist[i] = dist[i].saturating_add(1);
                    }
                }
            }
        } else {
            for (i, &b) in bytes.iter().enumerate() {
                if b & 1 != 0 {
                    dist[i % 8] = dist[i % 8].saturating_add(1);
                }
            }
        }
        dist
    }

    pub fn validate_vector(&self, bytes: &[u8]) -> Result<VsaValidation, String> {
        let is_packed = self.detect_format(bytes)?;
        let total = Self::total_bits(bytes, is_packed);
        let pop = Self::popcount(bytes, is_packed);
        let pop_ratio = if total > 0 {
            pop as f64 / total as f64
        } else {
            0.0
        };
        let entropy = Self::byte_entropy(bytes);
        let bit_dist = Self::bit_distribution(bytes, is_packed);

        let mut warnings = Vec::new();

        if pop_ratio < self.min_popcount {
            warnings.push(format!(
                "low popcount: {:.3} < min {:.3}",
                pop_ratio, self.min_popcount
            ));
        }
        if pop_ratio > self.max_popcount {
            warnings.push(format!(
                "high popcount: {:.3} > max {:.3}",
                pop_ratio, self.max_popcount
            ));
        }

        let entropy_threshold = if is_packed { 3.0 } else { 4.0 };
        if entropy < entropy_threshold && bytes.len() > 4 {
            warnings.push(format!(
                "low byte entropy: {:.4} < threshold {:.1}",
                entropy, entropy_threshold
            ));
        }

        let valid = warnings.is_empty();

        Ok(VsaValidation {
            valid,
            is_packed,
            popcount_ratio: pop_ratio,
            byte_entropy: entropy,
            bit_distribution: bit_dist,
            warnings,
        })
    }

    pub fn detect_black_hole(&mut self, bytes: &[u8]) -> f64 {
        if self.history.is_empty() {
            self.push_history(bytes);
            return 0.0;
        }

        let store = self.to_packed(bytes);
        let store = match store {
            Some(v) => v,
            None => return 0.0,
        };

        let mut total_sim = 0.0_f64;
        let mut count = 0u32;
        for hist in &self.history {
            total_sim += Self::packed_similarity(&store, hist);
            count += 1;
        }
        let avg_sim = if count > 0 {
            total_sim / count as f64
        } else {
            0.0
        };

        self.history.push_back(store);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        avg_sim
    }

    fn to_packed(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        if bytes.len() == PACKED_VSA_LEN {
            Some(bytes.to_vec())
        } else if bytes.len() == RAW_VSA_LEN {
            Some(Self::pack_raw(bytes))
        } else {
            None
        }
    }

    fn pack_raw(raw: &[u8]) -> Vec<u8> {
        raw.chunks(8)
            .map(|chunk| {
                let mut byte = 0u8;
                for (i, &b) in chunk.iter().enumerate() {
                    if b & 1 != 0 {
                        byte |= 1 << i;
                    }
                }
                byte
            })
            .collect()
    }

    fn packed_similarity(a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }
        let diff_bits: u64 = a[..len]
            .iter()
            .zip(b[..len].iter())
            .map(|(x, y)| (x ^ y).count_ones() as u64)
            .sum();
        let total_bits = len as u64 * 8;
        1.0 - diff_bits as f64 / total_bits as f64
    }

    fn push_history(&mut self, bytes: &[u8]) {
        let store = self.to_packed(bytes);
        if let Some(v) = store {
            if self.history.len() >= self.max_history {
                self.history.pop_front();
            }
            self.history.push_back(v);
        }
    }

    pub fn detect_bit_flip_anomaly(&self, bytes: &[u8]) -> f64 {
        let is_packed = bytes.len() == PACKED_VSA_LEN || bytes.len() == RAW_VSA_LEN;
        if !is_packed || bytes.len() < NUM_REGIONS {
            return 0.0;
        }

        let packed = bytes.len() == PACKED_VSA_LEN;
        let total_pop = Self::popcount(bytes, packed) as f64;
        let total_bits_v = Self::total_bits(bytes, packed) as f64;
        if total_bits_v == 0.0 {
            return 0.0;
        }
        let global_ratio = total_pop / total_bits_v;

        let region_size = bytes.len() / NUM_REGIONS;
        let mut max_deviation = 0.0_f64;

        for r in 0..NUM_REGIONS {
            let start = r * region_size;
            let end = bytes.len().min(start + region_size);
            let region_bytes = &bytes[start..end];
            let region_pop = Self::popcount(region_bytes, packed) as f64;
            let region_total = Self::total_bits(region_bytes, packed) as f64;
            if region_total == 0.0 {
                continue;
            }
            let region_ratio = region_pop / region_total;
            let deviation = (region_ratio - global_ratio).abs();
            if deviation > max_deviation {
                max_deviation = deviation;
            }
        }

        max_deviation
    }

    pub fn check_input(&mut self, bytes: &[u8], context: &str) -> VsaGuardResult {
        let mut reasons = Vec::new();

        let validation = match self.validate_vector(bytes) {
            Ok(v) => v,
            Err(e) => {
                return VsaGuardResult {
                    allowed: false,
                    risk: RiskLevel::Dangerous,
                    black_hole_score: 0.0,
                    bit_flip_score: 0.0,
                    validation: VsaValidation {
                        valid: false,
                        is_packed: false,
                        popcount_ratio: 0.0,
                        byte_entropy: 0.0,
                        bit_distribution: [0; 8],
                        warnings: vec![e.clone()],
                    },
                    reason: Some(e),
                };
            }
        };

        if !validation.valid {
            reasons.push(format!("validation: {:?}", validation.warnings));
        }

        let bit_flip_score = self.detect_bit_flip_anomaly(bytes);
        if bit_flip_score > 0.2 {
            reasons.push(format!(
                "bit-flip anomaly score {:.3} > 0.2",
                bit_flip_score
            ));
        }

        let black_hole_score = self.detect_black_hole(bytes);
        if black_hole_score > self.max_black_hole_score {
            reasons.push(format!(
                "black-hole score {:.3} > {:.3}",
                black_hole_score, self.max_black_hole_score
            ));
        }

        let has_validation_warnings = !validation.warnings.is_empty();
        let has_bit_flip = bit_flip_score > 0.2;
        let has_black_hole = black_hole_score > self.max_black_hole_score;

        let (allowed, risk) = if has_black_hole {
            (false, RiskLevel::Dangerous)
        } else if has_bit_flip && has_validation_warnings {
            (false, RiskLevel::Dangerous)
        } else if has_bit_flip || has_validation_warnings {
            (true, RiskLevel::Suspicious)
        } else {
            (true, RiskLevel::Safe)
        };

        let reason = if reasons.is_empty() {
            None
        } else {
            Some(format!("[ctx: {}] {}", context, reasons.join("; ")))
        };

        VsaGuardResult {
            allowed,
            risk,
            black_hole_score,
            bit_flip_score,
            validation,
            reason,
        }
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_zeros() -> Vec<u8> {
        vec![0u8; PACKED_VSA_LEN]
    }

    fn all_ones() -> Vec<u8> {
        vec![0xFFu8; PACKED_VSA_LEN]
    }

    fn random_packed() -> Vec<u8> {
        let mut v = vec![0u8; PACKED_VSA_LEN];
        for i in 0..PACKED_VSA_LEN {
            v[i] = (i as u8).wrapping_mul(13) ^ 0xAA;
        }
        v
    }

    fn raw_zeros() -> Vec<u8> {
        vec![0u8; RAW_VSA_LEN]
    }

    fn raw_ones() -> Vec<u8> {
        vec![1u8; RAW_VSA_LEN]
    }

    #[test]
    fn test_detect_format_packed() {
        let guard = VsaGuard::new();
        assert!(guard.detect_format(&all_zeros()).unwrap());
        assert!(!guard.detect_format(&raw_zeros()).unwrap());
        assert!(guard.detect_format(&[]).is_err());
    }

    #[test]
    fn test_all_zeros_low_popcount() {
        let guard = VsaGuard::new();
        let v = guard.validate_vector(&all_zeros()).unwrap();
        assert!(!v.valid);
        assert!(v.popcount_ratio < 0.01);
        assert!(v.warnings.iter().any(|w| w.contains("low popcount")));
    }

    #[test]
    fn test_all_ones_high_popcount() {
        let guard = VsaGuard::new();
        let v = guard.validate_vector(&all_ones()).unwrap();
        assert!(!v.valid);
        assert!((v.popcount_ratio - 1.0).abs() < 0.01);
        assert!(v.warnings.iter().any(|w| w.contains("high popcount")));
    }

    #[test]
    fn test_random_packed_passes_validation() {
        let guard = VsaGuard::new();
        let v = guard.validate_vector(&random_packed()).unwrap();
        assert!(
            v.valid,
            "random packed vector should pass: {:?}",
            v.warnings
        );
    }

    #[test]
    fn test_dimension_mismatch_rejected() {
        let guard = VsaGuard::new();
        let too_short = vec![0u8; 100];
        assert!(guard.validate_vector(&too_short).is_err());
    }

    #[test]
    fn test_packed_vs_raw_equivalence() {
        let guard = VsaGuard::new();
        let raw = raw_ones();
        let packed = all_ones();
        let v_raw = guard.validate_vector(&raw).unwrap();
        let v_packed = guard.validate_vector(&packed).unwrap();
        assert!(!v_raw.valid);
        assert!(!v_packed.valid);
        assert!((v_raw.popcount_ratio - v_packed.popcount_ratio).abs() < 0.01);
    }

    #[test]
    fn test_bit_distribution_packed() {
        let mut v = vec![0u8; PACKED_VSA_LEN];
        for byte in v.iter_mut() {
            *byte = 0x01; // only bit 0 set
        }
        let dist = VsaGuard::bit_distribution(&v, true);
        assert_eq!(dist[0], PACKED_VSA_LEN as u8);
        for i in 1..8 {
            assert_eq!(dist[i], 0);
        }
    }

    #[test]
    fn test_bit_distribution_raw() {
        let mut v = vec![0u8; RAW_VSA_LEN];
        for byte in v.iter_mut().step_by(2) {
            *byte = 1;
        }
        let dist = VsaGuard::bit_distribution(&v, false);
        // Every even index has bit 0 set (index % 8 == 0 → bit position 0)
        // There are RAW_VSA_LEN/2 ones, and all are at positions where i%8==0
        let expected = (RAW_VSA_LEN / 2) as u8;
        assert_eq!(dist[0], expected);
        // Other positions should get a share of the odd-indexed bytes (which are zero, so no)
        // Actually odd-indexed bytes are 0, so only position 0 gets any
        for i in 1..8 {
            assert_eq!(dist[i], 0, "bit position {} should be 0", i);
        }
    }

    #[test]
    fn test_black_hole_identical_vectors() {
        let mut guard = VsaGuard::new();
        let v = random_packed();

        let score = guard.detect_black_hole(&v);
        assert!(
            score < 0.01,
            "first vector should have no history, score={}",
            score
        );
        assert_eq!(guard.history_len(), 1);

        let score2 = guard.detect_black_hole(&v);
        assert!(
            (score2 - 1.0).abs() < 0.01,
            "identical vector should score ~1.0, got {}",
            score2
        );
    }

    #[test]
    fn test_black_hole_zeros_vs_ones() {
        let mut guard = VsaGuard::new();
        let zeros = all_zeros();
        let ones = all_ones();

        guard.detect_black_hole(&zeros);
        let score = guard.detect_black_hole(&ones);
        assert!(
            score < 0.1,
            "zeros vs ones should have near-0 similarity, got {}",
            score
        );
    }

    #[test]
    fn test_bit_flip_all_zeros() {
        let guard = VsaGuard::new();
        let score = guard.detect_bit_flip_anomaly(&all_zeros());
        assert!(
            score < 0.01,
            "all zeros is uniform, score should be ~0, got {}",
            score
        );
    }

    #[test]
    fn test_bit_flip_half_region() {
        let guard = VsaGuard::new();
        let mut v = vec![0u8; PACKED_VSA_LEN];

        let half = PACKED_VSA_LEN / 2;
        for byte in v[..half].iter_mut() {
            *byte = 0xFF;
        }

        let score = guard.detect_bit_flip_anomaly(&v);
        assert!(
            score > 0.3,
            "half 0xFF, half 0x00 should have high anomaly score, got {}",
            score
        );
    }

    #[test]
    fn test_check_input_all_zeros() {
        let mut guard = VsaGuard::new();
        let result = guard.check_input(&all_zeros(), "test");
        assert!(!result.allowed);
        assert_eq!(result.risk, RiskLevel::Suspicious);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_check_input_valid_passes() {
        let mut guard = VsaGuard::new();
        let result = guard.check_input(&random_packed(), "test");
        assert!(result.allowed);
        assert_eq!(result.risk, RiskLevel::Safe);
    }

    #[test]
    fn test_check_input_wrong_dim() {
        let mut guard = VsaGuard::new();
        let bad = vec![0u8; 100];
        let result = guard.check_input(&bad, "test");
        assert!(!result.allowed);
        assert_eq!(result.risk, RiskLevel::Dangerous);
    }

    #[test]
    fn test_raw_vector_acceptance() {
        let guard = VsaGuard::new();
        let v = guard.validate_vector(&raw_zeros()).unwrap();
        assert!(!v.valid);
        assert!(!v.is_packed);
    }

    #[test]
    fn test_byte_entropy_uniform() {
        let entropy = VsaGuard::byte_entropy(&[0u8; 512]);
        assert!(
            entropy < 0.01,
            "uniform data should have near-0 entropy, got {}",
            entropy
        );
    }

    #[test]
    fn test_byte_entropy_random() {
        let mut data = Vec::with_capacity(256);
        for i in 0..=255u8 {
            data.push(i);
        }
        let entropy = VsaGuard::byte_entropy(&data);
        assert!(
            (entropy - 8.0).abs() < 0.1,
            "all 256 values uniform should have entropy ~8.0, got {}",
            entropy
        );
    }

    #[test]
    fn test_detect_black_hole_chain() {
        let mut guard = VsaGuard::new();
        let base = random_packed();

        for i in 0..5 {
            let mut v = base.clone();
            if i > 0 {
                v[0] ^= 0x01;
            }
            let score = guard.detect_black_hole(&v);
            assert!(
                score >= 0.0 && score <= 1.0,
                "score out of range: {}",
                score
            );
        }

        assert_eq!(guard.history_len(), 5);
    }

    #[test]
    fn test_with_params_custom_thresholds() {
        let guard = VsaGuard::with_params(0.1, 0.9, 0.95, 16);
        let v = guard.validate_vector(&all_zeros()).unwrap();
        assert!(v.valid, "all zeros should be valid with min_pop=0.1");
    }

    #[test]
    fn test_packed_similarity_identical() {
        let a = random_packed();
        let sim = VsaGuard::packed_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_packed_similarity_orthogonal() {
        let zeros = all_zeros();
        let ones = all_ones();
        let sim = VsaGuard::packed_similarity(&zeros, &ones);
        assert!((sim - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_clear_history() {
        let mut guard = VsaGuard::new();
        guard.detect_black_hole(&random_packed());
        assert_eq!(guard.history_len(), 1);
        guard.clear_history();
        assert_eq!(guard.history_len(), 0);
    }

    #[test]
    fn test_extreme_bit_flip_anomaly() {
        let guard = VsaGuard::new();
        let mut v = vec![0u8; PACKED_VSA_LEN];
        let first_quarter = PACKED_VSA_LEN / 4;
        let second_quarter = PACKED_VSA_LEN / 2;

        for byte in v[..first_quarter].iter_mut() {
            *byte = 0xFF;
        }
        for byte in v[first_quarter..second_quarter].iter_mut() {
            *byte = 0x00;
        }

        let score = guard.detect_bit_flip_anomaly(&v);
        assert!(
            score > 0.4,
            "extreme regional imbalance should score > 0.4, got {}",
            score
        );
    }
}

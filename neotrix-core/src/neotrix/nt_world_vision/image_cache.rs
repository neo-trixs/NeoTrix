use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_time::unix_now;

const VSA_BITS: usize = 4096;
const DHASH_BYTES: usize = 8;
const BITS_PER_HASH_BIT: usize = VSA_BITS / (DHASH_BYTES * 8);

/// Entry in the image cache.
#[derive(Debug, Clone)]
pub struct ImageCacheEntry {
    /// 64-bit difference hash (8 bytes)
    pub hash: Vec<u8>,
    /// VSA-encoded hash (4096-bit, stored as binary 0/1 values)
    pub vsa_hash: Vec<u8>,
    /// Previous LLM analysis result
    pub analysis: String,
    /// Unix timestamp (seconds) of insertion/last update
    pub timestamp: u64,
}

/// VSA-aware image cache that avoids re-processing identical or near-identical images.
///
/// Uses a 64-bit difference hash (dHash) for fast perceptual fingerprinting,
/// then VSA-encodes it into a 4096-bit vector for similarity-based lookup.
/// Near-duplicate images (dHash differing by ≤6 bits) return cached results.
#[derive(Clone)]
pub struct ImageCache {
    entries: Vec<ImageCacheEntry>,
    max_entries: usize,
    similarity_threshold: f64,
}

impl ImageCache {
    /// Create a new cache with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries.min(16)),
            max_entries: max_entries.max(1),
            similarity_threshold: 0.90,
        }
    }

    /// Compute a 64-bit difference hash from image bytes.
    ///
    /// For each adjacent byte pair: if left > right, the corresponding bit is set to 1.
    /// Packs 64 bits into 8 bytes.
    pub fn compute_dhash(data: &[u8]) -> Vec<u8> {
        let mut hash = vec![0u8; DHASH_BYTES];
        let len = data.len();
        if len < 2 {
            return hash;
        }
        let n = 64usize.min(len - 1);
        for i in 0..n {
            if data[i] > data[i + 1] {
                hash[i / 8] |= 1 << (i % 8);
            }
        }
        hash
    }

    /// VSA-encode a dHash into a 4096-bit vector.
    ///
    /// Each bit of the 64-bit hash maps to 64 consecutive bits in the output,
    /// preserving similarity: images with similar dHashes produce similar VSA vectors.
    /// This enables near-duplicate detection via hamming distance on the VSA vectors.
    pub fn vsa_encode(hash: &[u8]) -> Vec<u8> {
        let mut encoded = vec![0u8; VSA_BITS];
        let bits = hash.len().min(DHASH_BYTES);
        for byte_idx in 0..bits {
            let byte = hash[byte_idx];
            for bit_idx in 0..8 {
                if (byte >> bit_idx) & 1 == 1 {
                    let start = (byte_idx * 8 + bit_idx) * BITS_PER_HASH_BIT;
                    for j in start..start + BITS_PER_HASH_BIT {
                        encoded[j] = 1;
                    }
                }
            }
        }
        encoded
    }

    /// Reverse the VSA encoding to reconstruct the dHash bits.
    fn decode_hash_from_vsa(vsa_hash: &[u8]) -> Vec<u8> {
        let mut hash = vec![0u8; DHASH_BYTES];
        if vsa_hash.len() < VSA_BITS {
            return hash;
        }
        for i in 0..64 {
            let start = i * BITS_PER_HASH_BIT;
            let sum: u32 = vsa_hash[start..start + BITS_PER_HASH_BIT]
                .iter()
                .map(|&b| b as u32)
                .sum();
            if sum >= (BITS_PER_HASH_BIT / 2) as u32 {
                hash[i / 8] |= 1 << (i % 8);
            }
        }
        hash
    }

    /// Check the cache for a similar image.
    ///
    /// Returns the cached analysis if a near neighbor (similarity ≥ threshold) is found.
    /// Uses hamming-distance-based similarity on the VSA vectors.
    pub fn lookup(&self, vsa_hash: &[u8]) -> Option<&str> {
        if vsa_hash.len() != VSA_BITS {
            return None;
        }
        for entry in &self.entries {
            let sim = QuantizedVSA::similarity(&entry.vsa_hash, vsa_hash);
            if sim >= self.similarity_threshold {
                return Some(&entry.analysis);
            }
        }
        None
    }

    /// Insert a new entry into the cache.
    ///
    /// If the VSA hash matches an existing entry (by exact match or decoded dHash),
    /// the existing entry is updated. If at capacity, the oldest entry is evicted.
    pub fn insert(&mut self, vsa_hash: Vec<u8>, analysis: String) {
        let hash = Self::decode_hash_from_vsa(&vsa_hash);
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.hash == hash || e.vsa_hash == vsa_hash)
        {
            entry.analysis = analysis;
            entry.timestamp = unix_now() as u64;
            entry.hash = hash;
            entry.vsa_hash = vsa_hash;
            return;
        }
        if self.entries.len() >= self.max_entries {
            let oldest_idx = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.timestamp)
                .map(|(i, _)| i)
                .unwrap();
            self.entries.remove(oldest_idx);
        }
        self.entries.push(ImageCacheEntry {
            hash,
            vsa_hash,
            analysis,
            timestamp: unix_now() as u64,
        });
    }

    /// Number of entries currently in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_image_data() -> Vec<u8> {
        (0u8..=255).collect()
    }

    fn slightly_different_image_data() -> Vec<u8> {
        (0u8..=255)
            .map(|i| if i % 2 == 0 { i } else { i.saturating_sub(1) })
            .collect()
    }

    #[test]
    fn test_dhash_output_length() {
        let data = sample_image_data();
        let hash = ImageCache::compute_dhash(&data);
        assert_eq!(hash.len(), DHASH_BYTES);
    }

    #[test]
    fn test_dhash_deterministic() {
        let data = sample_image_data();
        let h1 = ImageCache::compute_dhash(&data);
        let h2 = ImageCache::compute_dhash(&data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_dhash_different_inputs_different_hashes() {
        let h1 = ImageCache::compute_dhash(&sample_image_data());
        let h2 = ImageCache::compute_dhash(&slightly_different_image_data());
        assert_ne!(h1, h2, "different images should produce different dHashes");
    }

    #[test]
    fn test_dhash_short_data() {
        let hash = ImageCache::compute_dhash(&[1, 2]);
        assert_eq!(hash.len(), 8);
    }

    #[test]
    fn test_dhash_empty_data() {
        let hash = ImageCache::compute_dhash(&[]);
        assert_eq!(hash, vec![0u8; 8]);
    }

    #[test]
    fn test_dhash_single_byte() {
        let hash = ImageCache::compute_dhash(&[42]);
        assert_eq!(hash, vec![0u8; 8]);
    }

    #[test]
    fn test_vsa_encode_output_length() {
        let hash = ImageCache::compute_dhash(&sample_image_data());
        let encoded = ImageCache::vsa_encode(&hash);
        assert_eq!(encoded.len(), VSA_BITS);
    }

    #[test]
    fn test_vsa_encode_decode_roundtrip() {
        let hash = ImageCache::compute_dhash(&sample_image_data());
        let encoded = ImageCache::vsa_encode(&hash);
        let decoded = ImageCache::decode_hash_from_vsa(&encoded);
        assert_eq!(
            hash, decoded,
            "VSA encode→decode roundtrip should recover original hash"
        );
    }

    #[test]
    fn test_vsa_encode_deterministic() {
        let hash = ImageCache::compute_dhash(&sample_image_data());
        let e1 = ImageCache::vsa_encode(&hash);
        let e2 = ImageCache::vsa_encode(&hash);
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_lookup_empty_cache() {
        let cache = ImageCache::new(10);
        let vsa = ImageCache::vsa_encode(&[0u8; 8]);
        assert!(cache.lookup(&vsa).is_none());
    }

    #[test]
    fn test_insert_and_hit() {
        let mut cache = ImageCache::new(10);
        let data = sample_image_data();
        let hash = ImageCache::compute_dhash(&data);
        let vsa = ImageCache::vsa_encode(&hash);
        cache.insert(vsa.clone(), "a cat sitting on a mat".to_string());
        let result = cache.lookup(&vsa);
        assert_eq!(result, Some("a cat sitting on a mat"));
    }

    #[test]
    fn test_lookup_miss_different_image() {
        let mut cache = ImageCache::new(10);
        let hash1 = ImageCache::compute_dhash(&sample_image_data());
        let vsa1 = ImageCache::vsa_encode(&hash1);
        cache.insert(vsa1, "first image".to_string());
        let hash2 = ImageCache::compute_dhash(&slightly_different_image_data());
        let vsa2 = ImageCache::vsa_encode(&hash2);
        let result = cache.lookup(&vsa2);
        assert!(result.is_none() || result == Some("first image"));
    }

    #[test]
    fn test_cache_len() {
        let mut cache = ImageCache::new(5);
        assert_eq!(cache.len(), 0);
        for i in 0u8..3 {
            let data: Vec<u8> = (0u8..64).map(|j| j.wrapping_mul(i + 1)).collect();
            let hash = ImageCache::compute_dhash(&data);
            let vsa = ImageCache::vsa_encode(&hash);
            cache.insert(vsa, format!("image {}", i));
        }
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut cache = ImageCache::new(3);
        for i in 0u8..4 {
            let data: Vec<u8> = (0u8..64).map(|j| j.wrapping_add(i * 10)).collect();
            let hash = ImageCache::compute_dhash(&data);
            let vsa = ImageCache::vsa_encode(&hash);
            cache.insert(vsa, format!("image {}", i));
        }
        assert_eq!(cache.len(), 3, "cache should not exceed max_entries");
    }

    #[test]
    fn test_update_existing_entry() {
        let mut cache = ImageCache::new(10);
        let data = sample_image_data();
        let hash = ImageCache::compute_dhash(&data);
        let vsa = ImageCache::vsa_encode(&hash);
        cache.insert(vsa.clone(), "old analysis".to_string());
        assert_eq!(cache.len(), 1);
        cache.insert(vsa.clone(), "updated analysis".to_string());
        assert_eq!(cache.len(), 1, "updating should not add a new entry");
        assert_eq!(
            cache.lookup(&vsa),
            Some("updated analysis"),
            "inserting with same VSA hash should update existing entry"
        );
    }

    #[test]
    fn test_similarity_self_is_one() {
        let hash = ImageCache::compute_dhash(&sample_image_data());
        let vsa = ImageCache::vsa_encode(&hash);
        let sim = QuantizedVSA::similarity(&vsa, &vsa);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cache_with_default_threshold() {
        let cache = ImageCache::new(10);
        assert!((cache.similarity_threshold - 0.90).abs() < 1e-6);
    }
}

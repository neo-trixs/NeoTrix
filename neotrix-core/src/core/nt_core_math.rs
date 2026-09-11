#![forbid(unsafe_code)]

/// f32 向量余弦相似度，返回 f64 精度。
/// 长度不匹配或零向量返回 0.0（NumPy 约定）。
pub fn cosine_similarity_f32(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(&x, &y)| (x as f64) * (y as f64)).sum();
    let norm_a: f64 = a.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// f32 向量余弦相似度，返回 f32（用于需要 f32 返回值的场景）。
pub fn cosine_similarity_f32_f32(a: &[f32], b: &[f32]) -> f32 {
    cosine_similarity_f32(a, b) as f32
}

/// f64 向量余弦相似度，返回 f64。
/// 长度不匹配或零向量返回 0.0。
pub fn cosine_similarity_f64(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// f32 向量余弦距离 = 1.0 - similarity。零向量返回 1.0。
pub fn cosine_distance_f32(a: &[f32], b: &[f32]) -> f32 {
    1.0 - cosine_similarity_f32(a, b) as f32
}

/// f64 向量余弦距离 = 1.0 - similarity。零向量返回 1.0。
pub fn cosine_distance_f64(a: &[f64], b: &[f64]) -> f64 {
    1.0 - cosine_similarity_f64(a, b)
}

/// 字节向量余弦相似度（基于 Hamming 距离）。
pub fn cosine_similarity_bytes(a: &[u8], b: &[u8]) -> f64 {
    let hd: u64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones() as u64)
        .sum();
    let dim = (a.len().min(b.len()) * 8) as f64;
    if dim == 0.0 {
        return 0.0;
    }
    1.0 - 2.0 * hd as f64 / dim
}

/// URL 规范化: 去空白 + 去 fragment (#) + 尾斜杠 + 域名小写。
/// 用于跨模块 URL 去重 (asset_graph, experience absorb, KB dedup)。
pub fn normalize_url(url: &str) -> String {
    let mut u = url.trim().to_string();
    if let Some(idx) = u.find('#') {
        u.truncate(idx);
    }
    u = u.trim_end_matches('/').to_string();
    // 域名小写 (仅 http/https)
    if let Some(pos) = u.find("://") {
        let rest = &u[pos + 3..];
        if let Some(slash) = rest.find('/') {
            let (host, path) = rest.split_at(slash);
            u = format!("{}://{}{}", &u[..pos], host.to_lowercase(), path);
        } else {
            let host = rest;
            u = format!("{}://{}", &u[..pos], host.to_lowercase());
        }
    }
    u
}

/// Hamming 距离：两个字节向量不同的比特数。
pub fn hamming_distance(a: &[u8], b: &[u8]) -> u64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones() as u64)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_identical() {
        let a = [1.0, 0.0, 0.0];
        assert!((cosine_similarity_f32(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn f32_orthogonal() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        assert!(cosine_similarity_f32(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn f32_mismatch_length() {
        assert_eq!(cosine_similarity_f32(&[1.0, 0.0], &[1.0, 0.0, 0.0]), 0.0);
    }

    #[test]
    fn f32_zero_norm() {
        assert_eq!(cosine_similarity_f32(&[0.0, 0.0], &[1.0, 0.0]), 0.0);
    }

    #[test]
    fn f32_f32_identical() {
        let a = [1.0f32, 0.0, 0.0];
        assert!((cosine_similarity_f32_f32(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn f32_f32_zero_norm() {
        assert_eq!(cosine_similarity_f32_f32(&[0.0], &[1.0]), 0.0);
    }

    #[test]
    fn f64_identical() {
        let a = [1.0, 0.0, 0.0];
        assert!((cosine_similarity_f64(&a, &a) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn f64_zero_norm() {
        assert_eq!(cosine_similarity_f64(&[0.0], &[1.0]), 0.0);
    }

    #[test]
    fn f64_both_zero_norm() {
        assert_eq!(cosine_similarity_f64(&[0.0], &[0.0]), 0.0);
    }

    #[test]
    fn cosine_distance_f32_identical() {
        let v = [1.0f32, 0.0, 0.0];
        assert!(cosine_distance_f32(&v, &v).abs() < 1e-6);
    }

    #[test]
    fn cosine_distance_f32_orthogonal() {
        let a = [1.0f32, 0.0];
        let b = [0.0f32, 1.0];
        assert!((cosine_distance_f32(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_distance_f64_identical() {
        let v = [1.0f64, 0.0, 0.0];
        assert!(cosine_distance_f64(&v, &v).abs() < 1e-9);
    }

    #[test]
    fn bytes_identical() {
        let v = vec![0b10101010u8];
        assert!((cosine_similarity_bytes(&v, &v) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bytes_opposite() {
        let a = vec![0b00000000u8];
        let b = vec![0b11111111u8];
        assert!((cosine_similarity_bytes(&a, &b) - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn bytes_big_vectors() {
        let a = vec![0xFFu8; 64];
        let b = vec![0xFFu8; 64];
        assert!((cosine_similarity_bytes(&a, &b) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hamming_identical() {
        let a = vec![0b10101010u8];
        assert_eq!(hamming_distance(&a, &a), 0);
    }

    #[test]
    fn hamming_all_bits() {
        let a = vec![0b00000000u8];
        let b = vec![0b11111111u8];
        assert_eq!(hamming_distance(&a, &b), 8);
    }

    #[test]
    fn normalize_url_strips_fragment() {
        assert_eq!(normalize_url("https://a.com/x#sec"), "https://a.com/x");
    }

    #[test]
    fn normalize_url_strips_trailing_slash() {
        assert_eq!(normalize_url("https://a.com/x/"), "https://a.com/x");
        assert_eq!(normalize_url("https://a.com/"), "https://a.com");
    }

    #[test]
    fn normalize_url_lowercases_domain() {
        assert_eq!(normalize_url("https://A.COM/X"), "https://a.com/X");
        assert_eq!(normalize_url("HTTP://Example.COM/Path"), "http://example.com/Path");
    }

    #[test]
    fn normalize_url_trims_whitespace() {
        assert_eq!(normalize_url("  https://a.com/x  "), "https://a.com/x");
    }

    #[test]
    fn normalize_url_no_scheme() {
        assert_eq!(normalize_url("example.com/x#y"), "example.com/x");
    }
}

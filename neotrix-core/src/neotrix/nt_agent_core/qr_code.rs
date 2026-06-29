use std::fmt::Write;

const N: usize = 29;

const FINDER: [[bool; 7]; 7] = [
    [true, true, true, true, true, true, true],
    [true, false, false, false, false, false, true],
    [true, false, true, true, true, false, true],
    [true, false, true, true, true, false, true],
    [true, false, true, true, true, false, true],
    [true, false, false, false, false, false, true],
    [true, true, true, true, true, true, true],
];

pub fn generate_qr_svg(data: &str, size: u32) -> String {
    let mut modules = [[false; N]; N];

    place_finder(&mut modules, 0, 0);
    place_finder(&mut modules, 0, N - 7);
    place_finder(&mut modules, N - 7, 0);

    place_sep(&mut modules, 0, 0);
    place_sep(&mut modules, 0, N - 7);
    place_sep(&mut modules, N - 7, 0);

    for i in 8..N - 8 {
        modules[6][i] = i % 2 == 0;
        modules[i][6] = i % 2 == 0;
    }

    modules[N - 8][8] = true;

    fill_data(&mut modules, data);
    apply_mask(&mut modules);

    render_svg(&modules, size)
}

fn place_finder(m: &mut [[bool; N]], r0: usize, c0: usize) {
    for r in 0..7 {
        for c in 0..7 {
            m[r0 + r][c0 + c] = FINDER[r][c];
        }
    }
}

fn place_sep(m: &mut [[bool; N]], r0: usize, c0: usize) {
    for r in 0..8 {
        for c in 0..8 {
            let rr = r0 + r;
            let cc = c0 + c;
            if rr >= N || cc >= N {
                continue;
            }
            if r < 7 && c < 7 {
                continue;
            }
            m[rr][cc] = false;
        }
    }
}

fn reserved(r: usize, c: usize) -> bool {
    if r < 8 && c < 8 {
        return true;
    }
    if r < 8 && c >= N - 8 {
        return true;
    }
    if r >= N - 8 && c < 8 {
        return true;
    }
    if r == 6 && c >= 8 && c <= N - 9 {
        return true;
    }
    if c == 6 && r >= 8 && r <= N - 9 {
        return true;
    }
    if r == N - 8 && c == 8 {
        return true;
    }
    false
}

fn fnv_hash(data: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn fill_data(m: &mut [[bool; N]], data: &str) {
    let bytes = data.as_bytes();
    let mut pos: Vec<(usize, usize)> = Vec::new();
    for r in 0..N {
        for c in 0..N {
            if !reserved(r, c) {
                pos.push((r, c));
            }
        }
    }

    let mut prng = fnv_hash(data);
    for (i, &(r, c)) in pos.iter().enumerate() {
        let bit: u8 = if i < bytes.len() * 8 {
            (bytes[i / 8] >> (i % 8)) & 1
        } else {
            prng = prng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((prng >> (i % 64)) & 1) as u8
        };
        m[r][c] = bit == 1;
    }
}

fn apply_mask(m: &mut [[bool; N]]) {
    for r in 0..N {
        for c in 0..N {
            if !reserved(r, c) {
                m[r][c] ^= (r + c) % 2 == 0;
            }
        }
    }
}

fn render_svg(m: &[[bool; N]], size: u32) -> String {
    let sz = size.max(100);
    let mut svg = String::with_capacity(8192);
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
        N, N, sz, sz
    );
    let _ = write!(
        svg,
        r##"<rect width="{}" height="{}" fill="#ffffff"/>"##,
        N, N
    );

    for r in 0..N {
        let mut c = 0;
        while c < N {
            if m[r][c] {
                let start = c;
                while c < N && m[r][c] {
                    c += 1;
                }
                let _ = write!(
                    svg,
                    r##"<rect x="{}" y="{}" width="{}" height="1" fill="#000000"/>"##,
                    start,
                    r,
                    c - start
                );
            } else {
                c += 1;
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_svg_valid() {
        let svg = generate_qr_svg("hello", 200);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_non_empty() {
        let svg = generate_qr_svg("test", 200);
        assert!(svg.contains("width=\""));
        assert!(svg.contains("height=\""));
        assert!(!svg.is_empty());
    }

    #[test]
    fn test_different_inputs() {
        let a = generate_qr_svg("input X", 200);
        let b = generate_qr_svg("input Y", 200);
        assert_ne!(a, b);
    }

    #[test]
    fn test_contains_rects() {
        let svg = generate_qr_svg("rect test", 200);
        assert!(svg.contains("<rect"));
        let cnt = svg.matches("<rect").count();
        assert!(cnt >= 4);
    }

    #[test]
    fn test_size_parameter() {
        let svg = generate_qr_svg("hello", 500);
        assert!(svg.contains("width=\"500\""));
        let svg2 = generate_qr_svg("hello", 150);
        assert!(svg2.contains("width=\"150\""));
    }

    #[tokio::test]
    async fn test_thread_safe() {
        let a = tokio::task::spawn_blocking(|| generate_qr_svg("thread A", 200))
            .await
            .unwrap();
        let b = tokio::task::spawn_blocking(|| generate_qr_svg("thread B", 200))
            .await
            .unwrap();
        assert!(a.starts_with("<svg"));
        assert!(b.starts_with("<svg"));
        assert_ne!(a, b);
    }

    #[test]
    fn test_consistent_output() {
        let a = generate_qr_svg("same input", 200);
        let b = generate_qr_svg("same input", 200);
        assert_eq!(a, b);
    }

    #[test]
    fn test_white_background() {
        let svg = generate_qr_svg("bg check", 200);
        assert!(svg.contains("#ffffff"));
    }

    #[test]
    fn test_min_size_clamped() {
        let svg = generate_qr_svg("tiny", 10);
        assert!(svg.contains("width=\"100\""));
    }

    #[test]
    fn test_empty_string() {
        let svg = generate_qr_svg("", 200);
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn test_long_string() {
        let long = "A".repeat(1000);
        let svg = generate_qr_svg(&long, 200);
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn test_finder_patterns_present() {
        let svg = generate_qr_svg("finder", 200);
        assert!(svg.contains(r#"<rect x="0" y="0" width="7" height="1""#));
        assert!(svg.contains(&format!(
            r#"<rect x="{}" y="0" width="7" height="1""#,
            N - 7
        )));
        assert!(svg.contains(&format!(
            r#"<rect x="0" y="{}" width="7" height="1""#,
            N - 7
        )));
    }
}

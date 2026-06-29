use crate::core::nt_core_hcube::QuantizedVSA;
use crate::core::nt_core_hcube::VSA_DIM;
use super::token_types::TokenValue;

fn string_to_seed(s: &str) -> u64 {
    s.bytes().fold(0xDEAD_BEEF_CAFE_F00Eu64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as u64)
    })
}

pub fn encode_token_name(name: &str) -> Vec<u8> {
    let seed = string_to_seed(name);
    QuantizedVSA::xor_bind(
        &QuantizedVSA::seeded_random(seed, VSA_DIM),
        &QuantizedVSA::seeded_random(seed ^ 0xABCD_1234, VSA_DIM),
    )
}

pub fn encode_token_value(value: &TokenValue) -> Vec<u8> {
    let seed = match value {
        TokenValue::Color { r, g, b, a } => {
            let ri = (r * 255.0) as u64;
            let gi = (g * 255.0) as u64;
            let bi = (b * 255.0) as u64;
            let ai = (a * 255.0) as u64;
            ri.wrapping_mul(31).wrapping_add(gi)
                .wrapping_mul(31).wrapping_add(bi)
                .wrapping_mul(31).wrapping_add(ai)
        }
        TokenValue::Spacing(v) => (*v as u64).wrapping_mul(1000),
        TokenValue::Easing { x1, y1, x2, y2 } => {
            let xi1 = (x1 * 1000.0) as u64;
            let yi1 = (y1 * 1000.0) as u64;
            let xi2 = (x2 * 1000.0) as u64;
            let yi2 = (y2 * 1000.0) as u64;
            xi1.wrapping_mul(31).wrapping_add(yi1)
                .wrapping_mul(31).wrapping_add(xi2)
                .wrapping_mul(31).wrapping_add(yi2)
        }
        TokenValue::Shadow { offset_x, offset_y, blur, spread, r, g, b, a } => {
            let ox = (offset_x * 100.0) as u64;
            let oy = (offset_y * 100.0) as u64;
            let bl = *blur as u64;
            let sp = *spread as u64;
            let ri = (r * 255.0) as u64;
            let gi = (g * 255.0) as u64;
            let bi = (b * 255.0) as u64;
            let ai = (a * 255.0) as u64;
            ox.wrapping_mul(31).wrapping_add(oy)
                .wrapping_mul(31).wrapping_add(bl)
                .wrapping_mul(31).wrapping_add(sp)
                .wrapping_mul(31).wrapping_add(ri)
                .wrapping_mul(31).wrapping_add(gi)
                .wrapping_mul(31).wrapping_add(bi)
                .wrapping_mul(31).wrapping_add(ai)
        }
        TokenValue::Motion { duration_ms, stiffness, damping } => {
            (*duration_ms as u64)
                .wrapping_mul(31).wrapping_add(*stiffness as u64)
                .wrapping_mul(31).wrapping_add(*damping as u64)
        }
        TokenValue::Font { family, size, weight } => {
            let name_seed: u64 = family.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
            name_seed.wrapping_mul(31).wrapping_add(*size as u64)
                .wrapping_mul(31).wrapping_add(*weight as u64)
        }
        TokenValue::Radius(v) => (*v as u64).wrapping_mul(10),
        TokenValue::Opacity(v) => (v * 255.0) as u64,
    };
    QuantizedVSA::seeded_random(seed, VSA_DIM)
}

pub fn bind_name_to_value(name: &str, value: &TokenValue) -> Vec<u8> {
    let name_vsa = encode_token_name(name);
    let value_vsa = encode_token_value(value);
    QuantizedVSA::xor_bind(&name_vsa, &value_vsa)
}

pub fn compose_tokens(token_a: &[u8], token_b: &[u8]) -> Vec<u8> {
    QuantizedVSA::bundle(&[token_a, token_b])
}

pub fn token_similarity(a: &[u8], b: &[u8]) -> f64 {
    QuantizedVSA::similarity(a, b)
}

pub fn hierarchy_bind(global: &[u8], semantic: &[u8]) -> Vec<u8> {
    QuantizedVSA::xor_bind(global, semantic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_name_deterministic() {
        let a = encode_token_name("color-brand-primary");
        let b = encode_token_name("color-brand-primary");
        assert_eq!(a, b);
    }

    #[test]
    fn test_encode_name_different() {
        let a = encode_token_name("color-brand-primary");
        let b = encode_token_name("color-brand-secondary");
        assert!(token_similarity(&a, &b) < 0.7);
    }

    #[test]
    fn test_bind_name_to_value() {
        let value = TokenValue::Color { r: 0.07, g: 0.09, b: 0.20, a: 1.0 };
        let bound = bind_name_to_value("color-brand-primary", &value);
        assert_eq!(bound.len(), VSA_DIM);
    }

    #[test]
    fn test_compose_tokens() {
        let a = encode_token_name("color-brand-primary");
        let b = encode_token_name("spacing-lg");
        let composed = compose_tokens(&a, &b);
        let sim_a = token_similarity(&composed, &a);
        let sim_b = token_similarity(&composed, &b);
        assert!(sim_a > 0.55 && sim_b > 0.55);
    }

    #[test]
    fn test_hierarchy_bind() {
        let global = encode_token_name("color");
        let semantic = encode_token_name("brand-primary");
        let bound = hierarchy_bind(&global, &semantic);
        assert_eq!(bound.len(), VSA_DIM);
    }
}

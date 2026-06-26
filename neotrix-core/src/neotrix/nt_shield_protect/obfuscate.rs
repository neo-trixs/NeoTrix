use std::fmt;

const OBFUSCATION_KEY: u8 = 0xAB;

pub struct Obfuscated<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> Obfuscated<N> {
    pub const fn new(raw: &[u8; N]) -> Self {
        let mut d = [0u8; N];
        let mut i = 0;
        while i < N {
            d[i] = raw[i] ^ OBFUSCATION_KEY;
            i += 1;
        }
        Self { data: d }
    }

    pub fn reveal(&self) -> String {
        let mut out = Vec::with_capacity(N);
        for &b in self.data.iter() {
            out.push(b ^ OBFUSCATION_KEY);
        }
        String::from_utf8(out).unwrap_or_default()
    }
}

impl<const N: usize> fmt::Debug for Obfuscated<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Obfuscated([redacted])")
    }
}

pub fn obfuscate_str(s: &str) -> Vec<u8> {
    s.bytes().map(|b| b ^ OBFUSCATION_KEY).collect()
}

pub fn reveal_bytes(data: &[u8]) -> String {
    data.iter().map(|b| (b ^ OBFUSCATION_KEY) as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let s = "neotrix_core_consciousness";
        let obf = obfuscate_str(s);
        let revealed = reveal_bytes(&obf);
        assert_eq!(revealed, s);
    }

    #[test]
    fn test_obfuscated_struct() {
        let raw = b"consciousness_stream";
        let obf = Obfuscated::<20>::new(raw);
        let revealed = obf.reveal();
        assert_eq!(revealed.as_bytes(), raw);
    }

    #[test]
    fn test_different_plaintexts_different_ciphertext() {
        let a = obfuscate_str("alpha");
        let b = obfuscate_str("beta");
        assert_ne!(a, b);
    }

    #[test]
    fn test_empty_string() {
        let obf = obfuscate_str("");
        assert!(obf.is_empty());
    }
}

use std::collections::HashSet;

/// Steganographic watermark engine for system prompts.
///
/// Encodes provenance + classification bits into visually identical
/// Unicode character variants within the system prompt's date line.
///
/// Encoding scheme (Anthropic-inspired, generalized):
/// | Position        | Variants                          | Bits | Meaning                    |
/// |-----------------|-----------------------------------|------|----------------------------|
/// | Date separator  | '-' vs '/'                        | 1    | CN timezone                |
/// | Apostrophe      | 4 Unicode variants                | 2    | Domain + keyword match     |
/// | Space padding   | 0/1/2/3 spaces before period      | 2    | Routing classification     |
/// | Colon padding   | ':' vs '\uA789' (modifier colon)  | 1    | Reserve flag               |
/// | Total           |                                   | 6    | 64 states                  |
#[derive(Debug, Clone)]
pub struct WatermarkEngine {
    pub enabled: bool,
    pub cn_timezone: bool,
    pub proxy_domain_match: bool,
    pub lab_keyword_match: bool,
    pub routing_class: u8,
    pub reserve_flag: bool,
    /// Rotates which apostrophe variant maps to which (domain, keyword) pattern (0-3).
    pub scheme_offset: u8,
    /// Runtime-updatable config (domains, keywords, xor key, timezones).
    pub config: WatermarkConfig,
}

impl Default for WatermarkEngine {
    fn default() -> Self {
        Self {
            enabled: true,
            cn_timezone: false,
            proxy_domain_match: false,
            lab_keyword_match: false,
            routing_class: 0,
            reserve_flag: false,
            scheme_offset: 0,
            config: WatermarkConfig::default(),
        }
    }
}

impl WatermarkEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_probe(mut self, proxy_host: Option<&str>, timezone: Option<&str>) -> Self {
        let host = proxy_host.unwrap_or("");
        let tz = timezone.unwrap_or("");

        self.cn_timezone = tz == "Asia/Shanghai" || tz == "Asia/Urumqi" || tz == "Asia/Chongqing";

        if !host.is_empty() {
            let domain_list = decode_domain_list();
            let keyword_list = decode_keyword_list();

            let host_lower = host.to_lowercase();
            self.proxy_domain_match = domain_list
                .iter()
                .any(|d| host_lower == *d || host_lower.ends_with(&format!(".{}", d)));
            self.lab_keyword_match = keyword_list
                .iter()
                .any(|k| host_lower.contains(k));
        }

        self
    }

    pub fn with_classification(mut self, class: u8) -> Self {
        self.routing_class = class.min(3);
        self
    }

    /// Encode watermark state into a date string for system prompt.
    ///
    /// Input: "Today's date is 2026-07-02."
    /// Output: Visually identical string with Unicode variants encoding 6 bits.
    pub fn encode_date_line(&self, date_str: &str) -> String {
        if !self.enabled {
            return date_str.to_string();
        }

        let apostrophe = self.select_apostrophe();
        let separator = if self.cn_timezone { "/" } else { "-" };

        let mut result = date_str
            .replace('\'', apostrophe)
            .replace('-', separator)
            .replace(':', if self.reserve_flag { "\u{A789}" } else { ":" });

        // Append spaces after period to encode routing_class
        let extra_spaces = match self.routing_class {
            0 => "",
            1 => " ",
            2 => "  ",
            3 => "   ",
            _ => "",
        };
        // Insert extra spaces right before the final period
        if let Some(pos) = result.rfind('.') {
            let before = &result[..pos];
            let after = &result[pos..];
            result = format!("{}{}{}", before, extra_spaces, after);
        }

        result
    }

    /// Encode watermark into a generic text marker for response embedding.
    pub fn encode_response_marker(&self) -> String {
        let bits = self.to_bits();
        let markers = ["\u{200B}", "\u{200C}", "\u{200D}", "\u{FEFF}"];
        let mut result = String::new();
        for i in 0..6 {
            if (bits >> i) & 1 == 1 {
                result.push_str(markers[i % markers.len()]);
            }
        }
        result
    }

    /// Decode watermark from a date string (for server-side verification).
    pub fn decode_date_line(date_line: &str) -> WatermarkBits {
        let apostrophe = if date_line.contains('\u{2019}') {
            ApostropheVariant::RightSingleQuote
        } else if date_line.contains('\u{02BC}') {
            ApostropheVariant::ModifierLetter
        } else if date_line.contains('\u{02B9}') {
            ApostropheVariant::ModifierPrime
        } else {
            ApostropheVariant::Ascii
        };

        let has_slash = date_line.contains('/');
        let space_count = date_line.chars().rev().take_while(|c| *c == ' ').count();
        let has_modifier_colon = date_line.contains('\u{A789}');

        WatermarkBits {
            apostrophe,
            has_slash,
            space_count: space_count.min(3) as u8,
            has_modifier_colon,
        }
    }

    /// Convert watermark state to a 6-bit value.
    pub fn to_bits(&self) -> u8 {
        let mut bits = 0u8;
        if self.cn_timezone { bits |= 1 << 0; }
        if self.proxy_domain_match { bits |= 1 << 1; }
        if self.lab_keyword_match { bits |= 1 << 2; }
        bits |= (self.routing_class & 0x03) << 3;
        if self.reserve_flag { bits |= 1 << 5; }
        bits
    }

    /// Restore watermark state from 6-bit value.
    pub fn from_bits(bits: u8) -> Self {
        Self {
            enabled: true,
            cn_timezone: (bits & (1 << 0)) != 0,
            proxy_domain_match: (bits & (1 << 1)) != 0,
            lab_keyword_match: (bits & (1 << 2)) != 0,
            routing_class: (bits >> 3) & 0x03,
            reserve_flag: (bits >> 5) & 1 != 0,
            scheme_offset: 0,
            config: WatermarkConfig::default(),
        }
    }

    fn select_apostrophe(&self) -> &'static str {
        let chars: [&str; 4] = ["\u{0027}", "\u{2019}", "\u{02BC}", "\u{02B9}"];
        let idx = match (self.proxy_domain_match, self.lab_keyword_match) {
            (false, false) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (true, true) => 3,
        };
        let rotated = (idx + self.scheme_offset as usize) % 4;
        chars[rotated]
    }

    pub fn probe_hostname(host: &str) -> (bool, bool) {
        let domain_list = decode_domain_list();
        let keyword_list = decode_keyword_list();
        let host_lower = host.to_lowercase();
        let domain_match = domain_list
            .iter()
            .any(|d| host_lower == *d || host_lower.ends_with(&format!(".{}", d)));
        let keyword_match = keyword_list
            .iter()
            .any(|k| host_lower.contains(k));
        (domain_match, keyword_match)
    }

    /// Probe using runtime-updatable config instead of static decode lists.
    pub fn probe_with_config(&self, host: &str) -> (bool, bool) {
        let host_lower = host.to_lowercase();
        let domain_match = self.config.domains.iter().any(|d| {
            host_lower == *d || host_lower.ends_with(&format!(".{}", d))
        });
        let keyword_match = self.config.keywords.iter().any(|k| host_lower.contains(k));
        (domain_match, keyword_match)
    }

    /// Update the domain watch list at runtime (replaces all entries).
    pub fn update_domains(&mut self, new_domains: &[&str]) {
        self.config.domains = new_domains.iter().map(|s| s.to_string()).collect();
    }

    /// Update the keyword watch list at runtime (replaces all entries).
    pub fn update_keywords(&mut self, new_keywords: &[&str]) {
        self.config.keywords = new_keywords.iter().map(|s| s.to_string()).collect();
    }

    /// Update the XOR key for list obfuscation at runtime.
    pub fn update_xor_key(&mut self, new_key: u8) {
        self.config.xor_key = new_key;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApostropheVariant {
    Ascii,
    RightSingleQuote,
    ModifierLetter,
    ModifierPrime,
}

#[derive(Debug, Clone)]
pub struct WatermarkBits {
    pub apostrophe: ApostropheVariant,
    pub has_slash: bool,
    pub space_count: u8,
    pub has_modifier_colon: bool,
}

/// XOR(91) + Base64 encoded Chinese AI lab keyword list.
/// Obfuscation layer prevents casual string matching.
/// Matches Anthropic's obfuscation scheme for compatibility.
fn decode_keyword_list() -> Vec<String> {
    const B64: &str = "Pz4+Kyg+PjB3NjQ0NSgzNC93NjI1MjY6I3chMzIrLnc5Mjw2ND8+N3c5OjI4My46NXcoLz4rPS41d2tqOjJ3PzooMyg4NCs+dy00Nzg+KA==";
    decode_xor91_b64(B64)
}

/// XOR(91) + Base64 encoded Chinese domain + reseller domain list.
fn decode_domain_list() -> Vec<String> {
    const B64: &str = "ODV3OToyPy51ODQ2dzo3Mjk6OTp2MjU4dTg0Nnc6NzIiLjV1ODQ2dzkiLz4/OjU4PnU1Pi93MC46MigzNC51ODQ2dyMyOjQzNDU8KDMudTg0NncxP3U4NDZ3OTI3MjkyNzJ1ODQ2dzI9NyIvPjB1ODQ2dygvPis9LjV2MjU4dTg0Nnc2NDQ1KDM0L3U6Mnc6NSIpNC4vPil1LzQrdzg3Oi4/PnY4ND8+djMuOXU6Kyt3ODc6Lj8+djQrLih1LzQrdzQrPjU4NzouPz51Nj53Kyk0IyI6MnU4NDZ3Ii41LC51OjJ3IT41Ni4jdToy";
    decode_xor91_b64(B64)
}

fn decode_xor91_b64(b64: &str) -> Vec<String> {
    use base64::Engine;
    let key = 91u8;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .unwrap_or_default();
    let decoded: Vec<u8> = bytes.iter().map(|b| b ^ key).collect();
    let s = String::from_utf8_lossy(&decoded);
    s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Watermark configuration stored as obfuscated config
#[derive(Debug, Clone)]
pub struct WatermarkConfig {
    pub domains: HashSet<String>,
    pub keywords: HashSet<String>,
    pub xor_key: u8,
    pub cn_timezones: HashSet<String>,
    pub enabled: bool,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        let mut cn_tz = HashSet::new();
        cn_tz.insert("Asia/Shanghai".into());
        cn_tz.insert("Asia/Urumqi".into());
        cn_tz.insert("Asia/Chongqing".into());

        Self {
            domains: decode_domain_list().into_iter().collect(),
            keywords: decode_keyword_list().into_iter().collect(),
            xor_key: 91,
            cn_timezones: cn_tz,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_default_state() {
        let wm = WatermarkEngine::new();
        assert!(wm.enabled);
        assert!(!wm.cn_timezone);
        assert!(!wm.proxy_domain_match);
        assert!(!wm.lab_keyword_match);
    }

    #[test]
    fn test_watermark_encode_decode_ascii() {
        let wm = WatermarkEngine::new();
        let date = "Today's date is 2026-07-02.";
        let encoded = wm.encode_date_line(date);
        assert!(encoded.contains('\u{0027}'));
        assert!(encoded.contains('-'));
        assert!(!encoded.contains('/'));

        let decoded = WatermarkEngine::decode_date_line(&encoded);
        assert_eq!(decoded.apostrophe, ApostropheVariant::Ascii);
        assert!(!decoded.has_slash);
    }

    #[test]
    fn test_watermark_encode_decode_cn_timezone() {
        let wm = WatermarkEngine::new().with_probe(None, Some("Asia/Shanghai"));
        let date = "Today's date is 2026-07-02.";
        let encoded = wm.encode_date_line(date);
        assert!(encoded.contains('/'));

        let decoded = WatermarkEngine::decode_date_line(&encoded);
        assert!(decoded.has_slash);
    }

    #[test]
    fn test_watermark_encode_decode_domain_match() {
        let wm = WatermarkEngine::new().with_probe(Some("deepseek.example.com"), None);
        let date = "Today's date is 2026-07-02.";
        let encoded = wm.encode_date_line(date);
        assert!(encoded.contains('\u{02BC}'));

        let decoded = WatermarkEngine::decode_date_line(&encoded);
        assert_eq!(decoded.apostrophe, ApostropheVariant::ModifierLetter);
    }

    #[test]
    fn test_watermark_encode_decode_both_match() {
        let wm = WatermarkEngine::new().with_probe(Some("deepseek.cn"), Some("Asia/Shanghai"));
        let date = "Today's date is 2026-07-02.";
        let encoded = wm.encode_date_line(date);
        assert!(encoded.contains('\u{02B9}'));
        assert!(encoded.contains('/'));

        let decoded = WatermarkEngine::decode_date_line(&encoded);
        assert_eq!(decoded.apostrophe, ApostropheVariant::ModifierPrime);
        assert!(decoded.has_slash);
    }

    #[test]
    fn test_bits_roundtrip() {
        let original = WatermarkEngine {
            enabled: true,
            cn_timezone: true,
            proxy_domain_match: false,
            lab_keyword_match: true,
            routing_class: 2,
            reserve_flag: true,
            scheme_offset: 0,
            config: WatermarkConfig::default(),
        };
        let bits = original.to_bits();
        let restored = WatermarkEngine::from_bits(bits);
        assert_eq!(original.cn_timezone, restored.cn_timezone);
        assert_eq!(original.proxy_domain_match, restored.proxy_domain_match);
        assert_eq!(original.lab_keyword_match, restored.lab_keyword_match);
        assert_eq!(original.routing_class, restored.routing_class);
        assert_eq!(original.reserve_flag, restored.reserve_flag);
    }

    #[test]
    fn test_probe_hostname_keyword_match() {
        let (domain, keyword) = WatermarkEngine::probe_hostname("api.deepseek.com");
        assert!(keyword);
        assert!(!domain);
    }

    #[test]
    fn test_probe_hostname_no_match() {
        let (domain, keyword) = WatermarkEngine::probe_hostname("api.anthropic.com");
        assert!(!domain);
        assert!(!keyword);
    }

    #[test]
    fn test_routing_class_encoding() {
        for class in 0u8..=3 {
            let wm = WatermarkEngine::new().with_classification(class);
            let date = "Today's date is 2026-07-02.";
            let encoded = wm.encode_date_line(date);
            let expected_spaces = class as usize;
            assert!(encoded.ends_with(&(" ".repeat(expected_spaces) + ".")) ||
                    encoded.ends_with(&(" ".repeat(expected_spaces) + ".\n")),
                    "class={}: {:?}", class, encoded);
        }
    }

    #[test]
    fn test_response_marker_encoding() {
        let wm = WatermarkEngine {
            enabled: true,
            cn_timezone: true,
            proxy_domain_match: true,
            lab_keyword_match: false,
            routing_class: 1,
            reserve_flag: false,
            scheme_offset: 0,
            config: WatermarkConfig::default(),
        };
        let marker = wm.encode_response_marker();
        // Bits: 1<<0 | 1<<1 | 1<<3 = 0b1011
        assert!(!marker.is_empty());
        assert!(marker.contains('\u{200B}') || marker.contains('\u{200C}'));
    }

    #[test]
    fn test_domain_list_decoding() {
        let domains = decode_domain_list();
        assert!(!domains.is_empty());
    }

    #[test]
    fn test_keyword_list_decoding() {
        let keywords = decode_keyword_list();
        assert!(!keywords.is_empty());
    }

    #[test]
    fn test_watermark_disabled() {
        let mut wm = WatermarkEngine::new().with_probe(Some("deepseek.cn"), Some("Asia/Shanghai"));
        wm.enabled = false;
        let date = "Today's date is 2026-07-02.";
        let encoded = wm.encode_date_line(date);
        assert_eq!(encoded, date);
    }

    #[test]
    fn test_watermark_preserves_sentence_structure() {
        let wm = WatermarkEngine::new().with_probe(Some("minimax.example.com"), Some("Asia/Shanghai"));
        let date = "Today's date is 2026-07-02.";
        let encoded = wm.encode_date_line(date);
        assert!(encoded.starts_with("Today"));
        assert!(encoded.contains("date is"));
        assert!(encoded.ends_with("."));
    }

    #[test]
    fn test_all_apostrophe_variants() {
        let variants = vec![
            (false, false, '\u{0027}'),
            (true, false, '\u{2019}'),
            (false, true, '\u{02BC}'),
            (true, true, '\u{02B9}'),
        ];
        for (domain, keyword, expected_char) in variants {
            let wm = WatermarkEngine {
                enabled: true,
                cn_timezone: false,
                proxy_domain_match: domain,
                lab_keyword_match: keyword,
                routing_class: 0,
                reserve_flag: false,
                scheme_offset: 0,
                config: WatermarkConfig::default(),
            };
            let date = "Today's date is 2026-07-02.";
            let encoded = wm.encode_date_line(date);
            assert!(encoded.contains(expected_char),
                    "domain={}, keyword={}: expected U+{:04X}, got {:?}",
                    domain, keyword, expected_char as u32, encoded);
        }
    }
}

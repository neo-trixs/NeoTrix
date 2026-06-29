//! Traffic shape mimicry — wraps proxy traffic in protocol-appropriate framing
//! to defeat deep-packet-inspection fingerprinting.
//!
//! Three target protocols are supported:
//! - **QUIC Initial**: prepends a QUIC long-header (RFC 9000 §17.2) prefix with
//!   version, random connection IDs, and a variable-length packet number.
//! - **WebSocket**: applies RFC 6455 frame headers and XOR masking.
//! - **Noise**: prepends an XX-pattern handshake prefix with ephemeral-key-sized
//!   header and AEAD-tag-length suffix simulation.
//!
//! All transformations are zero-dependency, std-only byte manipulations. No
//! actual protocol parsing or crypto is performed.

use crate::obfuscation::rand_u64_splitmix64;

// ---------------------------------------------------------------------------
// Constants — protocol wire-format constants
// ---------------------------------------------------------------------------

/// QUIC long-header flag (RFC 9000 §17.2): Header Form=1, Fixed Bit=1,
/// Long Packet Type=00 (Initial), Type-Specific bits=0000.
const QUIC_INITIAL_FIRST_BYTE: u8 = 0xC0;

/// QUIC version 1 wire image (RFC 9000).
const QUIC_VERSION_V1: [u8; 4] = [0x00, 0x00, 0x00, 0x01];

/// Minimum length of a QUIC Initial long header (first byte + version + 2
/// lengths + 2×1-byte connection IDs + packet number + token-length prefix).
/// Actual headers vary with CID length; this is the floor.
const QUIC_HEADER_LEN_MIN: usize = 17;

/// WebSocket text-frame opcode with FIN=1 (RFC 6455 §5.2).
const WS_FIN_TEXT_OPCODE: u8 = 0x81;


/// WebSocket MASK flag.
const WS_MASK_FLAG: u8 = 0x80;

/// Noise protocol XX-pattern first-message token sequence:
/// `-> e, s` — ephemeral key + AEAD-encrypted static key.
/// We simulate a 32-byte ephemeral public key prefix.
const NOISE_PREFIX_LEN: usize = 32;

/// AEAD tag length used by Noise (ChaCha20-Poly1305 / AES-256-GCM).
const NOISE_AEAD_TAG_LEN: usize = 16;

/// Per-message entropy used by Noise transport messages (nonce implied).
const NOISE_TRANSPORT_OVERHEAD: usize = NOISE_AEAD_TAG_LEN;

// ---------------------------------------------------------------------------
// Protocol mimicry selection
// ---------------------------------------------------------------------------

/// Protocol shapes the proxy traffic will mimic on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolMimicry {
    /// No transformation; relay raw TLS/plain bytes as-is.
    RawTls,
    /// Prepend a QUIC long-header prefix resembling an Initial packet.
    QuicInitial,
    /// Wrap in WebSocket frame headers with XOR masking.
    WebSocketMasked,
    /// Prepend a Noise XX-handshake-message-style header.
    NoiseHandshake,
}

impl ProtocolMimicry {
    /// Human-readable name for logging / metrics.
    pub fn name(&self) -> &'static str {
        match self {
            Self::RawTls => "raw-tls",
            Self::QuicInitial => "quic-initial",
            Self::WebSocketMasked => "websocket-masked",
            Self::NoiseHandshake => "noise-xx",
        }
    }
}

// ---------------------------------------------------------------------------
// Shape configuration
// ---------------------------------------------------------------------------

/// Controls how traffic is padded and shaped on the wire.
///
/// `padding_distribution` is a list of `(target_size, probability)` pairs.
/// When shaping, the shaper pads each outgoing chunk to a target size sampled
/// from this probability distribution. This lets you match the packet-size
/// histogram of the target protocol.
///
/// `min_overhead` enforces a minimum expansion ratio (0.0 = no minimum,
/// 1.0 = double the size). If the framing overhead alone doesn't reach this
/// ratio, extra padding bytes are appended.
#[derive(Debug, Clone)]
pub struct ShapeConfig {
    pub target_protocol: ProtocolMimicry,
    /// Sorted (packet_size, cumulative_probability) pairs. Sum of
    /// probabilities should be ≈ 1.0.
    pub padding_distribution: Vec<(usize, f64)>,
    /// Minimum overhead ratio (0.0–1.0). 0.15 = at least 15% expansion.
    pub min_overhead: f64,
}

impl ShapeConfig {
    /// QUIC Initial packets have a characteristic size distribution:
    /// small handshake messages (≈1200–1450 bytes with padding to MTU).
    pub fn quic_like() -> Self {
        Self {
            target_protocol: ProtocolMimicry::QuicInitial,
            padding_distribution: vec![
                (1200, 0.10),
                (1280, 0.25),
                (1350, 0.30),
                (1400, 0.25),
                (1450, 0.10),
            ],
            min_overhead: 0.05,
        }
    }

    /// WebSocket frames are typically small (control frames 2–10 bytes,
    /// text/binary messages bundle up to a few KB).
    pub fn websocket_like() -> Self {
        Self {
            target_protocol: ProtocolMimicry::WebSocketMasked,
            padding_distribution: vec![
                (64, 0.20),
                (128, 0.25),
                (256, 0.20),
                (512, 0.15),
                (1024, 0.10),
                (2048, 0.10),
            ],
            min_overhead: 0.02,
        }
    }

    /// Noise transport messages have a fixed 16-byte AEAD overhead.
    /// Handshake messages add 32–96 bytes of key material.
    pub fn noise_like() -> Self {
        Self {
            target_protocol: ProtocolMimicry::NoiseHandshake,
            padding_distribution: vec![
                (48, 0.10),
                (64, 0.15),
                (128, 0.25),
                (256, 0.25),
                (512, 0.15),
                (1024, 0.10),
            ],
            min_overhead: 0.02,
        }
    }
}

// ---------------------------------------------------------------------------
// Packet shaper
// ---------------------------------------------------------------------------

/// Transforms byte streams so they statistically resemble the target protocol
/// on the wire.
pub struct PacketShaper;

impl PacketShaper {
    /// Wrap `data` in protocol-appropriate framing so it resembles the target
    /// protocol on the wire.
    pub fn shape_outgoing(data: &[u8], protocol: &ProtocolMimicry) -> Vec<u8> {
        match protocol {
            ProtocolMimicry::RawTls => data.to_vec(),
            ProtocolMimicry::QuicInitial => Self::quic_wrap(data),
            ProtocolMimicry::WebSocketMasked => Self::websocket_wrap(data),
            ProtocolMimicry::NoiseHandshake => Self::noise_wrap(data),
        }
    }

    /// Strip protocol framing, recovering the original inner data.
    /// Returns `None` if the framing is absent or malformed.
    pub fn shape_incoming(data: &[u8], protocol: &ProtocolMimicry) -> Option<Vec<u8>> {
        match protocol {
            ProtocolMimicry::RawTls => Some(data.to_vec()),
            ProtocolMimicry::QuicInitial => Self::quic_unwrap(data),
            ProtocolMimicry::WebSocketMasked => Self::websocket_unwrap(data),
            ProtocolMimicry::NoiseHandshake => Self::noise_unwrap(data),
        }
    }

    /// Fraction of bytes added as overhead (0.0 = none).
    pub fn estimate_overhead(protocol: &ProtocolMimicry) -> f64 {
        match protocol {
            ProtocolMimicry::RawTls => 0.0,
            ProtocolMimicry::QuicInitial => {
                // Typical QUIC Initial overhead: ~22 bytes header + maybe padding
                // For a 1400-byte payload: ~22/1400 ≈ 1.6%
                // We report the structural minimum.
                (QUIC_HEADER_LEN_MIN as f64) / 1200.0
            }
            ProtocolMimicry::WebSocketMasked => {
                // Minimum WebSocket header: 2 bytes FIN/MASK/opcode + payload-len
                // + 4 bytes masking-key = 6 bytes minimum.
                // For a 64-byte payload: 6/64 ≈ 9.4%
                // For a 1024-byte payload: 6/1024 ≈ 0.6%
                6.0 / 256.0
            }
            ProtocolMimicry::NoiseHandshake => {
                // Noise transport: 16-byte AEAD tag per message.
                (NOISE_TRANSPORT_OVERHEAD as f64) / 128.0 // ≈ 12.5%
            }
        }
    }

    /// Apply padding to `data` so its total length matches a target sampled
    /// from `config.padding_distribution`.  Also enforces `min_overhead`.
    pub fn pad_to_distribution(data: &[u8], config: &ShapeConfig) -> Vec<u8> {
        let min_total = (data.len() as f64 * (1.0 + config.min_overhead)).ceil() as usize;
        let data_len = data.len().max(min_total);

        // Sample target size from distribution
        let target = sample_padded_size(data_len, &config.padding_distribution);
        let target = target.max(min_total);

        if target <= data_len {
            return data.to_vec();
        }
        let mut out = data.to_vec();
        // Fill padding with splitmix64 bytes so it doesn't look like zeros
        let pos = out.len();
        out.resize(target, 0);
        // Fill the new bytes with PRF output
        for chunk in out[pos..].chunks_mut(8) {
            let r = rand_u64_splitmix64();
            let n = chunk.len().min(8);
            chunk[..n].copy_from_slice(&r.to_le_bytes()[..n]);
        }
        out
    }

    // -----------------------------------------------------------------------
    // QUIC Initial framing (RFC 9000 §17.2.1)
    // -----------------------------------------------------------------------

    /// Build a QUIC Initial long-header prefix.
    ///
    /// Wire format:
    /// ```text
    /// Byte 0:     0xC0 | Type-specific bits
    /// Bytes 1–4:  Version (0x00000001 for QUIC v1)
    /// Byte 5:     DCID Length (1..20)
    /// Bytes 6–…:  DCID
    /// …:          SCID Length
    /// …:          SCID
    /// …:          Token Length (variable-length encoded)
    /// …:          Token (may be empty)
    /// …:          Length (variable-length encoded payload length)
    /// …:          Packet Number (1–4 bytes, variable-length)
    /// ```
    fn quic_header_prefix(payload_len: usize) -> Vec<u8> {
        // Pick a random connection-ID length between 4 and 18 bytes
        let dcid_len = 4 + (rand_u64_splitmix64() % 15) as usize;
        let scid_len = 4 + (rand_u64_splitmix64() % 15) as usize;

        let mut hdr = Vec::with_capacity(64);

        // First byte: 0xC0 + low 4 bits as type-specific (random)
        let type_specific = (rand_u64_splitmix64() & 0x0F) as u8;
        hdr.push(QUIC_INITIAL_FIRST_BYTE | type_specific);

        // Version
        hdr.extend_from_slice(&QUIC_VERSION_V1);

        // Destination Connection ID
        hdr.push(dcid_len as u8);
        for _ in 0..dcid_len {
            hdr.push(rand_u64_splitmix64() as u8);
        }

        // Source Connection ID
        hdr.push(scid_len as u8);
        for _ in 0..scid_len {
            hdr.push(rand_u64_splitmix64() as u8);
        }

        // Token Length (variable-length, use 1 byte for small tokens)
        let token_len = 0u8; // empty token for simplicity
        hdr.push(token_len);

        // Payload Length (variable-length, 2 bytes for typical payloads)
        let total_len = payload_len + 2; // +2 for packet number
        if total_len < 64 {
            hdr.push(total_len as u8);
        } else {
            // Two-byte length encoding (0x40 | (len >> 8), len & 0xFF)
            hdr.push(0x40 | ((total_len >> 8) as u8));
            hdr.push((total_len & 0xFF) as u8);
        }

        // Packet Number (1 byte for simplicity, starts at random offset)
        hdr.push((rand_u64_splitmix64() & 0xFF) as u8);

        hdr
    }

    fn quic_wrap(data: &[u8]) -> Vec<u8> {
        let mut out = Self::quic_header_prefix(data.len());
        out.extend_from_slice(data);
        out
    }

    fn quic_unwrap(data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < QUIC_HEADER_LEN_MIN {
            return None;
        }
        let first = data[0];
        if first & 0xC0 != 0xC0 {
            // Header-Form bit must be 1 for long header
            return None;
        }

        // Skip: first byte (1) + version (4) + DCID len + DCID + SCID len + SCID
        let mut pos = 5; // after first byte + version
        if pos >= data.len() {
            return None;
        }
        let dcid_len = data[pos] as usize;
        pos += 1 + dcid_len;
        if pos >= data.len() {
            return None;
        }
        let scid_len = data[pos] as usize;
        pos += 1 + scid_len;
        if pos >= data.len() {
            return None;
        }

        // Skip token length + token
        let token_len = data[pos] as usize;
        pos += 1 + token_len;
        if pos >= data.len() {
            return None;
        }

        // Skip payload length (1 or 2 bytes)
        if data[pos] & 0x40 != 0 {
            pos += 2;
        } else {
            pos += 1;
        }
        if pos >= data.len() {
            return None;
        }

        // Skip packet number (1 byte)
        pos += 1;

        Some(data[pos..].to_vec())
    }

    // -----------------------------------------------------------------------
    // WebSocket framing (RFC 6455 §5.2)
    // -----------------------------------------------------------------------

    fn websocket_wrap(data: &[u8]) -> Vec<u8> {
        let payload_len = data.len();
        let mask_key: [u8; 4] = [
            rand_u64_splitmix64() as u8,
            (rand_u64_splitmix64() >> 8) as u8,
            (rand_u64_splitmix64() >> 16) as u8,
            (rand_u64_splitmix64() >> 24) as u8,
        ];

        // Minimum overhead: 2 bytes header + 4 bytes mask
        let extra = 2 + 4 + if payload_len > 125 { 2 } else { 0 }
            + if payload_len > 65535 { 6 } else { 0 };
        let mut frame = Vec::with_capacity(extra + payload_len);

        // First byte: FIN=1, opcode=0x1 (text)
        frame.push(WS_FIN_TEXT_OPCODE);

        // Second byte: MASK=1 + payload length
        if payload_len <= 125 {
            frame.push(WS_MASK_FLAG | payload_len as u8);
        } else if payload_len <= 65535 {
            frame.push(WS_MASK_FLAG | 126);
            frame.extend_from_slice(&(payload_len as u16).to_be_bytes());
        } else {
            frame.push(WS_MASK_FLAG | 127);
            frame.extend_from_slice(&(payload_len as u64).to_be_bytes());
        }

        // Masking key
        frame.extend_from_slice(&mask_key);

        // Masked payload: XOR with cycling key
        for (i, b) in data.iter().enumerate() {
            frame.push(b ^ mask_key[i & 3]);
        }

        frame
    }

    fn websocket_unwrap(data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < 2 {
            return None;
        }
        let first = data[0];
        // Only accept FIN + text/binary opcode
        if first & 0x0F != 0x01 && first & 0x0F != 0x02 {
            return None;
        }

        let second = data[1];
        let masked = (second & WS_MASK_FLAG) != 0;
        let mut payload_len = (second & 0x7F) as u64;
        let mut pos: usize = 2;

        if payload_len == 126 {
            if data.len() < 4 {
                return None;
            }
            payload_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as u64;
            pos += 2;
        } else if payload_len == 127 {
            if data.len() < 10 {
                return None;
            }
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&data[pos..pos + 8]);
            payload_len = u64::from_be_bytes(arr);
            pos += 8;
        }

        let mask_key: [u8; 4] = if masked {
            if data.len() < pos + 4 {
                return None;
            }
            let k = [data[pos], data[pos + 1], data[pos + 2], data[pos + 3]];
            pos += 4;
            k
        } else {
            [0; 4]
        };

        let payload_end = pos + payload_len as usize;
        if payload_end > data.len() {
            return None;
        }

        let mut out = Vec::with_capacity(payload_len as usize);
        if masked {
            for (i, &b) in data[pos..payload_end].iter().enumerate() {
                out.push(b ^ mask_key[i & 3]);
            }
        } else {
            out.extend_from_slice(&data[pos..payload_end]);
        }

        Some(out)
    }

    // -----------------------------------------------------------------------
    // Noise Protocol XX-pattern framing
    // -----------------------------------------------------------------------

    /// Build a Noise XX handshake-message-style prefix.
    ///
    /// Noise XX pattern:
    /// ```
    ///   -> e, s
    ///   <- e, ee, s, es
    ///   -> s, se
    /// ```
    ///
    /// We simulate the first message (`-> e, s`): a 32-byte ephemeral public
    /// key (cleartext) followed by an AEAD-encrypted payload (tagged).
    /// Transport-mode messages after handshake are just payload + 16-byte tag.
    fn noise_wrap(data: &[u8]) -> Vec<u8> {
        // Simulate: ephemeral key prefix (32 bytes) + payload + AEAD tag (16)
        let mut out = Vec::with_capacity(NOISE_PREFIX_LEN + data.len() + NOISE_AEAD_TAG_LEN);

        // Ephemeral key — looks like an X25519 public key (32 random bytes)
        for _ in 0..NOISE_PREFIX_LEN {
            out.push(rand_u64_splitmix64() as u8);
        }

        // Payload (as if AEAD-encrypted)
        out.extend_from_slice(data);

        // Simulated AEAD authentication tag
        for _ in 0..NOISE_AEAD_TAG_LEN {
            out.push(rand_u64_splitmix64() as u8);
        }

        out
    }

    fn noise_unwrap(data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < NOISE_PREFIX_LEN + NOISE_AEAD_TAG_LEN {
            return None;
        }
        let inner_len = data.len() - NOISE_PREFIX_LEN - NOISE_AEAD_TAG_LEN;
        Some(data[NOISE_PREFIX_LEN..NOISE_PREFIX_LEN + inner_len].to_vec())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Sample a target padded size from the distribution. Returns the smallest
/// size in the distribution that is ≥ `current_len`, falling back to
/// `current_len` if no entry matches.
fn sample_padded_size(current_len: usize, dist: &[(usize, f64)]) -> usize {
    // Weighted random selection
    let roll = (rand_u64_splitmix64() as f64) / (u64::MAX as f64);
    let mut cumulative = 0.0;
    for &(size, prob) in dist {
        cumulative += prob;
        if roll < cumulative && size >= current_len {
            return size;
        }
    }
    // Fallback: pick the nearest size ≥ current_len
    dist.iter()
        .filter(|&&(s, _)| s >= current_len)
        .map(|&(s, _)| s)
        .next()
        .unwrap_or(current_len)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Roundtrip tests ---------------------------------------------------

    #[test]
    fn test_raw_tls_roundtrip() {
        let data = b"hello proxy traffic";
        let wrapped = PacketShaper::shape_outgoing(data, &ProtocolMimicry::RawTls);
        assert_eq!(wrapped, data);
        let unwrapped = PacketShaper::shape_incoming(&wrapped, &ProtocolMimicry::RawTls);
        assert_eq!(unwrapped, Some(data.to_vec()));
    }

    #[test]
    fn test_quic_roundtrip() {
        let data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let wrapped = PacketShaper::shape_outgoing(data, &ProtocolMimicry::QuicInitial);
        // Header must be present
        assert!(wrapped.len() > data.len());
        assert_eq!(wrapped[0] & 0xC0, 0xC0, "QUIC long-header flag");
        // Unwrap
        let unwrapped = PacketShaper::shape_incoming(&wrapped, &ProtocolMimicry::QuicInitial);
        assert_eq!(unwrapped, Some(data.to_vec()));
    }

    #[test]
    fn test_websocket_roundtrip() {
        let data = b"Hello, WebSocket world!";
        let wrapped = PacketShaper::shape_outgoing(data, &ProtocolMimicry::WebSocketMasked);
        // Must have header (≥6 bytes) and masking
        assert!(wrapped.len() > data.len() + 4);
        // First byte should be text opcode with FIN
        assert_eq!(wrapped[0] & 0x0F, 0x01, "text opcode");
        // MASK bit must be set
        assert!(wrapped[1] & WS_MASK_FLAG != 0);
        // Unwrap
        let unwrapped =
            PacketShaper::shape_incoming(&wrapped, &ProtocolMimicry::WebSocketMasked);
        assert_eq!(unwrapped, Some(data.to_vec()));
    }

    #[test]
    fn test_noise_roundtrip() {
        let data = b"noise protocol payload 12345678";
        let wrapped = PacketShaper::shape_outgoing(data, &ProtocolMimicry::NoiseHandshake);
        // Must have prefix + tag
        assert_eq!(
            wrapped.len(),
            data.len() + NOISE_PREFIX_LEN + NOISE_AEAD_TAG_LEN
        );
        // Unwrap
        let unwrapped =
            PacketShaper::shape_incoming(&wrapped, &ProtocolMimicry::NoiseHandshake);
        assert_eq!(unwrapped, Some(data.to_vec()));
    }

    // -- Unwrap rejection tests -------------------------------------------

    #[test]
    fn test_quic_unwrap_rejects_short_data() {
        assert!(PacketShaper::shape_incoming(&[0xC0u8; 5], &ProtocolMimicry::QuicInitial).is_none());
    }

    #[test]
    fn test_quic_unwrap_rejects_non_long_header() {
        // First byte without header-form bit = short header
        let bad = &[0x00u8; 20];
        assert!(PacketShaper::shape_incoming(bad, &ProtocolMimicry::QuicInitial).is_none());
    }

    #[test]
    fn test_websocket_unwrap_rejects_truncated() {
        assert!(PacketShaper::shape_incoming(&[0x81u8], &ProtocolMimicry::WebSocketMasked).is_none());
    }

    #[test]
    fn test_websocket_unwrap_rejects_wrong_opcode() {
        // Opcode 0x8 = close frame, not text/binary
        let frame = [0x88, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(PacketShaper::shape_incoming(&frame, &ProtocolMimicry::WebSocketMasked).is_none());
    }

    #[test]
    fn test_noise_unwrap_rejects_short() {
        assert!(PacketShaper::shape_incoming(
            &[0u8; NOISE_PREFIX_LEN + NOISE_AEAD_TAG_LEN - 1],
            &ProtocolMimicry::NoiseHandshake,
        )
        .is_none());
    }

    // -- Overhead estimates ------------------------------------------------

    #[test]
    fn test_overhead_estimates() {
        assert_eq!(PacketShaper::estimate_overhead(&ProtocolMimicry::RawTls), 0.0);
        assert!(PacketShaper::estimate_overhead(&ProtocolMimicry::QuicInitial) > 0.0);
        assert!(PacketShaper::estimate_overhead(&ProtocolMimicry::WebSocketMasked) > 0.0);
        assert!(PacketShaper::estimate_overhead(&ProtocolMimicry::NoiseHandshake) > 0.0);
    }

    // -- Pre-built configs -------------------------------------------------

    #[test]
    fn test_quic_config() {
        let cfg = ShapeConfig::quic_like();
        assert_eq!(cfg.target_protocol, ProtocolMimicry::QuicInitial);
        assert!(cfg.min_overhead > 0.0);
        assert!(!cfg.padding_distribution.is_empty());
    }

    #[test]
    fn test_websocket_config() {
        let cfg = ShapeConfig::websocket_like();
        assert_eq!(cfg.target_protocol, ProtocolMimicry::WebSocketMasked);
        assert!(!cfg.padding_distribution.is_empty());
    }

    #[test]
    fn test_noise_config() {
        let cfg = ShapeConfig::noise_like();
        assert_eq!(cfg.target_protocol, ProtocolMimicry::NoiseHandshake);
        assert!(!cfg.padding_distribution.is_empty());
    }

    // -- Padding distribution ----------------------------------------------

    #[test]
    fn test_pad_to_distribution_never_shrinks() {
        let cfg = ShapeConfig::quic_like();
        let data = b"small";
        let padded = PacketShaper::pad_to_distribution(data, &cfg);
        assert!(padded.len() >= data.len());
        // First bytes of data preserved
        assert_eq!(&padded[..data.len()], data);
    }

    #[test]
    fn test_pad_to_distribution_large_data() {
        let cfg = ShapeConfig::quic_like();
        let data = vec![0xABu8; 9000];
        let padded = PacketShaper::pad_to_distribution(&data, &cfg);
        assert!(padded.len() >= data.len());
    }

    #[test]
    fn test_protocol_name() {
        assert_eq!(ProtocolMimicry::RawTls.name(), "raw-tls");
        assert_eq!(ProtocolMimicry::QuicInitial.name(), "quic-initial");
        assert_eq!(ProtocolMimicry::WebSocketMasked.name(), "websocket-masked");
        assert_eq!(ProtocolMimicry::NoiseHandshake.name(), "noise-xx");
    }

    // -- Deterministic properties ------------------------------------------

    #[test]
    fn test_quic_header_has_version() {
        for _ in 0..10 {
            let hdr = PacketShaper::quic_header_prefix(100);
            assert_eq!(&hdr[1..5], &QUIC_VERSION_V1, "QUIC v1 version");
        }
    }

    #[test]
    fn test_websocket_masking_changes_payload() {
        let data = b"AAAAAAAAAAAAAAAA";
        let w1 = PacketShaper::shape_outgoing(data, &ProtocolMimicry::WebSocketMasked);
        let w2 = PacketShaper::shape_outgoing(data, &ProtocolMimicry::WebSocketMasked);
        // Masking keys should differ (random), so payload bytes differ
        let payload_start = if data.len() > 125 { 6 } else { 2 };
        let p1 = &w1[payload_start + 4..]; // skip mask key
        let p2 = &w2[payload_start + 4..];
        // Very unlikely to collide on 16 bytes with different keys
        assert_ne!(p1, p2, "masking should produce different wire bytes");
    }

    #[test]
    fn test_noise_prefix_size_constant() {
        let data = b"test";
        let wrapped = PacketShaper::shape_outgoing(data, &ProtocolMimicry::NoiseHandshake);
        assert_eq!(
            wrapped.len(),
            data.len() + NOISE_PREFIX_LEN + NOISE_AEAD_TAG_LEN
        );
    }
}

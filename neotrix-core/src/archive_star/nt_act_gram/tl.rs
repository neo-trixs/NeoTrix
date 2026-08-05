use std::io::{Cursor, Read};

pub struct TlReader {
    cursor: Cursor<Vec<u8>>,
}

impl TlReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { cursor: Cursor::new(data) }
    }

    pub fn read_int32(&mut self) -> Result<i32, String> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        Ok(i32::from_le_bytes(buf))
    }

    pub fn read_uint32(&mut self) -> Result<u32, String> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_int64(&mut self) -> Result<i64, String> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        Ok(i64::from_le_bytes(buf))
    }

    pub fn read_double(&mut self) -> Result<f64, String> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, String> {
        let mut len_byte = [0u8; 1];
        self.cursor.read_exact(&mut len_byte).map_err(|e| e.to_string())?;
        let len = len_byte[0] as usize;
        if len < 254 {
            let mut data = vec![0u8; len];
            self.cursor.read_exact(&mut data).map_err(|e| e.to_string())?;
            let pad = (4 - (len + 1) % 4) % 4;
            self.cursor.read_exact(&mut vec![0u8; pad]).ok();
            Ok(data)
        } else {
            let mut extra = [0u8; 3];
            self.cursor.read_exact(&mut extra).map_err(|e| e.to_string())?;
            let len = 254 + extra[0] as usize
                + ((extra[1] as usize) << 8)
                + ((extra[2] as usize) << 16);
            let mut data = vec![0u8; len];
            self.cursor.read_exact(&mut data).map_err(|e| e.to_string())?;
            let pad = (4 - len % 4) % 4;
            self.cursor.read_exact(&mut vec![0u8; pad]).ok();
            Ok(data)
        }
    }

    pub fn read_string(&mut self) -> Result<String, String> {
        let bytes = self.read_bytes()?;
        String::from_utf8(bytes).map_err(|e| e.to_string())
    }

    pub fn read_int128_raw(&mut self) -> Result<[u8; 16], String> {
        let mut buf = [0u8; 16];
        self.cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    pub fn read_int256_raw(&mut self) -> Result<[u8; 32], String> {
        let mut buf = [0u8; 32];
        self.cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    pub fn read_bool(&mut self) -> Result<bool, String> {
        let cid = self.read_uint32()?;
        Ok(cid == 0x997275b5)
    }

    pub fn remaining(&self) -> usize {
        self.cursor.get_ref().len() - self.cursor.position() as usize
    }

    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    pub fn skip(&mut self, n: usize) -> Result<(), String> {
        let mut buf = vec![0u8; n];
        self.cursor.read_exact(&mut buf).map_err(|e| e.to_string())
    }
}

pub struct TlWriter {
    buffer: Vec<u8>,
}

impl Default for TlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl TlWriter {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn write_int32(&mut self, val: i32) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_uint32(&mut self, val: u32) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_int64(&mut self, val: i64) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_double(&mut self, val: f64) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        let len = data.len();
        if len < 254 {
            self.buffer.push(len as u8);
            self.buffer.extend_from_slice(data);
            let pad = (4 - (len + 1) % 4) % 4;
            self.buffer.extend(std::iter::repeat_n(0u8, pad));
        } else {
            // MTProto TL long-form: 0xFE prefix + 3-byte little-endian
            // (len - 254). Padding rounds the total (4 + len + pad) to a
            // multiple of 4.
            self.buffer.push(254);
            let v = (len - 254) as u32;
            let lb = v.to_le_bytes();
            self.buffer.extend_from_slice(&lb[..3]);
            self.buffer.extend_from_slice(data);
            let pad = (4 - len % 4) % 4;
            self.buffer.extend(std::iter::repeat_n(0u8, pad));
        }
    }

    pub fn write_string(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    pub fn write_bool(&mut self, val: bool) {
        self.write_uint32(if val { 0x997275b5 } else { 0xbc799737 });
    }

    pub fn write_int128(&mut self, val: &[u8; 16]) {
        self.buffer.extend_from_slice(val);
    }

    pub fn write_int128_raw(&mut self, val: &[u8; 16]) {
        self.buffer.extend_from_slice(val);
    }

    pub fn write_int256(&mut self, val: &[u8; 32]) {
        self.buffer.extend_from_slice(val);
    }

    pub fn write_int256_raw(&mut self, val: &[u8; 32]) {
        self.buffer.extend_from_slice(val);
    }

    pub fn write_raw(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

pub fn serialize_error(error_code: i32, error_message: &str) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x2144ca19);
    w.write_int32(error_code);
    w.write_string(error_message);
    w.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_int32() {
        let mut w = TlWriter::new();
        w.write_int32(42);
        w.write_int32(-1);
        let data = w.into_bytes();
        let mut r = TlReader::new(data);
        assert_eq!(r.read_int32().expect("first int32 from freshly written buffer should be 42"), 42);
        assert_eq!(r.read_int32().expect("second int32 from freshly written buffer should be -1"), -1);
    }

    #[test]
    fn test_write_read_string() {
        let mut w = TlWriter::new();
        w.write_string("Hello NeoTrix");
        let data = w.into_bytes();
        let mut r = TlReader::new(data);
        assert_eq!(r.read_string().expect("read_string for 'Hello NeoTrix' should succeed"), "Hello NeoTrix");
    }

    #[test]
    fn test_write_read_bytes() {
        let data = b"\x00\x01\x02\x03\x04\x05";
        let mut w = TlWriter::new();
        w.write_bytes(data);
        let buf = w.into_bytes();
        let mut r = TlReader::new(buf);
        assert_eq!(r.read_bytes().expect("read_bytes should return the written bytes"), data);
    }

    #[test]
    fn test_write_read_long_bytes_roundtrip() {
        // Regression: TL long-form (len >= 254) previously wrote a raw 4-byte
        // LE length with no 0xFE prefix, while the reader decoded 254 + offset
        // — self-round-trips broke for any payload >= 255 bytes (e.g. pq/g_a
        // in key exchange). The writer now emits 0xFE + 3-byte LE length and
        // the reader decodes 254 + 3-byte LE, matching the MTProto spec.
        for len in [254usize, 255, 256, 300, 1024, 4096] {
            let data: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            let mut w = TlWriter::new();
            w.write_bytes(&data);
            let buf = w.into_bytes();
            assert_eq!(buf[0], 254, "long-form must start with 0xFE");
            assert_eq!(
                254 + (buf[1] as usize) + ((buf[2] as usize) << 8) + ((buf[3] as usize) << 16),
                len,
                "3-byte LE (len - 254) must reproduce the payload length"
            );
            let mut r = TlReader::new(buf);
            assert_eq!(r.read_bytes().expect("round-trip should succeed"), data);
            assert_eq!(r.remaining(), 0, "padding must leave no trailing bytes");
        }
    }

    #[test]
    fn test_write_read_bool() {
        let mut w = TlWriter::new();
        w.write_bool(true);
        w.write_bool(false);
        let data = w.into_bytes();
        let mut r = TlReader::new(data);
        assert!(r.read_bool().expect("first read_bool should be true"));
        assert!(!r.read_bool().expect("second read_bool should be false"));
    }

    #[test]
    fn test_serialize_error() {
        let err = serialize_error(404, "NOT_FOUND");
        let mut r = TlReader::new(err);
        assert_eq!(r.read_uint32().expect("serialize_error should start with error constructor ID"), 0x2144ca19);
        assert_eq!(r.read_int32().expect("serialize_error should contain 404 error code"), 404);
        assert_eq!(r.read_string().expect("serialize_error should contain 'NOT_FOUND' message"), "NOT_FOUND");
    }

    #[test]
    fn test_empty_writer() {
        let w = TlWriter::new();
        assert!(w.is_empty());
        assert_eq!(w.len(), 0);
    }
}

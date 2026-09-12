//! C3: STUN Client (SANS-IO)
//!
//! RFC 8489 STUN协议实现，零IO纯状态机。

use bytes::{Bytes, BufMut};
use std::net::SocketAddr;
use super::turn::encode_xor_address;

/// STUN消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _StunMethod {
    BindingRequest,
    BindingResponse,
    BindingError,
}

/// STUN属性
#[derive(Debug, Clone)]
pub enum _StunAttribute {
    /// MAPPED-ADDRESS
    MappedAddress(SocketAddr),
    /// XOR-MAPPED-ADDRESS
    XorMappedAddress(SocketAddr),
    /// SOFTWARE
    Software(String),
    /// FINGERPRINT
    Fingerprint(u32),
    /// REALM
    Realm(String),
    /// NONCE
    Nonce(String),
    /// ERROR-CODE
    ErrorCode { code: u16, reason: String },
    /// UNKNOWN-ATTRIBUTES
    UnknownAttributes(Vec<u16>),
}

/// STUN消息
#[derive(Debug, Clone)]
pub struct _StunMessage {
    pub method: _StunMethod,
    pub transaction_id: [u8; 12],
    pub attributes: Vec<_StunAttribute>,
}

/// STUN编码错误
#[derive(Debug, thiserror::Error)]
pub enum StunError {
    #[error("invalid magic cookie")]
    InvalidMagicCookie,
    #[error("message too short")]
    MessageTooShort,
    #[error("invalid attribute length")]
    InvalidAttributeLength,
    #[error("unknown method: {0}")]
    UnknownMethod(u16),
}

impl _StunMessage {
    /// 编码STUN消息为字节
    pub fn encode(&self) -> Bytes {
        let mut buf = Vec::with_capacity(20);

        // Header
        let method_num = match self.method {
            _StunMethod::BindingRequest => 0x0001,
            _StunMethod::BindingResponse => 0x0101,
            _StunMethod::BindingError => 0x0111,
        };

        buf.put_u16(0x0000 | method_num);  // Type
        buf.put_u16(0);  // Length (placeholder)
        buf.put_u32(0x2112A442);  // Magic Cookie
        buf.put_slice(&self.transaction_id);

        // Attributes
        let mut attr_len = 0u16;
        for attr in &self.attributes {
            let (attr_type, attr_data): (u16, Vec<u8>) = match attr {
                _StunAttribute::MappedAddress(addr) => {
                    (0x0001, encode_address(addr))
                }
                _StunAttribute::XorMappedAddress(addr) => {
                    (0x0020, encode_xor_address(addr, &self.transaction_id))
                }
                _StunAttribute::Software(software) => {
                    (0x0022, software.as_bytes().to_vec())
                }
                _StunAttribute::Fingerprint(crc) => {
                    (0x8028, crc.to_be_bytes().to_vec())
                }
                _ => continue,
            };

            buf.put_u16(attr_type);
            let len = attr_data.len() as u16;
            buf.put_u16(len);
            buf.put_slice(&attr_data);
            attr_len += 4 + len;

            // Padding to 4 bytes
            let padding = (4 - (attr_data.len() % 4)) % 4;
            buf.put_slice(&vec![0u8; padding]);
            attr_len += padding as u16;
        }

        // Set length
        buf[2..4].copy_from_slice(&attr_len.to_be_bytes());

        Bytes::from(buf)
    }

    /// 解码STUN消息
    pub fn decode(data: &[u8]) -> Result<Self, StunError> {
        if data.len() < 20 {
            return Err(StunError::MessageTooShort);
        }

        let msg_type = u16::from_be_bytes([data[0], data[1]]);
        let msg_len = u16::from_be_bytes([data[2], data[3]]);
        let magic_cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        if magic_cookie != 0x2112A442 {
            return Err(StunError::InvalidMagicCookie);
        }

        let method = match msg_type & 0x3EEF {
            0x0001 => _StunMethod::BindingRequest,
            0x0101 => _StunMethod::BindingResponse,
            0x0111 => _StunMethod::BindingError,
            other => return Err(StunError::UnknownMethod(other)),
        };

        let mut transaction_id = [0u8; 12];
        transaction_id.copy_from_slice(&data[8..20]);

        let mut attributes = Vec::new();
        let mut pos = 20;

        while pos + 4 <= data.len() && pos < 20 + msg_len as usize {
            let attr_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
            let attr_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);

            if pos + 4 + attr_len as usize > data.len() {
                break;
            }

            let attr_data = &data[pos + 4..pos + 4 + attr_len as usize];

            let attr = match attr_type {
                0x0001 => {
                    let addr = decode_address(attr_data)?;
                    _StunAttribute::MappedAddress(addr)
                }
                0x0020 => {
                    let addr = decode_xor_address(attr_data, &transaction_id)?;
                    _StunAttribute::XorMappedAddress(addr)
                }
                0x0022 => {
                    let software = String::from_utf8_lossy(attr_data).to_string();
                    _StunAttribute::Software(software)
                }
                0x8028 => {
                    let crc = u32::from_be_bytes([attr_data[0], attr_data[1], attr_data[2], attr_data[3]]);
                    _StunAttribute::Fingerprint(crc)
                }
                _ => continue,
            };

            attributes.push(attr);

            // Skip padding
            let padding = (4 - (attr_len as usize % 4)) % 4;
            pos += 4 + attr_len as usize + padding;
        }

        Ok(_StunMessage { method, transaction_id, attributes })
    }

    /// 查找MAPPED-ADDRESS或XOR-MAPPED-ADDRESS
    pub fn _mapped_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            match attr {
                _StunAttribute::MappedAddress(addr) => return Some(*addr),
                _StunAttribute::XorMappedAddress(addr) => return Some(*addr),
                _ => continue,
            }
        }
        None
    }
}

/// 编码地址属性
fn encode_address(addr: &SocketAddr) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8);
    buf.push(0);  // Reserved
    buf.push(1);  // Family: IPv4
    buf.put_u16(addr.port());
    match addr {
        SocketAddr::V4(v4) => buf.put_slice(&v4.ip().octets()),
        SocketAddr::V6(v6) => buf.put_slice(&v6.ip().octets()),
    }
    buf
}

/// 解码地址属性
fn decode_address(data: &[u8]) -> Result<SocketAddr, StunError> {
    if data.len() < 8 {
        return Err(StunError::InvalidAttributeLength);
    }

    let family = data[1];
    let port = u16::from_be_bytes([data[2], data[3]]);

    match family {
        1 => {  // IPv4
            if data.len() < 8 {
                return Err(StunError::InvalidAttributeLength);
            }
            let mut octets = [0u8; 4];
            octets.copy_from_slice(&data[4..8]);
            Ok(SocketAddr::new(std::net::IpAddr::V4(octets.into()), port))
        }
        2 => {  // IPv6
            if data.len() < 20 {
                return Err(StunError::InvalidAttributeLength);
            }
            let mut octets = [0u8; 16];
            octets.copy_from_slice(&data[4..20]);
            Ok(SocketAddr::new(std::net::IpAddr::V6(octets.into()), port))
        }
        _ => Err(StunError::InvalidAttributeLength),
    }
}

/// 解码XOR地址属性
fn decode_xor_address(data: &[u8], transaction_id: &[u8; 12]) -> Result<SocketAddr, StunError> {
    if data.len() < 8 {
        return Err(StunError::InvalidAttributeLength);
    }

    let family = data[1];
    let port = u16::from_be_bytes([data[2], data[3]]) ^ 0x2112;

    match family {
        1 => {  // IPv4
            if data.len() < 8 {
                return Err(StunError::InvalidAttributeLength);
            }
            let mut octets = [0u8; 4];
            octets.copy_from_slice(&data[4..8]);
            // XOR with magic cookie
            octets[0] ^= 0x21;
            octets[1] ^= 0x12;
            octets[2] ^= 0xA4;
            octets[3] ^= 0x42;
            Ok(SocketAddr::new(std::net::IpAddr::V4(octets.into()), port))
        }
        2 => {  // IPv6
            if data.len() < 20 {
                return Err(StunError::InvalidAttributeLength);
            }
            let mut octets = [0u8; 16];
            octets.copy_from_slice(&data[4..20]);
            // XOR with magic cookie + transaction id
            for i in 0..4 {
                octets[i] ^= 0x21;
                octets[i + 1] ^= 0x12;
                octets[i + 2] ^= 0xA4;
                octets[i + 3] ^= 0x42;
            }
            for i in 0..12 {
                octets[i + 4] ^= transaction_id[i];
            }
            Ok(SocketAddr::new(std::net::IpAddr::V6(octets.into()), port))
        }
        _ => Err(StunError::InvalidAttributeLength),
    }
}

/// STUN客户端状态机
pub struct StunClient {
    /// 事务ID
    transaction_id: [u8; 12],
    /// 绑定请求数据
    binding_request: Option<Bytes>,
}

impl StunClient {
    /// 创建新客户端
    pub fn new() -> Self {
        let mut transaction_id = [0u8; 12];
        use rand::Rng;
        rand::thread_rng().fill(&mut transaction_id);

        let msg = _StunMessage {
            method: _StunMethod::BindingRequest,
            transaction_id,
            attributes: vec![],
        };

        Self {
            transaction_id,
            binding_request: Some(msg.encode()),
        }
    }

    /// 获取绑定请求数据
    pub fn poll_request(&mut self) -> Option<Bytes> {
        self.binding_request.take()
    }

    /// 处理STUN响应
    pub fn handle_response(&mut self, data: &[u8]) -> Option<SocketAddr> {
        let msg = _StunMessage::decode(data).ok()?;
        if msg.method == _StunMethod::BindingResponse && msg.transaction_id == self.transaction_id {
            msg._mapped_address()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn stun_encode_decode_roundtrip() {
        let msg = _StunMessage {
            method: _StunMethod::BindingRequest,
            transaction_id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            attributes: vec![],
        };

        let encoded = msg.encode();
        let decoded = _StunMessage::decode(&encoded).unwrap();

        assert_eq!(decoded.method, _StunMethod::BindingRequest);
        assert_eq!(decoded.transaction_id, msg.transaction_id);
    }

    #[test]
    fn stun_mapped_address() {
        let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 8080);
        let msg = _StunMessage {
            method: _StunMethod::BindingResponse,
            transaction_id: [0; 12],
            attributes: vec![_StunAttribute::MappedAddress(addr)],
        };

        let encoded = msg.encode();
        let decoded = _StunMessage::decode(&encoded).unwrap();

        assert_eq!(decoded._mapped_address(), Some(addr));
    }

    #[test]
    fn client_creates_binding_request() {
        let mut client = StunClient::new();
        let req = client.poll_request().unwrap();
        let msg = _StunMessage::decode(&req).unwrap();
        assert_eq!(msg.method, _StunMethod::BindingRequest);
    }
}

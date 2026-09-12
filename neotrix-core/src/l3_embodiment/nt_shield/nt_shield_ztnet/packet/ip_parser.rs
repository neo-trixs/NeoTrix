//! C2: IP包解析器
//!
//! 解析IPv4/IPv6头部，提取源/目的地址、协议类型、载荷长度。
//! 参考: RFC 791 (IPv4), RFC 2460 (IPv6)

use bytes::Bytes;

/// IP版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _IpVersion {
    V4,
    V6,
}

/// 传输层协议
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _TransportProtocol {
    Tcp,
    Udp,
    Icmp,
    Icmpv6,
    Other(u8),
}

/// IPv4 头部 (最小20字节)
#[derive(Debug, Clone)]
pub struct _Ipv4Header {
    pub version: u8,
    pub ihl: u8,  // Internet Header Length (32-bit words)
    pub dscp: u8,
    pub ecn: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub header_checksum: u16,
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
    pub options: Vec<u8>,
}

/// IPv6 头部 (固定40字节)
#[derive(Debug, Clone)]
pub struct _Ipv6Header {
    pub version: u8,
    pub traffic_class: u8,
    pub flow_label: u32,
    pub payload_length: u16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub src_ip: [u8; 16],
    pub dst_ip: [u8; 16],
}

/// 解析后的IP包
#[derive(Debug, Clone)]
pub struct _IpPacket {
    pub version: _IpVersion,
    pub src_ip: _IpAddress,
    pub dst_ip: _IpAddress,
    pub transport: _TransportProtocol,
    pub payload: Bytes,
    pub ttl: u8,
}

/// IP地址 (v4或v6)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum _IpAddress {
    V4([u8; 4]),
    V6([u8; 16]),
}

impl _IpAddress {
    /// 从字节切片解析
    pub fn from_bytes(version: _IpVersion, bytes: &[u8]) -> Result<Self, ParseError> {
        match version {
            _IpVersion::V4 => {
                if bytes.len() < 4 {
                    return Err(ParseError::InsufficientData);
                }
                let mut ip = [0u8; 4];
                ip.copy_from_slice(&bytes[..4]);
                Ok(_IpAddress::V4(ip))
            }
            _IpVersion::V6 => {
                if bytes.len() < 16 {
                    return Err(ParseError::InsufficientData);
                }
                let mut ip = [0u8; 16];
                ip.copy_from_slice(&bytes[..16]);
                Ok(_IpAddress::V6(ip))
            }
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            _IpAddress::V4(ip) => format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
            _IpAddress::V6(ip) => {
                let groups: Vec<String> = ip.chunks(2)
                    .map(|chunk| format!("{:02x}{:02x}", chunk[0], chunk[1]))
                    .collect();
                groups.join(":")
            }
        }
    }
}

impl std::fmt::Display for _IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// 解析错误
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("insufficient data")]
    InsufficientData,
    #[error("invalid version: {0}")]
    InvalidVersion(u8),
    #[error("invalid header length")]
    InvalidHeaderLength,
    #[error("checksum mismatch")]
    ChecksumMismatch,
}

/// 解析IPv4头部
pub fn _parse_ipv4(data: &[u8]) -> Result<(_Ipv4Header, usize), ParseError> {
    if data.len() < 20 {
        return Err(ParseError::InsufficientData);
    }

    let version = (data[0] >> 4) & 0x0F;
    if version != 4 {
        return Err(ParseError::InvalidVersion(version));
    }

    let ihl = data[0] & 0x0F;
    if ihl < 5 {
        return Err(ParseError::InvalidHeaderLength);
    }

    let header_len = (ihl as usize) * 4;
    if data.len() < header_len {
        return Err(ParseError::InsufficientData);
    }

    let header = _Ipv4Header {
        version,
        ihl,
        dscp: (data[1] >> 2) & 0x3F,
        ecn: data[1] & 0x03,
        total_length: u16::from_be_bytes([data[2], data[3]]),
        identification: u16::from_be_bytes([data[4], data[5]]),
        flags: (data[6] >> 5) & 0x07,
        fragment_offset: u16::from_be_bytes([data[6] & 0x1F, data[7]]),
        ttl: data[8],
        protocol: data[9],
        header_checksum: u16::from_be_bytes([data[10], data[11]]),
        src_ip: [data[12], data[13], data[14], data[15]],
        dst_ip: [data[16], data[17], data[18], data[19]],
        options: if ihl > 5 { data[20..header_len].to_vec() } else { vec![] },
    };

    Ok((header, header_len))
}

/// 解析IPv6头部
pub fn _parse_ipv6(data: &[u8]) -> Result<(_Ipv6Header, usize), ParseError> {
    if data.len() < 40 {
        return Err(ParseError::InsufficientData);
    }

    let version = (data[0] >> 4) & 0x0F;
    if version != 6 {
        return Err(ParseError::InvalidVersion(version));
    }

    let header = _Ipv6Header {
        version,
        traffic_class: ((data[0] & 0x0F) << 4) | ((data[1] >> 4) & 0x0F),
        flow_label: u32::from_be_bytes([data[1] & 0x0F, data[2], data[3], 0]),
        payload_length: u16::from_be_bytes([data[4], data[5]]),
        next_header: data[6],
        hop_limit: data[7],
        src_ip: {
            let mut ip = [0u8; 16];
            ip.copy_from_slice(&data[8..24]);
            ip
        },
        dst_ip: {
            let mut ip = [0u8; 16];
            ip.copy_from_slice(&data[24..40]);
            ip
        },
    };

    Ok((header, 40))
}

/// 协议号→传输协议
pub fn _protocol_from_number(num: u8) -> _TransportProtocol {
    match num {
        1 => _TransportProtocol::Icmp,
        6 => _TransportProtocol::Tcp,
        17 => _TransportProtocol::Udp,
        58 => _TransportProtocol::Icmpv6,
        other => _TransportProtocol::Other(other),
    }
}

/// 解析完整IP包
pub fn _parse_ip_packet(data: &[u8]) -> Result<_IpPacket, ParseError> {
    if data.is_empty() {
        return Err(ParseError::InsufficientData);
    }

    let version = (data[0] >> 4) & 0x0F;
    let version = match version {
        4 => _IpVersion::V4,
        6 => _IpVersion::V6,
        v => return Err(ParseError::InvalidVersion(v)),
    };

    match version {
        _IpVersion::V4 => {
            let (header, header_len) = _parse_ipv4(data)?;
            let payload = Bytes::copy_from_slice(&data[header_len..]);
            Ok(_IpPacket {
                version,
                src_ip: _IpAddress::V4(header.src_ip),
                dst_ip: _IpAddress::V4(header.dst_ip),
                transport: _protocol_from_number(header.protocol),
                payload,
                ttl: header.ttl,
            })
        }
        _IpVersion::V6 => {
            let (header, header_len) = _parse_ipv6(data)?;
            let payload = Bytes::copy_from_slice(&data[header_len..]);
            Ok(_IpPacket {
                version,
                src_ip: _IpAddress::V6(header.src_ip),
                dst_ip: _IpAddress::V6(header.dst_ip),
                transport: _protocol_from_number(header.next_header),
                payload,
                ttl: header.hop_limit,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ipv4_minimal() {
        // IPv4 header: version=4, ihl=5, total_length=20
        let mut data = vec![0u8; 20];
        data[0] = 0x45; // version=4, ihl=5
        data[8] = 64;   // ttl
        data[9] = 6;    // protocol=TCP
        data[12] = 192; // src=192.168.1.1
        data[13] = 168;
        data[14] = 1;
        data[15] = 1;
        data[16] = 10;  // dst=10.0.0.1
        data[17] = 0;
        data[18] = 0;
        data[19] = 1;

        let (header, len) = _parse_ipv4(&data).unwrap();
        assert_eq!(header.version, 4);
        assert_eq!(header.ihl, 5);
        assert_eq!(len, 20);
        assert_eq!(header.src_ip, [192, 168, 1, 1]);
        assert_eq!(header.dst_ip, [10, 0, 0, 1]);
    }

    #[test]
    fn parse_ipv6_minimal() {
        let mut data = vec![0u8; 40];
        data[0] = 0x60; // version=6
        data[6] = 59;   // next_header=NoNextHeader
        data[7] = 64;   // hop_limit

        let (header, len) = _parse_ipv6(&data).unwrap();
        assert_eq!(header.version, 6);
        assert_eq!(len, 40);
    }

    #[test]
    fn ip_address_display() {
        let ip4 = _IpAddress::V4([192, 168, 1, 1]);
        assert_eq!(ip4.to_string(), "192.168.1.1");

        let ip6 = _IpAddress::V6([0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(ip6.to_string(), "2001:0db8:0000:0000:0000:0000:0000:0001");
    }

    #[test]
    fn protocol_mapping() {
        assert_eq!(_protocol_from_number(6), _TransportProtocol::Tcp);
        assert_eq!(_protocol_from_number(17), _TransportProtocol::Udp);
        assert_eq!(_protocol_from_number(1), _TransportProtocol::Icmp);
    }
}

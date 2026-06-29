use std::net::ToSocketAddrs;

use crate::socks5::parse_host_port;

pub(crate) fn is_private_target(target: &str) -> bool {
    let (host, _port) = match parse_host_port(target) {
        Some(hp) => hp,
        None => return false,
    };
    let addrs: Vec<std::net::SocketAddr> = match (host.as_str(), 0).to_socket_addrs() {
        Ok(a) => a.collect(),
        Err(_) => return false,
    };
    addrs.iter().any(|addr| is_private_ip(addr.ip()))
}

pub(crate) fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            if octets[0] == 0 {
                return true;
            }
            if octets[0] == 10 {
                return true;
            }
            if octets[0] == 172 && (octets[1] & 0xF0) == 16 {
                return true;
            }
            if octets[0] == 192 && octets[1] == 168 {
                return true;
            }
            if octets[0] == 169 && octets[1] == 254 {
                return true;
            }
            if octets[0] == 127 {
                return true;
            }
            if octets[0] == 100 && (octets[1] & 0xC0) == 64 {
                return true;
            }
            false
        }
        std::net::IpAddr::V6(v6) => {
            let segments = v6.segments();
            if segments == [0, 0, 0, 0, 0, 0, 0, 1] {
                return true;
            }
            if segments[0] & 0xFFC0 == 0xFE80 {
                return true;
            }
            if segments[0] & 0xFE00 == 0xFC00 {
                return true;
            }
            if segments[0] == 0
                && segments[1] == 0
                && segments[2] == 0
                && segments[3] == 0
                && segments[4] == 0
                && segments[5] == 0xffff
            {
                let v4_octets = [
                    (segments[6] >> 8) as u8,
                    (segments[6] & 0xff) as u8,
                    (segments[7] >> 8) as u8,
                    (segments[7] & 0xff) as u8,
                ];
                if v4_octets[0] == 0 || v4_octets[0] == 10 {
                    return true;
                }
                if v4_octets[0] == 172 && (v4_octets[1] & 0xF0) == 16 {
                    return true;
                }
                if v4_octets[0] == 192 && v4_octets[1] == 168 {
                    return true;
                }
                if v4_octets[0] == 169 && v4_octets[1] == 254 {
                    return true;
                }
                if v4_octets[0] == 127 {
                    return true;
                }
                if v4_octets[0] == 100 && (v4_octets[1] & 0xC0) == 64 {
                    return true;
                }
            }
            false
        }
    }
}

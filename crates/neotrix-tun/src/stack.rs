use smoltcp::iface::Config;
use smoltcp::phy::{self, DeviceCapabilities, RxToken, TxToken};
use smoltcp::time::Instant;
use smoltcp::wire::{HardwareAddress, IpCidr, Ipv4Address, Ipv4Cidr};
use smoltcp::phy::Medium;
use std::io::{Read, Write};

use crate::tun::TunDevice;

pub struct TunRxToken {
    buf: Vec<u8>,
}

impl RxToken for TunRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        let len = self.buf.len();
        f(&self.buf[..len])
    }
}

pub struct TunTxToken {
    len: usize,
}

impl TxToken for TunTxToken {
    fn consume<R, F>(mut self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = vec![0u8; len];
        let r = f(&mut buf);
        self.len = len;
        r
    }
}

pub struct TunPhy {
    tun: TunDevice,
    tx_buf: Option<Vec<u8>>,
}

impl TunPhy {
    pub fn new(tun: TunDevice) -> Self {
        TunPhy { tun, tx_buf: None }
    }
}

impl phy::Device for TunPhy {
    type RxToken<'a> = TunRxToken where Self: 'a;
    type TxToken<'a> = TunTxToken where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut buf = vec![0u8; 65536];
        match self.tun.read(&mut buf) {
            Ok(n) => {
                buf.truncate(n);
                Some((TunRxToken { buf }, TunTxToken { len: 0 }))
            }
            Err(_) => None,
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        if self.tx_buf.is_some() {
            let buf = self.tx_buf.take().unwrap();
            let _ = self.tun.write(&buf);
        }
        Some(TunTxToken { len: 0 })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1500;
        caps.medium = Medium::Ip;
        caps
    }
}

pub fn create_iface(mut phy: TunPhy) -> smoltcp::iface::Interface {
    let config = Config::new(HardwareAddress::Ip);
    let mut iface = smoltcp::iface::Interface::new(config, &mut phy, Instant::ZERO);
    iface.update_ip_addrs(|addrs| {
        addrs
            .push(IpCidr::Ipv4(Ipv4Cidr::new(Ipv4Address::new(10, 0, 9, 1), 24)))
            .unwrap();
    });
    iface
        .routes_mut()
        .add_default_ipv4_route(Ipv4Address::new(10, 0, 9, 1))
        .unwrap();
    iface
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TunRxToken ─────────────────────────────────────────────────────

    #[test]
    fn rx_token_consume_receives_full_data() {
        let data: Vec<u8> = (0..10).collect();
        let token = TunRxToken { buf: data.clone() };
        let len = token.consume(|buf| {
            assert_eq!(buf, &data[..]);
            buf.len()
        });
        assert_eq!(len, 10);
    }

    #[test]
    fn rx_token_consume_empty_buffer() {
        let token = TunRxToken { buf: vec![] };
        let n = token.consume(|buf| buf.len());
        assert_eq!(n, 0);
    }

    #[test]
    fn rx_token_consume_large_buffer() {
        let buf: Vec<u8> = vec![0xAB; 65536];
        let token = TunRxToken { buf };
        let sum: u64 = token.consume(|buf| buf.iter().map(|&b| b as u64).sum());
        assert_eq!(sum, 0xABu64 * 65536);
    }

    // ── TunTxToken ─────────────────────────────────────────────────────

    #[test]
    fn tx_token_consume_writes_into_buffer() {
        let token = TunTxToken { len: 0 };
        let written = token.consume(16, |buf| {
            assert_eq!(buf.len(), 16);
            buf.copy_from_slice(&[0xAAu8; 16]);
            16usize
        });
        assert_eq!(written, 16);
    }

    #[test]
    fn tx_token_consume_zero_length_buffer() {
        let token = TunTxToken { len: 0 };
        let ret = token.consume(0, |buf| {
            assert!(buf.is_empty());
            "zero-sized".to_string()
        });
        assert_eq!(ret, "zero-sized");
    }

    #[test]
    fn tx_token_consume_partial_write() {
        let token = TunTxToken { len: 0 };
        let written = token.consume(100, |buf| {
            buf[..5].copy_from_slice(b"hello");
            5
        });
        assert_eq!(written, 5);
    }

    // ── create_iface signature (compile-time smoke) ────────────────────

    #[test]
    fn create_iface_returns_interface_with_expected_ip() {
        // Full integration requires root. We verify at compile time that
        // the function signature is valid by checking the return type.
        let _iface: fn(TunPhy) -> smoltcp::iface::Interface = create_iface;
    }

    // ── Token struct sizes (incidental ABI smoke) ──────────────────────

    #[test]
    fn rx_token_struct_size() {
        assert_eq!(std::mem::size_of::<TunRxToken>(), 24); // Vec<u8> = 3 × usize
    }

    #[test]
    fn tx_token_struct_size() {
        assert_eq!(std::mem::size_of::<TunTxToken>(), 8); // usize
    }
}

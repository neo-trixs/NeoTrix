use std::io::{self, Read, Write};
use std::net::Ipv4Addr;
use std::process::Command;

pub fn create_tun() -> io::Result<tun::Device> {
    let mut config = tun::Configuration::default();
    config
        .tun_name("utun9")
        .address(Ipv4Addr::new(10, 0, 9, 1))
        .netmask(Ipv4Addr::new(255, 255, 255, 0))
        .mtu(1500)
        .up();

    let device = tun::create(&config)?;

    add_default_route()?;

    log::info!("TUN utun9 created: 10.0.9.1/24");
    Ok(device)
}

fn add_default_route() -> io::Result<()> {
    Command::new("route")
        .args(["add", "-net", "0.0.0.0/1", "10.0.9.1"])
        .output().map_err(io::Error::other)?;
    Command::new("route")
        .args(["add", "-net", "128.0.0.0/1", "10.0.9.1"])
        .output().map_err(io::Error::other)?;
    Ok(())
}

pub fn remove_default_route() -> io::Result<()> {
    let _ = Command::new("route")
        .args(["delete", "-net", "0.0.0.0/1", "10.0.9.1"]).output();
    let _ = Command::new("route")
        .args(["delete", "-net", "128.0.0.0/1", "10.0.9.1"]).output();
    Ok(())
}

pub struct TunDevice(tun::Device);

impl TunDevice {
    pub fn new() -> io::Result<Self> {
        create_tun().map(TunDevice)
    }
}

impl Read for TunDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf)
    }
}

impl Write for TunDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

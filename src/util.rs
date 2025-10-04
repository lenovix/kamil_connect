// Fungsi utilitas umum seperti local_ip()

use std::io;
use std::net::{Ipv4Addr, UdpSocket};

pub fn local_ip() -> io::Result<Ipv4Addr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    if let Ok(addr) = socket.local_addr() {
        if let std::net::IpAddr::V4(ipv4) = addr.ip() {
            return Ok(ipv4);
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, "Gagal mendeteksi IP lokal"))
}
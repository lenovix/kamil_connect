use std::io::{self, Write};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;

fn main() -> io::Result<()> {
    let port = 34254;

    // 🔍 Deteksi IP lokal (IPv4)
    let local_ip = local_ip()?;
    println!("Detected local IP: {}", local_ip);

    // Hitung alamat broadcast dari subnet kelas C (misal 192.168.x.255)
    let broadcast_ip = Ipv4Addr::new(local_ip.octets()[0], local_ip.octets()[1], local_ip.octets()[2], 255);
    let broadcast_addr: SocketAddr = SocketAddr::from((broadcast_ip, port));

    // 🔊 Bind ke semua interface supaya bisa menerima pesan broadcast
    let socket = UdpSocket::bind(("0.0.0.0", port))?;
    socket.set_broadcast(true)?;
    println!("Listening on {:?}", socket.local_addr()?);

    // Clone socket untuk thread listener
    let recv_socket = socket.try_clone()?;

    // Thread untuk menerima pesan
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match recv_socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    let msg = String::from_utf8_lossy(&buf[..amt]);
                    println!("\n[{}] {}", src, msg);
                    print!("> ");
                    io::stdout().flush().unwrap();
                }
                Err(e) => eprintln!("Error receiving: {}", e),
            }
        }
    });

    println!("Kamil Connect (broadcast) running on port {}", port);
    println!("Type your message and press Enter to send (/quit to exit):");

    // Loop input user
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "/quit" {
            break;
        }

        socket.send_to(input.as_bytes(), broadcast_addr)?;
    }

    Ok(())
}

/// Deteksi IP lokal dengan membuat koneksi dummy ke alamat eksternal (tanpa benar-benar mengirim data)
fn local_ip() -> io::Result<Ipv4Addr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?; // Google DNS, hanya untuk tahu IP lokal kita
    if let Ok(addr) = socket.local_addr() {
        if let std::net::IpAddr::V4(ipv4) = addr.ip() {
            return Ok(ipv4);
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, "Failed to detect local IP"))
}

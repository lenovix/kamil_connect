use std::net::{UdpSocket, SocketAddr};
use std::thread;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Port yang digunakan
    let port = 34254;

    // Bind socket untuk menerima pesan (UDP listener)
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_broadcast(true)?;
    
    // Clone socket untuk thread
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
                Err(e) => {
                    eprintln!("Error receiving: {}", e);
                }
            }
        }
    });

    println!("Kamil Connect (broadcast) running on port {}", port);
    println!("Type your message and hit Enter to send:");

    let broadcast_addr: SocketAddr = format!("255.255.255.255:{}", port).parse().unwrap();

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

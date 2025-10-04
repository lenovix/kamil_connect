use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    let udp_port = 34254;
    let tcp_port = 40000;

    // --- Minta nickname user ---
    print!("Masukkan nickname Anda: ");
    io::stdout().flush()?;
    let mut nickname = String::new();
    io::stdin().read_line(&mut nickname)?;
    let nickname = nickname.trim().to_string();

    // --- Dapatkan IP lokal ---
    let local_ip = local_ip()?;
    println!("Detected local IP: {}", local_ip);

    // --- Setup UDP socket ---
    let broadcast_ip =
        Ipv4Addr::new(local_ip.octets()[0], local_ip.octets()[1], local_ip.octets()[2], 255);
    let udp_socket = UdpSocket::bind(("0.0.0.0", udp_port))?;
    udp_socket.set_broadcast(true)?;
    println!("UDP listening on {}", udp_port);

    let udp_recv = udp_socket.try_clone()?;

    // --- Shared user list ---
    let active_users: Arc<Mutex<HashMap<String, Instant>>> = Arc::new(Mutex::new(HashMap::new()));

    // Thread: Menerima pesan UDP
    {
        let users = Arc::clone(&active_users);
        thread::spawn(move || udp_listener(udp_recv, users));
    }

    // Thread: Mengirimkan sinyal HELLO berkala
    {
        let udp_socket = udp_socket.try_clone()?;
        let users = Arc::clone(&active_users);
        let nickname = nickname.clone();
        thread::spawn(move || loop {
            let msg = format!("HELLO:{}", nickname);
            let broadcast_addr: SocketAddr = SocketAddr::from((broadcast_ip, udp_port));
            let _ = udp_socket.send_to(msg.as_bytes(), broadcast_addr);

            // Hapus user yang timeout (tidak aktif > 10 detik)
            {
                let mut users = users.lock().unwrap();
                users.retain(|_, last_seen| last_seen.elapsed() < Duration::from_secs(10));
            }

            thread::sleep(Duration::from_secs(3));
        });
    }

    // --- Setup TCP listener ---
    let tcp_listener = TcpListener::bind(("0.0.0.0", tcp_port))?;
    println!("TCP listening on {}", tcp_port);

    // Thread untuk menerima koneksi TCP masuk
    thread::spawn(move || tcp_server(tcp_listener));

    // --- Input utama pengguna ---
    println!("\nKamil Connect running!");
    println!("Commands:");
    println!("  (chat umum) ketik pesan biasa");
    println!("  /connect <ip>  → mulai chat pribadi");
    println!("  /users         → tampilkan user aktif");
    println!("  /quit          → keluar\n");

    let broadcast_addr: SocketAddr = SocketAddr::from((broadcast_ip, udp_port));

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "/quit" {
            break;
        }

        if input == "/users" {
            let users = active_users.lock().unwrap();
            println!("\n--- User Aktif ---");
            for (ip, last_seen) in users.iter() {
                println!("{} ({} detik lalu)", ip, last_seen.elapsed().as_secs());
            }
            println!("------------------\n");
            continue;
        }

        if input.starts_with("/connect ") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() == 2 {
                let ip = parts[1];
                if let Err(e) = tcp_client(ip.to_string(), tcp_port) {
                    eprintln!("Failed to connect to {}: {}", ip, e);
                }
            } else {
                println!("Usage: /connect <ip>");
            }
            continue;
        }

        // Kirim pesan umum (UDP)
        udp_socket.send_to(input.as_bytes(), broadcast_addr)?;
    }

    Ok(())
}

fn udp_listener(socket: UdpSocket, users: Arc<Mutex<HashMap<String, Instant>>>) {
    let mut buf = [0u8; 1024];
    loop {
        if let Ok((amt, src)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..amt]);
            if msg.starts_with("HELLO:") {
                let _nick = msg.strip_prefix("HELLO:").unwrap_or("Unknown");
                let ip = src.ip().to_string();
                users.lock().unwrap().insert(ip, Instant::now());
            } else {
                println!("\n[UDP:{}] {}", src, msg);
                print!("> ");
                io::stdout().flush().unwrap();
            }
        }
    }
}

fn tcp_server(listener: TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_tcp_client(stream));
            }
            Err(e) => eprintln!("TCP accept error: {}", e),
        }
    }
}

fn handle_tcp_client(mut stream: TcpStream) {
    let peer = stream.peer_addr().unwrap();
    let reader = BufReader::new(stream.try_clone().unwrap());

    println!("\n[Connected TCP from {}]", peer);
    for line in reader.lines() {
        match line {
            Ok(msg) => {
                println!("\n[TCP:{}] {}", peer, msg);
                print!("> ");
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                eprintln!("Error reading from {}: {}", peer, e);
                break;
            }
        }
    }
}

fn tcp_client(ip: String, port: u16) -> io::Result<()> {
    let addr = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(&addr)?;
    println!("Connected to {} via TCP. Type messages, /exit to close.", addr);

    let mut input = String::new();
    loop {
        print!("(TCP)> ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let msg = input.trim();

        if msg == "/exit" {
            println!("Closing connection.");
            break;
        }

        writeln!(stream, "{}", msg)?;
    }

    Ok(())
}

fn local_ip() -> io::Result<Ipv4Addr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    if let Ok(addr) = socket.local_addr() {
        if let std::net::IpAddr::V4(ipv4) = addr.ip() {
            return Ok(ipv4);
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, "Failed to detect local IP"))
}

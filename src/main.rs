use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::sync::atomic::{AtomicUsize, Ordering};

// --- ID pesan global ---
static MSG_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

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

    // --- Shared user list dan ACK state ---
    let active_users: Arc<Mutex<HashMap<String, (String, Instant)>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let received_acks: Arc<Mutex<HashSet<usize>>> = Arc::new(Mutex::new(HashSet::new()));

    // --- Channel untuk ACK antara thread ---
    let (ack_tx, ack_rx) = mpsc::channel::<usize>();

    // Thread: Menerima pesan UDP
    {
        let users = Arc::clone(&active_users);
        let acks = Arc::clone(&received_acks);
        let tx = ack_tx.clone();
        thread::spawn(move || udp_listener(udp_recv, users, acks, tx));
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

            // Hapus user yang timeout (>10 detik)
            {
                let mut users = users.lock().unwrap();
                users.retain(|_, (_, last_seen)| last_seen.elapsed() < Duration::from_secs(10));
            }

            thread::sleep(Duration::from_secs(3));
        });
    }

    // --- Setup TCP listener ---
    let tcp_listener = TcpListener::bind(("0.0.0.0", tcp_port))?;
    println!("TCP listening on {}", tcp_port);
    thread::spawn(move || tcp_server(tcp_listener));

    // --- Input utama pengguna ---
    println!("\nKamil Connect running!");
    println!("Commands:");
    println!("  (chat umum) ketik pesan biasa");
    println!("  /connect <nickname>  → mulai chat pribadi");
    println!("  /users               → tampilkan user aktif");
    println!("  /quit                → keluar\n");

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
            for (ip, (nick, last_seen)) in users.iter() {
                if ip != &local_ip.to_string() {
                    println!("{} ({} detik lalu)", nick, last_seen.elapsed().as_secs());
                }
            }
            println!("------------------\n");
            continue;
        }

        if input.starts_with("/connect ") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() == 2 {
                let target_nick = parts[1];
                let users = active_users.lock().unwrap();

                if let Some((ip, _)) = users.iter().find_map(|(ip, (nick, t))| {
                    if nick == target_nick {
                        Some((ip.clone(), t))
                    } else {
                        None
                    }
                }) {
                    if let Err(e) = tcp_client(ip.to_string(), tcp_port) {
                        eprintln!("Failed to connect to {}: {}", ip, e);
                    }
                } else {
                    println!("User '{}' tidak ditemukan.", target_nick);
                }
            } else {
                println!("Usage: /connect <nickname>");
            }
            continue;
        }

        // Kirim pesan umum (UDP) dengan ACK
        let msg_id = MSG_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let full_msg = format!("MSG:{}:{}", msg_id, input);

        for attempt in 1..=3 {
            udp_socket.send_to(full_msg.as_bytes(), broadcast_addr)?;
            println!("[UDP] Pesan dikirim (attempt {})...", attempt);

            // Tunggu ACK 1 detik
            if let Ok(received_id) = ack_rx.recv_timeout(Duration::from_secs(1)) {
                if received_id == msg_id {
                    println!("[UDP] Pesan berhasil diterima.");
                    break;
                }
            } else if attempt == 3 {
                eprintln!("[UDP] Pesan tidak diterima setelah 3 kali percobaan.");
            }
        }
    }

    Ok(())
}

fn udp_listener(
    socket: UdpSocket,
    users: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    acks: Arc<Mutex<HashSet<usize>>>,
    ack_tx: mpsc::Sender<usize>,
) {
    let mut buf = [0u8; 1024];
    loop {
        if let Ok((amt, src)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..amt]);
            if msg.starts_with("HELLO:") {
                let nick = msg.strip_prefix("HELLO:").unwrap_or("Unknown").to_string();
                let ip = src.ip().to_string();
                users.lock().unwrap().insert(ip, (nick, Instant::now()));
            } else if msg.starts_with("MSG:") {
                // Format: MSG:<id>:<content>
                let parts: Vec<&str> = msg.splitn(3, ':').collect();
                if parts.len() == 3 {
                    if let Ok(id) = parts[1].parse::<usize>() {
                        let content = parts[2];
                        println!("\n[UDP:{}] {}", src, content);
                        print!("> ");
                        io::stdout().flush().unwrap();

                        // Kirim ACK ke pengirim
                        let ack_msg = format!("ACK:{}", id);
                        let _ = socket.send_to(ack_msg.as_bytes(), src);
                    }
                }
            } else if msg.starts_with("ACK:") {
                if let Ok(id) = msg.strip_prefix("ACK:").unwrap_or("").parse::<usize>() {
                    acks.lock().unwrap().insert(id);
                    let _ = ack_tx.send(id);
                }
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
    println!("\n[Connected TCP from {}]", peer);
    print!("(TCP)> ");
    io::stdout().flush().unwrap();

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream.try_clone().unwrap();

    // Thread: menerima pesan
    thread::spawn(move || {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    println!("\n[Connection closed by {}]", peer);
                    break;
                }
                Ok(_) => {
                    let msg = line.trim();
                    if !msg.is_empty() {
                        println!("\n[TCP:{}] {}", peer, msg);
                        print!("(TCP)> ");
                        io::stdout().flush().unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from {}: {}", peer, e);
                    break;
                }
            }
        }
    });

    // Thread utama: kirim pesan balik
    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        input.clear();
        stdin.read_line(&mut input).unwrap();
        let msg = input.trim();

        if msg == "/exit" {
            println!("Menutup koneksi TCP dengan {}", peer);
            break;
        }

        if let Err(e) = writeln!(writer, "{}", msg) {
            eprintln!("Error sending to {}: {}", peer, e);
            break;
        }
        let _ = writer.flush();
    }
}

fn tcp_client(ip: String, port: u16) -> io::Result<()> {
    let addr = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(&addr)?;
    println!("Connected to {} via TCP. Type messages, /exit to close.", addr);

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream.try_clone().unwrap();

    // Thread: menerima pesan dari lawan bicara
    thread::spawn(move || {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    println!("\n[Connection closed by {}]", addr);
                    break;
                }
                Ok(_) => {
                    let msg = line.trim();
                    if !msg.is_empty() {
                        println!("\n[TCP:{}] {}", addr, msg);
                        print!("(TCP)> ");
                        io::stdout().flush().unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from {}: {}", addr, e);
                    break;
                }
            }
        }
    });

    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        print!("(TCP)> ");
        io::stdout().flush()?;
        input.clear();
        stdin.read_line(&mut input)?;
        let msg = input.trim();

        if msg == "/exit" {
            println!("Menutup koneksi TCP.");
            break;
        }

        writeln!(writer, "{}", msg)?;
        writer.flush()?;
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

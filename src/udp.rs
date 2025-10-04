// Semua fungsi terkait broadcast, listener, dan ACK

use std::collections::HashSet;
use std::io::{self, Write};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

use crate::user::{UserMap, cleanup_inactive};
use crate::constants::{USER_TIMEOUT, HELLO_INTERVAL, MAX_RETRY};

pub static MSG_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn start_udp(
    nickname: String,
    local_ip: Ipv4Addr,
    udp_port: u16,
    users: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    ack_tx: mpsc::Sender<usize>,
) -> io::Result<UdpSocket> {
    let broadcast_ip = Ipv4Addr::new(local_ip.octets()[0], local_ip.octets()[1], local_ip.octets()[2], 255);
    let socket = UdpSocket::bind(("0.0.0.0", udp_port))?;
    socket.set_broadcast(true)?;

    let recv_socket = socket.try_clone()?;
    let received_acks: Arc<Mutex<HashSet<usize>>> = Arc::new(Mutex::new(HashSet::new()));

    // Listener
    {
        let users = Arc::clone(&users);
        let acks = Arc::clone(&received_acks);
        let tx = ack_tx.clone();
        thread::spawn(move || udp_listener(recv_socket, users, acks, tx));
    }

    // Broadcaster
    {
        let socket = socket.try_clone()?;
        let users = Arc::clone(&users);
        let nickname = nickname.clone();
        thread::spawn(move || loop {
            let msg = format!("HELLO:{}", nickname);
            let broadcast_addr: SocketAddr = SocketAddr::from((broadcast_ip, udp_port));
            let _ = socket.send_to(msg.as_bytes(), broadcast_addr);
            cleanup_inactive(&users, Duration::from_secs(USER_TIMEOUT));
            thread::sleep(Duration::from_secs(HELLO_INTERVAL));
        });
    }

    Ok(socket)
}

pub fn send_udp_message(socket: &UdpSocket, message: &str, broadcast_addr: SocketAddr, ack_rx: &mpsc::Receiver<usize>) -> io::Result<()> {
    let msg_id = MSG_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let full_msg = format!("MSG:{}:{}", msg_id, message);

    for attempt in 1..=MAX_RETRY {
        socket.send_to(full_msg.as_bytes(), broadcast_addr)?;
        println!("[UDP] Pesan dikirim (attempt {})...", attempt);

        if let Ok(received_id) = ack_rx.recv_timeout(Duration::from_secs(1)) {
            if received_id == msg_id {
                println!("[UDP] Pesan berhasil diterima.");
                break;
            }
        } else if attempt == MAX_RETRY {
            eprintln!("[UDP] Pesan tidak diterima setelah {} kali percobaan.", MAX_RETRY);
        }
    }
    Ok(())
}

fn udp_listener(socket: UdpSocket, users: UserMap, acks: Arc<Mutex<HashSet<usize>>>, ack_tx: mpsc::Sender<usize>) {
    let mut buf = [0u8; 1024];
    loop {
        if let Ok((amt, src)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..amt]);
            if msg.starts_with("HELLO:") {
                let nick = msg.strip_prefix("HELLO:").unwrap_or("Unknown").to_string();
                let ip = src.ip().to_string();
                users.lock().unwrap().insert(ip, (nick, Instant::now()));
            } else if msg.starts_with("MSG:") {
                let parts: Vec<&str> = msg.splitn(3, ':').collect();
                if parts.len() == 3 {
                    if let Ok(id) = parts[1].parse::<usize>() {
                        let content = parts[2];
                        println!("\n[UDP:{}] {}", src, content);
                        print!("> ");
                        io::stdout().flush().unwrap();
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
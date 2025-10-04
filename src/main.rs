// Titik masuk aplikasi; mengatur logika utama dan koordinasi antar modul

mod udp;
mod tcp;
mod user;
mod util;
mod constants;

use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex, mpsc};
// use std::time::Duration;
use std::io::{self, Write};
use user::UserMap;
use udp::{start_udp, send_udp_message};
use tcp::{tcp_server, tcp_client};
use constants::*;

fn main() -> io::Result<()> {
    print!("Masukkan nickname Anda: ");
    io::stdout().flush()?;
    let mut nickname = String::new();
    io::stdin().read_line(&mut nickname)?;
    let nickname = nickname.trim().to_string();

    let local_ip = util::local_ip()?;
    println!("Detected local IP: {}", local_ip);

    let users: UserMap = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let (ack_tx, ack_rx) = mpsc::channel::<usize>();

    let udp_socket = start_udp(nickname.clone(), local_ip, UDP_PORT, Arc::clone(&users), ack_tx)?;
    let broadcast_addr: SocketAddr = SocketAddr::from(([local_ip.octets()[0], local_ip.octets()[1], local_ip.octets()[2], 255], UDP_PORT));

    let tcp_listener = TcpListener::bind(("0.0.0.0", TCP_PORT))?;
    std::thread::spawn(move || tcp_server(tcp_listener));

    println!("\nKamil-Connect running as \"{}\" ({})", nickname, local_ip);
    println!("Commands:");
    println!("  /users  → tampilkan user aktif");
    println!("  /connect <nickname> → chat pribadi");
    println!("  /quit   → keluar\n");

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "/quit" => break,
            "/users" => user::print_active(&users, &local_ip.to_string()),
            _ if input.starts_with("/connect ") => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() == 2 {
                    let target = parts[1];
                    let users = users.lock().unwrap();
                    if let Some((ip, _)) = users.iter().find(|(_, (nick, _))| nick == target) {
                        if let Err(e) = tcp_client(ip.clone(), TCP_PORT) {
                            eprintln!("Error: {}", e);
                        }
                    } else {
                        println!("User '{}' tidak ditemukan.", target);
                    }
                }
            }
            _ => {
                send_udp_message(&udp_socket, input, broadcast_addr, &ack_rx)?;
            }
        }
    }

    Ok(())
}
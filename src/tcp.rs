// Semua fungsi terkait koneksi personal (TCP server/client)

use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn tcp_server(listener: TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_tcp_client(stream));
            }
            Err(e) => eprintln!("TCP accept error: {}", e),
        }
    }
}

pub fn handle_tcp_client(stream: TcpStream) {
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

pub fn tcp_client(ip: String, port: u16) -> io::Result<()> {
    let addr = format!("{}:{}", ip, port);
    let stream = TcpStream::connect(&addr)?;
    println!("Connected to {} via TCP. Type messages, /exit to close.", addr);

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream.try_clone()?;

    // Thread: menerima pesan dari lawan bicara
    {
        let addr_clone = addr.clone();
        thread::spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        println!("\n[Connection closed by {}]", addr_clone);
                        break;
                    }
                    Ok(_) => {
                        let msg = line.trim();
                        if !msg.is_empty() {
                            println!("\n[TCP:{}] {}", addr_clone, msg);
                            print!("(TCP)> ");
                            io::stdout().flush().unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from {}: {}", addr_clone, e);
                        break;
                    }
                }
            }
        });
    }

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
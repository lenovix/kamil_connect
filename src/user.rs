// Struktur data dan logika manajemen user aktif

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type UserMap = Arc<Mutex<HashMap<String, (String, Instant)>>>;

pub fn cleanup_inactive(users: &UserMap, timeout: Duration) {
    let mut users = users.lock().unwrap();
    users.retain(|_, (_, last_seen)| last_seen.elapsed() < timeout);
}

pub fn print_active(users: &UserMap, local_ip: &str) {
    let users = users.lock().unwrap();
    println!("\n--- User Aktif ---");
    for (ip, (nick, last_seen)) in users.iter() {
        if ip != local_ip {
            println!("{} ({} detik lalu)", nick, last_seen.elapsed().as_secs());
        }
    }
    println!("------------------\n");
}
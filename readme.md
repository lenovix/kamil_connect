# 🛰️ Kamil Connect v0.0.1

![Rust](https://img.shields.io/badge/Rust-Programming%20Language-orange)
![Version](https://img.shields.io/badge/Version-v0.0.1-blue)
![Status](https://img.shields.io/badge/Status-Active-brightgreen)
![License](https://img.shields.io/badge/License-MIT-lightgrey)

Kamil Connect adalah aplikasi sederhana berbasis **Rust** untuk komunikasi antar komputer dalam satu jaringan lokal menggunakan **UDP** (untuk broadcast pesan umum) dan **TCP** (untuk chat pribadi).

## 🚀 Fitur Utama
- **Broadcast pesan umum** ke seluruh jaringan via UDP  
- **Chat pribadi (direct message)** via TCP  
- **Deteksi pengguna aktif** otomatis melalui sinyal `HELLO`  
- **Real-time update** daftar pengguna aktif  

## 🧱 Arsitektur
- **UDP** digunakan untuk:
  - Broadcast pesan umum
  - Mengirim sinyal `HELLO:<nickname>` setiap 3 detik
- **TCP** digunakan untuk:
  - Chat langsung antar 2 pengguna
- Data pengguna disimpan sementara di `HashMap` yang disinkronisasi dengan `Arc<Mutex<...>>`

## ⚙️ Cara Menjalankan

### 1. Install Rust
Pastikan Rust sudah terpasang di komputer Anda.
```bash
curl https://sh.rustup.rs -sSf | sh
```

### 2. Clone atau buat proyek baru
```bash
cargo new kamil_connect
cd kamil_connect
```

### 3. Ganti isi file `src/main.rs`
Salin kode dari versi 0.0.1 ke file `src/main.rs`.

### 4. Jalankan program
Buka **dua terminal di komputer berbeda** dalam jaringan yang sama (Wi-Fi/LAN) dan jalankan:
```bash
cargo run
```

### 5. Perintah yang tersedia
| Perintah | Deskripsi |
|-----------|------------|
| `/users` | Menampilkan daftar user aktif |
| `/connect <nickname>` | Menghubungi user tertentu via TCP |
| `/quit` | Keluar dari aplikasi |
| `/exit` | Menutup sesi TCP |

## ⚠️ Catatan
- Pastikan **firewall** mengizinkan akses port UDP `34254` dan TCP `40000`.
- Jika pesan tidak terkirim, kemungkinan disebabkan oleh latency jaringan atau OS membatasi broadcast UDP.

---

## 🎯 Rencana Fitur Mendatang

### 🧩 Sistem Status & Presence (0.0.2)
   Menampilkan status pengguna (online, idle, offline) secara otomatis dan manual.  
   - Menambahkan perintah `/status <pesan>`  
   - Status diperbarui berdasarkan aktivitas dan sinyal `HELLO`.

### 💬 Private Chat dengan History Lokal (0.0.3)
   Menyimpan riwayat percakapan antar pengguna di folder `chat_logs/`.  
   - File log otomatis dibuat untuk setiap sesi chat pribadi.  
   - Mendukung penyimpanan teks dan timestamp pesan.

### 🌐 Grup Chat (Room System) (0.0.4)
   Menambahkan dukungan grup agar pengguna bisa membuat dan bergabung ke ruang obrolan.  
   - Perintah baru: `/create`, `/join`, `/leave`, `/groups`  
   - Pesan hanya diterima oleh anggota grup yang sama.

### 🔒 Enkripsi Pesan (0.0.5)
   Menambahkan lapisan keamanan dengan enkripsi AES-256-GCM atau ChaCha20.  
   - Semua pesan (UDP & TCP) dienkripsi.  
   - Mendukung kunci sementara atau negosiasi antar pengguna.

### 🖥️ Antarmuka Pengguna (TUI/GUI) (0.0.6)
   Memberikan pengalaman pengguna yang lebih baik:  
   - Tahap awal: **TUI (Text UI)** dengan crate `tui` atau `crossterm`.  
   - Tahap lanjut: **GUI desktop** dengan framework seperti `egui` atau `tauri`.  
   - Menampilkan daftar user, status, dan area chat dalam satu tampilan.

### 📁 File Transfer (0.0.7)
   - Mengirim file antar perangkat melalui TCP stream.

---

## 📦 Versi
**v0.0.1** — Rilis awal, mendukung broadcast UDP, TCP chat, dan deteksi user aktif.

---

Dibuat oleh **Ichsanul Kamil Sudarmi** ✨
# Kamil Connect v0.0.1

Kamil Connect adalah aplikasi sederhana berbasis **Rust** untuk komunikasi antar komputer dalam satu jaringan lokal menggunakan **UDP** (untuk broadcast pesan umum) dan **TCP** (untuk chat pribadi).

## 🚀 Fitur Utama
- **Broadcast pesan umum** ke seluruh jaringan via UDP
- **Chat pribadi (direct message)** via TCP
- **Deteksi pengguna aktif** otomatis melalui sinyal `HELLO`
- **Real-time update** daftar pengguna aktif

## 🧱 Arsitektur
- UDP digunakan untuk:
  - Broadcast pesan umum
  - Mengirim sinyal `HELLO:<nickname>` setiap 3 detik
- TCP digunakan untuk:
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

## 📦 Versi
**v0.0.1** — Rilis awal, mendukung broadcast UDP, TCP chat, dan deteksi user aktif.

---
Dibuat oleh **len vix** ✨

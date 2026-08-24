# PLAN_AUDIO_CUTTER.md
## PotongAudio — Desktop App Offline (Cross-Platform)

**Status:** Draft v3 (arsitektur hybrid: React UI + Rust native processing) — Fase 0 (PoC) SELESAI, lihat `POC_FINDINGS.md`
**Tanggal:** 2026-08-16
**Prasyarat:** Fase 0 (PoC validasi teknis) WAJIB selesai sebelum Fase 1 dimulai.

---

## 0. Ringkasan Perubahan dari Plan Awal

| Isu | Plan v1 (full web/WASM) | Plan v3 (hybrid Rust + React) |
|---|---|---|
| Validasi teknis | Langsung ke Fase 1 | Ditambah **Fase 0: PoC & Spike** sebagai gate wajib |
| Audio processing berat (trim/encode/filter) | FFmpeg.wasm di browser/webview | **FFmpeg native dipanggil dari Rust (Tauri command)** |
| COOP/COEP multi-thread WASM | Perlu divalidasi per-platform | **Tidak relevan lagi** — FFmpeg jalan di proses native |
| Ukuran distribusi | ffmpeg-core.wasm ~25-30MB dibundle ke frontend | FFmpeg binary sebagai **Tauri sidecar**, terpisah per OS |
| Performa export | Terbatas WASM (single-thread by default) | Native, multi-thread otomatis |
| Preview vs export | Diasumsikan konsisten | Tetap pakai "Effect Parameter Contract" sebagai source of truth |
| Speed/Pitch | Tidak dijelaskan mekanismenya | SoundTouch.js (preview, tetap di JS) + `atempo` native (export, di Rust) |
| Format M4R | Dianggap format biasa | Step khusus rename + metadata, dieksekusi di Rust command |
| Memori file besar | Tidak dibahas | File besar diproses di Rust (di luar JS heap), bukan didecode penuh ke `AudioBuffer` |
| State management | Tidak eksplisit | Zustand store terstruktur (tetap di frontend) |
| Progress reporting | Tidak dibahas | **Baru:** stream progress FFmpeg dari Rust ke UI via Tauri event system |

---

## 1. Tech Stack (Final — Hybrid)

| Layer | Komponen | Teknologi | Catatan |
|---|---|---|---|
| Frontend | UI Framework | Vite + React + TypeScript | Tetap seperti plan awal |
| Frontend | Styling | Tailwind CSS + Lucide Icons | Tetap |
| Frontend | Waveform | WaveSurfer.js v7 + Regions Plugin | Cek kompatibilitas versi dengan React 18 |
| Frontend | State Management | Zustand | Region, effect params, playback state, progress export |
| Frontend | Preview Audio Engine | Web Audio API + SoundTouch.js (time-stretch) | Tetap di JS — preview real-time tidak butuh performa native |
| **Backend (native)** | **Runtime** | **Rust (Tauri v2 core)** | Command layer yang dipanggil dari React via `invoke()` |
| **Backend (native)** | **Audio Processing** | **FFmpeg binary sebagai Tauri sidecar** | Dipanggil via `std::process::Command`, bukan FFmpeg.wasm |
| **Backend (native)** | **(Opsional) Binding langsung** | crate `ffmpeg-next` | Alternatif jika ingin hindari spawn subprocess; lebih kompleks setup-nya |
| Bridge | Komunikasi | Tauri `invoke()` + Tauri Event System | React kirim perintah → Rust proses → stream progress balik ke React |
| Desktop Wrapper | Packaging | Tauri v2 | Tidak perlu konfigurasi COOP/COEP lagi |

**Prinsip pembagian kerja:** semua yang butuh *interaktivitas instan* (drag handle, preview cepat, live waveform) tetap di JS/Web Audio. Semua yang butuh *pemrosesan berat/final* (trim, filter, encode, convert format) dilempar ke Rust command yang memanggil FFmpeg native.

---

## 2. Fase 0 — PoC & Validasi Teknis (WAJIB)

Tujuan: membuktikan asumsi teknis berisiko tinggi *sebelum* investasi waktu ke UI penuh. Fokus utama sekarang bergeser dari "apakah WASM bisa jalan optimal" ke "apakah sidecar Rust ↔ FFmpeg native bisa jalan mulus lintas OS".

### 2.1 Validasi Tauri Sidecar + FFmpeg Native
- Bundling FFmpeg binary sebagai `externalBin` di `tauri.conf.json` untuk 3 target: Windows (`.exe`), macOS (Mach-O, perlu dua arch: `x86_64` & `aarch64` untuk Apple Silicon), Linux (ELF).
- Uji Rust command sederhana: terima path file dari frontend → panggil FFmpeg sidecar dengan argumen trim dasar → return path file output.
- **Exit criteria:** command berhasil dieksekusi dan file valid dihasilkan di ketiga OS.

### 2.2 Validasi Progress Streaming (Rust → React)
- FFmpeg menulis progress ke `stderr`. Rust command perlu parse output ini secara real-time (bukan tunggu proses selesai).
- Uji: stream progress via Tauri Event System (`app.emit()`) → React dengarkan event via `listen()` → update progress bar.
- **Exit criteria:** progress bar di UI update secara live selama proses export file besar (>5 menit durasi audio).

### 2.3 Validasi Ukuran & Packaging Instalasi
- Build installer dengan FFmpeg sidecar terbundle → catat ukuran final per OS.
- Bandingkan dengan estimasi awal (FFmpeg static build biasanya 40-70MB tergantung fitur yang di-include — bisa di-strip codec yang tidak dipakai untuk perkecil ukuran).
- Putuskan: pakai FFmpeg full build atau custom build minimal (hanya codec yang dibutuhkan: mp3, aac, flac, wav).

### 2.4 Validasi Speed/Pitch
- Spike: SoundTouch.js untuk preview real-time speed change tanpa pitch shift (tetap di JS, tidak berubah).
- Spike: uji filter `atempo` di FFmpeg native untuk rasio di luar 0.5–2.0 (perlu chaining, misal `atempo=2.0,atempo=1.5` untuk 3x) — dieksekusi dari Rust command.
- Bandingkan hasil preview (JS) vs hasil export (native) pada sample yang sama untuk memastikan tidak ada drift signifikan.

### 2.5 Validasi Format M4R
- Uji dari sisi Rust command: encode ke AAC dalam container M4A via FFmpeg native → rename ekstensi ke `.m4r`.
- Cek apakah cukup dikenali sebagai ringtone, atau perlu metadata tambahan (tag `stik` untuk iTunes) — bisa di-inject via `ffmpeg -metadata` atau post-process terpisah.

### 2.6 Validasi Signing/Notarization (khusus macOS)
- FFmpeg binary pihak ketiga yang di-bundle sebagai sidecar perlu ikut proses code-signing & notarization Apple, atau app akan diblokir Gatekeeper.
- Cek proses ini sejak awal karena bisa jadi blocker rilis, bukan sekadar detail teknis di akhir.

**Output Fase 0:** dokumen singkat `POC_FINDINGS.md` berisi keputusan final untuk tiap poin di atas, termasuk pilihan FFmpeg build (full vs minimal) dan status validasi signing macOS.

---

## 3. Fitur & Komponen UI

Tidak berubah signifikan dari plan awal, dengan tambahan detail:

### A. Upload / Landing Page
- Drag-and-drop + file picker native
- Format: `.mp3 .wav .m4a .aac .flac .ogg .wma`
- **Tambahan:** validasi ukuran file di sisi client, tampilkan warning jika file > threshold tertentu (misal 200MB) karena `AudioBuffer` penuh di memori bisa berat.

### B. Waveform & Workspace
- Interactive timeline waveform (region selection, draggable handles)
- Top toolbar: Trim, Volume/Gain, Speed & Pitch, Equalizer, Reset/Close
- Bottom dock: Play/Pause, Fade In/Out, numeric time input presisi, format selector, Save

**Tambahan — Effect Parameter Contract:**
Semua parameter efek (fade curve type, gain dB, speed ratio, EQ band) disimpan sebagai objek data tunggal di Zustand store, bukan langsung sebagai state Web Audio node. Objek ini menjadi source of truth yang di-translate ke:
1. Web Audio API graph (untuk preview)
2. FFmpeg filter string (untuk export)

Ini mencegah drift antara apa yang didengar user saat preview vs hasil file akhir.

---

## 4. Arsitektur Pemrosesan Audio (Hybrid)

```
[ Input Audio File ]
        │
        ▼
┌─────────────────────────────┐
│ FRONTEND (React/JS)          │
│                               │
│ Decode ringan utk waveform:  │
│ - Web Audio API decode       │
│   (hanya untuk visualisasi   │
│    & preview, bukan proses   │
│    final)                    │
└─────────────────────────────┘
        │
        ▼
┌─────────────────────────────┐
│ Effect Parameter Store       │  (Zustand, single source of truth)
│ (region, gain, fade, speed)  │
└─────────────────────────────┘
        │
   ┌────┴──────────────────┐
   ▼                        ▼
Preview                  Export (klik "Save")
(tetap di frontend)      │
- Web Audio API graph    ▼
- SoundTouch.js          Tauri invoke('export_audio', params)
  (speed/pitch)          │
                          ▼
              ┌───────────────────────────┐
              │ BACKEND (Rust, native)     │
              │                             │
              │ 1. Terima Effect Params     │
              │    dari frontend            │
              │ 2. Bangun FFmpeg filter     │
              │    graph (trim, fade,       │
              │    gain, atempo, format)    │
              │ 3. Spawn FFmpeg sidecar     │
              │ 4. Parse stderr → progress  │
              │    → emit event ke frontend │
              │ 5. Return path file output  │
              └───────────────────────────┘
                          │
                          ▼
        [ Native Save Dialog via Tauri ]
```

**Poin kunci:** file audio asli **tidak perlu di-load penuh ke JS heap** untuk proses export — Rust command bekerja langsung dari path file di disk. Ini yang menyelesaikan concern memori untuk file besar di plan sebelumnya.

---

## 5. Roadmap Revisi

### Fase 0: PoC & Spike (gate wajib, ~4-6 hari kerja)
- Lihat Section 2. Bertambah durasi dari plan sebelumnya karena sekarang mencakup validasi sidecar, signing macOS, dan progress streaming — lebih banyak moving parts dibanding validasi WASM murni, tapi risiko yang divalidasi juga lebih fundamental (blocker rilis, bukan cuma performa).

### Fase 1: Setup Proyek & UI Basic
- Init Vite + React + TS + Tailwind
- Init Tauri v2 project, setup struktur `src-tauri/` untuk Rust commands
- Struktur halaman upload dropzone
- Tema dark (Navy `#0f172a`, Cyan `#06b6d4`, Green `#10b981`)
- Setup Zustand store skeleton (UI state + effect params + export progress state)

### Fase 2: Waveform & Trimming Logic (Frontend murni)
- Integrasi WaveSurfer.js + Regions Plugin
- Start/End handle → sinkron ke Effect Store, bukan local state komponen
- Preview playback terbatas pada rentang region (Web Audio API)
- Input numerik presisi (MM:SS.ms) dua arah (drag handle ↔ input angka)
- Integrasi SoundTouch.js untuk preview speed/pitch

### Fase 3: Rust Backend — Audio Processing Command
- Setup FFmpeg sebagai Tauri sidecar (`externalBin`) untuk 3 platform
- Tulis Rust command `export_audio(params)`:
  - Terima Effect Params dari frontend (region, gain, fade, speed, target format)
  - Bangun argumen filter FFmpeg dari params tersebut
  - Spawn proses FFmpeg, parse `stderr` untuk progress
  - Emit event progress ke frontend via Tauri Event System
  - Return path file hasil setelah selesai
- Implementasi konversi format (MP3, WAV, AAC/M4A, FLAC, **M4R dengan step rename + metadata**)

### Fase 4: Integrasi Frontend ↔ Backend
- Hubungkan tombol "Save" di UI ke `invoke('export_audio', ...)`
- Render progress bar dari event yang di-emit Rust
- Error handling: FFmpeg gagal / file corrupt / format tidak didukung → tampilkan pesan jelas ke user, bukan silent fail

### Fase 5: Desktop Packaging & Release
- Native save dialog (Tauri `dialog` plugin)
- Code signing & notarization untuk macOS (termasuk FFmpeg sidecar binary)
- Build installer per OS (.exe, .dmg, .AppImage)
- Smoke test matrix — export end-to-end di ketiga OS sebelum rilis

---

## 6. Risiko Terbuka yang Masih Perlu Dipantau

- **Lisensi FFmpeg** — build FFmpeg yang di-bundle bisa terikat GPL atau LGPL tergantung codec yang di-include (misal `libx264`/beberapa encoder tertentu bersifat GPL). Perlu pilih build yang LGPL-compliant kalau aplikasi didistribusikan closed-source.
- **Update FFmpeg sidecar** — karena dibundle sebagai binary terpisah (bukan dependency npm), strategi update FFmpeg ke versi baru perlu proses build ulang & re-release aplikasi, bukan sekadar `npm update`.
- **Undo/redo** untuk multi-region editing belum masuk scope — perlu diputuskan apakah v1 hanya single-region trim, atau multi-clip dari awal.
- **Lisensi SoundTouch.js** — cek kompatibilitas lisensi (LGPL) dengan model distribusi aplikasi.
- **Rust learning curve tim** — kalau belum ada pengalaman Rust sama sekali, alokasikan waktu belajar dasar (ownership, error handling `Result<T,E>`, async dengan `tokio` untuk spawn process) sebelum Fase 3, supaya tidak jadi blocker mendadak.

---

## 7. Pertanyaan Terbuka untuk Diputuskan Sebelum Fase 0

1. Apakah v1 perlu support multi-region (potong beberapa bagian sekaligus) atau cukup single trim range?
2. Batas durasi/ukuran file maksimum yang di-support di v1?
3. Apakah undo/redo termasuk scope v1 atau v2?

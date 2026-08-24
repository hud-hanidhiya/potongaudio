# Scope Brief — Potong Audio

## Tujuan inti
Trim + efek dasar + konversi format audio, offline penuh, cross-platform
Windows & Linux.

## In scope (v1)
- Upload file audio (drag-drop + file picker native)
- Trim **single-region** (satu rentang start-end per file)
- Efek: gain (-20 s/d +20 dB), fade in/out, speed/pitch (opsi preserve pitch)
- Export ke: MP3, WAV, M4A, FLAC, M4R
- Preview real-time sebelum export
- Native save dialog, progress bar saat export, cancel di tengah proses

## Out of scope (sengaja, untuk sekarang)
- Multi-region trim, undo/redo, equalizer — semua resmi ditunda ke v2
- macOS (target v1: Windows x86_64 + Linux x86_64 saja)
- Code signing/notarization (ditunda ke fase rilis)

## Stack
- Frontend: Vite + React + TypeScript + Tailwind CSS + Zustand + WaveSurfer.js
- Backend: Rust (Tauri v2), FFmpeg **native** via sidecar (bukan FFmpeg.wasm)

## Constraints
- 100% offline saat runtime — tidak ada dependency server eksternal.
- Ukuran installer wajar untuk app offline (referensi: NSIS Windows 80MB;
  AppImage Linux 177MB masih diselidiki, lihat known risk di §04).
- FFmpeg yang dibundel: **build LGPL BtbN** (bukan GPL full) — dipilih
  karena v1 fokus murni trim/efek/convert audio, tidak butuh codec video
  GPL-only (`libx264`/`libx265`) yang jadi satu-satunya alasan signifikan
  sebuah build BtbN harus GPL. Lihat `00-guardrails.md` untuk detail
  keputusan dan kewajiban sanity-check codec setelah swap.

## Sistem/integrasi yang disentuh
- [x] FFmpeg/FFprobe (sidecar native, bukan library/API eksternal)
- [x] Filesystem lokal (baca file input, tulis file output)
- [ ] Tidak ada network call runtime
- [ ] Tidak ada database
- [ ] Tidak ada auth/user account

## Definition of done
- Bisa di-install di mesin bersih Windows & Linux.
- Alur penuh: upload → trim single-region → atur efek → export minimal
  3 format berbeda → file hasil valid dan bisa diputar, tanpa crash.
- Batas ukuran/durasi file maksimum v1 **masih terbuka** — lihat
  `03-spec.md` bagian Pertanyaan Terbuka.

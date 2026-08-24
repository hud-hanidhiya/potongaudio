# Project Guardrails — Potong Audio

> Potong Audio tidak punya uang/billing/auth/multi-user data seperti
> kebanyakan contoh guardrail generik. Risk trigger di sini spesifik
> domain audio processing offline + distribusi binary pihak ketiga.

## Risk Trigger (hal-hal yang mahal kalau salah)
- [x] **Urutan/matematika filter FFmpeg** — hasil audio yang salah
  (fade timing, trim boundary) sulit terdeteksi tanpa test yang sangat
  spesifik; user baru sadar setelah dengar file hasil export.
- [x] **Kontrak `EffectParams` (TypeScript ↔ Rust)** — setara "kontrak API"
  di project ini. Drift antar sisi tidak tertangkap oleh test masing-masing
  sisi sendiri-sendiri (test TS lulus, test Rust lulus, tapi keduanya
  salah paham field yang sama).
- [x] **Operasi file yang destruktif** — export bisa menimpa file lain
  (termasuk berpotensi file sumber) kalau output path tidak divalidasi.
- [x] **Lifecycle proses sidecar FFmpeg (spawn/cancel)** — kegagalan kill
  proses bisa menyisakan zombie process atau file output parsial/corrupt.
  (Sudah pernah kejadian: `CommandChild::kill()` by-value vs desain
  registry awal — lihat `07-debug-log.md`.)
- [x] **Supply chain binary pihak ketiga (FFmpeg/FFprobe)** — binary
  di-download saat build/CI, bukan di-commit. Checksum wajib diverifikasi.
- [x] **Lisensi FFmpeg build yang dibundel** — **pindah ke build LGPL**
  (bukan GPL full) mulai sekarang, karena scope v1 cuma audio trim/efek/
  convert. Tetap risk trigger karena kalau nanti butuh codec GPL-only,
  keputusan ini perlu direview ulang secara sadar, bukan drift diam-diam.

## Aturan untuk tiap trigger yang dicentang

**Urutan/matematika filter FFmpeg**
- Urutan filter WAJIB: `atrim` → `atempo` (speed) → `afade` → `volume`
  (gain). Ini bukan gaya penulisan — fade-out HARUS dihitung dari durasi
  SETELAH speed berubah, bukan durasi asli. Jangan ubah urutan ini tanpa
  menulis ulang test yang mengunci urutan (`urutan_filter_trim_sebelum_...`
  di `filter_builder.rs`).
- Setiap perubahan ke `filter_builder.rs` WAJIB disertai unit test baru
  untuk kasus yang diubah — jangan cuma andalkan test lama tetap lulus.

**Kontrak `EffectParams`**
- Field apapun yang ditambah/diubah di `src/types/audio.types.ts` WAJIB
  disinkronkan di saat yang sama ke `src-tauri/src/commands/export.rs`
  (dan sebaliknya). Satu commit, bukan dua commit terpisah.
- Naming: `camelCase` di TS, `snake_case` di Rust, di-mapping otomatis
  lewat `#[serde(rename_all = "camelCase")]` — jangan bikin mapping manual.

**Operasi file destruktif**
- Output path export WAJIB berasal dari native save dialog (user pilih
  eksplisit), TIDAK BOLEH auto-overwrite file source tanpa konfirmasi.
- Kalau output path == input path, tolak atau minta konfirmasi eksplisit
  — jangan diam-diam ditimpa.

**Lifecycle proses sidecar**
- Semua kode yang menyimpan handle proses (`CommandChild`/`Child`) WAJIB
  punya test cancel yang benar-benar membunuh proses (pakai fake script
  fixture yang sengaja jalan lama, seperti pola `long_ffmpeg.sh`/`.bat`
  yang sudah ada) — bukan cuma test happy-path yang proses selesai sendiri.

**Supply chain binary**
- Download binary FFmpeg/FFprobe apapun (lokal maupun CI) WAJIB verifikasi
  checksum resmi (`checksums.sha256` dari rilis BtbN) sebelum dipakai.
  Gagal keras kalau checksum tidak cocok — jangan fallback diam-diam.

**Lisensi FFmpeg**
- **Keputusan (diperbarui):** pakai build **LGPL** BtbN (bukan GPL full)
  sebagai default sekarang, karena scope v1 murni audio (trim + fade/gain/
  speed + convert) — TIDAK butuh `libx264`/`libx265` (dua library yang
  jadi alasan utama sebuah build harus GPL, bukan LGPL, per dokumentasi
  resmi BtbN: "Lacking libraries that are GPL-only. Most prominently
  libx264 and libx265").
- Codec audio yang dibutuhkan v1 (mp3 via `libmp3lame`, aac/m4a native
  FFmpeg, flac, wav) semuanya LGPL-compatible dan tetap ada di build LGPL.
- **Wajib sanity-check setelah swap**: jalankan `ffmpeg -codecs` /
  `-encoders` pada build LGPL yang di-download, konfirmasi `libmp3lame`,
  `aac`, `flac` benar-benar ada sebelum dipakai — dokumentasi BtbN pakai
  frasa "most prominently" (menyiratkan kemungkinan ada library GPL-only
  lain yang ikut hilang, tidak dirinci lengkap). Jangan asumsikan 100%
  sama persis dengan build GPL dikurangi x264/x265 saja.
- **Koreksi catatan lama**: proyeksi "penghematan ukuran ~50%+" di
  `PLAN_AUDIO_CUTTER.md` sebelumnya adalah SPEKULASI, belum terverifikasi.
  Angka resmi terbaru (rilis BtbN "latest", win64 static): GPL 162.8MB vs
  LGPL 141.6MB — selisih riil **~13%**, jauh lebih kecil dari perkiraan
  lama. Jangan pakai angka lama sebagai acuan lagi.
- Kalau nanti butuh codec tambahan yang ternyata GPL-only, keputusan ini
  harus direview ulang — jangan diam-diam kembali ke GPL tanpa update
  dokumen ini.

## Verification hooks
- `cargo test` — mencakup `filter_builder.rs` (16 test), `sidecar.rs`
  (5 test termasuk cancel), `progress_parser.rs` (9 test).
- CI (`.github/workflows/`) — verifikasi checksum FFmpeg otomatis sebelum
  build, di kedua OS.
- Belum ada: test otomatis untuk kesetaraan kurva fade preview (Web Audio
  API) vs export (FFmpeg `afade`) — masih manual, dicatat sebagai risiko
  terbuka di `04-architecture-notes.md`.

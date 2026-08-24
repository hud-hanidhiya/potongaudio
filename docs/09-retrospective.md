# Retrospective — Potong Audio (Fase 0–5, MVP v1.0)

> Retro ini mencakup dari rancangan awal sampai MVP v1.0 (Fase 0–5) ditutup
> (2026-08-24). Update lagi setelah Fase berikutnya / project di-ship lanjut /
> project di-abandon.

## Apa yang berjalan efektif
- Disiplin "tulis lalu buktikan jalan" dipegang konsisten — modul Rust
  kritis (`filter_builder`, `progress_parser`, `sidecar`) ditulis DENGAN
  unit test yang benar-benar dijalankan. Ini membuat PoC Fase 0 lolos
  relatif cepat begitu sidecar sungguhan disambungkan.
- Review arsitektur di awal (sebelum nulis kode) menyelamatkan dari
  pilihan FFmpeg.wasm yang bermasalah.
- Pin versi FFmpeg identik di Windows & Linux CI membuat hasil PoC lintas
  OS benar-benar sebanding.

## Apa yang memperlambat pengerjaan
- Keterbatasan sandbox pembuatan skeleton awal (tidak bisa compile Tauri
  v2 penuh) membuat sebagian kode command layer baru tervalidasi saat
  integrasi sungguhan di Fase 0 — gap `CommandChild` by-value baru ketahuan
  di titik itu.
- Struktur project bergeser beberapa kali tanpa semua dokumen ter-update
  serempak — risiko dokumen usang dipercaya begitu saja.
- Rename branding sempat berubah arah (Potong-Audio berhyphen →
  PotongAudio) — keputusan branding sebaiknya difinalkan lebih awal.

## Keputusan yang akan diulang
- Dependency Tauri dibuat `optional` via feature flag — `cargo test` tetap
  cepat tanpa runtime GUI.
- Fake script (`fake_ffmpeg.sh`/`.bat`) untuk test yang butuh spawn proses
  eksternal, alih-alih skip test atau butuh binary asli di CI.
- Menandai eksplisit status verifikasi (✅/⚠️) di README.

## Keputusan yang akan diubah
- Cek dari awal apakah lingkungan development punya toolchain lengkap
  untuk compile Tauri penuh, supaya tidak ada fase "belum tervalidasi"
  yang menyembunyikan bug integrasi.
- Finalisasi nama/branding SEBELUM menyebar ke banyak file.

## Bagian yang bisa dipakai ulang untuk proyek berikutnya
- Pola `Killable`/`Mutex<Option<T>>` untuk API by-value yang perlu
  disimpan di struct/registry.
- Struktur `JobRegistry` + event streaming progress (`jobId` untuk
  filtering) — generik untuk task async berat lain via Tauri.
- Workflow CI yang download+verifikasi checksum binary eksternal sebelum
  dipakai.
- Kit workflow ini sendiri (sekarang versi Universal 2-tier) — diterapkan
  sebagai retrofit, membantu menyatukan histori keputusan yang sebelumnya
  tersebar di banyak dokumen ad-hoc.

## Fase 1–5 — MVP v1.0 (2026-08-24)

Periode ini membawa project dari skeleton Fase 0 ke aplikasi utuh yang bisa
di-rilis: UI (WaveSurfer + region trim), wiring export backend↔frontend,
dan pipeline rilis otomatis.

### Yang berjalan efektif
- **Satu sumber kebenaran `EffectParams`** (Zustand `useAudioStore`) dipakai
  baik oleh preview (Web Audio API) maupun payload export ke Rust — kontrak
  TS↔Rust disinkronkan via `serde(rename_all = "camelCase")`, tanpa mapping
  manual.
- **WaveSurfer.js + RegionsPlugin** untuk region trim: drag handle dua-arah
  sinkron dengan `TimeInput` (guard toleransi 5ms cegah loop event). File audio
  dibaca SEKALI (`plugin-fs` `readFile`) lalu dipakai untuk waveform (blob URL)
  dan decode preview.
- **Swap FFmpeg GPL→LGPL** (build BtbN, tag sama) tanpa mengubah urutan filter
  maupun kontrak apa pun — hanya suffix asset build. Sanity-check codec
  (`libmp3lame`/`aac`/`flac`) jadi gate wajib sebelum binary dianggap layak.
- **AC-04 ditolak eksplisit** di frontend (`useExportStore.startExport`) sebelum
  IPC — source tidak pernah ke-overwrite diam-diam.
- **Pipeline rilis additive** di `build-verify.yml`: tag `v*` → job
  `publish-release` mempublikasikan GitHub Release dari artifact yang sudah
  di-build (tanpa mengubah job verifikasi yang sudah hijau).

### Keputusan / catatan
- **Speed/pitch**: `atempo` di export tetap mengubah pitch; preservasi pitch
  butuh `rubberband` (GPL, tidak ada di build LGPL) — sengaja ditunda, bukan
  bug. Preview speed juga belum ada (butuh library time-stretch).
- **Installer rilis unsigned** (T0.7 code signing ditunda) — di Windows muncul
  peringatan SmartScreen.
- **Anomali ukuran AppImage** (177MB vs NSIS 80MB) belum diinvestigasi.

## Backlog / ide v2
- Multi-region trim, undo/redo, equalizer (resmi di-scope ke v2).
- Support macOS.
- Investigasi anomali ukuran AppImage (177MB vs 80MB NSIS).
- Perbandingan FFmpeg build full vs minimal (T0.6) — sudah diputuskan pakai
  LGPL off-the-shelf, selisih riil ~13% (bukan ~50% seperti proyeksi lama).
- Code signing Windows/macOS (T0.7 / rilis).
- Cleanup otomatis file output parsial saat cancel/error (ditemukan saat
  menulis Failure Mode di `04-architecture-notes.md` — belum ada di task
  plan sebelumnya).
- Preservasi pitch export + preview speed (time-stretch library).
- DEFERRED dari review v1.0 (`review.md`, 2026-08-24): clamp fade-in preview
  (M4), satukan AudioContext (M5), hint preview speed + hapus dead code
  `subscribeExportEvents` (M6), unregister job saat channel tutup + timeout
  FFmpeg hang (M8), pengetatan scope fs `**` + duplikasi scope L7 (H4 langkah 2),
  peaks dari AudioBuffer agar tidak decode dua kali (L1), prettier.

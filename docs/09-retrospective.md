# Retrospective — Potong Audio (Fase 0)

> Retro ini mencakup periode dari rancangan awal sampai Fase 0 ditutup
> (2026-08-21). Update lagi setelah Fase 1 selesai / project di-ship /
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

## Backlog / ide v2
- Multi-region trim, undo/redo, equalizer (resmi di-scope ke v2).
- Support macOS.
- Investigasi anomali ukuran AppImage (177MB vs 80MB NSIS).
- Perbandingan FFmpeg build full vs minimal (T0.6).
- Code signing Windows/macOS (T0.7 / rilis).
- Cleanup otomatis file output parsial saat cancel/error (ditemukan saat
  menulis Failure Mode di `04-architecture-notes.md` — belum ada di task
  plan sebelumnya).

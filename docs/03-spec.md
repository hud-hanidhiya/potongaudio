# Spec — Potong Audio

## Fitur

| # | Fitur | Prioritas | Catatan |
|---|---|---|---|
| 1 | Upload file audio (drag-drop) | Must | Native `tauri://file-drop` — status implementasi dicek ulang di kickoff Fase 1 |
| 2 | Upload file audio (file picker) | Must | Via Tauri dialog plugin |
| 3 | Probe metadata file | Must | `probe_audio_file` — 7 unit test lulus |
| 4 | Render waveform interaktif | Must | WaveSurfer.js — Fase 2, belum dimulai |
| 5 | Trim single-region | Must | Bukan multi-clip |
| 6 | Input waktu presisi (MM:SS.ms) | Must | `TimeInput.tsx` |
| 7 | Preview playback region | Must | `previewEngine.ts` |
| 8 | Gain adjustment | Must | -20 s/d +20 dB |
| 9 | Fade in/out | Must | Lihat guardrail urutan filter |
| 10 | Speed/pitch + preserve-pitch | Must | Preview butuh library time-stretch (belum dipilih) |
| 11 | Export MP3/WAV/M4A/FLAC/M4R | Must | `filter_builder.rs` — 16 test |
| 12 | Progress bar export | Must | `ProgressTracker` + event streaming |
| 13 | Cancel export | Must | `JobRegistry`/`Killable` |
| 14 | Native save dialog | Must | |
| 15 | Multi-region trim | Nice (v2) | Ditunda |
| 16 | Undo/redo | Nice (v2) | Ditunda |
| 17 | Equalizer | Nice (v2) | Ditunda |

## User flow
1. Buka aplikasi → layar upload.
2. Pilih/drop file → probe metadata → workspace.
3. Drag handle waveform untuk set region trim.
4. (Opsional) atur gain/fade/speed, dengar preview.
5. Pilih format output → Save → native dialog → progress bar.
6. Selesai/cancel → notifikasi status.

## Data flow & transisi status
Status export: `idle → running → done | error | cancelled`. Tidak ada
status yang bisa "setengah jalan" tersimpan permanen — kalau proses
terputus (cancel atau crash), file output parsial harus dianggap tidak
valid (lihat guardrail lifecycle proses). Tidak ada retry otomatis untuk
export gagal — user harus klik Save lagi secara eksplisit.

## Error handling
- **Technical error** (FFmpeg exit non-zero, disk penuh, sidecar gagal
  spawn): `AppError` terpusat, pesan Bahasa Indonesia, tampilkan ke user,
  JANGAN retry otomatis diam-diam.
- **Business/logic error** (region invalid, file bukan audio, format tidak
  didukung): divalidasi sebelum sampai ke FFmpeg kalau memungkinkan
  (`filter_builder.rs` sudah validasi region), pesan spesifik per kasus.

## Edge case yang harus ditangani
- Region end/start terbalik atau di luar durasi → ditolak (sudah teruji).
- Fade out lebih panjang dari durasi region → di-clamp, bukan crash.
- Speed ratio di luar rentang native FFmpeg (0.5–2.0) → chaining `atempo`.
- File corrupt/format tidak didukung → pesan error jelas, bukan crash.
- Disk penuh / gagal tulis → `AppError::OutputWriteFailed`.
- Output path == input path → **belum ditangani eksplisit**, lihat
  guardrail "Operasi file destruktif" — perlu ditambah validasi.
- File sangat besar → belum ada batas eksplisit, lihat Pertanyaan Terbuka.

## Acceptance criteria

**AC-01 — Trim dasar**
- Given: file audio 60 detik sudah dimuat
- When: user set region 10s–20s, klik Save (format MP3)
- Then: file output berdurasi 10 detik ± toleransi kecil, bisa diputar

**AC-02 — Cancel export**
- Given: export sedang berjalan
- When: user klik Cancel
- Then: proses FFmpeg berhenti, tidak ada file output parsial tertinggal,
  tidak ada zombie process, UI kembali ke idle

**AC-03 — Fade out melebihi durasi**
- Given: region 3 detik, fade out diset 10 detik
- When: export dijalankan
- Then: tidak crash; fade dimulai dari awal region (clamped)

**AC-04 — Output path bentrok dengan input (didefinisikan & diimplementasi)**
- Given: user pilih output path yang sama dengan file sumber
- When: klik Save
- Then: export DITOLAK sebelum IPC/FFmpeg dijalankan, pesan error Bahasa
  Indonesia ("Path output tidak boleh sama dengan file sumber…"). Keputusan
  2026-08-24: tolak otomatis, bukan dialog konfirmasi. Ter-cover unit test
  `useExportStore.test.ts`.

## Pertanyaan terbuka
- Batas ukuran/durasi file maksimum v1 — belum diputuskan.
- Library time-stretch untuk preview speed — belum divalidasi.
- Perilaku saat output path == input path (AC-04) — **diputuskan 2026-08-24**:
  DITOLAK otomatis (pre-flight di `useExportStore.startExport`, case-insensitive
  hanya untuk path gaya Windows); guardrail "Operasi file destruktif" terpenuhi.
- Preservasi pitch export butuh `rubberband` (GPL) yang tidak ada di build
  LGPL — checkbox UI dinonaktifkan sampai library time-stretch diputuskan.

## Backlog / DEFERRED (dari code review v1.0, jangan dikerjakan tanpa persetujuan)

Fitur & perbaikan yang ditunda agar fokus pada rilis v0.1.0 bersih:
- **M4**: Clamp fade-in di `previewEngine.ts` (fade-out sudah di-clamp; fade-in
  yang lebih panjang dari durasi region bisa menyebabkan urutan automation
  gain ambigu antar-browser).
- **M5**: Satukan `AudioContext` — `WaveformView.handlePlayToggle` masih
  membuat context baru, seharusnya pakai shared context di `audioDecode.ts` +
  `void ctx.resume()` sebelum play (autoplay policy webview).
- **M6**: Preview speed = no-op diam; tambah hint "preview speed belum
  tersedia" + hapus dead code `subscribeExportEvents` di `ipc.ts`
  (tidak di-import siapa pun).
- **M8**: Lifecycle job `export_audio` — unregister saat channel tertutup
  tanpa `Terminated` (`rx.recv()→None` bocor registry) + timeout untuk
  FFmpeg yang hang (user hanya bisa Cancel manual).
- **H4 langkah 2**: Perketat scope `fs:read-file` — ganti `allow: ["**"]`
  ke pattern per-file yang dipilih user (capability dinamis) atau route
  pembacaan bytes lewat command Rust khusus.
- **L1**: Decode file dua kali (Web Audio + WaveSurfer internal). Optimasi
  v1.1: kirim `peaks` hasil compute dari `AudioBuffer`.
- **L2**: `height: 96` WaveSurfer vs container `h-32` (128px) — waveform
  tidak mengisi box.
- **L3**: `Dropzone.tsx` file-drop tidak memvalidasi ekstensi.
- **L4**: `ExportDock.handleSave` tanpa try/catch.
- **L7**: Scope fs duplikat (`tauri.conf.json plugins.fs.scope` + capability
  `fs:read-file.allow`) — pilih satu sumber kebenaran.
- **L8**: Kontrak dobel pada sukses export — `emit("export://done")` +
  return `ExportResult`; frontend hanya pakai return value.
- **Prettier** (bukan eslint — formatting terpisah).
- **Batas ukuran/durasi file v1** — belum diputuskan.
- **Library time-stretch akhir** untuk preview speed & preserve-pitch export.

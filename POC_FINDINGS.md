# POC_FINDINGS.md — Hasil Proof of Concept Fase 0 (PotongAudio)

> Dokumen ini menjawab 8 pertanyaan PoC secara eksplisit (ya/tidak) dengan
> bukti yang diambil langsung dari eksekusi di Windows x86_64 (build host
> Fase 0). Tanggal eksekusi: 2026-08-20.

---

## 1. Sidecar FFmpeg bisa di-spawn di Windows x86_64?

**YA.**

- BtbN gpl build (win64) di-download dan ditempatkan di
  `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe`.
- `tauri build` me-rename sidecar ke `target/release/ffmpeg.exe` (konvensi
  Tauri `externalBin`) — diverifikasi file ada dan versinya:
  `ffmpeg version N-126217-ge1e325235e-20260819` (built 2026-08-19).
- `ffmpeg -version` jalan dari prompt dengan exit 0.
- Aplikasi hasil build (`potong-audio.exe`) sukses diluncurkan dan bertahan
  running (smoke test), membuktikan runtime Tauri + WebView2 tidak crash.
- Command baru `get_ffmpeg_version` (spawn `sidecar("ffmpeg") -version`,
  return string) sudah di-wire di `lib.rs` dan frontend menampilkan versinya
  di header (`src/App.tsx`).
- Terverifikasi ulang via CI: job `build-windows` mengunduh FFmpeg/FFprobe
  Windows (BtbN `win64-gpl`, tag `autobuild-2026-08-19-19-21`), verifikasi
  `checksums.sha256`, jalankan `-version` (exit 0), dan `cargo test` 39 passed
  di Windows (termasuk `fake_ffmpeg.bat`).

## 2. Sidecar FFmpeg bisa di-spawn di Linux x86_64?

**YA.** (terverifikasi via CI, 2026-08-21)

- Job `verify-linux-build` (ubuntu-22.04) mengunduh BtbN `linux64-gpl`
  (`ffmpeg-N-126217-ge1e325235e-linux64-gpl.tar.xz`, tag sama dengan Windows
  `autobuild-2026-08-19-19-21`), verifikasi `checksums.sha256`, lalu
  `ffmpeg -version` & `ffprobe -version` jalan (exit 0) di runner.
- `cargo test` (default features) 39 passed DI LUAR Tauri — termasuk 3 test yang
  me-spawn `fake_ffmpeg.sh` (bukti spawn sidecar jalan di Linux, bukan cuma Windows).
- `cargo tauri build` menghasilkan AppImage + `.deb` (lihat item #6).
- Kode spawn OS-agnostic (`app.shell().sidecar(...)`), risiko residual rendah —
  kini sudah dibuktikan di kedua OS.

## 3. Sidecar FFprobe bisa di-spawn di kedua OS?

**YA (Windows).** `ffprobe-x86_64-pc-windows-msvc.exe` di-rename Tauri menjadi
`target/release/ffprobe.exe`; `ffprobe -version` jalan dengan exit 0.
Parsing JSON (`parse_ffprobe_json` di `probe.rs`) teruji 7 unit test.

**Linux: YA.** (terverifikasi via CI) — binary FFmpeg/FFprobe Linux di-download,
diverifikasi checksum, dan `-version` jalan di job `verify-linux-build`;
`parse_ffprobe_json` (probe.rs) teruji 7 unit test dan `run_export` tests
memakai fixture di Linux lolos.

## 4. Progress streaming live ke UI?

**YA.**

- Trim sungguhan file 5 menit memakai filter chain `filter_builder` +
  `-progress pipe:2` menghasilkan 4 titik `out_time_us` sebelum `progress=end`
  (98.06s → 181.79s → 262.01s → 280.00s), membuktikan FFmpeg streaming
  progress ke stderr selama proses (bukan hanya di akhir).
- `ProgressTracker` memetakan `out_time_us` → persen dan meng-throttle emit
  (9 unit test termasuk simulasi stream end-to-end `[10,50,100,Done]`).
- `sidecar::run_export` + `export_audio` meng-emit `export://progress` /
  `export://done` / `export://error` (verified via test fake ffmpeg
  cross-platform).

## 5. Trim end-to-end menghasilkan file valid?

**YA.**

- Sample 10s WAV (lavfi sine 440Hz) di-trim region 2000–5000ms memakai
  argumen persis yang dihasilkan `build_filter_plan` + `build_args`:
  `atrim=start=2000ms:end=5000ms,asetpts=PTS-STARTPTS` → output MP3 3.000000s,
  durasi sesuai region (ffprobe verifikasi).
- Unit test `run_export_sukses_...` memverifikasi alur penuh (spawn →
  progress → file output → unregister job) dengan fake ffmpeg.
- Flow cancel: `cancel_menghentikan_proses_yang_sedang_berjalan` lulus di
  Windows dengan fixture `.bat`.

## 6. Keputusan FFmpeg build (full vs minimal) dengan ukuran

| Item | OS | Ukuran |
|---|---|---|
| `ffmpeg` (BtbN gpl full, win64, 20260819) | Windows | 139.05 MB |
| `ffprobe` (BtbN gpl full, win64) | Windows | 138.85 MB |
| `ffmpeg` (BtbN gpl full, linux64) | Linux | ~139 MB |
| `ffprobe` (BtbN gpl full, linux64) | Linux | ~139 MB |
| `potong-audio.exe` (release, stripped) | Windows | 4.44 MB |
| `PotongAudio_0.1.0_x64-setup.exe` (NSIS) | Windows | 80.13 MB (84.012.300 byte — terkonfirmasi via artifact CI `build-windows`) |
| `PotongAudio_0.1.0_amd64.AppImage` | Linux | 177 MB |
| `PotongAudio_0.1.0_amd64.deb` | Linux | 114 MB |

**Keputusan sementara: BtbN gpl full** untuk Fase 0/1 karena:
- Satu build mencakup semua format yang dibutuhkan v1
  (mp3, wav, m4a/aac, flac, m4r) + codec lama (ogg, wma) tanpa pekerjaan
  per-konfigurasi.
- Installer 80 MB (terkompresi NSIS) masih dalam ambang wajar untuk
  aplikasi offline.
- **Catatan untuk T0.4 lanjutan**: bandingkan dengan build BtbN `gpl-shared`
  atau minimal (hanya libmp3lame/aac/flac) kalau ukuran installer jadi
  masalah di distribusi; proyeksi penghematan signifikan (~50%+) tapi
  butuh konfigurasi FFmpeg custom. Keputusan final ditulis di sini setelah
  perbandingan dilakukan di host Linux.

## 7. Status code signing

- **Windows**: ditunda ke Fase 5 (T5.1) — installer NSIS tidak ditandatangani
  di Fase 0. SmartScreen akan memperingatkan saat distribusi publik.
- **macOS**: di luar scope Fase 0 (target Windows + Linux x86_64 saja).

## 8. Rekomendasi lanjut ke Fase 1

**YA.**

Sekarang SELURUH PoC kritis Fase 0 lulus di Windows DAN Linux (via CI workflow
`Verifikasi Build Linux & Windows (Fase 0 PoC)`), termasuk runtime Linux
(AppImage + `.deb` build sukses, smoke test lewat xvfb, `cargo test` 39 passed
termasuk spawn `fake_ffmpeg.sh`). Sisa pekerjaan di luar Fase 0: code signing
(Fase 5, T5.1) dan perbandingan build FFmpeg full vs minimal (T0.4).

---

## Catatan tambahan hasil implementasi Fase 0

- Struktur modul diperbaiki: `commands/mod.rs` = `export`, `probe`, `version`;
  `ffmpeg/mod.rs` = `filter_builder`, `progress_parser`, `sidecar`.
- `tauri.conf.json`: trailing comma dihapus; `build.features = ["tauri-runtime"]`
  ditambahkan supaya `cargo tauri build` mengaktifkan feature tanpa flag manual.
- Config frontend dipindah `src/` → root; `vite.config.ts` perlu
  `root: 'src'` + `build.outDir: '../dist'` (kompensasi root non-default),
  dan `src/index.html` script src diubah `/src/main.tsx` → `/main.tsx`.
- Bug pre-existing diperbaiki: import `StatusBadge.tsx`
  (`../../types` → `../types`).
- `fake_ffmpeg.sh` dipindah ke `test-fixtures/`; `fake_ffmpeg.bat` dibuat
  setara (cek `--fail`, output path = argumen terakhir, 3× progress,
  `progress=end`, buat file output).
- `Killable` untuk `CommandChild` memakai `Mutex<Option<CommandChild>>`
  karena API tauri-plugin-shell `CommandChild::kill(self)` by-value.
- Test cancel cross-platform (`.bat`/`.sh`).
- Non-blocking, dilaporkan saja (di luar scope Fase 0): `landing/index.html`
  masih menyebut "License: MIT" dan branding "PotongAudio" (tanpa hyphen).

# Code Review — PotongAudio (Senior Developer Review)

> Tanggal: 2026-08-24 · Scope: seluruh kode aktif v1.0 (Fase 0–5)
> Fokus: correctness, keamanan, UX honesty, arsitektur, test gap, CI/supply chain.
> Referensi memakai `file` + simbol (fungsi/struct), bukan nomor baris, agar
> tahan terhadap pergeseran edit.

---

## Ringkasan Eksekutif

**Verdict: LAYAK rilis internal/beta, dengan 4 temuan High yang sebaiknya
diperbaiki dulu (total estimasi kecil, hitungan jam).** Fondasi engineering
kuat: modul risiko (filter math, proses lifecycle, parsing progress) semua
dikunci unit test, kontrak TS↔Rust tunggal, supply-chain binary diverifikasi
checksum, dan CI mem-verifikasi dua OS secara apple-to-apple. Kekhawatiran
terbesar bukan di core processing, melainkan di **lapisan UI/promise**:
ada fitur yang ditampilkan tapi tidak benar-benar bekerja (`preserve_pitch`),
beberapa bug state saat ganti file, dan satu celah validasi region yang bisa
menghasilkan output tidak sesuai ekspektasi user tanpa error.

---

## Temuan HIGH (fix sebelum tag rilis)

### H1 — Checkbox "Preserve pitch" adalah fitur mati (UX honesty)
- **Bukti:** `Speed.preserve_pitch` di-deserialize di `commands/export.rs`
  (`pub preserve_pitch: bool`) tetapi `ffmpeg/filter_builder.rs::build_filter_plan`
  **tidak pernah membacanya** — `atempo` selalu mengubah pitch. Checkbox di
  `Toolbar.tsx` aktif dan tampak berfungsi.
- **Dampak:** user centang preserve-pitch, export mengubah pitch — janji UI
  yang dilanggar. Ini bug tipe paling mahal: tidak crash, tapi merusak trust.
- **Fix minimum:** disable checkbox + tooltip "butuh rubberband (GPL), belum
  dibundel di build LGPL" (sesuai catatan di README/retro). Fix ideal:
  implement time-stretch (butuh keputusan library, lihat backlog).

### H2 — State basi di `WaveformView` saat ganti file (3 bug sekaligus)
Di `useEffect([loadedFile])` (`WaveformView.tsx`) tidak ada reset state di awal:
1. `loadError` tidak di-reset → jika file sebelumnya gagal dimuat, pesan error
   lama **tetap tampil** di atas waveform file baru yang sukses.
2. `decoded` tidak di-null-kan → selama decode file baru berjalan, tombol Play
   masih aktif dan **memutar audio buffer file LAMA**.
3. `isPlaying` tidak di-reset → cleanup menghentikan preview, tapi label tombol
   mentok di "Pause".
- **Fix:** di awal effect: `setLoadError(false); setDecoded(null);
  setIsPlaying(false);` (sebelum async load).

### H3 — Region melebihi durasi file tidak divalidasi di mana pun
- **Bukti:** `filter_builder.rs::validate_params` hanya menolak
  `end_ms <= start_ms`. Frontend juga tidak clamp nilai store (`TimeInput`
  bebas mengetik end > durasi; `syncRegionToWaveSurfer` hanya clamp tampilan
  region WaveSurfer, bukan nilai store).
- **Dampak:** `atrim=end=Xms` dengan X > durasi menghasilkan output sampai
  akhir file — durasi hasil ≠ permintaan user, tanpa warning; persen progress
  (dihitung dari `total_duration_ms` region) bisa tak pernah capai 100%.
- **Fix:** (a) clamp/validasi di `setRegion` store terhadap
  `probe.durationMs`; (b) defense-in-depth: validasi di Rust — informasi
  durasi sudah dikirim sebagai `total_duration_ms`, tolak `end_ms` yang
  melebihinya (atau clamp eksplisit + log).

### H4 — Capability filesystem terlalu luas + CSP mati (hardening)
- **Bukti:** `capabilities/default.json` memberi `fs:read-file` dengan
  `allow: [{ "path": "**" }]`, dan `tauri.conf.json` `"csp": null`.
- **Dampak:** webview dapat membaca **semua** file di disk. Model ancaman
  aktual rendah (app offline, tanpa remote content), tapi ini permukaan
  serangan besar jika suatu saat ada konten/pihak ketiga di webview.
- **Fix bertahap:** (1) aktifkan CSP default Tauri; (2) idealnya persempit
  scope fs per-file yang dipilih user (pola capability dinamis), atau baca
  bytes lewat command Rust khusus yang memvalidasi input.

---

## Temuan MEDIUM

- **M1 — Stale closure di sinkronisasi region awal.**
  `WaveformView.syncRegionToWaveSurfer` membaca `effectParams` dari closure
  render, dan dipanggil dari callback `ws.on('ready')` yang dibuat di render
  pertama. Handler drag sudah benar pakai `useAudioStore.getState()`, tapi
  fungsi sync belum. → Baca region via `getState()` juga di sini.
- **M2 — AC-04 compare lowercase salah arah di Linux.**
  `useExportStore.startExport` membandingkan `toLowerCase()` kedua path.
  Di filesystem case-sensitive, `Song.MP3` vs `song.mp3` adalah file berbeda
  yang sah → false-reject. → Normalisasi case hanya di Windows, atau
  bandingkan path ternormalisasi (canonicalize) per OS.
- **M3 — Race pada cancel.** `useExportStore.cancel` menyetel
  `status:'cancelled'` **sebelum** `ipcCancelExport` berhasil; jika invoke
  gagal, UI bilang cancelled padahal FFmpeg masih jalan. Return value
  `cancel_export` (bool) diabaikan. → Set 'cancelled' hanya setelah IPC sukses;
  tampilkan error jika gagal.
- **M4 — Edge automation gain di `previewEngine`.** Jika `fadeInSec ≥ durasi`
  region, `setValueAtTime(targetGain, safeStart≈now)` dijadwalkan SETELAH
  ramp fade-in yang lebih panjang — urutan event automation jadi ambigu
  antar-browser. → Clamp `fadeInSec = min(fadeInSec, durationSec)` seperti
  yang dilakukan sisi FFmpeg.
- **M5 — Dua `AudioContext`.** `audioDecode.ts` punya shared context, tapi
  `WaveformView.handlePlayToggle` membuat context sendiri. Boros dan
  inkonsisten; juga belum ada `resume()` untuk autoplay-policy webview.
  → Pakai shared context + `void ctx.resume()` sebelum play.
- **M6 — Preview speed = no-op diam.** `needsTimeStretch(ratio)` membuat
  tombol Play tidak melakukan apa pun untuk speed ≠ 1× tanpa feedback apa pun
  ke user (stub `lib/soundtouch.ts`). Minimal tampilkan hint "preview speed
  belum tersedia". Juga `subscribeExportEvents` di `ipc.ts` dead code
  (komentarnya mengaku dipakai `useExportStore`, faktanya tidak).
- **M7 — Tidak ada lint gate sama sekali.** Tanpa clippy/rustfmt (Rust) dan
  eslint/prettier (TS), baik lokal maupun CI. Untuk project yang mengandalkan
  disiplin guardrail, ini celah murah untuk ditutup.
- **M8 — Lifecycle job di `export_audio`.** Jika channel tertutup tanpa
  `Terminated` (`rx.recv()` → `None`), job tidak di-unregister (leak registry),
  dan tidak ada timeout untuk FFmpeg yang hang (user hanya bisa Cancel manual).

---

## Temuan LOW / Nit

- **L1** File di-decode dua kali (Web Audio + decoder internal WaveSurfer).
  Optimasi v1.1: kirim `peaks` hasil compute dari `AudioBuffer` +
  blob URL sebagai media — hemat satu decode penuh.
- **L2** `height: 96` WaveSurfer vs container `h-32` (128px) — waveform
  tidak mengisi box, cosmetik.
- **L3** `Dropzone.tsx`: jalur file-drop tidak memvalidasi ekstensi
  (`SUPPORTED_EXTENSIONS` hanya memfilter dialog) — non-audio lolos ke probe
  lalu gagal dengan error generik. Pre-check ekstensi = UX lebih baik.
- **L4** `ExportDock.handleSave` tanpa try/catch — rejection dari
  `pickSaveLocation` menjadi unhandled.
- **L5** `outputBitrateKbps: 192` default juga untuk wav/flac (lossless
  mengabaikannya) — tidak salah, tapi kontrak jadi membawa field kosong makna.
- **L6** Snippet Development di README mencampur direktori: `npm run
  setup:ffmpeg` dan `cargo tauri dev` ditulis setelah `cd src-tauri`.
- **L7** `tauri.conf.json plugins.fs.scope` duplikat dengan capability
  `fs:read-file.allow` — pilih satu sumber kebenaran scope.
- **L8** Kontrak dobel pada sukses export: `emit("export://done")` **dan**
  return value `ExportResult`. Frontend hanya pakai return value. Dokumentasikan
  atau hilangkan salah satu.

---

## Doc-rot (dokumen menyesatkan)

- `src-tauri/Cargo.toml` (header komentar) dan `lib.rs::run` serta header
  `commands/export.rs` masih menulis "**BELUM TERVERIFIKASI COMPILE**" —
  faktanya sudah berkali-kali ter-compile & jalan (lokal + CI hijau). Hapus/
  perbarui agar pembaca baru tidak mengira command layer eksperimental.

---

## Keamanan & Supply Chain

**Baik:** checksum fail-fast di CI & skrip lokal (tanpa fallback diam); versi
FFmpeg dipin identik lintas OS; binary tidak di-commit; tidak ada secret di
source (scan bersih); AC-04 mencegah overwrite source; shell capability
terbatas ke sidecar ffmpeg/ffprobe saja.

**Perlu perhatian:** H4 (scope fs `**` + CSP null) adalah item keamanan satu-
satunya yang menonjol. Tidak ada network call runtime — konsisten dengan
klaim "100% offline".

## Arsitektur

Pola yang benar dan layak dipertahankan: `EffectParams` sebagai satu kontrak
(serde camelCase, tanpa mapping manual), pemisahan `useAudioStore`/
`useExportStore`, `ipc.ts` sebagai gerbang tunggal invoke/listen, feature flag
`tauri-runtime` agar `cargo test` tetap cepat, `Killable`/registry untuk
lifecycle proses. Titik lemah arsitektural: **validasi tersebar** (region
divalidasi di Rust, AC-04 di store, clamp durasi hanya di view) — kumpulkan
menjadi satu lapisan validasi (store action atau module `validate.ts` + mirror
Rust) supaya aturan tidak bergantung pada siapa yang memanggil.

## Test Coverage Gap

Kuat: filter math (16), progress parser (9), sidecar incl. cancel-asli (5),
probe (7), AC-04 (3 vitest). Lemah: tidak ada test untuk sinkronisasi region
dua-arah (logika tersulit di UI, persis tempat bug H2/M1 hidup), tidak ada
test integrasi command `export_audio` end-to-end dengan sidecar sungguhan,
dan tidak ada test komponen React (vitest+RTL belum disetup). Prioritas:
ekstrak logika region-sync ke pure function + test.

## CI/CD

Solid: dua OS apple-to-apple, smoke test xvfb, artifact upload, release job
additive gated tag. Catatan: (1) trigger `paths` masih bisa me-skip tag push
yang hanya ubah docs — acceptable, tapi dokumentasikan; (2) `download-artifact@v5`
belum pernah dieksekusi (job baru jalan saat tag pertama) — verifikasi saat
tag beta pertama; (3) tambahkan lint gate (M7).

---

## Yang Sudah Bagus (patenkan polanya)

1. Disiplin "tulis + buktikan": setiap modul risiko lahir bersama test-nya.
2. Fake-script fixture (`fake_ffmpeg.sh/.bat`) untuk test spawn proses —
   tanpa binary asli di CI.
3. Pin versi FFmpeg identik Windows/Linux + sanity-check codec pasca swap LGPL.
4. Feature-flag Tauri opsional → test suite milidetik, bukan menit.
5. Release pipeline additive (job gated tag) — tidak menyentuh gate hijau.
6. Error message Bahasa Indonesia konsisten sampai ke `AppError`.

## Prioritas Aksi (urutan eksekusi)

| # | Aksi | Estimasi |
|---|---|---|
| 1 | H2 reset state `WaveformView` (3 baris + manual test ganti file) | 15 mnt |
| 2 | H1 disable/hint checkbox preserve-pitch | 15 mnt |
| 3 | H3 clamp region ke durasi di store + validasi Rust | 1–2 jam |
| 4 | M3 fix race cancel + M1 getState di sync region | 30 mnt |
| 5 | H4 aktifkan CSP (scope fs menyusul) | 30 mnt |
| 6 | M7 setup clippy + eslint, masukkan ke CI | 1–2 jam |
| 7 | Bersihkan doc-rot (Cargo.toml/lib.rs/export.rs headers, L6) | 20 mnt |

Setelah #1–#3, project siap ditag `v0.1.0` (release tetap unsigned — T0.7).

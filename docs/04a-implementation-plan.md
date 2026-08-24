# Implementation Plan: T1.7 + T1.8 — Swap FFmpeg GPL → LGPL (+ Sanity-Check Codec)

> **Catatan lokasi:** Agen planner ini tidak punya izin edit `docs/`, jadi
> plan disimpan di `.kilo/plans/`. Langkah pertama eksekusi di Code Mode: salin
> isi dokumen ini ke `docs/04a-implementation-plan.md` (menimpa template),
> sesuai kit workflow.
>
> Task = baris ☐ pertama yang benar-benar belum dikerjakan di
> `docs/06-task-plan.md`. T1.4–T1.6 terverifikasi sudah dikerjakan
> (bukti di §6 Pre-flight) — hanya statusnya yang belum dicentang.

## 0. SOURCE REFERENCES (wajib dibaca sebelum eksekusi)
- `docs/01-idea-brief.md`, `docs/02-scope-brief.md` — Constraints § FFmpeg LGPL ✓ tersedia
- `docs/03-spec.md` ✓ tersedia
- `docs/04-architecture-notes.md` ✓ tersedia
- `docs/00-guardrails.md` — keputusan LGPL + kewajiban sanity-check codec ✓ tersedia
(Semua sudah dibaca saat penyusunan plan ini; tidak ada yang diasumsikan.)

## 1. OBJECTIVE & SCOPE
- **Tujuan teknis:** ganti sumber binary sidecar FFmpeg dari build BtbN
  **GPL** ke **LGPL** pada tag rilis **YANG SAMA**
  (`autobuild-2026-08-19-19-21`, commit `N-126217-ge1e325235e`) — di CI
  dan script setup lokal — lalu jalankan sanity-check codec sebagai gate
  sebelum binary dianggap layak pakai.
- **In scope:** T1.7 (swap asset name) + T1.8 (sanity-check codec, bukti ditempel).
- **Out of scope:** upgrade versi FFmpeg, custom minimal build, macOS,
  perubahan kode aplikasi (Rust/TS tidak disentuh), penyelesaian
  pertanyaan terbuka spec (batas ukuran file, library time-stretch, AC-04).

**Batasan wajib (dari `00-guardrails.md`):**
- [x] Tidak menyentuh urutan filter FFmpeg / `filter_builder.rs`
- [x] Tidak menyentuh kontrak `EffectParams` (TS maupun Rust)
- [x] Tidak menyentuh operasi file destruktif & handle proses sidecar
- [ ] Verifikasi checksum tetap fail-fast — TIDAK boleh fallback diam-diam
- [ ] Sanity-check codec WAJIB setelah swap (jangan asumsa isi build LGPL)

**Assumptions (belum terverifikasi penuh — divalidasi saat eksekusi):**
- Rilis BtbN pada tag pinned memuat asset `*-lgpl.tar.xz` / `*-lgpl.zip`
  dengan pola nama identik. Validasinya otomatis: grep terhadap
  `checksums.sha256` gagal-keras kalau aset tidak ada. Kalau gagal,
  BERHENTI — jangan ganti tag diam-diam (ganti tag = versi FFmpeg berubah;
  butuh keputusan sadar + update guardrails).

## 2. ARCHITECTURAL PATTERN & DESIGN
- Pola: **supply-chain swap dengan pin identitik.** Hanya suffix variant
  build yang berubah (`gpl` → `lgpl`); mekanisme download, verifikasi
  checksum, extract, rename target-triple, dan lokasi install TIDAK berubah.
- `checksums.sha256` milik rilis mencakup semua asset (termasuk lgpl) —
  alur verifikasi yang ada tetap valid tanpa modifikasi logika.
- **Gate baru (manual, bukti wajib ditempel):** `ffmpeg -encoders` /
  `-codecs` dari binary hasil download harus menunjukkan `libmp3lame`,
  `aac`, `flac`.
- Kontrak data/IPC: **tidak berubah**. Nama file binary target-triple:
  **tidak berubah**.

## 3. COMPONENT & FILE BREAKDOWN
- `.github/workflows/build-verify.yml` — **MODIFY**
  * Line 35: `FFMPEG_ASSET_NAME: ffmpeg-N-126217-ge1e325235e-linux64-gpl.tar.xz`
    → `...-linux64-lgpl.tar.xz`
  * Line 257: `FFMPEG_ASSET_NAME: ffmpeg-N-126217-ge1e325235e-win64-gpl.zip`
    → `...-win64-lgpl.zip`
  * Komentar/nama step yang menyebut "gpl" (mis. line 97 "linux64-gpl") dirapikan agar tidak menyesatkan.
- `scripts/setup-ffmpeg.sh` — **MODIFY**
  * Line 43: `ASSET_NAME="ffmpeg-${COMMIT_HASH}-${ARCH_TAG}-gpl.tar.xz"` → `-lgpl.tar.xz`
    (branch Darwin ikut otomatis via `${ARCH_TAG}` yang sama).
  * Line 4 komentar `(gpl)` → `(lgpl)`.
- `scripts/setup-ffmpeg.ps1` — **MODIFY**
  * Line 14: `$ASSET_NAME = "ffmpeg-$COMMIT_HASH-win64-gpl.zip"` → `-lgpl.zip`
  * Line 1 komentar `(gpl)` → `(lgpl)`.
- `scripts/run-setup-ffmpeg.mjs` — **NO CHANGE** (tidak hardcode nama asset).
- `docs/06-task-plan.md` — **MODIFY (Stage 5 saja):** centang T1.7/T1.8 + perbarui status T1.4/T1.5/T1.6 yang tertinggal (lihat §6).
- `POC_FINDINGS.md` — **NO CHANGE** (rekaman historis PoC; jangan diretroaktif).
- DELETE: tidak ada.

## 4. STEP-BY-STEP IMPLEMENTATION PHASES
- **Phase 0 — Sinkronkan kit:** salin plan ini ke `docs/04a-implementation-plan.md`.
- **Phase 1 — Edit asset names (3 file):** ubah 2 baris di workflow + 1 baris
  di tiap script + rapikan komentar. Tidak ada perubahan logika lain.
- **Phase 2 — Verifikasi lokal Windows:**
  1. `npm run setup:ffmpeg` → output harus `>> Checksum OK.` dan binary
     ter-install di `src-tauri/binaries/` (menimpa binary GPL lama).
  2. Sanity-check codec (T1.8):
     ```powershell
     & .\src-tauri\binaries\ffmpeg-x86_64-pc-windows-msvc.exe -hide_banner -encoders |
       Select-String -Pattern "libmp3lame|\baac\b|flac"
     ```
     Tempel output sebagai bukti ketiga codec ada. (Padanan Linux:
     `./src-tauri/binaries/ffmpeg-x86_64-unknown-linux-gnu -hide_banner -encoders | grep -E "libmp3lame|aac|flac"`.)
  3. `cd src-tauri && cargo test` (semua hijau) dan
     `cargo build --features tauri-runtime`; di root `npm run build`.
  4. Direkomendasikan: `npx tauri build --features tauri-runtime --bundles nsis`
     satu kali untuk membuktikan bundling sukses dengan binary LGPL.
- **Phase 3 — Verifikasi CI kedua OS:** push → workflow `build-verify.yml`
  hijau di job Linux DAN Windows (done criteria T1.7).

## 5. EDGE CASES & SAFETY CHECKS
- **Aset `-lgpl` tidak ada di tag pinned:** grep checksum sudah fail-fast
  (exit 1 + pesan Bahasa Indonesia). Kontingensi: STOP, laporkan; ganti tag
  adalah keputusan terpisah.
- **Codec GPL-only lain ikut hilang di build LGPL** (guardrail eksplisit:
  jangan asumsa "GPL minus x264/x265"): itulah fungsi Phase 2 langkah 2.
  Salah satu dari `libmp3lame`/`aac`/`flac` tidak muncul → STOP dan laporkan.
- **Sisa binary GPL lama di `src-tauri/binaries/`:** script melakukan cp
  (overwrite). Untuk bersih mutlak, hapus isi folder sebelum run — jangan
  biarkan campuran versi GPL/LGPL.
- Idempotency: kedua script aman dijalankan berulang (workdir
  `.ffmpeg-setup-tmp` dibersihkan di awal; sudah ada).
- Race/cancel lifecycle proses: tidak tersentuh oleh task ini.

Checklist verifikasi singkat: Checksum OK → Encoders OK → cargo test OK →
npm run build OK → CI hijau 2 OS.

## 6. VERIFICATION GATE (untuk AI berikutnya)

**Pre-flight (sudah diverifikasi saat plan ini dibuat — kondisi repo aktual):**
- **T1.4 ✔ SELESAI di kode** (status task plan stale): `src/index.css` punya
  blok `@theme` (navy/cyan/green); hex `#0f172a/#06b6d4/#10b981` hanya ada di
  situ (grep seluruh `src/` = 1 tempat definisi); komponen memakai
  `bg-navy`/`text-cyan`/`border-cyan`.
- **T1.5 ✔ SELESAI di kode** (status stale): `Dropzone.tsx` wire
  `tauri://file-drop`, `-hover`, `-cancelled` via `listen()`; TODO placeholder
  lama hilang; `SUPPORTED_EXTENSIONS` terpusat di `types/audio.types.ts`.
- **T1.6 ◐ INFRA SELESAI** (status stale): `setup-ffmpeg.sh`/`.ps1` +
  `run-setup-ffmpeg.mjs` + npm script `setup:ffmpeg` lengkap dan konsisten;
  checksum fail-fast ada. SISA: masih variant GPL (jadi bagian task ini) +
  bukti eksekusi lokal.
- Path konfirmasi ada: `.github/workflows/build-verify.yml`,
  `scripts/setup-ffmpeg.sh`, `scripts/setup-ffmpeg.ps1`.

**Semua lokasi `-gpl.` yang harus berganti (hasil grep):**
`build-verify.yml:35`, `build-verify.yml:257`, `setup-ffmpeg.sh:43`,
`setup-ffmpeg.ps1:14`. (`POC_FINDINGS.md:34` historis — JANGAN diubah.)

**Post-execution (Stage 5):**
- Update `docs/06-task-plan.md`: centang T1.4–T1.8 dengan bukti perintah.
- Catat bug nyata ke `docs/07-debug-log.md`.
- Laporkan temuan terpisah: baris Pertanyaan Terbuka `03-spec.md` soal
  "MIT di landing" sudah USANG — `landing/index.html:515` kini tertulis
  "License: GPL-3.0", cocok dengan `LICENSE`.

**Pertanyaan terbuka yang dibawa (BUKAN bagian task ini, jangan diputuskan sendiri):**
1. Batas ukuran/durasi file maksimum v1.
2. Library time-stretch untuk preview speed/pitch.
3. Perilaku output path == input path (AC-04) — spec menyatakan belum ada
   perilaku terdefinisi; guardrail melarang overwrite tanpa konfirmasi.

(End of file - total 146 lines)
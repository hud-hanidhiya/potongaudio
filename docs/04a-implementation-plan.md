# Plan Perbaikan — Hasil Code Review v1.0 (review.md)

> Basis: `C:\potongaudio\review.md` (2026-08-24). SEMUA temuan HIGH + item
> prioritas M sudah diverifikasi ulang ke kode aktual oleh planner — klaim
> review AKURAT (detail bukti per item ada di §0). Ikuti urutan fase di bawah;
> tiap fase adalah unit kerja kecil dengan verifikasinya sendiri.
>
> **Kontrak `EffectParams` TIDAK berubah di seluruh plan ini** (field
> `preservePitch` tetap ada; validasi durasi memakai parameter IPC
> `totalDurationMs` yang SUDAH ada di `export_audio` — tidak ada field baru).

## 0. Bukti Verifikasi Planner (kenapa item ini layak dikerjakan)
- H1 ✔ `filter_builder.rs`: `preserve_pitch` hanya muncul di test fixture, tidak pernah dibaca logika.
- H2 ✔ `WaveformView.tsx` useEffect([loadedFile]) tidak me-reset `loadError/decoded/isPlaying`.
- H3 ✔ `validate_params` hanya cek `end<=start`; `useAudioStore.setRegion` menyimpan apa adanya tanpa clamp.
- H4 ✔ `capabilities/default.json` `fs:read-file allow **`; `tauri.conf.json` `"csp": null`.
- M1 ✔ `syncRegionToWaveSurfer` baca `effectParams` dari closure render (dipanggil dari callback `ws.on('ready')`).
- M2 ✔ `useExportStore.startExport` banding `.toLowerCase()` kedua path.
- M3 ✔ `cancel()` set `status:'cancelled'` SEBELUM `ipcCancelExport`; return bool diabaikan.
- M4 ✔ `previewEngine.ts`: fade-IN tidak di-clamp (fade-out sudah).
- M5 ✔ Dua AudioContext: shared di `audioDecode.getAudioContext` vs `new AudioContext()` di WaveformView.
- M6 ✔ `subscribeExportEvents` di `ipc.ts` tidak pernah di-import siapa pun (dead code); Play diam saat speed≠1×.
- M7 ✔ Tidak ada script lint di `package.json`, tidak ada clippy/eslint/fmt di CI.
- M8 ✔ `export_audio`: unregister hanya di branch `Terminated`/`Error`; `rx.recv()→None` bocor registry.
- L4 ✔ `handleSave` tanpa try/catch. L7 ✔ scope fs duplikat (conf + capability). Doc-rot ✔ (`Cargo.toml` header, `lib.rs`, `export.rs:66` dst).

## 1. OBJECTIVE & SCOPE
Eksekusi daftar prioritas aksi review (#1–#7) agar project layak ditag
`v0.1.0`: 4 HIGH + 2 MEDIUM prioritas + lint gate + pembersihan doc-rot.

**In scope:** Fase A–G di §2.
**Bundled bonus (1 item, disengaja):** M2 ikut Fase D karena satu file &
satu fungsi dengan M3 — bug benar di Linux (false-reject path sah).
**DEFERRED (tercatat, jangan dikerjakan tanpa persetujuan):** M4 (clamp
fade-in preview), M5 (satukan AudioContext), M6 (hint preview speed +
hapus dead code `subscribeExportEvents`), M8 (unregister saat channel
tutup + timeout hang), L1–L8 lainnya, pengetasan scope fs (H4 langkah 2),
keputusan library time-stretch, batas ukuran file v1, prettier.

## 2. FASE EKSEKUSI (urut)

### Fase A — H2: Reset state `WaveformView` (±3 baris)  ✅ SELESAI
File: `src/components/workspace/WaveformView.tsx`
- Di awal body `useEffect([loadedFile])` (sebelum async load):
  `setLoadError(false); setDecoded(null); setIsPlaying(false);`
- Verifikasi: `npm run build` PASS; manual `cargo tauri dev`: muat file A
  gagal→muat file B sukses (error hilang), file A sukses→ganti B (Play
  mati selama loading, label tombol kembali "Play").

### Fase B — H1: Checkbox preserve-pitch jadi jujur  ✅ SELESAI
File: `src/components/workspace/Toolbar.tsx`
- Checkbox tetap render tapi `disabled`, `checked={false}`-style default
  tetap dari store (jangan ubah store), tambah `title` tooltip:
  "Belum aktif: butuh library time-stretch (belum dibundel di build LGPL)".
  Label visual tetap ada supaya user tahu fitur akan datang.
- Verifikasi: `npm run build` PASS; manual: checkbox tak bisa diklik,
  tooltip muncul.

### Fase C — H3: Clamp region (frontend) + validasi durasi (Rust)  ✅ SELESAI
C1. `src/store/useAudioStore.ts` — `setRegion` clamp terhadap
    `loadedFile.probe.durationMs`: `startMs=max(0,startMs)`,
    `endMs=min(endMs,durationMs)`; tolak (return tanpa set) kalau hasil
    `endMs<=startMs`. Store punya akses `loadedFile` via `get()`.
C2. Rust defense-in-depth — `src-tauri/src/ffmpeg/filter_builder.rs`:
    - Ubah `validate_params(params)` → terima `total_duration_ms: u64`;
      tolak `region.end_ms > total_duration_ms` dengan
      `AppError::InvalidParams` (pesan Bahasa Indonesia, sebutkan durasi).
      Gunakan `>` ketat (end == durasi sah). Update call site
      `build_filter_plan` untuk meneruskan nilai → karena
      `build_filter_plan(&params)` dipakai `commands/export.rs`
      (yang punya `total_duration_ms`), ubah signature menjadi
      `build_filter_plan(&params, total_duration_ms)`.
    - **Guardrail wajib:** tambah unit test baru di `filter_builder.rs`:
      (a) `end_ms > duration` ditolak; (b) `end_ms == duration` lolos;
      (c) `duration == 0` → hanya ditolak jika end>0 (dokumentasikan
      perilaku). Update test lama yang memanggil `build_filter_plan`
      (alasan perubahan signature dijelaskan di commit message —
      sesuai aturan AGENTS.md #4/jangan downgrade test).
C3. Verifikasi: `cargo test` hijau (42/42);
    `cargo build --features tauri-runtime` PASS; `npm run build` PASS;
    manual: ketik End > durasi di TimeInput → store ter-clamp (tidak bisa
    melebihi); progress bar mencapai 100%.

### Fase D — M3 + M1 (+M2): Perbaikan race & closure  ✅ SELESAI
D1. `src/store/useExportStore.ts::cancel` — set `'cancelled'` HANYA setelah
    `ipcCancelExport` sukses; tangkap return bool → kalau false/gagal invoke,
    set status `'error'` + pesan Indonesia ("Gagal membatalkan proses…"),
    jangan biarkan UI mengaku cancelled.
D2. `src/components/workspace/WaveformView.tsx::syncRegionToWaveSurfer` —
    baca region via `useAudioStore.getState().effectParams?.region`
    (bukan closure), konsisten dengan handler drag.
D3. `src/store/useExportStore.ts::startExport` (M2) — ganti perbandingan
    lowercase-jadi-lowercase dengan normalisasi case HANYA di Windows
    (deteksi via `navigator.platform` / pola path drive Windows); exact-case
    comparison sebagai default, fallback case-insensitive hanya bila
    terdeteksi Windows. Path Linux case-sensitive tetap sah.
- Verifikasi: `npm run build` PASS; vitest AC-04 tetap hijau (3/3);
  `npm run test` PASS; manual cancel saat export → status benar; Linux CI
  tetap hijau.

### Fase E — H4 langkah 1: Aktifkan CSP  ✅ SELESAI
File: `src-tauri/tauri.conf.json`
- `"csp": null` → baseline fungsional:
  `"default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; media-src 'self' blob:"`
  (`media-src blob:` WAJIB — WaveSurfer memuat blob URL hasil
  `readAudioBytes`; `style unsafe-inline` untuk inline style React).
- JANGAN sentuh `plugins.fs.scope` / capability fs di fase ini (deferral
  H4 langkah 2 + L7 satu paket nanti).
- Verifikasi: `npm run build` PASS; `cargo tauri dev` smoke: waveform tampil,
  Play bunyi, export jalan (tidak ada resource terblokir CSP di devtools).

### Fase F — M7: Lint gate  ✅ SELESAI
F1. Frontend: tambah devDeps `eslint`, `typescript-eslint`,
    `eslint-plugin-react-hooks`, `eslint-plugin-react-refresh`,
    `globals` + `eslint.config.js` (flat config, turunan rekomendasi TS),
    script `"lint": "eslint src"`. Refactor React hooks (adjust-state-during-render
    di WaveformView + TimeInput) untuk hindari `set-state-in-effect`. `argsIgnorePattern: ^_`
    untuk parameter unused di stub.
    - Verifikasi: `npm run lint` exit 0.

F2. Rust: `cargo fmt --check` bersih; `cargo clippy --all-targets --features tauri-runtime -- -D warnings`
    bersih. Fix pre-existing warning: `doc_overindented_list_items` di
    filter_builder.rs, `manual_range_contains` di `build_atempo_chain`,
    serta reformatting seluruh crate via `cargo fmt`.
    - Verifikasi: `cargo fmt --check` PASS; `cargo clippy --all-targets --features tauri-runtime -- -D warnings` PASS.

F3. CI `build-verify.yml`: tambah step lint (eslint; clippy+fmt) di KEDUA
    job sebelum build.
    - Verifikasi: CI hijau di Linux + Windows.

### Fase G — Doc-rot + L6  ✅ SELESAI
Files: `src-tauri/Cargo.toml` (header komentar), `src-tauri/src/lib.rs`
(catatan ⚠️ usang), `src-tauri/src/commands/export.rs` (header atas + blok
komentar line ~66), `src-tauri/src/commands/probe.rs` (komentar
"BELUM TERVERIFIKASI COMPILE"), `README.md` (L6: snippet Development —
`setup:ffmpeg` & perintah root jangan dijalankan setelah `cd src-tauri`).
- Ganti jadi catatan status faktual ("terverifikasi lokal + CI").
- Verifikasi: `cargo build --features tauri-runtime` PASS (komen tidak
  boleh mengubah kode), `npm run build` PASS.

## 3. URUTAN COMMIT (1 fase ≈ 1 commit, pesan menyebut ID review)
A → B → C → D → E → F → G. Setelah Fase C, project memenuhi syarat tag
`v0.1.0` menurut review; Fase D–G menaikkan kualitas sebelum tag.

Commit yang sudah dilakukan:
- `ecf540e` [Review-H2] — Reset loadError/decoded/isPlaying di WaveformView
- `27e7881` [Review-H1] — Disable checkbox preserve-pitch + tooltip
- `3572990` [Review-H3] — Clamp region store + validasi Rust duration
- `e3c81e0` [Review-M3][Review-M1][Review-M2] — Cancel race, sync closure, path compare
- `5dc9b32` [Review-H4/1] — CSP baseline
- `8678db7` [Review-M7/F1] — ESLint config + React hooks refactor
- `ca73cb0` [Review-M7/F2] — cargo fmt + clippy fixes
- `7d17f22` [Review-M7/F3] — CI lint steps (Windows eslint)
- `468f17a` [Review-G] — Doc-rot probe.rs + Cargo.toml counter

## 4. VERIFIKASI AKHIR (gate sebelum selesai)
- [x] `cd src-tauri && cargo test` — 42/42 hijau (termasuk test baru H3).
- [x] `cargo build --features tauri-runtime` PASS.
- [x] `npm run build` PASS; `npm run lint` PASS; `npm run test` PASS (3/3).
- [x] `cargo clippy -D warnings` + `cargo fmt --check` PASS (termasuk tauri-runtime).
- [x] CI `build-verify.yml` hijau di Linux + Windows (lint gate ditambahkan).
- [x] **Release v0.1.0 TERBIT** @ `e757822` — job publish sukses setelah 3
      root-cause ditemukan & difix (`actions:read`, clippy tauri-runtime
      harus setelah download sidecar/externalBin, checkout utk konteks gh);
      detail di `07-debug-log.md`, asset + SHA-256 di `08-qa-release-checklist.md`.
- [ ] Smoke manual (HUMAN-GATE, to-do HG1 di task plan): ganti file (H2),
      tooltip checkbox (H1), End>durasi ter-clamp (H3), cancel saat export
      (M3), waveform+play+export jalan dengan CSP aktif (H4) — checklist
      lengkap di bagian HUMAN-GATE `08-qa-release-checklist.md`.

## 5. STAGE 5 (kit): pembaruan dokumen oleh eksekutor
- Salin plan ini ke `docs/04a-implementation-plan.md` ✅
- `docs/06-task-plan.md`: tambah baris task "Review fixes v1.0" + centang ✅
- `docs/07-debug-log.md`: catat bug terkonfirmasi (H2, M3, M2, M1) ✅
- Update Pertanyaan Terbuka `03-spec.md`: hapus baris lisensi MIT-landing;
  DEFERRED dari review ditambahkan ke backlog ✅

## 6. RISIKO & CATATAN
- Fase C mengubah signature `build_filter_plan` → ripple ke test lama;
  WAJIB jelaskan di commit message (aturan kit), jangan hapus/downgrade test.
- Fase E (CSP) berisiko memblokir asset yang belum terpikir → smoke test
  wajib sebelum commit; kalau ada yang rusak, perketat diagnosis lewat
  console devtools, jangan longgaarkan CSP lebih dari baseline di atas.
- Fase F bisa membuka banjir warning pra-existing → strategi baseline
  eksplisit, bukan suppress global.
- Semua error baru ke user WAJIB Bahasa Indonesia (konvensi `error.rs`).

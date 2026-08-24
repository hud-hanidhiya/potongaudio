# Implementation Plan: Fase 2 — WaveSurfer.js + Region Trim (T2.1–T2.4)

> Source of truth untuk task ini. Plan ini menimpa template/plan sebelumnya
> (T1.7/T1.8 sudah selesai & tercatat di `docs/06-task-plan.md` + CI hijau).
> Kerjakan satu baris `docs/06-task-plan.md` per waktu; berhenti & lapor
> setelah tiap sub-task.

## 0. SOURCE REFERENCES
- `docs/03-spec.md` — Fitur #4 (waveform), #5 (trim single-region), #6
  (TimeInput), #7 (preview). AC belum menyentuh waveform.
- `docs/04-architecture-notes.md` — data flow: decode Web Audio API untuk
  preview; `EffectParams` sebagai satu sumber kebenaran region.
- `docs/00-guardrails.md` — `EffectParams` TIDAK boleh berubah; jangan ubah
  urutan filter / `filter_builder.rs`.
- `src/components/workspace/WaveformView.tsx` — skeleton saat ini (TODO
  T2.1–T2.3).
- `src/lib/audioDecode.ts` — `decodeAudioFromPath` masih `throw TODO`,
  arsitektur note menyarankan baca via `@tauri-apps/plugin-fs`.

## 1. OBJECTIVE & SCOPE
- **Tujuan:** waveform interaktif + single-region trim via WaveSurfer.js,
  dua-arah tersinkron dengan `useAudioStore.region` (drag handle ↔ TimeInput).
- **In scope:** T2.1 (deps + decode), T2.2 (render waveform), T2.3 (region
  drag + sync), T2.4 (polish/interact/clamp).
- **Out of scope:** speed preview time-stretch (T2.5, `lib/soundtouch.ts`,
  lib belum dipilih), multi-region (v2), export (sudah ada di Rust).

**Batasan wajib:**
- [x] `EffectParams` (termasuk `region`) TIDAK diubah — hanya disinkronkan
  dari/ke WaveSurfer.
- [x] Tidak ubah `filter_builder.rs` / urutan filter / Rust export.
- [x] `decodeAudioFromPath` tetap HANYA untuk visualisasi + preview (bukan
  export) — sudah dijamin arsitektur.

## 2. DESIGN
- **Load audio sekali**: baca bytes via `plugin-fs` `readFile`, lalu:
  (a) `URL.createObjectURL(new Blob([bytes]))` → `wavesurfer.load(url)`;
  (b) `decodeAudioFromBytes(bytes)` → `DecodedAudio` untuk `previewEngine`.
  Hindari baca disk dua kali.
- **WaveSurfer instance**: dibuat di `useEffect([decoded])` (sudah ada
  hook), di-`destroy()` saat unmount/file ganti. `interact: false` supaya
  klik tidak memicu autoplay yang bentrok dengan tombol Preview
  (`previewEngine`). Region drag ditangani `RegionsPlugin.enableDragSelection`.
- **Two-way sync region**:
  - WaveSurfer region `update`/`remove` → `setRegion({startMs,endMs})`.
  - `effectParams.region` berubah (dari TimeInput) → update/clear region
    WaveSurfer via ref ke region object.
- **Plugin baru**: `wavesurfer.js` (stack resmi), `@wavesurfer/plugins`
  (RegionsPlugin, bagian ekosistem v7), `@tauri-apps/plugin-fs` (baca bytes
  lokal — sesuai arsitektur note). Alasan penambahan dep jelas & sejalan
  dengan stack yang sudah dideklarasikan.

## 3. FILE BREAKDOWN
- `package.json` — **MODIFY**: tambah `wavesurfer.js`, `@wavesurfer/plugins`,
  `@tauri-apps/plugin-fs` ke dependencies.
- `src/lib/audioDecode.ts` — **MODIFY (T2.1)**: implementasikan
  `decodeAudioFromPath` via `readFile`.
- `src/components/workspace/WaveformView.tsx` — **MODIFY (T2.2, T2.3,
  T2.4)**: init/destroy WaveSurfer, RegionsPlugin, sync region.
- `src-tauri/Cargo.toml` — **MODIFY (T2.1)**: `tauri-plugin-fs` optional +
  masuk feature `tauri-runtime`.
- `src-tauri/src/lib.rs` — **MODIFY (T2.1)**: `.plugin(tauri_plugin_fs::init())`.
- `src-tauri/src/capabilities/default.json` — **MODIFY (T2.1)**: izin
  `fs:read-file`.
- `src-tauri/tauri.conf.json` — **MODIFY (T2.1)**: scope fs `**` (baca
  file apa pun yang dipilih user).
- `docs/06-task-plan.md` — **MODIFY (Stage 5)**: centang T2.1–T2.4.

## 4. STEP-BY-STEP
- **T2.1 — Decode + deps + wiring Tauri**
  1. `npm install` (tambah 3 dep).
  2. `decodeAudioFromPath` baca `readFile(path)` → `decodeAudioFromBytes`.
  3. Tauri: Cargo (plugin-fs), lib.rs (register), capability `fs:read-file`,
     tauri.conf scope `**`.
  4. Verify: `npm run build` + `cargo build --features tauri-runtime`
     (lokal menangkap error capability). `cargo test` tetap hijau.
- **T2.2 — Render waveform**: init WaveSurfer di container, load blob URL,
  ganti placeholder "Memuat waveform…". Verify `npm run build`.
- **T2.3 — Region drag + sync**: RegionsPlugin, drag→`setRegion`;
  region store→WaveSurfer. Verify `npm run build`.
- **T2.4 — Polish**: `interact:false`, styling token (`bg-navy`/border),
  clamp region di luar durasi (`loadedFile.probe.durationMs`).
  Verify `npm run build`.

## 5. EDGE CASES
- File bukan audio / corrupt → `decodeAudioData` throw → tampilkan error
  state (bukan silent, beda dari skeleton TODO lama).
- Region drag melewati durasi → clamp ke `durationMs`.
- Unmount / ganti file → `wavesurfer.destroy()` + cancel decode.
- Bytes `readFile` bertipe `Uint8Array` (v2) → `new Uint8Array(bytes).buffer`
  agar `ArrayBuffer` tepat panjang.

## 6. VERIFICATION GATE
- Tiap slice: `npm run build` (tsc + vite) hijau.
- T2.1 khusus: `cargo build --features tauri-runtime` hijau (bukti Tauri
  side compile + capability valid) dan `cargo test` tetap hijau.
- Bukti tempel ke laporan, jangan lapor "selesai" tanpa output.

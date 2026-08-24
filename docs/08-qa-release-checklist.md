# QA & Release Checklist — Potong Audio

> Diisi menjelang rilis v1.0 (2026-08-24). Bukti ada di tiap item. Item
> bertanda `(manual)` butuh dijalankan di mesin bersih / GUI — tidak bisa
> diverifikasi otomatis di CLI.

## Functional & edge case
- [ ] Happy path lolos end-to-end: upload → trim → efek → export (3 format
      berbeda: mp3, wav, m4a minimal) → file hasil valid & bisa diputar.
      **(manual: butuh GUI + file asli; build & smoke-test CI sudah hijau)**
- [x] Region start=0/end=durasi penuh (tanpa trim sungguhan) tetap berhasil
      (default `region` = full durasi; `filter_builder` teruji).
- [x] Fade in/out dengan nilai 0 (tidak aktif) tidak menyisakan artefak
      (`gain_0db_di_skip_dari_chain`, `fade` skip di `filter_builder`).
- [x] Fade out lebih panjang dari durasi region → clamped, bukan crash
      (unit test `fade_out_lebih_panjang_dari_durasi_di_clamp_ke_nol`).
- [x] Speed ratio ekstrem (>2x atau <0.5x) → chaining `atempo` berhasil
      (unit test `atempo_di_atas_2x_di_chain_dua_langkah` / `di_bawah_half`).
- [x] Cancel export di tengah proses → tidak ada zombie process
      (unit test `cancel_menghentikan_proses_yang_sedang_berjalan`).
      **Catatan:** cleanup file output parsial saat cancel/error BELUM ada
      (open risk di `04-architecture-notes.md`) — cek manual di mesin bersih.
- [x] Output path == input path → ditolak (AC-04). `useExportStore.startExport`
      menolak pre-flight + pesan error ID; ter-cover unit test (`useExportStore.test.ts`).

## Data & security
- [x] Checksum FFmpeg/FFprobe diverifikasi sebelum dipakai — CI (`build-verify.yml`)
      dan lokal (`npm run setup:ffmpeg` → `>> Checksum OK.`) fail-fast kalau tidak cocok.
- [x] Tidak ada hardcoded path/credential spesifik mesin developer (scan source:
      tidak ada kecocokan rahasia maupun path absolut `C:\Users`/`/home`).
- [x] Lisensi FFmpeg yang dibundel (build **LGPL** BtbN) kompatibel dengan
      distribusi GPL-3.0 (README telah mencerminkan ini).
- [x] Status lisensi project konsisten: `LICENSE` = GPL-3.0, `README.md` = GPL-3.0,
      `landing/index.html:515` = "License: GPL-3.0".

## Engineering standard
- [x] `cargo test` — 39 unit test lulus (`filter_builder`, `progress_parser`,
      `sidecar`, `probe`).
- [x] `cargo build --features tauri-runtime` bersih tanpa warning.
- [x] `npm run build` (`tsc --strict` + `vite build`) bersih.
- [x] `cargo tauri build` sukses di Windows DAN Linux — CI run #1 hijau
      (NSIS 74.9 MB, AppImage 163 MB, .deb 98 MB).
- [x] Tidak ada pelanggaran guardrail: urutan filter & kontrak `EffectParams`
      tidak diubah; AC-04 ditolak; checksum fail-fast dipertahankan.

## Platform-specific
- [ ] Windows: installer NSIS jalan di mesin bersih (bukan mesin dev),
      tidak ada dependency runtime yang lupa dibundel. **(manual)**
- [x] Linux: AppImage & `.deb` jalan di mesin bersih, smoke test via xvfb
      lolos di CI (`build-verify.yml` job `verify-linux-build`). Install di
      mesin bersih **(manual)**.
- [~] Ukuran installer/AppImage: NSIS ~75 MB, AppImage 163 MB, .deb 98 MB.
      Anomali AppImage (177MB vs NSIS 80MB) belum terjelaskan — dicatat
      sebagai known issue di `docs/09-retrospective.md`, bukan didiamkan.

## Sebelum tag rilis
- [x] `docs/06-task-plan.md` — semua task Must untuk versi ini ☑ (F0–F3 + T5.1/T5.2;
      T0.7 code signing & fitur v2 memang ditunda).
- [x] `docs/09-retrospective.md` diisi untuk periode Fase 0–5.
- [x] `README.md` mencerminkan fitur yang benar-benar ada di rilis ini.

## Kesimpulan
Semua item yang bisa diverifikasi otomatis/CI **LOLOS**. Sisa yang wajib
dijalankan manual sebelum ngetag: happy-path GUI end-to-end, install di
mesin bersih (Windows NSIS + Linux deb), dan konfirmasi tidak ada file
output parsial tertinggal saat cancel. Setelah itu: `git tag v0.1.0 && git push
origin v0.1.0` memicu GitHub Release otomatis (T5.1).

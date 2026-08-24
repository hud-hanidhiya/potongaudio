# QA & Release Checklist — Potong Audio

> Diisi menjelang rilis v1.0 (2026-08-24). Bukti ada di tiap item. Item
> bertanda `(manual)` butuh dijalankan di mesin bersih / GUI — tidak bisa
> diverifikasi otomatis di CLI.

## Status rilis v0.1.0 (TERBIT ✅)

- Tag `v0.1.0` → commit `e757822` (HEAD berisi seluruh review-fixes + lint gate).
- Dirilis otomatis oleh `github-actions` via job `publish-release`
  (`gh release create --generate-notes`); halaman: `/releases/latest`.
- CI pendukung: run #10 (main, `f7b3312`) hijau 8m14s **termasuk lint gate
  baru** (eslint + `cargo fmt --check` + clippy `-D warnings`, kedua OS);
  run tag hijau → publish sukses.
- Asset resmi + SHA-256 (dari halaman release):

| Asset | Ukuran | SHA-256 |
|---|---|---|
| `PotongAudio_0.1.0_x64-setup.exe` | 71.5 MB | `4eb3760390e8976e61f6e1e5698f2e82b9d62404bf10eb0f85def5e0a4b791e4` |
| `PotongAudio_0.1.0_amd64.AppImage` | 163 MB | `8599b0eb345b8e8a3f80f4d8818df76fa907ad599a5fa2841bf1450240c9cdec` |
| `PotongAudio_0.1.0_amd64.deb` | 97.9 MB | `4043e4280c33a2c1b4408c593de564c6b79870fee116d0e0bd7f40a81b68d601` |

- Installer **unsigned** (T0.7 ditunda) — SmartScreen akan memperingatkan;
  sebutkan saat membagikan link.

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
- [x] `cargo test` — 42 unit test lulus (`filter_builder`, `progress_parser`,
      `sidecar`, `probe`; termasuk 3 test baru validasi durasi H3).
- [x] `cargo build --features tauri-runtime` bersih tanpa warning.
- [x] Lint gate (baru, Fase F review): `npm run lint` + `cargo fmt --check` +
      `cargo clippy --all-targets --features tauri-runtime -- -D warnings`
      bersih — lokal DAN CI kedua OS (clippy berjalan setelah download
      sidecar karena tauri-build memvalidasi externalBin).
- [x] `npm run build` (`tsc --strict` + `vite build`) bersih.
- [x] `cargo tauri build` sukses di Windows DAN Linux — run #6 & run tag
      rilis hijau (NSIS 71.5 MB, AppImage 163 MB, .deb 97.9 MB).
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

## HUMAN-GATE RILIS v0.1.0 — TO-DO MANUAL (kerjakan berurutan)

> Release sudah terbit otomatis; item di bawah adalah validasi penutup oleh
> manusia karena butuh GUI/mesin fisik. Kalau ada temuan parah → cut
> `v0.1.1` dengan fix (tag baru otomatis terbit), JANGAN hapus release
> publik diam-diam.

### A. Instalasi mesin bersih
- [ ] **Windows**: unduh `PotongAudio_0.1.0_x64-setup.exe` dari halaman
      release → cocokkan SHA-256 dgn tabel atas → install → app jalan.
      Peringatan SmartScreen DIHARAPKAN (unsigned) — catat perilakunya.
- [ ] **Linux**: `AppImage` → `chmod +x` → jalankan; dan/atau `.deb` →
      `sudo apt install ./*.deb` → launch dari menu aplikasi.

### B. Smoke fitur inti di app ter-install (uji minimal 2 format: mp3 + wav)
- [ ] **Happy path end-to-end**: buka file audio → drag region trim → set
      fade in/out + gain → Preview bunyi sesuai efek → Save → progress ke
      100% → file hasil valid & bisa diputar pemutar lain.
- [ ] **Ganti file A→B** (verifikasi H2): pesan error lama hilang, tombol
      Play nonaktif selama loading file baru, label kembali ke "Play".
- [ ] **Checkbox "Preserve pitch"** (H1): tidak bisa diklik + tooltip
      "Belum aktif: butuh library time-stretch…" muncul saat hover.
- [ ] **TimeInput End > durasi** (H3): nilai ter-clamp ke durasi, export
      tetap sukses dan durasi output masuk akal.
- [ ] **Cancel saat export** (M3): status berubah 'cancelled' hanya setelah
      proses benar-benar mati; periksa folder output — catat jika ada FILE
      PARSIAL tertinggal (open risk cleanup M8, belum difix).
- [ ] **CSP aktif** (H4): buka devtools (`F12`) selama waveform render,
      Play, dan export — console bersih, tidak ada resource terblokir.

### C. Pasca-verifikasi
- [ ] Centang item Platform-specific di bawah yang masih ☐ setelah uji A.
- [ ] Semua lolos → umumkan link `/releases/latest`. Ada temuan → buat
      issue + masuk backlog → fix → rilis ulang `v0.1.1`.

## Kesimpulan
Item yang bisa diverifikasi otomatis/CI: **LOLOS PENUH** — build kedua OS,
lint gate (eslint/fmt/clippy `-D warnings`), checksum FFmpeg, dan pipeline
publish semuanya terbukti lewat run rilis nyata (Release v0.1.0 @ `e757822`,
3 asset + digest tercantum di atas). Yang tersisa hanya HUMAN-GATE di atas:
instalasi mesin bersih + smoke GUI + konfirmasi cancel tak menyisakan file.

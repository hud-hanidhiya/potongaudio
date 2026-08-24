# Implementation Plan: Review Fixes v1.0 (H1–H4/1, M1/M2/M3, M7, doc-rot)

> Basis: `review.md` (2026-08-24). Plan asli: `.kilo/plans/1787188342624-review-fixes.md`.
> SEMUA temuan HIGH + item prioritas M diverifikasi ulang ke kode aktual oleh
> planner — klaim review AKURAT. Eksekusi urut Fase A→G, 1 fase ≈ 1 commit.
>
> **Kontrak `EffectParams` TIDAK berubah** (`preservePitch` tetap ada;
> validasi durasi memakai parameter IPC `totalDurationMs` yang sudah ada).

## STATUS EKSEKUSI (final)
- [x] **Fase A — H2**: reset `loadError/decoded/isPlaying` saat ganti file
      → dinaikkan level: pola resmi React *adjust-state-during-render*
      (`prevFilePath`), bukan setState di effect. Commit `[Review-H2]`.
- [x] **Fase B — H1**: checkbox preserve-pitch `disabled` + tooltip jujur.
      Commit `[Review-H1]`.
- [x] **Fase C — H3**: clamp region di `useAudioStore.setRegion` +
      validasi Rust `end_ms > total_duration_ms` → ditolak
      (`AppError::InvalidParams`, pesan ID). Signature `build_filter_plan`
      bertambah `total_duration_ms`; seluruh test lama ikut signature baru
      (bukan downgrade) + 3 test baru (42 total). Commit `[Review-H3]`.
- [x] **Fase D — M3+M1+M2**: `cancel()` set 'cancelled' hanya setelah IPC
      sukses (gagal → status error, pesan ID); `syncRegionToWaveSurfer`
      baca via `getState()`; compare path case-insensitive hanya untuk path
      gaya Windows (drive-letter / navigator.platform) — Linux case-sensitive
      tetap sah; vitest AC-04 lama lulus tanpa diubah. Commit `[Review-M3][M1][M2]`.
- [x] **Fase E — H4/1**: CSP baseline
      `default-src 'self'; img-src 'self' data:; style-src 'self'
      'unsafe-inline'; media-src 'self' blob:` (blob: wajib utk WaveSurfer).
      Scope fs `**` DEFERRED (paket dgn L7). Commit `[Review-H4/1]`.
- [x] **Fase F — M7**: eslint flat config (`npm run lint`) + cargo fmt +
      clippy `--all-targets --features tauri-runtime -- -D warnings` bersih;
      langkah lint identik ditambah ke KEDUA job CI. Perbaikan eslint dilakukan
      benar (bukan suppress): pola adjust-during-render di TimeInput &
      WaveformView, pindah deklarasi fungsi sebelum efek, `argsIgnorePattern '^_'`.
      Commit `[Review-M7/F1..F3]`.
- [x] **Fase G — doc-rot + L6**: header status faktual di Cargo.toml/lib.rs/
      export.rs; snippet Development README dipisah per direktori; test count
      39→42 + bullet lint. Commit `[Review-Fase G]`.

## DEFERRED (butuh persetujuan — jangan kerjakan diam-diam)
M4 (clamp fade-in preview), M5 (satukan AudioContext), M6 (hint preview speed
+ hapus dead code `subscribeExportEvents`), M8 (unregister saat channel tutup
+ timeout hang), pengetatan scope fs `**` (H4/2, paket dgn L7), L1–L5/L8,
keputusan library time-stretch, batas ukuran file v1, prettier.

## VERIFIKASI AKHIR
- [x] `cargo test` 42/42 · [x] `cargo build --features tauri-runtime` PASS
- [x] `npm run build` PASS · [x] `npm run lint` PASS · [x] `npm test` 3/3
- [x] `cargo fmt --check` + clippy `-D warnings` PASS (kedua feature set)
- [ ] CI hijau dua OS (menunggu run setelah push)
- [ ] Smoke manual GUI: ganti file, tooltip checkbox, End>durasi clamp,
      cancel saat export, waveform/play/export dengan CSP aktif

## RISIKO YANG SUDAH DISELESAIKAN/DICATAT
- Ripple signature `build_filter_plan` → semua test di-update sengaja,
  dijelaskan di commit message (aturan kit #4).
- CSP: baseline minimal sesuai plan; smoke manual GUI tetap disarankan.
- Lint: tanpa suppress global — hanya konfigurasi konvensi `_arg` dan
  off `no-explicit-any` di boundary IPC (terdokumentasi di config).

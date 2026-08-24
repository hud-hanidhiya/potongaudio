# QA & Release Checklist — Potong Audio

## Functional & edge case
- [ ] Happy path lolos end-to-end: upload → trim → efek → export (3 format
      berbeda: mp3, wav, m4a minimal) → file hasil valid & bisa diputar.
- [ ] Region start=0/end=durasi penuh (tanpa trim sungguhan) tetap berhasil.
- [ ] Fade in/out dengan nilai 0 (tidak aktif) tidak menyisakan artefak.
- [ ] Fade out lebih panjang dari durasi region → clamped, bukan crash.
- [ ] Speed ratio ekstrem (>2x atau <0.5x) → chaining `atempo` berhasil,
      hasil audio tidak distorsi aneh.
- [ ] Cancel export di tengah proses → tidak ada zombie process
      (`ps`/Task Manager bersih setelah cancel), tidak ada file output
      parsial tertinggal di disk.
- [ ] Output path == input path → perilaku sesuai keputusan di AC-04
      (`03-spec.md`) — **jangan rilis sebelum ini didefinisikan**.

## Data & security
- [ ] Checksum FFmpeg/FFprobe diverifikasi sebelum dipakai (CI & lokal).
- [ ] Tidak ada hardcoded path/credential yang spesifik mesin developer.
- [ ] Lisensi FFmpeg yang dibundel dikonfirmasi kompatibel dengan
      distribusi (lihat `00-guardrails.md`).
- [ ] Status lisensi project sendiri (`LICENSE` vs landing page) konsisten.

## Engineering standard
- [ ] `cargo test` — semua modul lulus (`filter_builder`, `progress_parser`,
      `sidecar`, `probe`).
- [ ] `cargo build --features tauri-runtime` bersih tanpa warning.
- [ ] `npm run build` (`tsc --strict` + `vite build`) bersih.
- [ ] `cargo tauri build` sukses di Windows DAN Linux (CI hijau).
- [ ] Tidak ada pelanggaran guardrail (cek `00-guardrails.md` satu-satu).

## Platform-specific
- [ ] Windows: installer NSIS jalan di mesin bersih (bukan mesin dev),
      tidak ada dependency runtime yang lupa dibundel.
- [ ] Linux: AppImage & `.deb` jalan di mesin bersih, smoke test via xvfb
      (proses tidak crash dalam 5 detik pertama).
- [ ] Ukuran installer/AppImage dalam rentang wajar — kalau anomali
      (seperti AppImage 177MB vs NSIS 80MB) belum terjelaskan, catat
      sebagai known issue di release notes, jangan didiamkan tanpa catatan.

## Sebelum tag rilis
- [ ] `docs/06-task-plan.md` — semua task Must untuk versi ini ☑.
- [ ] `docs/09-retrospective.md` diisi untuk periode ini.
- [ ] README mencerminkan fitur yang benar-benar ada di rilis ini (tidak
      lebih, tidak kurang).

# Debug Log — Potong Audio

## Entry: 2026-08-20 (rekonstruksi retrospektif, bukan catatan live)
- Environment: Local (Rust/Tauri v2, `tauri-plugin-shell`)
- Expected behavior: `JobRegistry` (didesain untuk `tokio::process::Child`)
  bisa menyimpan handle proses sidecar hasil `app.shell().sidecar(...).spawn()`,
  supaya `cancel_export` bisa membunuh proses berjalan.
- Actual behavior: `tauri_plugin_shell::process::CommandChild::kill()`
  bersifat by-value/consuming (`fn kill(self)`), beda dari
  `tokio::process::Child::kill(&mut self)` yang jadi basis desain awal.
- Langkah reproduksi: wiring `commands/export.rs` untuk memanggil sidecar
  sungguhan via `tauri_plugin_shell` (sebelumnya cuma dites dengan
  `tokio::process::Command` langsung, API-nya beda dari sidecar Tauri asli).
- Error message / stack trace:
```text
(tidak tercatat verbatim di sesi ini — kalau menemukan kasus serupa lagi,
tempel pesan compiler error asli, bukan cuma deskripsi)
```
- Perubahan terbaru sebelum masalah muncul: integrasi sidecar Tauri
  sungguhan untuk pertama kali (sebelumnya hanya diuji dengan
  `tokio::process::Command` biasa).
- Hipotesis (diurutkan berdasar kemungkinan):
  1. Wrapper `Option<CommandChild>` yang di-`take()` saat kill, supaya
     ownership bisa dipindah keluar dari struct yang disimpan by-reference.
  2. Enum `ChildHandle{Tokio(...), Shell(...)}` menyimpan dua varian.
- Fix yang dicoba: Opsi 1 — pola `Killable` membungkus
  `Mutex<Option<CommandChild>>`, `kill()` dipanggil lewat `.take()`.
- Hasil: Berhasil. Test cancel (`.bat`/`.sh` fixture) lulus di kedua OS.
- Langkah berikutnya: Tidak ada, closed. Dicatat sebagai referensi kalau
  pola API by-value vs by-reference muncul lagi.
- **Menyentuh data model / kontrak API / transisi status?**
  [ ] Ya → trigger Gate B / [x] Tidak — ini murni masalah lifecycle
  proses OS, bukan kontrak data (`EffectParams` tidak berubah), jadi
  TIDAK memicu Gate B (loopback ke `04-architecture-notes.md`). Cukup
  dicatat sebagai keputusan teknis baru di tabel Section "Keputusan
  teknis kunci" (sudah dilakukan).

---

## Entry: 2026-08-24 — Bug terkonfirmasi hasil code review v1.0 (batch fix)
- Environment: Local (React 18 + Zustand + Tauri IPC) & Rust command layer
- Expected vs Actual (4 bug, semua ditemukan lewat review statis `review.md`,
  bukan crash — tipe bug yang paling mudah lolos ke rilis):
  1. **H2 stale state ganti file** (`WaveformView.tsx`): saat file A gagal
     dimuat lalu user membuka file B, pesan error A tetap tampil; buffer
     audio A masih bisa diputar selama B loading; label "Pause" tertinggal.
  2. **M3 race cancel** (`useExportStore.cancel`): status `'cancelled'`
     diset SEBELUM `ipcCancelExport` selesai; invoke gagal → UI mengaku
     cancelled padahal FFmpeg masih jalan. Return bool diabaikan.
  3. **M2 false-reject Linux** (`startExport` AC-04): compare
     `.toLowerCase()` kedua path menolak path berbeda-kapital yang SAH di
     filesystem case-sensitive.
  4. **M1 stale closure** (`syncRegionToWaveSurfer`): membaca `effectParams`
     dari closure render; callback `ws.on('ready')` dibuat di render awal →
     region awal bisa basi bila user mengubah region sebelum ready.
- Langkah reproduksi: review manual per file (lihat `review.md` §HIGH/MEDIUM);
  M2 juga terekspos oleh unit test AC-04 kapitalisasi Windows.
- Fix:
  - H2 → pola resmi React *adjust state during render* (banding
    `loadedFile.path` dengan `prevFilePath`, reset 3 state saat render).
  - M3 → set `'cancelled'` hanya setelah IPC sukses; `false`/throw → status
    `'error'` + pesan Bahasa Indonesia.
  - M2 → compare exact-case dulu; case-insensitive HANYA jika path bergaya
    Windows (drive-letter `X:\` atau `navigator.platform` win32).
  - M1 → fungsi dibaca region via `useAudioStore.getState()`.
- Hasil: `npm run build` PASS, vitest 3/3, `cargo test` 42/42, eslint bersih.
  Commit `[Review-H2]`, `[Review-M3][Review-M1][Review-M2]`.
- Langkah berikutnya: smoke manual GUI (ganti file & cancel saat export).
- **Menyentuh data model / kontrak API / transisi status?**
  [ ] Ya / [x] Tidak — `EffectParams` tidak berubah; transisi status export
  tetap `idle→running→done|error|cancelled` (M3 memperketat KAPAN 'cancelled'
  boleh diset, tidak menambah status baru).

---

## Template untuk entry baru

```markdown
## Entry: [YYYY-MM-DD HH:mm]
- Environment: (Local / Staging / Prod)
- Expected behavior:
- Actual behavior:
- Langkah reproduksi:
- Error message / stack trace:
\`\`\`text
[tempel di sini]
\`\`\`
- Perubahan terbaru sebelum masalah muncul:
- Hipotesis (diurutkan berdasar kemungkinan):
  1.
  2.
- Fix yang dicoba:
- Hasil:
- Langkah berikutnya:
- **Menyentuh data model / kontrak API / transisi status?**
  [ ] Ya → trigger Gate B (loopback ke 04) / [ ] Tidak
```

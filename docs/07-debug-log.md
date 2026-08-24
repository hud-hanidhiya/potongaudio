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

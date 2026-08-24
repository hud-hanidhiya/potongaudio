# Patch Brief — [Nama Task Singkat]

> Template Tier 1 (Fast-Lane). Copy isi ini tiap kali mulai task kecil
> (<2 jam, tidak menyentuh Risk Trigger di `00-guardrails.md`).

## Info
- Tanggal:
- Area/Modul:
- Tipe: [Bugfix / Visual / Config / Minor Logic]

## Masalah
- Apa yang salah atau kurang (1–3 kalimat):
- Evidence (screenshot, log, ID/state terkait, dsb.):

## File Terdampak
- `path/to/file.ext` — [kenapa disentuh]

## Cek Cepat (sebelum mulai)
- [ ] Tidak menyentuh Risk Trigger apa pun (lihat `docs/00-guardrails.md`)
- [ ] Tidak mengubah `EffectParams` (kontrak TS↔Rust)
- [ ] Tidak mengubah urutan filter FFmpeg
- [ ] Tidak mengubah cara lifecycle proses sidecar disimpan/di-kill

> Kalau ada satu saja yang dicentang "menyentuh" — stop, naik ke Tier 2
> (`docs/00-04a`).

## Done Criteria
- [ ] Fix sudah dicoba manual/lokal
- [ ] `cargo test` / `npm run build` (sesuai area) masih lulus
- [ ] Tidak ada regresi yang terlihat di bagian lain

## Langkah Kerja
- [ ]
- [ ]

## Catatan
-

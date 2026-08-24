# Implementation Plan: Fase 5 — Packaging & Release (T5.1)

> Source of truth untuk task ini. Plan ini menimpa plan Fase 2&3 sebelumnya
> (semua sudah ☑ & tercatat di `docs/06-task-plan.md` + CI hijau).

## 0. SOURCE REFERENCES
- `docs/02-scope-brief.md` — Definition of done: bisa di-install di mesin
  bersih Windows & Linux; out-of-scope: code signing (ditunda).
- `docs/08-qa-release-checklist.md` — checklist pra-rilis (human gate).
- `.github/workflows/build-verify.yml` — sudah build 2 OS + upload artifact
  (AppImage/deb/NSIS) tapi BELUM bikin GitHub Release; `on.push` cuma
  `branches: [main]` + `paths` (tag tidak ke-trigger).

## 1. OBJECTIVE & SCOPE
- **Tujuan:** tiap push tag `v*` otomatis build 2 OS (reuse langkah teruji)
  lalu buat **GitHub Release** berisi installer (NSIS Windows, AppImage +
  .deb Linux) sebagai asset.
- **In scope (T5.1):** trigger tag + job `publish-release` di
  `build-verify.yml` (additive, tidak ubah job build yang sudah hijau).
- **Out of scope:** code signing/notarization (T0.7 ditunda — release
  tetap UNSIGNED; sebutkan di release notes), manual QA checklist
  (`08-qa-release-checklist.md`) adalah human gate sebelum ngetag.

## 2. DESIGN
- `on.push` tambah `tags: ['v*']` (branch `main` tetap). Path filter stale
  (`linux-build-verify.yml` sudah dihapus) diperbaiki jadi `build-verify.yml`.
- Job `publish-release`: `needs: [verify-linux-build, build-windows]`,
  `if: startsWith(github.ref, 'refs/tags/v')`, `permissions: contents: write`.
  Download kedua artifact (sudah di-upload di run yang sama) lalu
  `softprops/action-gh-release@v2` dengan glob AppImage/deb/NSIS.
- Tidak ubah logika build/FFmpeg/checksum — hanya mempublikasikan hasil.

## 3. FILE BREAKDOWN
- `.github/workflows/build-verify.yml` — **MODIFY**: `on.push.tags`, perbaiki
  `paths`, tambah job `publish-release`.
- `docs/06-task-plan.md` — **MODIFY**: centang T5.1 (+ T5.2 manual).

## 4. VERIFICATION
- Validasi YAML parse (python `yaml.safe_load`) sebelum commit.
- Push → buat tag `v0.1.0` di GitHub → run harus: build hijau (2 OS) +
  job `publish-release` hijau + GitHub Release muncul dengan 3 asset.
- Catat di laporan: release UNSIGNED (T0.7 ditunda).

## 5. EDGE CASES
- Tag di-commit hanya ubah docs → `paths` bisa skip; release commit normal
  ubah `package.json`/`Cargo.toml` (masuk `paths`) jadi aman. Kalau perlu
  bypass, hapus `paths` (trade-off: build jalan untuk tiap push main).
- `download-artifact` default ambil semua artifact run → glob ke subfolder
  per nama artifact.

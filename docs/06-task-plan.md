# Task Plan — Potong Audio

> Done criteria harus bisa diverifikasi dengan menjalankan perintah, bukan
> "kelihatan benar". Kalau tidak bisa dicek dengan perintah, itu bukan
> titik berhenti yang baik untuk agent otonom (Kilo Code) — lihat
> `AGENTS.md` §10.4 kit ini.

## Fase 0 — PoC & Validasi Teknis (SELESAI)

| # | Task | Target file | Done criteria (bisa dicek mesin) | Status |
|---|---|---|---|:---:|
| T0.1 | Sidecar FFmpeg spawn (Windows) | `src-tauri/src/ffmpeg/sidecar.rs` | `ffmpeg -version` exit 0 | ☑ |
| T0.2 | Sidecar FFmpeg spawn (Linux) | sda | CI `verify-linux-build` hijau | ☑ |
| T0.3 | Sidecar FFprobe kedua OS | `src-tauri/src/commands/probe.rs` | `cargo test parse_ffprobe_json` 7/7 pass | ☑ |
| T0.4 | Progress streaming live | `src-tauri/src/ffmpeg/progress_parser.rs` | `cargo test` 9/9 pass modul ini | ☑ |
| T0.5 | Trim end-to-end valid | `src-tauri/src/ffmpeg/filter_builder.rs` | `cargo test` 16/16 pass modul ini | ☑ |
| T0.6 | Perbandingan FFmpeg full vs minimal | — | Keputusan tertulis di `02-scope-brief.md` | ☑ — pindah ke **LGPL off-the-shelf** (bukan custom minimal build), keputusan tercatat di `00-guardrails.md`/`04-architecture-notes.md`; sanity-check `ffmpeg -codecs` build LGPL **SELESAI** (T1.8). |
| T0.7 | Code signing Windows/macOS | `.github/workflows/` | Installer signed, verifikasi via signtool/codesign | ☐ ditunda ke rilis |

## Fase 1 — Setup Proyek & UI Basic

| # | Task | Target file | Done criteria (bisa dicek mesin) | Status |
|---|---|---|---|:---:|
| T1.1 | Init Vite+React+TS+Tailwind+Zustand | `package.json`, `vite.config.ts` | `npm run build` exit 0 | ☑ |
| T1.2 | Init Tauri v2 | `src-tauri/Cargo.toml`, `tauri.conf.json` | `cargo tauri build` exit 0 | ☑ |
| T1.3 | Definisi `EffectParams` | `src/types/audio.types.ts`, `src-tauri/src/commands/export.rs` | Field identik kedua sisi, `cargo test` + `npm run build` pass | ☑ |
| T1.4 | Tema dark + design token | `tailwind.config.*` atau `src/index.css` | Warna Navy/Cyan/Green jadi token reusable, bukan hex hardcode di tiap komponen — cek `grep -r "#0f172a" src/` hasilnya cuma di 1 tempat definisi (`src/index.css:4` --color-navy) | ☑ |
| T1.5 | Drag-drop native | `src/components/upload/Dropzone.tsx` | `tauri://file-drop` event terpasang (bukan cuma TODO comment) — grep `file-drop` di file ini | ☑ |
| T1.6 | Script setup FFmpeg lokal | `scripts/setup-ffmpeg.sh`, `scripts/setup-ffmpeg.ps1` | Script jalan, `src-tauri/binaries/` terisi, checksum diverifikasi | ☑ |
| T1.7 (baru) | Swap CI + script lokal ke build FFmpeg LGPL | `.github/workflows/*.yml`, script setup lokal (T1.6) | Asset name diganti dari `*-gpl.tar.xz`/`.zip` ke `*-lgpl.tar.xz`/`.zip` (tag rilis TETAP SAMA, `autobuild-2026-08-19-19-21`, supaya versi FFmpeg tidak ikut berubah); `cargo tauri build` tetap sukses kedua OS | ☑ |
| T1.8 (baru) | Sanity-check codec build LGPL | — | `ffmpeg -codecs` dari binary LGPL yang di-download menunjukkan `libmp3lame`, `aac`, `flac` tersedia — tempel output sebagai bukti | ☑ |

## Fase 2+ (belum dimulai)

| # | Task | Target file | Done criteria | Status |
|---|---|---|---|:---:|
| T2.1 | Decode audio dari path lokal (plugin-fs) + pasang dep WaveSurfer | `src/lib/audioDecode.ts`, `package.json`, `src-tauri/{Cargo.toml,src/lib.rs,src/capabilities/default.json,tauri.conf.json}` | `npm run build` + `cargo build --features tauri-runtime` hijau; `decodeAudioFromPath` tidak throw | ☑ |
| T2.2 | Render waveform via WaveSurfer | `src/components/workspace/WaveformView.tsx` | Waveform tampil (bukan placeholder) setelah file dimuat; `npm run build` hijau | ☐ |
| T2.3 | Region trim drag (RegionsPlugin) + two-way sync store | `src/components/workspace/WaveformView.tsx` | Drag handle mengubah `region` di store; edit TimeInput mengubah region di waveform | ☐ |
| T2.4 | Polish: interact:false, styling token, clamp durasi | `src/components/workspace/WaveformView.tsx` | Klik waveform tidak autoplay bentrok Preview; region di-clamp ke durasi | ☐ |
| T3.x | Wiring backend Rust penuh ke UI | `src/lib/ipc.ts` | — | ☐ |
| T4.x | Integrasi frontend↔backend penuh | — | — | ☐ |
| T5.x | Packaging & release | `.github/workflows/` | — | ☐ |

## Backlog v2 (jangan dikerjakan sekarang)
- Multi-region trim, undo/redo, equalizer, support macOS

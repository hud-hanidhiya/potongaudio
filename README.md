# PotongAudio

**Potong audio dengan presisi.**

Aplikasi desktop offline cross-platform (Windows x86_64 / Linux x86_64) untuk
trim, efek audio (fade, gain, speed), dan konversi format audio.

## Status Proyek

**MVP v1.0 — SELESAI (Fase 0–3 + pipeline rilis).** Seluruh fitur inti
sudah terimplementasi dan terverifikasi:

- `cargo test` — 42 unit test lulus (tanpa perlu binary FFmpeg asli)
- `cargo build --features tauri-runtime` — lulus
- Lint gate: `npm run lint` (eslint) + `cargo fmt --check` + `cargo clippy --all-targets --features tauri-runtime -- -D warnings` — bersih
- `cargo tauri build` — lulus, installer NSIS (Windows) + AppImage & `.deb` (Linux)
- `npm run build` — lulus (tsc strict + vite)
- CI `build-verify.yml` hijau di Windows & Linux; push tag `v*` otomatis
  mempublikasikan GitHub Release (T5.1)

Hasil PoC lengkap ada di [`POC_FINDINGS.md`](./POC_FINDINGS.md).

## Fitur

- **Probe audio** — metadata file (durasi, sample rate, channel, format) via FFprobe sidecar
- **Waveform interaktif** — render via WaveSurfer.js; region trim bisa di-drag
  langsung di waveform, dua-arah tersinkron dengan input waktu (MM:SS.ms)
- **Trim** — potong region dengan akurasi milidetik (`atrim`, teruji end-to-end)
- **Efek** — fade in/out, gain (−20 s/d +20 dB), speed (filter `atempo` dengan
  chaining otomatis di luar rentang 0.5–2.0×)
  - *Catatan:* di build FFmpeg **LGPL** (default v1) efek speed tetap mengubah
    pitch — preservasi pitch butuh `rubberband` (GPL), belum dibundel. Preview
    speed juga belum ada (lihat Roadmap).
- **Preview** — putar region via Web Audio API dengan gain & fade diterapkan
  secara real-time sebelum export
- **Cancel** — batalkan export yang sedang berjalan via `JobRegistry`
- **Progress streaming** — event `export://progress` / `export://done` / `export://error`
- **Save dialog native** — pilih lokasi output via Tauri dialog
- **AC-04** — export ditolak kalau path output sama dengan file sumber
  (tidak overwrite source diam-diam)
- **100% Offline** — semua processing lulus di mesin lokal, tanpa server

Keputusan scope: **multi-region trim, undo/redo, equalizer, dan macOS ditunda
ke v2**. Code signing ditunda (installer rilis **unsigned** — di Windows muncul
peringatan SmartScreen).

## Tech Stack

### Frontend
- **React 18** + **TypeScript** (strict)
- **Vite** — build tool
- **Zustand** — state management
- **Tailwind CSS** — styling
- **WaveSurfer.js** — waveform interaktif + region plugin (via `wavesurfer.js/plugins/regions`)

### Backend
- **Rust** — core processing
- **Tauri v2** — desktop framework (dependency `optional` via feature `tauri-runtime`)
- **FFmpeg** — audio encoding/decoding (sidecar binary, **BtbN LGPL build**)
- **FFprobe** — audio metadata probing (sidecar binary, BtbN LGPL build)
- **Tokio** — async runtime
- **@tauri-apps/plugin-fs** — baca bytes file lokal untuk decode waveform/preview
- **@tauri-apps/plugin-dialog** — save/open dialog native

## Prerequisites

- Node.js 18+ dan npm
- Rust (toolchain stable) + build tools sistem (WebKitGTK 4.1 di Linux, MSVC di Windows)
- FFmpeg/FFprobe binary di `src-tauri/binaries/` (target-triple naming):
  - Windows: `ffmpeg-x86_64-pc-windows-msvc.exe` / `ffprobe-x86_64-pc-windows-msvc.exe`
  - Linux: `ffmpeg-x86_64-unknown-linux-gnu` / `ffprobe-x86_64-unknown-linux-gnu`
- Setup pertama klon baru: jalankan `npm run setup:ffmpeg` untuk otomatis
  mendownload & memverifikasi checksum FFmpeg/FFprobe **build LGPL** BtbN
  (tag `autobuild-2026-08-19-19-21`) — versi dipin identik di Windows & Linux.

## Development

```bash
# dari root repo:
npm install
npm run setup:ffmpeg              # download & verifikasi binary FFmpeg/FFprobe LGPL

# backend (folder src-tauri):
cd src-tauri
cargo test                        # unit test cepat, tanpa dependency Tauri

# jalankan app (kembali ke root repo):
cd ..
cargo tauri dev                   # feature tauri-runtime aktif otomatis via tauri.conf.json
```

## Build Production

```bash
npm run build                    # frontend -> dist/
cd src-tauri
cargo tauri build                # installer: NSIS (Windows), AppImage + .deb (Linux)
```

Output di `src-tauri/target/release/bundle/`:
- Windows: `nsis/PotongAudio_0.1.0_x64-setup.exe`
- Linux: `appimage/` dan `deb/`

### Rilis otomatis (GitHub Release)

Push tag `v*` (mis. `git tag v0.1.0 && git push origin v0.1.0`) memicu
`build-verify.yml` membangun kedua OS lalu job `publish-release` mempublikasikan
GitHub Release berisi installer NSIS, AppImage, dan `.deb`. Installer rilis
**unsigned** (T0.7 code signing ditunda).

**Rilis pertama sudah terbit:** [v0.1.0](../../releases/latest) —
`PotongAudio_0.1.0_x64-setup.exe` (71.5 MB, Windows),
`PotongAudio_0.1.0_amd64.AppImage` (163 MB) & `.deb` (97.9 MB, Linux).
SHA-256 tiap asset tercantum di halaman release dan di
[`docs/08-qa-release-checklist.md`](./docs/08-qa-release-checklist.md).

## Struktur Project

```
potongaudio/
├── package.json                 # config frontend di root (bukan di src/)
├── vite.config.ts
├── tsconfig.json
├── src/                         # Frontend React/TypeScript
│   ├── components/
│   │   ├── upload/Dropzone.tsx
│   │   └── workspace/{Toolbar,WaveformView,TimeInput,ExportDock}.tsx
│   ├── lib/                     # ipc.ts, audioDecode, previewEngine, soundtouch
│   ├── store/                   # useAudioStore, useExportStore
│   ├── types/audio.types.ts     # kontrak data frontend <-> backend
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                   # Backend Rust
│   ├── build.rs
│   ├── Cargo.toml               # lib.name = potong_audio_lib
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── binaries/                # ffmpeg/ffprobe sidecar (target-triple naming)
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── error.rs
│       ├── commands/            # export.rs, probe.rs, version.rs
│       └── ffmpeg/              # filter_builder.rs, progress_parser.rs, sidecar.rs
├── scripts/                     # setup-ffmpeg.sh / .ps1 + run-setup-ffmpeg.mjs
├── docs/                        # dokumentasi perencanaan (workflow kit 2-tier)
├── landing/                     # landing page (static HTML)
├── POC_FINDINGS.md              # hasil & keputusan PoC Fase 0
└── README.md
```

## Dokumentasi

- [`POC_FINDINGS.md`](./POC_FINDINGS.md) — hasil PoC Fase 0 dan keputusan teknis
- [`docs/00-guardrails.md`](./docs/00-guardrails.md) — risk trigger & aturan wajib
- [`docs/03-spec.md`](./docs/03-spec.md) — spec fitur & acceptance criteria
- [`docs/04-architecture-notes.md`](./docs/04-architecture-notes.md) — arsitektur & keputusan
- [`docs/06-task-plan.md`](./docs/06-task-plan.md) — task plan (status tiap fase)
- [`docs/08-qa-release-checklist.md`](./docs/08-qa-release-checklist.md) — QA pra-rilis

## Roadmap

### v1.0 (MVP) — SELESAI
- [x] Sidecar FFmpeg/FFprobe (build **LGPL**)
- [x] Trim end-to-end + progress streaming + cancel
- [x] Waveform interaktif + region trim drag (WaveSurfer.js)
- [x] Fade in/out, gain, speed dari UI
- [x] Preview real-time (gain/fade)
- [x] Save dialog native & export MP3/WAV/M4A/FLAC/M4R
- [x] AC-04: tolak output == input
- [x] Pipeline rilis (tag `v*` → GitHub Release)

### v1.1 / v2 (ditunda)
- [ ] Multi-region trim
- [ ] Undo/redo
- [ ] Equalizer
- [ ] Preservasi pitch di export (butuh `rubberband`/time-stretch — di luar build LGPL)
- [ ] Preview speed (time-stretch) — pilih library
- [ ] Code signing Windows (T0.7)
- [ ] Support macOS

## Lisensi

Aplikasi didistribusikan di bawah lisensi **GPL-3.0** (lihat `LICENSE`).
Binary FFmpeg/FFprobe yang di-bundel adalah **build LGPL** BtbN — kompatibel
untuk dibundel dalam distribusi GPL-3.0.

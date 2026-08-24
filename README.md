# PotongAudio

**Potong audio dengan presisi.**

Aplikasi desktop offline cross-platform (Windows x86_64 / Linux x86_64) untuk
trim, efek audio (fade, gain, speed), dan konversi format audio.

## Status Proyek

**Fase 0 (PoC & Validasi Teknis) — SELESAI.** Semua PoC kritis sudah
diverifikasi di Windows x86_64: sidecar FFmpeg/FFprobe berjalan, trim
end-to-end menghasilkan file valid, progress streaming live, installer
NSIS ter-build. Hasil lengkap ada di [`POC_FINDINGS.md`](./POC_FINDINGS.md).

- `cargo test` — 39 unit test lulus (tanpa perlu binary FFmpeg asli)
- `cargo build --features tauri-runtime` — lulus
- `cargo tauri build` — lulus, installer NSIS `PotongAudio_0.1.0_x64-setup.exe` (80 MB)
- `npm run build` — lulus (tsc + vite)

## Fitur (kondisi sekarang)

- **Probe audio** — metadata file (durasi, sample rate, channel, format) via FFprobe sidecar
- **Trim** — potong region dengan akurasi milidetik (`atrim`, teruji end-to-end)
- **Efek** — fade in/out, gain, speed/pitch (`filter_builder` siap; UI sudah ada di `Toolbar.tsx`)
- **Cancel** — batalkan export yang sedang berjalan via `JobRegistry`
- **Progress streaming** — event `export://progress` / `export://done` / `export://error`
- **100% Offline** — semua processing lokal, tidak ada upload ke server

Keputusan scope Fase 0: **multi-region trim, undo/redo, dan equalizer ditunda
ke v2**. Target Fase 0 hanya Windows x86_64 + Linux x86_64 (macOS di luar
scope). Code signing Windows ditunda ke Fase 5.

## Tech Stack

### Frontend
- **React 18** + **TypeScript** (strict)
- **Vite** — build tool
- **Zustand** — state management
- **Tailwind CSS** — styling

### Backend
- **Rust** — core processing
- **Tauri v2** — desktop framework
- **FFmpeg** — audio encoding/decoding (sidecar binary, BtbN gpl build)
- **FFprobe** — audio metadata probing (sidecar binary)
- **Tokio** — async runtime

## Prerequisites

- Node.js 18+ dan npm
- Rust (toolchain stable MSVC) + build tools (lihat build host Windows)
- FFmpeg/FFprobe binary di `src-tauri/binaries/` (sudah disediakan untuk
  `x86_64-pc-windows-msvc`; untuk Linux perlu download build `linux64-gpl`
  dengan nama `ffmpeg-x86_64-unknown-linux-gnu` / `ffprobe-x86_64-unknown-linux-gnu`)
- Setup pertama klon baru: jalankan `npm run setup:ffmpeg` untuk otomatis
  mendownload & memverifikasi FFmpeg/FFprobe build BtbN (gpl) yang sama
  dipakai di CI. Script tersedia untuk Linux/macOS (`.sh`) dan Windows (`.ps1`).

## Development

```bash
# Install dependencies frontend
npm install

# Test backend (cepat, tanpa dependency Tauri)
cd src-tauri
cargo test

# Build aplikasi Tauri (feature tauri-runtime aktif otomatis via tauri.conf.json)
cargo tauri dev
```

## Build Production

```bash
npm run build                    # frontend -> dist/
cd src-tauri
cargo tauri build                # menghasilkan installer (NSIS di Windows, AppImage di Linux)
```

Output di `src-tauri/target/release/bundle/`:
- Windows: `nsis/PotongAudio_0.1.0_x64-setup.exe`
- Linux: `appimage/` (butuh host Linux)

## Struktur Project

```
potongaudio/
├── package.json                 # config frontend di root (bukan di src/)
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── src/                         # Frontend React/TypeScript
│   ├── components/
│   │   ├── upload/Dropzone.tsx
│   │   └── workspace/{Toolbar,WaveformView,TimeInput,ExportDock,StatusBadge}.tsx
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
│   ├── icons/
│   ├── test-fixtures/           # fake_ffmpeg.{sh,bat}, sample/
│   └── src/
│       ├── main.rs              # memanggil potong_audio_lib::run()
│       ├── lib.rs
│       ├── error.rs
│       ├── commands/            # export.rs, probe.rs, version.rs
│       └── ffmpeg/              # filter_builder.rs, progress_parser.rs, sidecar.rs
├── docs/                        # dokumentasi perencanaan
├── landing/                     # landing page (static HTML)
├── POC_FINDINGS.md              # hasil & keputusan PoC Fase 0
└── README.md
```

## Dokumentasi

- [`POC_FINDINGS.md`](./POC_FINDINGS.md) — hasil PoC Fase 0 dan keputusan teknis
- [`docs/PLAN_AUDIO_CUTTER.md`](./docs/PLAN_AUDIO_CUTTER.md) — arsitektur, tech stack, roadmap fase
- [`docs/TECH_IMPLEMENTATION_PLAN.md`](./docs/TECH_IMPLEMENTATION_PLAN.md) — struktur detail, kontrak data/IPC, task breakdown per fase

## Roadmap

### v1.0 (MVP) — setelah Fase 0
- [x] Sidecar FFmpeg/FFprobe (PoC Fase 0)
- [x] Trim end-to-end + progress streaming (PoC Fase 0)
- [ ] UI trim & waveform interaktif (WaveSurfer.js)
- [ ] Fade in/out, gain, speed/pitch dari UI
- [ ] Save dialog native & export MP3/WAV/M4A/FLAC/M4R

### v1.1
- [ ] Multi-region trim
- [ ] Undo/redo
- [ ] Equalizer 
- [ ] Batch processing
- [ ] Keyboard shortcuts

## Testing

### Frontend
```bash
npm run build       # tsc --noEmit + vite build
```

### Backend (Rust)
```bash
cd src-tauri
cargo test          # 39 unit test
```

## Lisensi

Distributed under the **GPL-3.0** License. Lihat file lisensi untuk detail.

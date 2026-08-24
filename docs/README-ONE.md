# PotongAudio (Desktop App)

> **Status: Fase 0 (PoC & Validasi Teknis) SELESAI.** Aplikasi sudah bisa
> di-build menjadi installer NSIS, sidecar FFmpeg/FFprobe berjalan, dan
> seluruh unit test lulus. README ini mencerminkan kondisi setelah Fase 0
> selesai — lihat `POC_FINDINGS.md` untuk bukti dan keputusan teknis.

Aplikasi desktop cross-platform (Windows x86_64 / Linux x86_64) untuk
memotong, memberi efek (fade, gain, speed), dan mengonversi format file
audio — sepenuhnya offline, tanpa server eksternal.

## Arsitektur

Hybrid: **React (frontend/UI)** + **Rust via Tauri (backend/processing)**.

- UI, waveform, dan preview real-time berjalan di frontend (Web Audio API + WaveSurfer.js).
- Trim, filter efek, dan encode final dilakukan oleh **FFmpeg native**, dipanggil dari Rust command (bukan FFmpeg.wasm) — dibundel sebagai Tauri sidecar.
- Alasan pemilihan arsitektur ini (vs FFmpeg.wasm murni) ada di `PLAN_AUDIO_CUTTER.md`.

## Dokumen Perencanaan

| Dokumen | Isi |
|---|---|
| [`PLAN_AUDIO_CUTTER.md`](./PLAN_AUDIO_CUTTER.md) | Arsitektur, tech stack, roadmap fase, risiko terbuka |
| [`TECH_IMPLEMENTATION_PLAN.md`](./TECH_IMPLEMENTATION_PLAN.md) | Struktur project detail, kontrak data/IPC, task breakdown per fase, strategi testing |
| [`POC_FINDINGS.md`](../POC_FINDINGS.md) | Hasil PoC Fase 0: jawaban eksplisit ya/tidak + ukuran installer |

## Keputusan Scope yang Sudah Dikunci (Fase 0)

- Nama aplikasi: **PotongAudio**; bundle identifier `com.potongaudio.app`
- Lisensi: **GPL-3.0** (mengizinkan bundling FFmpeg full-GPL)
- Single-region trim saja untuk v1; **multi-region ditunda ke v2**
- **Undo/redo ditunda ke v2** (cukup tombol Reset)
- **Equalizer ditunda ke v2** (tombol di-disable)
- Target platform Fase 0: **Windows x86_64 + Linux x86_64** (macOS di luar scope)
- Code signing Windows: **ditunda ke Fase 5** (T5.1)
- FFmpeg source: **BtbN GitHub Actions builds (gpl)**; bundle targets NSIS + AppImage

## Struktur Repo Saat Ini

```
potongaudio/
├── package.json                  # config frontend (di root, bukan di src/)
├── vite.config.ts                # root: 'src', outDir: '../dist'
├── tsconfig.json / tsconfig.node.json
├── src/                          (frontend React)
│   ├── App.tsx                   ✅ menampilkan versi FFmpeg sidecar + header "PotongAudio"
│   ├── main.tsx, index.css, index.html
│   ├── components/
│   │   ├── upload/Dropzone.tsx
│   │   └── workspace/{WaveformView,Toolbar,TimeInput,ExportDock,StatusBadge}.tsx
│   ├── store/{useAudioStore,useExportStore}.ts
│   ├── lib/{ipc,audioDecode,previewEngine,soundtouch}.ts
│   └── types/audio.types.ts
├── src-tauri/
│   ├── build.rs                  ✅ tauri_build::build() (feature-gated)
│   ├── Cargo.toml                ✅ lib.name = potong_audio_lib; feature tauri-runtime
│   ├── tauri.conf.json           ✅ valid; build.features = ["tauri-runtime"]
│   ├── capabilities/default.json ✅ shell:allow-execute untuk ffmpeg/ffprobe
│   ├── binaries/                 ✅ ffmpeg/ffprobe-x86_64-pc-windows-msvc(.exe)
│   ├── icons/                    ✅ di-generate dari icon-source.png
│   ├── test-fixtures/            ✅ fake_ffmpeg.sh + fake_ffmpeg.bat + sample/
│   └── src/
│       ├── main.rs               ✅ memanggil potong_audio_lib::run()
│       ├── lib.rs                ✅ invoke_handler: export/cancel/probe/version
│       ├── error.rs              ✅ 3 unit test lulus
│       ├── commands/
│       │   ├── export.rs         ✅ struct teruji + command Tauri (tauri-runtime)
│       │   ├── probe.rs          ✅ parser JSON teruji (7 test); command Tauri
│       │   ├── version.rs        ✅ get_ffmpeg_version (T0.1)
│       │   └── mod.rs
│       └── ffmpeg/
│           ├── filter_builder.rs  ✅ 16 unit test lulus
│           ├── progress_parser.rs ✅ 9 unit test lulus
│           ├── sidecar.rs         ✅ 6 test lulus (termasuk cancel Windows via .bat)
│           └── mod.rs
├── docs/                         # perencanaan
├── landing/index.html            # landing page static (lisensi sudah GPL-3.0)
├── POC_FINDINGS.md               # hasil PoC
└── README.md
```

## Status Verifikasi Kode

Simbol di tabel struktur di atas:
- ✅ = sudah dijalankan sungguhan dan lulus:
  - Rust: `cargo test` (**39 test**), `cargo build --features tauri-runtime`
  - Frontend: `tsc --noEmit --strict` + `vite build` produksi (51 modul)
  - `cargo tauri build` → installer NSIS berhasil di-build (80 MB)

Semua tanda ⚠️ "belum tervalidasi" pada versi pra-Fase 0 sudah ter-resolve:
modul declaration, `tauri.conf.json`, `capabilities`, command layer, dan
build script Tauri semuanya sudah compile penuh dengan feature `tauri-runtime`.

Dependency `tauri`/`tauri-plugin-shell`/`tauri-plugin-dialog` di
`Cargo.toml` tetap **optional**, diaktifkan lewat feature flag
`tauri-runtime` (di-set otomatis oleh `tauri.conf.json` → `build.features`).
`cargo test` (default) tetap cepat tanpa perlu resolve dependency Tauri.

## Menjalankan Test

**Backend (Rust) — cepat, tanpa Tauri:**
```bash
cd src-tauri
cargo test
```

**Backend — build aplikasi Tauri sungguhan:**
```bash
cd src-tauri
cargo build --features tauri-runtime
cargo tauri dev       # development
cargo tauri build     # release + installer
```

**Frontend:**
```bash
npm install
npm run build     # tsc --noEmit + vite build
```

## Status Fase 0 vs Checklist Validasi

- [x] `commands/mod.rs` + `ffmpeg/mod.rs` benar; `cargo build --features tauri-runtime` lulus (Windows; Linux menunggu host)
- [x] `npm run build` lulus tanpa error TypeScript/Vite
- [x] `cargo test` lulus 39 test, termasuk Windows tanpa WSL (pakai `fake_ffmpeg.bat`)
- [x] `fake_ffmpeg.bat` tersedia dan setara `fake_ffmpeg.sh`
- [x] T0.1: `ffmpeg -version` + `ffprobe` dari sidecar; versi tampil di UI
- [x] T0.2: Trim end-to-end menghasilkan file output valid (3.0s MP3 dari region 2000–5000ms)
- [x] T0.3: Progress streaming live (4 titik `out_time_us` selama trim file 5 menit)
- [x] T0.4: Installer Windows (NSIS) berhasil; ukuran tercatat di `POC_FINDINGS.md`
- [ ] Verifikasi runtime Linux x86_64 (butuh host Linux — build AppImage + sidecar linux64)
- [x] `POC_FINDINGS.md` terisi dengan keputusan eksplisit ya/tidak

## TODO Sebelum Fase 1 Selesai

- [ ] Verifikasi sidecar FFmpeg/FFprobe di Linux x86_64 (`ffmpeg-x86_64-unknown-linux-gnu`) + build AppImage
- [ ] Isi `decodeAudioFromPath` (`lib/audioDecode.ts`) — butuh `@tauri-apps/plugin-fs`
- [ ] Pasang WaveSurfer.js sungguhan di `WaveformView.tsx` (kerangka state sudah siap)
- [ ] Wiring drag-drop native `tauri://file-drop` di `Dropzone.tsx` (TODO T1.5)
- [ ] Code signing Windows (ditunda ke T5.1)
- [ ] `landing/index.html` — ganti URL GitHub placeholder `yourusername/potongaudio`

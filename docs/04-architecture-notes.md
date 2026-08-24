# Architecture Notes — Potong Audio

## Stack & alasannya
**Hybrid Tauri v2**: UI React/TypeScript, audio processing berat di Rust
memanggil **FFmpeg native via sidecar** (bukan FFmpeg.wasm).

Pivot dari rancangan awal (full web-stack + FFmpeg.wasm) karena: risiko
COOP/COEP untuk multi-thread WASM di webview Tauri, ukuran bundle WASM
tidak sesuai klaim awal, dan FFmpeg native lebih cepat tanpa kehilangan
kecepatan development UI (dibanding opsi full-native GUI yang lebih lambat
dikembangkan).

## Struktur proyek
```
potongaudio/
├── docs/                    (workflow kit ini, 00-09)
├── AGENTS.md                (auto-loaded Kilo Code)
├── .kilocode/rules/
├── PLAN_AUDIO_CUTTER.md, TECH_IMPLEMENTATION_PLAN.md, POC_FINDINGS.md  (histori detail)
├── landing/index.html
├── .github/workflows/       (CI build+verify Windows & Linux)
├── src/                     (frontend, di root repo — bukan nested)
│   ├── components/{upload,workspace,shared}/
│   ├── store/{useAudioStore,useExportStore}.ts
│   ├── lib/{ipc,audioDecode,previewEngine,soundtouch}.ts
│   └── types/audio.types.ts
└── src-tauri/
    ├── Cargo.toml            (dependency Tauri OPTIONAL via feature `tauri-runtime`)
    ├── tauri.conf.json
    ├── capabilities/default.json
    ├── binaries/              (FFmpeg/FFprobe — TIDAK di-commit, di-download saat build)
    └── src/
        ├── main.rs, lib.rs
        ├── error.rs
        ├── commands/{export,probe,version}.rs
        └── ffmpeg/{filter_builder,progress_parser,sidecar}.rs
```

## Keputusan teknis kunci

| Keputusan | Alasan | Alternatif dipertimbangkan |
|---|---|---|
| FFmpeg native via Rust sidecar, bukan FFmpeg.wasm | Performa + hindari masalah COOP/COEP | FFmpeg.wasm (ditolak) |
| `EffectParams` sebagai kontrak tunggal TS↔Rust | Satu source of truth preview vs export | State terpisah per-layer (ditolak) |
| Urutan filter dipaksa trim→speed→fade→volume | Fade-out harus dihitung dari durasi setelah speed berubah | — |
| `JobRegistry` pakai `Killable`/`Mutex<Option<CommandChild>>` | `CommandChild::kill()` by-value tidak kompatibel pola awal | Enum `ChildHandle{Tokio,Shell}` |
| Dependency Tauri `optional` via feature `tauri-runtime` | `cargo test` cepat tanpa runtime GUI | Semua dependency wajib (ditolak) |
| Versi FFmpeg di-pin sama persis Windows & Linux CI | PoC lintas OS sebanding | — |
| **Build FFmpeg: LGPL, bukan GPL full** | Scope v1 murni audio (tidak butuh `libx264`/`libx265`, satu-satunya alasan signifikan build harus GPL); mengurangi risiko lisensi distribusi | GPL full (dipakai saat Fase 0 PoC, sekarang digantikan); build custom minimal (tidak dipilih — LGPL off-the-shelf sudah cukup tanpa kerja konfigurasi tambahan) |

## Integration point & kontrak

| Sistem target | Protokol | Auth | Payload/interface |
|---|---|---|---|
| FFmpeg sidecar | Spawn proses + stdio (stderr untuk `-progress pipe:2`) | N/A (binary lokal) | Argumen CLI dari `filter_builder.rs`, output = file di disk |
| FFprobe sidecar | Spawn proses, stdout JSON | N/A | `-print_format json`, di-parse `parse_ffprobe_json` |
| Frontend ↔ Rust | Tauri IPC (`invoke`/`emit`) | N/A (proses lokal sama) | `EffectParams` (lihat `types/audio.types.ts` ↔ `commands/export.rs`), event `export://progress\|done\|error` |

## Perubahan data model / skema
Tidak ada database. "Kontrak" yang setara skema di project ini adalah
struct `EffectParams`:
```typescript
interface EffectParams {
  sourceFilePath: string;
  region: { startMs: number; endMs: number };
  gainDb: number;
  fade: { inMs: number; outMs: number };
  speed: { ratio: number; preservePitch: boolean };
  outputFormat: 'mp3' | 'wav' | 'm4a' | 'flac' | 'm4r';
  outputBitrateKbps?: number;
}
```
Perubahan ke struktur ini tunduk pada guardrail "Kontrak `EffectParams`"
di `00-guardrails.md`.

## Data flow
1. User pilih file → path lokal dari Tauri dialog/file-drop.
2. `probe_audio_file` (Rust, via `ffprobe`) → durasi/sample rate/channel.
3. `EffectParams` dibentuk di `useAudioStore`, region default = full durasi.
4. Preview: Web Audio API decode **hanya untuk preview**, gain/fade via
   `GainNode`, speed via time-stretch library (belum final).
5. Export: `EffectParams` → Rust `export_audio` → `filter_builder.rs` →
   sidecar FFmpeg native → progress di-stream ke UI → file ditulis
   langsung ke disk (tidak pernah full-load ke JS heap).

## Failure mode & recovery
- **Sidecar gagal spawn**: `AppError::SidecarSpawnFailed`, tidak ada retry
  otomatis — user diminta coba lagi manual.
- **FFmpeg exit non-zero di tengah proses**: `AppError::FfmpegExecutionFailed`
  dengan stderr tail, file output parsial dianggap tidak valid (belum ada
  cleanup otomatis file parsial — **risiko terbuka**, cek saat QA).
- **Cancel di tengah proses**: `JobRegistry.cancel()` kill proses via
  `Killable`, job di-unregister. Belum ada cleanup file output parsial
  otomatis — sama seperti di atas.
- **Koneksi/proses terputus**: tidak relevan (semua lokal, tidak ada network
  runtime), tapi disk penuh/permission error ditangani sebagai
  `AppError::OutputWriteFailed`.

## Risiko yang diketahui
- **Ukuran AppImage Linux (177MB) vs NSIS Windows (80MB)** untuk FFmpeg
  yang sama — penyebab belum dikonfirmasi.
- **Binary FFmpeg/FFprobe tidak di-commit** — developer baru butuh script
  setup lokal (belum dibuat).
- **Preview vs export bisa drift** kalau kurva fade Web Audio API dan
  FFmpeg `afade` tidak identik matematis — belum ada test otomatis untuk
  kesetaraan ini.
- **File output parsial saat cancel/error tidak dibersihkan otomatis** —
  ditemukan saat menulis Failure Mode di atas, belum ada di task plan
  manapun sebelumnya.
- **Lisensi FFmpeg** — sudah dipindah ke build LGPL (lihat tabel Keputusan
  teknis kunci di atas dan `00-guardrails.md`). Risiko residual: build
  LGPL BtbN didokumentasikan "most prominently" kehilangan `libx264`/
  `libx265` — frasa ini menyiratkan kemungkinan ada library GPL-only lain
  yang ikut hilang, tidak dirinci lengkap oleh BtbN. **Wajib** jalankan
  `ffmpeg -codecs`/`-encoders` pada build LGPL yang dipakai untuk
  konfirmasi `libmp3lame`/`aac`/`flac` tetap tersedia — belum dilakukan,
  jadi masih risiko terbuka sampai dicek.
- **Penghematan ukuran dari GPL→LGPL lebih kecil dari perkiraan lama** —
  proyeksi "~50%+" di `PLAN_AUDIO_CUTTER.md` adalah spekulasi belum
  terverifikasi. Data resmi rilis terbaru BtbN (win64 static): GPL
  162.8MB vs LGPL 141.6MB, selisih riil **~13%**. Jangan berharap
  penurunan besar pada ukuran installer hanya dari swap lisensi ini;
  anomali AppImage Linux (177MB) kemungkinan besar tetap perlu
  diinvestigasi terpisah, bukan otomatis terselesaikan oleh swap ini.
- **Mismatch lisensi project sendiri** — belum dikonfirmasi status final.

## Path file yang terdampak (task aktif saat ini)
Diisi ulang tiap kali ada implementation plan baru di `04a-implementation-plan.md`
— jangan biarkan bagian ini statis/usang.

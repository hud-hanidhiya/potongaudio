# TECH_IMPLEMENTATION_PLAN.md
## PotongAudio — Technical Implementation Plan

**Companion dokumen dari:** `PLAN_AUDIO_CUTTER.md` (v3)
**Tujuan dokumen ini:** breakdown teknis siap-eksekusi — struktur project, kontrak data/IPC, urutan implementasi per fase, dan definisi "selesai" untuk tiap task.

---

## 1. Struktur Project

```
potong-audio/
├── package.json / vite.config.ts / tsconfig*.json   # config di root (bukan di src/)
├── src/                          # Frontend (React)
│   ├── components/
│   │   ├── upload/
│   │   │   └── Dropzone.tsx
│   │   ├── workspace/
│   │   │   ├── WaveformView.tsx
│   │   │   ├── Toolbar.tsx
│   │   │   ├── TimeInput.tsx
│   │   │   └── ExportDock.tsx
│   │   └── shared/
│   │       └── StatusBadge.tsx   # (opsional, reuse pola dari proyek lain)
│   ├── store/
│   │   ├── useAudioStore.ts      # Zustand: file, region, effect params
│   │   └── useExportStore.ts     # Zustand: progress, status export
│   ├── lib/
│   │   ├── audioDecode.ts        # Web Audio API decode utk waveform
│   │   ├── previewEngine.ts      # Web Audio graph utk preview (gain, fade)
│   │   ├── soundtouch.ts         # Wrapper SoundTouch.js utk preview speed/pitch
│   │   └── ipc.ts                # Wrapper semua invoke() ke Rust commands
│   ├── types/
│   │   └── audio.types.ts        # Semua @typedef / type Effect Params, dll
│   └── App.tsx
│
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── main.rs               # memanggil potong_audio_lib::run()
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── export.rs         # Command: export_audio
│   │   │   ├── probe.rs          # Command: probe_audio_file (metadata info)
│   │   │   └── version.rs        # Command: get_ffmpeg_version (T0.1)
│   │   ├── ffmpeg/
│   │   │   ├── mod.rs
│   │   │   ├── filter_builder.rs # Bangun filter graph string dari Effect Params
│   │   │   ├── progress_parser.rs# Parse stderr FFmpeg → persen progress
│   │   │   └── sidecar.rs        # Wrapper spawn proses FFmpeg sidecar
│   │   └── error.rs               # Error type terpusat
│   ├── binaries/                  # FFmpeg sidecar binaries (per-target, lihat Section 4)
│   ├── test-fixtures/             # fake_ffmpeg.{sh,bat} + sample audio utk test
│   └── tauri.conf.json
│
├── POC_FINDINGS.md                # Output Fase 0 (selesai)
└── PLAN_AUDIO_CUTTER.md
```

**Konvensi:** frontend pakai TypeScript native types (bukan JSDoc, karena project ini React+TS dari awal — beda dengan PPKEK/NK Tools yang plain JS+JSDoc). Backend Rust pakai `struct` + `serde` untuk serialisasi otomatis ke/dari JSON saat lewat IPC.

---

## 2. Kontrak Data (Effect Parameter Contract)

Ini objek tunggal yang jadi *source of truth*, dikirim dari frontend ke Rust saat export. Didefinisikan di dua sisi (harus identik secara struktur):

**Frontend — `src/types/audio.types.ts`**
```typescript
export interface EffectParams {
  sourceFilePath: string;       // path file asli di disk (bukan blob)
  region: {
    startMs: number;
    endMs: number;
  };
  gainDb: number;                // -20 s/d +20
  fade: {
    inMs: number;                // 0 = tidak ada fade in
    outMs: number;
  };
  speed: {
    ratio: number;                // 0.25 - 4.0, 1.0 = normal
    preservePitch: boolean;
  };
  outputFormat: 'mp3' | 'wav' | 'm4a' | 'flac' | 'm4r';
  outputBitrateKbps?: number;    // relevan utk mp3/m4a
}
```

**Backend — `src-tauri/src/commands/export.rs`**
```rust
#[derive(serde::Deserialize, Debug)]
pub struct EffectParams {
    pub source_file_path: String,
    pub region: Region,
    pub gain_db: f32,
    pub fade: Fade,
    pub speed: Speed,
    pub output_format: OutputFormat,
    pub output_bitrate_kbps: Option<u32>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Region { pub start_ms: u64, pub end_ms: u64 }

#[derive(serde::Deserialize, Debug)]
pub struct Fade { pub in_ms: u64, pub out_ms: u64 }

#[derive(serde::Deserialize, Debug)]
pub struct Speed { pub ratio: f32, pub preserve_pitch: bool }

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat { Mp3, Wav, M4a, Flac, M4r }
```

**Aturan penting:** field naming di TS pakai `camelCase`, di Rust pakai `snake_case` — serde otomatis handle konversi ini asal struct Rust tidak diberi `#[serde(rename_all = "camelCase")]` yang salah arah. **Tetapkan `#[serde(rename_all = "camelCase")]` di level struct Rust** supaya JSON yang diterima cocok dengan bentuk objek JS tanpa mapping manual.

---

## 3. Kontrak IPC (Tauri Commands & Events)

### 3.1 Commands (Frontend → Rust, request-response)

| Command | Input | Output | Kegunaan |
|---|---|---|---|
| `probe_audio_file` | `{ path: string }` | `{ durationMs, sampleRate, channels, format }` | Ambil metadata file setelah upload, sebelum decode penuh di JS |
| `export_audio` | `EffectParams` | `{ outputPath: string }` (atau error) | Trigger seluruh proses trim/effect/encode |
| `cancel_export` | `{ jobId: string }` | `{ cancelled: boolean }` | Batalkan proses FFmpeg yang sedang berjalan |

### 3.2 Events (Rust → Frontend, streaming)

| Event Name | Payload | Kegunaan |
|---|---|---|
| `export://progress` | `{ jobId: string, percent: number }` | Update progress bar real-time |
| `export://done` | `{ jobId: string, outputPath: string }` | Trigger UI success state |
| `export://error` | `{ jobId: string, message: string }` | Trigger UI error state dengan pesan jelas |

**Contoh wrapper frontend — `src/lib/ipc.ts`:**
```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { EffectParams } from '../types/audio.types';

export async function exportAudio(
  params: EffectParams,
  onProgress: (percent: number) => void
): Promise<string> {
  const jobId = crypto.randomUUID();

  const unlistenProgress = await listen<{ jobId: string; percent: number }>(
    'export://progress',
    (e) => { if (e.payload.jobId === jobId) onProgress(e.payload.percent); }
  );

  try {
    const result = await invoke<{ outputPath: string }>('export_audio', {
      jobId,
      params,
    });
    return result.outputPath;
  } finally {
    unlistenProgress();
  }
}
```

**Alasan pakai `jobId`:** kalau user export dua file berturut-turut cepat (atau cancel lalu retry), event listener bisa filter payload berdasarkan `jobId` supaya tidak salah update progress ke job yang sudah tidak relevan.

---

## 4. FFmpeg Sidecar — Detail Setup

### 4.1 Binary per Platform
```
src-tauri/binaries/
├── ffmpeg-x86_64-pc-windows-msvc.exe
├── ffmpeg-x86_64-apple-darwin
├── ffmpeg-aarch64-apple-darwin
└── ffmpeg-x86_64-unknown-linux-gnu
```
Naming ini mengikuti konvensi target-triple yang diwajibkan Tauri untuk `externalBin`.

### 4.2 Konfigurasi `tauri.conf.json`
```json
{
  "bundle": {
    "externalBin": ["binaries/ffmpeg"]
  }
}
```

### 4.3 Filter Graph Builder (Rust) — Logika Inti

Fungsi `build_filter_string(params: &EffectParams) -> String` menyusun argumen FFmpeg dari Effect Params:

```
Urutan filter (penting — urutan chain memengaruhi hasil):
1. atrim=start=Xms:end=Yms      (dari region)
2. atempo=ratio (chained jika ratio di luar 0.5-2.0)
3. afade=t=in:d=Xms              (jika fade.inMs > 0)
4. afade=t=out:st=...:d=Yms      (jika fade.outMs > 0, start time dihitung dari durasi hasil trim)
5. volume=XdB                    (dari gainDb)
```

**Catatan implementasi:** `afade` fade-out butuh parameter `st` (start time) yang dihitung, bukan konstan — ini titik yang paling sering salah kalau tidak hati-hati (harus dihitung dari durasi setelah trim & speed change, bukan durasi file asli).

### 4.4 Progress Parsing
FFmpeg dengan flag `-progress pipe:2` menulis output terstruktur `key=value` per baris ke stderr, termasuk `out_time_ms`. Rust command baca stream ini per baris, hitung `percent = out_time_ms / total_duration_ms * 100`, emit event tiap kali persen berubah signifikan (misal tiap kenaikan >1%, supaya tidak flood event).

---

## 5. Urutan Implementasi (Task Breakdown per Fase)

### Fase 0 — PoC (lihat `PLAN_AUDIO_CUTTER.md` Section 2 untuk detail exit criteria)
- [ ] T0.1 — Setup Tauri sidecar minimal, jalankan `ffmpeg -version` dari Rust command, tampilkan hasil di UI
- [ ] T0.2 — Command trim sederhana end-to-end (input file → output file terpotong), tanpa efek
- [ ] T0.3 — Progress streaming PoC dengan `-progress pipe:2`
- [ ] T0.4 — Build installer test di ketiga OS, ukur ukuran, cek signing macOS
- [ ] T0.5 — Tulis `POC_FINDINGS.md`

### Fase 1 — Setup Proyek
- [ ] T1.1 — Init Vite + React + TS + Tailwind + Zustand
- [ ] T1.2 — Init Tauri v2, integrasikan ke project Vite
- [ ] T1.3 — Definisikan `EffectParams` type di frontend (Section 2)
- [ ] T1.4 — Setup tema dark + design tokens (Navy/Cyan/Green)
- [ ] T1.5 — Halaman upload dropzone (drag-drop + file picker native via Tauri `dialog` plugin)

### Fase 2 — Waveform & Trimming (Frontend)
- [ ] T2.1 — Integrasi WaveSurfer.js + Regions plugin, render waveform dari file lokal
- [ ] T2.2 — Sinkronisasi drag handle region ↔ `useAudioStore`
- [ ] T2.3 — Preview playback terbatas region (Web Audio API, `AudioBufferSourceNode` dengan offset/duration)
- [ ] T2.4 — Komponen `TimeInput.tsx` (MM:SS.ms), dua arah dengan region
- [ ] T2.5 — Integrasi SoundTouch.js untuk preview speed (tanpa pitch shift)
- [ ] T2.6 — Preview gain via `GainNode`, preview fade via `GainNode` automation (`linearRampToValueAtTime`)

### Fase 3 — Rust Backend
- [ ] T3.1 — Struct `EffectParams` + serde di Rust, cocokkan dengan TS type
- [ ] T3.2 — `filter_builder.rs` — implementasi logika Section 4.3
- [ ] T3.3 — `sidecar.rs` — spawn proses FFmpeg, kirim argumen dari filter builder
- [ ] T3.4 — `progress_parser.rs` — parse stderr, emit event `export://progress`
- [ ] T3.5 — Command `probe_audio_file` — ambil metadata pakai `ffprobe` (biasanya satu paket dengan FFmpeg)
- [ ] T3.6 — Handling khusus format M4R: encode ke M4A → rename → cek/inject metadata `stik`
- [ ] T3.7 — Error handling terpusat (`error.rs`) — pastikan semua error dari FFmpeg (file corrupt, format tidak didukung, disk penuh) diteruskan sebagai pesan yang bisa ditampilkan ke user, bukan raw stderr

### Fase 4 — Integrasi Frontend ↔ Backend
- [ ] T4.1 — `src/lib/ipc.ts` — wrapper `invoke`/`listen` (Section 3.1)
- [ ] T4.2 — `useExportStore.ts` — state progress, status (idle/running/done/error)
- [ ] T4.3 — Hubungkan tombol Save → `exportAudio()` → render progress bar
- [ ] T4.4 — UI error state dengan pesan dari `export://error`
- [ ] T4.5 — Native save dialog — user pilih lokasi simpan sebelum/sesudah proses (tentukan UX: pilih lokasi dulu baru proses, atau proses ke temp lalu "Save As")
- [ ] T4.6 — Command `cancel_export` — tombol cancel saat progress berjalan

### Fase 5 — Packaging & Release
- [ ] T5.1 — Code signing Windows (opsional tapi disarankan — hindari warning SmartScreen)
- [ ] T5.2 — Code signing + notarization macOS (wajib, termasuk FFmpeg sidecar binary)
- [ ] T5.3 — Build installer 3 platform (`.exe`, `.dmg`, `.AppImage`)
- [ ] T5.4 — Smoke test matrix: upload → trim → semua efek → export tiap format → verifikasi file hasil bisa diputar

---

## 6. Testing Strategy

| Layer | Tool | Cakupan |
|---|---|---|
| Frontend unit | Vitest | Logika store (Zustand), konversi waktu (ms ↔ MM:SS.ms), filter param builder di sisi TS (jika ada) |
| Rust unit | `cargo test` | `filter_builder.rs` (paling kritis — banyak edge case: fade tanpa trim, trim tanpa fade, speed ekstrem) |
| Integrasi Rust | `cargo test` + sample audio file kecil di `test-fixtures/` | End-to-end: params → filter string → jalankan FFmpeg beneran → assert file output valid (durasi, format) |
| E2E manual | Smoke test matrix (Fase 5) | Karena Tauri app susah di-automated-test penuh tanpa setup tambahan (WebDriver), untuk v1 cukup manual checklist per platform |

**Edge case wajib di-test untuk `filter_builder.rs`:**
- Fade out dengan speed change (start time fade harus dihitung dari durasi *setelah* speed diterapkan, bukan durasi asli)
- Region di ujung file (end_ms mendekati/melebihi durasi asli — harus di-clamp)
- Gain 0dB (filter `volume` sebaiknya di-skip sama sekali dari chain, bukan tetap disertakan sebagai no-op)
- Speed ratio > 2.0 atau < 0.5 (perlu chaining `atempo`)

---

## 7. Definisi "Selesai" per Fase (Exit Criteria)

- **Fase 0 selesai** jika: PoC trim+progress+packaging jalan di ketiga OS, `POC_FINDINGS.md` terisi lengkap, keputusan FFmpeg build (full/minimal) sudah diambil.
- **Fase 2 selesai** jika: user bisa upload file, lihat waveform, drag region, dengar preview (termasuk fade/gain/speed) tanpa backend Rust terlibat sama sekali.
- **Fase 3 selesai** jika: `cargo test` untuk `filter_builder` lulus semua edge case di Section 6, dan command `export_audio` bisa dipanggil manual (misal via Tauri dev console) menghasilkan file benar.
- **Fase 4 selesai** jika: alur penuh upload → edit → export → save berjalan di dev mode tanpa error, progress bar akurat.
- **Fase 5 selesai** jika: installer tervalidasi jalan di mesin bersih (bukan mesin development) untuk ketiga OS.

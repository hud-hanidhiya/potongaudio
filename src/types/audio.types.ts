/**
 * Kontrak data utama frontend <-> backend.
 *
 * PENTING: struktur ini HARUS identik secara field dengan struct Rust di
 * `src-tauri/src/commands/export.rs` (EffectParams/Region/Fade/Speed/
 * OutputFormat). Kalau salah satu berubah, ubah juga yang satunya —
 * lihat TECH_IMPLEMENTATION_PLAN.md Section 2 untuk kontrak lengkapnya.
 *
 * Naming: camelCase di sini, di-mapping otomatis oleh serde
 * `#[serde(rename_all = "camelCase")]` di sisi Rust — tidak perlu mapping
 * manual di kode manapun.
 */

export type OutputFormat = 'mp3' | 'wav' | 'm4a' | 'flac' | 'm4r';

export interface Region {
  startMs: number;
  endMs: number;
}

export interface Fade {
  inMs: number;
  outMs: number;
}

export interface Speed {
  ratio: number; // 0.25 - 4.0, 1.0 = normal
  preservePitch: boolean;
}

export interface EffectParams {
  sourceFilePath: string;
  region: Region;
  gainDb: number; // -20 s/d +20
  fade: Fade;
  speed: Speed;
  outputFormat: OutputFormat;
  outputBitrateKbps?: number;
}

/** Hasil dari command Rust `probe_audio_file`. */
export interface ProbeResult {
  durationMs: number;
  sampleRate: number;
  channels: number;
  formatName: string;
}

/** Status siklus hidup satu job export, dipakai di useExportStore. */
export type ExportStatus = 'idle' | 'running' | 'done' | 'error' | 'cancelled';

export interface ExportJobState {
  jobId: string | null;
  status: ExportStatus;
  percent: number;
  outputPath: string | null;
  errorMessage: string | null;
}

/** Metadata file yang sedang dibuka di workspace (hasil upload + probe). */
export interface LoadedAudioFile {
  path: string;
  fileName: string;
  probe: ProbeResult;
}

/** Bentuk default EffectParams saat file baru dibuka — region penuh, tanpa efek. */
export function createDefaultEffectParams(
  sourceFilePath: string,
  durationMs: number
): EffectParams {
  return {
    sourceFilePath,
    region: { startMs: 0, endMs: durationMs },
    gainDb: 0,
    fade: { inMs: 0, outMs: 0 },
    speed: { ratio: 1.0, preservePitch: true },
    outputFormat: 'mp3',
    outputBitrateKbps: 192,
  };
}

export const SUPPORTED_EXTENSIONS = ['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg', 'wma'] as const;
export type SupportedExtension = (typeof SUPPORTED_EXTENSIONS)[number];

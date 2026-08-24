/**
 * Satu-satunya tempat frontend memanggil `invoke`/`listen` Tauri.
 * Tujuan: kalau kontrak IPC berubah (nama command, bentuk payload), cukup
 * ubah di sini — komponen React tidak perlu tahu detail Tauri sama sekali.
 *
 * Kontrak command & event lengkap ada di TECH_IMPLEMENTATION_PLAN.md
 * Section 3.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { type EffectParams, type ProbeResult, SUPPORTED_EXTENSIONS } from '../types/audio.types';

// ---------------------------------------------------------------------
// get_ffmpeg_version
// ---------------------------------------------------------------------

/** Versi FFmpeg sidecar yang di-bundle, untuk verifikasi T0.1. */
export async function getFfmpegVersion(): Promise<string> {
  return invoke<string>('get_ffmpeg_version');
}

// ---------------------------------------------------------------------
// probe_audio_file
// ---------------------------------------------------------------------

export async function probeAudioFile(filePath: string): Promise<ProbeResult> {
  return invoke<ProbeResult>('probe_audio_file', { filePath });
}

// ---------------------------------------------------------------------
// export_audio + event streaming
// ---------------------------------------------------------------------

interface ExportAudioResult {
  outputPath: string;
}

interface ProgressEventPayload {
  jobId: string;
  percent: number;
}

interface DoneEventPayload {
  jobId: string;
  outputPath: string;
}

interface ErrorEventPayload {
  jobId: string;
  message: string;
}

export interface ExportAudioArgs {
  params: EffectParams;
  totalDurationMs: number;
  outputPath: string;
  onProgress?: (percent: number) => void;
}

/**
 * Menjalankan export penuh: kirim EffectParams ke Rust, dengarkan event
 * progress sampai selesai atau gagal, lalu kembalikan path file hasil.
 *
 * `jobId` di-generate di sini (bukan diterima dari luar) supaya listener
 * event bisa langsung difilter per-job tanpa pemanggil perlu tahu detail ini.
 * Kalau butuh cancel, simpan `jobId` yang dikembalikan lewat callback
 * `onJobStart` (lihat parameter opsional di bawah) lalu panggil `cancelExport`.
 */
export async function exportAudio(
  args: ExportAudioArgs & { onJobStart?: (jobId: string) => void }
): Promise<string> {
  const jobId = crypto.randomUUID();
  args.onJobStart?.(jobId);

  const unlistenFns: UnlistenFn[] = [];

  try {
    if (args.onProgress) {
      const unlisten = await listen<ProgressEventPayload>(
        'export://progress',
        (event) => {
          if (event.payload.jobId === jobId) {
            args.onProgress?.(event.payload.percent);
          }
        }
      );
      unlistenFns.push(unlisten);
    }

    const result = await invoke<ExportAudioResult>('export_audio', {
      jobId,
      params: args.params,
      totalDurationMs: args.totalDurationMs,
      outputPath: args.outputPath,
    });

    return result.outputPath;
  } finally {
    // Selalu lepas listener, baik sukses maupun gagal — mencegah leak kalau
    // user export berkali-kali dalam satu sesi aplikasi.
    for (const unlisten of unlistenFns) {
      unlisten();
    }
  }
}

/**
 * Varian low-level bagi caller yang ingin kontrol penuh atas event
 * done/error (mis. untuk ditampilkan di useExportStore), bukan cuma
 * menunggu promise resolve/reject. Dipakai oleh useExportStore, BUKAN
 * dipanggil langsung dari komponen React.
 */
export function subscribeExportEvents(
  jobId: string,
  handlers: {
    onProgress?: (percent: number) => void;
    onDone?: (outputPath: string) => void;
    onError?: (message: string) => void;
  }
): Promise<UnlistenFn[]> {
  const subscriptions: Promise<UnlistenFn>[] = [];

  if (handlers.onProgress) {
    subscriptions.push(
      listen<ProgressEventPayload>('export://progress', (e) => {
        if (e.payload.jobId === jobId) handlers.onProgress?.(e.payload.percent);
      })
    );
  }
  if (handlers.onDone) {
    subscriptions.push(
      listen<DoneEventPayload>('export://done', (e) => {
        if (e.payload.jobId === jobId) handlers.onDone?.(e.payload.outputPath);
      })
    );
  }
  if (handlers.onError) {
    subscriptions.push(
      listen<ErrorEventPayload>('export://error', (e) => {
        if (e.payload.jobId === jobId) handlers.onError?.(e.payload.message);
      })
    );
  }

  return Promise.all(subscriptions);
}

// ---------------------------------------------------------------------
// cancel_export
// ---------------------------------------------------------------------

export async function cancelExport(jobId: string): Promise<boolean> {
  return invoke<boolean>('cancel_export', { jobId });
}

// ---------------------------------------------------------------------
// Native save dialog (T4.5) — dipakai sebelum memanggil exportAudio untuk
// menentukan `outputPath`.
// ---------------------------------------------------------------------

export async function pickSaveLocation(
  suggestedFileName: string
): Promise<string | null> {
  const { save } = await import('@tauri-apps/plugin-dialog');
  const result = await save({ defaultPath: suggestedFileName });
  return result ?? null;
}

export async function pickOpenAudioFile(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const result = await open({
    multiple: false,
    filters: [
      { name: 'Audio', extensions: [...SUPPORTED_EXTENSIONS] },
    ],
  });
  return typeof result === 'string' ? result : null;
}

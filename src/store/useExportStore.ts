/**
 * Store untuk siklus hidup satu job export: idle -> running -> done/error/cancelled.
 * Terpisah dari useAudioStore karena punya lifecycle berbeda (event-driven
 * dari Rust lewat Tauri, bukan diubah langsung oleh interaksi user seperti
 * drag handle waveform).
 */

import { create } from 'zustand';
import type { ExportStatus } from '../types/audio.types';
import { cancelExport as ipcCancelExport, exportAudio, type ExportAudioArgs } from '../lib/ipc';

interface ExportStoreState {
  jobId: string | null;
  status: ExportStatus;
  percent: number;
  outputPath: string | null;
  errorMessage: string | null;

  startExport: (args: Omit<ExportAudioArgs, 'onProgress' | 'onJobStart'>) => Promise<void>;
  cancel: () => Promise<void>;
  reset: () => void;
}

const initialState = {
  jobId: null as string | null,
  status: 'idle' as ExportStatus,
  percent: 0,
  outputPath: null as string | null,
  errorMessage: null as string | null,
};

export const useExportStore = create<ExportStoreState>((set, get) => ({
  ...initialState,

  startExport: async (args) => {
    // AC-04: jangan biarkan output menimpa file sumber tanpa konfirmasi.
    // Guardrail "Operasi file destruktif" melarang auto-overwrite source —
    // kita TOLAK eksplisit (bukan confirm dialog) supaya aman secara default.
    //
    // M2: perbandingan case-insensitive HANYA untuk path bergaya Windows
    // (filesystem case-insensitive). Di Linux `Song.MP3` vs `song.mp3` adalah
    // file berbeda yang sah — jangan false-reject.
    const { outputPath } = args;
    const sourcePath = args.params.sourceFilePath;
    const windowsStyle =
      /^[a-zA-Z]:[\\/]/.test(outputPath) ||
      /^[a-zA-Z]:[\\/]/.test(sourcePath) ||
      (typeof navigator !== 'undefined' && /win/i.test(navigator.platform ?? ''));
    const samePath =
      outputPath === sourcePath ||
      (windowsStyle && outputPath.toLowerCase() === sourcePath.toLowerCase());
    if (samePath) {
      set({
        ...initialState,
        status: 'error',
        errorMessage:
          'Path output tidak boleh sama dengan file sumber. Pilih nama atau lokasi lain.',
      });
      return;
    }

    set({ ...initialState, status: 'running' });

    try {
      const outputPath = await exportAudio({
        ...args,
        onJobStart: (jobId) => set({ jobId }),
        onProgress: (percent) => set({ percent }),
      });
      set({ status: 'done', outputPath, percent: 100 });
    } catch (err) {
      // Kalau job sudah di-cancel oleh user sebelum promise reject sampai
      // di sini, jangan timpa status 'cancelled' yang sudah di-set oleh cancel().
      if (get().status === 'cancelled') return;

      const message = err instanceof Error ? err.message : String(err);
      set({ status: 'error', errorMessage: message });
    }
  },

  cancel: async () => {
    const { jobId } = get();
    if (!jobId) return;

    // M3: status 'cancelled' HANYA setelah IPC cancel benar-benar sukses.
    // Kalau invoke gagal / job tidak ditemukan, jangan biarkan UI mengaku
    // cancelled padahal proses FFmpeg masih berjalan.
    try {
      const cancelled = await ipcCancelExport(jobId);
      if (cancelled) {
        set({ status: 'cancelled' });
      } else {
        set({
          status: 'error',
          errorMessage:
            'Gagal membatalkan proses export — job tidak ditemukan atau sudah selesai.',
        });
      }
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      set({ status: 'error', errorMessage: `Gagal membatalkan proses export: ${detail}` });
    }
  },

  reset: () => set({ ...initialState }),
}));

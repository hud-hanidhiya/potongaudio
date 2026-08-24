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
    if (args.outputPath.toLowerCase() === args.params.sourceFilePath.toLowerCase()) {
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

    set({ status: 'cancelled' });
    await ipcCancelExport(jobId);
  },

  reset: () => set({ ...initialState }),
}));

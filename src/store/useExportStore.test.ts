import { describe, it, expect } from 'vitest';
import { useExportStore } from './useExportStore';
import type { EffectParams } from '../types/audio.types';

function makeParams(sourceFilePath: string): EffectParams {
  return {
    sourceFilePath,
    region: { startMs: 0, endMs: 1000 },
    gainDb: 0,
    fade: { inMs: 0, outMs: 0 },
    speed: { ratio: 1, preservePitch: true },
    outputFormat: 'mp3',
    outputBitrateKbps: 192,
  };
}

describe('useExportStore.startExport — AC-04', () => {
  it('menolak export kalau output path persis sama dengan source path', async () => {
    useExportStore.getState().reset();
    await useExportStore.getState().startExport({
      params: makeParams('/musik/lagu.mp3'),
      totalDurationMs: 1000,
      outputPath: '/musik/lagu.mp3',
    });
    const s = useExportStore.getState();
    expect(s.status).toBe('error');
    expect(s.errorMessage).toMatch(/tidak boleh sama/);
  });

  it('menolak juga kalau kapitalisasi berbeda (kasus Windows)', async () => {
    useExportStore.getState().reset();
    await useExportStore.getState().startExport({
      params: makeParams('C:\\musik\\lagu.mp3'),
      totalDurationMs: 1000,
      outputPath: 'c:\\musik\\lagu.mp3',
    });
    expect(useExportStore.getState().status).toBe('error');
  });

  it('tidak menolak kalau output path berbeda (guard AC-04 tidak fire)', async () => {
    useExportStore.getState().reset();
    // Path beda → guard lewat; di node, invoke Tauri gagal sehingga status jadi
    // 'error' dari IPC (bukan dari AC-04). Kita cukup pastikan pesan error BUKAN
    // pesan penolakan AC-04.
    await useExportStore.getState().startExport({
      params: makeParams('/musik/lagu.mp3'),
      totalDurationMs: 1000,
      outputPath: '/musik/lagu_edit.mp3',
    });
    expect(useExportStore.getState().errorMessage).not.toMatch(/tidak boleh sama/);
  });
});

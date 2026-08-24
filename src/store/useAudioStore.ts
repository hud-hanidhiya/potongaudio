/**
 * Store utama untuk state file audio yang sedang dikerjakan: file yang
 * dimuat, region terpilih, dan seluruh parameter efek. Ini adalah
 * implementasi konkret dari "Effect Parameter Contract" yang dibahas di
 * PLAN_AUDIO_CUTTER.md — SATU sumber kebenaran yang dipakai baik oleh
 * preview engine (Web Audio API) maupun oleh payload yang dikirim ke
 * Rust saat export.
 */

import { create } from 'zustand';
import type {
  EffectParams,
  Fade,
  LoadedAudioFile,
  OutputFormat,
  Region,
  Speed,
} from '../types/audio.types';
import { createDefaultEffectParams } from '../types/audio.types';

interface AudioStoreState {
  loadedFile: LoadedAudioFile | null;
  effectParams: EffectParams | null;

  // --- actions ---
  loadFile: (file: LoadedAudioFile) => void;
  clearFile: () => void;

  setRegion: (region: Region) => void;
  setGainDb: (gainDb: number) => void;
  setFade: (fade: Partial<Fade>) => void;
  setSpeed: (speed: Partial<Speed>) => void;
  setOutputFormat: (format: OutputFormat) => void;
  setOutputBitrateKbps: (kbps: number | undefined) => void;

  resetEffects: () => void;
}

export const useAudioStore = create<AudioStoreState>((set, get) => ({
  loadedFile: null,
  effectParams: null,

  loadFile: (file) => {
    set({
      loadedFile: file,
      effectParams: createDefaultEffectParams(file.path, file.probe.durationMs),
    });
  },

  clearFile: () => set({ loadedFile: null, effectParams: null }),

  setRegion: (region) => {
    const current = get().effectParams;
    const file = get().loadedFile;
    if (!current || !file) return;

    // H3: clamp region ke durasi file — jangan biarkan End melebihi durasi
    // (output FFmpeg akan berhenti di akhir file diam-diam, progress tak
    // pernah 100%). Tolak bila hasil clamp membuat region kosong.
    const startMs = Math.max(0, region.startMs);
    const endMs = Math.min(region.endMs, file.probe.durationMs);
    if (endMs <= startMs) return;

    set({ effectParams: { ...current, region: { startMs, endMs } } });
  },

  setGainDb: (gainDb) => {
    const current = get().effectParams;
    if (!current) return;
    set({ effectParams: { ...current, gainDb } });
  },

  setFade: (fade) => {
    const current = get().effectParams;
    if (!current) return;
    set({ effectParams: { ...current, fade: { ...current.fade, ...fade } } });
  },

  setSpeed: (speed) => {
    const current = get().effectParams;
    if (!current) return;
    set({ effectParams: { ...current, speed: { ...current.speed, ...speed } } });
  },

  setOutputFormat: (outputFormat) => {
    const current = get().effectParams;
    if (!current) return;
    set({ effectParams: { ...current, outputFormat } });
  },

  setOutputBitrateKbps: (outputBitrateKbps) => {
    const current = get().effectParams;
    if (!current) return;
    set({ effectParams: { ...current, outputBitrateKbps } });
  },

  resetEffects: () => {
    const file = get().loadedFile;
    if (!file) return;
    set({ effectParams: createDefaultEffectParams(file.path, file.probe.durationMs) });
  },
}));

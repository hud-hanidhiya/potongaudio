/**
 * Decode file audio untuk keperluan VISUALISASI WAVEFORM & PREVIEW saja.
 * TIDAK dipakai untuk proses export final — itu tugas Rust/FFmpeg native
 * (lihat PLAN_AUDIO_CUTTER.md Section 4, "file audio asli tidak perlu
 * di-load penuh ke JS heap untuk proses export").
 *
 * Karena ini cuma untuk preview, decode via Web Audio API sudah cukup;
 * tidak perlu strategi chunked-decode kompleks untuk v1 (lihat catatan
 * risiko "memory pressure" di PLAN_AUDIO_CUTTER.md § 6 — kalau nanti jadi
 * masalah nyata di file besar, di sinilah tempat menambahkan streaming
 * decode).
 */

let sharedAudioContext: AudioContext | null = null;

function getAudioContext(): AudioContext {
  if (!sharedAudioContext) {
    sharedAudioContext = new AudioContext();
  }
  return sharedAudioContext;
}

export interface DecodedAudio {
  audioBuffer: AudioBuffer;
  durationMs: number;
}

/**
 * Decode dari path file lokal (via Tauri, bukan File API browser biasa —
 * asumsinya file sudah ada di disk, path didapat dari native file picker
 * `pickOpenAudioFile()` di ipc.ts, bukan drag-drop File object langsung).
 *
 * Membaca bytes file via `@tauri-apps/plugin-fs` `readFile` (scope `**` di
 * `tauri.conf.json` + izin `fs:read-file` di capabilities), lalu decode
 * Web Audio API. Bytes yang sama dipakai ulang oleh WaveSurfer (blob URL)
 * supaya file tidak dibaca dua kali dari disk.
 */
export async function decodeAudioFromPath(filePath: string): Promise<DecodedAudio> {
  const { readFile } = await import('@tauri-apps/plugin-fs');
  const bytes = await readFile(filePath);
  return decodeAudioFromBytes(new Uint8Array(bytes).buffer);
}

export async function decodeAudioFromBytes(bytes: ArrayBuffer): Promise<DecodedAudio> {
  const ctx = getAudioContext();
  const audioBuffer = await ctx.decodeAudioData(bytes);
  return {
    audioBuffer,
    durationMs: Math.round(audioBuffer.duration * 1000),
  };
}

export function disposeSharedAudioContext(): void {
  if (sharedAudioContext) {
    void sharedAudioContext.close();
    sharedAudioContext = null;
  }
}

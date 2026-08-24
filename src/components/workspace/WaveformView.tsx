/**
 * Render waveform interaktif + region selection.
 *
 * TODO(Fase 2, T2.1-T2.3): integrasi WaveSurfer.js + Regions Plugin belum
 * dipasang di skeleton ini (dependency belum diinstal). Struktur di bawah
 * adalah KERANGKA yang sudah disambungkan ke useAudioStore dan
 * previewEngine.ts, supaya begitu WaveSurfer dipasang, tinggal isi
 * `initWaveSurfer()` tanpa perlu re-wiring state.
 */

import { useEffect, useRef, useState } from 'react';
import { useAudioStore } from '../../store/useAudioStore';
import { decodeAudioFromPath, type DecodedAudio } from '../../lib/audioDecode';
import { playPreview, type PreviewHandle } from '../../lib/previewEngine';
import { needsTimeStretch } from '../../lib/soundtouch';
import { TimeInput } from './TimeInput';

export function WaveformView() {
  const loadedFile = useAudioStore((s) => s.loadedFile);
  const effectParams = useAudioStore((s) => s.effectParams);
  const setRegion = useAudioStore((s) => s.setRegion);

  const containerRef = useRef<HTMLDivElement>(null);
  const [decoded, setDecoded] = useState<DecodedAudio | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const previewHandleRef = useRef<PreviewHandle | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);

  useEffect(() => {
    if (!loadedFile) return;

    let cancelled = false;
    decodeAudioFromPath(loadedFile.path)
      .then((result) => {
        if (!cancelled) setDecoded(result);
      })
      .catch(() => {
        // TODO(T2.1): tampilkan error state yang layak — untuk skeleton ini
        // dibiarkan silent karena decodeAudioFromPath memang belum
        // diimplementasi penuh (lihat TODO di audioDecode.ts).
      });

    return () => {
      cancelled = true;
    };
  }, [loadedFile]);

  useEffect(() => {
    // TODO(T2.1): panggil WaveSurfer.create({ container: containerRef.current, ... })
    // di sini begitu dependency terpasang, lalu daftarkan region-updated
    // listener yang memanggil setRegion(...) dari useAudioStore.
  }, [decoded]);

  const handlePlayToggle = () => {
    if (!decoded || !effectParams) return;

    if (isPlaying) {
      previewHandleRef.current?.stop();
      setIsPlaying(false);
      return;
    }

    if (needsTimeStretch(effectParams.speed.ratio)) {
      // TODO(T2.5): ganti ke playWithTimeStretch dari lib/soundtouch.ts
      // begitu library time-stretch pilihan sudah divalidasi di Fase 0.
      return;
    }

    if (!audioContextRef.current) {
      audioContextRef.current = new AudioContext();
    }

    previewHandleRef.current = playPreview({
      audioBuffer: decoded.audioBuffer,
      audioContext: audioContextRef.current,
      region: effectParams.region,
      gainDb: effectParams.gainDb,
      fade: effectParams.fade,
      onEnded: () => setIsPlaying(false),
    });
    setIsPlaying(true);
  };

  if (!loadedFile || !effectParams) return null;

  return (
    <div className="flex flex-col gap-3">
      <div
        ref={containerRef}
        className="h-32 rounded-lg bg-slate-900"
        aria-label="Audio waveform"
      >
        {!decoded && (
          <div className="flex h-full items-center justify-center text-xs text-slate-500">
            Memuat waveform...
          </div>
        )}
      </div>

      <div className="flex items-center gap-4">
        <button
          type="button"
          onClick={handlePlayToggle}
          disabled={!decoded}
          className="rounded-full bg-cyan px-4 py-2 text-sm font-medium text-slate-900 hover:bg-cyan/90 disabled:opacity-50"
        >
          {isPlaying ? 'Pause' : 'Play'}
        </button>

        <TimeInput
          label="Start"
          valueMs={effectParams.region.startMs}
          onChange={(ms) => setRegion({ ...effectParams.region, startMs: ms })}
        />
        <TimeInput
          label="End"
          valueMs={effectParams.region.endMs}
          onChange={(ms) => setRegion({ ...effectParams.region, endMs: ms })}
        />
      </div>
    </div>
  );
}

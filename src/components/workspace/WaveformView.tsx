/**
 * Render waveform interaktif + region selection via WaveSurfer.js.
 *
 * File audio dibaca SEKALI via `readAudioBytes` (plugin-fs): bytes dipakai
 * untuk (a) blob URL → WaveSurfer visualisasi, dan (b) decode Web Audio API
 * → preview engine. Region trim dua-arah tersinkron dengan `useAudioStore`:
 * drag handle di waveform mengubah `region`, dan edit TimeInput memperbarui
 * region di waveform.
 */

import { useEffect, useRef, useState } from 'react';
import WaveSurfer from 'wavesurfer.js';
import RegionsPlugin from 'wavesurfer.js/plugins/regions';
import { useAudioStore } from '../../store/useAudioStore';
import {
  decodeAudioFromBytes,
  readAudioBytes,
  type DecodedAudio,
} from '../../lib/audioDecode';
import { playPreview, type PreviewHandle } from '../../lib/previewEngine';
import { needsTimeStretch } from '../../lib/soundtouch';
import { TimeInput } from './TimeInput';

const REGION_ID = 'trim';
const EPS_MS = 5;

export function WaveformView() {
  const loadedFile = useAudioStore((s) => s.loadedFile);
  const effectParams = useAudioStore((s) => s.effectParams);
  const setRegion = useAudioStore((s) => s.setRegion);

  const containerRef = useRef<HTMLDivElement>(null);
  const wsRef = useRef<WaveSurfer | null>(null);
  const regionPluginRef = useRef<RegionsPlugin | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);

  const [decoded, setDecoded] = useState<DecodedAudio | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [loadError, setLoadError] = useState(false);
  const previewHandleRef = useRef<PreviewHandle | null>(null);

  // --- Load audio (sekali) + init WaveSurfer ---
  useEffect(() => {
    if (!loadedFile) return;

    // Reset state sebelumnya (H2): error file lama jangan menempel ke file
    // baru, buffer lama jangan bisa diputar saat decode baru berjalan, dan
    // label tombol kembali ke "Play".
    setLoadError(false);
    setDecoded(null);
    setIsPlaying(false);

    let cancelled = false;
    let objectUrl: string | null = null;
    let ws: WaveSurfer | null = null;

    (async () => {
      const bytes = await readAudioBytes(loadedFile.path);
      if (cancelled) return;

      const result = await decodeAudioFromBytes(bytes.buffer as ArrayBuffer);
      if (cancelled) return;
      setDecoded(result);

      if (!containerRef.current) return;

      objectUrl = URL.createObjectURL(new Blob([bytes.buffer as ArrayBuffer]));
      ws = WaveSurfer.create({
        container: containerRef.current,
        height: 96,
        waveColor: '#64748b',
        progressColor: '#06b6d4',
        cursorColor: '#10b981',
        interact: false,
        url: objectUrl,
      });
      wsRef.current = ws;

      const regions = ws.registerPlugin(RegionsPlugin.create());
      regionPluginRef.current = regions;

      regions.on('region-updated', (region) => {
        if (region.id !== REGION_ID) return;
        const startMs = Math.round(region.start * 1000);
        const endMs = Math.round(region.end * 1000);
        const current = useAudioStore.getState().effectParams?.region;
        if (current &&
            (Math.abs(startMs - current.startMs) > EPS_MS ||
             Math.abs(endMs - current.endMs) > EPS_MS)) {
          setRegion({ startMs, endMs });
        }
      });

      ws.on('ready', () => syncRegionToWaveSurfer(result.durationMs));
    })().catch(() => {
      if (!cancelled) setLoadError(true);
    });

    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
      ws?.destroy();
      wsRef.current = null;
      regionPluginRef.current = null;
      previewHandleRef.current?.stop();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loadedFile]);

  // --- Sync region dari store → WaveSurfer (edit TimeInput) ---
  useEffect(() => {
    if (!decoded) return;
    syncRegionToWaveSurfer(decoded.durationMs);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [effectParams?.region, decoded]);

  function syncRegionToWaveSurfer(durationMs: number) {
    const regions = regionPluginRef.current;
    // M1: baca region via getState(), bukan closure render — fungsi ini
    // dipanggil dari callback `ws.on('ready')` yang dibuat di render awal,
    // sehingga closure bisa stale jika user mengubah region sebelum ready.
    const region = useAudioStore.getState().effectParams?.region;
    if (!regions || !region) return;

    const start = Math.max(0, region.startMs) / 1000;
    const end = Math.min(durationMs, region.endMs) / 1000;
    if (end <= start) return;

    const existing = regions
      .getRegions()
      .find((r) => r.id === REGION_ID);

    if (existing) {
      const curStart = Math.round(existing.start * 1000);
      const curEnd = Math.round(existing.end * 1000);
      if (Math.abs(curStart - region.startMs) <= EPS_MS &&
          Math.abs(curEnd - region.endMs) <= EPS_MS) {
        return; // sudah sinkron, hindari loop
      }
      existing.setOptions({ start, end });
    } else {
      regions.addRegion({
        id: REGION_ID,
        start,
        end,
        content: 'Trim',
        color: 'rgba(16, 185, 129, 0.18)',
      });
    }
  }

  const handlePlayToggle = () => {
    if (!decoded || !effectParams) return;

    if (isPlaying) {
      previewHandleRef.current?.stop();
      setIsPlaying(false);
      return;
    }

    if (needsTimeStretch(effectParams.speed.ratio)) {
      // TODO(T2.5): ganti ke playWithTimeStretch dari lib/soundtouch.ts
      // begitu library time-stretch pilihan sudah divalidasi.
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
        className="h-32 rounded-lg border border-cyan/30 bg-slate-900"
        aria-label="Audio waveform"
      >
        {!decoded && !loadError && (
          <div className="flex h-full items-center justify-center text-xs text-slate-500">
            Memuat waveform...
          </div>
        )}
        {loadError && (
          <div className="flex h-full items-center justify-center text-xs text-red-400">
            Gagal memuat file audio. Pastikan format didukung.
          </div>
        )}
      </div>

      <div className="flex items-center gap-4">
        <button
          type="button"
          onClick={handlePlayToggle}
          disabled={!decoded || loadError}
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

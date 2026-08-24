/**
 * Top toolbar: Trim (implisit via region), Volume/Gain, Speed & Pitch,
 * Equalizer (ditunda ke v2), Reset & Close.
 *
 * Catatan: Equalizer ditunda ke v2 sesuai keputusan scope Fase 0 — tidak
 * masuk EffectParams (audio.types.ts) maupun filter_builder.rs Rust.
 * Tombolnya di-disable dulu sampai v2.
 */

import { useAudioStore } from '../../store/useAudioStore';

export function Toolbar() {
  const effectParams = useAudioStore((s) => s.effectParams);
  const setGainDb = useAudioStore((s) => s.setGainDb);
  const setSpeed = useAudioStore((s) => s.setSpeed);
  const resetEffects = useAudioStore((s) => s.resetEffects);
  const clearFile = useAudioStore((s) => s.clearFile);

  if (!effectParams) return null;

  return (
    <div className="flex items-center gap-6 rounded-lg bg-slate-800 px-4 py-2">
      <div className="flex flex-col gap-1">
        <span className="text-xs text-slate-400">Gain ({effectParams.gainDb} dB)</span>
        <input
          type="range"
          min={-20}
          max={20}
          step={0.5}
          value={effectParams.gainDb}
          onChange={(e) => setGainDb(Number(e.target.value))}
          className="w-32"
        />
      </div>

      <div className="flex flex-col gap-1">
        <span className="text-xs text-slate-400">
          Speed ({effectParams.speed.ratio.toFixed(2)}x)
        </span>
        <input
          type="range"
          min={0.25}
          max={4}
          step={0.05}
          value={effectParams.speed.ratio}
          onChange={(e) => setSpeed({ ratio: Number(e.target.value) })}
          className="w-32"
        />
      </div>

      <label className="flex items-center gap-2 text-xs text-slate-400">
        <input
          type="checkbox"
          disabled
          title="Belum aktif: butuh library time-stretch (belum dibundel di build LGPL)"
          checked={false}
          readOnly
          onChange={() => {}}
        />
        Preserve pitch
      </label>

      <button
        type="button"
        disabled
        title="Ditunda ke v2"
        className="rounded bg-slate-700 px-3 py-1 text-xs text-slate-500"
      >
        Equalizer
      </button>

      <div className="ml-auto flex gap-2">
        <button
          type="button"
          onClick={resetEffects}
          className="rounded bg-slate-700 px-3 py-1 text-xs text-slate-200 hover:bg-slate-600"
        >
          Reset
        </button>
        <button
          type="button"
          onClick={clearFile}
          className="rounded bg-slate-700 px-3 py-1 text-xs text-slate-200 hover:bg-slate-600"
        >
          Tutup
        </button>
      </div>
    </div>
  );
}

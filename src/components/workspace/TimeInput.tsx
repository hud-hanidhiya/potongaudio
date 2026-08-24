/**
 * Input angka presisi format MM:SS.ms — dua arah dengan region waveform
 * (drag handle mengubah nilai di sini, mengetik di sini mengubah region).
 */

import { useState } from 'react';

interface TimeInputProps {
  valueMs: number;
  onChange: (ms: number) => void;
  label: string;
  disabled?: boolean;
}

function msToDisplay(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  const millis = ms % 1000;
  return `${pad(minutes, 2)}:${pad(seconds, 2)}.${pad(millis, 3)}`;
}

function pad(n: number, width: number): string {
  return n.toString().padStart(width, '0');
}

/** Parsing "MM:SS.ms" -> total milidetik. Mengembalikan null kalau format tidak valid. */
function parseDisplay(input: string): number | null {
  const match = input.trim().match(/^(\d{1,3}):(\d{1,2})(?:\.(\d{1,3}))?$/);
  if (!match) return null;

  const [, minStr, secStr, msStr = '0'] = match;
  const minutes = Number(minStr);
  const seconds = Number(secStr);
  const millis = Number(msStr.padEnd(3, '0'));

  if (seconds >= 60) return null;

  return minutes * 60_000 + seconds * 1000 + millis;
}

export function TimeInput({ valueMs, onChange, label, disabled }: TimeInputProps) {
  const [text, setText] = useState(() => msToDisplay(valueMs));
  const [isInvalid, setIsInvalid] = useState(false);
  // Sinkron ulang tampilan kalau value berubah dari luar (mis. drag handle
  // waveform), TAPI jangan timpa apa yang sedang diketik user.
  // Pola "adjust state during render" (resmi React) — bukan setState di effect.
  const [prevValueMs, setPrevValueMs] = useState(valueMs);
  if (valueMs !== prevValueMs) {
    setPrevValueMs(valueMs);
    setText(msToDisplay(valueMs));
    setIsInvalid(false);
  }

  const handleBlur = () => {
    const parsed = parseDisplay(text);
    if (parsed === null) {
      setIsInvalid(true);
      return;
    }
    setIsInvalid(false);
    onChange(parsed);
  };

  return (
    <label className="flex flex-col gap-1 text-xs text-slate-400">
      {label}
      <input
        type="text"
        value={text}
        disabled={disabled}
        onChange={(e) => setText(e.target.value)}
        onBlur={handleBlur}
        placeholder="00:00.000"
        className={[
          'w-28 rounded border bg-slate-900 px-2 py-1 font-mono text-sm text-slate-100',
          isInvalid ? 'border-red-500' : 'border-slate-700',
        ].join(' ')}
      />
    </label>
  );
}

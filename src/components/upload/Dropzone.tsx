/**
 * Halaman upload/landing — drag-drop area + tombol file picker native.
 * File picker native pakai Tauri dialog plugin (lihat pickOpenAudioFile
 * di lib/ipc.ts), BUKAN <input type="file"> HTML biasa, supaya user dapat
 * path file asli di disk (dibutuhkan Rust command, bukan Blob/File object
 * browser).
 *
 * Drag-drop native via Tauri webview event (tauri://file-drop) — memberi
 * path file asli seperti halnya dialog picker, bukan Blob object.
 * Event tauri://file-drop-hover mengatur state isDragActive untuk visual
 * feedback; tauri://file-drop-cancelled mereset state bila drag dibatalkan.
 */

import { useCallback, useEffect, useState } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { pickOpenAudioFile, probeAudioFile } from '../../lib/ipc';
import { SUPPORTED_EXTENSIONS } from '../../types/audio.types';
import { useAudioStore } from '../../store/useAudioStore';

export function Dropzone() {
  const loadFile = useAudioStore((s) => s.loadFile);
  const [isDragActive, setIsDragActive] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openAndLoad = useCallback(
    async (filePath: string) => {
      setError(null);
      setIsLoading(true);
      try {
        const probe = await probeAudioFile(filePath);
        const fileName = filePath.split(/[/\\]/).pop() ?? filePath;
        loadFile({ path: filePath, fileName, probe });
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setIsLoading(false);
      }
    },
    [loadFile]
  );

  const handleBrowseClick = useCallback(async () => {
    const filePath = await pickOpenAudioFile();
    if (filePath) await openAndLoad(filePath);
  }, [openAndLoad]);

  useEffect(() => {
    const unsubs: UnlistenFn[] = [];

    const setupListeners = async () => {
      unsubs.push(
        await listen<string[]>('tauri://file-drop', (event) => {
          const paths = event.payload;
          if (paths.length > 0) {
            setIsDragActive(false);
            void openAndLoad(paths[0]);
          }
        })
      );

      unsubs.push(
        await listen<string[]>('tauri://file-drop-hover', () => {
          setIsDragActive(true);
        })
      );

      unsubs.push(
        await listen('tauri://file-drop-cancelled', () => {
          setIsDragActive(false);
        })
      );
    };

    void setupListeners();

    return () => {
      unsubs.forEach((fn) => fn());
    };
  }, [openAndLoad]);

  return (
    <div
      className={[
        'flex flex-col items-center justify-center gap-4 rounded-xl border-2 border-dashed p-16 text-center transition-colors',
        isDragActive ? 'border-cyan bg-cyan/20' : 'border-slate-700',
      ].join(' ')}
    >
      <p className="text-slate-300">
        Seret file audio ke sini, atau
      </p>
      <button
        type="button"
        onClick={handleBrowseClick}
        disabled={isLoading}
        className="rounded-lg bg-cyan px-4 py-2 font-medium text-slate-900 hover:bg-cyan/90 disabled:opacity-50"
      >
        {isLoading ? 'Memuatan...' : 'Pilih File'}
      </button>
      <p className="text-xs text-slate-500">
        Format didukung: {SUPPORTED_EXTENSIONS.join(', ')}
      </p>
      {error && <p className="text-sm text-red-400">{error}</p>}
    </div>
  );
}

import { useEffect, useState } from 'react';
import { Dropzone } from './components/upload/Dropzone';
import { ExportDock } from './components/workspace/ExportDock';
import { Toolbar } from './components/workspace/Toolbar';
import { WaveformView } from './components/workspace/WaveformView';
import { useAudioStore } from './store/useAudioStore';
import { getFfmpegVersion } from './lib/ipc';


export default function App() {
  const loadedFile = useAudioStore((s) => s.loadedFile);
  const [ffmpegVersion, setFfmpegVersion] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getFfmpegVersion()
      .then((v) => {
        if (!cancelled) setFfmpegVersion(v.trim().split('\n')[0] ?? v);
      })
      .catch(() => {
        if (!cancelled) setFfmpegVersion('FFmpeg sidecar tidak tersedia');
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="min-h-screen bg-navy p-6 text-slate-100">
      <header className="mb-6 flex items-baseline justify-between">
        <h1 className="text-lg font-semibold text-cyan">PotongAudio</h1>
        {ffmpegVersion && (
          <span className="text-xs text-slate-500">{ffmpegVersion}</span>
        )}
      </header>

      <main className="mx-auto flex max-w-3xl flex-col gap-4">
        {!loadedFile ? (
          <Dropzone />
        ) : (
          <>
            <Toolbar />
            <WaveformView />
            <ExportDock />
          </>
        )}
      </main>
    </div>
  );
}

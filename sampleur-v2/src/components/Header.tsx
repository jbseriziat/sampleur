import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect, useRef } from 'react';
import { usePadStore } from '../store/usePadStore';
import { useFxStore } from '../store/useFxStore';

/** Format seconds into MM:SS */
function fmtTime(secs: number): string {
  const m = Math.floor(secs / 60).toString().padStart(2, '0');
  const s = (secs % 60).toString().padStart(2, '0');
  return `${m}:${s}`;
}

export function Header() {
  const { kitName, setKitName, gridSize, setGridSize, editMode, setEditMode, resetAllPads } = usePadStore();
  const { bpm, setBpm, quantize, setQuantize } = useFxStore();

  // ── Recording state ─────────────────────────────────────────────────────────
  const [isRecording, setIsRecording] = useState(false);
  const [recSeconds, setRecSeconds] = useState(0);
  const recTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (isRecording) {
      setRecSeconds(0);
      recTimerRef.current = setInterval(() => setRecSeconds((s) => s + 1), 1000);
    } else {
      if (recTimerRef.current) clearInterval(recTimerRef.current);
    }
    return () => { if (recTimerRef.current) clearInterval(recTimerRef.current); };
  }, [isRecording]);

  const handleStopAll = async () => {
    await invoke('stop_all');
  };

  const handleRecord = async () => {
    if (!isRecording) {
      try {
        const filePath = await invoke<string>('start_recording');
        setIsRecording(true);
        console.info('Recording started:', filePath);
      } catch (err) {
        console.error('Failed to start recording:', err);
      }
    } else {
      try {
        const filePath = await invoke<string>('stop_recording');
        setIsRecording(false);
        alert(`Enregistrement sauvegard\u00e9 :\n${filePath}`);
      } catch (err) {
        setIsRecording(false);
        console.error('Failed to stop recording:', err);
      }
    }
  };

  const handleNewKit = async () => {
    if (!window.confirm('Créer un nouveau kit ? Les pads non sauvegardés seront perdus.')) return;
    await invoke('reset_kit');
    resetAllPads();
    setKitName('Nouveau Kit');
  };

  const handleBpmChange = async (v: number) => {
    setBpm(v);
    await invoke('set_bpm', { bpm: v });
  };

  const handleQuantizeToggle = async () => {
    const next = !quantize;
    setQuantize(next);
    await invoke('set_quantize', { enabled: next });
  };

  return (
    <div className="flex items-center gap-3 px-4 py-2 bg-slate-800 border-b border-slate-700 flex-wrap">
      {/* Kit name */}
      <input
        type="text"
        value={kitName}
        onChange={(e) => setKitName(e.target.value)}
        className="bg-slate-700 text-white text-sm rounded px-2 py-1 w-40 border border-slate-600 focus:outline-none focus:border-pink-500"
        placeholder="Nom du kit"
      />

      {/* Grid size toggle */}
      <div className="flex rounded overflow-hidden border border-slate-600">
        <button
          onClick={() => setGridSize(16)}
          className={`px-3 py-1 text-xs font-bold ${gridSize === 16 ? 'bg-pink-600 text-white' : 'bg-slate-700 text-gray-400'}`}
        >
          PADAWAN (16)
        </button>
        <button
          onClick={() => setGridSize(64)}
          className={`px-3 py-1 text-xs font-bold ${gridSize === 64 ? 'bg-pink-600 text-white' : 'bg-slate-700 text-gray-400'}`}
        >
          JEDI (64)
        </button>
      </div>

      {/* Edit / Play mode */}
      <div className="flex rounded overflow-hidden border border-slate-600">
        <button
          onClick={() => setEditMode(true)}
          className={`px-3 py-1 text-xs font-bold ${editMode ? 'bg-blue-600 text-white' : 'bg-slate-700 text-gray-400'}`}
        >
          CONFIG
        </button>
        <button
          onClick={() => setEditMode(false)}
          className={`px-3 py-1 text-xs font-bold ${!editMode ? 'bg-green-600 text-white' : 'bg-slate-700 text-gray-400'}`}
        >
          JOUER
        </button>
      </div>

      {/* BPM control */}
      <div className="flex items-center gap-2">
        <span className="text-gray-400 text-xs">BPM</span>
        <input
          type="number"
          value={bpm}
          min={40}
          max={300}
          onChange={(e) => handleBpmChange(Number(e.target.value))}
          className="w-16 bg-slate-700 text-white text-sm text-center rounded px-1 py-1 border border-slate-600 focus:outline-none focus:border-pink-500"
        />
        <input
          type="range"
          min={40}
          max={240}
          value={bpm}
          onChange={(e) => handleBpmChange(Number(e.target.value))}
          className="w-24"
        />
      </div>

      {/* Quantize toggle */}
      <button
        onClick={handleQuantizeToggle}
        className={`px-3 py-1 text-xs font-bold rounded border ${quantize ? 'bg-yellow-600 text-white border-yellow-500' : 'bg-slate-700 text-gray-400 border-slate-600'}`}
      >
        AIMANT
      </button>

      {/* New kit */}
      <button
        onClick={handleNewKit}
        title="Réinitialiser tous les pads et créer un nouveau kit"
        className="px-3 py-1 text-xs font-bold bg-slate-600 hover:bg-slate-500 text-white rounded border border-slate-500"
      >
        + NOUVEAU KIT
      </button>

      {/* Stop all */}
      <button
        onClick={handleStopAll}
        className="px-3 py-1 text-xs font-bold bg-red-700 hover:bg-red-600 text-white rounded"
      >
        STOP
      </button>

      {/* Recording */}
      <div className="flex items-center gap-1">
        <button
          onClick={handleRecord}
          title={isRecording ? 'Arr\u00eater l\'enregistrement' : 'D\u00e9marrer l\'enregistrement (WAV 32-bit float)'}
          className={[
            'px-3 py-1 text-xs font-bold rounded border',
            isRecording
              ? 'bg-red-600 text-white border-red-500 animate-pulse'
              : 'bg-slate-700 text-red-400 border-red-700 hover:bg-red-900 hover:text-red-300',
          ].join(' ')}
        >
          {isRecording ? '\u25a0 STOP REC' : '\u25cf REC'}
        </button>
        {isRecording && (
          <span className="text-red-400 text-xs font-mono tabular-nums">
            {fmtTime(recSeconds)}
          </span>
        )}
      </div>

      <div className="ml-auto flex items-center gap-2">
        <span className="text-slate-400 text-xs font-mono">Sampleur V2</span>
      </div>
    </div>
  );
}

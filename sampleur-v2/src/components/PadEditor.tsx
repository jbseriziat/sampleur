import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { usePadStore } from '../store/usePadStore';
import { useMidiStore } from '../store/useMidiStore';
import { COLORS, ColorDef } from '../types';

export function PadEditor() {
  const { pads, selectedPadId, updatePad, selectPad } = usePadStore();
  const { learnMode, learnPadId, startLearn, stopLearn, lastNote } = useMidiStore();

  const pad = selectedPadId !== null ? pads[selectedPadId] : null;

  if (!pad) {
    return (
      <div className="w-64 bg-slate-800 border-l border-slate-700 p-4 flex items-center justify-center">
        <p className="text-slate-500 text-sm text-center">
          S\u00e9lectionne un pad pour l'\u00e9diter
        </p>
      </div>
    );
  }

  const handleLoadSample = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Audio', extensions: ['wav', 'mp3', 'flac', 'ogg', 'aac'] }],
      });
      if (!selected) return;
      const filePath = typeof selected === 'string'
        ? selected
        : (selected as { path?: string }).path ?? String(selected);
      const result = await invoke<{ pad_id: number; file_name: string; duration_secs: number }>(
        'load_sample', { padId: pad.id, filePath },
      );
      updatePad(pad.id, {
        hasSample: true,
        fileName: result.file_name,
        durationSecs: result.duration_secs,
        filePath,
      });
    } catch (err) {
      console.error('Failed to load sample:', err);
    }
  };

  const handleRemoveSample = async () => {
    await invoke('remove_sample', { padId: pad.id });
    updatePad(pad.id, {
      hasSample: false,
      fileName: undefined,
      durationSecs: undefined,
      filePath: undefined,
    });
  };

  const handleModeChange = async (mode: string) => {
    updatePad(pad.id, { mode: mode as 'oneshot' | 'loop' | 'hold' });
    await invoke('set_pad_config', { padId: pad.id, mode });
  };

  const handleVolumeChange = async (volume: number) => {
    updatePad(pad.id, { volume });
    await invoke('set_pad_config', { padId: pad.id, volume });
  };

  const handleDetuneChange = async (detuneCents: number) => {
    updatePad(pad.id, { detuneCents });
    await invoke('set_pad_config', { padId: pad.id, detuneCents });
  };

  const handleOriginalBpmChange = async (originalBpm: number) => {
    updatePad(pad.id, { originalBpm });
    await invoke('set_pad_config', { padId: pad.id, originalBpm });
  };

  const handleColorChange = (color: ColorDef) => {
    updatePad(pad.id, { color });
    invoke('set_pad_config', { padId: pad.id, colorMidi: color.midiVelocity });
  };

  const handleStartLearn = async () => {
    if (learnMode && learnPadId === pad.id) {
      stopLearn();
      await invoke('cancel_midi_learn');
    } else {
      startLearn(pad.id);
      await invoke('start_midi_learn', { padId: pad.id });
    }
  };

  const isLearning = learnMode && learnPadId === pad.id;

  return (
    <div className="w-64 bg-slate-800 border-l border-slate-700 p-3 overflow-y-auto flex flex-col gap-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-white font-bold text-sm">Pad {pad.label}</h3>
        <button
          onClick={() => selectPad(null)}
          className="text-slate-400 hover:text-white text-xs"
        >
          \u2715
        </button>
      </div>

      {/* Sample */}
      <div className="bg-slate-700 rounded p-2">
        <p className="text-gray-400 text-xs mb-1 uppercase tracking-wide">Sample</p>
        {pad.hasSample ? (
          <div>
            <p className="text-white text-xs truncate">{pad.fileName}</p>
            {pad.durationSecs !== undefined && (
              <p className="text-gray-400 text-xs">{pad.durationSecs.toFixed(2)}s</p>
            )}
            <div className="flex gap-1 mt-1">
              <button
                onClick={handleLoadSample}
                className="flex-1 py-1 text-xs bg-slate-600 hover:bg-slate-500 text-white rounded"
              >
                Changer
              </button>
              <button
                onClick={handleRemoveSample}
                className="px-2 py-1 text-xs bg-red-800 hover:bg-red-700 text-white rounded"
              >
                \u2715
              </button>
            </div>
          </div>
        ) : (
          <button
            onClick={handleLoadSample}
            className="w-full py-2 text-xs bg-pink-700 hover:bg-pink-600 text-white rounded font-bold"
          >
            + Charger un sample
          </button>
        )}
      </div>

      {/* Mode */}
      <div>
        <p className="text-gray-400 text-xs mb-1 uppercase tracking-wide">Mode</p>
        <div className="flex gap-1">
          {(['oneshot', 'loop', 'hold'] as const).map((m) => (
            <button
              key={m}
              onClick={() => handleModeChange(m)}
              className={`flex-1 py-1 text-xs rounded font-bold ${pad.mode === m ? 'bg-pink-600 text-white' : 'bg-slate-700 text-gray-400 hover:bg-slate-600'}`}
            >
              {m === 'oneshot' ? '\u25b7' : m === 'loop' ? '\u221e' : '\u2299'}
            </button>
          ))}
        </div>
      </div>

      {/* Volume */}
      <div>
        <div className="flex justify-between text-xs mb-1">
          <span className="text-gray-400 uppercase tracking-wide">Volume</span>
          <span className="text-white font-mono">{pad.volume.toFixed(2)}</span>
        </div>
        <input
          type="range"
          min={0}
          max={2}
          step={0.01}
          value={pad.volume}
          onChange={(e) => handleVolumeChange(Number(e.target.value))}
          className="w-full"
        />
      </div>

      {/* Pitch */}
      <div>
        <div className="flex justify-between text-xs mb-1">
          <span className="text-gray-400 uppercase tracking-wide">Pitch</span>
          <span className="text-white font-mono">{(pad.detuneCents / 100).toFixed(1)} st</span>
        </div>
        <input
          type="range"
          min={-1200}
          max={1200}
          step={10}
          value={pad.detuneCents}
          onChange={(e) => handleDetuneChange(Number(e.target.value))}
          className="w-full"
        />
      </div>

      {/* Original BPM */}
      <div>
        <div className="flex justify-between text-xs mb-1">
          <span className="text-gray-400 uppercase tracking-wide">BPM Original</span>
          <span className="text-white font-mono">{pad.originalBpm}</span>
        </div>
        <input
          type="range"
          min={60}
          max={200}
          step={1}
          value={pad.originalBpm}
          onChange={(e) => handleOriginalBpmChange(Number(e.target.value))}
          className="w-full"
        />
      </div>

      {/* Color */}
      <div>
        <p className="text-gray-400 text-xs mb-1 uppercase tracking-wide">Couleur</p>
        <div className="flex gap-1 flex-wrap">
          {COLORS.map((c) => (
            <button
              key={c.name}
              onClick={() => handleColorChange(c)}
              title={c.name}
              className={`w-6 h-6 rounded ${c.tw} ${pad.color.name === c.name ? 'ring-2 ring-white scale-110' : ''}`}
            />
          ))}
        </div>
      </div>

      {/* MIDI Learn */}
      <div className="bg-slate-700 rounded p-2">
        <p className="text-gray-400 text-xs mb-1 uppercase tracking-wide">MIDI</p>
        <div className="flex items-center justify-between mb-1">
          <span className="text-white text-xs">Note: {pad.midiNote ?? '\u2014'}</span>
          <button
            onClick={handleStartLearn}
            className={`px-2 py-1 text-xs rounded font-bold ${isLearning ? 'bg-yellow-500 text-black animate-pulse' : 'bg-slate-600 text-white hover:bg-slate-500'}`}
          >
            {isLearning ? 'En attente...' : 'LEARN'}
          </button>
        </div>
        {isLearning && lastNote !== null && (
          <p className="text-yellow-400 text-xs">Derni\u00e8re note: {lastNote}</p>
        )}
      </div>
    </div>
  );
}

import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { usePadStore } from '../store/usePadStore';
import { useFxStore } from '../store/useFxStore';
import { useMidiStore } from '../store/useMidiStore';

export function PresetPanel() {
  const { pads, kitName, gridSize, loadFromPreset } = usePadStore();
  const { fx, bpm, quantize, loadFx } = useFxStore();
  const {
    midiInputs, midiOutputs,
    selectedInput, selectedOutput,
    setSelectedInput, setSelectedOutput,
    setInputs, setOutputs,
  } = useMidiStore();
  const [saveMode, setSaveMode] = useState<'lightweight' | 'portable'>('lightweight');

  const handleSave = async () => {
    try {
      const presetData = {
        version: 2,
        name: kitName,
        createdAt: new Date().toISOString(),
        kitMode: saveMode,
        bpm,
        quantize,
        gridSize,
        fx,
        pads: pads.map((p) => {
          if (!p.hasSample) return null;
          return {
            id: p.id,
            label: p.label,
            color: p.color,
            mode: p.mode,
            midiNote: p.midiNote,
            volume: p.volume,
            detuneCents: p.detuneCents,
            originalBpm: p.originalBpm,
            sample: p.filePath
              ? {
                  fileName: p.fileName ?? '',
                  relativePath: p.filePath,
                  absolutePathHint: p.filePath,
                  durationSecs: p.durationSecs ?? 0,
                  channels: 2,
                  sampleRate: 44100,
                }
              : undefined,
          };
        }),
      };

      await invoke('save_preset', {
        presetJson: JSON.stringify(presetData),
        kitMode: saveMode,
      });
    } catch (err) {
      if (String(err) !== 'Save cancelled') {
        console.error('Save error:', err);
      }
    }
  };

  const handleLoad = async () => {
    try {
      const json = await invoke<string>('load_preset');
      const preset = JSON.parse(json) as {
        pads: any[];
        name?: string;
        gridSize?: 16 | 64;
        fx?: any;
        bpm?: number;
        quantize?: boolean;
      };

      // 1. Update frontend store
      loadFromPreset(preset.pads, preset.name ?? 'Kit Charg\u00e9', preset.gridSize ?? 16);
      if (preset.fx) loadFx(preset.fx, preset.bpm ?? 120, preset.quantize ?? false);

      // 2. Sync FX params to Rust engine
      if (preset.fx) {
        for (const [key, value] of Object.entries(preset.fx)) {
          await invoke('set_fx_param', { param: key, value: value as number }).catch(() => {});
        }
      }
      if (preset.bpm !== undefined) {
        await invoke('set_bpm', { bpm: preset.bpm }).catch(console.warn);
      }
      if (preset.quantize !== undefined) {
        await invoke('set_quantize', { enabled: preset.quantize }).catch(console.warn);
      }

      // 3. Auto-load samples and sync pad config to Rust
      for (const padData of preset.pads) {
        if (!padData?.sample?.absolutePathHint) continue;
        try {
          const result = await invoke<{ pad_id: number; file_name: string; duration_secs: number }>(
            'load_sample', { padId: padData.id, filePath: padData.sample.absolutePathHint },
          );
          usePadStore.getState().updatePad(padData.id, {
            hasSample: true,
            fileName: result.file_name,
            durationSecs: result.duration_secs,
            filePath: padData.sample.absolutePathHint,
          });

          // Sync mode, volume, detune, originalBpm and Launchpad color to Rust backend
          const configArgs: Record<string, unknown> = {
            padId:       padData.id,
            volume:      padData.volume      ?? 1.0,
            detuneCents: padData.detuneCents ?? 0,
            mode:        padData.mode        ?? 'oneshot',
            originalBpm: padData.originalBpm ?? 120,
            colorMidi:   padData.color?.midiVelocity ?? 5,
          };
          // Restore explicit MIDI note if set (avoids wiping the Launchpad default mapping)
          if (typeof padData.midiNote === 'number') {
            configArgs.midiNote = padData.midiNote;
          }
          await invoke('set_pad_config', configArgs).catch(console.warn);
        } catch (err) {
          console.warn(`Cannot auto-load sample for pad ${padData.id}:`, err);
        }
      }

      // 4. Refresh Launchpad LEDs with the restored colors
      await invoke('refresh_leds').catch(console.warn);
    } catch (err) {
      if (String(err) !== 'Load cancelled') {
        console.error('Load error:', err);
      }
    }
  };

  const handleRefreshMidi = async () => {
    try {
      const [inputs, outputs] = await Promise.all([
        invoke<string[]>('get_midi_inputs'),
        invoke<string[]>('get_midi_outputs'),
      ]);
      setInputs(inputs);
      setOutputs(outputs);
    } catch (err) {
      console.error('MIDI refresh error:', err);
    }
  };

  const handleMidiInput = async (portName: string) => {
    setSelectedInput(portName);
    if (!portName) return;
    try {
      await invoke('set_midi_input', { portName });
    } catch (err) {
      console.error('MIDI input error:', err);
    }
  };

  const handleMidiOutput = async (portName: string) => {
    setSelectedOutput(portName);
    if (!portName) return;
    try {
      await invoke('set_midi_output', { portName });
      await invoke('init_launchpad');
    } catch (err) {
      console.error('MIDI output error:', err);
    }
  };

  const handleResetLeds = async () => {
    await invoke('reset_leds');
  };

  return (
    <div className="flex items-center gap-3 px-4 py-2 bg-slate-900 border-t border-slate-700 flex-wrap">
      {/* Preset controls */}
      <div className="flex items-center gap-1">
        <select
          value={saveMode}
          onChange={(e) => setSaveMode(e.target.value as 'lightweight' | 'portable')}
          className="bg-slate-700 text-gray-300 text-xs rounded px-1 py-1 border border-slate-600"
        >
          <option value="lightweight">L\u00e9ger (chemins)</option>
          <option value="portable">Portable (copies)</option>
        </select>
        <button
          onClick={handleSave}
          className="px-3 py-1 text-xs font-bold bg-blue-700 hover:bg-blue-600 text-white rounded"
        >
          Sauvegarder
        </button>
        <button
          onClick={handleLoad}
          className="px-3 py-1 text-xs font-bold bg-green-700 hover:bg-green-600 text-white rounded"
        >
          Charger
        </button>
      </div>

      {/* MIDI section */}
      <div className="flex items-center gap-1 ml-4">
        <span className="text-gray-400 text-xs">MIDI:</span>
        <button
          onClick={handleRefreshMidi}
          title="Rafra\u00eechir la liste des p\u00e9riph\u00e9riques MIDI"
          className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 text-gray-300 rounded"
        >
          ↺
        </button>

        <span className="text-gray-400 text-xs ml-1">Entr\u00e9e:</span>
        <select
          value={selectedInput ?? ''}
          onChange={(e) => handleMidiInput(e.target.value)}
          className="bg-slate-700 text-gray-300 text-xs rounded px-1 py-1 border border-slate-600 max-w-40"
        >
          <option value="">— Aucune —</option>
          {midiInputs.map((i) => (
            <option key={i} value={i}>{i.slice(0, 25)}</option>
          ))}
        </select>

        <span className="text-gray-400 text-xs ml-1">Sortie:</span>
        <select
          value={selectedOutput ?? ''}
          onChange={(e) => handleMidiOutput(e.target.value)}
          className="bg-slate-700 text-gray-300 text-xs rounded px-1 py-1 border border-slate-600 max-w-40"
        >
          <option value="">— Aucune —</option>
          {midiOutputs.map((o) => (
            <option key={o} value={o}>{o.slice(0, 25)}</option>
          ))}
        </select>
        <button
          onClick={handleResetLeds}
          title="R\u00e9initialiser les LEDs"
          className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 text-gray-300 rounded"
        >
          LEDs off
        </button>
      </div>
    </div>
  );
}

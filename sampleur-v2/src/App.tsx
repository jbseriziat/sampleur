import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Header } from './components/Header';
import { PadGrid } from './components/PadGrid';
import { PadEditor } from './components/PadEditor';
import { FxPanel } from './components/FxPanel';
import { PresetPanel } from './components/PresetPanel';
import { usePadStore } from './store/usePadStore';
import { useMidiStore } from './store/useMidiStore';
import { useTauriEvents } from './hooks/useTauriEvents';

export default function App() {
  useTauriEvents();

  const { gridSize, editMode } = usePadStore();
  const { setInputs, setOutputs } = useMidiStore();

  // Initialize MIDI device list on mount
  useEffect(() => {
    invoke<string[]>('get_midi_inputs').then(setInputs).catch(console.error);
    invoke<string[]>('get_midi_outputs').then(setOutputs).catch(console.error);
  }, []);

  const showFx = gridSize === 64 && editMode;
  const showEditor = editMode;

  return (
    <div className="flex flex-col h-screen bg-slate-900 text-white overflow-hidden">
      {/* Top bar */}
      <Header />

      {/* Main content area */}
      <div className="flex flex-1 overflow-hidden">
        {/* FX panel (left, only in Jedi mode) */}
        {showFx && <FxPanel />}

        {/* Pad grid (center) */}
        <PadGrid />

        {/* Pad editor (right, only in edit mode) */}
        {showEditor && <PadEditor />}
      </div>

      {/* Bottom bar */}
      <PresetPanel />
    </div>
  );
}

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { usePadStore } from '../store/usePadStore';
import { Pad } from './Pad';

export function PadGrid() {
  const { pads, gridSize, swapPads } = usePadStore();

  // ── Drag & drop state ───────────────────────────────────────────────────────
  const [dragPadId, setDragPadId] = useState<number | null>(null);
  const [overPadId, setOverPadId] = useState<number | null>(null);

  const handleSwap = async (dragId: number, dropId: number) => {
    if (dragId === dropId) return;

    // 1. Optimistic update in the store (instant visual feedback)
    swapPads(dragId, dropId);

    // 2. Sync both pads to the Rust backend
    //    Read the store AFTER the swap so we get the new content for each position.
    const syncPad = async (padId: number) => {
      const pad = usePadStore.getState().pads.find((p) => p.id === padId);
      if (!pad) return;
      if (pad.hasSample && pad.filePath) {
        await invoke('load_sample', { padId, filePath: pad.filePath }).catch(console.warn);
        await invoke('set_pad_config', {
          padId,
          volume:      pad.volume,
          detuneCents: pad.detuneCents,
          mode:        pad.mode,
          originalBpm: pad.originalBpm,
          colorMidi:   pad.color.midiVelocity,
        }).catch(console.warn);
      } else {
        await invoke('remove_sample', { padId }).catch(console.warn);
      }
    };

    await syncPad(dragId);
    await syncPad(dropId);

    // 3. Refresh Launchpad LEDs to reflect new colours
    await invoke('refresh_leds').catch(console.warn);
  };

  // Padawan mode: 4x4 using pads 0-3, 8-11, 16-19, 24-27 (first 4 of each row of 8)
  const visiblePads = gridSize === 64
    ? pads
    : pads.filter((_, i) => {
        const row = Math.floor(i / 8);
        const col = i % 8;
        return row < 4 && col < 4;
      });

  const cols = gridSize === 64 ? 8 : 4;

  return (
    <div
      className="p-3 flex-1"
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${cols}, 1fr)`,
        gap: '6px',
      }}
    >
      {visiblePads.map((pad) => (
        <Pad
          key={pad.id}
          pad={pad}
          size={gridSize === 64 ? 'small' : 'large'}
          isDragSource={dragPadId === pad.id}
          isDragTarget={overPadId === pad.id && dragPadId !== null && dragPadId !== pad.id}
          onDragStart={() => setDragPadId(pad.id)}
          onDragEnd={() => { setDragPadId(null); setOverPadId(null); }}
          onDragOver={() => setOverPadId(pad.id)}
          onDragLeave={() => setOverPadId((prev) => (prev === pad.id ? null : prev))}
          onDrop={() => {
            if (dragPadId !== null) handleSwap(dragPadId, pad.id);
            setDragPadId(null);
            setOverPadId(null);
          }}
        />
      ))}
    </div>
  );
}

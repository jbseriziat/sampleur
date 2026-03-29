import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { usePadStore } from '../store/usePadStore';
import { Pad } from './Pad';

export function PadGrid() {
  const { pads, gridSize, editMode } = usePadStore();

  // ── Mouse-event drag & drop ─────────────────────────────────────────────────
  // We use refs for the "truth" (no stale-closure risk in global mouseup handler)
  // and separate state only for re-rendering the visual feedback.
  const dragFromRef = useRef<number | null>(null);
  const overPadRef  = useRef<number | null>(null);
  const [visualDragFrom, setVisualDragFrom] = useState<number | null>(null);
  const [visualOverPad,  setVisualOverPad]  = useState<number | null>(null);

  const clearDragState = () => {
    dragFromRef.current = null;
    overPadRef.current  = null;
    setVisualDragFrom(null);
    setVisualOverPad(null);
  };

  /** Swap two pads and sync the Rust backend. Uses getState() so it stays stable. */
  const executeSwap = async (fromId: number, toId: number) => {
    if (fromId === toId) return;

    // Optimistic store update
    usePadStore.getState().swapPads(fromId, toId);

    // Sync each pad (post-swap content) to Rust
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

    await syncPad(fromId);
    await syncPad(toId);
    await invoke('refresh_leds').catch(console.warn);
  };

  // Global mouseup — finalise or cancel the drag.
  // Using refs guarantees we always read the latest values without stale closures.
  useEffect(() => {
    const onMouseUp = () => {
      const from = dragFromRef.current;
      const to   = overPadRef.current;
      clearDragState();
      if (from !== null && to !== null && from !== to) {
        void executeSwap(from, to);
      }
    };
    window.addEventListener('mouseup', onMouseUp);
    return () => window.removeEventListener('mouseup', onMouseUp);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // stable: refs + getState() don't need to be in deps

  // ── Drag callbacks passed to each Pad ──────────────────────────────────────

  const handlePadMouseDown = (padId: number) => {
    if (!editMode) return;
    dragFromRef.current = padId;
    setVisualDragFrom(padId);
  };

  const handlePadMouseEnter = (padId: number) => {
    if (dragFromRef.current === null) return;
    overPadRef.current = padId;
    setVisualOverPad(padId);
  };

  const handlePadMouseLeave = (padId: number) => {
    if (overPadRef.current === padId) {
      overPadRef.current = null;
      setVisualOverPad(null);
    }
  };

  // ── Grid layout ────────────────────────────────────────────────────────────

  // Padawan mode: 4×4 using the first 4 columns of rows 0-3
  const visiblePads = gridSize === 64
    ? pads
    : pads.filter((_, i) => {
        const row = Math.floor(i / 8);
        const col = i % 8;
        return row < 4 && col < 4;
      });

  const cols = gridSize === 64 ? 8 : 4;
  const rows = gridSize === 64 ? 8 : 4;
  const isDragging = visualDragFrom !== null;

  return (
    <div
      className="p-2 flex-1 min-h-0"
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${cols}, 1fr)`,
        gridTemplateRows:    `repeat(${rows}, 1fr)`,
        gap: '6px',
        cursor: isDragging ? 'grabbing' : undefined,
      }}
    >
      {visiblePads.map((pad) => (
        <Pad
          key={pad.id}
          pad={pad}
          size={gridSize === 64 ? 'small' : 'large'}
          isDragSource={visualDragFrom === pad.id}
          isDragTarget={
            isDragging &&
            visualOverPad === pad.id &&
            visualDragFrom !== pad.id
          }
          onDragMouseDown={() => handlePadMouseDown(pad.id)}
          onDragMouseEnter={() => handlePadMouseEnter(pad.id)}
          onDragMouseLeave={() => handlePadMouseLeave(pad.id)}
        />
      ))}
    </div>
  );
}

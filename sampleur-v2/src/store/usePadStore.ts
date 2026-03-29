import { create } from 'zustand';
import { PadState, PadMode, COLORS } from '../types';

function createDefaultPad(id: number): PadState {
  return {
    id,
    label: String(id + 1),
    color: COLORS[id % COLORS.length],
    mode: 'oneshot',
    isPlaying: false,
    progress: 0,
    hasSample: false,
    volume: 1.0,
    detuneCents: 0,
    originalBpm: 120,
  };
}

interface PadStore {
  pads: PadState[];
  selectedPadId: number | null;
  kitName: string;
  gridSize: 16 | 64;
  editMode: boolean;

  updatePad: (id: number, partial: Partial<PadState>) => void;
  updateProgress: (updates: Array<{ padId: number; progress: number; isPlaying: boolean }>) => void;
  selectPad: (id: number | null) => void;
  setKitName: (name: string) => void;
  setGridSize: (size: 16 | 64) => void;
  setEditMode: (edit: boolean) => void;
  resetAllPads: () => void;
  loadFromPreset: (pads: (any | null)[], kitName: string, gridSize: 16 | 64) => void;
  /**
   * Swap the *content* of two pads (label, color, sample, mode, volume…).
   * The pad `id` and `midiNote` are intentionally NOT swapped so that the
   * Launchpad hardware mapping stays consistent with the physical grid position.
   */
  swapPads: (idA: number, idB: number) => void;
}

export const usePadStore = create<PadStore>((set) => ({
  pads: Array.from({ length: 64 }, (_, i) => createDefaultPad(i)),
  selectedPadId: null,
  kitName: 'Mon Kit',
  gridSize: 16,
  editMode: true,

  updatePad: (id, partial) => set((state) => ({
    pads: state.pads.map((p) => p.id === id ? { ...p, ...partial } : p),
  })),

  updateProgress: (updates) => set((state) => {
    const newPads = [...state.pads];
    for (const u of updates) {
      if (u.padId < newPads.length) {
        newPads[u.padId] = { ...newPads[u.padId], progress: u.progress, isPlaying: u.isPlaying };
      }
    }
    return { pads: newPads };
  }),

  selectPad: (id) => set({ selectedPadId: id }),
  setKitName: (name) => set({ kitName: name }),
  setGridSize: (size) => set({ gridSize: size }),
  setEditMode: (edit) => set({ editMode: edit }),
  resetAllPads: () => set({
    pads: Array.from({ length: 64 }, (_, i) => createDefaultPad(i)),
    selectedPadId: null,
    // Note: kitName is set explicitly by the caller after this
  }),

  swapPads: (idA, idB) => set((state) => {
    // Fields that belong to the "content" of a pad and should travel with the drag.
    // `id` and `midiNote` are intentionally excluded — they stay with the grid position.
    const CONTENT_KEYS: (keyof PadState)[] = [
      'label', 'color', 'mode', 'volume', 'detuneCents', 'originalBpm',
      'hasSample', 'fileName', 'durationSecs', 'filePath',
      'isPlaying', 'progress',
    ];
    const a = state.pads.find((p) => p.id === idA);
    const b = state.pads.find((p) => p.id === idB);
    if (!a || !b) return {};
    return {
      pads: state.pads.map((p) => {
        if (p.id === idA) {
          const swapped = { ...p };
          for (const k of CONTENT_KEYS) (swapped as any)[k] = (b as any)[k];
          return swapped;
        }
        if (p.id === idB) {
          const swapped = { ...p };
          for (const k of CONTENT_KEYS) (swapped as any)[k] = (a as any)[k];
          return swapped;
        }
        return p;
      }),
    };
  }),

  loadFromPreset: (presetPads, kitName, gridSize) => set((state) => {
    const newPads = state.pads.map((defaultPad) => {
      const p = presetPads[defaultPad.id];
      if (!p) return { ...createDefaultPad(defaultPad.id), hasSample: false };

      // Re-hydrate the full ColorDef (Rust strips tw/twText during load/save).
      // Match by name first, then by midiVelocity, then fall back to default.
      const rawColor = p.color as Partial<typeof COLORS[0]> | undefined;
      const color =
        COLORS.find((c) => c.name.toLowerCase() === rawColor?.name?.toLowerCase()) ??
        COLORS.find((c) => c.midiVelocity === rawColor?.midiVelocity) ??
        defaultPad.color;

      return {
        ...defaultPad,
        label: p.label || String(defaultPad.id + 1),
        color,
        mode: (p.mode || 'oneshot') as PadMode,
        midiNote: p.midiNote,
        volume: p.volume ?? 1.0,
        detuneCents: p.detuneCents ?? 0,
        originalBpm: p.originalBpm ?? 120,
        hasSample: false, // reset, samples are reloaded via absolutePathHint
        isPlaying: false,
        progress: 0,
        filePath: p.sample?.absolutePathHint,
        fileName: p.sample?.fileName,
        durationSecs: p.sample?.durationSecs,
      };
    });
    return { pads: newPads, kitName, gridSize };
  }),
}));

import { create } from 'zustand';
import { FxState } from '../types';

interface FxStore {
  fx: FxState;
  bpm: number;
  quantize: boolean;
  setFxParam: (key: keyof FxState, value: number) => void;
  setBpm: (bpm: number) => void;
  setQuantize: (q: boolean) => void;
  loadFx: (fx: FxState, bpm: number, quantize: boolean) => void;
}

const defaultFx: FxState = {
  distortion: 0,
  filterFreq: 20000,
  filterResonance: 0.707,
  delayTime: 0.3,
  delayFeedback: 0.4,
  delayMix: 0,
  reverbMix: 0,
  gateRate: 0,
  flangerDepth: 0.005,
  flangerRate: 0.5,
  masterVolume: 1.0,
};

export const useFxStore = create<FxStore>((set) => ({
  fx: defaultFx,
  bpm: 120,
  quantize: false,

  setFxParam: (key, value) => set((s) => ({ fx: { ...s.fx, [key]: value } })),
  setBpm: (bpm) => set({ bpm }),
  setQuantize: (q) => set({ quantize: q }),
  loadFx: (fx, bpm, quantize) => set({ fx, bpm, quantize }),
}));

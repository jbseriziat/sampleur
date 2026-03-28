import { create } from 'zustand';

interface MidiStore {
  midiInputs: string[];
  midiOutputs: string[];
  selectedInput: string | null;
  selectedOutput: string | null;
  midiStatus: 'disconnected' | 'connected' | 'error';
  midiStatusMsg: string;
  learnMode: boolean;
  learnPadId: number | null;
  lastNote: number | null;

  setInputs: (inputs: string[]) => void;
  setOutputs: (outputs: string[]) => void;
  setSelectedInput: (input: string | null) => void;
  setSelectedOutput: (output: string | null) => void;
  setStatus: (status: 'disconnected' | 'connected' | 'error', msg?: string) => void;
  startLearn: (padId: number) => void;
  stopLearn: () => void;
  setLastNote: (note: number) => void;
}

export const useMidiStore = create<MidiStore>((set) => ({
  midiInputs: [],
  midiOutputs: [],
  selectedInput: null,
  selectedOutput: null,
  midiStatus: 'disconnected',
  midiStatusMsg: '',
  learnMode: false,
  learnPadId: null,
  lastNote: null,

  setInputs: (inputs) => set({ midiInputs: inputs }),
  setOutputs: (outputs) => set({ midiOutputs: outputs }),
  setSelectedInput: (input) => set({ selectedInput: input }),
  setSelectedOutput: (output) => set({ selectedOutput: output }),
  setStatus: (status, msg = '') => set({ midiStatus: status, midiStatusMsg: msg }),
  startLearn: (padId) => set({ learnMode: true, learnPadId: padId }),
  stopLearn: () => set({ learnMode: false, learnPadId: null }),
  setLastNote: (note) => set({ lastNote: note }),
}));

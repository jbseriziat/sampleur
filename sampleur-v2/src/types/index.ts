export interface ColorDef {
  name: string;
  hex: string;
  midiVelocity: number;
  tw: string; // Tailwind class for background
  twText: string; // Tailwind class for text contrast
}

export const COLORS: ColorDef[] = [
  { name: "Rouge",  hex: "#ef4444", midiVelocity: 5,  tw: "bg-red-500",    twText: "text-white" },
  { name: "Orange", hex: "#f97316", midiVelocity: 9,  tw: "bg-orange-500", twText: "text-white" },
  { name: "Jaune",  hex: "#eab308", midiVelocity: 13, tw: "bg-yellow-500", twText: "text-black" },
  { name: "Vert",   hex: "#22c55e", midiVelocity: 21, tw: "bg-green-500",  twText: "text-white" },
  { name: "Cyan",   hex: "#06b6d4", midiVelocity: 31, tw: "bg-cyan-500",   twText: "text-white" },
  { name: "Bleu",   hex: "#3b82f6", midiVelocity: 45, tw: "bg-blue-500",   twText: "text-white" },
  { name: "Violet", hex: "#a855f7", midiVelocity: 49, tw: "bg-purple-500", twText: "text-white" },
  { name: "Rose",   hex: "#ec4899", midiVelocity: 53, tw: "bg-pink-500",   twText: "text-white" },
];

export type PadMode = "oneshot" | "loop" | "hold";

export interface PadState {
  id: number;
  label: string;
  color: ColorDef;
  mode: PadMode;
  isPlaying: boolean;
  progress: number;       // 0..1
  hasSample: boolean;
  fileName?: string;
  durationSecs?: number;
  midiNote?: number;
  volume: number;         // 0..2
  detuneCents: number;    // -1200..1200
  originalBpm: number;
  // For preset save/load
  filePath?: string;      // absolute path on disk
}

export interface FxState {
  distortion: number;
  filterFreq: number;
  filterResonance: number;
  delayTime: number;
  delayFeedback: number;
  delayMix: number;
  reverbMix: number;
  gateRate: number;
  flangerDepth: number;
  flangerRate: number;
  masterVolume: number;
}

export interface PresetV2 {
  version: number;
  name: string;
  createdAt: string;
  kitMode: "lightweight" | "portable";
  bpm: number;
  quantize: boolean;
  gridSize: 16 | 64;
  fx: FxState;
  pads: (PadConfig | null)[];
}

export interface PadConfig {
  id: number;
  label: string;
  color: ColorDef;
  mode: PadMode;
  midiNote?: number;
  volume: number;
  detuneCents: number;
  originalBpm: number;
  sample?: {
    fileName: string;
    relativePath: string;
    absolutePathHint: string;
    durationSecs: number;
    channels: number;
    sampleRate: number;
  };
}

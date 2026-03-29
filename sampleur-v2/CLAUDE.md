# Sampleur V2 — Claude Code Context

## Project
Tauri v2 desktop sampler app (Rust backend + React/TypeScript frontend).
GitHub: https://github.com/jbseriziat/sampleur
Local: /home/jb/Dev/Sampleur-Project/

## Stack
- Tauri 2.10.1 | Rust 1.94 | React 18 | TypeScript 5.8 | Vite 7 | Tailwind CSS v3 | Zustand v5
- Audio: CPAL 0.15 | Symphonia 0.5 | rubato 0.15 | hound 3.5
- MIDI: midir 0.10

## Directory layout
```
src/                     # React frontend
  App.tsx                # Root layout
  types/index.ts         # PadState, FxState, PresetV2, ColorDef
  store/usePadStore.ts   # 64 pads Zustand state + swapPads
  store/useFxStore.ts    # FX params, BPM, quantize
  store/useMidiStore.ts  # MIDI devices, learn mode
  hooks/useTauriEvents.ts# All Tauri event listeners
  components/            # Header, PadGrid, Pad, FxPanel, PadEditor, PresetPanel

src-tauri/src/
  state.rs               # AudioCommand, FxParam enums, AppState, RecordingState
  audio/engine.rs        # CPAL stream, mixer (1/sqrt(N)), recording tap
  audio/pad.rs           # PadPlayer: oneshot/loop/hold, fractional pos, speed
  audio/loader.rs        # Symphonia decode → f32 PCM
  audio/resampler.rs     # rubato SincFixedIn → 48kHz
  audio/effects/mod.rs   # FxChain: dist→filter→delay→reverb→gate→flanger
  audio/effects/*.rs     # biquad, distortion, delay, reverb, gate, flanger
  midi/engine.rs         # midir input, note routing, learn state machine
  midi/launchpad.rs      # Novation Launchpad MK2 SysEx + LED (COLOR constants)
  preset/schema.rs       # PresetV2 struct (.sampleur2 JSON, no base64)
  preset/io.rs           # save (lightweight/portable) + load with path fallback
  commands/              # audio_commands, fx_commands, midi_commands, preset_commands
  lib.rs                 # Tauri setup: audio init, MIDI auto-connect, 30fps progress
```

## Build commands
```bash
npm run tauri dev          # Full dev (HMR)
npm run dev                # Frontend only (Vite)
npx tsc --noEmit           # TS type check
cargo build --manifest-path src-tauri/Cargo.toml  # Rust check
npm run tauri build        # Production (.deb + .rpm + AppImage)
```

## Key design decisions
- **Mixer gain**: `1/sqrt(N)` equal-power for N concurrent pads
- **Audio pipeline**: CPAL callback → drain mpsc(4096) → mix pads → FxChain → recording tap
- **Sample rate**: All samples resampled to 48000 Hz at load time
- **Drag & drop**: Mouse events ONLY (no HTML5 DnD — unreliable in Tauri/WebKit2GTK)
- **swapPads**: Swaps content fields; keeps `id` and `midiNote` in place (MIDI note = grid position)
- **Preset format**: JSON (.sampleur2), file paths only (no base64). Lightweight = abs path hints; Portable = copies samples
- **Recording**: hound WAV 32-bit float stereo, tap after FxChain, files saved to ~/Sampleur-Recordings/
- **AppImage build**: Requires `APPIMAGE_EXTRACT_AND_RUN=1` env var — Tauri doesn't set it automatically

## IPC commands (invoke)
trigger_pad, stop_all, reset_kit, load_sample, remove_sample, set_pad_config, get_progress,
start_recording, stop_recording, set_fx_param, set_bpm, set_quantize,
get_midi_inputs, get_midi_outputs, set_midi_input, set_midi_output,
init_launchpad, refresh_leds, set_pad_led, assign_midi_note,
start_midi_learn, cancel_midi_learn, reset_leds, save_preset, load_preset

## Tauri events (emit → listen)
playback-progress, sample-loaded, midi-note-received, midi-learn-complete, midi-status, launchpad-mapping-applied

## Launchpad MK2 LED colors (midi_velocity)
Rouge=5 | Orange=9 | Jaune=13 | Vert=21 | Cyan=29 | Bleu=45 | Violet=49 | Rose=53

## Active branch
sampleurv2-2 (merged to main on 2026-03-29)

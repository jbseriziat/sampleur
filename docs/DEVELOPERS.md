# Sampleur V2 — Developer Documentation

> Tauri v2 + Rust + React/TypeScript sampler application with MIDI Launchpad MK2 support.
> GitHub: https://github.com/jbseriziat/sampleur

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Technology Stack](#2-technology-stack)
3. [Repository Structure](#3-repository-structure)
4. [Audio Engine](#4-audio-engine)
5. [Effects Chain](#5-effects-chain)
6. [MIDI Pipeline](#6-midi-pipeline)
7. [IPC Command Reference](#7-ipc-command-reference)
8. [State Management (Frontend)](#8-state-management-frontend)
9. [Preset System](#9-preset-system)
10. [Frontend Components](#10-frontend-components)
11. [Build & Development](#11-build--development)
12. [Design Decisions & Trade-offs](#12-design-decisions--trade-offs)
13. [Known Limitations](#13-known-limitations)
14. [Contributing](#14-contributing)

---

## 1. Architecture Overview

Sampleur V2 is a **Tauri v2 desktop application**: a thin WebView frontend (React/TypeScript) communicates with a Rust backend via Tauri's IPC bridge (`invoke`/`emit`).

```
┌─────────────────────────────────────────────────────────┐
│  WebView (React + Tailwind)                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │ PadGrid  │ │ FxPanel  │ │PadEditor │ │PresetPanel│  │
│  └──────────┘ └──────────┘ └──────────┘ └───────────┘  │
│       │ invoke()                  ↑ emit()              │
├───────┼───────────────────────────┼─────────────────────┤
│  Rust Core (Tauri)                │                     │
│  ┌──────────────────────┐  ┌──────────────────────────┐ │
│  │  Audio Engine (CPAL) │  │  MIDI Engine (midir)     │ │
│  │  - 64 PadPlayers     │  │  - Input routing         │ │
│  │  - FxChain           │  │  - Launchpad LED control │ │
│  │  - Recording tap     │  │  - Learn mode            │ │
│  └──────────┬───────────┘  └──────────┬───────────────┘ │
│             │ mpsc::sync_channel       │                 │
│             └──────────┬──────────────┘                 │
│                        │ AudioCommand enum              │
│                 AppState (Arc<Mutex<>>)                  │
└─────────────────────────────────────────────────────────┘
```

**Communication patterns:**
- **Frontend → Rust:** `invoke(command_name, args)` (synchronous from JS perspective)
- **Rust → Frontend:** `app_handle.emit(event_name, payload)` (async events)
- **Intra-Rust:** `std::sync::mpsc::sync_channel` bounded to 4096 commands

---

## 2. Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Desktop shell | Tauri | 2.10.1 |
| Backend language | Rust | 1.94 |
| Audio I/O | CPAL | 0.15 |
| Audio decode | Symphonia | 0.5 |
| Resampling | rubato SincFixedIn | 0.15 |
| MIDI I/O | midir | 0.10 |
| WAV encode | hound | 3.5 |
| Frontend framework | React | 18 (CDN-free, bundled) |
| Frontend language | TypeScript | 5.8 |
| Build tool | Vite | 7 |
| CSS | Tailwind CSS | 3.4 |
| State management | Zustand | 5 |
| Async runtime | Tokio | 1 (full) |

---

## 3. Repository Structure

```
sampleur-v2/
├── src/                          # React/TypeScript frontend
│   ├── main.tsx                  # React 18 entry point
│   ├── App.tsx                   # Root layout (Header + PadGrid + panels)
│   ├── types/
│   │   └── index.ts              # PadState, FxState, PresetV2, ColorDef interfaces
│   ├── store/
│   │   ├── usePadStore.ts        # Zustand: 64 pads state + swapPads
│   │   ├── useFxStore.ts         # Zustand: FX params, BPM, quantize
│   │   └── useMidiStore.ts       # Zustand: MIDI devices, learn mode
│   ├── hooks/
│   │   └── useTauriEvents.ts     # All Tauri event listeners (listen())
│   └── components/
│       ├── Header.tsx            # Kit name, grid toggle, BPM, recording controls
│       ├── PadGrid.tsx           # 8×8 grid, mouse-event drag-and-drop
│       ├── Pad.tsx               # Single pad: color, mode glyph, progress bar
│       ├── PadEditor.tsx         # Per-pad config: sample, mode, volume, detune, BPM
│       ├── FxPanel.tsx           # Global FX sliders
│       └── PresetPanel.tsx       # Save/load presets, MIDI device selection
│
└── src-tauri/                    # Rust backend
    ├── tauri.conf.json           # App config (1280×800, .sampleur2 association)
    ├── Cargo.toml                # Rust dependencies
    └── src/
        ├── main.rs               # Entry: calls lib::run()
        ├── lib.rs                # Tauri setup: audio init, MIDI init, progress broadcaster
        ├── state.rs              # AudioCommand, FxParam, AppState, RecordingState
        ├── audio/
        │   ├── engine.rs         # CPAL stream, command dispatch, quantize, mixer
        │   ├── pad.rs            # PadPlayer: oneshot/loop/hold, fractional playback
        │   ├── loader.rs         # Symphonia decoder → DecodedAudio
        │   ├── resampler.rs      # rubato SincFixedIn → 48 kHz
        │   └── effects/
        │       ├── mod.rs        # FxChain: full stereo effects pipeline
        │       ├── biquad.rs     # 2-pole lowpass filter
        │       ├── distortion.rs # atan soft saturation
        │       ├── delay.rs      # 5-second delay line
        │       ├── reverb.rs     # Freeverb (8 combs + 4 allpass)
        │       ├── gate.rs       # LFO gate
        │       └── flanger.rs    # LFO chorus/flange
        ├── midi/
        │   ├── engine.rs         # midir input, routing, MIDI learn state machine
        │   ├── launchpad.rs      # Novation Launchpad MK2 SysEx + LED control
        │   └── learn.rs          # MIDI learn helper
        ├── preset/
        │   ├── schema.rs         # PresetV2, PadConfig, FxConfig, SampleRef structs
        │   └── io.rs             # save_preset() / load_preset() with path resolution
        └── commands/
            ├── audio_commands.rs # trigger_pad, load_sample, set_pad_config, recording
            ├── fx_commands.rs    # set_fx_param, set_bpm, set_quantize
            ├── midi_commands.rs  # MIDI device management, Launchpad init, LED refresh
            └── preset_commands.rs# save_preset, load_preset Tauri commands
```

---

## 4. Audio Engine

### Initialization (`lib.rs`)

```rust
let audio_engine = AudioEngine::new(cmd_rx, progress_arc.clone());
let sample_rate = audio_engine.sample_rate; // captured from CPAL device
let shared = Arc::new(AppState {
    audio: Mutex::new(AudioShared { cmd_tx, sample_rate }),
    midi: Mutex::new(MidiShared { ... }),
    recording: Mutex::new(RecordingState::default()),
});
```

The audio engine opens the default CPAL output device (ALSA on Linux, WASAPI on Windows, CoreAudio on macOS), detects the sample rate, and starts a real-time callback thread.

### Audio Callback (per-buffer)

Located in `audio/engine.rs`, the callback runs on a dedicated real-time thread:

```
1. Drain mpsc channel (up to 4096 commands) → update engine state
2. For each output sample pair (L, R):
   a. Check quantize beat boundary (if quantize enabled)
   b. Mix all active PadPlayers at equal-power gain: mix_gain = 1/sqrt(N)
   c. Write to output[i], output[i+1]
3. Apply FxChain.process(output) in-place (stereo interleaved)
4. Recording tap: if rec_tx is Some, try_send(output.to_vec())
```

### PadPlayer (`audio/pad.rs`)

Each of the 64 pads has a `PadPlayer` with:
- `samples: Arc<Vec<f32>>` — stereo interleaved PCM at 48 kHz
- `pos: f64` — fractional playback position (sub-sample accuracy)
- `speed: f64` — computed from `detune_cents` and `original_bpm` vs current BPM

**Speed calculation:**
```rust
let bpm_ratio = current_bpm / original_bpm;
let detune_ratio = 2f64.powf(detune_cents / 1200.0);
self.speed = bpm_ratio * detune_ratio;
```

**Playback modes:**
- `oneshot` — plays once, stops at end
- `loop` — wraps position to 0 at end
- `hold` — plays while trigger is held, stops on release

### Sample Loading (`audio/loader.rs` + `audio/resampler.rs`)

```rust
// 1. Symphonia decodes any format (WAV/MP3/FLAC/OGG/AAC) to f32 planar
let decoded = decode_audio_file(path)?;

// 2. rubato SincFixedIn resamples to 48000 Hz if needed
let resampled = resample_to_rate(decoded, 48000)?;

// 3. Convert to stereo interleaved (mono → duplicate channel)
// 4. Send LoadSample command to audio thread via mpsc
```

### Recording (`audio/engine.rs` + `commands/audio_commands.rs`)

1. Frontend calls `start_recording()` Tauri command
2. Command creates `hound::WavWriter` (32-bit float, stereo, 48 kHz), spawns writer thread
3. `StartRecording { tx }` sent to audio engine → `rec_tx = Some(tx)`
4. Each audio callback: `tx.try_send(output.to_vec())` (non-blocking; drops frame on overflow)
5. Frontend calls `stop_recording()` → `StopRecording` sent → `rec_tx = None`
6. Dropping the `SyncSender` signals the writer thread to stop and finalize WAV header
7. Returns the file path: `~/Sampleur-Recordings/Sampleur_YYYY-MM-DD_HH-MM-SS.wav`

---

## 5. Effects Chain

Signal flow in `audio/effects/mod.rs`:

```
Input buffer (stereo interleaved f32)
    │
    ▼
[1] Distortion   — atan saturation: y = atan(x * drive) * (2/π)
    │
    ▼
[2] Biquad Filter — 2-pole lowpass, Butterworth, separate L/R state
    │              freq: 20..20000 Hz, resonance Q: 0.1..10
    ▼
[3] Delay         — max 5s ring buffer, mono send, stereo return
    │              time: 0..2s, feedback: 0..0.95, mix: 0..1
    ▼
[4] Reverb        — Freeverb stereo (8 comb + 4 allpass per channel)
    │              mix: 0..1 (dry/wet)
    ▼
[5] Gate LFO      — amplitude modulation at LFO rate
    │              rate: 0..20 Hz
    ▼
[6] Flanger       — LFO-modulated short delay (chorus/flange)
    │              depth: 0..0.02s, rate: 0..5 Hz
    ▼
[7] Master Volume + soft clip (tanh)
    │
    ▼
Output buffer
```

All effect parameters are updated via `AudioCommand::SetFxParam(FxParam::*)`.

---

## 6. MIDI Pipeline

### Input Routing

```
MIDI Device (midir callback, hardware thread)
    │
    │ try_send() — non-blocking, drops on full channel
    ▼
mpsc::sync_channel (shared with audio commands)
    │
    ▼
Audio Engine (real-time thread)
    │
    ├── Note On → TriggerPad(id, Start/Toggle)
    └── Note Off → TriggerPad(id, Stop) [hold mode only]
```

### MIDI Note Mapping

- Default mapping: Launchpad MK2 grid note numbers → pad indices 0..63
- Stored in `MidiShared.pad_notes: [Option<u8>; 64]`
- Customizable via `assign_midi_note(pad_id, note)` or MIDI learn

### Launchpad MK2 LED Control (`midi/launchpad.rs`)

**Programmer mode init SysEx:**
```
F0 00 20 29 02 0D 0E 01 F7
```

**Single LED SysEx (velocity = MIDI color):**
```
F0 00 20 29 02 0D 03 [channel=0] [note] [velocity] F7
```

**Color constants (velocity values):**
| Name | Velocity |
|------|---------|
| Rouge | 5 |
| Orange | 9 |
| Jaune | 13 |
| Vert | 21 |
| Cyan | 29 |
| Bleu | 45 |
| Violet | 49 |
| Rose | 53 |

### MIDI Learn State Machine

```
idle
  │ start_midi_learn(pad_id)
  ▼
waiting_for_note { pad_id }
  │ incoming Note On
  ▼
assign_midi_note(pad_id, note)
  │
  ▼
emit("midi-learn-complete", { pad_id, note })
  │
  ▼
idle
```

---

## 7. IPC Command Reference

All commands are registered in `lib.rs` via `tauri::Builder::invoke_handler![]`.

### Audio Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `trigger_pad` | `pad_id: usize, action: String` ("start"\|"stop"\|"toggle") | `Result<(), String>` | Trigger pad playback |
| `stop_all` | — | `Result<(), String>` | Stop all playing pads |
| `reset_kit` | — | `Result<(), String>` | Clear all pads (audio engine) |
| `load_sample` | `pad_id: usize, file_path: String` | `Result<SampleLoadedResult, String>` | Decode + resample + load |
| `remove_sample` | `pad_id: usize` | `Result<(), String>` | Unload sample from pad |
| `set_pad_config` | `pad_id, volume?, detune_cents?, mode?, original_bpm?, midi_note?, color_midi?` | `Result<(), String>` | Update pad parameters |
| `get_progress` | — | `Result<Vec<PadProgress>, String>` | Get playback progress for all pads |
| `start_recording` | — | `Result<String, String>` | Start WAV recording, returns file path |
| `stop_recording` | — | `Result<String, String>` | Stop recording, finalizes WAV, returns path |

### FX Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `set_fx_param` | `param: String, value: f32` | `Result<(), String>` | Update effect parameter |
| `set_bpm` | `bpm: f32` | `Result<(), String>` | Set global BPM (affects all pads) |
| `set_quantize` | `enabled: bool` | `Result<(), String>` | Enable/disable beat quantization |

**`set_fx_param` parameter names:**
`distortion`, `filterFreq`, `filterResonance`, `delayTime`, `delayFeedback`, `delayMix`, `reverbMix`, `gateRate`, `flangerDepth`, `flangerRate`, `masterVolume`

### MIDI Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_midi_inputs` | — | `Vec<String>` | List available MIDI input ports |
| `get_midi_outputs` | — | `Vec<String>` | List available MIDI output ports |
| `set_midi_input` | `port_name: String` | `Result<String, String>` | Connect MIDI input |
| `set_midi_output` | `port_name: String` | `Result<(), String>` | Connect MIDI output (for LEDs) |
| `init_launchpad` | — | `Result<(), String>` | Apply default mapping + programmer mode + LEDs |
| `refresh_leds` | — | `Result<(), String>` | Sync all LED colors from current pad states |
| `set_pad_led` | `pad_id: usize, playing: bool` | `Result<(), String>` | Set single LED (pad color or white) |
| `assign_midi_note` | `pad_id: usize, note: u8` | `Result<(), String>` | Assign MIDI note to pad |
| `start_midi_learn` | `pad_id: usize` | `Result<(), String>` | Enter MIDI learn for pad |
| `cancel_midi_learn` | — | `Result<(), String>` | Cancel MIDI learn |
| `reset_leds` | — | `Result<(), String>` | Turn off all Launchpad LEDs |

### Preset Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `save_preset` | `preset_json: String, kit_mode: String` ("lightweight"\|"portable") | `Result<String, String>` | Opens save dialog, writes .sampleur2 |
| `load_preset` | — | `Result<String, String>` | Opens open dialog, returns preset JSON |

### Tauri Events (Rust → Frontend)

| Event | Payload | Description |
|-------|---------|-------------|
| `playback-progress` | `Array<{ padId, progress, isPlaying }>` | 30 fps progress updates |
| `sample-loaded` | `{ padId, fileName, durationSecs }` | After successful `load_sample` |
| `midi-note-received` | `{ padId, note }` | Incoming MIDI note (for UI feedback) |
| `midi-learn-complete` | `{ padId, note }` | MIDI learn assignment done |
| `midi-status` | `String` | Connection status message |
| `launchpad-mapping-applied` | `{ pads: Array<{padId, note}> }` | After `init_launchpad` |

---

## 8. State Management (Frontend)

### `usePadStore` (Zustand)

Central store for all 64 pads. Initialized with default empty `PadState` for each index.

```typescript
interface PadState {
  id: number;           // 0..63
  label: string;
  color: ColorDef;      // { name, hex, midi_velocity }
  mode: "oneshot" | "loop" | "hold";
  isPlaying: boolean;
  progress: number;     // 0..1
  hasSample: boolean;
  fileName?: string;
  durationSecs?: number;
  midiNote?: number;    // MIDI note number 0..127
  volume: number;       // 0..2 (1.0 = unity gain)
  detuneCents: number;  // -1200..1200
  originalBpm: number;  // 60..200
  filePath?: string;    // absolute path for preset save
}
```

Key action: `swapPads(idA, idB)` — swaps all content fields EXCEPT `id` and `midiNote` (MIDI mapping stays with grid position).

### `useFxStore` (Zustand)

```typescript
// Default FX state (all effects bypassed):
{
  distortion: 0,         // 0..10
  filterFreq: 20000,     // 20..20000 Hz
  filterResonance: 0.707,// 0.1..10
  delayTime: 0.3,        // 0..2 s
  delayFeedback: 0.4,    // 0..0.95
  delayMix: 0,           // 0..1
  reverbMix: 0,          // 0..1
  gateRate: 0,           // 0..20 Hz (0 = off)
  flangerDepth: 0.005,   // 0..0.02 s
  flangerRate: 0.5,      // 0..5 Hz
  masterVolume: 1.0,     // 0..2
}
```

### `useMidiStore` (Zustand)

Tracks MIDI device lists (refreshed via `get_midi_inputs`/`get_midi_outputs`), selected ports, and MIDI learn state.

### Event Wiring (`hooks/useTauriEvents.ts`)

All `listen()` calls are centralized here. Mounted once in `App.tsx`. Handles cleanup via unlisten callbacks returned by `listen()`.

---

## 9. Preset System

### File Format

Files use extension `.sampleur2`, MIME type `application/x-sampleur2`. Format is JSON.

```json
{
  "version": 2,
  "name": "My Kit",
  "createdAt": "2024-01-15T10:30:00Z",
  "kitMode": "lightweight",
  "bpm": 120.0,
  "quantize": false,
  "gridSize": 64,
  "fx": { "distortion": 0, "filterFreq": 20000, ... },
  "pads": [
    {
      "id": 0,
      "label": "Kick",
      "color": { "name": "Rouge", "hex": "#FF4444", "midiVelocity": 5 },
      "mode": "oneshot",
      "midiNote": 36,
      "volume": 1.0,
      "detuneCents": 0,
      "originalBpm": 120.0,
      "sample": {
        "fileName": "kick.wav",
        "relativePath": "./kick.wav",
        "absolutePathHint": "/home/user/Samples/kick.wav",
        "durationSecs": 0.5,
        "channels": 2,
        "sampleRate": 44100
      }
    },
    null,  // empty pad
    ...
  ]
}
```

### Save Modes

**Lightweight:** Stores absolute path hint. Fast, small file (~10 KB). Requires samples to remain at the same path. Best for personal use.

**Portable:** Creates `{preset_name}_samples/` directory alongside the `.sampleur2` file. Copies all samples into it, updates `relativePath`. Self-contained, shareable. Can produce large archives.

### Load Path Resolution

```rust
// 1. Try relative path (relative to preset file location)
// 2. Fall back to absolutePathHint
// 3. If both fail → pad loads as empty (no error thrown)
```

---

## 10. Frontend Components

### App.tsx Layout

```
┌─────────────────────────────────────┐
│ Header (kit name, BPM, controls)    │
├──────────┬──────────────┬───────────┤
│ FxPanel  │  PadGrid     │ PadEditor │
│ (Jedi    │  (8×8 or     │ (edit     │
│  mode)   │   4×4)       │  mode)    │
├──────────┴──────────────┴───────────┤
│ PresetPanel (save/load, MIDI setup) │
└─────────────────────────────────────┘
```

FxPanel is visible when "Jedi" mode is toggled. PadEditor is visible when a pad is selected in edit mode.

### PadGrid.tsx — Drag & Drop

HTML5 Drag & Drop is **not used** (unreliable in Tauri/WebKit2GTK). Instead, mouse events:

```typescript
// Refs for stable drag state (no stale closure):
const dragFromRef = useRef<number | null>(null);
const overPadRef  = useRef<number | null>(null);

// Global mouseup via useEffect:
useEffect(() => {
  const onMouseUp = () => {
    const from = dragFromRef.current;
    const to   = overPadRef.current;
    clearDragState();
    if (from !== null && to !== null && from !== to) {
      executeSwap(from, to); // swapPads() + sync Rust
    }
  };
  window.addEventListener('mouseup', onMouseUp);
  return () => window.removeEventListener('mouseup', onMouseUp);
}, []);
```

### Pad.tsx

Visual states:
- **Idle:** pad color background
- **Playing:** brighter/white overlay, progress bar at bottom
- **Drag source:** reduced opacity (0.5)
- **Drag target:** white ring outline

Mode glyphs: `▷` (oneshot), `∞` (loop), `⊙` (hold)

---

## 11. Build & Development

### Prerequisites

- Rust 1.70+ (`rustup update`)
- Node.js 18+
- On Linux: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libasound2-dev`, `libssl-dev`

### Development

```bash
cd sampleur-v2
npm install
npm run tauri dev       # Opens Tauri window with HMR
```

### Production Build

```bash
npm run tauri build     # Produces .deb, .rpm, AppImage in target/release/bundle/
```

If AppImage generation fails (FUSE issue), build manually:

```bash
# 1. Build without AppImage first
npm run tauri build --bundles deb,rpm

# 2. Package AppImage manually
APPIMAGE_EXTRACT_AND_RUN=1 linuxdeploy-x86_64.AppImage \
  --appdir AppDir --plugin gtk --executable ./sampleur-v2
APPIMAGE_EXTRACT_AND_RUN=1 appimagetool-x86_64.AppImage AppDir Sampleur.AppImage
```

### Rust-only check

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

### TypeScript check

```bash
cd sampleur-v2 && npx tsc --noEmit
```

---

## 12. Design Decisions & Trade-offs

| Decision | Rationale |
|----------|-----------|
| **`1/sqrt(N)` mixer gain** | Equal-power mixing: N simultaneous pads sum to same perceived loudness. Alternative `1/N` would be too quiet for a single pad. |
| **`sync_channel(4096)` not lock-free** | Simplicity over latency. MIDI callbacks use `try_send` to avoid blocking the MIDI thread. Real-time audio thread drains synchronously. |
| **Load-time resampling** | Avoids per-sample-rate conversion during playback. All audio is at 48 kHz internally. Cost paid once on load. |
| **No `chrono` dependency** | Custom `days_to_ymd` / `unix_to_datetime_str` based on Howard Hinnant's algorithm for timestamp generation. Avoids ~500KB dependency. |
| **Mouse events for DnD** | Tauri/WebKit2GTK drops `drop` events unreliably. `useRef` avoids stale closures in the global `mouseup` handler. |
| **`Arc<Vec<f32>>` for samples** | Allows cheap cloning of sample data references for concurrent pad playback without copying. |
| **Fractional `f64` position** | Sub-sample accuracy for BPM stretch and pitch detune. Avoids accumulation errors at high speeds. |
| **No base64 in presets** | V1 used base64-encoded audio in JSON (avg 80 MB per preset). V2 stores paths only → presets are ~10 KB. |
| **Zustand over Redux** | Lower boilerplate, faster prototyping. `usePadStore.getState()` called in event handlers to avoid stale closures. |

---

## 13. Known Limitations

- **AppImage FUSE**: `linuxdeploy` requires FUSE or `APPIMAGE_EXTRACT_AND_RUN=1`. Tauri does not set this automatically — requires manual workaround (see Build section).
- **Audio device detection**: Only the default CPAL output device is used. No device selection UI.
- **WAV recording format**: Always 32-bit float stereo at the device sample rate. No format options.
- **MIDI input only**: MIDI output is used only for Launchpad LED control. No MIDI clock output.
- **Preset portability**: Lightweight presets break if samples are moved. Use portable mode for sharing.
- **Detune range**: ±1200 cents (±1 octave). No wider range currently.
- **Max concurrent pads**: Limited by CPU. No hard cap; mixer gain adjusts automatically.

---

## 14. Contributing

### Branch Strategy

- `main` — stable releases
- `sampleurv2-2` — active development branch (merge into main when stable)

### Code Style

**Rust:** Standard `rustfmt` formatting. Error handling via `anyhow::Result`. All Tauri commands return `Result<T, String>` (serialize-able error type).

**TypeScript:** Functional components, hooks. No class components. `const` everywhere.

### Adding a New Effect

1. Create `src-tauri/src/audio/effects/my_effect.rs`
2. Add the struct and `process()` method
3. Add variant to `FxParam` enum in `state.rs`
4. Add field to `FxChain` in `effects/mod.rs`, call in `process()`
5. Handle new `FxParam` variant in `audio/engine.rs` `handle_command()`
6. Add `set_fx_param` case in `commands/fx_commands.rs`
7. Add slider in `src/components/FxPanel.tsx`
8. Add field to `FxState` in `src/types/index.ts` and `useFxStore.ts`

### Adding a New IPC Command

1. Add function in `commands/your_commands.rs` with `#[tauri::command]`
2. Register in `lib.rs` `invoke_handler![]`
3. Call from frontend: `invoke<ReturnType>("command_name", { args })`

---

*Last updated: 2026-03-29 — Sampleur V2 v2.0.0*

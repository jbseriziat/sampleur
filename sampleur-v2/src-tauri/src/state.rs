use std::sync::{mpsc, Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AudioCommand {
    TriggerPad { id: usize, action: PadAction },
    StopAll,
    /// Clear all pad samples and stop playback — used by "Nouveau Kit"
    ResetKit,
    LoadSample { id: usize, samples: Arc<Vec<f32>>, sample_rate: u32, channels: u16 },
    RemoveSample { id: usize },
    SetPadVolume { id: usize, volume: f32 },
    SetPadDetune { id: usize, detune_cents: f32 },
    SetPadMode { id: usize, mode: PadMode },
    SetPadOriginalBpm { id: usize, bpm: f32 },
    SetFxParam(FxParam),
    SetBpm(f32),
    SetQuantize(bool),
    /// Start capturing the mixed+FX output into the provided channel.
    StartRecording { tx: mpsc::SyncSender<Vec<f32>> },
    /// Drop the recording sender — the writer thread will finalize the WAV on disconnect.
    StopRecording,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PadAction {
    Start,
    Stop,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PadMode {
    Oneshot,
    Loop,
    Hold,
}

#[derive(Debug, Clone)]
pub enum FxParam {
    FilterFreq(f32),
    FilterResonance(f32),
    DelayTime(f32),
    DelayFeedback(f32),
    DelayMix(f32),
    ReverbMix(f32),
    DistortionDrive(f32),
    GateRate(f32),
    FlangerDepth(f32),
    FlangerRate(f32),
    MasterVolume(f32),
}

#[derive(Debug, Clone, Serialize)]
pub struct PadProgress {
    pub pad_id: usize,
    pub progress: f64,  // 0.0 to 1.0
    pub is_playing: bool,
}

// Shared state accessed by Tauri command handlers
pub struct AudioShared {
    pub cmd_tx: mpsc::SyncSender<AudioCommand>,
    /// Native sample rate of the CPAL output device (needed for WAV header).
    pub sample_rate: u32,
}

/// State for the live recording feature.
pub struct RecordingState {
    /// When the recording started (for display only — timer is on the frontend).
    pub start_time: Option<std::time::Instant>,
    /// Absolute path of the WAV file being written.
    pub file_path: Option<String>,
    /// Handle to the writer thread — joined on stop to ensure the WAV is finalized.
    pub writer_handle: Option<std::thread::JoinHandle<()>>,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self { start_time: None, file_path: None, writer_handle: None }
    }
}

pub struct MidiShared {
    pub note_map: std::collections::HashMap<u8, usize>,  // midi_note -> pad_id
    pub pad_modes: [PadMode; 64],
    pub pad_colors: [u8; 64],     // Launchpad MK2 velocity color values
    pub pad_has_sample: [bool; 64],
    pub learn_state: MidiLearnState,
    pub output_conn: Option<midir::MidiOutputConnection>,
    /// Active MIDI input connection — stored here so it can be hot-swapped via set_midi_input.
    /// Dropping this stops the MIDI callback thread (safe because callback uses try_lock).
    pub input_conn: Option<midir::MidiInputConnection<()>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MidiLearnState {
    Idle,
    WaitingForNote { pad_id: usize },
}

impl Default for MidiShared {
    fn default() -> Self {
        Self {
            note_map: std::collections::HashMap::new(),
            pad_modes: std::array::from_fn(|_| PadMode::Oneshot),
            pad_colors: [0u8; 64],
            pad_has_sample: [false; 64],
            learn_state: MidiLearnState::Idle,
            output_conn: None,
            input_conn: None,
        }
    }
}

pub struct AppState {
    pub audio: Mutex<AudioShared>,
    pub midi: Arc<Mutex<MidiShared>>,
    pub progress: Arc<Mutex<Vec<PadProgress>>>,
    pub bpm: Arc<Mutex<f32>>,
    pub recording: Mutex<RecordingState>,
}

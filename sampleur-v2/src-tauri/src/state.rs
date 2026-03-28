use std::sync::{mpsc, Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AudioCommand {
    TriggerPad { id: usize, action: PadAction },
    StopAll,
    LoadSample { id: usize, samples: Arc<Vec<f32>>, sample_rate: u32, channels: u16 },
    RemoveSample { id: usize },
    SetPadVolume { id: usize, volume: f32 },
    SetPadDetune { id: usize, detune_cents: f32 },
    SetPadMode { id: usize, mode: PadMode },
    SetPadOriginalBpm { id: usize, bpm: f32 },
    SetFxParam(FxParam),
    SetBpm(f32),
    SetQuantize(bool),
    StartRecording,
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
}

pub struct MidiShared {
    pub note_map: std::collections::HashMap<u8, usize>,  // midi_note -> pad_id
    pub pad_modes: [PadMode; 64],
    pub pad_colors: [u8; 64],     // Launchpad MK2 velocity color values
    pub pad_has_sample: [bool; 64],
    pub learn_state: MidiLearnState,
    pub output_conn: Option<midir::MidiOutputConnection>,
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
        }
    }
}

pub struct AppState {
    pub audio: Mutex<AudioShared>,
    pub midi: Arc<Mutex<MidiShared>>,
    pub progress: Arc<Mutex<Vec<PadProgress>>>,
    pub bpm: Arc<Mutex<f32>>,
}

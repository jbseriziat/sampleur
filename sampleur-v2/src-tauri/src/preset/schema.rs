use serde::{Deserialize, Serialize};
use crate::state::PadMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetV2 {
    pub version: u8,
    pub name: String,
    pub created_at: String,
    pub kit_mode: KitMode,
    pub bpm: f32,
    pub quantize: bool,
    pub grid_size: u8,   // 16 or 64
    pub fx: FxConfig,
    pub pads: Vec<Option<PadConfig>>,  // None = empty pad
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KitMode {
    Lightweight,
    Portable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FxConfig {
    pub distortion: f32,
    pub filter_freq: f32,
    pub filter_resonance: f32,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,
    pub reverb_mix: f32,
    pub gate_rate: f32,
    pub flanger_depth: f32,
    pub flanger_rate: f32,
    pub master_volume: f32,
}

impl Default for FxConfig {
    fn default() -> Self {
        Self {
            distortion: 0.0,
            filter_freq: 20000.0,
            filter_resonance: 0.707,
            delay_time: 0.3,
            delay_feedback: 0.4,
            delay_mix: 0.0,
            reverb_mix: 0.0,
            gate_rate: 0.0,
            flanger_depth: 0.005,
            flanger_rate: 0.5,
            master_volume: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PadConfig {
    pub id: usize,
    pub label: String,
    pub color: ColorDef,
    pub mode: PadMode,
    pub midi_note: Option<u8>,
    pub volume: f32,
    pub detune_cents: f32,
    pub original_bpm: f32,
    pub sample: Option<SampleRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleRef {
    pub file_name: String,
    pub relative_path: String,
    pub absolute_path_hint: String,
    pub duration_secs: f64,
    pub channels: u16,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorDef {
    pub name: String,
    pub hex: String,
    pub midi_velocity: u8,
}

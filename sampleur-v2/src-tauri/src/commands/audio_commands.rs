use std::sync::Arc;
use tauri::State;
use crate::state::{AppState, AudioCommand, PadAction, PadMode};
use crate::audio::{loader, resampler};

#[tauri::command]
pub async fn trigger_pad(
    state: State<'_, AppState>,
    pad_id: usize,
    action: String,
) -> Result<(), String> {
    let action = match action.as_str() {
        "start" => PadAction::Start,
        "stop" => PadAction::Stop,
        "toggle" => PadAction::Toggle,
        _ => return Err(format!("Unknown action: {}", action)),
    };
    let audio = state.audio.lock().map_err(|e| e.to_string())?;
    audio.cmd_tx.try_send(AudioCommand::TriggerPad { id: pad_id, action })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_all(state: State<'_, AppState>) -> Result<(), String> {
    let audio = state.audio.lock().map_err(|e| e.to_string())?;
    audio.cmd_tx.try_send(AudioCommand::StopAll).map_err(|e| e.to_string())
}

/// Reset all pads: stops playback, unloads all samples, clears MIDI pad state.
/// The frontend must also call resetAllPads() on its store.
#[tauri::command]
pub async fn reset_kit(state: State<'_, AppState>) -> Result<(), String> {
    {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;
        audio.cmd_tx.try_send(AudioCommand::ResetKit).map_err(|e| e.to_string())?;
    }
    {
        let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
        midi.pad_has_sample = [false; 64];
    }
    Ok(())
}

#[derive(serde::Serialize)]
pub struct SampleLoadedResult {
    pub pad_id: usize,
    pub file_name: String,
    pub duration_secs: f64,
}

#[tauri::command]
pub async fn load_sample(
    state: State<'_, AppState>,
    pad_id: usize,
    file_path: String,
) -> Result<SampleLoadedResult, String> {
    let path = std::path::PathBuf::from(&file_path);
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Decode on Tokio thread pool (blocking I/O)
    let decoded = tokio::task::spawn_blocking(move || {
        loader::decode_audio_file(&path)
    }).await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    let duration_secs = decoded.duration_secs;

    // Standard output rate — engine handles conversion at load time
    let target_rate = 48000u32;

    let samples = if decoded.sample_rate != target_rate {
        let src_rate = decoded.sample_rate;
        let src_channels = decoded.channels;
        let src_samples = decoded.samples;
        tokio::task::spawn_blocking(move || {
            resampler::resample_to_rate(src_samples, src_rate, target_rate, src_channels)
        }).await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?
    } else {
        decoded.samples
    };

    let samples = Arc::new(samples);

    // Update MIDI shared state
    {
        let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
        midi.pad_has_sample[pad_id] = true;
    }

    // Send to audio engine
    {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;
        audio.cmd_tx.try_send(AudioCommand::LoadSample {
            id: pad_id,
            samples,
            sample_rate: target_rate,
            channels: 2,
        }).map_err(|e| e.to_string())?;
    }

    Ok(SampleLoadedResult { pad_id, file_name, duration_secs })
}

#[tauri::command]
pub async fn remove_sample(
    state: State<'_, AppState>,
    pad_id: usize,
) -> Result<(), String> {
    {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;
        audio.cmd_tx.try_send(AudioCommand::RemoveSample { id: pad_id })
            .map_err(|e| e.to_string())?;
    }
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    midi.pad_has_sample[pad_id] = false;
    Ok(())
}

#[tauri::command]
pub async fn set_pad_config(
    state: State<'_, AppState>,
    pad_id: usize,
    volume: Option<f32>,
    detune_cents: Option<f32>,
    mode: Option<String>,
    original_bpm: Option<f32>,
    midi_note: Option<Option<u8>>,
    color_midi: Option<u8>,
) -> Result<(), String> {
    {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;

        if let Some(v) = volume {
            audio.cmd_tx.try_send(AudioCommand::SetPadVolume { id: pad_id, volume: v }).ok();
        }
        if let Some(d) = detune_cents {
            audio.cmd_tx.try_send(AudioCommand::SetPadDetune { id: pad_id, detune_cents: d }).ok();
        }
        if let Some(bpm) = original_bpm {
            audio.cmd_tx.try_send(AudioCommand::SetPadOriginalBpm { id: pad_id, bpm }).ok();
        }

        if let Some(ref m) = mode {
            let pad_mode = match m.as_str() {
                "loop" => PadMode::Loop,
                "hold" => PadMode::Hold,
                _ => PadMode::Oneshot,
            };
            audio.cmd_tx.try_send(AudioCommand::SetPadMode { id: pad_id, mode: pad_mode }).ok();
        }
    } // drop audio lock

    // Update MIDI shared for mode and note/color
    {
        let mut midi = state.midi.lock().map_err(|e| e.to_string())?;

        if let Some(ref m) = mode {
            let pad_mode = match m.as_str() {
                "loop" => PadMode::Loop,
                "hold" => PadMode::Hold,
                _ => PadMode::Oneshot,
            };
            midi.pad_modes[pad_id] = pad_mode;
        }

        if let Some(note_opt) = midi_note {
            // Remove old mapping for this pad
            midi.note_map.retain(|_, &mut pid| pid != pad_id);
            if let Some(note) = note_opt {
                midi.note_map.insert(note, pad_id);
            }
        }
        if let Some(color) = color_midi {
            midi.pad_colors[pad_id] = color;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_progress(state: State<'_, AppState>) -> Result<Vec<crate::state::PadProgress>, String> {
    let progress = state.progress.lock().map_err(|e| e.to_string())?;
    Ok(progress.clone())
}

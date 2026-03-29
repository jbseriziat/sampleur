use std::sync::Arc;
use tauri::State;
use crate::state::{AppState, AudioCommand, PadAction, PadMode, RecordingState};
extern crate hound;
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

/// Start live recording to a WAV file (PCM 32-bit float, stereo).
/// The file is auto-named with a timestamp and saved to ~/Sampleur-Recordings/.
/// Returns the absolute path of the recording file.
#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<String, String> {
    // Guard: do not start a second recording while one is already in progress.
    {
        let rec = state.recording.lock().map_err(|e| e.to_string())?;
        if rec.start_time.is_some() {
            return Err("Recording already in progress".into());
        }
    }

    let sample_rate = {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;
        audio.sample_rate
    };

    // Build output path: ~/Sampleur-Recordings/Sampleur_YYYY-MM-DD_HH-MM-SS.wav
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let recordings_dir = std::path::PathBuf::from(&home).join("Sampleur-Recordings");
    std::fs::create_dir_all(&recordings_dir).map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Convert unix timestamp to simple YYYYMMDD_HHMMSS string
    let dt = unix_to_datetime_str(now);
    let file_name = format!("Sampleur_{}.wav", dt);
    let file_path = recordings_dir.join(&file_name);
    let file_path_str = file_path.to_string_lossy().to_string();

    // Create the WAV writer
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let writer = hound::WavWriter::create(&file_path, spec)
        .map_err(|e| e.to_string())?;

    // Bounded channel: 256 buffers × ~512 samples × 4 bytes × 2 ch ≈ 1 MB max backlog
    let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(256);

    // Spawn a dedicated writer thread — it owns the WavWriter and finalizes on channel close.
    let writer_handle = std::thread::spawn(move || {
        let mut w = writer;
        while let Ok(buf) = rx.recv() {
            for sample in buf {
                let _ = w.write_sample(sample);
            }
        }
        // Sender dropped (StopRecording) → finalize the WAV header
        let _ = w.finalize();
    });

    // Send the sender to the audio engine
    {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;
        audio.cmd_tx
            .try_send(AudioCommand::StartRecording { tx })
            .map_err(|e| e.to_string())?;
    }

    // Store recording state
    {
        let mut rec = state.recording.lock().map_err(|e| e.to_string())?;
        *rec = RecordingState {
            start_time: Some(std::time::Instant::now()),
            file_path: Some(file_path_str.clone()),
            writer_handle: Some(writer_handle),
        };
    }

    log::info!("Recording started: {}", file_path_str);
    Ok(file_path_str)
}

/// Stop live recording and finalize the WAV file.
/// Returns the path of the saved file.
#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<String, String> {
    // Send stop command to audio engine — this drops the SyncSender inside the callback,
    // which signals the writer thread to exit and finalize the WAV.
    {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;
        audio.cmd_tx
            .try_send(AudioCommand::StopRecording)
            .map_err(|e| e.to_string())?;
    }

    // Retrieve state and clear it
    let (file_path, writer_handle) = {
        let mut rec = state.recording.lock().map_err(|e| e.to_string())?;
        let path = rec.file_path.take().unwrap_or_default();
        let handle = rec.writer_handle.take();
        rec.start_time = None;
        (path, handle)
    };

    // Wait for the writer thread to finalize the WAV (blocking, but fast)
    if let Some(handle) = writer_handle {
        tokio::task::spawn_blocking(move || {
            let _ = handle.join();
        })
        .await
        .map_err(|e| e.to_string())?;
    }

    log::info!("Recording stopped: {}", file_path);
    Ok(file_path)
}

/// Convert a Unix timestamp (seconds) to a "YYYY-MM-DD_HH-MM-SS" string.
/// This avoids pulling in a heavy date/time crate.
fn unix_to_datetime_str(unix_secs: u64) -> String {
    // Simple integer arithmetic — no DST, no timezones (UTC-like)
    let s  = unix_secs % 60;
    let m  = (unix_secs / 60) % 60;
    let h  = (unix_secs / 3600) % 24;
    // Days since epoch
    let days = unix_secs / 86400;
    // Gregorian calendar approximation
    let (year, month, day) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", year, month, day, h, m, s)
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

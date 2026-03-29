mod state;
mod audio;
mod midi;
mod preset;
mod commands;

use std::sync::{mpsc, Arc, Mutex};
use tauri::Emitter;
use state::{AppState, AudioShared, MidiShared, PadProgress};
use audio::engine::AudioEngine;
use commands::{
    audio_commands::{trigger_pad, stop_all, reset_kit, load_sample, remove_sample, set_pad_config, get_progress},
    fx_commands::{set_fx_param, set_bpm, set_quantize},
    midi_commands::{
        get_midi_inputs, get_midi_outputs,
        set_midi_output, set_midi_input,
        init_launchpad, refresh_leds, set_pad_led,
        assign_midi_note, start_midi_learn, cancel_midi_learn,
        reset_leds,
    },
    preset_commands::{save_preset, load_preset},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    // Create audio command channel (bounded MPSC)
    let (cmd_tx, cmd_rx) = mpsc::sync_channel::<state::AudioCommand>(4096);

    // Shared progress state
    let progress_state: Arc<Mutex<Vec<PadProgress>>> = Arc::new(Mutex::new(
        (0..64).map(|i| PadProgress { pad_id: i, progress: 0.0, is_playing: false }).collect()
    ));

    // MIDI shared state
    let midi_shared = Arc::new(Mutex::new(MidiShared::default()));

    // BPM state
    let bpm_state = Arc::new(Mutex::new(120.0_f32));

    // Start audio engine
    let audio_engine = AudioEngine::new(cmd_rx, Arc::clone(&progress_state))
        .expect("Failed to start audio engine");
    let _ = audio_engine; // Keep alive

    let app_state = AppState {
        audio: Mutex::new(AudioShared { cmd_tx: cmd_tx.clone() }),
        midi: Arc::clone(&midi_shared),
        progress: Arc::clone(&progress_state),
        bpm: Arc::clone(&bpm_state),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let midi_shared_setup = Arc::clone(&midi_shared);
            let cmd_tx_midi = cmd_tx.clone();

            // Auto-connect MIDI input (Launchpad auto-detect, no blocking loop needed:
            // the connection is stored in MidiShared.input_conn and kept alive there).
            std::thread::spawn(move || {
                match midi::engine::connect_input(
                    cmd_tx_midi,
                    midi_shared_setup,
                    app_handle.clone(),
                    None, // auto-detect Launchpad, fall back to first port
                ) {
                    Ok(port_name) => {
                        log::info!("MIDI auto-connected to: {}", port_name);
                        let _ = app_handle.emit("midi-status", serde_json::json!({
                            "status": "connected",
                            "portName": port_name,
                        }));
                    }
                    Err(e) => {
                        log::warn!("MIDI auto-connect failed: {}", e);
                        let _ = app_handle.emit("midi-status", serde_json::json!({
                            "status": "error",
                            "message": e.to_string(),
                        }));
                    }
                }
            });

            // Spawn progress broadcaster at ~30fps
            let app_handle2 = app.handle().clone();
            let progress_broadcaster = Arc::clone(&progress_state);
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(33));
                loop {
                    interval.tick().await;
                    if let Ok(progress) = progress_broadcaster.try_lock() {
                        // Only emit if something is playing
                        if progress.iter().any(|p| p.is_playing) {
                            let _ = app_handle2.emit("playback-progress", &*progress);
                        }
                    }
                }
            });

            // Spawn beat ticker
            let app_handle3 = app.handle().clone();
            let bpm_for_ticker = Arc::clone(&bpm_state);
            tauri::async_runtime::spawn(async move {
                let mut beat_phase = 0.0f64;
                let mut last_tick = std::time::Instant::now();
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
                    let now = std::time::Instant::now();
                    let elapsed = now.duration_since(last_tick).as_secs_f64();
                    last_tick = now;

                    let bpm = *bpm_for_ticker.lock().unwrap();
                    beat_phase += elapsed * bpm as f64 / 60.0;
                    if beat_phase >= 1.0 { beat_phase -= beat_phase.floor(); }

                    let _ = app_handle3.emit("beat-tick", serde_json::json!({ "beatPhase": beat_phase }));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            trigger_pad,
            stop_all,
            reset_kit,
            load_sample,
            remove_sample,
            set_pad_config,
            get_progress,
            set_fx_param,
            set_bpm,
            set_quantize,
            get_midi_inputs,
            get_midi_outputs,
            set_midi_output,
            set_midi_input,
            init_launchpad,
            refresh_leds,
            set_pad_led,
            assign_midi_note,
            start_midi_learn,
            cancel_midi_learn,
            reset_leds,
            save_preset,
            load_preset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

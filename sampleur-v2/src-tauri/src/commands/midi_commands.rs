use std::sync::Arc;
use std::time::Duration;
use tauri::{State, Emitter};
use crate::state::AppState;
use crate::state::MidiLearnState;
use crate::midi::{launchpad, engine as midi_engine};

#[tauri::command]
pub async fn get_midi_inputs() -> Vec<String> {
    midi_engine::list_midi_inputs()
}

#[tauri::command]
pub async fn get_midi_outputs() -> Vec<String> {
    midi_engine::list_midi_outputs()
}

#[tauri::command]
pub async fn set_midi_output(
    state: State<'_, AppState>,
    port_name: String,
) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = midi_engine::connect_output(&port_name) {
        midi.output_conn = Some(conn);
        Ok(())
    } else {
        Err(format!("Cannot connect to MIDI output: {}", port_name))
    }
}

/// Connect (or reconnect) the MIDI input to the given port.
/// Stores the connection in MidiShared.input_conn — dropping any previous one safely.
#[tauri::command]
pub async fn set_midi_input(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    port_name: String,
) -> Result<String, String> {
    let cmd_tx = {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;
        audio.cmd_tx.clone()
    };
    let midi_shared = Arc::clone(&state.midi);

    tokio::task::spawn_blocking(move || {
        midi_engine::connect_input(cmd_tx, midi_shared, app, Some(port_name.as_str()))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Initialise the Launchpad MK2:
///  1. Apply the default note mapping (grid layout) if note_map is empty.
///  2. Emit "launchpad-mapping-applied" so the frontend updates midiNote on each pad.
///  3. Send SysEx Programmer-mode commands.
///  4. Wait 150 ms for the hardware to switch.
///  5. Light up all 64 LEDs.
#[tauri::command]
pub async fn init_launchpad(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // ── Step 1 & 2: apply default mapping + collect LED data ────────────────
    let (mapping_payload, led_data) = {
        let mut midi = state.midi.lock().map_err(|e| e.to_string())?;

        // Apply default Launchpad MK2 Programmer-mode layout if note_map is empty.
        // pad 0 → note 81 (top-left), pad 63 → note 11 (bottom-right)
        if midi.note_map.is_empty() {
            for pad_id in 0..64usize {
                let note = launchpad::pad_id_to_note(pad_id);
                midi.note_map.insert(note, pad_id);
                if midi.pad_colors[pad_id] == 0 {
                    midi.pad_colors[pad_id] = launchpad::PAD_COLORS_BY_INDEX[pad_id % 8];
                }
            }
        }

        // Build payload for the frontend: [{padId, note}, …]
        let mapping: Vec<serde_json::Value> = midi.note_map.iter()
            .map(|(&note, &pad_id)| serde_json::json!({ "padId": pad_id, "note": note }))
            .collect();

        // Build LED list: always use pad_id_to_note for physical position
        let led_data: Vec<(u8, u8)> = (0..64usize).map(|pad_id| {
            let note = launchpad::pad_id_to_note(pad_id);
            let color = if midi.pad_has_sample[pad_id] {
                midi.pad_colors[pad_id]
            } else {
                launchpad::COLOR_OFF
            };
            (note, color)
        }).collect();

        (serde_json::json!({ "mapping": mapping }), led_data)
    }; // mutex released here

    // ── Step 2: notify the frontend ─────────────────────────────────────────
    let _ = app.emit("launchpad-mapping-applied", &mapping_payload);

    // ── Step 3: send SysEx Programmer-mode commands ─────────────────────────
    {
        let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut conn) = midi.output_conn {
            launchpad::init_programmer_mode(conn);
        }
    }

    // ── Step 4: let the hardware switch modes ────────────────────────────────
    tokio::time::sleep(Duration::from_millis(150)).await;

    // ── Step 5: light up all pads ────────────────────────────────────────────
    {
        let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut conn) = midi.output_conn {
            for (note, color) in led_data {
                launchpad::set_led(conn, note, color);
            }
        }
    }

    Ok(())
}

/// Re-sync all 64 LEDs with the current pad state (has_sample + color).
#[tauri::command]
pub async fn refresh_leds(state: State<'_, AppState>) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    // Collect separately to avoid double-borrow on `midi`
    let led_data: Vec<(u8, u8)> = (0..64usize).map(|pad_id| {
        let note = launchpad::pad_id_to_note(pad_id);
        let color = if midi.pad_has_sample[pad_id] {
            midi.pad_colors[pad_id]
        } else {
            launchpad::COLOR_OFF
        };
        (note, color)
    }).collect();
    if let Some(ref mut conn) = midi.output_conn {
        for (note, color) in led_data {
            launchpad::set_led(conn, note, color);
        }
    }
    Ok(())
}

/// Set the LED of a single pad — bright green while playing, pad color when stopped.
#[tauri::command]
pub async fn set_pad_led(
    state: State<'_, AppState>,
    pad_id: usize,
    playing: bool,
) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    let note = launchpad::pad_id_to_note(pad_id);
    let color = if playing {
        launchpad::COLOR_GREEN
    } else if midi.pad_has_sample[pad_id] {
        midi.pad_colors[pad_id]
    } else {
        launchpad::COLOR_OFF
    };
    if let Some(ref mut conn) = midi.output_conn {
        launchpad::set_led(conn, note, color);
    }
    Ok(())
}

#[tauri::command]
pub async fn assign_midi_note(
    state: State<'_, AppState>,
    pad_id: usize,
    note: u8,
) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    midi.note_map.retain(|_, &mut pid| pid != pad_id);
    midi.note_map.insert(note, pad_id);
    Ok(())
}

#[tauri::command]
pub async fn start_midi_learn(
    state: State<'_, AppState>,
    pad_id: usize,
) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    midi.learn_state = MidiLearnState::WaitingForNote { pad_id };
    Ok(())
}

#[tauri::command]
pub async fn cancel_midi_learn(state: State<'_, AppState>) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    midi.learn_state = MidiLearnState::Idle;
    Ok(())
}

#[tauri::command]
pub async fn reset_leds(state: State<'_, AppState>) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut conn) = midi.output_conn {
        launchpad::clear_all_leds(conn);
    }
    Ok(())
}

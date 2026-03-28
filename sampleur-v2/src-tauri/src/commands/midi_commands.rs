use tauri::State;
use crate::state::{AppState, MidiLearnState};
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

#[tauri::command]
pub async fn init_launchpad(state: State<'_, AppState>) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    // Collect LED data before borrowing conn mutably
    let led_data: Vec<(u8, u8)> = (0..64usize).filter_map(|pad_id| {
        let note = midi.note_map.iter()
            .find(|(_, &pid)| pid == pad_id)
            .map(|(&note, _)| note)?;
        let color = if midi.pad_has_sample[pad_id] { midi.pad_colors[pad_id] } else { launchpad::COLOR_OFF };
        Some((note, color))
    }).collect();
    if let Some(ref mut conn) = midi.output_conn {
        launchpad::init_programmer_mode(conn);
        for (note, color) in led_data {
            launchpad::set_led(conn, note, color);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn refresh_leds(state: State<'_, AppState>) -> Result<(), String> {
    let mut midi = state.midi.lock().map_err(|e| e.to_string())?;
    // Collect LED data before borrowing conn mutably
    let led_data: Vec<(u8, u8)> = (0..64usize).filter_map(|pad_id| {
        let note = midi.note_map.iter()
            .find(|(_, &pid)| pid == pad_id)
            .map(|(&note, _)| note)?;
        let color = if midi.pad_has_sample[pad_id] { midi.pad_colors[pad_id] } else { launchpad::COLOR_OFF };
        Some((note, color))
    }).collect();
    if let Some(ref mut conn) = midi.output_conn {
        for (note, color) in led_data {
            launchpad::set_led(conn, note, color);
        }
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

use std::sync::{mpsc, Arc, Mutex};
use midir::{MidiInput, MidiOutput, MidiOutputConnection};
use tauri::Emitter;
use crate::state::{AudioCommand, MidiShared, PadAction};

/// Connect to a MIDI input port and store the connection inside MidiShared.
/// Dropping MidiShared.input_conn stops the callback thread safely because
/// the callback uses try_lock() (non-blocking) and never holds the mutex across yields.
///
/// port_name_filter = None  → auto-detect Launchpad, fall back to first port
/// port_name_filter = Some  → match by exact name or substring
pub fn connect_input(
    cmd_tx: mpsc::SyncSender<AudioCommand>,
    midi_shared: Arc<Mutex<MidiShared>>,
    app_handle: tauri::AppHandle,
    port_name_filter: Option<&str>,
) -> anyhow::Result<String> {
    let input = MidiInput::new("sampleur-midi-in")?;
    let ports = input.ports();

    if ports.is_empty() {
        return Err(anyhow::anyhow!("No MIDI input ports found"));
    }

    let port = if let Some(filter) = port_name_filter {
        ports.iter()
            .find(|p| input.port_name(p).map(|n| n == filter).unwrap_or(false))
            .or_else(|| ports.iter().find(|p| {
                input.port_name(p)
                    .map(|n| n.to_lowercase().contains(&filter.to_lowercase()))
                    .unwrap_or(false)
            }))
    } else {
        ports.iter()
            .find(|p| input.port_name(p)
                .map(|n| n.to_lowercase().contains("launchpad"))
                .unwrap_or(false))
            .or_else(|| ports.first())
    };

    let port = port.ok_or_else(|| anyhow::anyhow!("No matching MIDI input port"))?;
    let port = port.clone();
    let connected_name = input.port_name(&port).unwrap_or_else(|_| "unknown".into());
    log::info!("Connecting MIDI input: {}", connected_name);

    let midi_shared_cb = Arc::clone(&midi_shared);
    let cmd_tx_cb = cmd_tx;
    let app_handle_cb = app_handle;

    let conn = input
        .connect(
            &port,
            "sampleur-in",
            move |_stamp, msg, _| {
                handle_midi_message(msg, &cmd_tx_cb, &midi_shared_cb, &app_handle_cb);
            },
            (),
        )
        .map_err(|e| anyhow::anyhow!("MIDI connect error: {}", e))?;

    // Store in MidiShared — dropping any old connection first.
    // Safe: old callback uses try_lock() so it won't deadlock.
    {
        let mut shared = midi_shared.lock().unwrap();
        shared.input_conn = Some(conn);
    }

    Ok(connected_name)
}

fn handle_midi_message(
    msg: &[u8],
    cmd_tx: &mpsc::SyncSender<AudioCommand>,
    midi_shared: &Arc<Mutex<MidiShared>>,
    app_handle: &tauri::AppHandle,
) {
    if msg.len() < 2 {
        return;
    }
    let status = msg[0] & 0xF0;
    let note = msg[1];
    let velocity = msg.get(2).copied().unwrap_or(0);

    let is_note_on = status == 0x90 && velocity > 0;
    let is_note_off = status == 0x80 || (status == 0x90 && velocity == 0);

    if is_note_on {
        let _ = app_handle.emit(
            "midi-note-received",
            serde_json::json!({ "note": note, "velocity": velocity }),
        );

        if let Ok(mut shared) = midi_shared.try_lock() {
            // MIDI Learn — inline to avoid double &mut borrow
            let learn_result =
                if let crate::state::MidiLearnState::WaitingForNote { pad_id } = shared.learn_state {
                    shared.note_map.retain(|_, &mut pid| pid != pad_id);
                    shared.note_map.insert(note, pad_id);
                    shared.learn_state = crate::state::MidiLearnState::Idle;
                    Some((pad_id, note))
                } else {
                    None
                };

            if let Some((pad_id, learned_note)) = learn_result {
                let _ = app_handle.emit(
                    "midi-learn-complete",
                    serde_json::json!({ "padId": pad_id, "note": learned_note }),
                );
                return;
            }

            // Regular trigger
            if let Some(&pad_id) = shared.note_map.get(&note) {
                let mode = shared.pad_modes[pad_id].clone();
                let action = match mode {
                    crate::state::PadMode::Loop => PadAction::Toggle,
                    _ => PadAction::Start,
                };
                let _ = cmd_tx.try_send(AudioCommand::TriggerPad { id: pad_id, action });
            }
        }
    } else if is_note_off {
        if let Ok(shared) = midi_shared.try_lock() {
            if let Some(&pad_id) = shared.note_map.get(&note) {
                if shared.pad_modes[pad_id] == crate::state::PadMode::Hold {
                    let _ = cmd_tx.try_send(AudioCommand::TriggerPad {
                        id: pad_id,
                        action: PadAction::Stop,
                    });
                }
            }
        }
    }
}

pub fn list_midi_inputs() -> Vec<String> {
    match MidiInput::new("sampleur-list") {
        Ok(midi_in) => midi_in
            .ports()
            .iter()
            .filter_map(|p| midi_in.port_name(p).ok())
            .collect(),
        Err(_) => vec![],
    }
}

pub fn list_midi_outputs() -> Vec<String> {
    match MidiOutput::new("sampleur-list") {
        Ok(midi_out) => midi_out
            .ports()
            .iter()
            .filter_map(|p| midi_out.port_name(p).ok())
            .collect(),
        Err(_) => vec![],
    }
}

pub fn connect_output(port_name: &str) -> Option<MidiOutputConnection> {
    let midi_out = MidiOutput::new("sampleur-out").ok()?;
    let ports = midi_out.ports();
    let port = ports.iter().find(|p| {
        midi_out.port_name(p).map(|n| n == port_name).unwrap_or(false)
    })?;
    midi_out.connect(port, "sampleur-out").ok()
}

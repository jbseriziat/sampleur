use crate::state::MidiLearnState;

pub fn handle_learn(
    learn_state: &mut MidiLearnState,
    note_map: &mut std::collections::HashMap<u8, usize>,
    received_note: u8,
) -> Option<(usize, u8)> {  // Returns (pad_id, note) if learn completed
    if let MidiLearnState::WaitingForNote { pad_id } = *learn_state {
        // Remove any existing mapping for this note
        note_map.retain(|_, &mut pid| pid != pad_id);
        note_map.insert(received_note, pad_id);
        *learn_state = MidiLearnState::Idle;
        Some((pad_id, received_note))
    } else {
        None
    }
}

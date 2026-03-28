use midir::MidiOutputConnection;

pub const SYSEX_HEADER: &[u8] = &[0xF0, 0x00, 0x20, 0x29, 0x02];
pub const SYSEX_END: u8 = 0xF7;

// Color velocity values for Launchpad MK2
pub const COLOR_RED: u8 = 5;
pub const COLOR_ORANGE: u8 = 9;
pub const COLOR_YELLOW: u8 = 13;
pub const COLOR_GREEN: u8 = 21;
pub const COLOR_BLUE: u8 = 45;
pub const COLOR_PURPLE: u8 = 49;
pub const COLOR_PINK: u8 = 53;
pub const COLOR_GRAY: u8 = 71;
#[allow(dead_code)]
pub const COLOR_BRIGHT: u8 = 21;  // Playing state
pub const COLOR_OFF: u8 = 0;

/// Launchpad MK2 Programmer mode default note colors (one per pad slot, cycling 8 colors)
/// Matches the frontend COLORS array: Rouge, Orange, Jaune, Vert, Cyan, Bleu, Violet, Rose
pub const PAD_COLORS_BY_INDEX: [u8; 8] = [5, 9, 13, 21, 31, 45, 49, 53];

/// Convert pad_id (0-63, top-left = 0) to Launchpad MK2 MIDI note number.
/// Launchpad MK2 Programmer mode layout:
///   Row 0 (top): notes 81-88
///   Row 1:       notes 71-78
///   ...
///   Row 7 (bot): notes 11-18
pub fn pad_id_to_note(pad_id: usize) -> u8 {
    let row = pad_id / 8;
    let col = pad_id % 8;
    ((8 - row) * 10 + (col + 1)) as u8
}

/// Convert a Launchpad MK2 note number back to pad_id (0-63). Returns None if out of grid.
pub fn note_to_pad_id(note: u8) -> Option<usize> {
    let row = note / 10;  // 1-8
    let col = note % 10;  // 1-8
    if row < 1 || row > 8 || col < 1 || col > 8 {
        return None;
    }
    let pad_row = (8 - row) as usize;
    let pad_col = (col - 1) as usize;
    Some(pad_row * 8 + pad_col)
}

pub fn init_programmer_mode(conn: &mut MidiOutputConnection) {
    // Model 0x0D = Launchpad MK2, command 0x0E 0x01 = Programmer mode
    let msg1: Vec<u8> = [SYSEX_HEADER, &[0x0D, 0x0E, 0x01], &[SYSEX_END]].concat();
    let msg2: Vec<u8> = [SYSEX_HEADER, &[0x0C, 0x0E, 0x01], &[SYSEX_END]].concat();
    let _ = conn.send(&msg1);
    let _ = conn.send(&msg2);
}

pub fn set_led(conn: &mut MidiOutputConnection, note: u8, color: u8) {
    let _ = conn.send(&[0x90, note, color]);
}

pub fn clear_all_leds(conn: &mut MidiOutputConnection) {
    // Turn off all 64 pads (notes 11-18, 21-28, 31-38, 41-48, 51-58, 61-68, 71-78, 81-88)
    for row in 0..8u8 {
        for col in 0..8u8 {
            let note = (row + 1) * 10 + (col + 1);
            set_led(conn, note, COLOR_OFF);
        }
    }
}

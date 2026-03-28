use tauri::State;
use crate::state::{AppState, AudioCommand, FxParam};

#[tauri::command]
pub async fn set_fx_param(
    state: State<'_, AppState>,
    param: String,
    value: f32,
) -> Result<(), String> {
    let fx_param = match param.as_str() {
        "filterFreq"       => FxParam::FilterFreq(value),
        "filterResonance"  => FxParam::FilterResonance(value),
        "delayTime"        => FxParam::DelayTime(value),
        "delayFeedback"    => FxParam::DelayFeedback(value),
        "delayMix"         => FxParam::DelayMix(value),
        "reverbMix"        => FxParam::ReverbMix(value),
        "distortion"       => FxParam::DistortionDrive(value),
        "gateRate"         => FxParam::GateRate(value),
        "flangerDepth"     => FxParam::FlangerDepth(value),
        "flangerRate"      => FxParam::FlangerRate(value),
        "masterVolume"     => FxParam::MasterVolume(value),
        _ => return Err(format!("Unknown FX param: {}", param)),
    };
    let audio = state.audio.lock().map_err(|e| e.to_string())?;
    audio.cmd_tx.try_send(AudioCommand::SetFxParam(fx_param)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_bpm(state: State<'_, AppState>, bpm: f32) -> Result<(), String> {
    let mut bpm_state = state.bpm.lock().map_err(|e| e.to_string())?;
    *bpm_state = bpm;
    drop(bpm_state);
    let audio = state.audio.lock().map_err(|e| e.to_string())?;
    audio.cmd_tx.try_send(AudioCommand::SetBpm(bpm)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_quantize(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let audio = state.audio.lock().map_err(|e| e.to_string())?;
    audio.cmd_tx.try_send(AudioCommand::SetQuantize(enabled)).map_err(|e| e.to_string())
}

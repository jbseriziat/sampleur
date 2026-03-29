use tauri::State;
use tauri_plugin_dialog::DialogExt;
use crate::state::AppState;
use crate::preset::{schema::PresetV2, io};

#[tauri::command]
pub async fn save_preset(
    app: tauri::AppHandle,
    _state: State<'_, AppState>,
    preset_json: String,
    kit_mode: String,
) -> Result<String, String> {
    use crate::preset::schema::KitMode;

    let mut preset: PresetV2 = serde_json::from_str(&preset_json).map_err(|e| e.to_string())?;
    preset.kit_mode = if kit_mode == "portable" { KitMode::Portable } else { KitMode::Lightweight };

    // Build a safe default file name from the kit name
    let safe_name: String = preset.name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect::<String>();
    let safe_name = safe_name.trim_matches('_').to_string();
    let default_filename = format!("{}.sampleur2",
        if safe_name.is_empty() { "preset".to_string() } else { safe_name });

    let file_path = tokio::task::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Sampleur Preset", &["sampleur2"])
            .set_file_name(&default_filename)
            .blocking_save_file()
    }).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Save cancelled".to_string())?;

    let path = file_path.into_path().map_err(|e| e.to_string())?;

    // Ensure .sampleur2 extension
    let path = if path.extension().and_then(|e| e.to_str()) == Some("sampleur2") {
        path
    } else {
        path.with_extension("sampleur2")
    };

    io::save_preset(&preset, &path).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn load_preset(app: tauri::AppHandle) -> Result<String, String> {
    let file_path = tokio::task::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Sampleur Preset", &["sampleur2"])
            .blocking_pick_file()
    }).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Load cancelled".to_string())?;

    let path = file_path.into_path().map_err(|e| e.to_string())?;
    let preset = io::load_preset(&path).map_err(|e| e.to_string())?;

    serde_json::to_string(&preset).map_err(|e| e.to_string())
}

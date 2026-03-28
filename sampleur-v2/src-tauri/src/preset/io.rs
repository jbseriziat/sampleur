use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use super::schema::{KitMode, PresetV2};

pub fn save_preset(preset: &PresetV2, save_path: &Path) -> Result<()> {
    if preset.kit_mode == KitMode::Portable {
        // Copy all samples to a _samples folder next to the preset file
        let preset_dir = save_path.parent().unwrap_or(Path::new("."));
        let samples_dir_name = format!("{}_samples",
            save_path.file_stem().and_then(|s| s.to_str()).unwrap_or("preset"));
        let samples_dir = preset_dir.join(&samples_dir_name);
        std::fs::create_dir_all(&samples_dir)
            .with_context(|| format!("Cannot create samples dir: {}", samples_dir.display()))?;

        let mut preset_copy = preset.clone();
        for pad in preset_copy.pads.iter_mut().flatten() {
            if let Some(ref mut sample) = pad.sample {
                // Copy file to samples dir
                let src = PathBuf::from(&sample.absolute_path_hint);
                let dst = samples_dir.join(&sample.file_name);
                if src.exists() {
                    std::fs::copy(&src, &dst)
                        .with_context(|| format!("Cannot copy sample: {}", src.display()))?;
                }
                // Update paths
                sample.relative_path = format!("{}/{}", samples_dir_name, sample.file_name);
                sample.absolute_path_hint = dst.to_string_lossy().to_string();
            }
        }

        let json = serde_json::to_string_pretty(&preset_copy)?;
        std::fs::write(save_path, json)?;
    } else {
        let json = serde_json::to_string_pretty(preset)?;
        std::fs::write(save_path, json)?;
    }
    Ok(())
}

pub fn load_preset(preset_path: &Path) -> Result<PresetV2> {
    let json = std::fs::read_to_string(preset_path)?;
    let mut preset: PresetV2 = serde_json::from_str(&json)?;

    let preset_dir = preset_path.parent().unwrap_or(Path::new("."));

    // Resolve sample paths
    for pad in preset.pads.iter_mut().flatten() {
        if let Some(ref mut sample) = pad.sample {
            // Try relative path first
            let rel = preset_dir.join(&sample.relative_path);
            if rel.exists() {
                sample.absolute_path_hint = rel.to_string_lossy().to_string();
            } else {
                // Fall back to absolute hint
                let abs = PathBuf::from(&sample.absolute_path_hint);
                if !abs.exists() {
                    // Path not found - UI will show warning
                    log::warn!("Sample not found: {} / {}", sample.relative_path, sample.absolute_path_hint);
                }
            }
        }
    }

    Ok(preset)
}

use std::sync::Arc;
use crate::state::PadMode;

pub struct PadPlayer {
    pub samples: Option<Arc<Vec<f32>>>,
    pub sample_rate: u32,
    pub mode: PadMode,
    pub volume: f32,
    pub detune_cents: f32,
    pub original_bpm: f32,

    // Playback state
    pub pos: f64,                // fractional sample index (stereo pairs)
    playback_ratio: f64,         // how many input samples to advance per output sample pair
    pub is_playing: bool,
    pub duration_secs: f64,
    pub progress: f64,           // 0.0..1.0

    // For loop mode: track toggle state
    loop_pending_stop: bool,
}

impl PadPlayer {
    pub fn new() -> Self {
        Self {
            samples: None,
            sample_rate: 44100,
            mode: PadMode::Oneshot,
            volume: 1.0,
            detune_cents: 0.0,
            original_bpm: 120.0,
            pos: 0.0,
            playback_ratio: 1.0,
            is_playing: false,
            duration_secs: 0.0,
            progress: 0.0,
            loop_pending_stop: false,
        }
    }

    pub fn load(&mut self, samples: Arc<Vec<f32>>, sample_rate: u32, channels: u16) {
        let n_frames = samples.len() / channels as usize;
        self.duration_secs = n_frames as f64 / sample_rate as f64;
        self.samples = Some(samples);
        self.sample_rate = sample_rate;
        self.pos = 0.0;
        self.is_playing = false;
        self.progress = 0.0;
    }

    pub fn remove(&mut self) {
        self.samples = None;
        self.is_playing = false;
        self.pos = 0.0;
        self.progress = 0.0;
    }

    pub fn trigger(&mut self, action: &crate::state::PadAction, global_bpm: f32) {
        use crate::state::PadAction;
        if self.samples.is_none() { return; }
        self.update_playback_ratio(global_bpm);
        match action {
            PadAction::Start => {
                self.pos = 0.0;
                self.is_playing = true;
                self.loop_pending_stop = false;
            }
            PadAction::Stop => {
                self.is_playing = false;
                self.pos = 0.0;
                self.progress = 0.0;
            }
            PadAction::Toggle => {
                if self.is_playing {
                    self.is_playing = false;
                    self.pos = 0.0;
                    self.progress = 0.0;
                } else {
                    self.pos = 0.0;
                    self.is_playing = true;
                }
            }
        }
    }

    pub fn update_playback_ratio(&mut self, global_bpm: f32) {
        let bpm_ratio = global_bpm as f64 / self.original_bpm.max(1.0) as f64;
        let detune_ratio = 2_f64.powf(self.detune_cents as f64 / 1200.0);
        self.playback_ratio = bpm_ratio * detune_ratio;
    }

    /// Render one stereo output sample pair.
    /// Returns the stereo sample (l, r) or None if not playing.
    #[inline(always)]
    pub fn render_sample(&mut self) -> Option<(f32, f32)> {
        if !self.is_playing { return None; }
        let samples = self.samples.as_ref()?;
        let n_stereo_frames = samples.len() / 2;
        if n_stereo_frames == 0 { return None; }

        let frame_idx = self.pos as usize;

        if frame_idx >= n_stereo_frames {
            match self.mode {
                PadMode::Loop => {
                    self.pos = 0.0;
                }
                PadMode::Hold => {
                    self.pos = 0.0;
                }
                PadMode::Oneshot => {
                    self.is_playing = false;
                    self.pos = 0.0;
                    self.progress = 0.0;
                    return None;
                }
            }
        }

        let frame_idx = self.pos as usize;
        let frac = self.pos - frame_idx as f64;

        // Linear interpolation between frames
        let idx0 = frame_idx * 2;
        let idx1 = ((frame_idx + 1) % n_stereo_frames) * 2;

        let l = samples[idx0] + (samples[idx1] - samples[idx0]) * frac as f32;
        let r = samples[idx0 + 1] + (samples[idx1 + 1] - samples[idx0 + 1]) * frac as f32;

        self.pos += self.playback_ratio;

        // Update progress
        self.progress = (frame_idx as f64 / n_stereo_frames as f64).min(1.0);

        Some((l * self.volume, r * self.volume))
    }

    pub fn render_into(&mut self, output: &mut [f32], mix_gain: f32) {
        let n_frames = output.len() / 2;
        for frame in 0..n_frames {
            if let Some((l, r)) = self.render_sample() {
                output[frame * 2] += l * mix_gain;
                output[frame * 2 + 1] += r * mix_gain;
            }
        }
    }
}

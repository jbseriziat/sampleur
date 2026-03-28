pub mod biquad;
pub mod delay;
pub mod distortion;
pub mod flanger;
pub mod gate;
pub mod reverb;

use biquad::BiquadFilter;
use delay::DelayLine;
use distortion::atan_distort;
use flanger::Flanger;
use gate::GateLfo;
use reverb::Reverb;

pub struct FxChain {
    pub distortion_drive: f32,
    pub filter: BiquadFilter,
    pub delay: DelayLine,
    pub reverb: Reverb,
    pub gate: GateLfo,
    pub flanger: Flanger,
    pub delay_mix: f32,
    pub reverb_mix: f32,
    pub master_volume: f32,
    sample_rate: f32,
}

impl FxChain {
    pub fn new(sample_rate: f32) -> Self {
        let mut filter = BiquadFilter::new();
        filter.set_lowpass(20000.0, 0.707, sample_rate);
        Self {
            distortion_drive: 0.0,
            filter,
            delay: DelayLine::new(sample_rate, 5.0),  // 5s max delay
            reverb: Reverb::new(sample_rate),
            gate: GateLfo::new(sample_rate),
            flanger: Flanger::new(sample_rate),
            delay_mix: 0.0,
            reverb_mix: 0.0,
            master_volume: 1.0,
            sample_rate,
        }
    }

    // Process a stereo interleaved buffer in-place
    pub fn process(&mut self, buffer: &mut [f32]) {
        for frame in buffer.chunks_exact_mut(2) {
            let l = frame[0];
            let r = frame[1];

            // Distortion
            let (l, r) = if self.distortion_drive > 0.01 {
                (atan_distort(l, self.distortion_drive), atan_distort(r, self.distortion_drive))
            } else {
                (l, r)
            };

            // Filter
            let (l, r) = (self.filter.process_l(l), self.filter.process_r(r));

            // Delay (mono send, stereo return)
            let delay_out = self.delay.process((l + r) * 0.5);
            let delay_wet = delay_out * self.delay_mix;

            // Reverb (stereo)
            let (rev_l, rev_r) = self.reverb.process(l, r);
            let rev_wet_l = rev_l * self.reverb_mix;
            let rev_wet_r = rev_r * self.reverb_mix;

            // Gate
            let gate_gain = self.gate.next_gain();

            // Flanger (subtle, low depth)
            let (fl_l, fl_r) = self.flanger.process(l, r);

            // Mix
            let out_l = (l + delay_wet + rev_wet_l + fl_l * 0.3) * gate_gain * self.master_volume;
            let out_r = (r + delay_wet + rev_wet_r + fl_r * 0.3) * gate_gain * self.master_volume;

            // Soft clip to prevent digital clipping at the very end
            frame[0] = soft_clip(out_l);
            frame[1] = soft_clip(out_r);
        }
    }

    pub fn set_filter_freq(&mut self, freq: f32) {
        let freq = freq.max(100.0).min(20000.0);
        let q = self.filter.q;
        self.filter.set_lowpass(freq, q, self.sample_rate);
    }

    pub fn set_filter_resonance(&mut self, q: f32) {
        let q = q.max(0.1).min(20.0);
        let freq = self.filter.freq;
        self.filter.set_lowpass(freq, q, self.sample_rate);
    }
}

#[inline(always)]
fn soft_clip(x: f32) -> f32 {
    if x > 1.0 {
        1.0 - (x - 1.0).powi(2).min(1.0)
    } else if x < -1.0 {
        -1.0 + (x + 1.0).powi(2).min(1.0)
    } else {
        x
    }
}

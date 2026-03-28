pub struct GateLfo {
    phase: f32,
    pub rate: f32,  // Hz, 0 = bypass
    sample_rate: f32,
}

impl GateLfo {
    pub fn new(sample_rate: f32) -> Self {
        Self { phase: 0.0, rate: 0.0, sample_rate }
    }

    #[inline(always)]
    pub fn next_gain(&mut self) -> f32 {
        if self.rate < 0.01 { return 1.0; }
        self.phase += self.rate / self.sample_rate;
        if self.phase >= 1.0 { self.phase -= 1.0; }
        if self.phase < 0.5 { 1.0 } else { 0.0 }
    }
}

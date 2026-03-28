pub struct BiquadFilter {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    s1l: f32, s2l: f32,  // left channel state
    s1r: f32, s2r: f32,  // right channel state
    pub freq: f32,
    pub q: f32,
}

impl BiquadFilter {
    pub fn new() -> Self {
        Self {
            b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0,
            s1l: 0.0, s2l: 0.0, s1r: 0.0, s2r: 0.0,
            freq: 20000.0, q: 0.707,
        }
    }

    pub fn set_lowpass(&mut self, freq_hz: f32, q: f32, sample_rate: f32) {
        self.freq = freq_hz;
        self.q = q;
        let w0 = 2.0 * std::f32::consts::PI * freq_hz / sample_rate;
        let alpha = w0.sin() / (2.0 * q.max(0.001));
        let cos_w0 = w0.cos();
        let a0_inv = 1.0 / (1.0 + alpha);
        self.b0 = (1.0 - cos_w0) / 2.0 * a0_inv;
        self.b1 = (1.0 - cos_w0) * a0_inv;
        self.b2 = self.b0;
        self.a1 = -2.0 * cos_w0 * a0_inv;
        self.a2 = (1.0 - alpha) * a0_inv;
    }

    #[inline(always)]
    pub fn process_l(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.s1l;
        self.s1l = self.b1 * x - self.a1 * y + self.s2l;
        self.s2l = self.b2 * x - self.a2 * y;
        y
    }

    #[inline(always)]
    pub fn process_r(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.s1r;
        self.s1r = self.b1 * x - self.a1 * y + self.s2r;
        self.s2r = self.b2 * x - self.a2 * y;
        y
    }
}

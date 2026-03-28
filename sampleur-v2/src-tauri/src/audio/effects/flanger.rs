pub struct Flanger {
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_pos: usize,
    lfo_phase: f32,
    pub rate: f32,   // Hz
    pub depth: f32,  // seconds max delay
    sample_rate: f32,
}

impl Flanger {
    pub fn new(sample_rate: f32) -> Self {
        let max_samples = (sample_rate * 0.1) as usize;
        Self {
            buffer_l: vec![0.0; max_samples],
            buffer_r: vec![0.0; max_samples],
            write_pos: 0,
            lfo_phase: 0.0,
            rate: 0.5,
            depth: 0.005,
            sample_rate,
        }
    }

    pub fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        // Advance LFO
        self.lfo_phase += self.rate / self.sample_rate;
        if self.lfo_phase >= 1.0 { self.lfo_phase -= 1.0; }
        let lfo = (self.lfo_phase * 2.0 * std::f32::consts::PI).sin();

        // Compute delay in samples
        let delay_secs = self.depth * (1.0 + lfo) * 0.5;
        let delay_samples = (delay_secs * self.sample_rate) as usize;
        let len = self.buffer_l.len();
        let delay_samples = delay_samples.min(len - 1).max(1);

        // Write to buffer
        self.buffer_l[self.write_pos] = l;
        self.buffer_r[self.write_pos] = r;

        // Read delayed
        let read_pos = (self.write_pos + len - delay_samples) % len;
        let out_l = self.buffer_l[read_pos];
        let out_r = self.buffer_r[read_pos];

        self.write_pos = (self.write_pos + 1) % len;
        (out_l, out_r)
    }
}

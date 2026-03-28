pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    pub delay_samples: usize,
    pub feedback: f32,
    sample_rate: f32,
}

impl DelayLine {
    pub fn new(sample_rate: f32, max_delay_secs: f32) -> Self {
        let max_samples = (sample_rate * max_delay_secs) as usize;
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            delay_samples: (sample_rate * 0.3) as usize,
            feedback: 0.4,
            sample_rate,
        }
    }

    pub fn set_delay_time(&mut self, secs: f32) {
        let secs = secs.max(0.01).min(4.9);
        self.delay_samples = (secs * self.sample_rate) as usize;
    }

    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        let len = self.buffer.len();
        let read_pos = (self.write_pos + len - self.delay_samples.min(len - 1)) % len;
        let delayed = self.buffer[read_pos];
        self.buffer[self.write_pos] = input + delayed * self.feedback;
        self.write_pos = (self.write_pos + 1) % len;
        delayed
    }
}

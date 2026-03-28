// Freeverb-style reverb algorithm
// 8 parallel feedback comb filters + 4 series allpass filters, stereo
//
// Classic Freeverb delay lengths (in samples at 44100 Hz):
// Combs L: 1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617
// Combs R: offset by +23 samples for stereo spread

const COMB_TUNINGS_L: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const COMB_TUNINGS_R: [usize; 8] = [1139, 1211, 1300, 1379, 1445, 1514, 1580, 1640];
const ALLPASS_TUNINGS_L: [usize; 4] = [556, 441, 341, 225];
const ALLPASS_TUNINGS_R: [usize; 4] = [579, 464, 364, 248];

const FIXED_GAIN: f32 = 0.015;
const SCALE_ROOM: f32 = 0.28;
const OFFSET_ROOM: f32 = 0.7;
const SCALE_DAMPING: f32 = 0.4;
const INITIAL_ROOM: f32 = 0.5;
const INITIAL_DAMPING: f32 = 0.5;
const STEREO_SPREAD: f32 = 23.0;

/// One-pole lowpass feedback comb filter (Freeverb style)
struct CombFilter {
    buffer: Vec<f32>,
    buf_pos: usize,
    feedback: f32,
    filter_store: f32,
    damp1: f32,
    damp2: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            buf_pos: 0,
            feedback: INITIAL_ROOM * SCALE_ROOM + OFFSET_ROOM,
            filter_store: 0.0,
            damp1: INITIAL_DAMPING * SCALE_DAMPING,
            damp2: 1.0 - INITIAL_DAMPING * SCALE_DAMPING,
        }
    }

    fn resize_for_rate(&mut self, base_size: usize, sample_rate: f32) {
        let scaled = ((base_size as f32) * sample_rate / 44100.0) as usize;
        let scaled = scaled.max(1);
        self.buffer = vec![0.0; scaled];
        self.buf_pos = 0;
        self.filter_store = 0.0;
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.buf_pos];
        // One-pole damp filter
        self.filter_store = output * self.damp2 + self.filter_store * self.damp1;
        self.buffer[self.buf_pos] = input + self.filter_store * self.feedback;
        self.buf_pos += 1;
        if self.buf_pos >= self.buffer.len() {
            self.buf_pos = 0;
        }
        output
    }

    fn set_room_size(&mut self, room_size: f32) {
        self.feedback = room_size * SCALE_ROOM + OFFSET_ROOM;
    }

    fn set_damping(&mut self, damping: f32) {
        self.damp1 = damping * SCALE_DAMPING;
        self.damp2 = 1.0 - self.damp1;
    }
}

/// Allpass filter (Schroeder allpass section)
struct AllpassFilter {
    buffer: Vec<f32>,
    buf_pos: usize,
}

impl AllpassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            buf_pos: 0,
        }
    }

    fn resize_for_rate(&mut self, base_size: usize, sample_rate: f32) {
        let scaled = ((base_size as f32) * sample_rate / 44100.0) as usize;
        let scaled = scaled.max(1);
        self.buffer = vec![0.0; scaled];
        self.buf_pos = 0;
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        const G: f32 = 0.5;
        let bufout = self.buffer[self.buf_pos];
        let output = -input + bufout;
        self.buffer[self.buf_pos] = input + bufout * G;
        self.buf_pos += 1;
        if self.buf_pos >= self.buffer.len() {
            self.buf_pos = 0;
        }
        output
    }
}

pub struct Reverb {
    combs_l: [CombFilter; 8],
    combs_r: [CombFilter; 8],
    allpasses_l: [AllpassFilter; 4],
    allpasses_r: [AllpassFilter; 4],
    #[allow(dead_code)]
    sample_rate: f32,
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        // Build comb filters with 44100 base tunings, then scale for actual rate
        let make_comb_l = |i: usize| {
            let mut c = CombFilter::new(1);
            c.resize_for_rate(COMB_TUNINGS_L[i], sample_rate);
            c
        };
        let make_comb_r = |i: usize| {
            let mut c = CombFilter::new(1);
            c.resize_for_rate(COMB_TUNINGS_R[i], sample_rate);
            c
        };
        let make_ap_l = |i: usize| {
            let mut a = AllpassFilter::new(1);
            a.resize_for_rate(ALLPASS_TUNINGS_L[i], sample_rate);
            a
        };
        let make_ap_r = |i: usize| {
            let mut a = AllpassFilter::new(1);
            a.resize_for_rate(ALLPASS_TUNINGS_R[i], sample_rate);
            a
        };

        Self {
            combs_l: [
                make_comb_l(0), make_comb_l(1), make_comb_l(2), make_comb_l(3),
                make_comb_l(4), make_comb_l(5), make_comb_l(6), make_comb_l(7),
            ],
            combs_r: [
                make_comb_r(0), make_comb_r(1), make_comb_r(2), make_comb_r(3),
                make_comb_r(4), make_comb_r(5), make_comb_r(6), make_comb_r(7),
            ],
            allpasses_l: [make_ap_l(0), make_ap_l(1), make_ap_l(2), make_ap_l(3)],
            allpasses_r: [make_ap_r(0), make_ap_r(1), make_ap_r(2), make_ap_r(3)],
            sample_rate,
        }
    }

    /// Set room size (0.0 to 1.0)
    pub fn set_room_size(&mut self, room_size: f32) {
        let rs = room_size.max(0.0).min(1.0);
        for c in self.combs_l.iter_mut() { c.set_room_size(rs); }
        for c in self.combs_r.iter_mut() { c.set_room_size(rs); }
    }

    /// Set damping (0.0 to 1.0)
    pub fn set_damping(&mut self, damping: f32) {
        let d = damping.max(0.0).min(1.0);
        for c in self.combs_l.iter_mut() { c.set_damping(d); }
        for c in self.combs_r.iter_mut() { c.set_damping(d); }
    }

    /// Process one stereo sample pair.
    /// Returns (wet_l, wet_r) — caller mixes wet into dry with reverb_mix.
    #[inline(always)]
    pub fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        // Mono input (equal mix of L+R) fed into all combs
        let input = (l + r) * FIXED_GAIN;

        // Sum all 8 comb filters in parallel
        let mut out_l = 0.0f32;
        let mut out_r = 0.0f32;
        for c in self.combs_l.iter_mut() { out_l += c.process(input); }
        for c in self.combs_r.iter_mut() { out_r += c.process(input); }

        // Pass through 4 allpass filters in series
        for ap in self.allpasses_l.iter_mut() { out_l = ap.process(out_l); }
        for ap in self.allpasses_r.iter_mut() { out_r = ap.process(out_r); }

        (out_l, out_r)
    }
}

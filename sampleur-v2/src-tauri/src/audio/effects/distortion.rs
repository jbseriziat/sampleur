#[inline(always)]
pub fn atan_distort(x: f32, drive: f32) -> f32 {
    if drive < 0.01 { return x; }
    let k = drive;
    let deg = std::f32::consts::PI / 180.0;
    (3.0 + k) * x * 20.0 * deg / (std::f32::consts::PI + k * x.abs())
}

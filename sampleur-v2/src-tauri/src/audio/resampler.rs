use anyhow::Result;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

pub fn resample_to_rate(
    samples: Vec<f32>,
    from_rate: u32,
    to_rate: u32,
    channels: u16,
) -> Result<Vec<f32>> {
    if from_rate == to_rate { return Ok(samples); }

    let channels = channels as usize;
    let ratio = to_rate as f64 / from_rate as f64;

    // De-interleave
    let n_frames = samples.len() / channels;
    let ch_data: Vec<Vec<f32>> = (0..channels)
        .map(|ch| samples.iter().skip(ch).step_by(channels).cloned().collect())
        .collect();

    let params = SincInterpolationParameters {
        sinc_len: 128,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 128,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = SincFixedIn::<f32>::new(
        ratio, 2.0, params, n_frames, channels
    )?;

    let out = resampler.process(&ch_data.iter().map(|v| v.as_slice()).collect::<Vec<_>>(), None)?;

    // Re-interleave
    let out_frames = out[0].len();
    let mut interleaved = Vec::with_capacity(out_frames * channels);
    for frame in 0..out_frames {
        for ch in 0..channels {
            interleaved.push(out[ch][frame]);
        }
    }

    Ok(interleaved)
}

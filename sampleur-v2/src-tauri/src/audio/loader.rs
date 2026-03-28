use anyhow::{Context, Result};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct DecodedAudio {
    pub samples: Vec<f32>,  // interleaved float32
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f64,
}

pub fn decode_audio_file(path: &std::path::Path) -> Result<DecodedAudio> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Cannot open file: {}", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .context("Failed to probe audio format")?;
    let mut format = probed.format;
    let track = format.default_track().context("No audio track found")?;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
    let n_frames = track.codec_params.n_frames;
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .context("Failed to create decoder")?;

    let mut samples: Vec<f32> = if let Some(frames) = n_frames {
        Vec::with_capacity((frames * channels as u64) as usize)
    } else {
        Vec::new()
    };

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(symphonia::core::errors::Error::ResetRequired) => break,
            Err(e) => return Err(e.into()),
        };
        if packet.track_id() != track_id { continue; }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let mut buf: SampleBuffer<f32> =
                    SampleBuffer::new(decoded.capacity() as u64, spec);
                buf.copy_interleaved_ref(decoded);
                samples.extend_from_slice(buf.samples());
            }
            Err(symphonia::core::errors::Error::IoError(_)) => break,
            Err(symphonia::core::errors::Error::DecodeError(e)) => {
                log::warn!("Decode error (skipping packet): {}", e);
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }

    // Convert to stereo if mono
    let (samples, channels) = if channels == 1 {
        let stereo: Vec<f32> = samples.iter().flat_map(|&s| [s, s]).collect();
        (stereo, 2u16)
    } else {
        (samples, channels)
    };

    let duration_secs = if sample_rate > 0 && channels > 0 {
        samples.len() as f64 / (sample_rate as f64 * channels as f64)
    } else {
        0.0
    };

    Ok(DecodedAudio { samples, sample_rate, channels, duration_secs })
}

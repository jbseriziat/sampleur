use std::sync::{mpsc, Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::state::{AudioCommand, FxParam, PadProgress};
use super::{pad::PadPlayer, effects::FxChain};

pub struct AudioEngine {
    _stream: cpal::Stream,
    pub sample_rate: u32,
}

impl AudioEngine {
    pub fn new(
        cmd_rx: mpsc::Receiver<AudioCommand>,
        progress_state: Arc<Mutex<Vec<PadProgress>>>,
    ) -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No audio output device found"))?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;

        log::info!("Audio device: {}", device.name().unwrap_or_default());
        log::info!("Sample rate: {} Hz", sample_rate);
        log::info!("Channels: {}", config.channels());

        let stream = build_stream(device, config, cmd_rx, progress_state, sample_rate)?;
        stream.play()?;

        Ok(Self { _stream: stream, sample_rate })
    }
}

fn build_stream(
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    cmd_rx: mpsc::Receiver<AudioCommand>,
    progress_state: Arc<Mutex<Vec<PadProgress>>>,
    sample_rate: u32,
) -> anyhow::Result<cpal::Stream> {
    use cpal::SampleFormat;

    match config.sample_format() {
        SampleFormat::F32 => build_stream_f32(device, config.into(), cmd_rx, progress_state, sample_rate),
        SampleFormat::I16 => build_stream_generic::<i16>(device, config.into(), cmd_rx, progress_state, sample_rate),
        SampleFormat::U16 => build_stream_generic::<u16>(device, config.into(), cmd_rx, progress_state, sample_rate),
        _ => build_stream_f32(device, config.into(), cmd_rx, progress_state, sample_rate),
    }
}

fn build_stream_f32(
    device: cpal::Device,
    config: cpal::StreamConfig,
    cmd_rx: mpsc::Receiver<AudioCommand>,
    progress_state: Arc<Mutex<Vec<PadProgress>>>,
    sample_rate: u32,
) -> anyhow::Result<cpal::Stream> {
    let sr = sample_rate as f32;
    let mut pads: Vec<PadPlayer> = (0..64).map(|_| PadPlayer::new()).collect();
    let mut fx = FxChain::new(sr);
    let mut global_bpm: f32 = 120.0;
    let mut quantize = false;
    let mut progress_counter = 0u32;
    let progress_update_interval = (sr / 30.0) as u32; // ~30fps

    // Beat-grid tracking for quantize (samples elapsed since engine start, never reset)
    let mut samples_since_beat: u64 = 0;

    // Pending quantised triggers: (pad_id, action, remaining_delay_in_samples)
    let mut pending_triggers: Vec<(usize, crate::state::PadAction, usize)> = Vec::new();

    // Recording sender — set via AudioCommand::StartRecording, cleared on StopRecording.
    let mut rec_tx: Option<mpsc::SyncSender<Vec<f32>>> = None;

    let stream = device.build_output_stream(
        &config,
        move |output: &mut [f32], _info: &cpal::OutputCallbackInfo| {
            let n_out_frames = output.len() / 2; // stereo

            // ── 1. Pre-compute how many samples remain until the next beat boundary ──
            let samples_per_beat = ((sr as f64 * 60.0) / global_bpm as f64) as u64;
            let beat_pos = samples_since_beat % samples_per_beat.max(1);
            let samples_to_next_beat = (samples_per_beat - beat_pos) as usize;

            // ── 2. Drain command queue ─────────────────────────────────────────────
            loop {
                match cmd_rx.try_recv() {
                    Ok(cmd) => handle_command(
                        cmd, &mut pads, &mut fx,
                        &mut global_bpm, &mut quantize,
                        &mut pending_triggers,
                        samples_to_next_beat,
                        &mut rec_tx,
                    ),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
            }

            // ── 3. Clear output buffer ─────────────────────────────────────────────
            for s in output.iter_mut() { *s = 0.0; }

            // ── 4. Fire pending quantised triggers ─────────────────────────────────
            let mut remaining = Vec::new();
            for (pad_id, action, delay) in pending_triggers.drain(..) {
                if delay <= n_out_frames {
                    // Due this buffer — trigger at its start (sample-accurate would
                    // require splitting the buffer; buffer-start is close enough)
                    if pad_id < 64 {
                        pads[pad_id].trigger(&action, global_bpm);
                    }
                } else {
                    remaining.push((pad_id, action, delay - n_out_frames));
                }
            }
            pending_triggers = remaining;

            // ── 5. Mix gain (equal-power headroom) ────────────────────────────────
            let active_count = pads.iter().filter(|p| p.is_playing).count().max(1);
            let mix_gain = 1.0_f32 / (active_count as f32).sqrt();

            // ── 6. Render active pads ─────────────────────────────────────────────
            for pad in pads.iter_mut() {
                if pad.is_playing {
                    pad.render_into(output, mix_gain);
                }
            }

            // ── 7. Effects chain ──────────────────────────────────────────────────
            fx.process(output);

            // ── 8. Recording tap (after FX, before device write) ─────────────────
            if let Some(tx) = &rec_tx {
                // try_send: if channel is full, drop the block (avoid blocking audio thread).
                // If the receiver is gone, clear the sender so we stop trying.
                if tx.try_send(output.to_vec()).is_err() {
                    rec_tx = None;
                }
            }

            // ── 9. Progress report at ~30fps ──────────────────────────────────────
            progress_counter += n_out_frames as u32;
            if progress_counter >= progress_update_interval {
                progress_counter = 0;
                let progresses: Vec<PadProgress> = pads.iter().enumerate().map(|(i, p)| {
                    PadProgress { pad_id: i, progress: p.progress, is_playing: p.is_playing }
                }).collect();
                if let Ok(mut state) = progress_state.try_lock() {
                    *state = progresses;
                }
            }

            // ── 10. Advance beat counter ───────────────────────────────────────────
            samples_since_beat = samples_since_beat.wrapping_add(n_out_frames as u64);
        },
        |err| log::error!("CPAL stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

fn build_stream_generic<T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>>(
    device: cpal::Device,
    config: cpal::StreamConfig,
    cmd_rx: mpsc::Receiver<AudioCommand>,
    progress_state: Arc<Mutex<Vec<PadProgress>>>,
    sample_rate: u32,
) -> anyhow::Result<cpal::Stream> {
    let sr = sample_rate as f32;
    let mut pads: Vec<PadPlayer> = (0..64).map(|_| PadPlayer::new()).collect();
    let mut fx = FxChain::new(sr);
    let mut global_bpm: f32 = 120.0;
    let mut quantize = false;
    let mut progress_counter = 0u32;
    let progress_update_interval = (sr / 30.0) as u32;
    let mut pending_triggers: Vec<(usize, crate::state::PadAction, usize)> = Vec::new();
    let mut samples_since_beat: u64 = 0;
    let mut rec_tx: Option<mpsc::SyncSender<Vec<f32>>> = None;

    let stream = device.build_output_stream(
        &config,
        move |output: &mut [T], _info: &cpal::OutputCallbackInfo| {
            let mut float_buf = vec![0.0f32; output.len()];
            let n_out_frames = float_buf.len() / 2;

            let samples_per_beat = ((sr as f64 * 60.0) / global_bpm as f64) as u64;
            let beat_pos = samples_since_beat % samples_per_beat.max(1);
            let samples_to_next_beat = (samples_per_beat - beat_pos) as usize;

            loop {
                match cmd_rx.try_recv() {
                    Ok(cmd) => handle_command(
                        cmd, &mut pads, &mut fx,
                        &mut global_bpm, &mut quantize,
                        &mut pending_triggers,
                        samples_to_next_beat,
                        &mut rec_tx,
                    ),
                    Err(_) => break,
                }
            }

            // Fire pending quantised triggers
            let mut remaining = Vec::new();
            for (pad_id, action, delay) in pending_triggers.drain(..) {
                if delay <= n_out_frames {
                    if pad_id < 64 { pads[pad_id].trigger(&action, global_bpm); }
                } else {
                    remaining.push((pad_id, action, delay - n_out_frames));
                }
            }
            pending_triggers = remaining;

            let active_count = pads.iter().filter(|p| p.is_playing).count().max(1);
            let mix_gain = 1.0_f32 / (active_count as f32).sqrt();

            for pad in pads.iter_mut() {
                if pad.is_playing { pad.render_into(&mut float_buf, mix_gain); }
            }

            fx.process(&mut float_buf);

            // Recording tap (after FX)
            if let Some(tx) = &rec_tx {
                if tx.try_send(float_buf.clone()).is_err() {
                    rec_tx = None;
                }
            }

            progress_counter += n_out_frames as u32;
            if progress_counter >= progress_update_interval {
                progress_counter = 0;
                let progresses: Vec<PadProgress> = pads.iter().enumerate().map(|(i, p)| {
                    PadProgress { pad_id: i, progress: p.progress, is_playing: p.is_playing }
                }).collect();
                if let Ok(mut state) = progress_state.try_lock() {
                    *state = progresses;
                }
            }

            for (i, sample) in float_buf.iter().enumerate() {
                output[i] = T::from_sample(*sample);
            }

            samples_since_beat = samples_since_beat.wrapping_add(n_out_frames as u64);
        },
        |err| log::error!("CPAL stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

fn handle_command(
    cmd: AudioCommand,
    pads: &mut Vec<PadPlayer>,
    fx: &mut FxChain,
    global_bpm: &mut f32,
    quantize: &mut bool,
    pending_triggers: &mut Vec<(usize, crate::state::PadAction, usize)>,
    samples_to_next_beat: usize,
    rec_tx: &mut Option<mpsc::SyncSender<Vec<f32>>>,
) {
    match cmd {
        AudioCommand::TriggerPad { id, action } => {
            if id < 64 {
                if *quantize && samples_to_next_beat > 0 {
                    // Defer until the next beat boundary
                    pending_triggers.push((id, action, samples_to_next_beat));
                } else {
                    pads[id].trigger(&action, *global_bpm);
                }
            }
        }
        AudioCommand::StopAll => {
            // Also cancel any pending quantised triggers
            pending_triggers.clear();
            for pad in pads.iter_mut() {
                pad.is_playing = false;
                pad.pos = 0.0;
                pad.progress = 0.0;
            }
        }
        AudioCommand::ResetKit => {
            pending_triggers.clear();
            for pad in pads.iter_mut() {
                pad.is_playing = false;
                pad.pos = 0.0;
                pad.progress = 0.0;
                pad.samples = None;
            }
        }
        AudioCommand::LoadSample { id, samples, sample_rate, channels } => {
            if id < 64 {
                pads[id].load(samples, sample_rate, channels);
            }
        }
        AudioCommand::RemoveSample { id } => {
            if id < 64 { pads[id].remove(); }
        }
        AudioCommand::SetPadVolume { id, volume } => {
            if id < 64 { pads[id].volume = volume; }
        }
        AudioCommand::SetPadDetune { id, detune_cents } => {
            if id < 64 {
                pads[id].detune_cents = detune_cents;
                pads[id].update_playback_ratio(*global_bpm);
            }
        }
        AudioCommand::SetPadMode { id, mode } => {
            if id < 64 { pads[id].mode = mode; }
        }
        AudioCommand::SetPadOriginalBpm { id, bpm } => {
            if id < 64 {
                pads[id].original_bpm = bpm;
                pads[id].update_playback_ratio(*global_bpm);
            }
        }
        AudioCommand::SetFxParam(param) => {
            match param {
                FxParam::FilterFreq(v) => fx.set_filter_freq(v),
                FxParam::FilterResonance(v) => fx.set_filter_resonance(v),
                FxParam::DelayTime(v) => fx.delay.set_delay_time(v),
                FxParam::DelayFeedback(v) => fx.delay.feedback = v.max(0.0).min(0.95),
                FxParam::DelayMix(v) => fx.delay_mix = v.max(0.0).min(1.0),
                FxParam::ReverbMix(v) => fx.reverb_mix = v.max(0.0).min(2.0),
                FxParam::DistortionDrive(v) => fx.distortion_drive = v.max(0.0).min(100.0),
                FxParam::GateRate(v) => fx.gate.rate = v.max(0.0).min(12.0),
                FxParam::FlangerDepth(v) => fx.flanger.depth = v.max(0.0).min(0.02),
                FxParam::FlangerRate(v) => fx.flanger.rate = v.max(0.1).min(5.0),
                FxParam::MasterVolume(v) => fx.master_volume = v.max(0.0).min(2.0),
            }
        }
        AudioCommand::SetBpm(bpm) => {
            *global_bpm = bpm.max(20.0).min(300.0);
            for pad in pads.iter_mut() {
                pad.update_playback_ratio(*global_bpm);
            }
        }
        AudioCommand::SetQuantize(q) => { *quantize = q; }
        AudioCommand::StartRecording { tx } => { *rec_tx = Some(tx); }
        AudioCommand::StopRecording => { *rec_tx = None; }
    }
}

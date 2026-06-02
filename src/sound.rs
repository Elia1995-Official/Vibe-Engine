//! Procedural audio engine – no asset files required.
//!
//! All sounds are synthesised at runtime from first principles using simple
//! sine oscillators (chirps) and filtered noise.  Shared primitives are
//! reused across all four games.
//!
//! Usage:
//!   1. Call `SoundEngine::new()` once at startup (returns `None` if no
//!      audio device is available).
//!   2. Games push `SoundEvent` values into `Input::sound_events`.
//!   3. `main.rs` drains the queue each frame and calls `engine.play(event, settings)`.

use std::f32::consts::PI;
use std::collections::HashSet;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

use rodio::buffer::SamplesBuffer;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{OutputStream, OutputStreamHandle, Source};

use crate::app::{SoundEvent, SoundSettings};

const SR: u32 = 44_100;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub struct SoundEngine {
    _stream: OutputStream, // must stay alive to keep the audio thread running
    handle: OutputStreamHandle,
    playback_error_reported: AtomicBool,
}

impl SoundEngine {
    /// Returns `None` if no audio output device is available.
    pub fn new() -> Option<Self> {
        let preferred_host = env::var("VIBE_AUDIO_HOST").ok();
        let preferred_device = env::var("VIBE_AUDIO_DEVICE").ok();
        if let Some(ref host) = preferred_host {
            eprintln!("[audio] Host hint requested: '{host}'");
        }
        if let Some(ref device) = preferred_device {
            eprintln!("[audio] Device hint requested: '{device}'");
        }

        let (stream, handle) = Self::try_best_output_stream(
            preferred_host.as_deref(),
            preferred_device.as_deref(),
        )?;
        Some(Self {
            _stream: stream,
            handle,
            playback_error_reported: AtomicBool::new(false),
        })
    }

    fn try_best_output_stream(
        preferred_host: Option<&str>,
        preferred_device: Option<&str>,
    ) -> Option<(OutputStream, OutputStreamHandle)> {
        let preferred_host = preferred_host.map(|s| s.to_ascii_lowercase());
        let preferred_device = preferred_device.map(|s| s.to_ascii_lowercase());

        let default_host = rodio::cpal::default_host();
        let default_host_id = default_host.id();
        let mut host_ids = vec![default_host_id];
        for host_id in rodio::cpal::available_hosts() {
            if host_id != default_host_id {
                host_ids.push(host_id);
            }
        }

        for host_id in host_ids {
            let host = match rodio::cpal::host_from_id(host_id) {
                Ok(host) => host,
                Err(err) => {
                    eprintln!("[audio] Could not access host {host_id:?}: {err}");
                    continue;
                }
            };

            if let Some(ref hint) = preferred_host {
                if !format!("{host_id:?}").to_ascii_lowercase().contains(hint) {
                    continue;
                }
            }

            let mut candidates = Vec::new();
            let mut seen_names = HashSet::new();

            if let Some(device) = host.default_output_device() {
                let device_name = device
                    .name()
                    .unwrap_or_else(|_| "<unnamed-device>".to_string());
                seen_names.insert(device_name.clone());
                let score = score_device_name(&device_name, preferred_device.as_deref()) + 1000;
                candidates.push((score, device_name, device));
            }

            let devices = match host.output_devices() {
                Ok(devices) => devices,
                Err(err) => {
                    eprintln!("[audio] Could not enumerate output devices for host {host_id:?}: {err}");
                    continue;
                }
            };

            for device in devices {
                let device_name = device
                    .name()
                    .unwrap_or_else(|_| "<unnamed-device>".to_string());
                if !seen_names.insert(device_name.clone()) {
                    continue;
                }
                let score = score_device_name(&device_name, preferred_device.as_deref());
                candidates.push((score, device_name, device));
            }

            candidates.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

            for (score, device_name, device) in candidates {
                match OutputStream::try_from_device(&device) {
                    Ok(stream) => {
                        eprintln!(
                            "[audio] Using output device '{device_name}' on host {host_id:?} (score {score})."
                        );
                        return Some(stream);
                    }
                    Err(err) => {
                        eprintln!(
                            "[audio] Output device '{device_name}' on host {host_id:?} failed: {err}"
                        );
                    }
                }
            }
        }

        eprintln!("[audio] No usable output device found; audio disabled.");
        None
    }

    /// Fire-and-forget playback; the rodio mixer drives the source concurrently.
    pub fn play(&self, event: SoundEvent, settings: &SoundSettings) {
        let volume = settings.master_volume * settings.sfx_volume;
        if volume < 0.001 {
            return;
        }
        let samples = mono_to_stereo(synthesize(event));
        let source = SamplesBuffer::new(2, SR, samples).amplify(volume);
        if let Err(err) = self.handle.play_raw(source.convert_samples()) {
            if !self.playback_error_reported.swap(true, Ordering::Relaxed) {
                eprintln!(
                    "[audio] Playback failed: {err}. Further playback errors will be suppressed."
                );
            }
        }
    }
}

fn score_device_name(name: &str, preferred_device: Option<&str>) -> i32 {
    let lower = name.to_ascii_lowercase();
    let mut score = 0;

    if let Some(hint) = preferred_device {
        if lower.contains(hint) {
            score += 600;
        } else {
            score -= 10;
        }
    }

    if lower.contains("pipewire") {
        score += 250;
    }
    if lower.contains("pulse") {
        score += 220;
    }
    if lower == "default" || lower.starts_with("default") || lower.contains(" default") {
        score += 160;
    }
    if lower.contains("sysdefault") {
        score += 130;
    }
    if lower.contains("usb") || lower.contains("headphone") || lower.contains("speaker") || lower.contains("analog") {
        score += 90;
    }
    if lower.contains("hdmi") || lower.contains("displayport") || lower.contains("digital") {
        score -= 60;
    }
    if lower.contains("jack") {
        score -= 50;
    }

    score
}

// ---------------------------------------------------------------------------
// Synthesis dispatch
// ---------------------------------------------------------------------------

fn synthesize(event: SoundEvent) -> Vec<f32> {
    match event {
        // ── Shared ─────────────────────────────────────────────────────────
        SoundEvent::MenuClick => chirp(800.0, 600.0, 0.04, 0.55),
        SoundEvent::GameOver => chirp(460.0, 55.0, 0.55, 0.70),

        // ── Space Shooter ──────────────────────────────────────────────────
        SoundEvent::SpaceShoot => chirp(700.0, 200.0, 0.07, 0.50),
        SoundEvent::AsteroidHit => noise(0.12, 320.0, 0.55),
        SoundEvent::PickupCollected => triple_beep([440.0, 554.0, 659.0], 0.06),
        SoundEvent::SpacePlayerHurt => chirp(180.0, 55.0, 0.15, 0.65),

        // ── Snake ──────────────────────────────────────────────────────────
        SoundEvent::FoodEaten => chirp(330.0, 440.0, 0.07, 0.50),
        SoundEvent::BadFoodEaten => chirp(200.0, 80.0, 0.12, 0.55),

        // ── FPS Arena ──────────────────────────────────────────────────────
        SoundEvent::FpsShoot => mix(noise(0.06, 600.0, 0.65), chirp(90.0, 30.0, 0.07, 0.40)),
        SoundEvent::EnemyHit => noise(0.10, 280.0, 0.50),
        SoundEvent::EnemyKill => mix(chirp(110.0, 40.0, 0.18, 0.60), noise(0.08, 180.0, 0.35)),
        SoundEvent::FpsPlayerHurt => chirp(75.0, 40.0, 0.18, 0.70),
        SoundEvent::FpsLevelComplete => fanfare([523.0, 659.0, 784.0], 0.12),

        // ── Platformer ─────────────────────────────────────────────────────
        SoundEvent::Jump => chirp(220.0, 660.0, 0.10, 0.45),
        SoundEvent::Land => chirp(150.0, 50.0, 0.08, 0.55),
        SoundEvent::LevelComplete => fanfare([523.0, 659.0, 784.0], 0.12),
        SoundEvent::PlatformerFall => chirp(380.0, 70.0, 0.40, 0.60),
    }
}

// ---------------------------------------------------------------------------
// Primitive generators (reused across games)
// ---------------------------------------------------------------------------

/// Sine oscillator that sweeps linearly from `f0` Hz to `f1` Hz over `dur`
/// seconds, shaped by a smooth attack/decay envelope.
fn chirp(f0: f32, f1: f32, dur: f32, amp: f32) -> Vec<f32> {
    let n = (dur * SR as f32) as usize;
    let mut phase = 0.0f32;
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let freq = f0 + (f1 - f0) * t;
            let s = phase.sin() * env(t) * amp;
            phase = (phase + 2.0 * PI * freq / SR as f32).rem_euclid(2.0 * PI);
            s
        })
        .collect()
}

/// White noise filtered through a one-pole IIR lowpass at `cutoff_hz`.
/// Uses a deterministic xorshift64 RNG so sounds are reproducible.
fn noise(dur: f32, cutoff_hz: f32, amp: f32) -> Vec<f32> {
    let n = (dur * SR as f32) as usize;
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    let alpha = (1.0 / SR as f32) / (rc + 1.0 / SR as f32);
    let mut prev = 0.0f32;
    let mut rng: u64 = 0x9E37_79B9_7F4A_7C15;
    (0..n)
        .map(|i| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let white = (rng as i64) as f32 / i64::MAX as f32;
            let filtered = prev + alpha * (white - prev);
            prev = filtered;
            filtered * env(i as f32 / n as f32) * amp
        })
        .collect()
}

/// Three successive sine tones (pickup / beep feedback).
fn triple_beep(freqs: [f32; 3], note_dur: f32) -> Vec<f32> {
    freqs.iter().flat_map(|&f| chirp(f, f, note_dur, 0.50)).collect()
}

/// Three ascending tones with a tiny pitch rise per note (victory fanfare).
/// Shared by FPS level-complete and platformer level-complete.
fn fanfare(freqs: [f32; 3], note_dur: f32) -> Vec<f32> {
    freqs
        .iter()
        .flat_map(|&f| chirp(f * 0.99, f * 1.02, note_dur, 0.55))
        .collect()
}

/// Overlay buffer `b` on top of buffer `a` (output length = max of both).
fn mix(mut a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    if b.len() > a.len() {
        a.resize(b.len(), 0.0);
    }
    for (s, v) in a.iter_mut().zip(b.iter()) {
        *s += v;
    }
    a
}

fn mono_to_stereo(samples: Vec<f32>) -> Vec<f32> {
    let mut stereo = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        stereo.push(sample);
        stereo.push(sample);
    }
    stereo
}

/// Smooth amplitude envelope: 8 % linear attack, quadratic release.
#[inline]
fn env(t: f32) -> f32 {
    const ATTACK: f32 = 0.08;
    if t < ATTACK {
        t / ATTACK
    } else {
        let r = (t - ATTACK) / (1.0 - ATTACK);
        1.0 - r * r
    }
}

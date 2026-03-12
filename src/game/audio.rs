//! Procedural audio for Starship Space.
//! All sounds are generated in-memory at startup — no audio files needed.

use macroquad::audio::{
    load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound,
};
use std::f32::consts::TAU;

use super::state::Phase;

const SR: u32 = 44_100; // sample rate (Hz)

// ── Public struct ─────────────────────────────────────────────────────────────

pub struct GameAudio {
    bgm_lobby:       Sound,
    bgm_playing:     Sound,
    bgm_market:      Sound,
    sfx_shoot:       Sound,
    sfx_explode:     Sound,
    sfx_player_hit:  Sound,
    sfx_pickup:      Sound,
    sfx_wave_start:  Sound,
    sfx_market_open: Sound,
    sfx_game_over:   Sound,
    last_phase:      Phase,
}

impl GameAudio {
    /// Generate all sounds and load them.  Must be called inside the macroquad
    /// async game loop (before the first frame).
    pub async fn load() -> Self {
        let ga = GameAudio {
            bgm_lobby:       lw(gen_bgm_lobby()).await,
            bgm_playing:     lw(gen_bgm_playing()).await,
            bgm_market:      lw(gen_bgm_market()).await,
            sfx_shoot:       lw(gen_sfx_shoot()).await,
            sfx_explode:     lw(gen_sfx_explode()).await,
            sfx_player_hit:  lw(gen_sfx_player_hit()).await,
            sfx_pickup:      lw(gen_sfx_pickup()).await,
            sfx_wave_start:  lw(gen_sfx_wave_start()).await,
            sfx_market_open: lw(gen_sfx_market_open()).await,
            sfx_game_over:   lw(gen_sfx_game_over()).await,
            // Use GameOver as sentinel so the first update_bgm() call
            // detects a "phase change" and starts the correct BGM track.
            last_phase: Phase::GameOver,
        };
        ga
    }

    /// Call once per frame — handles BGM transitions when the phase changes.
    pub fn update_bgm(&mut self, phase: &Phase) {
        if phase == &self.last_phase { return; }
        stop_sound(&self.bgm_lobby);
        stop_sound(&self.bgm_playing);
        stop_sound(&self.bgm_market);
        match phase {
            Phase::Lobby =>
                play_sound(&self.bgm_lobby,   PlaySoundParams { looped: true, volume: 0.30 }),
            Phase::Playing =>
                play_sound(&self.bgm_playing, PlaySoundParams { looped: true, volume: 0.40 }),
            Phase::Market =>
                play_sound(&self.bgm_market,  PlaySoundParams { looped: true, volume: 0.28 }),
            Phase::GameOver => {} // ไม่มีเพลง
        }
        self.last_phase = phase.clone();
    }

    // ── SFX play methods ──────────────────────────────────────────────────────

    pub fn play_shoot(&self) {
        play_sound(&self.sfx_shoot, PlaySoundParams { looped: false, volume: 0.30 });
    }
    pub fn play_explode(&self) {
        play_sound(&self.sfx_explode, PlaySoundParams { looped: false, volume: 0.55 });
    }
    pub fn play_player_hit(&self) {
        play_sound(&self.sfx_player_hit, PlaySoundParams { looped: false, volume: 0.65 });
    }
    pub fn play_pickup(&self) {
        play_sound(&self.sfx_pickup, PlaySoundParams { looped: false, volume: 0.50 });
    }
    pub fn play_wave_start(&self) {
        play_sound(&self.sfx_wave_start, PlaySoundParams { looped: false, volume: 0.70 });
    }
    pub fn play_market_open(&self) {
        play_sound(&self.sfx_market_open, PlaySoundParams { looped: false, volume: 0.55 });
    }
    pub fn play_game_over(&self) {
        play_sound(&self.sfx_game_over, PlaySoundParams { looped: false, volume: 0.65 });
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Async helper: build WAV bytes → load Sound
async fn lw(bytes: Vec<u8>) -> Sound {
    load_sound_from_bytes(&bytes).await.expect("audio load")
}

/// Number of samples for the given duration in seconds
fn secs(s: f32) -> usize { (SR as f32 * s) as usize }

/// Deterministic pseudo-random noise sample in [-1.0, 1.0]
fn noise(i: usize) -> f32 {
    let h = i.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (h & 0x7FFF_FFFF) as f32 / 0x3FFF_FFFF as f32 - 1.0
}

/// Encode f32 samples → 16-bit mono PCM WAV bytes
fn make_wav(samples: Vec<f32>) -> Vec<u8> {
    let pcm: Vec<i16> = samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * 32_767.0) as i16)
        .collect();
    let data_size = (pcm.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + data_size as usize);
    // RIFF header
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data_size).to_le_bytes());
    w.extend_from_slice(b"WAVE");
    // fmt chunk
    w.extend_from_slice(b"fmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());          // PCM
    w.extend_from_slice(&1u16.to_le_bytes());          // mono
    w.extend_from_slice(&SR.to_le_bytes());
    w.extend_from_slice(&(SR * 2).to_le_bytes());      // byte rate
    w.extend_from_slice(&2u16.to_le_bytes());          // block align
    w.extend_from_slice(&16u16.to_le_bytes());         // bits/sample
    // data chunk
    w.extend_from_slice(b"data");
    w.extend_from_slice(&data_size.to_le_bytes());
    for s in pcm { w.extend_from_slice(&s.to_le_bytes()); }
    w
}

// ── SFX generators ────────────────────────────────────────────────────────────

/// Laser pew: frequency sweep 1200→400 Hz, 0.1 s
fn gen_sfx_shoot() -> Vec<u8> {
    let sr  = SR as f32;
    let dur = 0.10_f32;
    let samples: Vec<f32> = (0..secs(dur)).map(|i| {
        let t    = i as f32 / sr;
        let prog = t / dur;
        let freq = 1_200.0 - prog * 800.0;
        (TAU * freq * t).sin() * (1.0 - prog) * 0.65
    }).collect();
    make_wav(samples)
}

/// Explosion: white noise + low sine, fast exponential decay, 0.5 s
fn gen_sfx_explode() -> Vec<u8> {
    let sr  = SR as f32;
    let dur = 0.50_f32;
    let samples: Vec<f32> = (0..secs(dur)).map(|i| {
        let t   = i as f32 / sr;
        let env = (-t / 0.12).exp();
        let n   = noise(i) * 0.65;
        let low = (TAU * 75.0 * t).sin() * 0.35;
        (n + low) * env * 0.85
    }).collect();
    make_wav(samples)
}

/// Player hit: low thud + noise, frequency drops, 0.28 s
fn gen_sfx_player_hit() -> Vec<u8> {
    let sr  = SR as f32;
    let dur = 0.28_f32;
    let samples: Vec<f32> = (0..secs(dur)).map(|i| {
        let t    = i as f32 / sr;
        let env  = (-t / 0.09).exp();
        let freq = 170.0 + (-t * 25.0).exp() * 130.0; // freq drop
        let n    = noise(i) * 0.30;
        ((TAU * freq * t).sin() * 0.60 + n) * env * 0.80
    }).collect();
    make_wav(samples)
}

/// Ascending chime: E4→A4→C#5→E5, 4 × 0.07 s
fn gen_sfx_pickup() -> Vec<u8> {
    let sr    = SR as f32;
    let nd    = 0.07_f32;
    let notes = [330.0_f32, 440.0, 554.0, 659.0];
    let total = nd * notes.len() as f32;
    let samples: Vec<f32> = (0..secs(total + 0.05)).map(|i| {
        let t = i as f32 / sr;
        if t >= total { return 0.0; }
        let ni   = (t / nd) as usize;
        if ni >= notes.len() { return 0.0; }
        let t_in = t - ni as f32 * nd;
        (TAU * notes[ni] * t).sin() * (1.0 - t_in / nd) * 0.70
    }).collect();
    make_wav(samples)
}

/// Fanfare: C4→E4→G4→C5, 4 × 0.10 s
fn gen_sfx_wave_start() -> Vec<u8> {
    let sr    = SR as f32;
    let nd    = 0.10_f32;
    let notes = [261.6_f32, 329.6, 392.0, 523.3];
    let total = nd * notes.len() as f32;
    let samples: Vec<f32> = (0..secs(total + 0.15)).map(|i| {
        let t = i as f32 / sr;
        if t >= total { return 0.0; }
        let ni   = (t / nd) as usize;
        if ni >= notes.len() { return 0.0; }
        let t_in = t - ni as f32 * nd;
        let env  = (1.0 - t_in / nd * 0.8) * 0.78;
        (TAU * notes[ni] * t).sin() * env
    }).collect();
    make_wav(samples)
}

/// Soft major chord C5+E5+G5, 0.6 s
fn gen_sfx_market_open() -> Vec<u8> {
    let sr  = SR as f32;
    let dur = 0.60_f32;
    let samples: Vec<f32> = (0..secs(dur)).map(|i| {
        let t   = i as f32 / sr;
        let env = (1.0 - t / dur).powf(0.4) * 0.55;
        let w   = (TAU * 523.3 * t).sin() * 0.40
                + (TAU * 659.3 * t).sin() * 0.30
                + (TAU * 784.0 * t).sin() * 0.20;
        w * env
    }).collect();
    make_wav(samples)
}

/// Descending sting: C5→G4→Eb4→C4, 4 × 0.22 s
fn gen_sfx_game_over() -> Vec<u8> {
    let sr    = SR as f32;
    let nd    = 0.22_f32;
    let notes = [523.3_f32, 392.0, 311.1, 261.6];
    let total = nd * notes.len() as f32;
    let samples: Vec<f32> = (0..secs(total + 0.10)).map(|i| {
        let t = i as f32 / sr;
        if t >= total { return 0.0; }
        let ni   = (t / nd) as usize;
        if ni >= notes.len() { return 0.0; }
        let t_in = t - ni as f32 * nd;
        let env  = (1.0 - t_in / nd * 0.75) * 0.72;
        (TAU * notes[ni] * t).sin() * env
    }).collect();
    make_wav(samples)
}

// ── BGM generators ────────────────────────────────────────────────────────────

/// 8-second ambient space drone with slow LFO (Lobby)
fn gen_bgm_lobby() -> Vec<u8> {
    let sr  = SR as f32;
    let dur = 8.0_f32;
    let samples: Vec<f32> = (0..secs(dur)).map(|i| {
        let t   = i as f32 / sr;
        let lfo = (TAU * 0.20 * t).sin() * 0.12 + 0.52;
        let d   = (TAU *  65.4 * t).sin() * 0.45   // C2
                + (TAU * 130.8 * t).sin() * 0.25   // C3
                + (TAU * 196.0 * t).sin() * 0.15   // G3
                + (TAU * 261.6 * t).sin() * 0.08   // C4
                + (TAU * 392.0 * t).sin() * 0.04;  // G4
        d * lfo * 0.35
    }).collect();
    make_wav(samples)
}

/// 2-second loop: pulsing bass (C2 G2 A2 F2) + 8-note melody (Playing)
fn gen_bgm_playing() -> Vec<u8> {
    let sr       = SR as f32;
    let beat     = 0.50_f32;                          // 120 BPM
    let bass_hz  = [65.4_f32, 98.0, 110.0, 87.3];    // C2 G2 A2 F2
    let mel_hz   = [261.6_f32, 329.6, 392.0, 329.6,  // C4 E4 G4 E4
                    440.0,     392.0, 349.2, 329.6];  // A4 G4 F4 E4
    let mel_dur  = beat / 2.0;                        // 0.25 s per melody note
    let dur      = beat * bass_hz.len() as f32;       // 2.0 s total
    let samples: Vec<f32> = (0..secs(dur)).map(|i| {
        let t  = i as f32 / sr;
        // Bass
        let bi   = ((t / beat) as usize).min(bass_hz.len() - 1);
        let t_b  = t - bi as f32 * beat;
        let benv = if t_b < 0.02 { t_b / 0.02 }
                   else { (1.0 - (t_b - 0.02) / (beat - 0.02)).max(0.30) };
        let bass = ((TAU * bass_hz[bi]       * t).sin() * 0.55
                 +  (TAU * bass_hz[bi] * 2.0 * t).sin() * 0.15) * benv * 0.38;
        // Melody
        let mi   = ((t / mel_dur) as usize).min(mel_hz.len() - 1);
        let t_m  = t - mi as f32 * mel_dur;
        let menv = if t_m < 0.01 { t_m / 0.01 }
                   else { (1.0 - (t_m - 0.01) / (mel_dur - 0.01)).max(0.10) };
        let mel  = (TAU * mel_hz[mi] * t).sin() * menv * 0.20;
        (bass + mel).clamp(-0.9, 0.9)
    }).collect();
    make_wav(samples)
}

/// 6-second gentle arpeggio on C major (Market)
fn gen_bgm_market() -> Vec<u8> {
    let sr    = SR as f32;
    let dur   = 6.0_f32;
    let nd    = 0.50_f32;
    let notes = [261.6_f32, 329.6, 392.0, 523.3,
                 392.0,     329.6, 261.6, 329.6,
                 261.6,     329.6, 392.0, 523.3];
    let samples: Vec<f32> = (0..secs(dur)).map(|i| {
        let t    = i as f32 / sr;
        let ni   = ((t / nd) as usize).min(notes.len() - 1);
        let t_in = t - ni as f32 * nd;
        let env  = if t_in < 0.02 { t_in / 0.02 * 0.5 }
                   else { (1.0 - (t_in - 0.02) / (nd - 0.02)).max(0.05) * 0.5 };
        (TAU * notes[ni] * t).sin() * env * 0.45
    }).collect();
    make_wav(samples)
}

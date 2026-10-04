use std::{
    collections::HashMap,
    f64::consts::TAU,
    sync::{Arc, Mutex, MutexGuard},
};

pub type Sample = f32;

pub trait Voice {
    fn sample(&mut self, dt: f64) -> Option<Sample>;
}

pub struct DurationEnvelope<V> {
    pub voice: V,
    pub duration: f64,
}

pub type Duration = f64;
pub type Gain = f64;

pub enum AdsrPhase {
    Silent,
    Attack(Gain, f64),
    Decay(Gain, f64),
    Sustain,
    Release(Gain, f64),
}

pub struct AdsrEnvelope<V> {
    pub voice: V,
    phase: AdsrPhase,
    attack: Duration,
    decay: Duration,
    sustain: Gain,
    release: Duration,
    amplitude: Gain,
}

impl<V> AdsrEnvelope<V> {
    pub fn new(
        voice: V,
        attack: Duration,
        decay: Duration,
        sustain: Gain,
        release: Duration,
    ) -> Self {
        AdsrEnvelope {
            voice,
            phase: AdsrPhase::Silent,
            amplitude: 0.0,
            attack,
            decay,
            sustain,
            release,
        }
    }

    pub fn attack(&mut self) {
        self.phase = AdsrPhase::Attack(self.amplitude, 0.0);
    }

    pub fn release(&mut self) {
        self.phase = AdsrPhase::Release(self.amplitude, 0.0);
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    a * (1.0 - t) + b * t
}

impl<V: Voice> Voice for AdsrEnvelope<V> {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        let sample = self.voice.sample(dt)?;

        match &mut self.phase {
            AdsrPhase::Silent => {
                self.amplitude = 0.0;
            }
            AdsrPhase::Attack(from, time) => {
                let t = *time / self.attack;
                self.amplitude = lerp(*from, 1.0, t);

                if t >= 1.0 {
                    self.phase = AdsrPhase::Decay(self.amplitude, 0.0);
                } else {
                    *time += dt;
                }
            }
            AdsrPhase::Decay(from, time) => {
                let t = *time / self.decay;
                self.amplitude = lerp(*from, self.sustain, t);

                if t >= 1.0 {
                    self.phase = AdsrPhase::Sustain;
                } else {
                    *time += dt;
                }
            }
            AdsrPhase::Sustain => {
                self.amplitude = self.sustain;
            }
            AdsrPhase::Release(from, time) => {
                let t = *time / self.release;
                self.amplitude = lerp(*from, 0.0, t);

                if t >= 1.0 {
                    self.phase = AdsrPhase::Silent;
                } else {
                    *time += dt;
                }
            }
        };

        Some(sample * self.amplitude as f32)
    }
}

impl<V> DurationEnvelope<V> {
    pub fn new(voice: V, duration: f64) -> Self {
        Self { voice, duration }
    }
}

impl<V: Voice> Voice for DurationEnvelope<V> {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        if self.duration <= 0.0 {
            None
        } else {
            let sample = self.voice.sample(dt);
            self.duration -= dt;
            sample
        }
    }
}

pub struct Track {
    voices: Vec<Box<dyn Voice>>,
}

#[derive(Clone, Copy)]
pub enum Waveform {
    Saw,
    Sine,
    Square,
    Triangle,
}

impl Waveform {
    fn sample(&self, phase: f64) -> f64 {
        match self {
            // Since the phase is between 0 and 1, it's already a sawtooth wave.
            Waveform::Saw => phase * 2.0 - 1.0,
            Waveform::Sine => (phase * TAU).sin(),
            Waveform::Square => phase.round() * 2.0 - 1.0,
            Waveform::Triangle => 4.0 * if phase < 0.5 { phase } else { 1.0 - phase } - 1.0,
        }
    }
}

pub struct Oscillator {
    pub waveform: Waveform,
    pub frequency: f64,
    pub transpose: f64,
    pub amplitude: f64,
    phase: f64,
}

impl From<Vec<Box<dyn Voice>>> for Track {
    fn from(voices: Vec<Box<dyn Voice>>) -> Self {
        Track { voices }
    }
}

impl Voice for Track {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        Some(
            self.voices
                .iter_mut()
                .filter_map(|voice| voice.sample(dt))
                .sum(),
        )
    }
}

impl Oscillator {
    pub fn new(waveform: Waveform, frequency: f64, amplitude: f64) -> Self {
        Oscillator {
            waveform,
            frequency,
            transpose: 1.0,
            amplitude,
            phase: 0.0,
        }
    }
}

impl Voice for Oscillator {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        let sample = self.amplitude * self.waveform.sample(self.phase);
        self.phase = (self.phase + dt * self.frequency * self.transpose).fract();
        Some(sample as Sample)
    }
}

pub struct RealTimeVoice<V> {
    inner: Arc<Mutex<V>>,
}

impl<V> Clone for RealTimeVoice<V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<V> RealTimeVoice<V> {
    pub fn new(voice: V) -> Self {
        Self {
            inner: Arc::new(Mutex::new(voice)),
        }
    }

    pub fn get<'a>(&'a self) -> MutexGuard<'a, V> {
        self.inner.lock().unwrap()
    }
}

impl<V: Voice> Voice for RealTimeVoice<V> {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        self.get().sample(dt)
    }
}

impl Voice for Box<dyn Voice> {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        self.as_mut().sample(dt)
    }
}

impl<V: Voice> Voice for Vec<V> {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        let mut sample = 0.0;
        self.retain_mut(|v| {
            if let Some(s) = v.sample(dt) {
                sample += s;
                true
            } else {
                false
            }
        });
        Some(sample)
    }
}

impl<K: Send, V: Voice> Voice for HashMap<K, V> {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        let mut sample = 0.0;
        self.retain(|_, v| {
            if let Some(s) = v.sample(dt) {
                sample += s;
                true
            } else {
                false
            }
        });
        Some(sample)
    }
}

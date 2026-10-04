use std::{
    f64::consts::TAU,
    sync::{Arc, Mutex, MutexGuard},
};

pub type Sample = f32;

pub trait Voice: Send {
    fn sample(&mut self, dt: f32) -> Option<Sample>;
}

pub struct Track {
    voices: Vec<Box<dyn Voice>>,
}

pub struct Oscillator {
    frequency: f64,
    phase: f64,
}

impl From<Vec<Box<dyn Voice>>> for Track {
    fn from(voices: Vec<Box<dyn Voice>>) -> Self {
        Track { voices }
    }
}

impl Voice for Track {
    fn sample(&mut self, dt: f32) -> Option<Sample> {
        Some(
            self.voices
                .iter_mut()
                .filter_map(|voice| voice.sample(dt))
                .sum(),
        )
    }
}

impl Oscillator {
    pub fn new(frequency: f64) -> Self {
        Oscillator {
            frequency,
            phase: 0.0,
        }
    }
}

impl Voice for Oscillator {
    fn sample(&mut self, dt: f32) -> Option<Sample> {
        let sample = (self.phase * TAU).sin();
        self.phase += (dt as f64 * self.frequency).fract();
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
    fn sample(&mut self, dt: f32) -> Option<Sample> {
        self.get().sample(dt)
    }
}

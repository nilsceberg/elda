use crate::synth::{DecayEnvelope, Oscillator, Sample, Voice, Waveform};

pub struct Song {
    voices: Vec<Box<dyn Voice + Send>>,
    metronome: Metronome,
}

pub struct Metronome {
    bpm: f64,
    signature: (i32, i32),
    beat_phase: f64,
    bar_phase: i32,
}

impl Song {
    pub fn empty() -> Self {
        Self {
            metronome: Metronome {
                bpm: 120.0,
                signature: (4, 4),
                beat_phase: 0.0,
                bar_phase: -1,
            },
            voices: Vec::new(),
        }
    }
}

pub enum Tick {
    Heavy,
    Light,
}

impl Metronome {
    pub fn step(&mut self, dt: f64) -> Option<Tick> {
        self.beat_phase += self.bpm / 60.0 * dt;
        if self.beat_phase >= 1.0 {
            self.beat_phase = self.beat_phase.fract();

            self.bar_phase = (self.bar_phase + 1) % self.signature.0;
            if self.bar_phase == 0 {
                Some(Tick::Heavy)
            } else {
                Some(Tick::Light)
            }
        } else {
            None
        }
    }
}

impl Voice for Song {
    fn sample(&mut self, dt: f64) -> Option<Sample> {
        if let Some(tick) = self.metronome.step(dt) {
            let frequency = match tick {
                Tick::Heavy => 880.0,
                Tick::Light => 440.0,
            };
            self.voices.push(Box::new(DecayEnvelope {
                inner: Oscillator::new(Waveform::Sine, frequency, 5.0),
                amplitude: 1.0,
                decay: 0.01,
            }));
        }

        self.voices.sample(dt)
    }

    fn transpose(&mut self, factor: f64) {
        self.voices.transpose(factor);
    }
}

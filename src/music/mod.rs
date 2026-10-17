pub mod song;

/// 12-tone equal temperament based on middle A.
pub const EQUAL_TEMPERAMENT: EqualTemperament = EqualTemperament {
    base: 440.0,
    octave: 12,
};

#[derive(Clone, Copy)]
pub struct Frequency(pub f64);

#[derive(Clone, Copy)]
pub struct Interval(pub f64);

#[derive(Clone, Copy)]
pub enum Pitch {
    Frequency(Frequency),
    Interval(Interval),
}

#[derive(Clone, Copy)]
pub enum Duration {
    Seconds(f32),
    Bars(f32),
}

#[derive(Clone, Copy)]
pub struct Note {
    pub pitch: Pitch,
    pub duration: Duration,
}

impl Pitch {
    pub fn interval(n: impl Into<Interval>) -> Self {
        Self::Interval(n.into())
    }
}

impl<T: Into<f64>> From<T> for Interval {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub trait Tuning {
    fn frequency(&self, pitch: Pitch) -> Frequency;
}

pub struct EqualTemperament {
    pub base: f64,
    pub octave: u32,
}

impl Tuning for EqualTemperament {
    fn frequency(&self, pitch: Pitch) -> Frequency {
        match pitch {
            Pitch::Interval(Interval(i)) => {
                Frequency(self.base * 2f64.powf(i as f64 / self.octave as f64))
            }
            Pitch::Frequency(f) => f,
        }
    }
}

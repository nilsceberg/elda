use std::collections::HashMap;

use alsa::seq::{EvCtrl, EvNote, EventType};
use clap::Parser;
use elda::{
    midi::MidiClient,
    music::{EQUAL_TEMPERAMENT, Pitch, Tuning, song::Song},
    sink::alsa::AlsaSink,
    synth::{AdsrEnvelope, Oscillator, RealTimeVoice, Voice, Waveform},
};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "default")]
    pcm_name: String,

    #[arg(short, long)]
    midi_port: Option<String>,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    let voices = RealTimeVoice::new((HashMap::<u8, _>::new(), Song::empty()));
    let _sink = AlsaSink::new(args.pcm_name, voices.clone());

    let mut client = MidiClient::new("elda");
    if let Some(midi_port) = args.midi_port {
        client.connect(&midi_port);
    }

    let tuning = EQUAL_TEMPERAMENT;
    let mut waveform = Waveform::Saw;
    let mut transpose = 1.0;

    let mut input = client.input();
    while let Ok(event) = input.event_input() {
        log::info!("input: {:?}", event);

        match event.get_type() {
            EventType::Noteon => {
                let data: EvNote = event.get_data().unwrap();
                let mut voices = voices.get();
                let note = voices.0.entry(data.note).or_insert_with(|| {
                    let middle_a = 57;
                    let interval = Pitch::interval(data.note as i32 - middle_a);
                    let frequency = tuning.frequency(interval);
                    log::info!("f: {}", frequency.0);
                    let note = Oscillator::new(waveform, frequency.0, 1.0);
                    AdsrEnvelope::new(note, 0.01, 0.7, 0.2, 0.5)
                });
                note.transpose(transpose);
                note.voice.amplitude = data.velocity as f64 / 64.0;
                note.attack();
            }
            EventType::Noteoff => {
                let data: EvNote = event.get_data().unwrap();
                if let Some(envelope) = voices.get().0.get_mut(&data.note) {
                    envelope.release();
                }
            }
            EventType::Pitchbend => {
                let data: EvCtrl = event.get_data().unwrap();
                transpose = 1.0 + 0.5 * (data.value as f64 / 8192.0);
                voices.get().transpose(transpose);
            }
            EventType::Controller => {
                let data: EvCtrl = event.get_data().unwrap();
                match data.param {
                    14 => {
                        waveform = Waveform::Saw;
                        log::info!("waveform: saw");
                    }
                    16 => {
                        waveform = Waveform::Sine;
                        log::info!("waveform: sine");
                    }
                    18 => {
                        waveform = Waveform::Square;
                        log::info!("waveform: square");
                    }
                    19 => {
                        waveform = Waveform::Triangle;
                        log::info!("waveform: triangle");
                    }
                    _ => {}
                }
                voices.get().0.clear();
            }
            _ => {}
        }
    }
}

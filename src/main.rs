use std::{collections::HashMap, ffi::CString};

use alsa::seq::{Addr, EvNote, EventType, PortCap, PortSubscribe, PortType};
use clap::Parser;
use elda::{
    sink::alsa::AlsaSink,
    synth::{DurationEnvelope, Oscillator, OscillatorFunction, RealTimeVoice},
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

    let voices = RealTimeVoice::new(HashMap::<u8, DurationEnvelope<Oscillator>>::new());
    let _sink = AlsaSink::new(args.pcm_name, voices.clone());

    let seq = alsa::Seq::open(None, None, false).unwrap();
    seq.set_client_name(&CString::new("elda").unwrap()).unwrap();
    let port_id = seq
        .create_simple_port(
            &CString::new("elda").unwrap(),
            PortCap::WRITE | PortCap::SUBS_WRITE | PortCap::READ | PortCap::SUBS_READ,
            PortType::APPLICATION,
        )
        .unwrap();
    let mut input = seq.input();

    if let Some(port) = args.midi_port {
        let (src_id, src_port_id) = port.split_once(":").expect("failed to parse MIDI port");
        let src_id: i32 = src_id.parse().expect("failed to parse MIDI source ID");
        let src_port_id: i32 = src_port_id
            .parse()
            .expect("failed to parse MIDI source port ID");

        let subscribe = PortSubscribe::empty().unwrap();
        subscribe.set_sender(Addr {
            client: src_id,
            port: src_port_id,
        });
        subscribe.set_dest(Addr {
            client: seq.client_id().unwrap(),
            port: port_id,
        });

        seq.subscribe_port(&subscribe).unwrap();
    }

    while let Ok(event) = input.event_input() {
        log::info!("input: {:?}", event);

        match event.get_type() {
            EventType::Noteon => {
                let data: EvNote = event.get_data().unwrap();
                let base_frequency = 440.0;
                let delta = data.note as i32 - 57;
                let frequency = base_frequency * 2f64.powf(delta as f64 / 12.0);
                let note = Oscillator::new(OscillatorFunction::Sine, frequency);
                voices
                    .get()
                    .insert(data.note, DurationEnvelope::new(note, 1.0));
            }
            EventType::Noteoff => {
                let data: EvNote = event.get_data().unwrap();
                log::info!("removed: {:?}", voices.get().remove(&data.note).is_some());
            }
            _ => {}
        }
    }
}

use std::ffi::CString;

use alsa::seq::{Addr, Connect, PortCap, PortInfo, PortSubscribe, PortType};
use clap::Parser;
use elda::{
    sink::alsa::AlsaSink,
    synth::{Oscillator, RealTimeVoice, Track, Voice},
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

    let track = RealTimeVoice::new(Track::from(vec![
        //Box::new(Oscillator::new(440.0)) as Box<dyn Voice>,
        //Box::new(Oscillator::new(880.0)) as Box<dyn Voice>,
        //Box::new(Oscillator::new(1320.0)) as Box<dyn Voice>,
    ]));

    let sink = AlsaSink::new(args.pcm_name, track);

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

    loop {
        log::info!("input: {:?}", input.event_input());
    }
}

use clap::Parser;
use elda::{
    sink::alsa::AlsaSink,
    synth::{Oscillator, RealTimeVoice, Track, Voice},
};

#[derive(Parser)]
struct Args {
    #[arg(default_value = "default")]
    pcm_name: String,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    let track = RealTimeVoice::new(Track::from(vec![
        Box::new(Oscillator::new(440.0)) as Box<dyn Voice>,
        Box::new(Oscillator::new(880.0)) as Box<dyn Voice>,
        Box::new(Oscillator::new(1320.0)) as Box<dyn Voice>,
    ]));

    let sink = AlsaSink::new(args.pcm_name, track);
    sink.wait();
}

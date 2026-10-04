use clap::Parser;

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

    let device = alsa::PCM::new(&args.pcm_name, alsa::Direction::Playback, false)
        .expect("failed to open PCM device");

    let params = alsa::pcm::HwParams::any(&device).expect("failed to initialize hw params");
    params.set_access(alsa::pcm::Access::RWInterleaved).unwrap();
    params.set_format(alsa::pcm::Format::S16LE).unwrap();
    let exact_rate = params
        .set_rate_near(44_100, alsa::ValueOr::Nearest)
        .unwrap();
    log::info!("set sampling rate to {} Hz", exact_rate);

    let period_size = 8192;
    let num_periods = 2;
    let num_channels = 2;
    let frame_size = num_channels * 2;
    let frames_per_period = period_size / frame_size;

    params.set_channels(num_channels).unwrap();
    params
        .set_periods(num_periods, alsa::ValueOr::Nearest)
        .unwrap();

    params
        .set_buffer_size((num_periods * frames_per_period) as i64)
        .unwrap();

    device
        .hw_params(&params)
        .expect("failed to configure pcm device");

    let io = device.io_i16().unwrap();
    let mut data = Vec::<i16>::with_capacity(frames_per_period as usize);
    //let mut t = 0f32;
    let mut i = 0u64;
    loop {
        data.clear();
        let frequency = 440f64;
        for _ in 0..frames_per_period {
            let t = i as f64 / exact_rate as f64;
            let sample = 0.0//(t * frequency * std::f64::consts::TAU).sin() * 8192.0
                + (50.0 * t * std::f64::consts::TAU).sin()
                    * 24000.0
                    * ((t * 20.0).sin() * 0.5 + 0.5);
            let panning = t.sin() * 0.5 + 0.5;
            data.push((sample * panning) as i16);
            data.push((sample * (1.0 - panning)) as i16);
            //t += 1.0 / exact_rate as f32;
            i += 1;
        }

        io.writei(&data).unwrap();
    }
    //device.drain().unwrap();
}

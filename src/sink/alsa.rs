use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use crate::synth::Voice;

pub struct AlsaSink {
    handle: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl AlsaSink {
    pub fn new(device_name: String, voice: impl Voice + 'static) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let handle = Some({
            let stop = stop.clone();
            thread::spawn(move || run(device_name, voice, stop.clone()))
        });
        Self { handle, stop }
    }

    pub fn wait(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

impl Drop for AlsaSink {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

fn run(device_name: String, mut voice: impl Voice, stop: Arc<AtomicBool>) {
    let device = alsa::PCM::new(&device_name, alsa::Direction::Playback, false)
        .expect("failed to open PCM device");

    let params = alsa::pcm::HwParams::any(&device).expect("failed to initialize hw params");
    params.set_access(alsa::pcm::Access::RWInterleaved).unwrap();
    params.set_format(alsa::pcm::Format::S16LE).unwrap();
    let exact_rate = params
        .set_rate_near(44_100, alsa::ValueOr::Nearest)
        .unwrap();
    log::info!("set sampling rate to {} Hz", exact_rate);

    let period_size = 256;
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
    let dt = 1.0 / exact_rate as f64;
    let gain = i16::MAX as f32 / 32.0;
    while !stop.load(Ordering::Relaxed) {
        data.clear();
        for _ in 0..frames_per_period {
            let sample = (voice.sample(dt).unwrap_or(0.0) * gain).round();
            data.push(sample as i16);
            data.push(sample as i16);
        }

        if let Err(e) = io.writei(&data) {
            log::warn!("write error: {:?}", e);
            device.prepare().expect("failed to recover from error");
        }
    }

    log::info!("stopping pcm device...");
    device.drain().unwrap();
}

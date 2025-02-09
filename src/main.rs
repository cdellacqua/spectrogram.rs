use std::{
	sync::mpsc::{sync_channel, TryRecvError},
	thread::spawn,
};

use audio::input::InputStreamPollerBuilder;
use audio::{
	analysis::fft::{self},
	AudioStreamSamplingState,
};
use macroquad::{input, prelude::*};
use spectrogram::{
	config::{FFT_BINS, HISTORY_SIZE, INPUT_DEVICE_NAME, SAMPLES_PER_WINDOW, SAMPLE_RATE},
	spectrogram_surface::SpectrogramSurface,
};

#[macroquad::main("spectrogram")]
async fn main() {
	let (fft_tx, fft_rx) = sync_channel(0);

	spawn({
		move || {
			let stream_poller = InputStreamPollerBuilder::<SAMPLE_RATE, 1>::new(
				SAMPLES_PER_WINDOW.into(),
				INPUT_DEVICE_NAME.map(str::to_owned),
			)
			.build()
			.unwrap();

			let mut fft = fft::StftAnalyzer::<SAMPLE_RATE, SAMPLES_PER_WINDOW>::default();
			loop {
				assert!(matches!(
					stream_poller.state(),
					AudioStreamSamplingState::Sampling
				));
				let _ = fft_tx.send(fft.analyze_bins(stream_poller.snapshot().as_mono()).clone());
			}
		}
	});

	let mut spectrogram_surface = SpectrogramSurface::new(HISTORY_SIZE, FFT_BINS);

	loop {
		if !input::is_key_down(KeyCode::Space) {
			match fft_rx.try_recv() {
				Err(TryRecvError::Disconnected) => panic!("broken fft channel"),
				Err(TryRecvError::Empty) => (),
				Ok(fft) => {
					spectrogram_surface.update(&fft);
				}
			}
		}
		spectrogram_surface.draw(screen_width(), screen_height());

		next_frame().await;
	}
}

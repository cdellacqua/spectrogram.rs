#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]

use core::f32;
use std::{
	sync::mpsc::{sync_channel, TryRecvError},
	thread::spawn,
};

use audio::AudioStreamSamplingState;
use audio::{analysis::dft::StftAnalyzer, input::InputStreamPollerBuilder};
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

			let mut fft = StftAnalyzer::<SAMPLE_RATE, SAMPLES_PER_WINDOW>::default();
			loop {
				assert!(matches!(
					stream_poller.state(),
					AudioStreamSamplingState::Sampling
				));
				let _ = fft_tx.send(fft.analyze(stream_poller.snapshot().as_mono()).clone());
			}
		}
	});

	let mut spectrogram_surface = SpectrogramSurface::new(HISTORY_SIZE, FFT_BINS);
	let mut max = None;

	let mut show_tutorial = true;
	let mut show_max = true;
	let mut pause = false;
	loop {
		if input::is_key_pressed(KeyCode::F1) || input::is_key_pressed(KeyCode::Escape) {
			show_tutorial = !show_tutorial;
		}
		if input::is_key_pressed(KeyCode::F) {
			show_max = !show_max;
		}
		if input::is_key_pressed(KeyCode::Space) {
			pause = !pause;
		}
		if input::is_key_pressed(KeyCode::Q) {
			break;
		}

		if !pause {
			match fft_rx.try_recv() {
				Err(TryRecvError::Disconnected) => panic!("broken fft channel"),
				Err(TryRecvError::Empty) => (),
				Ok(fft) => {
					max = fft
						.iter()
						.max_by(|a, b| a.power().total_cmp(&b.power()))
						.copied();
					spectrogram_surface.update(&fft);
				}
			}
		}
		spectrogram_surface.draw(screen_width(), screen_height());

		if show_max {
			if let Some(max) = max {
				draw_text(
					&format!("Max freq: {}, power: {}", max.frequency(), max.power()),
					16.,
					32.,
					24.,
					WHITE,
				);
			}
		}

		if show_tutorial {
			draw_tutorial();
		}

		next_frame().await;
	}
}

fn draw_tutorial() {
	let font_size = 32;
	let help_texts = [
		"F1/Esc - Toggle tutorial",
		"Space - Toggle sampling",
		"F - Toggle max frequency",
		"Q - Quit",
	]
	.iter()
	.map(|text| (text, measure_text(text, None, font_size, 1.)))
	.collect::<Vec<_>>();

	let max_width = help_texts
		.iter()
		.map(|(_, dim)| dim.width)
		.max_by(f32::total_cmp)
		.unwrap();

	let min_x = (screen_width() - max_width) / 2.;
	let line_height = f32::from(font_size) * 1.5;
	let margin_top = screen_height() / 8.;
	let x_padding = 20.;
	let y_padding = 10.;

	// backdrop
	draw_rectangle(
		0.,
		0.,
		screen_width(),
		screen_height(),
		Color::from_rgba(0, 0, 0, 128),
	);
	// dialog box
	draw_rectangle(
		min_x - x_padding,
		margin_top,
		max_width + x_padding * 2.,
		line_height * help_texts.len() as f32 + y_padding * 2.,
		Color::from_rgba(0, 0, 0, 200),
	);

	// text lines
	for (i, (help, dim)) in help_texts.iter().enumerate() {
		let x = (screen_width() - dim.width) / 2.;
		let y = margin_top + dim.height - dim.offset_y
			+ line_height / 2.
			+ y_padding
			+ i as f32 * line_height;
		draw_text(help, x, y, f32::from(font_size), WHITE);
	}
}

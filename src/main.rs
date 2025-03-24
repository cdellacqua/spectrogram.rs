#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]

use core::f32;
use std::{
	process::exit,
	sync::{Arc, RwLock},
};

use audio::{
	analysis::{dft::StftAnalyzer, frequency_to_bin_idx},
	input::InputStreamBuilder,
};
use buffer_hopper::BufferHopper;
use macroquad::{input, prelude::*};
use spectrogram::{
	config::{HISTORY_SIZE, INPUT_DEVICE_NAME, MAX_FREQUENCY, SAMPLES_PER_WINDOW, SAMPLE_RATE},
	dft_surface::DftSurface,
	spectrogram_surface::SpectrogramSurface,
};

#[macroquad::main("spectrogram")]
async fn main() {
	let mut buffer = BufferHopper::new(SAMPLES_PER_WINDOW);
	let mut analyzer = StftAnalyzer::<SAMPLE_RATE, SAMPLES_PER_WINDOW>::default();
	let spectrogram_surface = Arc::new(RwLock::new(SpectrogramSurface::new(
		HISTORY_SIZE,
		frequency_to_bin_idx(SAMPLE_RATE, SAMPLES_PER_WINDOW, MAX_FREQUENCY) + 1,
	)));
	let dft_surface = Arc::new(RwLock::new(DftSurface::new(
		frequency_to_bin_idx(SAMPLE_RATE, SAMPLES_PER_WINDOW, MAX_FREQUENCY) + 1,
	)));
	let pause = Arc::new(RwLock::new(false));
	let max = Arc::new(RwLock::new(None));

	let _stream_poller = InputStreamBuilder::<SAMPLE_RATE, 1>::new(
		INPUT_DEVICE_NAME.map(str::to_owned),
		Box::new({
			let spectrogram_surface = spectrogram_surface.clone();
			let dft_surface = dft_surface.clone();
			let pause = pause.clone();
			let max = max.clone();
			move |chunk| {
				let (spectrogram_surface, dft_surface, pause, max, analyzer) = (
					&spectrogram_surface,
					&dft_surface,
					&pause,
					&max,
					&mut analyzer,
				);
				buffer.feed(chunk.as_mono(), move |window, _| {
					if !*pause.read().unwrap() {
						let fft = analyzer.analyze(window);
						*max.write().unwrap() = fft
							.iter()
							.max_by(|a, b| a.power().total_cmp(&b.power()))
							.copied();
						spectrogram_surface.write().unwrap().update(fft);
						dft_surface.write().unwrap().update(fft);
					}
				});
			}
		}),
		Some(Box::new(|err| {
			println!("input stream stopped, reason {err}");
			exit(1);
		})),
	)
	.build()
	.unwrap();

	let mut show_tutorial = true;
	let mut show_max = true;
	let mut show_instantaneous = false;
	loop {
		if input::is_key_pressed(KeyCode::F1) || input::is_key_pressed(KeyCode::Escape) {
			show_tutorial = !show_tutorial;
		}
		if input::is_key_pressed(KeyCode::F) {
			show_max = !show_max;
		}
		if input::is_key_pressed(KeyCode::V) {
			show_instantaneous = !show_instantaneous;
		}
		if input::is_key_pressed(KeyCode::Space) {
			let is_paused = *pause.read().unwrap();
			*pause.write().unwrap() = !is_paused;
		}
		if input::is_key_pressed(KeyCode::Q) {
			break;
		}

		if show_instantaneous {
			dft_surface
				.read()
				.unwrap()
				.draw(screen_width(), screen_height());
		} else {
			spectrogram_surface
				.read()
				.unwrap()
				.draw(screen_width(), screen_height());
		}

		if show_max {
			if let Some(max) = *max.read().unwrap() {
				draw_multiline_text(
					&format!("Max freq: {}\nPower: {}", max.frequency(), max.power()),
					16.,
					32.,
					24.,
					Some(1.2),
					YELLOW,
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
		"V - Switch view mode",
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

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]

use core::f32;
use std::{
	process::exit,
	sync::{Arc, RwLock},
};

use audio::{
	analysis::{dft::StftAnalyzer, windowing_fns::HannWindow, DftCtx},
	input::InputStream,
	SamplingCtx,
};
use buffer_hopper::BufferHopper;
use macroquad::{input, prelude::*};
use spectrogram::{
	config::{
		ANALYZE_FREQUENCY, HISTORY_SIZE, INPUT_DEVICE_NAME, MAX_FREQUENCY, SAMPLES_PER_WINDOW,
		SAMPLE_RATE,
	},
	dft_surface::DftSurface,
	spectrogram_surface::SpectrogramSurface,
};

#[macroquad::main("spectrogram")]
async fn main() {
	let mut buffer = BufferHopper::new(SAMPLES_PER_WINDOW);
	let dft_ctx = DftCtx::new(SAMPLE_RATE, SAMPLES_PER_WINDOW);
	let sampling_ctx = SamplingCtx::new(SAMPLE_RATE, 1);
	let mut analyzer = StftAnalyzer::new(dft_ctx, &HannWindow);
	let spectrogram_surface = Arc::new(RwLock::new(SpectrogramSurface::new(
		HISTORY_SIZE,
		dft_ctx.frequency_to_bin(MAX_FREQUENCY) + 1,
	)));
	let dft_surface = Arc::new(RwLock::new(DftSurface::new(
		dft_ctx.frequency_to_bin(MAX_FREQUENCY) + 1,
	)));
	let pause = Arc::new(RwLock::new(false));
	let target = Arc::new(RwLock::new(None));

	let bin_idx =
		ANALYZE_FREQUENCY.map(|analyze_frequency| dft_ctx.frequency_to_bin(analyze_frequency));

	let _stream_poller = InputStream::new(
		sampling_ctx,
		INPUT_DEVICE_NAME,
		Box::new({
			let spectrogram_surface = spectrogram_surface.clone();
			let dft_surface = dft_surface.clone();
			let pause = pause.clone();
			let target = target.clone();
			move |chunk| {
				let (spectrogram_surface, dft_surface, pause, target, analyzer) = (
					&spectrogram_surface,
					&dft_surface,
					&pause,
					&target,
					&mut analyzer,
				);
				buffer.feed(chunk.to_mono(), move |window, _| {
					if !*pause.read().unwrap() {
						let fft = analyzer.analyze(window);
						if let Some(bin_idx) = bin_idx {
							*target.write().unwrap() = Some(fft[bin_idx]);
						} else {
							*target.write().unwrap() = fft
								.iter()
								.skip(1) // exclude DC components (0Hz)
								.max_by(|a, b| a.power().total_cmp(&b.power()))
								.copied();
						}
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
	.unwrap();

	let mut show_tutorial = true;
	let mut show_target = true;
	let mut show_instantaneous = false;
	loop {
		if input::is_key_pressed(KeyCode::F1) || input::is_key_pressed(KeyCode::Escape) {
			show_tutorial = !show_tutorial;
		}
		if input::is_key_pressed(KeyCode::F) {
			show_target = !show_target;
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

		if show_target {
			if let Some(target) = *target.read().unwrap() {
				draw_multiline_text(
					&format!(
						"{} freq: {}\ndB: {}\nPhase: {}{}\n       {}{}deg",
						if ANALYZE_FREQUENCY.is_some() {
							"Target"
						} else {
							"Max"
						},
						dft_ctx.bin_to_frequency(target.bin()),
						target.dB(),
						if target.phase().signum() > 0. {
							'+'
						} else {
							'-'
						},
						target.phase().abs(),
						if target.phase().signum() > 0. {
							'+'
						} else {
							'-'
						},
						target.phase().abs().to_degrees(),
					),
					16.,
					32.,
					24.,
					Some(1.2),
					YELLOW,
				);
				{
					let r = 15.;
					let cx = 30.;
					let cy = 120.;
					draw_circle(cx, cy, r, RED);
					draw_line(
						cx,
						cy,
						cx + r * target.phase().cos(),
						cy + r * target.phase().sin(),
						1.,
						WHITE,
					);
				}
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
		"F - Toggle max/target data overlay",
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

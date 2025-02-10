#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]

use audio::analysis::fft::FftBinPoint;
use macroquad::{
	color::WHITE,
	miniquad::window::screen_size,
	prelude::{
		gl_use_material, load_material, Material, MaterialParams, PipelineParams, ShaderSource,
		UniformDesc, UniformType,
	},
	shapes::draw_rectangle,
	texture::{FilterMode, Texture2D},
};

use crate::config::{MAX_MAGNITUDE, SAMPLES_PER_WINDOW, SAMPLE_RATE};

pub struct SpectrogramSurface {
	material: Material,
	spectrogram_as_texture: Vec<u8>,
	history_size: usize,
	fft_real_size: usize,
}

impl SpectrogramSurface {
	/// # Panics
	/// - if the macroquad material associated with the surface can't be instantiated
	#[must_use]
	pub fn new(history_size: usize, fft_real_size: usize) -> Self {
		let material = load_material(
			ShaderSource::Glsl {
				vertex: VERTEX_SHADER,
				fragment: FRAGMENT_SHADER,
			},
			MaterialParams {
				pipeline_params: PipelineParams::default(),
				textures: vec!["spectrogram".to_string()],
				uniforms: vec![
					UniformDesc::new("screen_size", UniformType::Float2),
					UniformDesc::new("max_magnitude", UniformType::Float1),
				],
			},
		)
		.unwrap();
		material.set_uniform("max_magnitude", MAX_MAGNITUDE);
		Self {
			history_size,
			fft_real_size,
			material,
			spectrogram_as_texture: vec![0u8; COLOR_CHANNELS * history_size * fft_real_size],
		}
	}

	pub fn update(&mut self, fft: &[FftBinPoint<SAMPLE_RATE, SAMPLES_PER_WINDOW>]) {
		let spectrogram_len = self.spectrogram_as_texture.len();
		self.spectrogram_as_texture
			.copy_within(self.fft_real_size * COLOR_CHANNELS..spectrogram_len, 0);
		let base_idx = spectrogram_len - self.fft_real_size * COLOR_CHANNELS;
		for (i, point) in fft.iter().take(self.fft_real_size).enumerate() {
			self.spectrogram_as_texture
				[base_idx + i * COLOR_CHANNELS..base_idx + (i + 1) * COLOR_CHANNELS]
				.copy_from_slice(&point.magnitude.to_be_bytes());
		}
	}

	pub fn draw(&self, width: f32, height: f32) {
		self.material.set_texture("spectrogram", {
			let tex = Texture2D::from_rgba8(
				self.fft_real_size as u16,
				self.history_size as u16,
				&self.spectrogram_as_texture,
			);
			tex.set_filter(FilterMode::Nearest);
			tex
		});
		self.material.set_uniform("screen_size", screen_size());

		gl_use_material(&self.material);
		draw_rectangle(0., 0., width, height, WHITE);
	}
}

const COLOR_CHANNELS: usize = 4;

const VERTEX_SHADER: &str = include_str!("./vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("./fragment.glsl");

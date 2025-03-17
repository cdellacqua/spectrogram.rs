#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]

use audio::analysis::DiscreteHarmonic;
use macroquad::{
	color::WHITE,
	prelude::{
		gl_use_default_material, gl_use_material, load_material, Material, MaterialParams,
		PipelineParams, ShaderSource, UniformDesc, UniformType,
	},
	shapes::draw_rectangle,
	texture::{FilterMode, Texture2D},
};

use crate::config::{MAX_POWER, SAMPLES_PER_WINDOW, SAMPLE_RATE};

pub struct DftSurface {
	material: Material,
	dft_as_texture: Vec<u8>,
	fft_real_size: usize,
}

impl DftSurface {
	/// # Panics
	/// - if the macroquad material associated with the surface can't be instantiated
	#[must_use]
	pub fn new(fft_real_size: usize) -> Self {
		let material = load_material(
			ShaderSource::Glsl {
				vertex: VERTEX_SHADER,
				fragment: FRAGMENT_SHADER,
			},
			MaterialParams {
				pipeline_params: PipelineParams::default(),
				textures: vec!["dft".to_string()],
				uniforms: vec![
					UniformDesc::new("surface_size", UniformType::Float2),
					UniformDesc::new("max_power", UniformType::Float1),
				],
			},
		)
		.unwrap();
		material.set_uniform("max_power", MAX_POWER);
		Self {
			fft_real_size,
			material,
			dft_as_texture: vec![0u8; COLOR_CHANNELS * fft_real_size],
		}
	}

	pub fn update(&mut self, fft: &[DiscreteHarmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>]) {
		for (i, point) in fft.iter().take(self.fft_real_size).enumerate() {
			self.dft_as_texture[i * COLOR_CHANNELS..(i + 1) * COLOR_CHANNELS]
				.copy_from_slice(&point.power().to_be_bytes());
		}
	}

	pub fn draw(&self, width: f32, height: f32) {
		self.material.set_texture("dft", {
			let tex = Texture2D::from_rgba8(self.fft_real_size as u16, 1, &self.dft_as_texture);
			tex.set_filter(FilterMode::Nearest);
			tex
		});
		self.material.set_uniform("surface_size", (width, height));

		gl_use_material(&self.material);
		draw_rectangle(0., 0., width, height, WHITE);
		gl_use_default_material();
	}
}

const COLOR_CHANNELS: usize = 4;

const VERTEX_SHADER: &str = include_str!("./vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("./fragment.glsl");

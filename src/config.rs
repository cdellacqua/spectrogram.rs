use audio::{analysis::fft::frequency_to_index, NOfSamples};

pub const SAMPLE_RATE: usize = 44100;
pub const SAMPLES_PER_WINDOW: usize = 2048;
pub const MAX_FREQUENCY: usize =
	frequency_to_index(15000., NOfSamples::<SAMPLE_RATE>::new(SAMPLES_PER_WINDOW));
pub const FFT_BINS: usize = MAX_FREQUENCY + 1;
pub const HISTORY_SIZE: usize = 512;
pub const MAX_MAGNITUDE: f32 = 0.1;
pub const INPUT_DEVICE_NAME: Option<&str> = None;
use audio::analysis::frequency_to_bin_idx;

pub const SAMPLE_RATE: usize = 44100;
pub const SAMPLES_PER_WINDOW: usize = 2048;
pub const MAX_FREQUENCY: usize = frequency_to_bin_idx(SAMPLE_RATE, SAMPLES_PER_WINDOW, 15000.);
pub const FFT_BINS: usize = MAX_FREQUENCY + 1;
pub const HISTORY_SIZE: usize = 512;
pub const MAX_POWER: f32 = 0.0025;
pub const INPUT_DEVICE_NAME: Option<&str> = None;

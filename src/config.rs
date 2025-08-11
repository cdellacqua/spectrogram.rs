pub const SAMPLE_RATE: audio::SampleRate = audio::SampleRate(44_100);
pub const SAMPLES_PER_WINDOW: usize = 2048;
pub const MAX_FREQUENCY: f32 = 15000.;
pub const HISTORY_SIZE: usize = 512;
pub const MAX_DB: f32 = -20.;
pub const MIN_DB: f32 = -50.;
pub const INPUT_DEVICE_NAME: Option<&str> = None;
pub const ANALYZE_FREQUENCY: Option<f32> = None;

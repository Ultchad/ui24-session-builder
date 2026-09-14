use std::time::{Duration, Instant};
use ui24_audio_processing::FlacEncoder;

const ITERATIONS: usize = 10;
const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u16 = 2;
const BITS_PER_SAMPLE: u16 = 16;
const FRAME_COUNT: usize = 48_000;

fn main() {
    let samples = stereo_samples();
    let encoder = FlacEncoder;
    let start = Instant::now();
    let mut output_size = 0;

    for _ in 0..ITERATIONS {
        let output = encoder
            .encode_pcm(&samples, SAMPLE_RATE, CHANNELS, BITS_PER_SAMPLE)
            .expect("benchmark input should be valid");
        output_size = output.len();
    }

    let elapsed = start.elapsed();
    println!(
        "encoded {FRAME_COUNT} stereo frames x {ITERATIONS} in {}; last output: {output_size} bytes",
        format_duration(elapsed)
    );
}

fn stereo_samples() -> Vec<i32> {
    (0..FRAME_COUNT)
        .flat_map(|frame| {
            let sample = ((frame % 256) as i32) - 128;
            [sample, -sample]
        })
        .collect()
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{}.{:03}s", duration.as_secs(), duration.subsec_millis())
    } else {
        format!("{}ms", duration.as_millis())
    }
}

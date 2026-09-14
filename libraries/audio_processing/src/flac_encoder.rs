use flacenc::bitsink::ByteSink;
use flacenc::component::BitRepr;
use flacenc::config::Encoder;
use flacenc::encode_with_fixed_block_size;
use flacenc::error::Verify;
use flacenc::source::MemSource;

const DEFAULT_BLOCK_SIZE: usize = 4096;

/// Encodes interleaved PCM samples as a FLAC stream.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlacEncoder;

impl FlacEncoder {
    /// Encodes signed, interleaved PCM samples into FLAC bytes.
    ///
    /// `samples` must contain complete frames: its length must be divisible by
    /// `channels`. Samples must fit within the declared `bits_per_sample`.
    pub fn encode_pcm(
        &self,
        samples: &[i32],
        sample_rate: u32,
        channels: u16,
        bits_per_sample: u16,
    ) -> Result<Vec<u8>, FlacEncodingError> {
        validate_parameters(samples, sample_rate, channels, bits_per_sample)?;

        let source = MemSource::from_samples(
            samples,
            usize::from(channels),
            usize::from(bits_per_sample),
            usize::try_from(sample_rate).expect("u32 fits in usize on supported targets"),
        );
        let config = Encoder::default()
            .into_verified()
            .map_err(|error| FlacEncodingError::Configuration(format!("{error:?}")))?;
        let stream = encode_with_fixed_block_size(&config, source, DEFAULT_BLOCK_SIZE)
            .map_err(|error| FlacEncodingError::Encoding(error.to_string()))?;
        let mut sink = ByteSink::new();
        stream
            .write(&mut sink)
            .map_err(|error| FlacEncodingError::Encoding(error.to_string()))?;
        Ok(sink.into_inner())
    }
}

fn validate_parameters(
    samples: &[i32],
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
) -> Result<(), FlacEncodingError> {
    if sample_rate == 0 {
        return Err(FlacEncodingError::InvalidParameters(
            "The sample rate must be greater than zero.".to_owned(),
        ));
    }
    if channels == 0 {
        return Err(FlacEncodingError::InvalidParameters(
            "The channel count must be greater than zero.".to_owned(),
        ));
    }
    if !(4..=32).contains(&bits_per_sample) {
        return Err(FlacEncodingError::InvalidParameters(
            "The bit depth must be between 4 and 32 bits per sample.".to_owned(),
        ));
    }
    if samples.len() % usize::from(channels) != 0 {
        return Err(FlacEncodingError::InvalidParameters(
            "The sample buffer must contain complete interleaved frames.".to_owned(),
        ));
    }
    if bits_per_sample < 32 {
        let minimum = -(1_i64 << (bits_per_sample - 1));
        let maximum = (1_i64 << (bits_per_sample - 1)) - 1;
        if samples
            .iter()
            .any(|&sample| i64::from(sample) < minimum || i64::from(sample) > maximum)
        {
            return Err(FlacEncodingError::InvalidParameters(
                "A PCM sample exceeds the declared bit depth.".to_owned(),
            ));
        }
    }
    Ok(())
}

/// Errors returned while encoding PCM samples as FLAC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlacEncodingError {
    /// The input parameters or PCM samples are invalid.
    InvalidParameters(String),
    /// The encoder configuration could not be created.
    Configuration(String),
    /// The encoder could not produce a valid FLAC stream.
    Encoding(String),
}

impl std::fmt::Display for FlacEncodingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParameters(message)
            | Self::Configuration(message)
            | Self::Encoding(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for FlacEncodingError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_mono_pcm_as_flac() {
        let samples = [0_i32, 100, -100, 500, -500];

        let result = FlacEncoder.encode_pcm(&samples, 48_000, 1, 16);

        assert!(result.is_ok());
        assert!(result.unwrap().starts_with(b"fLaC"));
    }

    #[test]
    fn rejects_incomplete_interleaved_frame() {
        let result = FlacEncoder.encode_pcm(&[0, 1, 2], 48_000, 2, 16);

        assert!(matches!(
            result,
            Err(FlacEncodingError::InvalidParameters(_))
        ));
    }

    #[test]
    fn rejects_zero_sample_rate() {
        let result = FlacEncoder.encode_pcm(&[0], 0, 1, 16);

        assert!(matches!(
            result,
            Err(FlacEncodingError::InvalidParameters(_))
        ));
    }

    #[test]
    fn rejects_samples_outside_declared_bit_depth() {
        let result = FlacEncoder.encode_pcm(&[128], 48_000, 1, 8);

        assert!(matches!(
            result,
            Err(FlacEncodingError::InvalidParameters(_))
        ));
    }
}

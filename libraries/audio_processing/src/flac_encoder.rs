use crate::decode::decode_pcm;
use crate::error::AudioConversionError;
use crate::AudioFormat;
use flacenc::bitsink::{BitSink, ByteSink};
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
    /// Converts audio and writes the resulting FLAC stream to a file.
    ///
    /// The destination is created or truncated. Source bytes are never
    /// modified.
    pub fn convert_to_flac_file<P: AsRef<std::path::Path>>(
        &self,
        source: &[u8],
        format: AudioFormat,
        destination: P,
    ) -> Result<(), AudioConversionError> {
        let mut output = std::fs::File::create(destination)
            .map_err(|error| AudioConversionError::Write(error.to_string()))?;
        self.convert_to_flac_into(source, format, &mut output)
    }

    /// Converts audio and writes the resulting FLAC stream to an output sink.
    ///
    /// The sink can be a file, memory buffer, network-independent application
    /// stream, or any other type implementing [`std::io::Write`].
    pub fn convert_to_flac_into<W: std::io::Write>(
        &self,
        source: &[u8],
        format: AudioFormat,
        output: &mut W,
    ) -> Result<(), AudioConversionError> {
        let encoded = self.convert_to_flac(source, format)?;
        output
            .write_all(&encoded)
            .map_err(|error| AudioConversionError::Write(error.to_string()))
    }

    /// Converts a supported audio container into a FLAC stream in memory.
    ///
    /// The source bytes are decoded without modifying the source. WAV, FLAC,
    /// AIFF, and MP3 inputs are accepted according to `format`.
    ///
    /// Malformed or truncated inputs (in particular MP3 files that trigger an
    /// internal decoder panic) are caught and reported as
    /// [`AudioConversionError::Decode`] instead of aborting the process.
    pub fn convert_to_flac(
        &self,
        source: &[u8],
        format: AudioFormat,
    ) -> Result<Vec<u8>, AudioConversionError> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.decode_and_encode(source, format)
        }))
        .unwrap_or_else(|_| {
            Err(AudioConversionError::Decode(
                "The audio decoder panicked while parsing a malformed or truncated input."
                    .to_owned(),
            ))
        })
    }

    fn decode_and_encode(
        &self,
        source: &[u8],
        format: AudioFormat,
    ) -> Result<Vec<u8>, AudioConversionError> {
        let decoded = decode_pcm(source, format)?;
        self.encode_pcm(
            &decoded.samples,
            decoded.sample_rate,
            decoded.channels,
            decoded.bits_per_sample,
        )
        .map_err(|error| AudioConversionError::Encoding(error.to_string()))
    }

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

        let original_total_samples = samples.len() / usize::from(channels);
        let mut padded_samples = samples.to_vec();
        let remainder = original_total_samples % DEFAULT_BLOCK_SIZE;
        if remainder != 0 {
            padded_samples.resize(
                padded_samples.len()
                    + (DEFAULT_BLOCK_SIZE - remainder) * usize::from(channels),
                0,
            );
        }
        let source = MemSource::from_samples(
            &padded_samples,
            usize::from(channels),
            usize::from(bits_per_sample),
            usize::try_from(sample_rate).expect("u32 fits in usize on supported targets"),
        );
        let mut encoder_config = Encoder::default();
        if bits_per_sample == 24 {
            encoder_config.subframe_coding.use_fixed = false;
            encoder_config.subframe_coding.use_lpc = false;
        }
        let config = encoder_config
            .into_verified()
            .map_err(|error| FlacEncodingError::Configuration(format!("{error:?}")))?;
        let stream = encode_with_fixed_block_size(&config, source, DEFAULT_BLOCK_SIZE)
            .map_err(|error| FlacEncodingError::Encoding(error.to_string()))?;
        let mut stream = stream;
        stream
            .stream_info_mut()
            .set_total_samples(original_total_samples);
        stream
            .verify()
            .map_err(|error| FlacEncodingError::Encoding(format!("{error:?}")))?;
        let mut sink = ByteSink::new();
        stream
            .write(&mut sink)
            .map_err(|error| FlacEncodingError::Encoding(error.to_string()))?;
        sink.align_to_byte()
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
    if !samples.len().is_multiple_of(usize::from(channels)) {
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
    use crate::{AudioMetadataReader, SymphoniaMetadataReader};

    #[test]
    fn encodes_mono_pcm_as_flac() {
        let samples = [0_i32, 100, -100, 500, -500];

        let result = FlacEncoder.encode_pcm(&samples, 48_000, 1, 16);

        assert!(result.is_ok(), "conversion failed: {result:?}");
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

    #[test]
    fn converts_pcm_wav_to_flac() {
        let source = pcm_wav(&[0, 100, -100, 500, -500]);

        let result = FlacEncoder.convert_to_flac(&source, AudioFormat::Wav);

        assert!(result.is_ok(), "conversion failed: {result:?}");
        assert!(result.unwrap().starts_with(b"fLaC"));
    }

    #[test]
    fn converts_generated_flac_to_flac() {
        let samples: Vec<i16> = (0..4096).map(|index| (index % 512) as i16 - 256).collect();
        let source = pcm_wav(&samples);
        let first_conversion = FlacEncoder
            .convert_to_flac(&source, AudioFormat::Wav)
            .expect("WAV conversion should succeed");

        let second_conversion = FlacEncoder
            .convert_to_flac(&first_conversion, AudioFormat::Flac)
            .expect("FLAC conversion should succeed");

        assert!(second_conversion.starts_with(b"fLaC"));
    }

    #[test]
    fn converts_pcm_aiff_to_flac() {
        let samples: Vec<i16> = (0..4096).map(|index| (index % 512) as i16 - 256).collect();
        let source = pcm_aiff(&samples);

        let result = FlacEncoder.convert_to_flac(&source, AudioFormat::Aiff);

        assert!(result.is_ok(), "conversion failed: {result:?}");
        assert!(result.unwrap().starts_with(b"fLaC"));
    }

    #[test]
    fn preserves_wav_metadata_in_flac_output() {
        let samples: Vec<i16> = (0..4096).map(|index| (index % 512) as i16 - 256).collect();
        let source = pcm_wav(&samples);
        let encoded = FlacEncoder
            .convert_to_flac(&source, AudioFormat::Wav)
            .expect("WAV conversion should succeed");
        let metadata = SymphoniaMetadataReader
            .read_metadata(&encoded, AudioFormat::Flac)
            .expect("encoded FLAC metadata should be readable");

        assert_eq!(metadata.format, AudioFormat::Flac);
        assert_eq!(metadata.sample_rate, 48_000);
        assert_eq!(metadata.bit_depth, 16);
        assert_eq!(metadata.channel_count, 1);
        assert_eq!(metadata.duration_samples, samples.len() as u64);
    }

    #[test]
    fn reads_real_24_bit_wav_after_flac_conversion() {
        for name in ["01_Kick", "02_Snare"] {
            let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../../tmp/Multitrack/Complainiacs_Etc_Full.wav/{name}.wav"
            ));
            if !fixture.is_file() {
                return;
            }
            let source = std::fs::read(fixture).expect("24-bit WAV fixture should be readable");
            let encoded = FlacEncoder
                .convert_to_flac(&source, AudioFormat::Wav)
                .expect("24-bit WAV conversion should succeed");
            let metadata = SymphoniaMetadataReader
                .read_metadata(&encoded, AudioFormat::Flac)
                .expect("converted 24-bit FLAC should be readable");

            assert_eq!(metadata.bit_depth, 24);
            assert_eq!(metadata.channel_count, 1);
        }
    }

    #[test]
    fn reads_aiff_metadata_before_conversion() {
        let samples: Vec<i16> = (0..4096).map(|index| (index % 512) as i16 - 256).collect();
        let source = pcm_aiff(&samples);
        let metadata = SymphoniaMetadataReader
            .read_metadata(&source, AudioFormat::Aiff)
            .expect("AIFF metadata should be readable");

        assert_eq!(metadata.format, AudioFormat::Aiff);
        assert_eq!(metadata.sample_rate, 48_000);
        assert_eq!(metadata.bit_depth, 16);
        assert_eq!(metadata.channel_count, 1);
        assert_eq!(metadata.duration_samples, samples.len() as u64);
    }

    #[test]
    fn writes_converted_wav_to_an_output_sink() {
        let source = pcm_wav(&[0, 100, -100]);
        let mut output = std::io::Cursor::new(Vec::new());

        FlacEncoder
            .convert_to_flac_into(&source, AudioFormat::Wav, &mut output)
            .expect("conversion should write to the sink");

        assert!(output.get_ref().starts_with(b"fLaC"));
    }

    #[test]
    fn writes_converted_wav_to_a_file() {
        let source = pcm_wav(&[0, 100, -100]);
        let destination = std::env::temp_dir().join(format!(
            "ui24-session-builder-test-{}.flac",
            std::process::id()
        ));

        FlacEncoder
            .convert_to_flac_file(&source, AudioFormat::Wav, &destination)
            .expect("conversion should write a file");
        let encoded = std::fs::read(&destination).expect("encoded file should be readable");
        std::fs::remove_file(destination).expect("test file should be removable");

        assert!(encoded.starts_with(b"fLaC"));
    }

    #[test]
    fn rejects_truncated_mp3_sources_without_panicking() {
        let truncated = [0xff, 0xfb, 0x90, 0x00, 0x00, 0x00, 0x00];

        let result = FlacEncoder.convert_to_flac(&truncated, AudioFormat::Mp3);

        assert!(
            matches!(result, Err(AudioConversionError::Decode(_))),
            "expected a controlled decode error: {result:?}"
        );
    }

    #[test]
    fn rejects_malformed_mp3_fixture_without_panicking() {
        let fixture =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp/mp3/Cri_wilhelm.mp3");
        if !fixture.is_file() {
            return;
        }
        let source = std::fs::read(&fixture).expect("fixture should be readable");

        let result = FlacEncoder.convert_to_flac(&source, AudioFormat::Mp3);

        assert!(
            matches!(result, Err(AudioConversionError::Decode(_))),
            "expected a controlled decode error instead of a panic: {result:?}"
        );
    }

    fn pcm_wav(samples: &[i16]) -> Vec<u8> {
        let data_size = samples.len() * 2;
        let mut wav = Vec::with_capacity(44 + data_size);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36_u32 + data_size as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&48_000_u32.to_le_bytes());
        wav.extend_from_slice(&96_000_u32.to_le_bytes());
        wav.extend_from_slice(&2_u16.to_le_bytes());
        wav.extend_from_slice(&16_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_size as u32).to_le_bytes());
        for sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }
        wav
    }

    fn pcm_aiff(samples: &[i16]) -> Vec<u8> {
        let data_size = samples.len() * 2;
        let form_size = 46 + data_size;
        let mut aiff = Vec::with_capacity(8 + form_size);
        aiff.extend_from_slice(b"FORM");
        aiff.extend_from_slice(&(form_size as u32).to_be_bytes());
        aiff.extend_from_slice(b"AIFF");
        aiff.extend_from_slice(b"COMM");
        aiff.extend_from_slice(&18_u32.to_be_bytes());
        aiff.extend_from_slice(&1_u16.to_be_bytes());
        aiff.extend_from_slice(&(samples.len() as u32).to_be_bytes());
        aiff.extend_from_slice(&16_u16.to_be_bytes());
        aiff.extend_from_slice(&[0x40, 0x0e, 0xbb, 0x80, 0, 0, 0, 0, 0, 0]);
        aiff.extend_from_slice(b"SSND");
        aiff.extend_from_slice(&(data_size as u32).to_be_bytes());
        aiff.extend_from_slice(&0_u32.to_be_bytes());
        aiff.extend_from_slice(&0_u32.to_be_bytes());
        for sample in samples {
            aiff.extend_from_slice(&sample.to_be_bytes());
        }
        aiff
    }
}

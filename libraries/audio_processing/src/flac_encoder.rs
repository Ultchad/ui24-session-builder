use crate::AudioFormat;
use flacenc::bitsink::ByteSink;
use flacenc::component::BitRepr;
use flacenc::config::Encoder;
use flacenc::encode_with_fixed_block_size;
use flacenc::error::Verify;
use flacenc::source::MemSource;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

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
    /// and AIFF inputs are accepted according to `format`.
    pub fn convert_to_flac(
        &self,
        source: &[u8],
        format: AudioFormat,
    ) -> Result<Vec<u8>, AudioConversionError> {
        if source.is_empty() {
            return Err(AudioConversionError::InvalidInput(
                "The audio source is empty.".to_owned(),
            ));
        }

        let mut hint = Hint::new();
        hint.with_extension(format.extension());
        let stream = MediaSourceStream::new(
            Box::new(std::io::Cursor::new(source.to_owned())),
            MediaSourceStreamOptions::default(),
        );
        let mut probed = get_probe()
            .format(
                &hint,
                stream,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|error| AudioConversionError::Decode(error.to_string()))?;
        let track = probed.format.default_track().ok_or_else(|| {
            AudioConversionError::InvalidInput("The audio source has no default track.".to_owned())
        })?;
        let track_id = track.id;
        let codec_parameters = track.codec_params.clone();
        let sample_rate = codec_parameters.sample_rate.ok_or_else(|| {
            AudioConversionError::InvalidInput("The audio source has no sample rate.".to_owned())
        })?;
        let channels = codec_parameters
            .channels
            .ok_or_else(|| {
                AudioConversionError::InvalidInput("The audio source has no channels.".to_owned())
            })?
            .count();
        let bits_per_sample = codec_parameters.bits_per_sample.ok_or_else(|| {
            AudioConversionError::InvalidInput("The audio source has no bit depth.".to_owned())
        })?;
        let mut decoder = get_codecs()
            .make(&codec_parameters, &DecoderOptions::default())
            .map_err(|error| AudioConversionError::Decode(error.to_string()))?;
        let mut samples = Vec::new();

        loop {
            let packet = match probed.format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    return Err(AudioConversionError::Decode(
                        "The audio decoder requires a reset.".to_owned(),
                    ));
                }
                Err(symphonia::core::errors::Error::IoError(error))
                    if error.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break
                }
                Err(error) => return Err(AudioConversionError::Decode(error.to_string())),
            };
            if packet.track_id() != track_id {
                continue;
            }
            let decoded = decoder
                .decode(&packet)
                .map_err(|error| AudioConversionError::Decode(error.to_string()))?;
            append_samples(decoded, bits_per_sample, &mut samples);
        }

        self.encode_pcm(
            &samples,
            sample_rate,
            u16::try_from(channels).map_err(|_| {
                AudioConversionError::InvalidInput("The channel count is too large.".to_owned())
            })?,
            u16::try_from(bits_per_sample).map_err(|_| {
                AudioConversionError::InvalidInput("The bit depth is too large.".to_owned())
            })?,
        )
        .map_err(AudioConversionError::Encoding)
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

fn append_samples(buffer: AudioBufferRef<'_>, bits_per_sample: u32, destination: &mut Vec<i32>) {
    let mut sample_buffer = SampleBuffer::<i32>::new(buffer.capacity() as u64, *buffer.spec());
    sample_buffer.copy_interleaved_ref(buffer);
    let shift = 32_u32.saturating_sub(bits_per_sample);
    destination.extend(sample_buffer.samples().iter().map(|sample| sample >> shift));
}

/// Errors returned while decoding an audio source before FLAC encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AudioConversionError {
    /// The source metadata or bytes are invalid.
    InvalidInput(String),
    /// The source could not be decoded.
    Decode(String),
    /// PCM encoding failed after decoding.
    Encoding(FlacEncodingError),
    /// The encoded stream could not be written to the output sink.
    Write(String),
}

impl std::fmt::Display for AudioConversionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) | Self::Decode(message) => formatter.write_str(message),
            Self::Encoding(error) => error.fmt(formatter),
            Self::Write(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AudioConversionError {}

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
}

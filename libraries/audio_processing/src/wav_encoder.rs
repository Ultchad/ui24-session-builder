//! Canonical PCM WAV encoding, sharing the same decode stage as FLAC.
use crate::decode::decode_pcm;
use crate::error::AudioConversionError;
use crate::AudioFormat;

const WAV_HEADER_SIZE: u32 = 44;

/// Encodes interleaved PCM samples as a canonical RIFF/WAVE PCM stream.
///
/// WAV output is not confirmed compatible with real Ui24R hardware: the
/// documented `.uirecsession` example only observed `.flac` audio. Use
/// [`crate::FlacEncoder`] for sessions intended for the mixer; use
/// `WavEncoder` for local, uncompressed exports.
#[derive(Clone, Copy, Debug, Default)]
pub struct WavEncoder;

impl WavEncoder {
    /// Converts audio and writes the resulting WAV stream to a file.
    ///
    /// The destination is created or truncated. Source bytes are never
    /// modified.
    pub fn convert_to_wav_file<P: AsRef<std::path::Path>>(
        &self,
        source: &[u8],
        format: AudioFormat,
        destination: P,
    ) -> Result<(), AudioConversionError> {
        let mut output = std::fs::File::create(destination)
            .map_err(|error| AudioConversionError::Write(error.to_string()))?;
        self.convert_to_wav_into(source, format, &mut output)
    }

    /// Converts audio and writes the resulting WAV stream to an output sink.
    pub fn convert_to_wav_into<W: std::io::Write>(
        &self,
        source: &[u8],
        format: AudioFormat,
        output: &mut W,
    ) -> Result<(), AudioConversionError> {
        let encoded = self.convert_to_wav(source, format)?;
        output
            .write_all(&encoded)
            .map_err(|error| AudioConversionError::Write(error.to_string()))
    }

    /// Converts a supported audio container into a canonical PCM WAV stream
    /// in memory.
    ///
    /// Malformed or truncated inputs (in particular MP3 files that trigger
    /// an internal decoder panic) are caught and reported as
    /// [`AudioConversionError::Decode`] instead of aborting the process.
    pub fn convert_to_wav(
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
        encode_pcm_as_wav(
            &decoded.samples,
            decoded.sample_rate,
            decoded.channels,
            decoded.bits_per_sample,
        )
    }
}

fn encode_pcm_as_wav(
    samples: &[i32],
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
) -> Result<Vec<u8>, AudioConversionError> {
    if sample_rate == 0 {
        return Err(AudioConversionError::InvalidInput(
            "The sample rate must be greater than zero.".to_owned(),
        ));
    }
    if channels == 0 {
        return Err(AudioConversionError::InvalidInput(
            "The channel count must be greater than zero.".to_owned(),
        ));
    }
    let bytes_per_sample = match bits_per_sample {
        8 | 16 | 24 | 32 => u32::from(bits_per_sample) / 8,
        _ => {
            return Err(AudioConversionError::Encoding(format!(
                "WAV output only supports 8, 16, 24, or 32-bit PCM; the source declares \
                 {bits_per_sample}-bit samples."
            )))
        }
    };
    if !samples.len().is_multiple_of(usize::from(channels)) {
        return Err(AudioConversionError::InvalidInput(
            "The sample buffer must contain complete interleaved frames.".to_owned(),
        ));
    }

    let data_size = samples.len() as u32 * bytes_per_sample;
    let byte_rate = sample_rate * u32::from(channels) * bytes_per_sample;
    let block_align = channels * u16::try_from(bytes_per_sample).unwrap_or(u16::MAX);

    let mut wav = Vec::with_capacity((WAV_HEADER_SIZE + data_size) as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(WAV_HEADER_SIZE - 8 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for &sample in samples {
        match bytes_per_sample {
            1 => wav.push((sample + 128) as u8),
            2 => wav.extend_from_slice(&(sample as i16).to_le_bytes()),
            3 => wav.extend_from_slice(&sample.to_le_bytes()[0..3]),
            4 => wav.extend_from_slice(&sample.to_le_bytes()),
            _ => unreachable!("validated above"),
        }
    }
    Ok(wav)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AudioMetadataReader, SymphoniaMetadataReader};

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

    #[test]
    fn converts_wav_to_wav_and_preserves_metadata() {
        let samples: Vec<i16> = (0..4096).map(|index| (index % 512) as i16 - 256).collect();
        let source = pcm_wav(&samples);

        let encoded = WavEncoder
            .convert_to_wav(&source, AudioFormat::Wav)
            .expect("WAV conversion should succeed");

        assert!(encoded.starts_with(b"RIFF"));
        assert_eq!(&encoded[8..12], b"WAVE");
        let metadata = SymphoniaMetadataReader
            .read_metadata(&encoded, AudioFormat::Wav)
            .expect("re-reading the encoded WAV should succeed");
        assert_eq!(metadata.sample_rate, 48_000);
        assert_eq!(metadata.bit_depth, 16);
        assert_eq!(metadata.channel_count, 1);
        assert_eq!(metadata.duration_samples, samples.len() as u64);
    }

    #[test]
    fn writes_converted_wav_to_an_output_sink() {
        let source = pcm_wav(&[0, 100, -100]);
        let mut output = std::io::Cursor::new(Vec::new());

        WavEncoder
            .convert_to_wav_into(&source, AudioFormat::Wav, &mut output)
            .expect("conversion should write to the sink");

        assert!(output.get_ref().starts_with(b"RIFF"));
    }

    #[test]
    fn writes_converted_wav_to_a_file() {
        let source = pcm_wav(&[0, 100, -100]);
        let destination = std::env::temp_dir().join(format!(
            "ui24-session-builder-wav-test-{}.wav",
            std::process::id()
        ));

        WavEncoder
            .convert_to_wav_file(&source, AudioFormat::Wav, &destination)
            .expect("conversion should write a file");
        let encoded = std::fs::read(&destination).expect("encoded file should be readable");
        std::fs::remove_file(destination).expect("test file should be removable");

        assert!(encoded.starts_with(b"RIFF"));
    }

    #[test]
    fn rejects_truncated_mp3_sources_without_panicking() {
        let truncated = [0xff, 0xfb, 0x90, 0x00, 0x00, 0x00, 0x00];

        let result = WavEncoder.convert_to_wav(&truncated, AudioFormat::Mp3);

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

        let result = WavEncoder.convert_to_wav(&source, AudioFormat::Mp3);

        assert!(
            matches!(result, Err(AudioConversionError::Decode(_))),
            "expected a controlled decode error instead of a panic: {result:?}"
        );
    }
}

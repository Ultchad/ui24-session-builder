use crate::decode::decode_pcm;
use crate::error::AudioConversionError;
use crate::AudioFormat;
use rusty_mp3::{Error as Mp3Error, Mp3Encoder as RustyMp3Encoder, Mp3EncoderConfig};

/// Constant-bitrate MP3 encoder configured for 320 kbps.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mp3Encoder;

impl Mp3Encoder {
    /// Encodes a supported audio container into a 320 kbps MP3 stream.
    pub fn convert_to_mp3(
        &self,
        source: &[u8],
        format: AudioFormat,
    ) -> Result<Vec<u8>, AudioConversionError> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let decoded = decode_pcm(source, format)?;
            self.encode_pcm(
                &decoded.samples,
                decoded.sample_rate,
                decoded.channels,
                decoded.bits_per_sample,
            )
        }))
        .unwrap_or_else(|_| {
            Err(AudioConversionError::Encoding(
                "The MP3 encoder panicked while processing the decoded audio.".to_owned(),
            ))
        })
    }

    /// Encodes signed interleaved PCM samples into a 320 kbps MP3 stream.
    pub fn encode_pcm(
        &self,
        samples: &[i32],
        sample_rate: u32,
        channels: u16,
        bits_per_sample: u16,
    ) -> Result<Vec<u8>, AudioConversionError> {
        if sample_rate == 0 || channels == 0 || !samples.len().is_multiple_of(usize::from(channels))
        {
            return Err(AudioConversionError::InvalidInput(
                "Invalid PCM parameters for MP3 encoding.".to_owned(),
            ));
        }
        let scale = (1_i64 << bits_per_sample.saturating_sub(1)) as f32;
        let pcm = samples
            .iter()
            .map(|sample| (*sample as f32 / scale).clamp(-1.0, 1.0))
            .collect::<Vec<_>>();
        let config = Mp3EncoderConfig {
            bitrate_kbps: 320,
            vbr_quality: None,
        };
        let mut encoder = RustyMp3Encoder::new(config);
        encoder
            .push_pcm_f32(&pcm, channels, sample_rate)
            .map_err(mp3_error)?;

        let mut output = Vec::new();
        drain_packets(&mut encoder, &mut output, false)?;
        encoder.finish();
        drain_packets(&mut encoder, &mut output, true)?;
        Ok(output)
    }

    /// Encodes a source and writes the 320 kbps MP3 stream to a file.
    pub fn convert_to_mp3_file<P: AsRef<std::path::Path>>(
        &self,
        source: &[u8],
        format: AudioFormat,
        destination: P,
    ) -> Result<(), AudioConversionError> {
        let encoded = self.convert_to_mp3(source, format)?;
        std::fs::write(destination, encoded)
            .map_err(|error| AudioConversionError::Write(error.to_string()))
    }
}

fn drain_packets(
    encoder: &mut RustyMp3Encoder,
    output: &mut Vec<u8>,
    finished: bool,
) -> Result<(), AudioConversionError> {
    loop {
        match encoder.next_packet() {
            Ok(packet) => output.extend(packet),
            Err(Mp3Error::Again) if !finished => break,
            Err(Mp3Error::Eof) => break,
            Err(error) => return Err(mp3_error(error)),
        }
    }
    Ok(())
}

fn mp3_error(error: Mp3Error) -> AudioConversionError {
    AudioConversionError::Encoding(format!("MP3 encoding failed: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AudioMetadataReader, SymphoniaMetadataReader};

    #[test]
    fn encodes_pcm_to_320_kbps_mp3() {
        let samples: Vec<i32> = (0..44_100).map(|index| (index % 512) * 32).collect();
        let encoded = Mp3Encoder
            .encode_pcm(&samples, 44_100, 1, 16)
            .expect("MP3 encoding should succeed");
        assert!(encoded.starts_with(b"ID3") || encoded.first().is_some_and(|byte| *byte == 0xff));
        let metadata = SymphoniaMetadataReader
            .read_metadata(&encoded, AudioFormat::Mp3)
            .expect("encoded MP3 metadata should be readable");
        assert_eq!(metadata.format, AudioFormat::Mp3);
        assert_eq!(metadata.sample_rate, 44_100);
        assert_eq!(metadata.channel_count, 1);
    }
}

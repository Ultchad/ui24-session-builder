use crate::{AudioFormat, AudioMetadata, AudioMetadataReader, AudioProcessingError};
use std::io::Cursor;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

/// Reads WAV, FLAC, AIFF, and MP3 metadata using Symphonia.
#[derive(Clone, Copy, Debug, Default)]
pub struct SymphoniaMetadataReader;

impl AudioMetadataReader for SymphoniaMetadataReader {
    fn read_metadata(
        &self,
        source: &[u8],
        format: AudioFormat,
    ) -> Result<AudioMetadata, AudioProcessingError> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::probe_metadata(source, format)
        }))
        .unwrap_or_else(|_| {
            Err(AudioProcessingError::ReadFailed(
                "The audio probe panicked while parsing a malformed or truncated input.".to_owned(),
            ))
        })
    }
}

impl SymphoniaMetadataReader {
    fn probe_metadata(
        source: &[u8],
        format: AudioFormat,
    ) -> Result<AudioMetadata, AudioProcessingError> {
        if source.is_empty() {
            return Err(AudioProcessingError::ReadFailed(
                "The audio source is empty.".to_owned(),
            ));
        }

        let mut hint = Hint::new();
        hint.with_extension(format.extension());

        let stream = MediaSourceStream::new(
            Box::new(Cursor::new(source.to_owned())),
            MediaSourceStreamOptions::default(),
        );
        let probed = get_probe()
            .format(
                &hint,
                stream,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|error| AudioProcessingError::ReadFailed(error.to_string()))?;
        let track = probed.format.default_track().ok_or_else(|| {
            AudioProcessingError::InvalidMetadata(
                "The audio source has no default track.".to_owned(),
            )
        })?;
        let codec_parameters = &track.codec_params;
        let sample_rate = codec_parameters.sample_rate.ok_or_else(|| {
            AudioProcessingError::InvalidMetadata(
                "The audio source does not declare a sample rate.".to_owned(),
            )
        })?;
        let bit_depth = codec_parameters.bits_per_sample.unwrap_or(0);
        let channel_count = codec_parameters
            .channels
            .ok_or_else(|| {
                AudioProcessingError::InvalidMetadata(
                    "The audio source does not declare any channels.".to_owned(),
                )
            })?
            .count();
        let bit_depth = u16::try_from(bit_depth).map_err(|_| {
            AudioProcessingError::InvalidMetadata(
                "The audio source declares an unsupported bit depth.".to_owned(),
            )
        })?;
        let channel_count = u16::try_from(channel_count).map_err(|_| {
            AudioProcessingError::InvalidMetadata(
                "The audio source declares an unsupported channel count.".to_owned(),
            )
        })?;
        let duration_samples = codec_parameters.n_frames.ok_or_else(|| {
            AudioProcessingError::InvalidMetadata(
                "The audio source does not declare a duration.".to_owned(),
            )
        })?;

        Ok(AudioMetadata {
            format,
            sample_rate,
            bit_depth,
            channel_count,
            duration_samples,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_audio_sources() {
        let result = SymphoniaMetadataReader.read_metadata(&[], AudioFormat::Wav);

        assert_eq!(
            result,
            Err(AudioProcessingError::ReadFailed(
                "The audio source is empty.".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_invalid_audio_sources() {
        let result = SymphoniaMetadataReader.read_metadata(&[0, 1, 2, 3], AudioFormat::Flac);

        assert!(matches!(result, Err(AudioProcessingError::ReadFailed(_))));
    }

    #[test]
    fn rejects_truncated_mp3_sources_without_panicking() {
        let truncated = [0xff, 0xfb, 0x90, 0x00, 0x00, 0x00, 0x00];

        let result = SymphoniaMetadataReader.read_metadata(&truncated, AudioFormat::Mp3);

        assert!(result.is_err(), "expected a controlled error: {result:?}");
    }
}

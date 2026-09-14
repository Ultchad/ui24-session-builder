use crate::AudioFormat;
use std::fmt;

/// Metadata extracted from one audio file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioMetadata {
    /// Source container or codec format.
    pub format: AudioFormat,
    /// Sample rate in hertz.
    pub sample_rate: u32,
    /// Bits per audio sample.
    pub bit_depth: u16,
    /// Number of interleaved audio channels.
    pub channel_count: u16,
    /// Number of samples in the audio stream.
    pub duration_samples: u64,
}

impl AudioMetadata {
    /// Calculates the duration in seconds from samples and sample rate.
    pub fn duration_seconds(&self) -> f64 {
        self.duration_samples as f64 / self.sample_rate as f64
    }
}

/// Reads metadata without exposing platform-specific file APIs to the core.
pub trait AudioMetadataReader {
    /// Reads metadata from an in-memory audio source.
    fn read_metadata(
        &self,
        source: &[u8],
        format: AudioFormat,
    ) -> Result<AudioMetadata, AudioProcessingError>;
}

/// Errors that can occur while obtaining audio metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AudioProcessingError {
    /// The source format is not supported by the selected reader.
    UnsupportedFormat(String),
    /// The source contains invalid or incomplete metadata.
    InvalidMetadata(String),
    /// The source could not be read.
    ReadFailed(String),
}

impl fmt::Display for AudioProcessingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat(message)
            | Self::InvalidMetadata(message)
            | Self::ReadFailed(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AudioProcessingError {}

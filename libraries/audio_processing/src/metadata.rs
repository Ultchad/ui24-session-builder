use crate::AudioFormat;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioMetadata {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channel_count: u16,
    pub duration_samples: u64,
}

impl AudioMetadata {
    pub fn duration_seconds(&self) -> f64 {
        self.duration_samples as f64 / self.sample_rate as f64
    }
}

pub trait AudioMetadataReader {
    fn read_metadata(&self, source: &[u8], format: AudioFormat) -> Result<AudioMetadata, AudioProcessingError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AudioProcessingError {
    UnsupportedFormat(String),
    InvalidMetadata(String),
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

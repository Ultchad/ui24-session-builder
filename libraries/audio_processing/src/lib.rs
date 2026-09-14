//! Platform-independent audio metadata contracts.

mod format;
mod metadata;

pub use format::AudioFormat;
pub use metadata::{AudioMetadata, AudioMetadataReader, AudioProcessingError};

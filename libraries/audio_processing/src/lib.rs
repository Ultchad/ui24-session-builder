//! Platform-independent audio metadata contracts.
#![deny(missing_docs)]

mod format;
mod metadata;

pub use format::AudioFormat;
pub use metadata::{AudioMetadata, AudioMetadataReader, AudioProcessingError};

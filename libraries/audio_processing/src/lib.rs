//! Platform-independent audio metadata contracts.
#![deny(missing_docs)]

mod format;
mod metadata;
mod symphonia_reader;

pub use format::AudioFormat;
pub use metadata::{AudioMetadata, AudioMetadataReader, AudioProcessingError};
pub use symphonia_reader::SymphoniaMetadataReader;

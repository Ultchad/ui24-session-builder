//! Platform-independent audio metadata contracts.
#![deny(missing_docs)]

mod flac_encoder;
mod format;
mod metadata;
mod symphonia_reader;

pub use flac_encoder::{AudioConversionError, FlacEncoder, FlacEncodingError};
pub use format::AudioFormat;
pub use metadata::{AudioMetadata, AudioMetadataReader, AudioProcessingError};
pub use symphonia_reader::SymphoniaMetadataReader;

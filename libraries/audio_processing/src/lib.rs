//! Platform-independent audio metadata contracts.
#![deny(missing_docs)]

mod decode;
mod error;
mod flac_encoder;
mod format;
mod metadata;
mod symphonia_reader;
mod wav_encoder;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::convert_audio_to_flac_bytes;

pub use error::AudioConversionError;
pub use flac_encoder::{FlacEncoder, FlacEncodingError};
pub use format::AudioFormat;
pub use metadata::{AudioMetadata, AudioMetadataReader, AudioProcessingError};
pub use symphonia_reader::SymphoniaMetadataReader;
pub use wav_encoder::WavEncoder;

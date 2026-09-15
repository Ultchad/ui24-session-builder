//! Errors shared by every audio output-format encoder.

/// Errors returned while decoding an audio source before encoding it in an
/// output format (FLAC or WAV).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AudioConversionError {
    /// The source metadata or bytes are invalid.
    InvalidInput(String),
    /// The source could not be decoded.
    Decode(String),
    /// The decoded samples could not be encoded in the destination format.
    Encoding(String),
    /// The encoded stream could not be written to the output sink.
    Write(String),
}

impl std::fmt::Display for AudioConversionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message)
            | Self::Decode(message)
            | Self::Encoding(message)
            | Self::Write(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AudioConversionError {}

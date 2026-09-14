use std::fmt;

/// Audio formats accepted as session input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioFormat {
    /// Waveform Audio File Format.
    Wav,
    /// Free Lossless Audio Codec.
    Flac,
    /// Audio Interchange File Format.
    Aiff,
}

impl AudioFormat {
    /// Maps a filename extension to a supported audio format.
    ///
    /// The leading dot is optional and matching is case-insensitive.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension
            .trim_start_matches('.')
            .to_ascii_lowercase()
            .as_str()
        {
            "wav" => Some(Self::Wav),
            "flac" => Some(Self::Flac),
            "aif" | "aiff" => Some(Self::Aiff),
            _ => None,
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Wav => "WAV",
            Self::Flac => "FLAC",
            Self::Aiff => "AIFF",
        };
        formatter.write_str(name)
    }
}

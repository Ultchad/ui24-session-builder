use std::fmt;

/// Audio formats accepted as session input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioFormat {
    Wav,
    Flac,
    Aiff,
}

impl AudioFormat {
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

use std::fmt;

/// Audio formats accepted as session input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioFormat {
    /// Waveform Audio File Format.
    Wav,
    /// Free Lossless Audio Codec.
    Flac,
    /// MPEG Audio Layer III.
    Mp3,
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
            "mp3" => Some(Self::Mp3),
            "aif" | "aiff" => Some(Self::Aiff),
            _ => None,
        }
    }

    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Mp3 => "mp3",
            Self::Aiff => "aiff",
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Wav => "WAV",
            Self::Flac => "FLAC",
            Self::Mp3 => "MP3",
            Self::Aiff => "AIFF",
        };
        formatter.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_extensions_case_insensitively() {
        assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
        assert_eq!(
            AudioFormat::from_extension(".FLAC"),
            Some(AudioFormat::Flac)
        );
        assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("Aif"), Some(AudioFormat::Aiff));
        assert_eq!(AudioFormat::from_extension("aiff"), Some(AudioFormat::Aiff));
    }

    #[test]
    fn rejects_unknown_extensions() {
        assert_eq!(AudioFormat::from_extension("ogg"), None);
    }

    #[test]
    fn displays_ui24r_audio_format_names() {
        assert_eq!(AudioFormat::Wav.to_string(), "WAV");
        assert_eq!(AudioFormat::Flac.to_string(), "FLAC");
        assert_eq!(AudioFormat::Mp3.to_string(), "MP3");
        assert_eq!(AudioFormat::Aiff.to_string(), "AIFF");
    }
}

use std::fmt;
use ui24_audio_processing::{AudioFormat, AudioMetadata};

/// A Ui24R playback session and its ordered audio tracks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
    /// Session-level metadata shared by all tracks.
    pub metadata: SessionMetadata,
    /// Tracks in their playback and channel-assignment order.
    pub tracks: Vec<SessionTrack>,
}

/// Metadata that applies to the complete playback session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMetadata {
    /// Human-readable session name.
    pub name: String,
    /// Required sample rate for every track, in hertz.
    pub sample_rate: u32,
    /// Session duration measured in audio samples.
    pub duration_samples: u64,
}

/// An audio file mapped to one Ui24R input channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTrack {
    /// Human-readable name shown to the user.
    pub display_name: String,
    /// Audio filename used by the generated session.
    pub file_name: String,
    /// Metadata extracted from the source audio file.
    pub metadata: AudioMetadata,
    /// Ui24R input channel assigned to this track.
    pub channel_assignment: ChannelAssignment,
}

/// A Ui24R input channel assignment.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChannelAssignment(u8);

impl ChannelAssignment {
    /// Highest supported zero-based Ui24R input index.
    pub const MAX_INDEX: u8 = 21;

    /// Creates an assignment for a zero-based Ui24R input index.
    ///
    /// Returns `None` when `index` is outside the supported range `0..=21`.
    pub fn new(index: u8) -> Option<Self> {
        (index <= Self::MAX_INDEX).then_some(Self(index))
    }

    /// Returns the zero-based input index.
    pub fn index(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for ChannelAssignment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "i.{}", self.0)
    }
}

impl SessionTrack {
    /// Returns the source audio format.
    pub fn format(&self) -> AudioFormat {
        self.metadata.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> SessionTrack {
        SessionTrack {
            display_name: "Lead".to_owned(),
            file_name: "lead.wav".to_owned(),
            metadata: AudioMetadata {
                format: AudioFormat::Wav,
                sample_rate: 48_000,
                bit_depth: 24,
                channel_count: 1,
                duration_samples: 48_000,
            },
            channel_assignment: ChannelAssignment::new(0).expect("valid test channel"),
        }
    }

    #[test]
    fn channel_assignment_accepts_supported_boundaries() {
        assert_eq!(ChannelAssignment::new(0).expect("first channel").index(), 0);
        assert_eq!(
            ChannelAssignment::new(ChannelAssignment::MAX_INDEX)
                .expect("last channel")
                .index(),
            ChannelAssignment::MAX_INDEX
        );
    }

    #[test]
    fn channel_assignment_rejects_out_of_range_index() {
        assert!(ChannelAssignment::new(ChannelAssignment::MAX_INDEX + 1).is_none());
    }

    #[test]
    fn channel_assignment_uses_ui24r_display_format() {
        let assignment = ChannelAssignment::new(3).expect("valid test channel");

        assert_eq!(assignment.to_string(), "i.3");
    }

    #[test]
    fn session_track_reports_audio_format() {
        assert_eq!(track().format(), AudioFormat::Wav);
    }
}

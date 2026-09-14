use std::fmt;
use ui24_audio_processing::{AudioFormat, AudioMetadata};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
    pub metadata: SessionMetadata,
    pub tracks: Vec<AudioTrack>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMetadata {
    pub name: String,
    pub sample_rate: u32,
    pub duration_samples: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioTrack {
    pub display_name: String,
    pub file_name: String,
    pub metadata: AudioMetadata,
    pub channel: ChannelAssignment,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChannelAssignment(u8);

impl ChannelAssignment {
    pub const MAX_INDEX: u8 = 21;

    pub fn new(index: u8) -> Option<Self> {
        (index <= Self::MAX_INDEX).then_some(Self(index))
    }

    pub fn index(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for ChannelAssignment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "i.{}", self.0)
    }
}

impl AudioTrack {
    pub fn format(&self) -> AudioFormat {
        self.metadata.format
    }
}

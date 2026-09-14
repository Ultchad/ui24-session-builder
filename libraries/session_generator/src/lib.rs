//! Ui24R `.uirecsession` configuration generation.
#![deny(missing_docs)]

use serde::Serialize;
use std::fmt;
use std::path::Path;
use ui24_core::{validate_session, Session, ValidationIssue};

const UI24R_AUDIO_EXTENSION: &str = ".flac";

/// A verified Ui24R session configuration ready for JSON serialization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Ui24rSessionConfiguration {
    complete: bool,
    ext: String,
    files: Vec<String>,
    #[serde(rename = "lengthSamples")]
    length_samples: u64,
    #[serde(rename = "lengthSeconds")]
    length_seconds: u64,
    mapping: Vec<String>,
    names: Vec<String>,
    #[serde(rename = "sampleRate")]
    sample_rate: u32,
}

impl Ui24rSessionConfiguration {
    /// Returns the generated audio extension, including its leading dot.
    pub fn extension(&self) -> &str {
        &self.ext
    }

    /// Returns the JSON field containing filenames without extensions.
    pub fn files(&self) -> &[String] {
        &self.files
    }

    /// Returns the JSON field containing display names.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Returns the JSON field containing Ui24R channel mappings.
    pub fn mappings(&self) -> &[String] {
        &self.mapping
    }

    /// Serializes this configuration as pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, SessionGenerationError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| SessionGenerationError::Serialization(error.to_string()))
    }

    /// Writes this configuration to a `.uirecsession` JSON file.
    pub fn write_json<P: AsRef<Path>>(&self, destination: P) -> Result<(), SessionGenerationError> {
        let json = self.to_json()?;
        std::fs::write(destination, json)
            .map_err(|error| SessionGenerationError::Write(error.to_string()))
    }
}

/// Generates a verified Ui24R configuration from a validated domain session.
pub fn generate_configuration(
    session: &Session,
) -> Result<Ui24rSessionConfiguration, SessionGenerationError> {
    let issues = validate_session(session);
    if !issues.is_empty() {
        return Err(SessionGenerationError::InvalidSession(issues));
    }

    let length_seconds =
        session.metadata.duration_samples / u64::from(session.metadata.sample_rate);
    let files = session
        .tracks
        .iter()
        .map(|track| filename_without_extension(&track.file_name))
        .collect();
    let names = session
        .tracks
        .iter()
        .map(|track| track.display_name.clone())
        .collect();
    let mapping = session
        .tracks
        .iter()
        .map(|track| track.channel_assignment.to_string())
        .collect();

    Ok(Ui24rSessionConfiguration {
        complete: true,
        ext: UI24R_AUDIO_EXTENSION.to_owned(),
        files,
        length_samples: session.metadata.duration_samples,
        length_seconds,
        mapping,
        names,
        sample_rate: session.metadata.sample_rate,
    })
}

fn filename_without_extension(filename: &str) -> String {
    let basename = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    basename
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(basename)
        .to_owned()
}

/// Errors produced while generating or writing a session configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionGenerationError {
    /// The domain session failed one or more validation rules.
    InvalidSession(Vec<ValidationIssue>),
    /// JSON serialization failed.
    Serialization(String),
    /// The JSON file could not be written.
    Write(String),
}

impl fmt::Display for SessionGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSession(issues) => {
                write!(formatter, "session is invalid ({} issue(s))", issues.len())
            }
            Self::Serialization(message) | Self::Write(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SessionGenerationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use ui24_audio_processing::{AudioFormat, AudioMetadata};
    use ui24_core::{ChannelAssignment, SessionMetadata, SessionTrack};

    fn session() -> Session {
        let tracks = [
            ("03 - Vocal 1", "03 - Vocal 1.flac", 0),
            ("04 - Vocal 2", "04 - Vocal 2.wav", 1),
        ]
        .into_iter()
        .map(|(name, file_name, channel)| SessionTrack {
            display_name: name.to_owned(),
            file_name: file_name.to_owned(),
            metadata: AudioMetadata {
                format: AudioFormat::Wav,
                sample_rate: 48_000,
                bit_depth: 16,
                channel_count: 2,
                duration_samples: 16_320_000,
            },
            channel_assignment: ChannelAssignment::new(channel).expect("valid channel"),
        })
        .collect();

        Session {
            metadata: SessionMetadata {
                name: "Reference".to_owned(),
                sample_rate: 48_000,
                duration_samples: 16_320_000,
            },
            tracks,
        }
    }

    #[test]
    fn generates_observed_uirecsession_fields() {
        let configuration = generate_configuration(&session()).expect("valid session");
        let json = configuration.to_json().expect("serializable configuration");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

        assert_eq!(parsed["complete"], true);
        assert_eq!(parsed["ext"], ".flac");
        assert_eq!(parsed["files"][0], "03 - Vocal 1");
        assert_eq!(parsed["files"][1], "04 - Vocal 2");
        assert_eq!(parsed["names"][0], "03 - Vocal 1");
        assert_eq!(parsed["mapping"][0], "i.0");
        assert_eq!(parsed["sampleRate"], 48_000);
        assert_eq!(parsed["lengthSamples"], 16_320_000_u64);
        assert_eq!(parsed["lengthSeconds"], 340);
    }

    #[test]
    fn rejects_invalid_sessions_before_serialization() {
        let mut invalid = session();
        invalid.tracks[1].channel_assignment = ChannelAssignment::new(0).expect("valid channel");

        assert!(matches!(
            generate_configuration(&invalid),
            Err(SessionGenerationError::InvalidSession(_))
        ));
    }

    #[test]
    fn strips_unix_and_windows_filename_extensions() {
        assert_eq!(filename_without_extension("dir/lead.wav"), "lead");
        assert_eq!(filename_without_extension(r"dir\lead.flac"), "lead");
    }
}

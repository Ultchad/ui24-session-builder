//! Ui24R `.uirecsession` configuration generation.
#![deny(missing_docs)]

use serde::Serialize;
use std::fmt;
use std::path::{Path, PathBuf};
use ui24_core::{validate_session, Session, ValidationIssue};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

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

/// Generates a complete session folder from already-converted FLAC sources.
///
/// `source_files` must contain one existing `.flac` file per session track, in
/// the same order as `session.tracks`. The destination directory is created
/// when necessary and may already exist.
pub fn generate_session_folder(
    session: &Session,
    source_files: &[PathBuf],
    destination: impl AsRef<Path>,
) -> Result<(), SessionGenerationError> {
    let configuration = generate_configuration(session)?;
    if source_files.len() != session.tracks.len() {
        return Err(SessionGenerationError::SourceFiles(format!(
            "expected {} FLAC source file(s), received {}",
            session.tracks.len(),
            source_files.len()
        )));
    }

    let destination = destination.as_ref();
    std::fs::create_dir_all(destination)
        .map_err(|error| SessionGenerationError::Write(error.to_string()))?;

    for (track, source) in session.tracks.iter().zip(source_files) {
        if source.extension().and_then(|extension| extension.to_str()) != Some("flac") {
            return Err(SessionGenerationError::SourceFiles(format!(
                "source {} is not a FLAC file",
                source.display()
            )));
        }
        if !source.is_file() {
            return Err(SessionGenerationError::SourceFiles(format!(
                "source file does not exist: {}",
                source.display()
            )));
        }

        let filename = format!("{}.flac", filename_without_extension(&track.file_name));
        std::fs::copy(source, destination.join(filename))
            .map_err(|error| SessionGenerationError::Write(error.to_string()))?;
    }

    configuration.write_json(destination.join(".uirecsession"))
}

/// Generates a ZIP archive containing the Ui24R session and prepared FLAC files.
///
/// The archive contains the audio files at its root and a `.uirecsession` file
/// at the root, matching the generated folder layout.
pub fn generate_session_zip(
    session: &Session,
    source_files: &[PathBuf],
    destination: impl AsRef<Path>,
) -> Result<(), SessionGenerationError> {
    let configuration = generate_configuration(session)?;
    validate_source_files(session, source_files)?;
    let output = std::fs::File::create(destination)
        .map_err(|error| SessionGenerationError::Write(error.to_string()))?;
    let mut archive = ZipWriter::new(output);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for (track, source) in session.tracks.iter().zip(source_files) {
        let filename = format!("{}.flac", filename_without_extension(&track.file_name));
        archive
            .start_file(filename, options)
            .map_err(|error| SessionGenerationError::Archive(error.to_string()))?;
        let mut input = std::fs::File::open(source)
            .map_err(|error| SessionGenerationError::Write(error.to_string()))?;
        std::io::copy(&mut input, &mut archive)
            .map_err(|error| SessionGenerationError::Write(error.to_string()))?;
    }

    archive
        .start_file(".uirecsession", options)
        .map_err(|error| SessionGenerationError::Archive(error.to_string()))?;
    let json = configuration.to_json()?;
    std::io::Write::write_all(&mut archive, json.as_bytes())
        .map_err(|error| SessionGenerationError::Write(error.to_string()))?;
    archive
        .finish()
        .map_err(|error| SessionGenerationError::Archive(error.to_string()))?;
    Ok(())
}

fn validate_source_files(
    session: &Session,
    source_files: &[PathBuf],
) -> Result<(), SessionGenerationError> {
    if source_files.len() != session.tracks.len() {
        return Err(SessionGenerationError::SourceFiles(format!(
            "expected {} FLAC source file(s), received {}",
            session.tracks.len(),
            source_files.len()
        )));
    }
    for source in source_files {
        if source.extension().and_then(|extension| extension.to_str()) != Some("flac") {
            return Err(SessionGenerationError::SourceFiles(format!(
                "source {} is not a FLAC file",
                source.display()
            )));
        }
        if !source.is_file() {
            return Err(SessionGenerationError::SourceFiles(format!(
                "source file does not exist: {}",
                source.display()
            )));
        }
    }
    Ok(())
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
    /// Source audio files do not match the session requirements.
    SourceFiles(String),
    /// ZIP archive creation failed.
    Archive(String),
}

impl fmt::Display for SessionGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSession(issues) => {
                write!(formatter, "session is invalid ({} issue(s))", issues.len())
            }
            Self::Serialization(message)
            | Self::Write(message)
            | Self::SourceFiles(message)
            | Self::Archive(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SessionGenerationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
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

    #[test]
    fn generates_session_folder_with_configuration_and_flac_sources() {
        let root = std::env::temp_dir().join(format!(
            "ui24-session-generator-test-{}",
            std::process::id()
        ));
        let source_one = root.join("source-one.flac");
        let source_two = root.join("source-two.flac");
        let destination = root.join("session");
        std::fs::create_dir_all(&root).expect("test root should be created");
        std::fs::write(&source_one, b"flac-one").expect("source one should be written");
        std::fs::write(&source_two, b"flac-two").expect("source two should be written");

        generate_session_folder(
            &session(),
            &[source_one.clone(), source_two.clone()],
            &destination,
        )
        .expect("session folder should be generated");

        assert_eq!(
            std::fs::read(destination.join("03 - Vocal 1.flac")).expect("track one should exist"),
            b"flac-one"
        );
        assert!(destination.join(".uirecsession").is_file());
        std::fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn generates_zip_with_audio_and_configuration_at_archive_root() {
        let root =
            std::env::temp_dir().join(format!("ui24-session-generator-zip-{}", std::process::id()));
        let source_one = root.join("source-one.flac");
        let source_two = root.join("source-two.flac");
        let archive_path = root.join("session.zip");
        std::fs::create_dir_all(&root).expect("test root should be created");
        std::fs::write(&source_one, b"flac-one").expect("source one should be written");
        std::fs::write(&source_two, b"flac-two").expect("source two should be written");

        generate_session_zip(&session(), &[source_one, source_two], &archive_path)
            .expect("ZIP should be generated");

        let archive = std::fs::File::open(&archive_path).expect("ZIP should be readable");
        let mut archive = zip::ZipArchive::new(archive).expect("ZIP should be valid");
        assert!(archive.by_name("03 - Vocal 1.flac").is_ok());
        assert!(archive.by_name("04 - Vocal 2.flac").is_ok());
        assert!(archive.by_name(".uirecsession").is_ok());
        std::fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn rejects_non_flac_source_files() {
        let root = std::env::temp_dir().join(format!(
            "ui24-session-generator-invalid-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("test root should be created");
        let wav_source = root.join("source.wav");
        std::fs::write(&wav_source, b"wav").expect("source should be written");

        let result = generate_session_folder(
            &session(),
            &[wav_source, root.join("missing.flac")],
            root.join("session"),
        );

        assert!(matches!(
            result,
            Err(SessionGenerationError::SourceFiles(_))
        ));
        std::fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn validates_official_ui24r_fixture_schema() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tmp/example/Multitrack/Metallica - Enter the sandman/.uirecsession");
        if !fixture.is_file() {
            return;
        }

        let json = std::fs::read_to_string(fixture).expect("official fixture should be readable");
        let value: Value = serde_json::from_str(&json).expect("official fixture should be JSON");
        assert_eq!(value["complete"], true);
        assert_eq!(value["ext"], ".flac");
        assert_eq!(value["sampleRate"], 48_000);
        assert_eq!(value["lengthSamples"], 16_320_000_u64);
        assert_eq!(value["lengthSeconds"], 340);
        assert_eq!(value["files"].as_array().expect("files array").len(), 15);
        assert_eq!(value["names"].as_array().expect("names array").len(), 15);
        assert_eq!(
            value["mapping"].as_array().expect("mapping array").len(),
            15
        );
        assert_eq!(value["mapping"][0], "i.0");
        assert_eq!(value["mapping"][14], "i.14");
    }
}

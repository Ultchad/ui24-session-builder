use crate::{Session, ValidationIssue, ValidationIssueKind};
use std::collections::HashMap;

/// Maximum number of tracks supported by the Ui24R session format.
pub const MAX_TRACK_COUNT: usize = 22;

/// Validates all documented Phase 2 session rules.
///
/// The returned vector is empty when the session is valid. Each issue contains
/// a category, optional track context, and a corrective explanation.
pub fn validate_session(session: &Session) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if session.tracks.is_empty() {
        issues.push(ValidationIssue::new(
            ValidationIssueKind::EmptySession,
            None,
            "A session must contain at least one audio track.",
        ));
        return issues;
    }

    if session.tracks.len() > MAX_TRACK_COUNT {
        issues.push(ValidationIssue::new(
            ValidationIssueKind::TooManyTracks,
            None,
            format!("A session cannot contain more than {MAX_TRACK_COUNT} tracks."),
        ));
    }

    if session.metadata.sample_rate == 0 {
        issues.push(ValidationIssue::new(
            ValidationIssueKind::InvalidSampleRate,
            None,
            "A session sample rate must be greater than zero.",
        ));
    }

    let mut channels = HashMap::new();
    for (track_index, track) in session.tracks.iter().enumerate() {
        if session.metadata.sample_rate != track.metadata.sample_rate {
            issues.push(ValidationIssue::new(
                ValidationIssueKind::SampleRateMismatch,
                Some(track_index),
                format!(
                    "Track {} has sample rate {} but the session requires {}.",
                    track.file_name, track.metadata.sample_rate, session.metadata.sample_rate
                ),
            ));
        }

        if let Some(previous_track) = channels.insert(track.channel_assignment.index(), track_index)
        {
            issues.push(ValidationIssue::new(
                ValidationIssueKind::ChannelConflict,
                Some(track_index),
                format!(
                    "Track {} conflicts with track {} on channel {}.",
                    track.file_name,
                    previous_track + 1,
                    track.channel_assignment
                ),
            ));
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChannelAssignment, SessionMetadata, SessionTrack};
    use ui24_audio_processing::{AudioFormat, AudioMetadata};

    fn track(name: &str, channel: u8, sample_rate: u32) -> SessionTrack {
        SessionTrack {
            display_name: name.to_owned(),
            file_name: format!("{name}.wav"),
            metadata: AudioMetadata {
                format: AudioFormat::Wav,
                sample_rate,
                bit_depth: 24,
                channel_count: 1,
                duration_samples: 48_000,
            },
            channel_assignment: ChannelAssignment::new(channel).expect("valid test channel"),
        }
    }

    fn session(tracks: Vec<SessionTrack>) -> Session {
        Session {
            metadata: SessionMetadata {
                name: "Test session".to_owned(),
                sample_rate: 48_000,
                duration_samples: 48_000,
            },
            tracks,
        }
    }

    #[test]
    fn rejects_empty_session() {
        let issues = validate_session(&session(Vec::new()));

        assert!(issues
            .iter()
            .any(|issue| issue.kind == ValidationIssueKind::EmptySession));
    }

    #[test]
    fn rejects_more_than_maximum_tracks() {
        let mut tracks: Vec<_> = (0..MAX_TRACK_COUNT)
            .map(|channel| track(&format!("track-{channel}"), channel as u8, 48_000))
            .collect();
        tracks.push(track("overflow", 0, 48_000));

        let issues = validate_session(&session(tracks));

        assert!(issues
            .iter()
            .any(|issue| issue.kind == ValidationIssueKind::TooManyTracks));
    }

    #[test]
    fn rejects_channel_conflicts() {
        let issues = validate_session(&session(vec![
            track("lead", 0, 48_000),
            track("backing", 0, 48_000),
        ]));

        assert!(issues
            .iter()
            .any(|issue| issue.kind == ValidationIssueKind::ChannelConflict));
    }

    #[test]
    fn rejects_zero_session_sample_rate() {
        let mut valid_session = session(vec![track("lead", 0, 48_000)]);
        valid_session.metadata.sample_rate = 0;

        let issues = validate_session(&valid_session);

        assert!(issues
            .iter()
            .any(|issue| issue.kind == ValidationIssueKind::InvalidSampleRate));
    }

    #[test]
    fn rejects_track_sample_rate_mismatch() {
        let issues = validate_session(&session(vec![track("lead", 0, 44_100)]));

        assert!(issues
            .iter()
            .any(|issue| issue.kind == ValidationIssueKind::SampleRateMismatch));
    }

    #[test]
    fn accepts_valid_session() {
        let issues = validate_session(&session(vec![
            track("lead", 0, 48_000),
            track("backing", 1, 48_000),
        ]));

        assert!(issues.is_empty());
    }
}

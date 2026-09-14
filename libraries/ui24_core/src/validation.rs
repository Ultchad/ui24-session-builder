use crate::{Session, ValidationIssue, ValidationIssueKind};
use std::collections::HashMap;

pub const MAX_TRACK_COUNT: usize = 22;

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

        if let Some(previous_track) = channels.insert(track.channel.index(), track_index) {
            issues.push(ValidationIssue::new(
                ValidationIssueKind::ChannelConflict,
                Some(track_index),
                format!(
                    "Track {} conflicts with track {} on channel {}.",
                    track.file_name,
                    previous_track + 1,
                    track.channel
                ),
            ));
        }
    }

    issues
}

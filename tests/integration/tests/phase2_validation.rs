use ui24_audio_processing::{AudioFormat, AudioMetadata};
use ui24_core::{validate_session, AudioTrack, ChannelAssignment, Session, SessionMetadata, ValidationIssueKind};

fn track(name: &str, channel: u8, sample_rate: u32) -> AudioTrack {
    AudioTrack {
        display_name: name.to_owned(),
        file_name: format!("{name}.wav"),
        metadata: AudioMetadata {
            format: AudioFormat::Wav,
            sample_rate,
            bit_depth: 24,
            channel_count: 1,
            duration_samples: 48_000,
        },
        channel: ChannelAssignment::new(channel).expect("test channel must be valid"),
    }
}

fn session(tracks: Vec<AudioTrack>) -> Session {
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
fn rejects_empty_sessions() {
    let issues = validate_session(&session(Vec::new()));

    assert_eq!(issues[0].kind, ValidationIssueKind::EmptySession);
}

#[test]
fn rejects_duplicate_channel_assignments() {
    let issues = validate_session(&session(vec![track("lead", 0, 48_000), track("backing", 0, 48_000)]));

    assert!(issues.iter().any(|issue| issue.kind == ValidationIssueKind::ChannelConflict));
}

#[test]
fn rejects_more_than_twenty_two_tracks() {
    let tracks = (0..23)
        .map(|channel| track(&format!("track-{channel}"), channel, 48_000))
        .collect();
    let issues = validate_session(&session(tracks));

    assert!(issues.iter().any(|issue| issue.kind == ValidationIssueKind::TooManyTracks));
}

#[test]
fn rejects_mismatched_sample_rates() {
    let issues = validate_session(&session(vec![track("lead", 0, 44_100)]));

    assert!(issues.iter().any(|issue| issue.kind == ValidationIssueKind::SampleRateMismatch));
}

#[test]
fn accepts_a_valid_session() {
    let issues = validate_session(&session(vec![track("lead", 0, 48_000), track("backing", 1, 48_000)]));

    assert!(issues.is_empty());
}

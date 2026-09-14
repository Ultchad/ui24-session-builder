/// Categories of validation failures that can prevent export.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationIssueKind {
    /// The session contains no tracks.
    EmptySession,
    /// The session exceeds the Ui24R track capacity.
    TooManyTracks,
    /// More than one track uses the same input channel.
    ChannelConflict,
    /// A track uses a sample rate different from the session.
    SampleRateMismatch,
    /// The session declares a zero sample rate.
    InvalidSampleRate,
}

/// A validation failure with context suitable for displaying or logging.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    /// Category of the failure.
    pub kind: ValidationIssueKind,
    /// Zero-based track index involved in the failure, when applicable.
    pub track_index: Option<usize>,
    /// Human-readable explanation and corrective context.
    pub message: String,
}

impl ValidationIssue {
    pub(crate) fn new(
        kind: ValidationIssueKind,
        track_index: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            track_index,
            message: message.into(),
        }
    }
}

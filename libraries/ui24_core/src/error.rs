#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationIssueKind {
    EmptySession,
    TooManyTracks,
    ChannelConflict,
    SampleRateMismatch,
    InvalidSampleRate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    pub kind: ValidationIssueKind,
    pub track_index: Option<usize>,
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

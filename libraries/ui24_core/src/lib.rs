//! Shared, platform-independent Ui24R session domain and validation.

mod domain;
mod error;
mod validation;

pub use domain::{AudioTrack, ChannelAssignment, Session, SessionMetadata};
pub use error::{ValidationIssue, ValidationIssueKind};
pub use validation::{validate_session, MAX_TRACK_COUNT};

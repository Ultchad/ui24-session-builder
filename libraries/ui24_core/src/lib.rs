//! Shared, platform-independent Ui24R session domain and validation.
#![deny(missing_docs)]

mod domain;
mod error;
mod validation;

pub use domain::{ChannelAssignment, Session, SessionMetadata, SessionTrack};
pub use error::{ValidationIssue, ValidationIssueKind};
pub use validation::{validate_session, MAX_TRACK_COUNT};

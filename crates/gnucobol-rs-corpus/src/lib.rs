//! Valid-COBOL corpus subsystem for gnucobol-rs.
//!
//! The subsystem admits valid COBOL programs from external and upstream sources under a
//! **profile-relative validity definition**, preserves original bytes, resolves dependencies and
//! licences, deduplicates, measures the candidate phase by phase, and produces reproducible
//! reports. It never calls a source valid merely because it looks plausible and never infers
//! validity from candidate behaviour: a unit reaches `ADMITTED` only through the explicit
//! custody -> licence -> dependencies -> oracle-compile -> oracle-run -> determinism state
//! machine, and only under a declared validity profile (oracle identity, dialect, source format,
//! encoding, compiler options, copybook paths, defines, runtime configuration, platform).

pub mod bytes;
pub mod cli;
pub mod dedup;
pub mod origin;
pub mod schema;
pub mod state;
pub mod store;

pub use schema::{Classification, CorpusClass, ProgramRecord, SourceFamily};
pub use state::{AdmissionState, RejectionReason};
pub use store::CorpusStore;

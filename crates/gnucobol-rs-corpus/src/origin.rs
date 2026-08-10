//! Origin custody and fetch specifications.
//!
//! Every admitted source must be reproducible from origin + immutable revision + expected hash +
//! extraction rules + licence decision. `check-updates` records newer available revisions
//! without mutating the admitted corpus (a campaign stays reproducible against its exact
//! revision).

use serde::{Deserialize, Serialize};

/// The immutable admission recipe for one source family/repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchSpec {
    pub family: String,
    pub url: String,
    /// Immutable revision (git sha / archive filename / tag) — never moved silently.
    pub revision: String,
    /// Expected SHA-256 of the fetched archive/bundle (rejects corruption / tampering).
    pub archive_sha256: String,
    /// Extraction rules: relative paths admitted from the archive, in order.
    pub extraction_rules: Vec<String>,
    /// Licence decision (see schema::Licence).
    pub licence: String,
    pub licence_decision: String,
    /// Snapshot of newer revisions seen by `check-updates` (never auto-adopted).
    #[serde(default)]
    pub newer_revisions_seen: Vec<String>,
}

impl FetchSpec {
    /// Structural validation of a fetch spec.
    pub fn validate(&self) -> Vec<String> {
        let mut errs = Vec::new();
        if self.family.trim().is_empty() {
            errs.push("family is empty".into());
        }
        if self.url.trim().is_empty() {
            errs.push("url is empty".into());
        }
        if self.revision.trim().is_empty() {
            errs.push("revision is empty".into());
        }
        if self.archive_sha256.len() != 64 {
            errs.push(format!(
                "archive_sha256 must be 64 hex chars, got {}",
                self.archive_sha256.len()
            ));
        }
        if self.licence.trim().is_empty() {
            errs.push("licence is empty".into());
        }
        errs
    }
}

/// Result of `check-updates` for one spec: what changed upstream WITHOUT mutating the admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateReport {
    pub family: String,
    pub pinned_revision: String,
    /// Latest revision currently available upstream ("" when unknown).
    pub latest_revision: String,
    pub has_newer: bool,
    pub note: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_validation() {
        let ok = FetchSpec {
            family: "ccvs85".into(),
            url: "https://www.itl.nist.gov/div897/ctg/soe/ccvs85/".into(),
            revision: "newcob.val.Z".into(),
            archive_sha256: "a".repeat(64),
            extraction_rules: vec!["*.cob".into()],
            licence: "public-domain".into(),
            licence_decision: "NIST CCVS85 is public-domain".into(),
            newer_revisions_seen: vec![],
        };
        assert!(ok.validate().is_empty());
        let mut bad = ok.clone();
        bad.archive_sha256 = "short".into();
        assert!(bad.validate().iter().any(|e| e.contains("64")));
        bad = ok;
        bad.revision = "".into();
        assert!(bad.validate().iter().any(|e| e.contains("revision")));
    }

    #[test]
    fn update_report_marks_newer() {
        let r = UpdateReport {
            family: "x".into(),
            pinned_revision: "abc123".into(),
            latest_revision: "def456".into(),
            has_newer: true,
            note: "3 new commits upstream; admission pin unchanged".into(),
        };
        assert!(r.has_newer);
        let r2 = UpdateReport {
            latest_revision: "abc123".into(),
            has_newer: false,
            ..r
        };
        assert!(!r2.has_newer);
    }
}

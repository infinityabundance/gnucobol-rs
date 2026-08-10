//! The admission state machine.
//!
//! Every unit must walk the ordered chain before it can be reported ADMITTED:
//!
//! ```text
//! DISCOVERED
//!   -> CUSTODY_VERIFIED
//!   -> LICENCE_VERIFIED
//!   -> DEPENDENCIES_RESOLVED
//!   -> ORACLE_COMPILE_VERIFIED
//!   -> ORACLE_RUN_VERIFIED
//!   -> DETERMINISM_VERIFIED
//!   -> ADMITTED
//! ```
//!
//! A unit may instead transition to a typed rejection state. No source may jump directly from
//! discovered to admitted.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionState {
    Discovered,
    CustodyVerified,
    LicenceVerified,
    DependenciesResolved,
    OracleCompileVerified,
    OracleRunVerified,
    DeterminismVerified,
    Admitted,
}

/// Typed rejection reasons. Rejected units stay rejected unless the underlying fact changes and
/// the rejection is explicitly lifted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RejectionReason {
    /// Origin could not be reproduced (hash mismatch / missing revision).
    CustodyFailed,
    /// Licence unknown or not reviewed.
    LicenceUnknown,
    /// Licence does not permit redistribution into the corpus.
    LicenceRestricted,
    /// Dependencies could not be resolved.
    MissingDependency,
    /// The oracle rejects the source under every declared profile.
    OracleReject,
    /// The source is not a complete program (fragment / copybook-only / generated).
    NotAProgram,
    /// Behaviour is nondeterministic under the profile.
    Nondeterministic,
    /// The unit duplicates an already-admitted unit.
    Duplicate,
    /// The unit is quarantined (e.g. pending licence or provenance review).
    Quarantined,
}

/// Valid forward transitions of the admission chain (exact ordering is enforced).
pub fn allowed_transition(from: AdmissionState, to: AdmissionState) -> bool {
    use AdmissionState::*;
    matches!(
        (from, to),
        (Discovered, CustodyVerified)
            | (CustodyVerified, LicenceVerified)
            | (LicenceVerified, DependenciesResolved)
            | (DependenciesResolved, OracleCompileVerified)
            | (OracleCompileVerified, OracleRunVerified)
            | (OracleRunVerified, DeterminismVerified)
            | (DeterminismVerified, Admitted)
    )
}

/// The full chain, in order.
pub const CHAIN: [AdmissionState; 8] = [
    AdmissionState::Discovered,
    AdmissionState::CustodyVerified,
    AdmissionState::LicenceVerified,
    AdmissionState::DependenciesResolved,
    AdmissionState::OracleCompileVerified,
    AdmissionState::OracleRunVerified,
    AdmissionState::DeterminismVerified,
    AdmissionState::Admitted,
];

impl AdmissionState {
    pub fn as_str(self) -> &'static str {
        match self {
            AdmissionState::Discovered => "DISCOVERED",
            AdmissionState::CustodyVerified => "CUSTODY_VERIFIED",
            AdmissionState::LicenceVerified => "LICENCE_VERIFIED",
            AdmissionState::DependenciesResolved => "DEPENDENCIES_RESOLVED",
            AdmissionState::OracleCompileVerified => "ORACLE_COMPILE_VERIFIED",
            AdmissionState::OracleRunVerified => "ORACLE_RUN_VERIFIED",
            AdmissionState::DeterminismVerified => "DETERMINISM_VERIFIED",
            AdmissionState::Admitted => "ADMITTED",
        }
    }

    pub fn parse(s: &str) -> Option<AdmissionState> {
        Some(match s {
            "DISCOVERED" => AdmissionState::Discovered,
            "CUSTODY_VERIFIED" => AdmissionState::CustodyVerified,
            "LICENCE_VERIFIED" => AdmissionState::LicenceVerified,
            "DEPENDENCIES_RESOLVED" => AdmissionState::DependenciesResolved,
            "ORACLE_COMPILE_VERIFIED" => AdmissionState::OracleCompileVerified,
            "ORACLE_RUN_VERIFIED" => AdmissionState::OracleRunVerified,
            "DETERMINISM_VERIFIED" => AdmissionState::DeterminismVerified,
            "ADMITTED" => AdmissionState::Admitted,
            _ => return None,
        })
    }
}

/// A state transition record (for the audit trail).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionLog {
    pub program_id: String,
    pub from: String,
    pub to: String,
    /// Free-form evidence pointer (e.g. blob sha / report path / oracle log).
    pub evidence: String,
}

/// Apply one transition; returns the new state or an error describing the illegal jump.
pub fn transition(from: AdmissionState, to: AdmissionState) -> Result<AdmissionState, String> {
    if allowed_transition(from, to) {
        Ok(to)
    } else {
        Err(format!(
            "illegal admission jump: {} -> {} (the chain is strictly ordered)",
            from.as_str(),
            to.as_str()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_orders_strictly() {
        for w in CHAIN.windows(2) {
            assert!(allowed_transition(w[0], w[1]), "{:?} -> {:?}", w[0], w[1]);
        }
        assert!(!allowed_transition(
            AdmissionState::Discovered,
            AdmissionState::Admitted
        ));
        assert!(!allowed_transition(
            AdmissionState::OracleCompileVerified,
            AdmissionState::Discovered
        ));
        assert!(!allowed_transition(
            AdmissionState::Discovered,
            AdmissionState::Discovered
        ));
    }

    #[test]
    fn transition_rejects_jumps() {
        assert!(transition(AdmissionState::Discovered, AdmissionState::Admitted).is_err());
        assert!(transition(
            AdmissionState::DeterminismVerified,
            AdmissionState::Admitted
        )
        .is_ok());
    }

    #[test]
    fn full_walk_to_admitted() {
        let mut s = AdmissionState::Discovered;
        for w in CHAIN.windows(2) {
            s = transition(s, w[1]).unwrap();
        }
        assert_eq!(s, AdmissionState::Admitted);
    }

    #[test]
    fn parse_round_trip() {
        for s in CHAIN {
            assert_eq!(AdmissionState::parse(s.as_str()), Some(s));
        }
        assert_eq!(AdmissionState::parse("UNKNOWN"), None);
    }
}

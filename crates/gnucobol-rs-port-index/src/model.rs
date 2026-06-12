//! Typed model for the port index. These are the records the symbol indexers emit and the parity
//! joiner consumes — the typed replacement for grep name-matching.

use serde::{Deserialize, Serialize};

/// Whether a C function is actually compiled into the admitted oracle, and if not, why not. This is the
/// field that closes the "does it function in GnuCOBOL?" argument — a `#if 0` source mirror is *present*
/// but not *compiled*, so it must never be conflated with live runtime behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreprocStatus {
    /// Reached by the preprocessor in the admitted build — live runtime behaviour.
    Compiled,
    /// Inside an `#if 0 ... #endif` block — source-present, never compiled.
    If0Disabled,
    /// Inside a `#ifdef COB_EXPERIMENTAL` (or equivalent) gate not enabled in the admitted build.
    ConfigDisabled,
    /// Could not be classified with certainty (reported, never silently treated as compiled).
    Unknown,
}

/// A C function as found in the admitted libcob source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibcobSymbol {
    pub file: String,
    pub function: String,
    pub line_start: usize,
    pub line_end: usize,
    pub preprocessor_status: PreprocStatus,
    /// `true` when the C definition is `static` (internal); `false` for an exported symbol.
    pub is_static: bool,
    /// The gating macro when `preprocessor_status` is config-gated (e.g. `COB_EXPERIMENTAL`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate: Option<String>,
}

/// How a Rust symbol that bears a libcob function's name actually exists in the port — the distinction
/// that stops a name in a doc comment from counting as a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustStatus {
    /// A live `fn` linked into the normal build.
    Active,
    /// A real `fn` but `#[allow(dead_code)]` / `#[cfg(...)]`-gated — the Rust mirror of a disabled C fn.
    InactiveMirror,
    /// A real `fn` that lives only under `#[cfg(test)]`.
    TestOnly,
    /// The name appears in `src` only inside a comment / string / docstring — NOT a real `fn`. A false
    /// hit that grep name-matching would wrongly count as ported.
    DocOnly,
    /// The name does not appear in `src` at all.
    Missing,
}

/// A real Rust function definition found in the port source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustSymbol {
    pub function: String,
    pub module: String,
    pub file: String,
    pub line: usize,
    pub is_pub: bool,
    pub status: RustStatus,
}

/// The joined per-C-function parity verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityFn {
    pub function: String,
    pub preprocessor_status: PreprocStatus,
    pub rust_status: RustStatus,
    /// The Rust module the counterpart lives in (when there is a real `fn`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_module: Option<String>,
}

/// Per-file parity counts — the stronger scoreboard the reviewer's "100% of what?" question needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityRow {
    pub file: String,
    pub source_funcs: usize,
    pub compiled: usize,
    pub disabled: usize,
    pub active_ported: usize,
    pub inactive_mirrors: usize,
    pub test_only: usize,
    pub doc_only: usize,
    pub missing: usize,
    /// Functions with NO real Rust counterpart (missing + doc-only false hits) — the honest gap list.
    pub gap: Vec<String>,
    /// Parity over compiled functions: a compiled C fn is satisfied by a real Rust `fn` (active, inactive
    /// mirror, or test-only). Disabled-source mirrors are tracked but not required for compiled-parity.
    pub compiled_parity_pct: f64,
    pub fns: Vec<ParityFn>,
}

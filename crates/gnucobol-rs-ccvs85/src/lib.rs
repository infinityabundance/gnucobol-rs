//! `gnucobol-rs-ccvs85` — the GNURUST.CCVS85.2/.3/.4 court harness library.
//!
//! The binary (`main.rs`) wires the CLI commands; this library exposes the phases so they can be
//! unit/integration tested without a full benchmark run.

pub mod candidate;
pub mod compare;
pub mod corpus;
pub mod gate;
pub mod model;
pub mod oracle;
pub mod receipts;
pub mod runner;

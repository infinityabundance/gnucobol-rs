//! Typed errors. The port **fails closed**: malformed attributes, undersized buffers, or
//! out-of-claim type pairs return a typed `Err`, never a panic, out-of-bounds index, or
//! arithmetic overflow on hostile input (`GNURUST.PANICPOLICY.0`).

use core::fmt;

/// Why a `cob_move` or field decode/encode could not be performed faithfully.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecimalError {
    /// The destination buffer is smaller than the field's declared `size`.
    DestTooSmall { need: usize, got: usize },
    /// The source buffer is smaller than the field's declared `size`.
    SrcTooSmall { need: usize, got: usize },
    /// The `(source type, destination type)` pair is outside the sealed claim
    /// (storage/move parity for DISPLAY/PACKED). Fails closed — see the crate claim boundary.
    UnsupportedConversion { src_type: u16, dst_type: u16 },
    /// A field attribute is self-inconsistent (e.g. `digits` cannot fit in `size`, or a
    /// nonsensical scale) such that no faithful interpretation exists.
    InvalidAttr(&'static str),
}

impl fmt::Display for DecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecimalError::DestTooSmall { need, got } => {
                write!(f, "destination buffer too small: need {need}, got {got}")
            }
            DecimalError::SrcTooSmall { need, got } => {
                write!(f, "source buffer too small: need {need}, got {got}")
            }
            DecimalError::UnsupportedConversion { src_type, dst_type } => write!(
                f,
                "unsupported conversion: src type 0x{src_type:02x} -> dst type 0x{dst_type:02x} \
                 (sealed claim covers DISPLAY<->PACKED and DISPLAY->DISPLAY)"
            ),
            DecimalError::InvalidAttr(why) => write!(f, "invalid field attribute: {why}"),
        }
    }
}

impl std::error::Error for DecimalError {}

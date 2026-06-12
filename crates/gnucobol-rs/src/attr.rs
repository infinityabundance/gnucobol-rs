//! The GnuCOBOL field-attribute model, ported faithfully from `libcob/common.h`.
//!
//! Citations: `cob_field_attr` / `cob_field` (`common.h:1097-1111`), the `COB_TYPE_*` numeric
//! type tags (`common.h:664-673`), and the `COB_FLAG_*` attribute flags (`common.h:690-712`).
//! Evidence kind: **source_port** (`GNURUST.EVIDENCEKIND.0`).

// --- Field type tags (subset relevant to the sealed decimal slice) --- common.h:664-673
/// `USAGE DISPLAY` numeric (zoned/display). `COB_TYPE_NUMERIC_DISPLAY`.
pub const COB_TYPE_NUMERIC_DISPLAY: u16 = 0x10;
/// `USAGE COMP-3` / `PACKED-DECIMAL`. `COB_TYPE_NUMERIC_PACKED`.
pub const COB_TYPE_NUMERIC_PACKED: u16 = 0x12;
/// `USAGE COMP`/`BINARY`/`COMP-5`/`COMP-X` — binary numeric. `COB_TYPE_NUMERIC_BINARY` (`common.h:666`).
pub const COB_TYPE_NUMERIC_BINARY: u16 = 0x11;
/// A group (record) item. `COB_TYPE_GROUP` (`common.h:661`).
pub const COB_TYPE_GROUP: u16 = 0x01;
/// `NUMERIC-EDITED` (edited numeric picture). `COB_TYPE_NUMERIC_EDITED` (`common.h:678`).
pub const COB_TYPE_NUMERIC_EDITED: u16 = 0x24;
/// `USAGE DISPLAY` alphanumeric (`PIC X`). `COB_TYPE_ALPHANUMERIC` (`common.h:681`).
pub const COB_TYPE_ALPHANUMERIC: u16 = 0x21;
/// A figurative-constant `ALL` source. `COB_TYPE_ALPHANUMERIC_ALL` (`common.h:682`).
pub const COB_TYPE_ALPHANUMERIC_ALL: u16 = 0x22;
/// `ALPHANUMERIC-EDITED` (edited alphanumeric picture). `COB_TYPE_ALPHANUMERIC_EDITED` (`common.h:683`).
pub const COB_TYPE_ALPHANUMERIC_EDITED: u16 = 0x23;

// --- Attribute flags --- common.h:690-701
/// Binary field stored byte-swapped relative to the native host (big-endian on an LE host):
/// `COMP`/`BINARY`/`COMP-X`. `COB_FLAG_BINARY_SWAP` (`common.h:695`).
pub const COB_FLAG_BINARY_SWAP: u16 = 0x0020;
/// "Real" binary (`COMP-5`): native byte order, full binary range (no PIC-digit truncation).
/// `COB_FLAG_REAL_BINARY` (`common.h:696`).
pub const COB_FLAG_REAL_BINARY: u16 = 0x0040;
/// Binary value truncated to the PICTURE digit range (`COMP`/`BINARY` under `binary-truncate: yes`).
/// `COB_FLAG_BINARY_TRUNC` (`common.h:701`).
pub const COB_FLAG_BINARY_TRUNC: u16 = 0x0800;

// --- Attribute flags --- common.h:690-698
/// Field carries a sign (`S` in the PIC). `COB_FLAG_HAVE_SIGN`.
pub const COB_FLAG_HAVE_SIGN: u16 = 0x0001;
/// `SIGN SEPARATE` — the sign occupies its own leading/trailing byte. `COB_FLAG_SIGN_SEPARATE`.
pub const COB_FLAG_SIGN_SEPARATE: u16 = 0x0002;
/// `SIGN LEADING` — the sign is at the front, not the back. `COB_FLAG_SIGN_LEADING`.
pub const COB_FLAG_SIGN_LEADING: u16 = 0x0004;
/// `COMP-6`-style packed with no sign nibble. `COB_FLAG_NO_SIGN_NIBBLE`.
pub const COB_FLAG_NO_SIGN_NIBBLE: u16 = 0x0100;
/// `JUSTIFIED RIGHT` — alphanumeric right-justification. `COB_FLAG_JUSTIFIED` (`common.h:694`).
pub const COB_FLAG_JUSTIFIED: u16 = 0x0010;

/// A field's attributes: type, digit count, scale, and flags.
///
/// Mirrors `cob_field_attr` (`common.h:1097-1103`) minus the `pic` pointer, which the decimal
/// byte semantics do not consult. `digits`, `scale`, and `flags` are the load-bearing fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldAttr {
    /// One of the `COB_TYPE_NUMERIC_*` tags.
    pub field_type: u16,
    /// Number of `9`s in the picture (`COB_FIELD_DIGITS`).
    pub digits: u16,
    /// Implied decimal scale (`COB_FIELD_SCALE`): positive = fractional digits, negative = `P`
    /// scaling positions to the left.
    pub scale: i16,
    /// Bitwise-or of the `COB_FLAG_*` constants (`COB_FIELD_*` predicate macros).
    pub flags: u16,
}

impl FieldAttr {
    /// `COB_FIELD_HAVE_SIGN` (`common.h:704`).
    #[inline]
    pub const fn have_sign(&self) -> bool {
        self.flags & COB_FLAG_HAVE_SIGN != 0
    }
    /// `COB_FIELD_SIGN_SEPARATE` (`common.h:705`).
    #[inline]
    pub const fn sign_separate(&self) -> bool {
        self.flags & COB_FLAG_SIGN_SEPARATE != 0
    }
    /// `COB_FIELD_SIGN_LEADING` (`common.h:706`).
    #[inline]
    pub const fn sign_leading(&self) -> bool {
        self.flags & COB_FLAG_SIGN_LEADING != 0
    }
    /// `COB_FIELD_NO_SIGN_NIBBLE` (`common.h:712`).
    #[inline]
    pub const fn no_sign_nibble(&self) -> bool {
        self.flags & COB_FLAG_NO_SIGN_NIBBLE != 0
    }
    /// `COB_FIELD_JUSTIFIED` — `JUSTIFIED RIGHT` (`common.h:708`).
    #[inline]
    pub const fn justified(&self) -> bool {
        self.flags & COB_FLAG_JUSTIFIED != 0
    }
    /// `COB_FIELD_SIGN_LEADSEP` — leading **and** separate (the only case that shifts
    /// `COB_FIELD_DATA` forward by one byte; `common.h:730`).
    #[inline]
    pub const fn sign_leadsep(&self) -> bool {
        self.sign_leading() && self.sign_separate()
    }

    /// Byte offset of the numeric data within the field's storage — `COB_FIELD_DATA`
    /// (`common.h:730`): `+1` when the sign is leading-separate.
    #[inline]
    pub const fn data_offset(&self) -> usize {
        if self.sign_leadsep() {
            1
        } else {
            0
        }
    }

    /// Number of data bytes excluding a separate sign byte — `COB_FIELD_SIZE`
    /// (`common.h:733`): `size - 1` when `SIGN SEPARATE`.
    #[inline]
    pub const fn data_size(&self, full_size: usize) -> usize {
        if self.sign_separate() {
            full_size.saturating_sub(1)
        } else {
            full_size
        }
    }
}

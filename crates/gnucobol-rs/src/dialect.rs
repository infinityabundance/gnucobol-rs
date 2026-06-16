//! Compile-time **dialect configuration** -- the subset of GnuCOBOL's `cobc` `config/*.conf` knobs that
//! change the *emitted field model* (so `-std=ibm` / `-std=mf` / `-std=mvs` produce different storage).
//! These are decided by the compiler analog (the field-model builder [`crate::pic::build_field_dialect`]),
//! NOT by the runtime config loader ([`crate::common_configload`], which ports `libcob`'s `COB_*` runtime
//! settings -- a different file and a different phase).
//!
//! The values below are the byte-identical-in-repo dialect configs (custody-gated `config/*.conf`):
//!
//! | knob (`config/*.conf`) | default / cobol85+ | ibm / mvs (`-strict` incl.) | mf (`-strict` incl.) |
//! |------------------------|--------------------|-----------------------------|----------------------|
//! | `binary-size`          | `1-2-4-8`          | `2-4-8`                     | `1--8`               |
//! | `binary-truncate`      | `yes`              | `no`                        | `no`                 |
//! | `complex-odo`          | `no`               | `yes`                       | `yes`                |
//!
//! Provenance: `config/default.conf`, `config/ibm-strict.conf` (lines 91/94/62), `config/mf-strict.conf`
//! (94/97/65), `config/mvs-strict.conf` (91/94/62). Oracle (built `cobc`, `PIC 9(1)/9(6) COMP`, and
//! `MOVE 70000 TO PIC 9(4) COMP`): default `LEN1=1 LEN6=4`, `0000`; ibm/mvs `LEN1=2 LEN6=4`, value 4464;
//! mf `LEN1=1 LEN6=3`, value 4464.

use crate::pic::{binary_size, comp_x_size};

/// The `binary-size` knob: how many storage bytes a `COMP` / `BINARY` / `COMP-5` field of `digits` 9s
/// occupies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinarySize {
    /// `1-2-4-8` (default / GnuCOBOL): the power-of-two table.
    Cob1248,
    /// `2-4-8` (ibm / mvs): like the default but with no 1-byte tier (1..4 digits take 2 bytes).
    Cob248,
    /// `1--8` (mf): the *tight* byte count -- the smallest width that holds the PIC digit range (the same
    /// rule as `COMP-X`).
    Cob1to8,
}

impl BinarySize {
    /// Storage bytes for a binary field of `digits` 9s under this `binary-size` setting.
    pub fn bytes(self, digits: u16) -> usize {
        match self {
            // the default 1-2-4-8 table is the single source of truth in pic.rs.
            BinarySize::Cob1248 => binary_size(digits),
            BinarySize::Cob248 => match digits {
                0 => 0,
                1..=4 => 2,
                5..=9 => 4,
                10..=18 => 8,
                _ => 16,
            },
            // tight: identical to the COMP-X minimum-byte rule (256^k >= 10^digits).
            BinarySize::Cob1to8 => comp_x_size(digits),
        }
    }
}

/// The `defaultbyte` knob: how an uninitialized `WORKING-STORAGE` item (no `VALUE`) is filled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefaultByte {
    /// `init` (default): the category default -- `'0'` (0x30) for numeric, space (0x20) for alphanumeric
    /// (GnuCOBOL's "INITIALIZE ALL TO VALUE THEN TO DEFAULT").
    Init,
    /// A single fill byte for ALL uninitialized storage: ibm/mvs `0` (0x00), mf `" "` (0x20). The
    /// cobol85/2002/2014 `none` ("undefined") is observed as 0x00, so it maps here as `Fill(0)`.
    Fill(u8),
}

impl DefaultByte {
    /// The fill byte for an uninitialized *elementary* item. `defaultbyte` governs only ALPHANUMERIC
    /// (and group/FILLER) storage: a numeric DISPLAY elementary item always initializes to figurative
    /// ZERO (`'0'` = 0x30), independent of the dialect -- proven by `cobc -std=ibm` on a standalone
    /// `01 N PIC 9(3)` (still `"000"`, while a standalone `01 A PIC X(3)` becomes 0x00). (A *group*'s
    /// numeric subordinate is filled as part of the alphanumeric group region, but the front-end's
    /// sealed subset is elementary items.)
    pub fn byte(self, is_alpha: bool) -> u8 {
        if !is_alpha {
            return b'0';
        }
        match self {
            DefaultByte::Init => b' ',
            DefaultByte::Fill(b) => b,
        }
    }
}

/// The compile-time dialect knobs that change the emitted field model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dialect {
    /// `binary-size` -- the COMP/BINARY storage-byte table.
    pub binary_size: BinarySize,
    /// `binary-truncate` -- when true (default), a stored value is truncated to the PIC digit range
    /// (`COB_FLAG_BINARY_TRUNC`); when false (ibm/mf/mvs), the field keeps its full binary range.
    pub binary_truncate: bool,
    /// `complex-odo` -- whether complex `OCCURS DEPENDING ON` (indirect / larger redefines / sliding) is
    /// permitted (ibm/mf/mvs `yes`).
    pub complex_odo: bool,
    /// `defaultbyte` -- the fill for uninitialized storage.
    pub defaultbyte: DefaultByte,
}

impl Dialect {
    /// `default.conf` / `-std=default` (the admitted oracle dialect, also `cobol85`/`cobol2002`/`cobol2014`
    /// for these three knobs): `1-2-4-8`, truncate, simple ODO.
    pub const DEFAULT: Dialect = Dialect {
        binary_size: BinarySize::Cob1248,
        binary_truncate: true,
        complex_odo: false,
        defaultbyte: DefaultByte::Init,
    };
    /// `-std=ibm` (`ibm.conf` -> `ibm-strict.conf`): `2-4-8`, no truncate, complex ODO, defaultbyte 0.
    pub const IBM: Dialect = Dialect {
        binary_size: BinarySize::Cob248,
        binary_truncate: false,
        complex_odo: true,
        defaultbyte: DefaultByte::Fill(0),
    };
    /// `-std=mf` (`mf.conf` -> `mf-strict.conf`): `1--8` (tight), no truncate, complex ODO, defaultbyte space.
    pub const MF: Dialect = Dialect {
        binary_size: BinarySize::Cob1to8,
        binary_truncate: false,
        complex_odo: true,
        defaultbyte: DefaultByte::Fill(b' '),
    };
    /// `-std=mvs` (`mvs.conf` -> `mvs-strict.conf`): same four knobs as ibm.
    pub const MVS: Dialect = Dialect {
        binary_size: BinarySize::Cob248,
        binary_truncate: false,
        complex_odo: true,
        defaultbyte: DefaultByte::Fill(0),
    };
    /// `-std=cobol85` / `cobol2002` / `cobol2014`: the same three field-model knobs as DEFAULT, but
    /// `defaultbyte: none` -- undefined storage, observed as 0x00.
    pub const COBOL85: Dialect = Dialect {
        binary_size: BinarySize::Cob1248,
        binary_truncate: true,
        complex_odo: false,
        defaultbyte: DefaultByte::Fill(0),
    };

    /// Resolve a `-std=` name to its [`Dialect`] (the field-model subset). Unknown names fall back to
    /// [`Dialect::DEFAULT`] (GnuCOBOL's behavior when `-std` is unset).
    pub fn from_std(name: &str) -> Dialect {
        match name {
            "ibm" | "ibm-strict" => Dialect::IBM,
            "mf" | "mf-strict" => Dialect::MF,
            "mvs" | "mvs-strict" => Dialect::MVS,
            "cobol85" | "cobol2002" | "cobol2014" => Dialect::COBOL85,
            _ => Dialect::DEFAULT,
        }
    }
}

impl Default for Dialect {
    fn default() -> Self {
        Dialect::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_size_table_matches_cobc_oracle() {
        // PIC 9(1) COMP: default 1, ibm/mvs 2, mf 1 (cobc LEN1).
        assert_eq!(BinarySize::Cob1248.bytes(1), 1);
        assert_eq!(BinarySize::Cob248.bytes(1), 2);
        assert_eq!(BinarySize::Cob1to8.bytes(1), 1);
        // PIC 9(6) COMP: default/ibm/mvs 4, mf 3 (cobc LEN6).
        assert_eq!(BinarySize::Cob1248.bytes(6), 4);
        assert_eq!(BinarySize::Cob248.bytes(6), 4);
        assert_eq!(BinarySize::Cob1to8.bytes(6), 3);
        // PIC 9(4) COMP: all three are 2 bytes (so the value 4464 fits under no-truncate).
        assert_eq!(BinarySize::Cob1248.bytes(4), 2);
        assert_eq!(BinarySize::Cob248.bytes(4), 2);
        assert_eq!(BinarySize::Cob1to8.bytes(4), 2);
    }

    #[test]
    fn from_std_resolves_the_field_model_knobs() {
        assert_eq!(Dialect::from_std("ibm"), Dialect::IBM);
        assert_eq!(Dialect::from_std("mf").binary_size, BinarySize::Cob1to8);
        assert_eq!(Dialect::from_std("mvs").binary_truncate, false);
        assert_eq!(Dialect::from_std("xyz"), Dialect::DEFAULT);
        assert!(Dialect::DEFAULT.binary_truncate);
        assert!(!Dialect::DEFAULT.complex_odo);
    }

    #[test]
    fn defaultbyte_matches_cobc_oracle() {
        // cobc -std=X on STANDALONE elementary items `01 A PIC X(3).` and `01 N PIC 9(3).`, hexdumped:
        //   numeric N is "000" (0x30) under EVERY dialect; only the alpha A changes:
        //   default/mf space (0x20), ibm/mvs/cobol85 0x00.
        for d in [Dialect::DEFAULT, Dialect::IBM, Dialect::MVS, Dialect::MF, Dialect::COBOL85] {
            assert_eq!(d.defaultbyte.byte(false), b'0', "numeric is always '0'");
        }
        assert_eq!(Dialect::DEFAULT.defaultbyte.byte(true), b' ');
        assert_eq!(Dialect::IBM.defaultbyte.byte(true), 0);
        assert_eq!(Dialect::MVS.defaultbyte.byte(true), 0);
        assert_eq!(Dialect::MF.defaultbyte.byte(true), b' ');
        assert_eq!(Dialect::COBOL85.defaultbyte.byte(true), 0);
        assert_eq!(Dialect::from_std("cobol85"), Dialect::COBOL85);
    }
}

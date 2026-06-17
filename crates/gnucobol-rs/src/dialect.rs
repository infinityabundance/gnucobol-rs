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
    /// `complex-odo` -- whether complex `OCCURS DEPENDING ON` (a field *after* an ODO table) is permitted
    /// (ibm/mf/mvs `yes`; default `no` rejects it, as `cobc` does at compile time).
    pub complex_odo: bool,
    /// `odoslide` -- when a field follows an ODO table, whether the trailing items *slide* to the runtime
    /// `DEPENDING ON` count (ibm/mvs `yes`: `LENGTH OF` shrinks with the count) or sit at the table's
    /// physical maximum (mf `no`: a fixed, count-independent length). Only meaningful with `complex-odo`.
    pub odoslide: bool,
    /// `defaultbyte` -- the fill for uninitialized storage.
    pub defaultbyte: DefaultByte,
    /// `move-ibm` -- when true (ibm/mvs), a self-overlapping MOVE (a field onto a shifted reference
    /// modification of itself) uses the IBM `MVC` byte-by-byte left-to-right *propagating* copy; when
    /// false (default/mf), the snapshot (memmove) copy. See [`crate::move_ops::cob_move_overlap`].
    pub move_ibm: bool,
}

impl Dialect {
    /// `default.conf` / `-std=default` (the admitted oracle dialect, also `cobol85`/`cobol2002`/`cobol2014`
    /// for these three knobs): `1-2-4-8`, truncate, simple ODO.
    pub const DEFAULT: Dialect = Dialect {
        binary_size: BinarySize::Cob1248,
        binary_truncate: true,
        complex_odo: false,
        odoslide: false,
        defaultbyte: DefaultByte::Init,
        move_ibm: false,
    };
    /// `-std=ibm` (`ibm.conf` -> `ibm-strict.conf`): `2-4-8`, no truncate, complex ODO + slide, defaultbyte 0.
    pub const IBM: Dialect = Dialect {
        binary_size: BinarySize::Cob248,
        binary_truncate: false,
        complex_odo: true,
        odoslide: true,
        defaultbyte: DefaultByte::Fill(0),
        move_ibm: true,
    };
    /// `-std=mf` (`mf.conf` -> `mf-strict.conf`): `1--8` (tight), no truncate, complex ODO (no slide),
    /// defaultbyte space.
    pub const MF: Dialect = Dialect {
        binary_size: BinarySize::Cob1to8,
        binary_truncate: false,
        complex_odo: true,
        odoslide: false,
        defaultbyte: DefaultByte::Fill(b' '),
        move_ibm: false,
    };
    /// `-std=mvs` (`mvs.conf` -> `mvs-strict.conf`): same five knobs as ibm.
    pub const MVS: Dialect = Dialect {
        binary_size: BinarySize::Cob248,
        binary_truncate: false,
        complex_odo: true,
        odoslide: true,
        defaultbyte: DefaultByte::Fill(0),
        move_ibm: true,
    };
    /// `-std=cobol85` / `cobol2002` / `cobol2014`: the same field-model knobs as DEFAULT, but
    /// `defaultbyte: none` -- undefined storage, observed as 0x00.
    pub const COBOL85: Dialect = Dialect {
        binary_size: BinarySize::Cob1248,
        binary_truncate: true,
        complex_odo: false,
        odoslide: false,
        defaultbyte: DefaultByte::Fill(0),
        move_ibm: false,
    };

    /// Resolve a `-std=` name to its [`Dialect`] (the field-model subset). Unknown names fall back to
    /// [`Dialect::DEFAULT`] (GnuCOBOL's behavior when `-std` is unset). The returned knobs are PROVEN equal
    /// to the parsed `config/<name>.conf` (see [`Dialect::from_conf`] + the `from_std_equals_parsed_conf`
    /// test), so this fast hardcoded path is 1:1 with the shipped dialect configuration files.
    pub fn from_std(name: &str) -> Dialect {
        match name {
            "ibm" | "ibm-strict" => Dialect::IBM,
            "mf" | "mf-strict" => Dialect::MF,
            "mvs" | "mvs-strict" => Dialect::MVS,
            "cobol85" | "cobol2002" | "cobol2014" => Dialect::COBOL85,
            _ => Dialect::DEFAULT,
        }
    }

    /// Parse a cobc dialect configuration file (`config/<name>.conf`) into the runtime [`Dialect`] knobs,
    /// 1:1 with cobc/config.c -- resolving `include "file"` chains in order (an included file's settings
    /// apply where the directive appears, so a later line overrides). `read` maps a config filename to its
    /// bytes (the OS boundary). Only the runtime field-model knobs are consumed (binary-size,
    /// binary-truncate, complex-odo, odoslide, move-ibm, defaultbyte); compiler-only settings (tab-width,
    /// word-length, warnings) and `.words` reserved-word includes are parsed-and-skipped, as the port has
    /// no native codegen. Returns the resolved [`Dialect`], starting from [`Dialect::DEFAULT`].
    pub fn from_conf(name: &str, read: &dyn Fn(&str) -> Option<Vec<u8>>) -> Option<Dialect> {
        let mut d = Dialect::DEFAULT;
        apply_conf(name, read, &mut d, 0)?;
        Some(d)
    }
}

/// One overlay pass of a dialect `.conf` onto `d` (recursing on `include`). Cycle-guarded by `depth`.
fn apply_conf(name: &str, read: &dyn Fn(&str) -> Option<Vec<u8>>, d: &mut Dialect, depth: u32) -> Option<()> {
    if depth > 16 {
        return None;
    }
    let body = read(name)?;
    for raw in body.split(|&b| b == b'\n') {
        let Some((key, val)) = parse_conf_line(raw) else { continue };
        match key.as_str() {
            "include" => {
                // include "file" / include: "file" -- resolve recursively (settings apply in place). A
                // `.words` reserved-word include is a separate concern (no field-model effect).
                if !val.ends_with(".words") {
                    apply_conf(&val, read, d, depth + 1);
                }
            }
            "binary-size" => {
                d.binary_size = match val.as_str() {
                    "2-4-8" => BinarySize::Cob248,
                    "1--8" => BinarySize::Cob1to8,
                    _ => BinarySize::Cob1248, // "1-2-4-8" (the GnuCOBOL default)
                }
            }
            "binary-truncate" => d.binary_truncate = val == "yes",
            "complex-odo" => d.complex_odo = val == "yes",
            "odoslide" => d.odoslide = val == "yes",
            "move-ibm" => d.move_ibm = val == "yes",
            "defaultbyte" => {
                d.defaultbyte = match val.as_str() {
                    "init" => DefaultByte::Init,
                    "0" | "none" => DefaultByte::Fill(0), // cobol85 'none' fills 0x00 per the oracle
                    " " | "32" | "space" => DefaultByte::Fill(b' '),
                    other => {
                        // a single decimal byte value (e.g. defaultbyte: 65) or a quoted char.
                        if let Ok(n) = other.parse::<u8>() {
                            DefaultByte::Fill(n)
                        } else if let Some(c) = other.bytes().next() {
                            DefaultByte::Fill(c)
                        } else {
                            DefaultByte::Init
                        }
                    }
                }
            }
            _ => {} // compiler-only or not field-model-relevant
        }
    }
    Some(())
}

/// Parse one `.conf` line into `(key, value)`. Returns `None` for a blank/comment line. The key is the
/// first token (a trailing `:` stripped); the value is the remainder up to an inline `#` comment, trimmed
/// and unquoted (`"..."`). Tabs and spaces both separate.
fn parse_conf_line(raw: &[u8]) -> Option<(String, String)> {
    // trim leading whitespace; skip blank / comment.
    let line: &[u8] = {
        let mut s = raw;
        while let [first, rest @ ..] = s {
            if *first == b' ' || *first == b'\t' || *first == b'\r' {
                s = rest;
            } else {
                break;
            }
        }
        s
    };
    if line.is_empty() || line[0] == b'#' {
        return None;
    }
    // key: up to whitespace or ':'.
    let mut i = 0;
    while i < line.len() && line[i] != b' ' && line[i] != b'\t' && line[i] != b':' {
        i += 1;
    }
    let key = String::from_utf8_lossy(&line[..i]).to_string();
    // skip a ':' and following whitespace.
    if i < line.len() && line[i] == b':' {
        i += 1;
    }
    while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
        i += 1;
    }
    // value: to end-of-line or an inline '#', then trim + unquote.
    let mut rest = &line[i..];
    if let Some(h) = rest.iter().position(|&b| b == b'#') {
        rest = &rest[..h];
    }
    // trim trailing whitespace/CR.
    while let [head @ .., last] = rest {
        if *last == b' ' || *last == b'\t' || *last == b'\r' {
            rest = head;
        } else {
            break;
        }
    }
    let mut val = String::from_utf8_lossy(rest).to_string();
    // unquote "..." (keeping the inner bytes, e.g. a single space for defaultbyte).
    if val.len() >= 2 && val.starts_with('"') && val.ends_with('"') {
        val = val[1..val.len() - 1].to_string();
    }
    if key.is_empty() {
        return None;
    }
    Some((key, val))
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
    fn from_std_equals_parsed_conf() {
        // The runtime dialect knobs (binary-size/truncate, complex-odo, odoslide, move-ibm, defaultbyte)
        // are 1:1 with the shipped config/<name>.conf files: parsing each -std= dialect's .conf (resolving
        // its include chain natively) must EQUAL the hardcoded from_std Dialect. This is the parity that
        // makes the config files genuinely wired in native Rust, not just copied.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config");
        let read = |fname: &str| std::fs::read(dir.join(fname)).ok();
        for name in ["default", "ibm", "ibm-strict", "mf", "mvs", "cobol85", "cobol2002", "cobol2014"] {
            let parsed = Dialect::from_conf(&format!("{name}.conf"), &read)
                .unwrap_or_else(|| panic!("parse config/{name}.conf"));
            assert_eq!(parsed, Dialect::from_std(name), "config/{name}.conf != from_std({name})");
        }
        // the parser actually reads the file (not a silent default): ibm.conf -> the ibm knobs, distinct
        // from DEFAULT.
        let ibm = Dialect::from_conf("ibm.conf", &read).unwrap();
        assert_ne!(ibm, Dialect::DEFAULT);
        assert_eq!(ibm.binary_size, BinarySize::Cob248);
        assert!(ibm.complex_odo && ibm.odoslide && ibm.move_ibm && !ibm.binary_truncate);
        assert_eq!(ibm.defaultbyte, DefaultByte::Fill(0));
    }

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
        // complex-odo + odoslide: default neither; ibm/mvs both; mf complex-odo but NO slide.
        assert!(Dialect::IBM.complex_odo && Dialect::IBM.odoslide);
        assert!(Dialect::MVS.complex_odo && Dialect::MVS.odoslide);
        assert!(Dialect::MF.complex_odo && !Dialect::MF.odoslide);
        assert!(!Dialect::DEFAULT.odoslide);
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

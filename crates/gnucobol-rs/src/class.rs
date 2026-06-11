//! Class-condition byte predicates (`GNURUST.CLASS.1`): `IF data-item IS NUMERIC / ALPHABETIC /
//! ALPHABETIC-UPPER / ALPHABETIC-LOWER` over an **alphanumeric** (`PIC X`) field, proven byte-identical
//! to cobc.
//!
//! **Doctrine.** These are pure byte predicates over the raw field bytes — the ubiquitous legacy
//! validation idiom (`IF CUST-NO IS NUMERIC`). cobc's admitted rules (diagnosed from the oracle):
//! - **NUMERIC**: every byte is an ASCII digit `0x30..=0x39` (a space, sign, or letter ⇒ not numeric);
//! - **ALPHABETIC**: every byte is an ASCII letter (`A-Z`/`a-z`) or space;
//! - **ALPHABETIC-UPPER**: every byte is `A-Z` or space;
//! - **ALPHABETIC-LOWER**: every byte is `a-z` or space.
//!
//! **Not modelled:** the signed-numeric (overpunch) class test on a `PIC S9` field, user-defined `CLASS`
//! names, national/UTF-8/DBCS classes, and the locale collating sequence.

/// `IS NUMERIC` for an alphanumeric field: non-empty and every byte an ASCII digit.
pub fn is_numeric(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.iter().all(u8::is_ascii_digit)
}

/// `IS ALPHABETIC`: every byte an ASCII letter or space.
pub fn is_alphabetic(bytes: &[u8]) -> bool {
    bytes.iter().all(|b| b.is_ascii_alphabetic() || *b == b' ')
}

/// `IS ALPHABETIC-UPPER`: every byte `A-Z` or space.
pub fn is_alphabetic_upper(bytes: &[u8]) -> bool {
    bytes.iter().all(|b| b.is_ascii_uppercase() || *b == b' ')
}

/// `IS ALPHABETIC-LOWER`: every byte `a-z` or space.
pub fn is_alphabetic_lower(bytes: &[u8]) -> bool {
    bytes.iter().all(|b| b.is_ascii_lowercase() || *b == b' ')
}

/// Fuzz entry (`GNURUST.CLASS.1`): the predicates are total over any bytes — never panic.
pub fn __fuzz_class(data: &[u8]) {
    let _ = (
        is_numeric(data),
        is_alphabetic(data),
        is_alphabetic_upper(data),
        is_alphabetic_lower(data),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_conditions_match_the_oracle() {
        assert!(is_numeric(b"0012"));
        assert!(!is_numeric(b" 12 ")); // spaces are not numeric
        assert!(!is_numeric(b"12AB"));
        assert!(!is_numeric(b"")); // empty is not numeric
        assert!(is_alphabetic(b"ABCD"));
        assert!(!is_alphabetic(b"AB12"));
        assert!(is_alphabetic(b"AB  ")); // spaces are alphabetic
        assert!(is_alphabetic(b"abcd"));
        assert!(is_alphabetic_upper(b"ABCD"));
        assert!(!is_alphabetic_upper(b"abcd"));
        assert!(is_alphabetic_lower(b"abcd"));
        assert!(!is_alphabetic_lower(b"ABCD"));
        assert!(is_alphabetic_upper(b"AB  ") && is_alphabetic_lower(b"ab  "));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.CLASS.1
    /// the class predicates are total over symbolic bytes (never panic).
    #[kani::proof]
    #[kani::unwind(5)]
    fn class_predicates_are_total() {
        let bytes: [u8; 4] = kani::any();
        let _ = is_numeric(&bytes);
        let _ = is_alphabetic(&bytes);
        let _ = is_alphabetic_upper(&bytes);
        let _ = is_alphabetic_lower(&bytes);
    }
}

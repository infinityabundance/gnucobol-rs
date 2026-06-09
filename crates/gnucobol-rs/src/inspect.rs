//! `INSPECT` byte effects (`GNURUST.INSPECT.1`): the receiver/target bytes and tally counts produced by
//! narrow `INSPECT ... TALLYING`, `... REPLACING`, and `... CONVERTING` statements, proven against
//! GnuCOBOL 3.2. The companion data-mutation court to `GNURUST.INITIALIZE.1`.
//!
//! **Witnessed rules (from the oracle):** scanning is **left-to-right, non-overlapping** (consume each
//! match); `ALL` counts/replaces every non-overlapping occurrence, `LEADING` only the run from the region
//! start, `FIRST` only the first occurrence, `CHARACTERS` counts every position in the region; `CONVERTING`
//! is a per-byte translation; `BEFORE INITIAL d` restricts the region to before the first `d` (whole region
//! if `d` is absent), `AFTER INITIAL d` to after the first `d` (empty if `d` is absent). `REPLACING`/
//! `CONVERTING` require equal-length operands (a sealed precondition).
//!
//! **Non-claims:** full Procedure Division execution, locale/case-folding, regex/pattern semantics,
//! national/UTF-8 multibyte behavior, unadmitted multi-clause ordering, business validation, all dialects.

/// The `BEFORE`/`AFTER INITIAL` region delimiter for an `INSPECT` clause.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Region<'a> {
    /// The whole target.
    All,
    /// `BEFORE INITIAL d` — up to (not including) the first `d`; the whole target if `d` is absent.
    Before(&'a [u8]),
    /// `AFTER INITIAL d` — after the first `d`; empty if `d` is absent.
    After(&'a [u8]),
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > hay.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The `[start, end)` byte bounds of the region within `target`.
fn region_bounds(target: &[u8], r: Region) -> (usize, usize) {
    match r {
        Region::All => (0, target.len()),
        Region::Before(d) => (0, find(target, d).unwrap_or(target.len())),
        Region::After(d) => (find(target, d).map_or(target.len(), |i| i + d.len()), target.len()),
    }
}

/// What `INSPECT ... TALLYING` counts in the region.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum TallyMode<'a> {
    /// `FOR ALL x` — every non-overlapping occurrence of `x`.
    All(&'a [u8]),
    /// `FOR LEADING x` — the run of `x` from the region start.
    Leading(&'a [u8]),
    /// `FOR CHARACTERS` — every character position in the region.
    Characters,
}

/// `INSPECT target TALLYING ... FOR <mode> [<region>]` — the number ADDED to the tally counter.
pub fn inspect_tallying(target: &[u8], mode: TallyMode, region: Region) -> u64 {
    let (s, e) = region_bounds(target, region);
    let reg = &target[s..e];
    match mode {
        TallyMode::Characters => reg.len() as u64,
        TallyMode::All(item) => {
            if item.is_empty() {
                return 0;
            }
            let (mut count, mut i) = (0u64, 0usize);
            while i + item.len() <= reg.len() {
                if &reg[i..i + item.len()] == item {
                    count += 1;
                    i += item.len();
                } else {
                    i += 1;
                }
            }
            count
        }
        TallyMode::Leading(item) => {
            if item.is_empty() {
                return 0;
            }
            let (mut count, mut i) = (0u64, 0usize);
            while i + item.len() <= reg.len() && &reg[i..i + item.len()] == item {
                count += 1;
                i += item.len();
            }
            count
        }
    }
}

/// How `INSPECT ... REPLACING` rewrites the region (operands must be equal length).
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ReplaceMode<'a> {
    /// `REPLACING ALL x BY y` — every non-overlapping `x`.
    All(&'a [u8], &'a [u8]),
    /// `REPLACING LEADING x BY y` — the run of `x` from the region start.
    Leading(&'a [u8], &'a [u8]),
    /// `REPLACING FIRST x BY y` — the first `x` in the region.
    First(&'a [u8], &'a [u8]),
}

/// `INSPECT target REPLACING <mode> [<region>]` — the target bytes after replacement.
pub fn inspect_replacing(target: &[u8], mode: ReplaceMode, region: Region) -> Vec<u8> {
    let (s, e) = region_bounds(target, region);
    let mut out = target.to_vec();
    let (item, by, only_leading, only_first) = match mode {
        ReplaceMode::All(x, y) => (x, y, false, false),
        ReplaceMode::Leading(x, y) => (x, y, true, false),
        ReplaceMode::First(x, y) => (x, y, false, true),
    };
    if item.is_empty() || item.len() != by.len() {
        return out;
    }
    let mut i = s;
    while i + item.len() <= e {
        if &out[i..i + item.len()] == item {
            out[i..i + item.len()].copy_from_slice(by);
            if only_first {
                break;
            }
            i += item.len();
        } else {
            if only_leading {
                break;
            }
            i += 1;
        }
    }
    out
}

/// `INSPECT target CONVERTING from TO to [<region>]` — the target bytes after the per-byte translation
/// (`from[i]` → `to[i]`; first match wins on a duplicate in `from`). `from` and `to` must be equal length.
pub fn inspect_converting(target: &[u8], from: &[u8], to: &[u8], region: Region) -> Vec<u8> {
    let (s, e) = region_bounds(target, region);
    let mut out = target.to_vec();
    if from.len() != to.len() {
        return out;
    }
    for b in &mut out[s..e] {
        if let Some(j) = from.iter().position(|&c| c == *b) {
            *b = to[j];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tallying_base_and_traps() {
        assert_eq!(inspect_tallying(b"ABABAA", TallyMode::All(b"A"), Region::All), 4);
        assert_eq!(inspect_tallying(b"AABBAA", TallyMode::Leading(b"A"), Region::All), 2);
        assert_eq!(inspect_tallying(b"ABCDEF", TallyMode::Characters, Region::All), 6);
        assert_eq!(inspect_tallying(b"AAAAAA", TallyMode::All(b"AA"), Region::All), 3); // non-overlapping
        assert_eq!(inspect_tallying(b"ABXABY", TallyMode::All(b"A"), Region::Before(b"X")), 1);
    }
    #[test]
    fn replacing_base_and_region() {
        assert_eq!(inspect_replacing(b"ABABAA", ReplaceMode::All(b"A", b"B"), Region::All), b"BBBBBB");
        assert_eq!(inspect_replacing(b"AABBAA", ReplaceMode::Leading(b"A", b"X"), Region::All), b"XXBBAA");
        assert_eq!(inspect_replacing(b"ABABAA", ReplaceMode::First(b"A", b"Z"), Region::All), b"ZBABAA");
        assert_eq!(inspect_replacing(b"ABXABY", ReplaceMode::All(b"A", b"Z"), Region::After(b"X")), b"ABXZBY");
    }
    #[test]
    fn converting_translates() {
        assert_eq!(inspect_converting(b"CABFED", b"ABCDEF", b"UVWXYZ", Region::All), b"WUVZYX");
    }
    #[test]
    fn absent_delimiter_regions() {
        // BEFORE absent -> whole target ; AFTER absent -> empty region
        assert_eq!(inspect_tallying(b"ABAB", TallyMode::All(b"A"), Region::Before(b"Q")), 2);
        assert_eq!(inspect_tallying(b"ABAB", TallyMode::All(b"A"), Region::After(b"Q")), 0);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.INSPECT.1
    /// INSPECT REPLACING with an equal-length from/to never changes the target length.
    #[kani::proof]
    #[kani::unwind(8)]
    fn inspect_replacing_equal_len_preserves_length() {
        let target: [u8; 5] = kani::any();
        let from: [u8; 1] = kani::any();
        let to: [u8; 1] = kani::any();
        let out = inspect_replacing(&target, ReplaceMode::All(&from, &to), Region::All);
        assert_eq!(out.len(), target.len());
    }
}

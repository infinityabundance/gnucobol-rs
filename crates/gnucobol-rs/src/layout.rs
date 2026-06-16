//! DATA DIVISION record layout (`GNURUST.4`): assign each item its byte **offset** and **size**
//! within an `01` record, handling level-numbered groups, fixed `OCCURS n TIMES`, `REDEFINES`
//! overlay, and `FILLER`, matching the GnuCOBOL compiler's own record-layout decision (verified
//! against `cobc`'s emitted field offsets and runtime `LENGTH OF`).
//!
//! **Sealed subset:** level-numbered groups + elementary items whose `PIC` is in the sealed
//! [`crate::pic`] subset; fixed `OCCURS n TIMES`; a single trailing `OCCURS min TO max DEPENDING ON`
//! contributing its **physical maximum** (`GNURUST.10`; the active/logical count is a non-claim);
//! `REDEFINES <name>` where the redefining item is **no larger** than the item it redefines;
//! `FILLER`. Everything else **fails closed** with a typed [`LayoutError`] — `SYNCHRONIZED`
//! alignment, a `REDEFINES` larger than its target, multiple/nested ODO, and ODO combined with
//! REDEFINES are deferred future courts, not silently mis-laid.

use crate::pic::{self, Usage};

/// One source item of a record (a line of the DATA DIVISION).
#[derive(Debug, Clone)]
pub struct Item {
    /// Level number (01, 05, 10, …).
    pub level: u16,
    /// Item name, or `"FILLER"`.
    pub name: String,
    /// `Some((pic, usage, sign_separate, sign_leading))` for an elementary item; `None` for a group.
    pub pic: Option<(String, Usage, bool, bool)>,
    /// Fixed `OCCURS n TIMES`.
    pub occurs: Option<u32>,
    /// `REDEFINES <name>` target, if any.
    pub redefines: Option<String>,
    /// `OCCURS min TO max TIMES DEPENDING ON <item>` (`GNURUST.10`). When present, the item
    /// contributes its **physical maximum** (`max` occurrences) to the record layout. The active
    /// (logical) occurrence count is **not** modelled here — that is an explicit non-claim.
    pub odo: Option<Odo>,
}

/// An `OCCURS DEPENDING ON` declaration. Only its `max` drives the **physical** layout; `min` and
/// `depending_on` are carried as metadata so a consumer never mistakes physical-max for logical length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Odo {
    pub min: u32,
    pub max: u32,
    /// The name of the `DEPENDING ON` control item (must reference a field in the record).
    pub depending_on: String,
}

/// A laid-out item: its byte `offset` from the record start and the `size` of **one** occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Laid {
    pub name: String,
    pub level: u16,
    pub offset: usize,
    pub size: usize,
}

/// Why a record cannot be laid out within the sealed subset (fail closed).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LayoutError {
    /// No items, or the first item is not the `01` record.
    NotARecord,
    /// An elementary item's `PIC` is outside the sealed subset.
    Pic(pic::PicError),
    /// `REDEFINES <name>` names an item that is not a prior sibling at the same level.
    RedefinesTargetNotFound(String),
    /// The redefining item is larger than the item it redefines (deferred — record-growth case).
    RedefinesLarger { name: String, target: String },
    /// A group item carried a `PIC` (groups have no picture).
    GroupWithPic(String),
    /// An elementary item had no `PIC`.
    ElementaryNoPic(String),
    /// `OCCURS 0` or an absurd count.
    BadOccurs,
    /// Malformed level nesting (a child level not under any open group).
    BadNesting,
    /// An `OCCURS DEPENDING ON` form outside the sealed `GNURUST.10` subset: the ODO item is not the
    /// last in its group, there is more than one ODO in the record (incl. nested ODO), it is combined
    /// with `REDEFINES`, `max < min`/`max == 0`, or the `DEPENDING ON` item is not a field in the
    /// record. (Only single, trailing, physical-maximum ODO is admitted.)
    OdoUnsupported(String),
}

impl core::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LayoutError::NotARecord => write!(f, "empty record or missing 01 level"),
            LayoutError::Pic(e) => write!(f, "elementary PIC outside sealed subset: {e}"),
            LayoutError::RedefinesTargetNotFound(n) => {
                write!(f, "REDEFINES target '{n}' not a prior sibling")
            }
            LayoutError::RedefinesLarger { name, target } => {
                write!(f, "REDEFINES '{name}' is larger than '{target}' (deferred)")
            }
            LayoutError::GroupWithPic(n) => write!(f, "group item '{n}' has a PIC"),
            LayoutError::ElementaryNoPic(n) => write!(f, "elementary item '{n}' has no PIC"),
            LayoutError::BadOccurs => write!(f, "OCCURS 0 or absurd count"),
            LayoutError::BadNesting => write!(f, "malformed level nesting"),
            LayoutError::OdoUnsupported(e) => write!(f, "unsupported OCCURS DEPENDING ON: {e}"),
        }
    }
}

impl std::error::Error for LayoutError {}

// Internal tree node.
struct Node {
    item: Item,
    children: Vec<Node>,
}

/// Build the level-number tree from a flat item list. Items with a higher level number than the
/// current top become its children.
fn build_tree(items: &[Item]) -> Result<Node, LayoutError> {
    let mut it = items.iter();
    let root_item = it.next().ok_or(LayoutError::NotARecord)?.clone();
    if root_item.level != 1 {
        return Err(LayoutError::NotARecord);
    }
    let mut root = Node {
        item: root_item,
        children: Vec::new(),
    };
    // Stack of raw pointers is awkward in safe Rust; rebuild via indices into a flat arena.
    // Simpler: recursive insertion using a path of indices.
    let mut path: Vec<usize> = Vec::new(); // indices into children chains, identifying the open node
    for item in it {
        // Find insertion parent: the deepest open node whose level < item.level.
        // Walk down `path` from root, dropping entries whose node.level >= item.level.
        loop {
            let lvl = node_at(&root, &path).item.level;
            if !path.is_empty() && lvl >= item.level {
                path.pop();
            } else {
                break;
            }
        }
        let parent = node_at_mut(&mut root, &path);
        if parent.item.level >= item.level && !path.is_empty() {
            return Err(LayoutError::BadNesting);
        }
        parent.children.push(Node {
            item: item.clone(),
            children: Vec::new(),
        });
        let new_idx = parent.children.len() - 1;
        path.push(new_idx);
    }
    Ok(root)
}

fn node_at<'a>(root: &'a Node, path: &[usize]) -> &'a Node {
    let mut n = root;
    for &i in path {
        n = &n.children[i];
    }
    n
}
fn node_at_mut<'a>(root: &'a mut Node, path: &[usize]) -> &'a mut Node {
    let mut n = root;
    for &i in path {
        n = &mut n.children[i];
    }
    n
}

/// Lay out a record under the **default** dialect: assign every item its offset and one-occurrence size.
/// The `01` record's own entry (offset 0, size = total record length) is included in the result; the
/// list is in layout (children-before-group) order, so look entries up by name rather than by index.
pub fn lay_out(items: &[Item]) -> Result<Vec<Laid>, LayoutError> {
    lay_out_dialect(items, crate::dialect::Dialect::DEFAULT)
}

/// As [`lay_out`] but under an explicit [`Dialect`](crate::dialect::Dialect): with `complex-odo` a field
/// is permitted *after* an `OCCURS DEPENDING ON` table (the default dialect rejects it, as `cobc` does at
/// compile time). The static offsets are the table's **physical maximum** in either case; the runtime
/// slide (`odoslide`) is applied by [`record_used_length`], which the static `Laid` cannot express.
pub fn lay_out_dialect(
    items: &[Item],
    dialect: crate::dialect::Dialect,
) -> Result<Vec<Laid>, LayoutError> {
    // Global OCCURS DEPENDING ON rules (sealed subset: a single trailing physical-max ODO, optionally --
    // under complex-odo -- followed by fixed fields).
    let odo_items: Vec<&Item> = items.iter().filter(|i| i.odo.is_some()).collect();
    if odo_items.len() > 1 {
        return Err(LayoutError::OdoUnsupported(
            "more than one OCCURS DEPENDING ON (incl. nested) in the record".into(),
        ));
    }
    if let Some(it) = odo_items.first() {
        let o = it.odo.as_ref().unwrap();
        if o.max == 0 || o.max <= o.min {
            return Err(LayoutError::OdoUnsupported(format!(
                "{}: bad bounds {} TO {}",
                it.name, o.min, o.max
            )));
        }
        if it.redefines.is_some() || it.occurs.is_some() {
            return Err(LayoutError::OdoUnsupported(format!(
                "{}: ODO combined with REDEFINES/fixed OCCURS",
                it.name
            )));
        }
        if !items.iter().any(|i| i.name == o.depending_on) {
            return Err(LayoutError::OdoUnsupported(format!(
                "DEPENDING ON '{}' is not a field in the record",
                o.depending_on
            )));
        }
    }
    let root = build_tree(items)?;
    let mut out = Vec::new();
    let _ = compute(&root, 0, &mut out, dialect.complex_odo)?;
    Ok(out)
}

/// Compute `node`'s one-occurrence size at `base` offset, appending every item to `out`. `complex_odo`
/// permits a field after an ODO table (otherwise such a layout fails closed, matching the default dialect).
fn compute(
    node: &Node,
    base: usize,
    out: &mut Vec<Laid>,
    complex_odo: bool,
) -> Result<usize, LayoutError> {
    if node.item.occurs == Some(0) {
        return Err(LayoutError::BadOccurs);
    }

    let size = if node.children.is_empty() {
        // Elementary (or the record itself if it has a PIC — unusual but handled).
        let (pic_s, usage, sep, lead) = node
            .item
            .pic
            .as_ref()
            .ok_or_else(|| LayoutError::ElementaryNoPic(node.item.name.clone()))?;
        match pic::build_field(pic_s, *usage, *sep, *lead) {
            Ok(pf) => pf.size,
            // An edited DISPLAY picture has a well-defined width (`GNURUST.16`): size it via
            // `edited_size` so edited fields lay out (their decode is the `edited` court, not `pic`).
            Err(pe) if *usage == Usage::Display => {
                crate::edited::edited_size(pic_s).map_err(|_| LayoutError::Pic(pe))?
            }
            Err(pe) => return Err(LayoutError::Pic(pe)),
        }
    } else {
        // Group: lay out children in order; size = span of one occurrence.
        if node.item.pic.is_some() {
            return Err(LayoutError::GroupWithPic(node.item.name.clone()));
        }
        let mut cursor = base;
        // Track laid siblings (name -> (offset, total_span)) for REDEFINES resolution.
        let mut sibling: Vec<(String, usize, usize)> = Vec::new();
        let last_idx = node.children.len() - 1;
        for (idx, child) in node.children.iter().enumerate() {
            // OCCURS DEPENDING ON contributes its physical maximum; it must be the last item in its
            // group (GnuCOBOL forbids a field after an ODO item without complex-ODO config).
            let child_occurs = if let Some(o) = &child.item.odo {
                // A field after an ODO table is only permitted under complex-odo (ibm/mf/mvs); the default
                // dialect rejects it, exactly as `cobc` errors ("cannot have OCCURS DEPENDING because of
                // <field>"). The static offsets here are the physical maximum either way.
                if idx != last_idx && !complex_odo {
                    return Err(LayoutError::OdoUnsupported(format!(
                        "{} is not the last item in its group",
                        child.item.name
                    )));
                }
                o.max
            } else {
                child.item.occurs.unwrap_or(1)
            };
            if child_occurs == 0 {
                return Err(LayoutError::BadOccurs);
            }
            let child_base = if let Some(target) = &child.item.redefines {
                // REDEFINES over an ODO item is not admitted.
                if node
                    .children
                    .iter()
                    .any(|c| &c.item.name == target && c.item.odo.is_some())
                {
                    return Err(LayoutError::OdoUnsupported(format!(
                        "REDEFINES over ODO item '{target}'"
                    )));
                }
                sibling
                    .iter()
                    .find(|(n, _, _)| n == target)
                    .map(|(_, off, _)| *off)
                    .ok_or_else(|| LayoutError::RedefinesTargetNotFound(target.clone()))?
            } else {
                cursor
            };
            let one = compute(child, child_base, out, complex_odo)?;
            let span = one
                .checked_mul(child_occurs as usize)
                .ok_or(LayoutError::BadOccurs)?;
            if let Some(target) = &child.item.redefines {
                // Overlay: must not be larger than the redefined sibling's span (sealed subset).
                let tgt_span = sibling
                    .iter()
                    .find(|(n, _, _)| n == target)
                    .map(|(_, _, s)| *s)
                    .unwrap_or(0);
                if span > tgt_span {
                    return Err(LayoutError::RedefinesLarger {
                        name: child.item.name.clone(),
                        target: target.clone(),
                    });
                }
                // cursor unchanged (overlay)
            } else {
                cursor += span;
            }
            sibling.push((child.item.name.clone(), child_base, span));
        }
        cursor - base
    };

    out.push(Laid {
        name: node.item.name.clone(),
        level: node.item.level,
        offset: base,
        size,
    });
    Ok(size)
}

/// The runtime byte length of an `01` record given the live `DEPENDING ON` count -- the `LENGTH OF`
/// semantics across `-std=`. Without an ODO this is the static record size. With a single ODO table:
///   - the table contributes `count` occurrences (not `max`);
///   - any fields *after* it either **slide** to immediately follow the runtime-sized table (`odoslide`,
///     ibm/mvs) or stay at the table's physical-maximum position (mf), making the record's length
///     count-independent.
/// `count` is clamped to the ODO's `[min, max]`. Matches `cobc`'s `LENGTH OF` (ibm 7/9/11, mf 11/11/11 for
/// a `CNT(1) + OCCURS 1 TO 3 OF X(2) + TAIL X(4)` record at counts 1/2/3).
pub fn record_used_length(
    items: &[Item],
    dialect: crate::dialect::Dialect,
    count: u32,
) -> Result<usize, LayoutError> {
    let laid = lay_out_dialect(items, dialect)?;
    // The 01 record is the first source item (its laid entry spans the whole record).
    let record_name = &items.first().ok_or(LayoutError::NotARecord)?.name;
    let record_size = laid
        .iter()
        .find(|l| &l.name == record_name)
        .ok_or(LayoutError::NotARecord)?
        .size;
    let odo_item = match items.iter().find(|i| i.odo.is_some()) {
        Some(it) => it,
        None => return Ok(record_size), // fixed-length record
    };
    let o = odo_item.odo.as_ref().unwrap();
    let count = count.clamp(o.min, o.max);
    let odo_laid = laid
        .iter()
        .find(|l| l.name == odo_item.name)
        .ok_or(LayoutError::NotARecord)?;
    let elem = odo_laid.size; // one-occurrence size
    let odo_offset = odo_laid.offset; // bytes before the table (fixed prefix)
    // Bytes physically after the table at its physical maximum (the trailing fixed fields).
    let trailing = record_size.saturating_sub(odo_offset + (o.max as usize) * elem);
    if dialect.odoslide {
        // Trailing fields slide to follow the runtime-sized table.
        Ok(odo_offset + (count as usize) * elem + trailing)
    } else if trailing == 0 {
        // Simple trailing ODO (nothing after): LENGTH OF still shrinks with the count.
        Ok(odo_offset + (count as usize) * elem)
    } else {
        // mf: trailing fields stay at the physical-max position -> count-independent length.
        Ok(record_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edited_field_lays_out_via_edited_size() {
        // An edited DISPLAY picture sizes via GNURUST.16 edited_size (it would fail `pic::build_field`).
        let items = vec![
            group(1, "R"),
            elem(5, "A", "9(3)", Usage::Display),
            elem(5, "PRINT-BAL", "$ZZ,ZZ9.99", Usage::Display), // 10 chars
            elem(5, "B", "X(2)", Usage::Display),
        ];
        let laid = lay_out(&items).unwrap();
        let by = |n: &str| laid.iter().find(|l| l.name == n).unwrap();
        assert_eq!((by("PRINT-BAL").offset, by("PRINT-BAL").size), (3, 10));
        assert_eq!((by("B").offset, by("B").size), (13, 2));
    }

    fn elem(level: u16, name: &str, pic: &str, usage: Usage) -> Item {
        Item {
            level,
            name: name.into(),
            pic: Some((pic.into(), usage, false, false)),
            occurs: None,
            redefines: None,
            odo: None,
        }
    }
    fn group(level: u16, name: &str) -> Item {
        Item {
            level,
            name: name.into(),
            pic: None,
            occurs: None,
            redefines: None,
            odo: None,
        }
    }

    #[test]
    fn matches_oracle_record() {
        // The probed record: REC=29; A@0 B@3 C@8 FILLER@12 D@14 G@18(G1@18 G2@20) T@23 E@23.
        let mut items = vec![
            group(1, "REC"),
            elem(5, "A", "9(3)", Usage::Display),
            elem(5, "B", "X(5)", Usage::Display),
            elem(5, "C", "S9(4)V99", Usage::Comp3),
            elem(5, "FILLER", "X(2)", Usage::Display),
            elem(5, "D", "9(4)", Usage::Display),
            group(5, "G"),
            elem(10, "G1", "9(2)", Usage::Display),
            elem(10, "G2", "X(3)", Usage::Display),
        ];
        let mut t = elem(5, "T", "9(2)", Usage::Display);
        t.occurs = Some(3);
        items.push(t);
        let mut e = elem(5, "E", "X(6)", Usage::Display);
        e.redefines = Some("T".into());
        items.push(e);

        let laid = lay_out(&items).unwrap();
        let by = |n: &str| laid.iter().find(|l| l.name == n).unwrap();
        assert_eq!((by("REC").offset, by("REC").size), (0, 29));
        assert_eq!((by("A").offset, by("A").size), (0, 3));
        assert_eq!((by("B").offset, by("B").size), (3, 5));
        assert_eq!((by("C").offset, by("C").size), (8, 4));
        assert_eq!((by("D").offset, by("D").size), (14, 4));
        assert_eq!((by("G").offset, by("G").size), (18, 5));
        assert_eq!((by("G1").offset, by("G1").size), (18, 2));
        assert_eq!((by("G2").offset, by("G2").size), (20, 3));
        assert_eq!((by("T").offset, by("T").size), (23, 2)); // one occurrence
        assert_eq!((by("E").offset, by("E").size), (23, 6)); // overlays T's 6-byte span
    }

    #[test]
    fn redefines_larger_fails_closed() {
        let items = vec![group(1, "R"), elem(5, "A", "9(2)", Usage::Display), {
            let mut e = elem(5, "B", "X(5)", Usage::Display);
            e.redefines = Some("A".into());
            e
        }];
        assert!(matches!(
            lay_out(&items),
            Err(LayoutError::RedefinesLarger { .. })
        ));
    }

    fn odo(item: &mut Item, min: u32, max: u32, dep: &str) {
        item.odo = Some(Odo {
            min,
            max,
            depending_on: dep.into(),
        });
    }

    #[test]
    fn odo_physical_max_matches_oracle() {
        // REC: CNT(1) + ITEM OCCURS 0 TO 5 DEPENDING ON CNT PIC X -> physical 6.
        let mut item = elem(5, "ITEM", "X", Usage::Display);
        odo(&mut item, 0, 5, "CNT");
        let items = vec![group(1, "REC"), elem(5, "CNT", "9", Usage::Display), item];
        let laid = lay_out(&items).unwrap();
        let by = |n: &str| laid.iter().find(|l| l.name == n).unwrap();
        assert_eq!((by("REC").offset, by("REC").size), (0, 6)); // physical max
        assert_eq!((by("CNT").offset, by("CNT").size), (0, 1));
        assert_eq!((by("ITEM").offset, by("ITEM").size), (1, 1)); // one occurrence

        // REC2: N2(2) + GRP OCCURS 1 TO 4 DEPENDING ON N2 {A 9(3), B X(2)} -> 2 + 4*5 = 22.
        let mut grp = group(5, "GRP");
        odo(&mut grp, 1, 4, "N2");
        let items2 = vec![
            group(1, "REC2"),
            elem(5, "N2", "99", Usage::Display),
            grp,
            elem(10, "A", "9(3)", Usage::Display),
            elem(10, "B", "X(2)", Usage::Display),
        ];
        let laid2 = lay_out(&items2).unwrap();
        let by2 = |n: &str| laid2.iter().find(|l| l.name == n).unwrap();
        assert_eq!((by2("REC2").offset, by2("REC2").size), (0, 22));
        assert_eq!((by2("GRP").offset, by2("GRP").size), (2, 5)); // one occurrence span
    }

    #[test]
    fn odo_fails_closed() {
        // ODO not last in its group.
        let mut item = elem(5, "ITEM", "X", Usage::Display);
        odo(&mut item, 0, 5, "CNT");
        let not_last = vec![
            group(1, "REC"),
            elem(5, "CNT", "9", Usage::Display),
            item.clone(),
            elem(5, "TRAILER", "X(3)", Usage::Display),
        ];
        assert!(matches!(
            lay_out(&not_last),
            Err(LayoutError::OdoUnsupported(_))
        ));

        // DEPENDING ON a non-existent field.
        let mut bad_dep = elem(5, "ITEM", "X", Usage::Display);
        odo(&mut bad_dep, 0, 5, "NOPE");
        let dep = vec![
            group(1, "REC"),
            elem(5, "CNT", "9", Usage::Display),
            bad_dep,
        ];
        assert!(matches!(lay_out(&dep), Err(LayoutError::OdoUnsupported(_))));

        // Two ODOs in one record.
        let mut o1 = elem(5, "T1", "X", Usage::Display);
        odo(&mut o1, 0, 3, "CNT");
        let mut o2 = elem(5, "T2", "X", Usage::Display);
        odo(&mut o2, 0, 3, "CNT");
        let two = vec![group(1, "REC"), elem(5, "CNT", "9", Usage::Display), o1, o2];
        assert!(matches!(lay_out(&two), Err(LayoutError::OdoUnsupported(_))));
    }

    #[test]
    fn complex_odo_slide_matches_dialect_oracle() {
        use crate::dialect::Dialect;
        // Record: CNT PIC 9 (1) + T OCCURS 1 TO 3 OF X(2) + TAIL X(4). cobc LENGTH OF R at CNT=1/2/3:
        //   default COMPILE ERROR (field after ODO); ibm 7/9/11 (slides); mf 11/11/11 (physical max).
        let rec = || {
            let mut t = elem(5, "T", "X(2)", Usage::Display);
            odo(&mut t, 1, 3, "CNT");
            vec![
                group(1, "R"),
                elem(5, "CNT", "9", Usage::Display),
                t,
                elem(5, "TAIL", "X(4)", Usage::Display),
            ]
        };
        // default: a field after an ODO table is rejected (the cobc compile error).
        assert!(matches!(
            lay_out_dialect(&rec(), Dialect::DEFAULT),
            Err(LayoutError::OdoUnsupported(_))
        ));
        // ibm/mf accept it; the STATIC offsets are the physical maximum (TAIL at 1 + 3*2 = 7, size 11).
        for d in [Dialect::IBM, Dialect::MF] {
            let laid = lay_out_dialect(&rec(), d).unwrap();
            let by = |n: &str| laid.iter().find(|l| l.name == n).unwrap();
            assert_eq!((by("R").offset, by("R").size), (0, 11));
            assert_eq!((by("T").offset, by("T").size), (1, 2));
            assert_eq!(by("TAIL").offset, 7);
        }
        // runtime LENGTH OF: ibm SLIDES (odoslide), mf is physical-max (count-independent).
        assert_eq!(record_used_length(&rec(), Dialect::IBM, 1).unwrap(), 7);
        assert_eq!(record_used_length(&rec(), Dialect::IBM, 2).unwrap(), 9);
        assert_eq!(record_used_length(&rec(), Dialect::IBM, 3).unwrap(), 11);
        assert_eq!(record_used_length(&rec(), Dialect::MF, 1).unwrap(), 11);
        assert_eq!(record_used_length(&rec(), Dialect::MF, 3).unwrap(), 11);
        // A simple trailing ODO (nothing after) still shrinks with the count under any dialect.
        let mut t = elem(5, "T", "X(2)", Usage::Display);
        odo(&mut t, 1, 3, "CNT");
        let simple = vec![group(1, "R"), elem(5, "CNT", "9", Usage::Display), t];
        assert_eq!(record_used_length(&simple, Dialect::MF, 2).unwrap(), 5); // 1 + 2*2
        assert_eq!(record_used_length(&simple, Dialect::DEFAULT, 2).unwrap(), 5);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.4, GNURUST.10
    /// lay_out is total over a record whose elementary usage is symbolic: Ok(layout) or a typed LayoutError.
    #[kani::proof]
    #[kani::unwind(6)]
    fn layout_is_total() {
        let u: u8 = kani::any();
        let usage = match u % 3 { 0 => Usage::Display, 1 => Usage::Comp3, _ => Usage::Comp };
        let items = vec![
            Item { level: 1, name: "R".into(), pic: None, occurs: None, redefines: None, odo: None },
            Item { level: 5, name: "A".into(), pic: Some(("X(3)".into(), Usage::Display, false, false)), occurs: None, redefines: None, odo: None },
            Item { level: 5, name: "B".into(), pic: Some(("9(4)".into(), usage, false, false)), occurs: None, redefines: None, odo: None },
        ];
        let _ = lay_out(&items);
    }
}

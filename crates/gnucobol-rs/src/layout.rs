//! DATA DIVISION record layout (`GNURUST.4`): assign each item its byte **offset** and **size**
//! within an `01` record, handling level-numbered groups, fixed `OCCURS n TIMES`, `REDEFINES`
//! overlay, and `FILLER`, matching the GnuCOBOL compiler's own record-layout decision (verified
//! against `cobc`'s emitted field offsets and runtime `LENGTH OF`).
//!
//! **Sealed subset:** level-numbered groups + elementary items whose `PIC` is in the sealed
//! [`crate::pic`] subset; fixed `OCCURS n TIMES`; `REDEFINES <name>` where the redefining item is
//! **no larger** than the item it redefines; `FILLER`. Everything else **fails closed** with a
//! typed [`LayoutError`] — in particular `OCCURS DEPENDING ON`, `SYNCHRONIZED` alignment, and a
//! `REDEFINES` larger than its target are deferred future courts, not silently mis-laid.

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
    /// Fixed `OCCURS n TIMES` (the sealed subset; `OCCURS DEPENDING ON` is rejected upstream).
    pub occurs: Option<u32>,
    /// `REDEFINES <name>` target, if any.
    pub redefines: Option<String>,
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

/// Lay out a record: assign every item its offset and one-occurrence size. The first element of the
/// result is the `01` record itself (offset 0, size = total record length).
pub fn lay_out(items: &[Item]) -> Result<Vec<Laid>, LayoutError> {
    let root = build_tree(items)?;
    let mut out = Vec::new();
    let _ = compute(&root, 0, &mut out)?;
    Ok(out)
}

/// Compute `node`'s one-occurrence size at `base` offset, appending every item to `out`.
fn compute(node: &Node, base: usize, out: &mut Vec<Laid>) -> Result<usize, LayoutError> {
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
        let pf = pic::build_field(pic_s, *usage, *sep, *lead).map_err(LayoutError::Pic)?;
        pf.size
    } else {
        // Group: lay out children in order; size = span of one occurrence.
        if node.item.pic.is_some() {
            return Err(LayoutError::GroupWithPic(node.item.name.clone()));
        }
        let mut cursor = base;
        // Track laid siblings (name -> (offset, total_span)) for REDEFINES resolution.
        let mut sibling: Vec<(String, usize, usize)> = Vec::new();
        for child in &node.children {
            let child_occurs = child.item.occurs.unwrap_or(1);
            if child_occurs == 0 {
                return Err(LayoutError::BadOccurs);
            }
            let child_base = if let Some(target) = &child.item.redefines {
                sibling
                    .iter()
                    .find(|(n, _, _)| n == target)
                    .map(|(_, off, _)| *off)
                    .ok_or_else(|| LayoutError::RedefinesTargetNotFound(target.clone()))?
            } else {
                cursor
            };
            let one = compute(child, child_base, out)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn elem(level: u16, name: &str, pic: &str, usage: Usage) -> Item {
        Item {
            level,
            name: name.into(),
            pic: Some((pic.into(), usage, false, false)),
            occurs: None,
            redefines: None,
        }
    }
    fn group(level: u16, name: &str) -> Item {
        Item {
            level,
            name: name.into(),
            pic: None,
            occurs: None,
            redefines: None,
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
}

//! Repo-root resolution + the canonical paths the port index reads and writes.

use std::path::{Path, PathBuf};

/// The admitted GnuCOBOL 3.2 libcob source tree (gitignored; extracted from the pinned tarball).
pub const LIBCOB_REL: &str = "lab/admit/gnucobol-3.2/libcob";
/// The Rust port library source.
pub const RUST_SRC_REL: &str = "crates/gnucobol-rs/src";
/// Where the machine indexes are written.
pub const PORT_INDEX_DIR: &str = "reports/port-index";

/// The 13 libcob translation units, in port order (kept in sync with the campaign).
pub const FILES: [&str; 13] = [
    "numeric.c", "move.c", "strings.c", "intrinsic.c", "cconv.c", "termio.c", "screenio.c", "call.c",
    "fileio.c", "mlio.c", "reportio.c", "common.c", "cobgetopt.c",
];

/// Resolve the repo root: `GNURUST_ROOT` if set, else the current directory.
pub fn root() -> PathBuf {
    std::env::var("GNURUST_ROOT").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."))
}

pub fn libcob_dir(root: &Path) -> PathBuf {
    root.join(LIBCOB_REL)
}

pub fn rust_src_dir(root: &Path) -> PathBuf {
    root.join(RUST_SRC_REL)
}

/// `true` when the admitted libcob source is extracted (the indexers are source-gated on this).
pub fn libcob_present(root: &Path) -> bool {
    libcob_dir(root).join("numeric.c").exists()
}

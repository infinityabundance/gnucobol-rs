//! Filesystem copybook resolver for the `cobc-rs` adapter: resolves `COPY name.` / `COPY name IN
//! "dir".` against the current directory, `-I` paths, and `COB_COPY_DIR` (GnuCOBOL's environment
//! copybook root), trying the plain name and each `-ext=` extension. Case-sensitive on the
//! filesystem (the suite's copybooks are written in the case cobc searches with).

use gnucobol_rs::copybook::CopyResolver;
use std::path::{Path, PathBuf};

pub struct FsCopyResolver {
    pub include_dirs: Vec<PathBuf>,
    pub extensions: Vec<String>,
}

impl FsCopyResolver {
    pub fn new(include_dirs: Vec<PathBuf>, extensions: Vec<String>) -> Self {
        // GnuCOBOL's default copybook extension is `.cpy` (COB_COPY_EXT default); when no -ext was
        // given, the resolver still tries the plain name and `<name>.cpy`.
        let extensions = if extensions.is_empty() {
            vec!["cpy".to_string()]
        } else {
            extensions
        };
        FsCopyResolver {
            include_dirs,
            extensions,
        }
    }

    /// The ordered search roots: cwd first, then -I dirs, then COB_COPY_DIR, then the candidate's
    /// system copybook directory (`gnucobol-rs/copy/`, the interpreted analogue of
    /// `$prefix/share/gnucobol/copy`). The system root is searched last so user copybooks always win.
    pub fn roots(&self) -> Vec<PathBuf> {
        let mut roots = vec![PathBuf::from(".")];
        roots.extend(self.include_dirs.iter().cloned());
        if let Ok(dir) = std::env::var("COB_COPY_DIR") {
            if !dir.is_empty() {
                roots.push(PathBuf::from(dir));
            }
        }
        roots.push(gnucobol_rs::copybook::system_copy_dir());
        roots
    }

    fn search(&self, name: &str, dir: Option<&str>) -> Option<String> {
        let mut candidates: Vec<PathBuf> = Vec::new();
        for root in self.roots() {
            let base = match dir {
                Some(d) => root.join(d),
                None => root.clone(),
            };
            candidates.push(base.join(name));
            for ext in &self.extensions {
                candidates.push(base.join(format!("{name}.{ext}")));
            }
        }
        for cand in candidates {
            if let Ok(bytes) = std::fs::read(&cand) {
                if let Ok(text) = String::from_utf8(bytes) {
                    return Some(text);
                }
            }
        }
        None
    }
}

impl CopyResolver for FsCopyResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        self.search(name, None)
    }

    fn resolve_in(&self, name: &str, dir: &str) -> Option<String> {
        if dir.is_empty() {
            return self.search(name, None);
        }
        self.search(name, Some(dir))
    }
}

/// The resolved copybook file paths for dependency generation (-MF/-MT): returns every copybook
/// actually consumed, in expansion order. This walks the source the same way `expand` does, but
/// only to collect file paths — done cheaply by re-running expansion with a path-recording
/// resolver.
pub fn collect_deps(source: &str, resolver: &FsCopyResolver) -> Vec<PathBuf> {
    let recorder = DepRecorder {
        resolver,
        deps: std::cell::RefCell::new(Vec::new()),
        seen: std::cell::RefCell::new(std::collections::HashSet::new()),
    };
    let _ = gnucobol_rs::copybook::expand(source, &recorder);
    recorder.deps.into_inner()
}

struct DepRecorder<'a> {
    resolver: &'a FsCopyResolver,
    deps: std::cell::RefCell<Vec<PathBuf>>,
    seen: std::cell::RefCell<std::collections::HashSet<String>>,
}

impl CopyResolver for DepRecorder<'_> {
    fn resolve(&self, name: &str) -> Option<String> {
        self.record(name, None)
    }
    fn resolve_in(&self, name: &str, dir: &str) -> Option<String> {
        self.record(name, Some(dir))
    }
}

impl DepRecorder<'_> {
    fn record(&self, name: &str, dir: Option<&str>) -> Option<String> {
        // find the actual file the Fs resolver would open (mirror the search order)
        let mut candidates: Vec<PathBuf> = Vec::new();
        for root in self.resolver.roots() {
            let base = match dir {
                Some(d) => root.join(d),
                None => root.clone(),
            };
            candidates.push(base.join(name));
            for ext in &self.resolver.extensions {
                candidates.push(base.join(format!("{name}.{ext}")));
            }
        }
        for cand in candidates {
            if cand.is_file() {
                let key = cand.to_string_lossy().into_owned();
                if self.seen.borrow_mut().insert(key.clone()) {
                    self.deps.borrow_mut().push(cand.clone());
                }
                return std::fs::read_to_string(&cand).ok();
            }
        }
        None
    }
}

/// Options for the dependency-file writer (upstream 49da19a3d's -MP / -MQ / -fcopybook-deps).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DepfileOpts {
    /// `-MP`: emit a phony target for every dependency.
    pub phony: bool,
    /// `-MQ`: Makefile-quote the targets (spaces -> `\ `, `$` -> `$$`).
    pub quote_targets: bool,
    /// `-fcopybook-deps`: list the COPYBOOK names only, omitting the main source.
    pub copybook_only: bool,
}

/// Makefile-quote a target: `$` -> `$$`, a space outside `$()` -> `\ ` (the upstream
/// `quote_dependencies` behaviour for -MQ).
pub fn makefile_quote(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '$' => out.push_str("$$"),
            ' ' | '\t' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Write the make-style dependency file for `-MF` (GnuCOBOL shape): each -MT target (or the
/// default `-o` output) followed by `:` and the space-joined dependency paths.
pub fn write_depfile(
    path: &Path,
    targets: &[String],
    deps: &[PathBuf],
    main_source: &Path,
    opts: DepfileOpts,
) -> Result<(), String> {
    let mut line = String::new();
    let emit = |s: &str, out: &mut String| {
        if opts.quote_targets {
            out.push_str(&makefile_quote(s));
        } else {
            out.push_str(s);
        }
    };
    if targets.is_empty() {
        emit(&main_source.to_string_lossy(), &mut line);
    } else {
        let joined = targets.join(" ");
        emit(&joined, &mut line);
    }
    line.push(':');
    for d in deps {
        line.push(' ');
        line.push_str(&d.to_string_lossy());
    }
    if !opts.copybook_only {
        // the main source is a dependency of the targets too
        line.push(' ');
        line.push_str(&main_source.to_string_lossy());
    }
    line.push('\n');
    if opts.phony {
        for d in deps {
            line.push_str(&d.to_string_lossy());
            line.push_str(":\n");
        }
        if !opts.copybook_only {
            line.push_str(&main_source.to_string_lossy());
            line.push_str(":\n");
        }
    }
    // atomically write
    let tmp = path.with_extension("cobr-dep.tmp");
    std::fs::write(&tmp, &line).map_err(|e| format!("write depfile: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename depfile: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_copy_dir_resolves_upstream_system_copybooks() {
        // The candidate's system copybook root (gnucobol-rs/copy/, custody-gated to upstream
        // a51ca02a68d5 + the pinned head) must resolve the system copybooks GnuCOBOL ships, so
        // programs that `COPY "screenio.cpy"` / `COPY sqlca` etc. can be preprocessed.
        let resolver = FsCopyResolver::new(vec![], vec![]);
        let sys = gnucobol_rs::copybook::system_copy_dir();
        assert!(sys.is_dir(), "system copy dir {sys:?} exists");
        for name in [
            "screenio.cpy",
            "sqlca.cpy",
            "sqlda.cpy",
            "xfhfcd.cpy",
            "xfhfcd3.cpy",
            "gcwindow.cpy",
        ] {
            assert!(sys.join(name).is_file(), "system copybook {name} admitted");
        }
        let text = resolver
            .resolve("screenio")
            .expect("screenio.cpy resolves via the system root");
        assert!(text.contains("SCREEN SECTION") || text.to_ascii_uppercase().contains("SCREEN"));
        // user roots still win: a shadowing cwd copybook is found first
        let tmp = std::env::temp_dir().join(format!("cobc_rs_syscopy_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("shadow.cpy"), b"user shadow\n").unwrap();
        let res2 = FsCopyResolver::new(vec![tmp.clone()], vec![]);
        assert_eq!(res2.resolve("shadow").as_deref(), Some("user shadow\n"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

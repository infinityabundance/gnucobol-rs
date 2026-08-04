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

    /// The ordered search roots: cwd first, then -I dirs, then COB_COPY_DIR.
    pub fn roots(&self) -> Vec<PathBuf> {
        let mut roots = vec![PathBuf::from(".")];
        roots.extend(self.include_dirs.iter().cloned());
        if let Ok(dir) = std::env::var("COB_COPY_DIR") {
            if !dir.is_empty() {
                roots.push(PathBuf::from(dir));
            }
        }
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

/// Write the make-style dependency file for `-MF` (GnuCOBOL shape): each -MT target (or the
/// default `-o` output) followed by `:` and the space-joined dependency paths.
pub fn write_depfile(
    path: &Path,
    targets: &[String],
    deps: &[PathBuf],
    main_source: &Path,
) -> Result<(), String> {
    let mut line = String::new();
    if targets.is_empty() {
        line.push_str(&main_source.to_string_lossy());
    } else {
        line.push_str(&targets.join(" "));
    }
    line.push(':');
    for d in deps {
        line.push(' ');
        line.push_str(&d.to_string_lossy());
    }
    // the main source is a dependency of the targets too
    line.push(' ');
    line.push_str(&main_source.to_string_lossy());
    line.push('\n');
    // atomically write
    let tmp = path.with_extension("cobr-dep.tmp");
    std::fs::write(&tmp, &line).map_err(|e| format!("write depfile: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename depfile: {e}"))?;
    Ok(())
}

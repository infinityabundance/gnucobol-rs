//! Candidate phase measurement for suite step packages.
//!
//! Each valid package is probed phase by phase (`preprocess`, `lex`, `parse`, `resolution`,
//! `layout`, `check`, `prepare`, `execute`) through `gnucobol-rs::frontend::probe_phases`,
//! with COPY expansion resolved against the package's own directory (the same filesystem search
//! the suite uses). The first failing phase is the package's first-failure classification;
//! nothing here is inferred from diagnostic text.

use crate::extract::package::StepPackage;
use gnucobol_rs::copybook::{self, CopyResolver};
use gnucobol_rs::dialect::Dialect;
use gnucobol_rs::frontend::probe_phases;
use std::path::{Path, PathBuf};

/// Filesystem copybook resolver rooted at the package dir (cwd analogue), with the candidate's
/// system copybooks as the last resort -- the same order GnuCOBOL searches.
struct DirCopyResolver {
    root: PathBuf,
    system: PathBuf,
}

impl CopyResolver for DirCopyResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        for base in [&self.root, &self.system] {
            let plain = base.join(name);
            if plain.is_file() {
                if let Ok(s) = std::fs::read_to_string(&plain) {
                    return Some(s);
                }
            }
            let with_ext = base.join(format!("{name}.cpy"));
            if with_ext.is_file() {
                if let Ok(s) = std::fs::read_to_string(&with_ext) {
                    return Some(s);
                }
            }
        }
        None
    }
    fn resolve_in(&self, name: &str, dir: &str) -> Option<String> {
        let d = self.root.join(dir);
        let p = d.join(name);
        if p.is_file() {
            if let Ok(s) = std::fs::read_to_string(&p) {
                return Some(s);
            }
        }
        let p = d.join(format!("{name}.cpy"));
        if p.is_file() {
            if let Ok(s) = std::fs::read_to_string(&p) {
                return Some(s);
            }
        }
        self.resolve(name)
    }
}

/// One phase outcome in corpus vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PhaseOutcome {
    pub phase: String,
    pub ok: bool,
    pub diagnostic: String,
}

/// The main-file bytes of a package (the `prog.cob`-shaped file; the suite's main files are
/// named `prog.cob` / `prog2.cob` / ...).
pub fn main_source(pkg: &StepPackage) -> Option<(String, String)> {
    pkg.files
        .iter()
        .find(|(name, _)| {
            name.ends_with(".cob") && !name.contains("expout") && !name.contains("experr")
        })
        .cloned()
}

/// Probe the candidate on a package directory directly (used by the bounded `probe-step`
/// subprocess and by embedders). `run` requests the prepare+execute phases.
pub fn probe_dir(group_dir: &Path, main_file: &str, run: bool) -> Vec<PhaseOutcome> {
    let path = group_dir.join(main_file);
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            return vec![PhaseOutcome {
                phase: "preprocess".to_string(),
                ok: false,
                diagnostic: format!("cannot read {}: {e}", path.display()),
            }]
        }
    };
    let resolver = DirCopyResolver {
        root: group_dir.to_path_buf(),
        system: copybook::system_copy_dir(),
    };
    let expanded = match copybook::expand(&source, &resolver) {
        Ok(e) => e.text(),
        Err(e) => {
            return vec![PhaseOutcome {
                phase: "preprocess".to_string(),
                ok: false,
                diagnostic: format!("copybook expansion failed: {e}"),
            }]
        }
    };
    probe_phases(&expanded, Dialect::DEFAULT, run)
        .into_iter()
        .map(|p| PhaseOutcome {
            phase: {
                let corpus = match p.phase.as_str() {
                    "execute" => "run",
                    other => other,
                };
                corpus.to_string()
            },
            ok: p.ok,
            diagnostic: p.diagnostic,
        })
        .collect()
}

/// Whether a step's shape requests execution (the run phases may execute the program, so they
/// are only probed for run-shaped steps).
pub fn run_shape(expanded_command: &str) -> bool {
    let c = expanded_command.trim();
    c.starts_with("./") || c.starts_with("cobcrun ") || c.starts_with(".\\")
}

/// Probe the candidate on a package. `workdir` must contain the package files (COPY resolution
/// uses it). Returns the phase probes; `first_failure` is the first non-ok probe.
pub fn probe_candidate(pkg: &StepPackage, workdir: &Path) -> Vec<PhaseOutcome> {
    let Some((name, _)) = main_source(pkg) else {
        return vec![PhaseOutcome {
            phase: "preprocess".to_string(),
            ok: false,
            diagnostic: "no COBOL main source in the package".to_string(),
        }];
    };
    probe_dir(workdir, &name, run_shape(&pkg.expanded_command))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver_finds_copybook_in_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("book.cpy"), b"01 X PIC 9.").unwrap();
        let r = DirCopyResolver {
            root: dir.path().to_path_buf(),
            system: PathBuf::from("/nonexistent"),
        };
        assert!(r.resolve("book").unwrap().contains("01 X"));
        assert!(r.resolve("missing").is_none());
    }
}

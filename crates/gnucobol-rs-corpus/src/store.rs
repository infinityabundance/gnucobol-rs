//! Content-addressed corpus store beneath `GNURUST_COBOL_CORPUS_ROOT` (XDG fallback).
//!
//! Addresses archives, git bundles, source files, copybooks, input data, generated data, oracle
//! and candidate binaries, and expected outputs by cryptographic hash. Every admitted source is
//! reproducible from origin + immutable revision + expected hash + extraction rules + licence
//! decision; hash mismatches are rejected.

use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const ENV_ROOT: &str = "GNURUST_COBOL_CORPUS_ROOT";
pub const DEFAULT_SUBDIR: &str = "gnucobol-rs-corpus";

/// Compute the SHA-256 hex of `bytes` (the content address used everywhere in the store).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex(&h.finalize())
}

pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// The corpus store. All large, mutable, downloaded, extracted, compiled and generated corpus
/// data lives beneath `root`; the repository only carries manifests, hashes, licences, small
/// admitted fixtures, summaries, patches and evidence metadata.
#[derive(Debug, Clone)]
pub struct CorpusStore {
    root: PathBuf,
}

impl CorpusStore {
    /// Open a store rooted at an explicit path (creating the layout directories).
    /// Tests and embedders use this directly so no process-wide environment is shared.
    pub fn open_at(root: &Path) -> std::io::Result<CorpusStore> {
        let store = CorpusStore {
            root: root.to_path_buf(),
        };
        for sub in [
            "blobs",
            "manifests",
            "origins",
            "licences",
            "packages",
            "expected",
            "evidence",
            "raw",
        ] {
            std::fs::create_dir_all(store.root.join(sub))?;
        }
        Ok(store)
    }

    /// Open (creating) the store. Root resolution order:
    /// 1. `GNURUST_COBOL_CORPUS_ROOT` when set;
    /// 2. `XDG_DATA_HOME` (or `~/.local/share`) + `gnucobol-rs-corpus`.
    pub fn open() -> std::io::Result<CorpusStore> {
        let root = match std::env::var_os(ENV_ROOT) {
            Some(r) if !r.is_empty() => PathBuf::from(r),
            _ => {
                let base = std::env::var_os("XDG_DATA_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        std::env::var_os("HOME")
                            .map(|h| PathBuf::from(h).join(".local").join("share"))
                            .unwrap_or_else(|| PathBuf::from("."))
                    });
                base.join(DEFAULT_SUBDIR)
            }
        };
        CorpusStore::open_at(&root)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn blob_path(&self, sha: &str) -> PathBuf {
        self.root.join("blobs").join(sha)
    }

    /// Store bytes content-addressed; returns the SHA-256. Idempotent.
    pub fn put_bytes(&self, bytes: &[u8]) -> std::io::Result<String> {
        let sha = sha256_hex(bytes);
        let path = self.blob_path(&sha);
        if path.exists() {
            return Ok(sha);
        }
        let mut f = std::fs::File::create(&path)?;
        f.write_all(bytes)?;
        Ok(sha)
    }

    /// Read a blob by SHA-256. `None` when absent.
    pub fn get_bytes(&self, sha: &str) -> Option<Vec<u8>> {
        std::fs::read(self.blob_path(sha)).ok()
    }

    /// Verify `bytes` against an expected SHA-256. Returns an error on mismatch (never silently
    /// accepts corrupted content).
    pub fn verify(&self, bytes: &[u8], expected: &str) -> Result<(), String> {
        let actual = sha256_hex(bytes);
        if actual == expected {
            Ok(())
        } else {
            Err(format!(
                "hash mismatch: expected {expected}, got {actual} ({} bytes)",
                bytes.len()
            ))
        }
    }

    /// Put a file from disk content-addressed (reads its bytes). Rejects (leaves the store
    /// untouched) when the file does not match `expected_sha`, when given.
    pub fn put_file(&self, path: &Path, expected_sha: Option<&str>) -> Result<String, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        if let Some(exp) = expected_sha {
            self.verify(&bytes, exp)?;
        }
        self.put_bytes(&bytes).map_err(|e| e.to_string())
    }

    /// Materialize a blob to a destination path.
    pub fn materialize(&self, sha: &str, dest: &Path) -> Result<(), String> {
        let bytes = self
            .get_bytes(sha)
            .ok_or_else(|| format!("blob {sha} absent from the store"))?;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(dest, &bytes).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_round_trip_and_address() {
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        let sha = store.put_bytes(b"hello corpus").unwrap();
        assert_eq!(sha, sha256_hex(b"hello corpus"));
        assert_eq!(store.get_bytes(&sha).unwrap(), b"hello corpus");
        // idempotent
        assert_eq!(store.put_bytes(b"hello corpus").unwrap(), sha);
        // layout dirs exist
        assert!(store.root().join("blobs").is_dir());
        assert!(store.root().join("manifests").is_dir());
    }

    #[test]
    fn verify_rejects_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        assert!(store.verify(b"abc", &"0".repeat(64)).is_err());
        assert!(store.verify(b"abc", &sha256_hex(b"abc")).is_ok());
    }

    #[test]
    fn put_file_checks_expected_hash() {
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        let f = dir.path().join("src.cob");
        std::fs::write(&f, b"IDENTIFICATION DIVISION.").unwrap();
        let good = sha256_hex(b"IDENTIFICATION DIVISION.");
        assert!(store.put_file(&f, Some(&good)).is_ok());
        assert!(store.put_file(&f, Some(&"0".repeat(64))).is_err());
    }

    #[test]
    fn materialize_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        let sha = store.put_bytes(b"zz").unwrap();
        let out = dir.path().join("sub").join("out.bin");
        store.materialize(&sha, &out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"zz");
        assert!(store.materialize(&"0".repeat(64), &out).is_err());
    }

    #[test]
    fn xdg_fallback_root_is_beneath_data_home() {
        let dir = tempfile::tempdir().unwrap();
        std::env::remove_var(ENV_ROOT);
        std::env::set_var("XDG_DATA_HOME", dir.path());
        let store = CorpusStore::open().unwrap();
        assert_eq!(store.root(), dir.path().join(DEFAULT_SUBDIR).as_path());
        std::env::remove_var("XDG_DATA_HOME");
    }
}

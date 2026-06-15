//! Port of the **allocator + string-utility** layer of `libcob/common.c`.
//!
//! The C functions in this group (`cob_malloc`, `cob_fast_malloc`, `cob_realloc`, `cob_free`, the
//! `cob_cache_*` family, and the `cob_strdup` / `cob_strndup` / `cob_strcat` / `cob_strjoin` helpers)
//! are thin wrappers over C's `malloc`/`calloc`/`realloc`/`free` plus `strlen`/`memcpy`/`sprintf`.
//!
//! In a `#![forbid(unsafe_code)]` Rust port there are no raw pointers: an "allocation" is a `Vec<u8>`
//! whose lifetime is governed by RAII. We therefore model these as **pure functions over `&[u8]` /
//! `Vec<u8>`**:
//!
//! * Allocators return a fresh `Vec<u8>` (`cob_malloc` zeroes it -- it wraps `calloc`).
//! * `cob_free` / `cob_cache_free` are the Rust drop: they **consume** the value and return `()`.
//!   In C they call `free`; in Rust dropping the owned `Vec` is the faithful equivalent.
//! * The C `cob_strcat` writes through a `static` `strbuff` buffer (and optionally frees an input).
//!   That mutable-static caching is a runtime side effect, not part of the string math; we port the
//!   **pure concatenation** (both inputs as slices, fresh `Vec<u8>` out). The `strbuff`/`str_to_free`
//!   bookkeeping is a declared non-claim of this module.
//! * The `cob_cache_*` allocator threads a global `cob_alloc_base` linked list so the runtime can
//!   bulk-free cached blocks; that global registry is a runtime side effect (a declared non-claim).
//!   The byte behaviour seen by a caller -- zeroed alloc, grow-preserves-prefix realloc -- is what we
//!   port, matching the plain `cob_malloc`/`cob_realloc` byte semantics.

#![forbid(unsafe_code)]

/// Port of `common.c:cob_fast_malloc` -- a bare `malloc(size)` wrapper (uninitialised memory in C).
///
/// Rust has no uninitialised allocation under `forbid(unsafe_code)`; we return a fresh `Vec<u8>` of
/// `size` zero bytes. Callers that immediately overwrite every byte (as the C callers do) observe no
/// difference; the only non-claim is the value of bytes left unwritten.
pub fn cob_fast_malloc(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

/// Port of `common.c:cob_malloc` -- `calloc(1, size)`, i.e. an allocation of `size` **zeroed** bytes.
pub fn cob_malloc(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

/// Port of `common.c:cob_realloc` -- resize `optr` (current logical length `osize`) to `nsize`.
///
/// C behaviour: same size returns the block unchanged; shrinking truncates; growing allocates new
/// zeroed memory and copies the old `osize` bytes into the front (the rest stays zero). We mirror that
/// on the owned `Vec`: `osize` is the count of live bytes to preserve, capped at the input length.
pub fn cob_realloc(optr: Vec<u8>, osize: usize, nsize: usize) -> Vec<u8> {
    if osize == nsize {
        return optr;
    }
    let live = if osize > optr.len() { optr.len() } else { osize };
    if osize > nsize {
        // Reducing size: keep the first nsize bytes.
        let mut out = optr;
        out.truncate(nsize);
        return out;
    }
    // Growing: new zeroed block, copy the old osize bytes into the front.
    let mut out = vec![0u8; nsize];
    let keep = if live > nsize { nsize } else { live };
    out[..keep].copy_from_slice(&optr[..keep]);
    out
}

/// Port of `common.c:cob_free` -- `free(mptr)`.
///
/// In Rust freeing is dropping: this consumes the owned allocation and returns `()`. Calling it is the
/// 1:1 analogue of the C `free`; the storage is reclaimed when the moved-in `Vec` is dropped here.
pub fn cob_free(mptr: Vec<u8>) {
    drop(mptr);
}

/// Port of `common.c:cob_strdup` -- duplicate a NUL-terminated string into fresh memory.
///
/// C copies `strlen(p) + 1` bytes (the terminating NUL included). We take the string body as a slice
/// (no embedded NUL expected, matching `strlen`) and return a fresh `Vec<u8>` ending in a `0x00` byte.
pub fn cob_strdup(p: &[u8]) -> Vec<u8> {
    let len = strlen(p);
    let mut out = Vec::with_capacity(len + 1);
    out.extend_from_slice(&p[..len]);
    out.push(0u8);
    out
}

/// Port of `common.c:cob_strndup` -- duplicate at most `max_len` bytes of `p`, then append a NUL.
///
/// C copies `min(strlen(p), max_len)` bytes and writes a terminating `0x00`.
pub fn cob_strndup(p: &[u8], max_len: usize) -> Vec<u8> {
    let mut len = strlen(p);
    if len > max_len {
        len = max_len;
    }
    let mut out = Vec::with_capacity(len + 1);
    out.extend_from_slice(&p[..len]);
    out.push(0u8);
    out
}

/// Port of `common.c:cob_strcat` -- concatenate `str1` and `str2` into a fresh allocation.
///
/// C computes `strlen(str1) + strlen(str2) + 1` and `sprintf`s `"%s%s"` into a `strbuff` static buffer,
/// optionally freeing one input (the `str_to_free` flag) when the result is reassigned to it. The
/// `strbuff` caching and the input-freeing are runtime side effects (a declared non-claim); we port the
/// pure string math: the bytes of `str1` up to its NUL, then `str2` up to its NUL, then a trailing NUL.
pub fn cob_strcat(str1: &[u8], str2: &[u8]) -> Vec<u8> {
    let l1 = strlen(str1);
    let l2 = strlen(str2);
    let mut out = Vec::with_capacity(l1 + l2 + 1);
    out.extend_from_slice(&str1[..l1]);
    out.extend_from_slice(&str2[..l2]);
    out.push(0u8);
    out
}

/// Port of `common.c:cob_strjoin` -- join `strarray` with `separator` between each element.
///
/// C returns `NULL` when the array is empty/`NULL` or the separator is `NULL`; it `strdup`s the first
/// element then repeatedly `strcat`s separator + next element. We return `None` for an empty array and
/// otherwise a fresh NUL-terminated `Vec<u8>` of `elem0 sep elem1 sep ... elemN` with one trailing NUL.
pub fn cob_strjoin(strarray: &[&[u8]], separator: &[u8]) -> Option<Vec<u8>> {
    if strarray.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    let first = strarray[0];
    out.extend_from_slice(&first[..strlen(first)]);
    let sl = strlen(separator);
    for &elem in &strarray[1..] {
        out.extend_from_slice(&separator[..sl]);
        out.extend_from_slice(&elem[..strlen(elem)]);
    }
    out.push(0u8);
    Some(out)
}

/// Port of `common.c:cob_cache_malloc` -- the caching allocator's `malloc`.
///
/// In C this `cob_malloc`s the block (zeroed) and registers it on the global `cob_alloc_base` list so
/// the runtime can bulk-free it later. The global registry is a runtime side effect (a declared
/// non-claim); the byte result a caller sees is exactly a zeroed block, so we port it to [`cob_malloc`].
pub fn cob_cache_malloc(size: usize) -> Vec<u8> {
    cob_malloc(size)
}

/// Port of `common.c:cob_cache_realloc` -- the caching allocator's `realloc`.
///
/// In C: a `NULL` pointer routes to `cob_cache_malloc`; otherwise, for a registered block, a request no
/// larger than the current size returns it unchanged, and a larger request allocates a new zeroed block,
/// copies the old bytes over, and updates the registry. The cache lookup/registry is a runtime side
/// effect (a declared non-claim); the byte semantics for the grow case match [`cob_realloc`].
pub fn cob_cache_realloc(ptr: Vec<u8>, size: usize) -> Vec<u8> {
    let osize = ptr.len();
    if size <= osize {
        // C returns the block unchanged when the request fits the current size.
        return ptr;
    }
    cob_realloc(ptr, osize, size)
}

/// Port of `common.c:cob_cache_free` -- the caching allocator's `free`.
///
/// In C this `free`s the registered block and unlinks it from `cob_alloc_base`. The unlinking is a
/// runtime side effect (a declared non-claim); freeing is dropping the owned `Vec`, as in [`cob_free`].
pub fn cob_cache_free(ptr: Vec<u8>) {
    drop(ptr);
}

/// C `strlen`: length up to (not including) the first `0x00` byte, or the whole slice if none.
fn strlen(p: &[u8]) -> usize {
    match p.iter().position(|&b| b == 0u8) {
        Some(i) => i,
        None => p.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_malloc_zeroed_len() {
        let m = cob_fast_malloc(5);
        assert_eq!(m, vec![0u8; 5]);
        assert_eq!(cob_fast_malloc(0), Vec::<u8>::new());
    }

    #[test]
    fn malloc_zeroes() {
        assert_eq!(cob_malloc(3), vec![0u8, 0u8, 0u8]);
    }

    #[test]
    fn realloc_same_size_unchanged() {
        let v = vec![1u8, 2u8, 3u8];
        assert_eq!(cob_realloc(v, 3, 3), vec![1u8, 2u8, 3u8]);
    }

    #[test]
    fn realloc_shrink_truncates() {
        let v = vec![1u8, 2u8, 3u8, 4u8];
        assert_eq!(cob_realloc(v, 4, 2), vec![1u8, 2u8]);
    }

    #[test]
    fn realloc_grow_preserves_prefix_and_zeroes_tail() {
        let v = vec![1u8, 2u8, 3u8];
        // osize = 3 live bytes, grow to 6.
        assert_eq!(cob_realloc(v, 3, 6), vec![1u8, 2u8, 3u8, 0u8, 0u8, 0u8]);
    }

    #[test]
    fn realloc_grow_caps_copy_at_osize() {
        let v = vec![1u8, 2u8, 3u8, 4u8];
        // Only osize = 2 bytes are "live"; the rest of the new block is zero.
        assert_eq!(cob_realloc(v, 2, 5), vec![1u8, 2u8, 0u8, 0u8, 0u8]);
    }

    #[test]
    fn free_consumes() {
        cob_free(vec![9u8, 9u8]);
    }

    #[test]
    fn strdup_appends_nul() {
        assert_eq!(cob_strdup(b"abc"), vec![b'a', b'b', b'c', 0u8]);
        assert_eq!(cob_strdup(b""), vec![0u8]);
    }

    #[test]
    fn strdup_stops_at_embedded_nul() {
        // strlen semantics: stop at the first NUL.
        assert_eq!(cob_strdup(b"ab\x00cd"), vec![b'a', b'b', 0u8]);
    }

    #[test]
    fn strndup_caps_at_max_len() {
        assert_eq!(cob_strndup(b"abcdef", 3), vec![b'a', b'b', b'c', 0u8]);
        // max_len longer than the string: full copy + NUL.
        assert_eq!(cob_strndup(b"ab", 10), vec![b'a', b'b', 0u8]);
        assert_eq!(cob_strndup(b"abc", 0), vec![0u8]);
    }

    #[test]
    fn strcat_concatenates_with_one_nul() {
        assert_eq!(cob_strcat(b"foo", b"bar"), vec![b'f', b'o', b'o', b'b', b'a', b'r', 0u8]);
        assert_eq!(cob_strcat(b"", b""), vec![0u8]);
        assert_eq!(cob_strcat(b"x", b""), vec![b'x', 0u8]);
    }

    #[test]
    fn strcat_respects_embedded_nul() {
        assert_eq!(cob_strcat(b"a\x00z", b"b"), vec![b'a', b'b', 0u8]);
    }

    #[test]
    fn strjoin_empty_is_none() {
        let empty: [&[u8]; 0] = [];
        assert_eq!(cob_strjoin(&empty, b","), None);
    }

    #[test]
    fn strjoin_single_element() {
        assert_eq!(cob_strjoin(&[b"abc"], b","), Some(vec![b'a', b'b', b'c', 0u8]));
    }

    #[test]
    fn strjoin_multiple_elements() {
        let parts: [&[u8]; 3] = [b"a", b"b", b"c"];
        assert_eq!(
            cob_strjoin(&parts, b"-"),
            Some(vec![b'a', b'-', b'b', b'-', b'c', 0u8])
        );
    }

    #[test]
    fn strjoin_multichar_separator() {
        let parts: [&[u8]; 2] = [b"x", b"y"];
        assert_eq!(
            cob_strjoin(&parts, b", "),
            Some(vec![b'x', b',', b' ', b'y', 0u8])
        );
    }

    #[test]
    fn cache_malloc_zeroes() {
        assert_eq!(cob_cache_malloc(4), vec![0u8; 4]);
    }

    #[test]
    fn cache_realloc_null_like_grows() {
        let v = vec![7u8, 8u8];
        assert_eq!(cob_cache_realloc(v, 4), vec![7u8, 8u8, 0u8, 0u8]);
    }

    #[test]
    fn cache_realloc_fits_returns_unchanged() {
        let v = vec![1u8, 2u8, 3u8];
        assert_eq!(cob_cache_realloc(v, 2), vec![1u8, 2u8, 3u8]);
    }

    #[test]
    fn cache_free_consumes() {
        cob_cache_free(vec![1u8]);
    }
}

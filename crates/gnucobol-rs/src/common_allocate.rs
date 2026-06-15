//! Faithful 1:1 port of the `ALLOCATE`/`FREE` runtime and the `EXTERNAL` item registry of
//! `libcob/common.c`.
//!
//! In libcob these manage raw heap pointers: `cob_allocate` `malloc`/`calloc`s a block and links it
//! onto the `cob_alloc_base` list, writing the pointer into a BASED item; `cob_free_alloc` unlinks +
//! frees; `cob_external_addr*` keep a `basext` linked list mapping an EXTERNAL name to its allocation.
//! The **raw-pointer return / based-item pointer write is the declared boundary**: this port models the
//! two registries as explicit owning structs ([`AllocCache`], [`ExternalRegistry`]) and returns the
//! zeroed `Vec<u8>` bytes / a stable handle index instead of a pointer. The exception side effects
//! (`cob_set_exception`) are returned as explicit [`AllocException`] values.
//!
//! The functions carry their **exact C names** (the port-index matches by C name).
//!
//! Citations: `cob_allocate` (`common.c:5770`), `cob_free_alloc` (`common.c:5821`),
//! `cob_external_addr` (`common.c:4564`), `cob_external_addr_lookup` (`common.c:4529`),
//! `cob_external_addr_create` (`common.c:4547`). Evidence kind: **source_port**.

#![forbid(unsafe_code)]

/// `COB_MAX_ALLOC_SIZE` -- the upper allocation bound `cob_allocate` enforces (`common.c:238`). The C
/// uses `COB_MAX_UNBOUNDED_SIZE` (`999999999`) when it exceeds `COB_MAX_FIELD_SIZE`; on the 64-bit
/// build `COB_MAX_FIELD_SIZE = 2147483646` is the larger, so that is the bound used here
/// (`coblocal.h:417`/`:424`).
pub const COB_MAX_ALLOC_SIZE: i64 = 2_147_483_646;

/// The `cob_set_exception` codes `cob_allocate` / `cob_free_alloc` raise (`common.c`/`common.h`). In
/// libcob these set `cob_exception_code`; here they are returned explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocException {
    /// `COB_EC_STORAGE_IMP` -- requested size exceeds [`COB_MAX_ALLOC_SIZE`].
    StorageImp,
    /// `COB_EC_STORAGE_NOT_AVAIL` -- the allocation itself failed (out of memory). Not reachable in
    /// this port (a `Vec` allocation that fails aborts), but modelled for fidelity with the C branch.
    StorageNotAvail,
    /// `COB_EC_STORAGE_NOT_ALLOC` -- `cob_free_alloc` got a pointer not in the cache.
    StorageNotAlloc,
}

/// One entry of the `cob_alloc_base` cache (`struct cob_alloc_cache`, `common.c`): a live `ALLOCATE`d
/// block. The C stores `(cob_pointer, size)`; here the block owns its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocBlock {
    /// A stable handle for this block (its index identity), the model's stand-in for `cob_pointer`.
    pub handle: usize,
    /// The block's bytes (`size` long, zeroed for the INITIALIZED/`calloc` path).
    pub data: Vec<u8>,
}

/// The `cob_alloc_base` linked list of live `ALLOCATE`d blocks, as an explicit owning registry. The C
/// list is LIFO (each new block is pushed to the head); this preserves that order so the handle a block
/// reports is stable across frees.
#[derive(Debug, Default)]
pub struct AllocCache {
    blocks: Vec<AllocBlock>,
    next_handle: usize,
}

/// The outcome of [`cob_allocate`]: the handle of the new block (the "pointer" written into the BASED
/// item / RETURNING field -- the declared boundary), or `None` when nothing was allocated (size `<= 0`
/// or an exception), plus any exception raised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocResult {
    /// The new block's handle, or `None` (the C `mptr` being `NULL`).
    pub handle: Option<usize>,
    /// The exception raised, if any (the C `cob_set_exception`).
    pub exception: Option<AllocException>,
}

impl AllocCache {
    /// A fresh, empty allocation cache.
    pub fn new() -> Self {
        Self { blocks: Vec::new(), next_handle: 0 }
    }

    /// Borrow a live block by handle (the model's `cob_pointer` deref).
    pub fn get(&self, handle: usize) -> Option<&AllocBlock> {
        self.blocks.iter().find(|b| b.handle == handle)
    }

    /// Mutably borrow a live block by handle.
    pub fn get_mut(&mut self, handle: usize) -> Option<&mut AllocBlock> {
        self.blocks.iter_mut().find(|b| b.handle == handle)
    }

    /// Number of live blocks.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Whether the cache holds no live blocks.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Port of `common.c:cob_allocate` (`common.c:5770`). Allocate `size` bytes (`cob_get_llint` of the
    /// SIZE field). `size > COB_MAX_ALLOC_SIZE` raises `STORAGE_IMP` and allocates nothing; `size <= 0`
    /// allocates nothing (the C only allocates for `fsize > 0`); otherwise a block of `size` zeroed
    /// bytes is created and pushed to the cache, its handle returned.
    ///
    /// `initialize` is the INITIALIZED operand: `Some(bytes)` makes the block a copy/`cob_move` of
    /// those bytes (truncated/zero-padded to `size`); the special "INITIALIZED with an `ALL x"\0"`
    /// figurative" (`calloc`) and the default (`malloc`, contents indeterminate) both yield zeroed
    /// bytes here -- zeroing is the safe, panic-free model of the C `malloc` whose contents COBOL must
    /// not rely on. Returns the [`AllocResult`].
    pub fn cob_allocate(&mut self, size: i64, initialize: Option<&[u8]>) -> AllocResult {
        if size > COB_MAX_ALLOC_SIZE {
            return AllocResult { handle: None, exception: Some(AllocException::StorageImp) };
        }
        if size <= 0 {
            return AllocResult { handle: None, exception: None };
        }
        let memsize = size as usize;
        let mut data = vec![0u8; memsize];
        if let Some(src) = initialize {
            // cob_move of the INITIALIZED value into the block: copy what fits, leave the rest zero.
            let n = src.len().min(memsize);
            data[..n].copy_from_slice(&src[..n]);
        }
        let handle = self.next_handle;
        self.next_handle += 1;
        // C pushes to the head of cob_alloc_base; keep newest-first.
        self.blocks.insert(0, AllocBlock { handle, data });
        AllocResult { handle: Some(handle), exception: None }
    }

    /// Port of `common.c:cob_free_alloc` (`common.c:5821`). Free a previously [`cob_allocate`]d block by
    /// its handle (the `*ptr1` / `*ptr2` the FREE statement passes). A handle in the cache is unlinked +
    /// dropped and the call succeeds (`Ok` -- the C nulls the caller's pointer, the boundary); a handle
    /// not in the cache raises `STORAGE_NOT_ALLOC`. A `None` handle (a null pointer) is a no-op `Ok`,
    /// matching the C falling through both `if (ptr1 && *ptr1)` / `if (ptr2 && ...)` guards.
    pub fn cob_free_alloc(&mut self, handle: Option<usize>) -> Result<(), AllocException> {
        let handle = match handle {
            Some(h) => h,
            None => return Ok(()),
        };
        if let Some(pos) = self.blocks.iter().position(|b| b.handle == handle) {
            self.blocks.remove(pos);
            Ok(())
        } else {
            Err(AllocException::StorageNotAlloc)
        }
    }
}

/// One `struct cob_external` entry (`common.c`): an EXTERNAL item's name, declared size, and its
/// allocation. The C list is `basext`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalEntry {
    /// `ename` -- the EXTERNAL item's name.
    pub ename: Vec<u8>,
    /// `esize` -- the size it was first allocated with.
    pub esize: i32,
    /// `ext_alloc` -- the item's bytes (zeroed; `cob_malloc` zero-initializes in libcob).
    pub data: Vec<u8>,
}

/// The `basext` registry of EXTERNAL items, as an explicit owning struct. The C pushes each new entry
/// to the head; this preserves that order (newest-first) so `cob_external_addr_lookup`'s first match is
/// the C's.
#[derive(Debug, Default)]
pub struct ExternalRegistry {
    entries: Vec<ExternalEntry>,
}

/// The result of an [`ExternalRegistry::cob_external_addr`] resolve: the index of the resolved entry in
/// the registry (the model's stand-in for the returned `void *`), whether it was newly created
/// (`cob_initial_external` in the C), and any runtime diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalResult {
    /// Index of the resolved entry in the registry (use [`ExternalRegistry::data`] to read its bytes).
    pub index: usize,
    /// `cob_initial_external` -- `true` when this call created the entry, `false` when it reused one.
    pub initial: bool,
    /// A `cob_runtime_error` (fatal) text when the requested size exceeds the stored size, or a
    /// `cob_runtime_warning` text when it is smaller; `None` otherwise.
    pub diagnostic: Option<ExternalDiagnostic>,
}

/// A diagnostic [`ExternalRegistry::cob_external_addr`] would emit (the C `cob_runtime_error` /
/// `cob_runtime_warning`); the abort on the error case is the caller's side effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalDiagnostic {
    /// The message text (`EXTERNAL item '...' previously allocated with size N, requested size is M`).
    pub message: Vec<u8>,
    /// `true` for the error (size grew, fatal), `false` for the warning (size shrank).
    pub fatal: bool,
}

impl ExternalRegistry {
    /// A fresh, empty EXTERNAL registry.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Borrow an entry's bytes by index.
    pub fn data(&self, index: usize) -> Option<&[u8]> {
        self.entries.get(index).map(|e| e.data.as_slice())
    }

    /// Mutably borrow an entry's bytes by index.
    pub fn data_mut(&mut self, index: usize) -> Option<&mut [u8]> {
        self.entries.get_mut(index).map(|e| e.data.as_mut_slice())
    }

    /// Number of registered EXTERNAL items.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Port of `common.c:cob_external_addr_lookup` (`common.c:4529`). Find an existing EXTERNAL by name
    /// (`strcmp`). Returns the matching entry's index and its stored `esize`, or `None`. A pure search
    /// of the `basext` list (the C returns its `ext_alloc` pointer + writes `*exlength`).
    pub fn cob_external_addr_lookup(&self, exname: &[u8]) -> Option<(usize, i32)> {
        self.entries
            .iter()
            .position(|e| e.ename == exname)
            .map(|i| (i, self.entries[i].esize))
    }

    /// Port of `common.c:cob_external_addr_create` (`common.c:4547`). Create a new EXTERNAL entry of
    /// `exlength` zeroed bytes (the C `cob_malloc`, which zero-initializes), pushed to the head of the
    /// registry. Returns the new entry's index. A negative `exlength` is clamped to zero (no allocation
    /// can be negative; the C casts to `size_t`). Pure registry insert.
    pub fn cob_external_addr_create(&mut self, exname: &[u8], exlength: i32) -> usize {
        let size = exlength.max(0) as usize;
        // C pushes to the head of basext; keep newest-first.
        self.entries.insert(
            0,
            ExternalEntry { ename: exname.to_vec(), esize: exlength, data: vec![0u8; size] },
        );
        0
    }

    /// Port of `common.c:cob_external_addr` (`common.c:4564`). Resolve an EXTERNAL item by name + size:
    /// look it up; if found, a requested size **larger** than the stored size is the fatal
    /// `cob_runtime_error` (the C then `cob_hard_failure`s), a **smaller** size is a warning, and
    /// `cob_initial_external` is cleared (`initial: false`); if not found, create it
    /// ([`Self::cob_external_addr_create`]) and set `cob_initial_external` (`initial: true`).
    ///
    /// The special `"ERRNO"` external (`exlength == sizeof(int)`) returns a pointer to the C `errno` --
    /// a host register outside this registry, the declared boundary; this port does not special-case it
    /// (an `"ERRNO"` name is registered like any other), which is the documented divergence.
    pub fn cob_external_addr(&mut self, exname: &[u8], exlength: i32) -> ExternalResult {
        if let Some((index, stored_length)) = self.cob_external_addr_lookup(exname) {
            let diagnostic = if exlength > stored_length {
                Some(ExternalDiagnostic {
                    message: external_size_message(exname, stored_length, exlength),
                    fatal: true,
                })
            } else if exlength < stored_length {
                Some(ExternalDiagnostic {
                    message: external_size_message(exname, stored_length, exlength),
                    fatal: false,
                })
            } else {
                None
            };
            ExternalResult { index, initial: false, diagnostic }
        } else {
            let index = self.cob_external_addr_create(exname, exlength);
            ExternalResult { index, initial: true, diagnostic: None }
        }
    }
}

/// The shared `cob_runtime_error`/`cob_runtime_warning` text of `cob_external_addr` (`common.c:4579` /
/// `:4584`): `EXTERNAL item '<name>' previously allocated with size <stored>, requested size is <req>`.
fn external_size_message(exname: &[u8], stored: i32, requested: i32) -> Vec<u8> {
    let name = String::from_utf8_lossy(exname);
    format!("EXTERNAL item '{name}' previously allocated with size {stored}, requested size is {requested}")
        .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_basic_and_size_bounds() {
        let mut cache = AllocCache::new();
        // size <= 0 allocates nothing, no exception
        let r0 = cache.cob_allocate(0, None);
        assert_eq!(r0, AllocResult { handle: None, exception: None });
        assert!(cache.is_empty());
        // over the max -> STORAGE_IMP, nothing allocated
        let rbig = cache.cob_allocate(COB_MAX_ALLOC_SIZE + 1, None);
        assert_eq!(rbig.exception, Some(AllocException::StorageImp));
        assert_eq!(rbig.handle, None);
        assert!(cache.is_empty());
        // a normal allocate -> a zeroed block
        let r = cache.cob_allocate(4, None);
        let h = r.handle.unwrap();
        assert_eq!(r.exception, None);
        assert_eq!(cache.get(h).unwrap().data, vec![0u8; 4]);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn allocate_initialized() {
        let mut cache = AllocCache::new();
        // INITIALIZED with a shorter value -> copied then zero-padded
        let r = cache.cob_allocate(5, Some(b"AB"));
        let h = r.handle.unwrap();
        assert_eq!(cache.get(h).unwrap().data, b"AB\x00\x00\x00".to_vec());
        // INITIALIZED with a longer value -> truncated to size
        let r2 = cache.cob_allocate(2, Some(b"ABCDE"));
        let h2 = r2.handle.unwrap();
        assert_eq!(cache.get(h2).unwrap().data, b"AB".to_vec());
    }

    #[test]
    fn free_alloc() {
        let mut cache = AllocCache::new();
        let h = cache.cob_allocate(3, None).handle.unwrap();
        // free a live block
        assert_eq!(cache.cob_free_alloc(Some(h)), Ok(()));
        assert!(cache.is_empty());
        // double free -> STORAGE_NOT_ALLOC
        assert_eq!(cache.cob_free_alloc(Some(h)), Err(AllocException::StorageNotAlloc));
        // null pointer -> no-op Ok
        assert_eq!(cache.cob_free_alloc(None), Ok(()));
        // free of an unknown handle -> STORAGE_NOT_ALLOC
        assert_eq!(cache.cob_free_alloc(Some(999)), Err(AllocException::StorageNotAlloc));
    }

    #[test]
    fn external_lookup_create_addr() {
        let mut reg = ExternalRegistry::new();
        // first resolve creates it -> initial true, no diagnostic
        let r = reg.cob_external_addr(b"SHARED", 8);
        assert!(r.initial);
        assert_eq!(r.diagnostic, None);
        assert_eq!(reg.data(r.index).unwrap(), &[0u8; 8]);
        // lookup finds it
        assert_eq!(reg.cob_external_addr_lookup(b"SHARED"), Some((0, 8)));
        assert_eq!(reg.cob_external_addr_lookup(b"OTHER"), None);
        // re-resolve same size -> reused, no diagnostic
        let r2 = reg.cob_external_addr(b"SHARED", 8);
        assert!(!r2.initial);
        assert_eq!(r2.diagnostic, None);
        // resolve with a larger size -> fatal error diagnostic
        let r3 = reg.cob_external_addr(b"SHARED", 16);
        let d3 = r3.diagnostic.unwrap();
        assert!(d3.fatal);
        assert_eq!(
            d3.message,
            b"EXTERNAL item 'SHARED' previously allocated with size 8, requested size is 16".to_vec()
        );
        // resolve with a smaller size -> non-fatal warning
        let r4 = reg.cob_external_addr(b"SHARED", 4);
        let d4 = r4.diagnostic.unwrap();
        assert!(!d4.fatal);
    }

    #[test]
    fn external_create_negative_clamped() {
        let mut reg = ExternalRegistry::new();
        reg.cob_external_addr_create(b"N", -5);
        assert_eq!(reg.data(0).unwrap().len(), 0);
        assert_eq!(reg.cob_external_addr_lookup(b"N"), Some((0, -5)));
    }
}

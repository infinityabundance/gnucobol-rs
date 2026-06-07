# Backend matrix (indexed files, SORT/MERGE) — non-claim today

`gnucobol-rs` makes **no** indexed-file or `SORT`/`MERGE` claim yet. This file exists to name
the boundary *before* any such crate, because in GnuCOBOL these surfaces depend on an ISAM
backend and the **same "GnuCOBOL 3.2"** can behave materially differently depending on it.

## Why this is a minefield, not a footnote

- `ORGANIZATION INDEXED` support depends on an ISAM backend (Berkeley DB, VBISAM, DISAM, or
  none). Record locking / file sharing further depend on the OS and build options.
- `libdb` is used by GnuCOBOL not only for indexed I/O but also for `SORT`/`MERGE` temporary
  storage — so future runtime compatibility is **not** one blob.
- Redistribution: Berkeley DB licensing is a known concern; VBISAM is LGPL. A future indexed
  campaign must record `file_backend_identity` and respect redistribution constraints.

## Future rows (each a separately sealed campaign, none claimed now)

| Backend | Status |
|---------|--------|
| Berkeley DB (this oracle is built `--with-db`, BDB 5.3.28) | future — `GNURUST.BACKEND.0` |
| VBISAM | future |
| DISAM | future |
| no-ISAM | future |

## Runtime surfaces, separated (not one "files" blob)

```
sequential files → relative files → indexed files
→ SORT/MERGE temporary-storage semantics → locking / file-status / error behavior
```

## Receipt fields a future files campaign must carry

- `file_backend_identity` (which ISAM, version, locking support)
- COBOL-visible **file status codes** as the primary artifact (a typed Rust error is
  secondary — see `docs/runtime-doctrine.md`).

**Non-claim:** decimal/byte/data parity does **not** imply indexed-file or SORT/MERGE
compatibility. The Berkeley DB in this oracle build affects only future file work.

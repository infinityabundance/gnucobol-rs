# Runtime & dialect doctrine — boundaries named early

`libcob` is not a bag of pure functions; it carries global state, config, locale, and module
context. And "GnuCOBOL compatibility" is not "COBOL compatibility". These boundaries are named
now so they can never be silently crossed.

## Primary oracle vs reference dialects

- **Primary oracle:** the built **GnuCOBOL 3.2** (identity in `reports/admission/`). The
  *only* authority for parity claims.
- **Reference dialect witnesses** (IBM Enterprise COBOL, Micro Focus, ACU, RM, …): **future,
  non-authoritative**. A dialect witness is never the project's authority unless explicitly
  admitted with its own receipt. Matching GnuCOBOL ≠ matching IBM.

## Special-names contaminants (named, mostly not-admitted today)

GnuCOBOL source can change numeric interpretation via `SPECIAL-NAMES`. Recorded as an explicit
receipt field so its absence is deliberate, not overlooked:

```
special_names_policy =
  decimal_point: period            # comma = not_admitted (DECIMAL-POINT IS COMMA)
  numeric_sign:  default | trailing_separate   # leading/other = per-attr, EBCDIC = not_admitted
  currency_sign: default ('$')     # custom = not_admitted
```

## Display vs storage (do not conflate)

A passing `DISPLAY` stdout test is **not** storage compatibility. These are distinct artifacts
and `cobol-decimal-rs` claims only the storage/move ones:

```
field_storage_bytes      # the bytes a field holds      <- CLAIMED (decimal slice)
move_conversion_bytes    # bytes after MOVE src -> dst   <- CLAIMED (decimal slice)
display_statement_stdout # what DISPLAY prints           <- NOT claimed by the decimal slice
human_formatted_value / debug_dump_value                 # NOT claimed
```

## Runtime global state & test isolation (LIBCOB-RUNTIME.0, future)

`cob_init`, environment, module loading, cancellation, locale, memory, and runtime config are
process-global. Therefore, as policy from day one:

- **Every oracle test that touches env/files/runtime state runs in a fresh child process.**
  (The decimal oracle is already an out-of-process C harness; the Rust mirror is a separate
  example binary — no shared in-process runtime state.)
- Receipts record `cwd`, an env whitelist, `umask`, and tempdir policy.
- No oracle test relies on Rust test ordering or parallelism. In-process speed is not worth a
  false positive/negative from shared global state.

A future `LIBCOB-RUNTIME.0` campaign will formalise the global-state admission; until then no
in-process `libcob` runtime claim is made.

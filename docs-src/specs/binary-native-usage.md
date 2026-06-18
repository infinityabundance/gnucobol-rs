# Implementation Spec — `USAGE BINARY-CHAR / BINARY-SHORT / BINARY-LONG / BINARY-DOUBLE`

Court target: `GNURUST.BINARY-NATIVE.1` (sibling of `GNURUST.INDEX.1`, the synthetic-PIC-usage precedent).
Crate baseline: gnucobol-rs 0.8.7.
Status: implementable as written — every fact below was verified against the live source (pic.rs lines 24-55, 314-356; frontend.rs lines 599-642, 1399-1513, 3150-3199; binary.rs 40-76).

---

## 0. Summary of the finding (why this is not a one-liner)

`BINARY-CHAR/SHORT/LONG/DOUBLE` are COMP-5-family **native binary integers with an implied PIC** (no `PIC` clause). They are *synonyms*, not new storage semantics: native byte order, full binary range, signed by default.

| Synonym | Implied PIC | Display digits | **Byte width (FIXED)** |
|---|---|---|---|
| `BINARY-CHAR`   | `S9(3)`  | 3  | **1** |
| `BINARY-SHORT`  | `S9(5)`  | 5  | **2** |
| `BINARY-LONG`   | `S9(10)` | 10 | **4** |
| `BINARY-DOUBLE` | `S9(20)` | 20 | **8** |

The byte width is **fixed by the synonym** and is NOT derivable from the implied PIC digit count. The existing `synthetic_usage_pic` route (POINTER→`X(8)`, INDEX→`S9(9)`) works only because those synthetic PICs' *natural* widths happen to match. Here they do NOT:

- naive route `BINARY-DOUBLE → S9(20) → Comp5 → binary_size(20) = 16` produces a **16-byte** field; oracle = **8**.
- `BINARY-LONG → S9(10) → binary_size(10) = 8`; oracle = **4**.
- `BINARY-CHAR → S9(3) → binary_size(3) = 2`; oracle = **1**.
- `BINARY-SHORT → S9(5) → binary_size(5) = 4`; oracle = **2** — coincidentally correct, the only one.

(`binary_size` table, pic.rs:46-55: `1..=2→1, 3..=4→2, 5..=9→4, 10..=18→8, _→16`.)

Therefore the fix is a **combined usage-plus-synthetic-PIC** form: the implied PIC supplies digits/scale (so DISPLAY prints the right digit count), but the byte width must be carried independently by a new width-bearing `Usage` variant that reaches `pic::build_field`. The fix **cannot** live in `synthetic_usage_pic` alone.

---

## 1. `pic.rs` — new width-carrying `Usage` variant

### 1a. Add the variant (`enum Usage`, pic.rs:24-42)

`Usage` is `#[non_exhaustive]`; downstream matches already carry wildcards, so adding a variant is non-breaking.

```rust
/// `USAGE BINARY-CHAR/SHORT/LONG/DOUBLE` (`GNURUST.BINARY-NATIVE.1`): COMP-5-family native binary
/// integer whose byte width is FIXED by the synonym (1/2/4/8), NOT derived from the implied-PIC
/// digit count. `digits`/`scale` still come from the implied PIC (S9(3)/S9(5)/S9(10)/S9(20),
/// scale 0) so DISPLAY prints the right digit count. The `u8` is the byte width.
CompNative(u8),
```

### 1b. Add the `build_field` arm (pic.rs:314 match, beside `Usage::Comp5`)

Mirror `Usage::Comp5` (native byte order = no `COB_FLAG_BINARY_SWAP`, full range = `COB_FLAG_REAL_BINARY`, no truncation) but take the size from the variant, NOT from `dialect.binary_size.bytes(nines)`:

```rust
Usage::CompNative(w) => (
    COB_TYPE_NUMERIC_BINARY,
    w as usize,
    COB_FLAG_REAL_BINARY,
),
```

`attr.digits`/`attr.scale` are computed above this match from the implied PIC (3/5/10/20, scale 0) and are unchanged — the digit count drives only DISPLAY formatting and (for truncating usages) the truncation modulus.

### 1c. Width × truncation safety (verified, no extra code)

`CompNative` sets `COB_FLAG_REAL_BINARY` and does **not** set `COB_FLAG_BINARY_TRUNC`. In `binary_encode` (binary.rs:55-76) the truncation modulus `10^digits` is only applied `if attr.flags & COB_FLAG_BINARY_TRUNC != 0` (binary.rs:58). So `digits = 20` (which exceeds the i128-exact `ten_pow_i128` range) is **never** fed to the modulus — the latent clamp at digits>38 is unreachable on this path. Encode/decode are driven entirely by `out.len()` / `bytes[..n]` (= the fixed width 1/2/4/8), with two's-complement sign extension gated on `COB_FLAG_HAVE_SIGN`. The implied PIC carries `S`, so `COB_FLAG_HAVE_SIGN` is set and signed values round-trip correctly. **Verified safe; no change to binary.rs.**

---

## 2. `frontend.rs` — keyword map + combined parse branch

### 2a. New helper `binary_native_usage` (place beside `synthetic_usage_pic`, ~frontend.rs:627)

Returns BOTH the width-bearing usage and the implied (synthetic) PIC string:

```rust
/// `BINARY-CHAR/SHORT/LONG/DOUBLE`: a COMP-5-family native binary integer with an IMPLIED PIC and a
/// byte width FIXED by the synonym. Returns `(Usage::CompNative(width), implied-PIC)`. Distinct from
/// `synthetic_usage_pic` (POINTER/INDEX) because the width here is NOT the implied PIC's natural width.
fn binary_native_usage(w: &str) -> Option<(crate::pic::Usage, &'static str)> {
    use crate::pic::Usage;
    match w {
        "BINARY-CHAR"   => Some((Usage::CompNative(1), "S9(3)")),
        "BINARY-SHORT"  => Some((Usage::CompNative(2), "S9(5)")),
        "BINARY-LONG"   => Some((Usage::CompNative(4), "S9(10)")),
        "BINARY-DOUBLE" => Some((Usage::CompNative(8), "S9(20)")),
        _ => None,
    }
}
```

No collision: `usage_from_kw` (frontend.rs:599-609) keys only on bare `BINARY` (and COMP variants), never `BINARY-*`; `synthetic_usage_pic` (619-627) keys only on POINTER/INDEX; `unsupported_usage_kw` (640) keys only on `NATIONAL`. The `BINARY-*` keywords currently fall through to the **"unrecognized USAGE"** error path (frontend.rs:1421), so they are presently fail-closed, not mis-run.

### 2b. Combined parse branch — set BOTH `usage` and `synthetic`

The clause loop tries, in order, `usage_from_kw` → `synthetic_usage_pic` → `float_usage_kind` → `unsupported_usage_kw`, each setting exactly ONE of `usage` / `synthetic` / `float_kind`. Insert the new branch in BOTH places, ordered **before** the `unsupported_usage_kw`/catch-all arms (it must win before they reject).

**(i) `USAGE [IS]` block** — insert after the `float_usage_kind` arm at frontend.rs:1413-1416, before the `unsupported_usage_kw` arm at 1417:

```rust
Some(Tok::Word(u)) if binary_native_usage(u).is_some() => {
    let (us, pic) = binary_native_usage(u).unwrap();
    usage = Some(us);
    synthetic = Some(pic);
    k += 1;
}
```

**(ii) bare-keyword block** — insert after the `float_usage_kind` arm at frontend.rs:1435-1438, before the `unsupported_usage_kw` arm at 1439:

```rust
Some(Tok::Word(w)) if binary_native_usage(w).is_some() => {
    let (us, pic) = binary_native_usage(w).unwrap();
    usage = Some(us);
    synthetic = Some(pic);
    k += 1;
}
```

### 2c. The existing synthetic→pic fold supplies the implied PIC (no change)

At frontend.rs:1496-1499 the existing fold already turns `synthetic` into the item PIC when no explicit `PIC` clause is present:

```rust
let pic = match pic {
    Some(p) => p,
    None => synthetic.map(|s| s.to_string()).unwrap_or_default(),
};
```

So a `01 D USAGE BINARY-DOUBLE.` item gets `it.pic = "S9(20)"`, `it.usage = Some(Usage::CompNative(8))`. `make_field` (called at frontend.rs:1557 with `it.usage.unwrap_or(Usage::Display)`) forwards `usage` straight into `build_field` (frontend.rs:3160), which now sizes the field at width 8 and fills `vec![fill; pf.size]` (frontend.rs:3167) at 8 bytes. **No change needed at the fold or `make_field` itself.**

### 2d. `resolve_usage_inheritance` — no change

`CompNative` propagates like any other stated `Usage` (frontend.rs:587-592 inherits a group's stated usage to PIC-less children). A `CompNative` leaf already states its usage, so inheritance is inert here.

---

## 3. UNSIGNED follow-on (explicit, OUT of this court)

`CBL_OC_DUMP.cob` writes `usage binary-long unsigned`. The `UNSIGNED` token is currently swallowed by the clause-loop catch-all `_ => k += 1` (frontend.rs:1491) and silently ignored.

For BINARY-LONG, signed and unsigned share the **same byte width (4)** and the same binary load for non-negative values. `CBL_OC_DUMP` uses these fields only as small non-negative counters/lengths, so it round-trips correctly even with `UNSIGNED` ignored.

**Decision for THIS court:** consume-and-ignore `UNSIGNED` under an **admitted non-claim** (recorded in the receipt): "small non-negative `BINARY-* UNSIGNED` round-trips byte-identically to signed; full unsigned semantics deferred." This unblocks `CBL_OC_DUMP` now. (The catch-all at 1491 already consumes the token; the only addition is the receipt-recorded non-claim — optionally an explicit `Some(Tok::Word(w)) if w == "UNSIGNED" => { /* admitted non-claim */ k += 1; }` arm to make the consumption intentional rather than incidental.)

**Deferred follow-on court** (`GNURUST.BINARY-NATIVE.UNSIGNED.1`): true `BINARY-* UNSIGNED` semantics = strip `COB_FLAG_HAVE_SIGN`, implied PIC `9(n)` (not `S9(n)`), full unsigned binary range, and the unsigned DISPLAY digit count oracle-confirmed. NOT in scope here.

(Alternative, if the non-claim is rejected: fail closed on `BINARY-* UNSIGNED` with `RunError::Unsupported`. This would re-block `CBL_OC_DUMP` and is NOT recommended.)

---

## 4. Oracle test (`LENGTH OF` is the load-bearing assertion)

Place at `lab/corpus/frontend/p94_binary_native.cob` (chronological numbering) and as the court's sealed case. The `LD=8` line is the assertion that distinguishes the correct fix from the naive `binary_size(20)=16` bug.

```cobol
       IDENTIFICATION DIVISION.
       PROGRAM-ID. BINNATIVE.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 C USAGE BINARY-CHAR.
       01 S USAGE BINARY-SHORT.
       01 L USAGE BINARY-LONG.
       01 D USAGE BINARY-DOUBLE.
       PROCEDURE DIVISION.
           DISPLAY "LC=" LENGTH OF C   *> oracle: 1
           DISPLAY "LS=" LENGTH OF S   *> oracle: 2
           DISPLAY "LL=" LENGTH OF L   *> oracle: 4
           DISPLAY "LD=" LENGTH OF D   *> oracle: 8
           MOVE 100 TO D
           DISPLAY "D=" D              *> oracle: +00000000000000000100
           MOVE -7 TO S
           DISPLAY "S=" S              *> oracle: -00007
           STOP RUN.
```

Oracle (`lab/oracle/prefix/bin/cobc -x`) captured output:

```
LC=1
LS=2
LL=4
LD=8
D=+00000000000000000100
S=-00007
```

Gate: `cobol_frontend_sweep.sh` runs the source through admitted `cobc -x` (oracle) and clean-room `cobrun`, then `cmp -s` byte-for-byte; any `rust.err` is a hard fail. The `LD=8` line MUST match (proves width 8, not 16). `LC=1` proves width 1 (not the naive 2). The signed-display lines (`D=+...`, `S=-00007`) prove the `S` in the implied PIC produced `COB_FLAG_HAVE_SIGN` and that signed two's-complement decode works.

---

## 5. What this unblocks

- **`extras/CBL_OC_DUMP.cob`** (`/home/one/gnucobol-rs/lab/admit/gnucobol-3.2/extras/CBL_OC_DUMP.cob`) — the GnuCOBOL-shipped hex-dump utility (same admit-bundle tier as CCVS85/extras). It uses `usage binary-long unsigned` for `counter`, `byline`, `len`, `was-called-before`, and `usage pointer` (already supported via `synthetic_usage_pic`) for `addr`. With this court (signed `CompNative`) + the admitted-`UNSIGNED` non-claim (§3), its data division parses fully.
- Any program using the ISO/MF `BINARY-CHAR / BINARY-SHORT / BINARY-LONG / BINARY-DOUBLE` native-integer synonyms.

This also closes the stale negative-capabilities row B-class implication that "`BINARY-*` reaches the unrecognized-USAGE path" — after this court, only `NATIONAL` remains as the single enumerated unsupported usage.

---

## 6. Change inventory (exact edit sites)

| File | Site | Edit |
|---|---|---|
| `pic.rs` | `enum Usage` (line 24-42) | add `CompNative(u8)` |
| `pic.rs` | `build_field` match (line 314, beside `Usage::Comp5` at 334) | add `Usage::CompNative(w) => (COB_TYPE_NUMERIC_BINARY, w as usize, COB_FLAG_REAL_BINARY)` |
| `frontend.rs` | after `synthetic_usage_pic` (~627) | add `binary_native_usage` helper |
| `frontend.rs` | `USAGE [IS]` block, after 1413-1416, before 1417 | add combined branch (§2b-i) |
| `frontend.rs` | bare-keyword block, after 1435-1438, before 1439 | add combined branch (§2b-ii) |
| `frontend.rs` | catch-all 1491 (optional) | explicit `UNSIGNED` consume arm + admitted non-claim |
| `binary.rs` | — | **no change** (verified: truncation gated off, width from `out.len()`) |
| `lab/corpus/frontend/p94_binary_native.cob` | new | §4 oracle test |

No change to: `synthetic_usage_pic`, `make_field` body, `resolve_usage_inheritance`, the synthetic→pic fold.

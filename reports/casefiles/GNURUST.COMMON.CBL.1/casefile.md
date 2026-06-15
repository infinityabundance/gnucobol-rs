<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COMMON.CBL.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.7.82

- **Oracle:** cobc CALL "CBL_AND"/.../"CBL_TOLOWER" (libcob/common.c cob_sys_*), captured from BOTH GnuCOBOL 3.1.2 and 3.2
- **Byte domain(s):** CALL USING buffer(s) + length -> the in-place transformed bytes of the destination buffer
- **Replay:** `bash lab/oracle/cbl_logic_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- faithful pure ports of common.c's CBL_ logic/bit/case system routines (cob_sys_and/or/nor/xor/imp/nimp/eq/not/xf4/xf5/toupper/tolower/printable) -- each a deterministic in-place byte operation over a caller-sized buffer -- proven byte-identical to GnuCOBOL's CALL "CBL_*" results. DIFFERENTIAL: verified across BOTH admitted oracles GnuCOBOL 3.1.2 + 3.2 (cbl_logic_sweep 2/0) via a COBOL program that CALLs each routine on fixed inputs (SRC=0xCC, B=0xAA) and DISPLAYs the result byte. The bit ops: AND d&=s, OR d|=s, NOR d=~(s|d), XOR d^=s, IMP d=~s|d, NIMP d=s&~d, EQ d=~(s^d), NOT d=~d
- XF4 packs 8 low-bits into 1 byte, XF5 unpacks
- TOUPPER/TOLOWER ASCII-case the buffer
- GC_PRINTABLE dots non-printable bytes.

## Negative claims (5) — negative capability is the trust surface
- the COB_CHK_PARMS parameter-count runtime check (the wrong-arg-count diagnostic)
- CBL_GC_PRINTABLE's optional locale (cob_locale_ctype) + custom dot-replacement-char variadic argument beyond the default
- cob_sys_x91 (the switch/param multiplexer -- a follow-on)
- the OS/process CBL routines (getpid/system/fork/nanosleep)
- lie prevented: the CBL_ builtins are libcob-internal -- NO: each is a deterministic byte transform reproduced byte-identically and proven stable across two GnuCOBOL versions

## Damage if overclaimed
claiming 'the CBL_ library' would hide that the parameter-count checks, the x91 multiplexer, the locale path, and the OS/process routines are unported

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

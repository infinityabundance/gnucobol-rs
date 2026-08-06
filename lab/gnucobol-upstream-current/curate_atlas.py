#!/usr/bin/env python3
"""Curate the upstream commit atlas: emit atlas_overrides.json.

This file is the versioned curation source for the Phase-1.4 commit atlas.
Every non-merge upstream commit touching a semantic surface (cobc/ libcob/
bin/ lpvm/ config/) MUST have an entry here; the generator fails closed on any
missing or extra SHA. Phase 2 updates entries (status/action/court) as commits
are integrated; run this script again and then gen_atlas.py.

Usage:
    python3 lab/gnucobol-upstream-current/curate_atlas.py
    python3 lab/gnucobol-upstream-current/gen_atlas.py
"""

from __future__ import annotations

import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "lab", "gnucobol-upstream-current", "atlas_overrides.json")

FS = "GNURUST.UPSTREAM.FRONTEND-SYNC.1"
RS = "GNURUST.UPSTREAM.RUNTIME-SYNC.1"
TS = "GNURUST.UPSTREAM.CURRENT-TESTSUITE.1"
CA = "GNURUST.UPSTREAM.COMMIT-ATLAS.1"

COURT = {
    "RUNTIME_PORTED": RS,
    "FRONTEND_REIMPLEMENTED": FS,
    "WRAPPER_INTEGRATED": FS,
    "CONFIGURATION_INTEGRATED": FS,
    "PLATFORM_BEHAVIOR_INTEGRATED": TS,
    "TEST_IMPORTED": TS,
    "HARNESS_ADOPTED": TS,
    "CI_ONLY_ACCOUNTED": CA,
    "DOCUMENTATION_TRACKED": CA,
    "UPSTREAM_MERGE_ACCOUNTED": CA,
    "NOT_APPLICABLE_WITH_PROOF": CA,
    "SUPERSEDED_BY_LATER_COMMIT": CA,
    "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY": CA,
}

# sha -> (status, action, behavior, residual, evidence, lane_adoption)
C: list[tuple[str, str, str, str, str, str | None, bool]] = []


def E(sha: str, status: str, action: str, behavior: str, residual: str = "",
      evidence: str | None = None, lane_adoption: bool = False) -> None:
    C.append((sha, status, action, behavior, residual, evidence, lane_adoption))


E("a672a26b52b594bd0ebfdcfe0200613c572018d5", "FRONTEND_REIMPLEMENTED",
  "Implement typed nested parser-context mechanism (ContextSet + stack with enter/leave guards, recovery cleanup, leak assertions); extend beyond 32 flags; match upstream accept/reject for CALL convention, CALL USING, REPOSITORY, EXIT, USAGE, TYPEDEF, SPECIAL-NAMES, VALIDATE STATUS, READY/RESET contexts",
  "Upstream expands special parser contexts beyond the 32-bit limit (new context enum replaces bitmask); context-sensitive reserved-word handling changes accepted/rejected behavior",
  "Do not copy the C bitmask macro implementation; Rust typed contexts, deterministic diagnostics")

E("47dda86c0013505df0aae22a4f8bcbc420169e38", "FRONTEND_REIMPLEMENTED",
  "Implement -ftab-width=w1,w2,... list semantics: each 1..12, last repeats indefinitely, malformed/empty/overflow lists fail with stable config errors; apply to fixed/free/auto formats, preprocessing, listing, diagnostics; repeated options follow upstream precedence",
  "-ftab-width accepts a comma-separated list of widths; the final width repeats; single-width legacy behavior preserved",
  "Also WRAPPER side: cobc-rs flag parsing")

E("02964e42e1fa1820210edae27116247ea96927e1", "RUNTIME_PORTED",
  "Rename the candidate runtime's exported is_test value to cob_is_test; update all references",
  "libcob extern value is_test renamed to cob_is_test (public API rename)",
  "Candidate has no C ABI export; rename the equivalent runtime symbol/metadata")

E("50b58f682700bdb1513f7b88769e1942fab73ef7", "RUNTIME_PORTED",
  "Implement COB_LOAD_GLOBAL runtime configuration: determine upstream default and platform history; define interpreted-module equivalent distinguishing local vs global registry visibility; test preload, duplicates, CANCEL/reload, process isolation; keep native-DSO non-claim",
  "New boolean runtime config COB_LOAD_GLOBAL controls loading shared modules into the global symbol namespace",
  "Native DSO loading remains a typed non-claim")

E("9e0d66418efce0cfd7a429b4cd0ef1c0be2b3204", "FRONTEND_REIMPLEMENTED",
  "Suppress the ORGANIZATION INDEXED warning when the file is EXTFH-enabled (candidate checker must not warn where upstream does not)",
  "cobc no longer warns about ORGANIZATION INDEXED for EXTFH files even when built --without-db",
  "EXCEPTION: indexed-file runtime remains unsupported; this is a checker-warning-only change")

E("c4eea8102820f2d9becd2572a0f0b16edbb557d7", "FRONTEND_REIMPLEMENTED",
  "Fix area-check: ENTRY statement must begin in area B, not area A (candidate checker area validation)",
  "ENTRY areacheck corrected to area B", "")

E("39ab4808c7e5365330c4d386db3a8e8fba391e5f", "WRAPPER_INTEGRATED",
  "Listing header must show the basename only (candidate listing generation)",
  "cobc listing header uses the basename of the source file only",
  "Listing shape parity is a separate dimension")

E("2c092ca140b49bc39289ed3ad72953c2a329b0dd", "FRONTEND_REIMPLEMENTED",
  "Check for terminating periods at the end of SET directives; accept/reject and diagnose per upstream",
  "SET directives with trailing periods are checked (period permitted per upstream)", "")

E("a207a45955ec1b1932e994fcc4db12677963d19a", "RUNTIME_PORTED",
  "Implement COB_SIGNAL_REGIME: valid values; registration policy (do-not-register / register-only-if-unclaimed / any other admitted modes); do not clobber external handlers; Unix coverage; classify unsupported platforms; async-signal-safe; runtime reporting",
  "New runtime config COB_SIGNAL_REGIME allows skipping registration of the signal handler",
  "Unsupported platforms classified honestly",
  evidence="8d786cda97bd9ed37a145de2889b40beef12db85")
E("13963e15a2da604b7a0392a06f9a8ec81db9bf04", "WRAPPER_INTEGRATED",
  "-ftcmd listing output must continue across multiple lines instead of truncating (candidate listing generation)",
  "Full -ftcmd output using multiple continuation lines as necessary", "")

E("23f8503529f02002876d1cc9c99ae0f4cc017355", "FRONTEND_REIMPLEMENTED",
  "Improve SD (sort description) syntax checks and error recovery",
  "Better SD syntax validation and recovery", "Completed by 277a07c2e (tests + runtime)")

E("277a07c2ee9c9a5302fe9a07c249ed55cfdfd5bc", "FRONTEND_REIMPLEMENTED",
  "Port the SD syntax-check behavior plus its tests; ensure no hang on malformed SD",
  "SD syntax checks + testsuite coverage", "")

E("7b324f50ebbb05f4c56838e21112a0f8544c6488", "FRONTEND_REIMPLEMENTED",
  "Parser cleanup + better handling of incomplete code: bounded recovery, no hangs, deterministic diagnostics",
  "Better recovery on incomplete source", "")

E("f4ffd50ecd2497ee1d4f45a6e5d6ab42b8c9e573", "FRONTEND_REIMPLEMENTED",
  "Reserved-word handling update + trace update: adopt the changed reserved-word set and trace output",
  "Reserved word handling adjusted; -trace output updated", "")

E("8954b5fc10e63ce3029297822cf8e5628cd7d1d4", "RUNTIME_PORTED",
  "Port the observable effects of the code cleanup across move/screenio/termio/mlio/fileio; adopt the updated tests (data_display, run_accept, run_extensions, run_file, run_manual_screen, run_misc, run_returncode, syn_*)",
  "Code and testsuite cleanup across libcob and cobc/tree.c",
  "Verify no behavior drift vs the updated expectations")

E("7fef5fde70afd9f865578cf20420566bb891a609", "NOT_APPLICABLE_WITH_PROOF",
  "None: C89/C23 source-compat and hardening plus gettext autotools infrastructure are native C build concerns",
  "C89/C23 compatibility adjustments and updated gettext infrastructure for the C build",
  "Proof: candidate is Rust; no C source compatibility surface; no observable COBOL behavior")

E("9d87cdbab8824674dd5c3cdf24ab34835e4fce75", "NOT_APPLICABLE_WITH_PROOF",
  "None: libxml2 ABI compatibility shim is native-library-specific",
  "libcob/mlio.c caters for the libxml2 ABI break (LIBXML_VERSION >= 21200)",
  "Proof: XML boundary is native-artifact-typed; no libxml2 in candidate")

E("3dd1d88da6ff57acf462a392763118069b2c5670", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: initial XML PARSE support is unfinished upstream and depends on the native XML backend",
  "Initial (unfinished) support for XML PARSE",
  "Upstream itself marks it unfinished; candidate XML PARSE stays a typed boundary")

E("1fc700cc0cd9f1e55f011c8c6060c163e4369ddf", "NOT_APPLICABLE_WITH_PROOF",
  "None: C89 compatibility adjustments for the C compiler",
  "C89 compat adjustments in cobc", "Proof: native C source-compat only")

E("ec76b500bb4f46cf30bbbee5b638c3cfa4f31b55", "NOT_APPLICABLE_WITH_PROOF",
  "None: VFILE is a native file backend (Microfocus/Fujitsu virtual file system)",
  "First VFILE update (libcob file backend)",
  "Proof: candidate file layer has no VFILE backend; runtime CALL to CBL_* VFILE functions cannot execute without it")

E("410097c16722c3750abc1e533042b24e3c8fe727", "NOT_APPLICABLE_WITH_PROOF",
  "None: CBL_* VFILE functions depend on the native VFILE backend",
  "New CBL functions for VFILE functionality consistent with Microfocus/Fujitsu",
  "Proof: same backend dependency as ec76b500b")

E("a5253353db128332c515aac0d601b491194da1f2", "NOT_APPLICABLE_WITH_PROOF",
  "None: C portability fixes across the native build",
  "Portability fixes (and more) for the C build", "Proof: native C portability only")

E("eb8536cfcd335924e92af7aac7f2b3d6fd94b78b", "RUNTIME_PORTED",
  "Apply the further date-from-epoch adjustments (final state of the family)",
  "Follow-up to r5531: further adjustments to date computation from epoch",
  "Supersedes the intermediate states of 946f3e638/486565722",
  evidence="7e901d13bc776a79d78e6567636f14de11c543b5")
E("946f3e638c8f1d7c25abc098275faf515869e493", "RUNTIME_PORTED",
  "Fix epoch date conversion (was off by one day) in the candidate date routines; adopt TZ=UTC global test environment",
  "Simplifies and fixes computation of dates from epoch (off-by-one-day); tests run with TZ=UTC for reproducibility",
  "TZ=UTC adoption is HARNESS_ADOPTED in the candidate test env",
  evidence="7e901d13bc776a79d78e6567636f14de11c543b5")
E("486565722c48b21b50165f61e830442e1d5d97ad", "RUNTIME_PORTED",
  "Fix SOURCE_DATE_EPOCH being ignored on subsequent invocations; fix epoch conversion; remove listing-sed dependency from harness",
  "SOURCE_DATE_EPOCH honored consistently; epoch conversion fixed; testsuite reduced sed usage", "",
  evidence="7e901d13bc776a79d78e6567636f14de11c543b5")
E("79c65d0ecf1a0752e96939979fe0b4210e960e36", "FRONTEND_REIMPLEMENTED",
  "Program-level low/high collating values: compute per-program collating low/high in the frontend; runtime comparison (HIGH-VALUE / LOW-VALUE in presence of collating sequences) uses them",
  "Fix comparison with HIGH-VALUE in the presence of collating sequences",
  "Frontend collating tables + runtime strings.c equivalent")

E("dc0cddebe0f026adbe27e82f83563a9c88b58510", "WRAPPER_INTEGRATED",
  "Fix -M/-fcopybook-deps behavior: do not keep the preprocessed file; gate -fcopybook-deps behind the experimental option; adopt tests",
  "Fixes to the dependency generation feature from r5345; -fcopybook-deps made experimental", "")

E("54d4963026a1279a6fc5c2cff3dbd6a53f92dee5", "NOT_APPLICABLE_WITH_PROOF",
  "None: --gentable generates native C translation tables (EBCDIC/ASCII)",
  "Adds an EBCDIC/ASCII table generation feature (--gentable)",
  "Proof: candidate does not generate C source; tables are runtime-internal in Rust")

E("3f99dba4743214d4a44e2d7b6cd177aa1b343e2d", "NOT_APPLICABLE_WITH_PROOF",
  "None: minor mostly-autotools build updates",
  "Minor, mostly build updates (autotools infrastructure)", "Proof: native build infra")

E("140aed5814bc75e5a23a3685645c07a5589a9116", "NOT_APPLICABLE_WITH_PROOF",
  "None: removes an erroneous ifdef/define in the C replace.c",
  "Remove erroneous ifdef/define in replace.c",
  "Proof: C internal preprocessor hygiene; candidate REPLACE layer is independent Rust")

E("bba2a4ee7a73b04a1ee7cbf65c2fbaa8e77eb303", "WRAPPER_INTEGRATED",
  "Show -fwinmain help text on both Win32 and Cygwin in cobc-rs help output",
  "Help text of -fwinmain displayed on both Win32 and Cygwin", "")

E("8a7c349d13ad4484f1ba07ad9add8d88fa115351", "FRONTEND_REIMPLEMENTED",
  "Implement the >>IMP INCLUDE directive (include .h/.c++ headers) at the preprocessing level; adopt the scanner change (leading space removed for internal directives)",
  "FR #176: GC directive >>IMP INCLUDE to include C/C++ header files in generated C code",
  "Candidate has no generated C; directive is accepted and recorded, header inclusion is a native-code boundary — keep typed")

E("23b5446c13ed379d4928b051c0fa576a2d72b67c", "NOT_APPLICABLE_WITH_PROOF",
  "None: indentation-only fix in C typeck.c",
  "Fix bad typeck.c indentation introduced by r5112", "Proof: whitespace only")

E("87500ead47bd937ee0388619be4aa1dc51245e1b", "FRONTEND_REIMPLEMENTED",
  "Fix nested-element handling with the 'with attributes' specification (SCREEN SECTION data-name qualification)",
  "Fixed bugs:#961: Nested elements mishandled despite 'with attributes' specification", "")


E("47ec5f5134164948f766106f6e3fc9934e36a2fd", "NOT_APPLICABLE_WITH_PROOF",
  "None: BDB indexed partial-key comparison",
  "Improve handling of partial keys in indexed_start_internal (BDB)", "Proof: BDB backend absent")

E("ff8f8953be8433cf3a02e95e452a04b059a0c517", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: PANEL functions depend on the native curses backend",
  "Follow-up to r5369 - panel update", "Curses/panel native dependency")

E("8cec9fdb89c0c4d5caf83439df3c320b06cad2c5", "RUNTIME_PORTED",
  "Float display: skip more than a single leading zero in exponent digits (candidate float formatting)",
  "Improve display of floats (clean_double exponent leading zeros)", "")

E("921108ea29fcc55ceaa60f98179cfec9f30f57e5", "RUNTIME_PORTED",
  "Fix out-of-bounds read in optimized move DISPLAY->edited (candidate edited move bounds)",
  "Fix an out of bounds read access in optimized_move_display_to_edited", "",
  evidence="8c6a411faece296923e902964f5a9e5b2190db31")
E("cb5fe73262cf9b32852ad0aaa7ecaa349529e5d1", "RUNTIME_PORTED",
  "Fix STRING/UNSTRING/INSPECT source-overwrite bug (source fields must not be clobbered mid-operation)",
  "Fix a bug where the source of STRING/UNSTRING/INSPECT is overwritten", "")

E("44c96d20a12e96e0802163fdb9a2d05bb41df578", "RUNTIME_PORTED",
  "BLANK WHEN ZERO on signed NUMERIC-EDITED fields: normalize numeric data in edited move; extend edited-move to sign variants",
  "Fix BLANK WHEN ZERO not working on signed NUMERIC-EDITED fields", "",
  evidence="8c6a411faece296923e902964f5a9e5b2190db31")
E("1c357b4a3894bd09940a01c7ee7e0a5ad90ceeb8", "NOT_APPLICABLE_WITH_PROOF",
  "None: BDB DBT app_data fix",
  "Fixed bugs:1032: app_data field of DBT structure not always copied in bdb_bt_compare",
  "Proof: BDB backend absent")

E("b162a03c3d9446b6c549b7c571fdc4f2272b0434", "NOT_APPLICABLE_WITH_PROOF",
  "None: sanitizer-related minor C adjustments",
  "Minor update for sanitizers", "Proof: C internal")

E("dca86ab692a474a1e77c801119dcf42e3c9c207e", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: PANEL portability fix (curses)",
  "Follow-up to r5369 - panel update (portability fix)", "")

E("7bddf706da7a96a5382c7523117ef792e1774490", "NOT_APPLICABLE_WITH_PROOF",
  "None: whitespace-only correction",
  "Fix copy+paste error in r5389 (whitespace)", "Proof: cosmetic")

E("87c1dd5799ff72425b6ede1efa3b2a789610e2a2", "RUNTIME_PORTED",
  "Fix move-to-edited regression with insertion symbols B, 0 and /; register COBOL2025 COB_EC_DATA_NULL and COB_EC_DATA_TRUNCATION exception definitions",
  "Fixed bugs:#1008 regression in move to numeric edited items; adds COB_EC_DATA_NULL + COB_EC_DATA_TRUNCATION",
  "Exceptions are defined but currently unused upstream",
  evidence="8c6a411faece296923e902964f5a9e5b2190db31")
E("d5eb0eb02335042b507d69f86d48ed6cd79346a4", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: PANEL update (curses)",
  "Follow-up to r5369 - panel update", "")

E("45ce8f622930f34306f64839dd397b9c4c4a0c00", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: PANEL update + tests (curses)",
  "Follow-up to r5369 - panel update", "")

E("0bf2ceb38ea46378e45ea0c865ef75c0374d94d6", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: PANEL update + tests (curses)",
  "Follow-up to r5369 - panel update", "")

E("2a53351eae5a6c53f5408a2594d651fe3d47cdb2", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: PANEL functions from CURSES (native curses/panel dependency)",
  "Add PANEL functions from CURSES", "")

E("a6c4f2440452661e69004da5178196e835f387b9", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track tests: XML/JSON GENERATE tests for PIC P depend on the native XML/JSON backend",
  "Add XML/JSON GENERATE tests for PIC P", "")

E("ac862070c3e821e6c451e3f08e70882a17d271b4", "RUNTIME_PORTED",
  "Fix ACCEPT with TIMEOUT looping through the verb (candidate ACCEPT TIMEOUT semantics)",
  "Fixed bugs:#999 ACCEPT with TIMEOUT issue when looping through the verb", "")

E("83ec07716d30732e050c0b6b96ecf5b462045e21", "NOT_APPLICABLE_WITH_PROOF",
  "None: const-correctness C fix",
  "Portability fix for last commits (const qualification)", "Proof: C internal")

E("ef11be499f4c6f2a3f6907443d54590e1db6f659", "NOT_APPLICABLE_WITH_PROOF",
  "None: duplicate include removal",
  "Fix previous commit (duplicate include)", "Proof: C internal")

E("3f897122aacdd6610c14117b28442013a4574cde", "RUNTIME_PORTED",
  "Signal and stack handling update: align candidate signal-registration and stack-guard behavior with upstream semantics",
  "Signal and stack handling update", "Feeds into COB_SIGNAL_REGIME work (a207a4595)",
  evidence="8d786cda97bd9ed37a145de2889b40beef12db85")
E("c53ae5f803518dc7a0e1e88ee08ac060e423ca1b", "RUNTIME_PORTED",
  "Signal handler updates: port the handler registration/behavior semantics",
  "Signal handler updates", "",
  evidence="8d786cda97bd9ed37a145de2889b40beef12db85")
E("88937849b8607f9f9b20a605d7f2bf65e9ed7427", "CONFIGURATION_INTEGRATED",
  "Adopt configurable version string / bug-report URL surfaces where the candidate exposes equivalents",
  "New configure options for customized version string / bug report URL",
  "Native configure surface; candidate records its own identity")

E("3f7c44b6f51605eda7da480388f8ae2efdd92811", "WRAPPER_INTEGRATED",
  "Improve stdin compilation: cobc-rs must compile from stdin with the documented naming/artifact behavior",
  "Improve stdin compilation", "",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("9b0259d78f87e479887f617206a002636c8e57cb", "NOT_APPLICABLE_WITH_PROOF",
  "None: collating sequence for indexed file keys of alphanumeric class (indexed backend)",
  "Support collating sequence for indexed file keys of alphanumeric class",
  "Proof: indexed-file backend absent")

E("49da19a3dfc05fa7beaed1ac9c1d08a16ee7dd61", "WRAPPER_INTEGRATED",
  "Implement -M/-MD/-MP/-MG/-MQ dependency options + -fcopybook-deps (copybook-only deps, forces -E -foneline-deps -MT=copybooks, disables missing-copybook errors)",
  "Add dependencies options and -fcopybook-deps", "",
  evidence="51a5096d5ff4e312d99585a58567c899e6319d08")
E("a3e00bed1f21ce0f66315039be08c629574c9184", "NOT_APPLICABLE_WITH_PROOF",
  "None: C preprocessor conditional fix (#elif)",
  "Fix bad line in r5343 (preprocessor)", "Proof: C internal")

E("903ba84ff9db736109d025159ee368c6e47ddb99", "NOT_APPLICABLE_WITH_PROOF",
  "Verify in Phase 2: mixed C cleanup/updates (cobc, libcob, build, tests) without a single identified candidate-visible behavior; no known semantic delta",
  "Assorted updates (C cleanup + build)",
  "Proof pending Phase-2 verification; if an observable behavior is found, the row is reclassified")

E("10daa94c89368eac5c84c8ce68d29ecbe0dc5188", "NOT_APPLICABLE_WITH_PROOF",
  "None: libtool/autotools build system update",
  "Build system update (libtool, m4)", "Proof: native build infra")

E("1104bda61e191efb343ba4fa13b71e484bf8b24f", "FRONTEND_REIMPLEMENTED",
  "Check for incompatible data in MOVE or SET only when the receiver is of category numeric",
  "Check for incompatible data only when a receiver is of category numeric in MOVE or SET", "",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("7ba5f9fcb116490a2fbe3e5c2afb9100a3e92c18", "RUNTIME_PORTED",
  "WINDOW pointer preparation: adopt the screenio WINDOW handling model where the candidate screen layer can represent it",
  "Preparation for Multiple Window support by WINDOW pointer",
  "Full multi-window support is a follow-on; keep typed boundary for curses-native windows",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("7b09c750ff7d354f0637678a6c1db425650d8359", "RUNTIME_PORTED",
  "Fix cursor positioning on line 1 (COBOL screen)",
  "Fix bugs:#990 COBOL screen: problem positioning cursor on line 1", "",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("111d21f03445f7d6db0e5cfc93e5f49e9fa584ce", "TEST_IMPORTED",
  "Adopt syn_definition.at updates; the pplex/scanner changes are C89-internal",
  "Minor adjustments (testsuite, ChangeLog entries, C89)", "")

E("42d9e7de0eb8c898cb0e4f13d6cef81319667c33", "NOT_APPLICABLE_WITH_PROOF",
  "None: ChangeLog-only commit",
  "Missing changelog entry for r4915", "Proof: prose only")

E("5a8666888fada0fdbbc904269ef809629fd93fbb", "FRONTEND_REIMPLEMENTED",
  "Give 'U' proper precedence in the expression precedence table (parser); port masking fixes in random/packed move where the candidate has equivalent numeric paths",
  "Fix bugs reported by the MSVC runtime checker (precedence table; explicit casts)", "",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("7529ba38d84bcb886090d4028245546b7b85b446", "NOT_APPLICABLE_WITH_PROOF",
  "None: Windows-only include removal",
  "Remove debugapi.h include from common.c", "Proof: Windows-only, no candidate surface")

E("816bd2be16d88e9ea9f17f2e047fd08844d7f8c2", "NOT_APPLICABLE_WITH_PROOF",
  "None: Windows/MSVC CRT report-mode setting",
  "Disable Windows error popups in programs compiled with MSVC", "Proof: MSVC-only, no candidate surface")

E("6bf47af0209e7cac2f395f71c6e99fd093e5afbc", "RUNTIME_PORTED",
  "Implement runtime configuration to hide the cursor for extended screenio",
  "[feature-request:#474] runtime configuration to hide cursor for extended screenio", "",
  evidence="51a5096d5ff4e312d99585a58567c899e6319d08")
E("41e2e4488de18f3aba4adc4085a63061159b124b", "FRONTEND_REIMPLEMENTED",
  "Complete ALPHABET FOR NATIONAL support (C90 follow-up)",
  "Work on ALPHABET definitions, especially ALPHABET FOR NATIONAL (C90 fix)", "",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("71ea358aa9101faa5f2c3732d763e59934aacc94", "FRONTEND_REIMPLEMENTED",
  "Implement ALPHABET definitions, especially ALPHABET FOR NATIONAL (parse + collating behavior)",
  "Work on ALPHABET definitions, especially ALPHABET FOR NATIONAL", "",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("ec5562cfb9f610ce72029547b68a1999aaa0322a", "RUNTIME_PORTED",
  "Support the 2023 standard for edited numeric picture strings and fix bugs:#935 (picture-string validation + runtime edited move)",
  "Adjustment to support the 2023 standard for edited numeric picture strings; fixes bugs:#935", "",
  evidence="8c6a411faece296923e902964f5a9e5b2190db31")
E("0fa2bf5f5238772b8eb46ace17f2ea958b7726d2", "FRONTEND_REIMPLEMENTED",
  "Increase dialect portability for Micro Focus and ACUCOBOL-GT (reserved words/config.def/parser)",
  "Increase portability for Micro Focus and ACUCOBOL-GT", "",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("9f1a64c32e11b60b6c157c2552905723882f1b76", "RUNTIME_PORTED",
  "Use state structures instead of state vars for STRING/UNSTRING/INSPECT: port the reworked string-operation state handling",
  "[feature-requests:#448] state structures instead of state vars for strings",
  "Structural rework; must preserve STRING/UNSTRING/INSPECT semantics exactly",
  evidence="a989bad32da58fe0edfe517d841aa5e89f9cc234")
E("7a173e6da655bee696bd22315b0ae51f6898dc9b", "NOT_APPLICABLE_WITH_PROOF",
  "None: Sanitizer-warning adjustment (C internal)",
  "Adjustment for Sanitizer warning", "Proof: C internal")

E("24ff1a9c93355660fecce5a44400459a51e24b62", "NOT_APPLICABLE_WITH_PROOF",
  "None: BDB keys of different length (USE_BDB_KEYDIFF flag)",
  "Allow keys of different length in the BDB backend (optional, flag-controlled)",
  "Proof: BDB backend absent")

E("73ad00d945450aba0c0da800494de15768785f2b", "NOT_APPLICABLE_WITH_PROOF",
  "None: curses-less build + C90 warnings",
  "Adjustment for build without curses and fix C90 warnings", "Proof: native build concerns")

E("314adc1ca83055a5676a8e1329e01d6b565f24a3", "NOT_APPLICABLE_WITH_PROOF",
  "None: curses-less build + C90 warnings",
  "Adjustment for build without curses and fix C90 warnings", "Proof: native build concerns")

E("1fa8db0d0e6bd4411f0db864511fd3d9bb6963a5", "CONFIGURATION_INTEGRATED",
  "Adopt the alignment/tab normalization in the candidate dialect-configuration files (verify no semantic value change)",
  "Fix minor alignment/tab issues in config/*.conf", "")

E("435454f8df3808669bfe75ea985446c409877a53", "RUNTIME_PORTED",
  "Adjustment for move to edited numeric (frontend picture + runtime edit alignment)",
  "Adjustment for move to edited numeric", "",
  evidence="8c6a411faece296923e902964f5a9e5b2190db31")
E("e51b091b99211fc8a99f446156e90b4fdf9754c2", "RUNTIME_PORTED",
  "Fix default ROUNDED option behavior",
  "Fix for bug 934 - default ROUNDED option", "",
  evidence="8c6a411faece296923e902964f5a9e5b2190db31")
E("d33f2ec97d726b9578cf5ef55f8acd89ec64e777", "RUNTIME_PORTED",
  "Implement CBL_GC_SCR_DUMP and CBL_GC_SCR_RESTORE as candidate runtime callable functions",
  "Added two new functions CBL_GC_SCR_DUMP and CBL_GC_SCR_RESTORE", "",
  evidence="8c6a411faece296923e902964f5a9e5b2190db31")
E("026a651ee4063b380eaccc07f54aa585d5c86924", "NOT_APPLICABLE_WITH_PROOF",
  "None: errors caught by the GCC Sanitizer (C internal)",
  "Fix errors caught by the Sanitizer functionality of GCC",
  "Proof: C internal; verify no observable behavior delta")

E("b33a87961f0d11b2d9d371374efae94d97c7b557", "NOT_APPLICABLE_WITH_PROOF",
  "None: READ PREVIOUS for VBISAM/VISAM (relative-indexed native backend)",
  "Adjustment to READ PREVIOUS for VBISAM/VISAM", "Proof: backend absent")

E("7c7b55b9311b7edf3bdab5e3c630995f28249958", "RUNTIME_PORTED",
  "Adjustment for move to edited numeric (with tests)",
  "Adjustment for move to edited numeric", "",
  evidence="8c6a411faece296923e902964f5a9e5b2190db31")
E("940f057e6522e4d25c5c4facbe7202c5ee0682e3", "NOT_APPLICABLE_WITH_PROOF",
  "None: Windows/MSVC build command fixes + atlocal_win path fixes",
  "Windows build fixes + ChangeLog cleanups", "Proof: Windows-only")

E("a4971637900ed693ade051aa20a9a5422b20366d", "NOT_APPLICABLE_WITH_PROOF",
  "None: warning fixes + build_windows improvements",
  "More warning fixes, additional improvement to build_windows", "Proof: Windows/native")

E("d671493076c3c041d6139cd04034566d7266482d", "NOT_APPLICABLE_WITH_PROOF",
  "None: cobc handling with the MSVC assembler (native assembly path)",
  "Improving cobc handling with the MSVC assembler", "Proof: native assembly/linker surface")

E("63cb06ce7b87d75a46e7533c0fb7f375bef90960", "NOT_APPLICABLE_WITH_PROOF",
  "None: compiler warnings fixed (C internal)",
  "More compiler warnings fixed", "Proof: C internal")

E("9261f40968684c3f642b7795011992ddf43badba", "NOT_APPLICABLE_WITH_PROOF",
  "None: minor cleanups and warning fixes in libcob (no observable behavior delta; verify with tests)",
  "Minor cleanups and warning fixes", "Proof: C internal hygiene")

E("67f93f93c5b59aaab22917d76195940df1717e18", "NOT_APPLICABLE_WITH_PROOF",
  "None: MSVC build fixes (flag.def macro usage, build_windows)",
  "Fix building with MSVC", "Proof: Windows/native")

E("d2df58ad96850695b8564067afb1f2f4d40fe69a", "NOT_APPLICABLE_WITH_PROOF",
  "None: assorted minor cleanups (C-wide, no single observable behavior; verify with tests)",
  "Assorted minor cleanups", "Proof: C internal; Phase-2 verification required")

E("ed789c8a9bc2c66e3c314f82b47cc07c05b12f49", "PLATFORM_BEHAVIOR_INTEGRATED",
  "Adopt Win32-relevant test expectations (run_extensions, run_file, run_misc, used_binaries); the common.c/fileio.c parts are Windows-path fixes",
  "Win32 fixes, mostly testcases",
  "Windows-only runtime parts are NOT_APPLICABLE to the candidate; test expectations adopted")


E("a0937bf4920e68730746b89706a6edb1967184a9", "FRONTEND_REIMPLEMENTED",
  "Improve handling of broken expressions (recovery, no hangs, correct reject)",
  "Fixing bugs:#933 #938 #966 - handling of broken expressions",
  "Rust has no null-deref class; the candidate rejects the same inputs fail-closed; court proves prompt termination",
  evidence="80673d0f40929789760c7e020e11bacb94f1172c")

E("1b8af634e882db9e0a80bce46d97f24ca0eb58fb", "NOT_APPLICABLE_WITH_PROOF",
  "None: BDB ABI comma fix (DB_VERSION_MAJOR >= 12)",
  "Fixing r5244 fix (BDB 12 ABI break)", "Proof: BDB backend absent")

E("442e6db6d430c208a42c1b25a6ccc870e5bddc12", "NOT_APPLICABLE_WITH_PROOF",
  "None: Win32 build and test fixes",
  "Build and test fixes for Win32", "Proof: Windows-only")

E("67f8f532e194db8d18365c20300c795e4fcf2c5b", "NOT_APPLICABLE_WITH_PROOF",
  "None: COLLATING SEQUENCE clause on SELECT/INDEXED files follow-up",
  "Follow up to r5215: support COLLATING SEQUENCE clause on SELECT/INDEXED files",
  "Proof: indexed backend absent")

E("6a23e2ce5a8c0f37073853953be261f525614ff9", "NOT_APPLICABLE_WITH_PROOF",
  "None: MinGW strcasecmp redefinition removal + whitespace",
  "Housekeeping (MinGW compat removal)", "Proof: C internal")

E("1ea4059c6547e62752e26debb3589ad7ddef8c55", "NOT_APPLICABLE_WITH_PROOF",
  "None: configure ncurses detection via pkg-config",
  "Configure now uses pkg-config/ncurses-config for ncurses", "Proof: native build infra")

E("4695ee78629d659ab9c1ca6cacd04952c3469786", "CONFIGURATION_INTEGRATED",
  "Adopt mf dialect missing-statement configuration",
  "mf dialect: adjusted missing-statement configuration (bugs:#965)",
  "mf-strict.conf custody-synced (missing-statement: ok); the knob itself is a compiler strictness not modeled by the fail-fast parser",
  evidence="80673d0f40929789760c7e020e11bacb94f1172c")

E("6fd7c72cd16e6b1ed50fab065a4443cf9d67697b", "NOT_APPLICABLE_WITH_PROOF",
  "None: build fix (patches:#64)",
  "Build fix", "Proof: native build")

E("8366e1be1cf82b9ee5b6337f032e614dda31737c", "NOT_APPLICABLE_WITH_PROOF",
  "None: housekeeping (build_aux file removal)",
  "Housekeeping", "Proof: native build infra")

E("82100d64de35c89ad5980d1b2c8d1ffdd3563570", "NOT_APPLICABLE_WITH_PROOF",
  "None: memory optimization in the C replace.c; candidate REPLACE layer is independent Rust",
  "Optimization of memory usage in replace.c", "Proof: C internal performance only")

E("7b6995042c4d224d7aed2827387278334b531d17", "RUNTIME_PORTED",
  "Implement the profiling feature for the interpreted candidate: -fprof flag; per-procedure time accounting in the interpreter; COB_PROF_FILE/COB_PROF_MAX_DEPTH/COB_PROF_ENABLE/COB_PROF_FORMAT env support; $b/$f/$d/$t expansion in env strings",
  "Add a profiling feature (-fprof; cob_prof_function_call; COB_PROF_* runtime env)",
  "profiling.rs port + paragraph hooks + -fprof; deterministic test-mode clock; depth-overflow warning",
  evidence="f9e4fa81291961c14be5af1c3d6a884d30ae793f")

E("14f0d0908d985b7747ddcac00d8fbfc06092f1c4", "FRONTEND_REIMPLEMENTED",
  "Fix SEGFAULT when checking BY VALUE arguments of a prototype with ANY LENGTH (checker robustness)",
  "Fix SEGFAULT in checking prototype arguments",
  "Prototype units recognized (never main; CALL fails closed); the C segfault class is inapplicable to Rust; ANY LENGTH checks are inside the prototype boundary",
  evidence="f2531db27e639fad5b4d77757698ce7211394e34")

E("61479ba0c7816ce62d9d559cce977f601d3dccc7", "FRONTEND_REIMPLEMENTED",
  "Fix VALUE ALL \"-\" in SCREEN SECTION (literal handling)",
  "Fix bugs:#947: VALUE ALL \"-\" not working in SCREEN SECTION",
  "VALUE ALL implemented for every VALUE clause (Tok::AllLiteral, oracle-matched repeat-fill); numeric corner recorded",
  evidence="f2531db27e639fad5b4d77757698ce7211394e34")

E("300b542f3caab9dac639e3eb62f60fdedb6c10a2", "NOT_APPLICABLE_WITH_PROOF",
  "Verify in Phase 2: fileio refactoring (native C internals); no candidate-visible behavior expected",
  "fileio refactoring", "Proof pending Phase-2 verification")

E("e36a124b2b7247b0b9bcded694ac3e007e461a01", "WRAPPER_INTEGRATED",
  "Implement --copy COPYBOOK and --include HEADER options (adopt source-location mapping)",
  "Add options --copy COPYBOOK and --include HEADER to cobc",
  "--copy prepends before preprocessing (court proves >>DEFINE visibility); --include is a native-C boundary (rejected)",
  evidence="516876868eec0c41a2e82b5b12a4d693c022a9dc")

E("47ffbd8363bf82482ef7ae3e6a8e9f53b24c1407", "NOT_APPLICABLE_WITH_PROOF",
  "None: performance optimization in the C numeric comparison layer; candidate numeric layer is independent Rust",
  "Improved performance for comparisons between numeric DISPLAY, numeric DISPLAY to literal, BCD + ZERO and to other",
  "Proof: performance-only C change; verify no semantic delta via testsuite")

E("f9596f55fe49c96f278d228db05306bf1042b43f", "NOT_APPLICABLE_WITH_PROOF",
  "None: compilation fix for !WITH_DB builds",
  "Fixing compilation of r5215 for !WITH_DB", "Proof: native build conditional")

E("106e7ce6c98c88bad50e7882a753db165065895f", "NOT_APPLICABLE_WITH_PROOF",
  "None: COLLATING SEQUENCE clause on SELECT/INDEXED files (BDB only) plus -fdefault-file-colseq flag affecting only indexed files",
  "FR #459: support COLLATING SEQUENCE clause on SELECT/INDEXED files (currently only for the BDB backend)",
  "Proof: indexed/BDB backend absent; the flag is indexed-file-only in effect")

E("5d0eecfbdd6d5301312cedaaa4c988a612963de7", "NOT_APPLICABLE_WITH_PROOF",
  "None: Windows longjmp module-unload postponement",
  "Fix random segfaults in cob_call_with_exception_check on Windows", "Proof: Windows-only")

E("2f9892458c548344e115404979a3961bb86ae3c7", "NOT_APPLICABLE_WITH_PROOF",
  "None: MinGW integer-literal codegen + stdint usage",
  "Fix bug #920: Codegen output of integer literals in generated C broken with MinGW",
  "Proof: native codegen")

E("12e31f960ebef69a1e8007b94734a53fcbad6168", "NOT_APPLICABLE_WITH_PROOF",
  "None: 3.3-dev build_windows/config.h adjustments",
  "Minor doc adjustments and build_windows/config.h adjustment for 3.3-dev", "Proof: native build")

E("44848f58b437cce2eac30106e79a7b943e899b7f", "WRAPPER_INTEGRATED",
  "save-temps directory behavior: do not move object/preprocessed files when an explicit target (-E, -c) was given; adopt env-string expansion overflow fix",
  "Minor fixes: save-temps with directory target; cob_get_strerror export; env expansion buffer overflow fix",
  "Native-codegen temp handling (candidate temps are always retained); the env-overflow class is inapplicable to the Rust expansion",
  evidence="516876868eec0c41a2e82b5b12a4d693c022a9dc")

E("140a030d52eefb0b197a2d994b52b3aefd020c35", "WRAPPER_INTEGRATED",
  "Implement -fdiagnostics-absolute-path flag (full paths within diagnostics)",
  "New flag -fdiagnostics-absolute-path to display full paths within error locations",
  "Flag parsed, threaded through the launch manifest; CLI court proves the absolute prefix",
  evidence="516876868eec0c41a2e82b5b12a4d693c022a9dc")

E("777852c35adf44d44bb615cb5b479115307365ce", "TEST_IMPORTED",
  "Adopt the testcase for r5195/bugs:#923",
  "Testcase for r5195 / bugs:#923", "")

E("303917744a6c7ce1bfad31a54f2e787fb1c54821", "FRONTEND_REIMPLEMENTED",
  "Generated modules init/clear unused decimal constants: candidate prepared-program must not emit unused constant state that alters module init/clear",
  "Fix bugs:#923: generated modules init/clear unused decimal constants",
  "C codegen layout change; the candidate has no persistent constant cache (bug class inapplicable); pinned by decimal_constant_after_cancel_and_recall_is_clean",
  evidence="5fc26350c2b9924e238665f5bdd5d9a23102ab54")

E("f67da51cae38c4469e96af8d8c2339175ef61c79", "RUNTIME_PORTED",
  "Decimal constants must live per-module (local storage) and be re-initialized after CANCEL — candidate module state model",
  "Fix bug #917: segfault when accessing a decimal constant after calling a sub-program (CANCEL on subprogram)",
  "C codegen layout change; CANCEL already drops all candidate module state (call_state removed, VALUE rebuild on next CALL); pinned by decimal_constant_after_cancel_and_recall_is_clean",
  evidence="5fc26350c2b9924e238665f5bdd5d9a23102ab54")

E("8e2ec25c26bcb09cb520431ee875bc2a13ddcc2d", "RUNTIME_PORTED",
  "Fix partial broken COB_LS_VALIDATE (line-sequential validation)",
  "Fix bugs:#918 partial broken COB_LS_VALIDATE",
  "C macro-argument fix; the candidate already checked every position; pinned by validate_checks_every_byte_position",
  evidence="1a28cdc579375de5fb4415c3ce5281c41dd606c4")

E("0b22d441757efc5fa1d18e0767bb54fb31203eb1", "RUNTIME_PORTED",
  "Fix DISPLAY and ACCEPT with simple attributes SIGSEGV (candidate screen statements with attribute handling)",
  "Fixing bugs:#913 DISPLAY and ACCEPT with simple attributes SIGSEGV",
  "screenio parms NULL guard is C-internal to a screen path the candidate does not model (pure byte display/accept); the cob_unlock LOCKED revert is integrated with 62b39805c",
  evidence="ccf13403cfdb77a6671f157a56a21a45f49d4141")

E("62b39805ca22be04c822267d2c90aaa6ef1e1610", "RUNTIME_PORTED",
  "Fix CLOSE LOCK abend on OPEN (file state handling)",
  "Fixing bugs:#914 CLOSE LOCK abends program on OPEN",
  "LOCKED state + 38/41/42/30 guards ported; indexed_close safeguards are an indexed-backend boundary",
  evidence="ccf13403cfdb77a6671f157a56a21a45f49d4141")

E("04614ac7afd2b26cdd4431987996726ffaa8004b", "RUNTIME_PORTED",
  "INSPECT optimizations and syntax checks: frontend syntax validation + runtime INSPECT behavior alignment",
  "Optimizations and syntax checks for INSPECT related functions",
  "validate_inspect ported to the checker; conversion table semantics verified identical; sign-on-early-exit is a typed later boundary",
  evidence="12239d2da2288e6782ca668abb18b55fc84d879f")

E("28b02be15485a8802639b34c8381a4e785251ef6", "RUNTIME_PORTED",
  "Restore the cob_decimal_get_display sign-in-diff fix (numeric display sign behavior)",
  "Restore code disabled by the previous commit (numeric sign fix)",
  "The restored branch is the general decimal-display path the candidate already implements; no candidate change needed",
  evidence="6921f51a3abadd577c346cfaabb2625979a534ab")

E("c3d5860bf219b0679e0771a0611cc91b61dbe3a1", "RUNTIME_PORTED",
  "Adopt cob_add_int scale handling and packed_is_negative semantics (numeric behavior)",
  "Minor cleanup and optimizations in libcob (numeric.c: scale handling, sign checks)",
  "Behavior-preserving for the candidate; semantics pinned by courts (add_int_on_scaling_p, packed_is_negative)",
  evidence="6921f51a3abadd577c346cfaabb2625979a534ab")

E("85dccf1c72fb4cf4a7ab32e313e09c0d2fbc7e33", "NOT_APPLICABLE_WITH_PROOF",
  "None: restores a missed tree.h hunk (codeoptim leading-zero skip) — native codegen optimization",
  "Missed commit of tree.h in r5185",
  "Proof: content belongs to the r5185 native-codegen optimization; candidate has no native codegen")

E("470f7db125a42594bcc187b60c2d6757731758f0", "FRONTEND_REIMPLEMENTED",
  "Adopt the adjusted error-handling behavior: error/warning selection, exit status, listings expectations (run_fundamental, run_misc, listings, syn_*); rm-strict.conf alignment",
  "Adjusted error handling (cobc-wide; 25 files incl. tests)",
  "Diagnostic wording parity remains a separate dimension; semantic accept/reject and exit statuses must match",
  evidence="5ca48188321193d158498003f2f7336161b138e3")

E("8208acac177e7d50ba68e99aa661b2f26c5a787a", "NOT_APPLICABLE_WITH_PROOF",
  "None: native version.h increase",
  "Missing commit for r5167 - version increase", "Proof: native version string only")

E("0166302909e95c91105bcaa6b1d5b4b6c7647185", "RUNTIME_PORTED",
  "Fix MOVE PACKED-DECIMAL unsigned to signed bad sign",
  "Fix bugs:#904 MOVE PACKED-DECIMAL unsigned to signed leads to bad sign", "",
  evidence="37a3779b1d660f7d25dc7340e5df23d618de5a58")

E("8ea9ac449c986a65299ad015c9930c977bf909cb", "NOT_APPLICABLE_WITH_PROOF",
  "None: ChangeLog entry redispatch only",
  "Redispatch ChangeLog entries", "Proof: prose only")

E("289c9aef58a9acbb934eb2e69022b9fa6018baf8", "CONFIGURATION_INTEGRATED",
  "Adopt the GCOS configuration file",
  "[GCOS] Add GCOS configuration file", "Dialect config only",
  evidence="7b97303952fe4ce00eb5b062c287e7833ba7db4e")

E("4b72d0a9faac66b05e149d0b33d75a589f8863c9", "RUNTIME_PORTED",
  "Improve memory handling in edge cases: port any observable bounds/state fixes; adopt tests",
  "Improve memory handling in edge-cases (cobc + libcob + tests)",
  "Verify each touched path for observable behavior")

E("b836c467e7ed3d93b87ffba1ca299846ca734043", "RUNTIME_PORTED",
  "Cleanup memory handling in libcob for restart: port module-restart state cleanup semantics",
  "Cleanup memory handling in libcob for restart", "Module lifecycle semantics")

E("bc5c13b27467cdd64aed63f7850a73ba6c83ec1b", "NOT_APPLICABLE_WITH_PROOF",
  "None: C portability updates",
  "Portability updates (C)", "Proof: native C portability")

E("0359d0a78f103046d6dc289ad4f40dbea28c671e", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: XML and JSON updates (native libxml2/json-c backend)",
  "XML and JSON updates", "")

E("bf0b5878a89808fc749023c063dfccc8887ffb09", "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
  "Track only: XML and JSON updates (native backend)",
  "XML and JSON updates", "")

E("5bb0fbe1bb594dcfea7e6aa904b38f30b9fbb854", "RUNTIME_PORTED",
  "CHAR and ORD intrinsics must consider the program collating sequence; CHAR outside collation range raises COB_EC_ARGUMENT_FUNCTION",
  "Fix CHAR and ORD intrinsics in presence of collating sequence", "",
  evidence="6f4f95fd7fc5bea5225659802324b30d38f60d30")
# ---- Lane-adopted test/harness commits (evidence recorded at Phase 3) ----
# The current-upstream suite lane runs the pinned source tree's own .at files, so these test-only
# and harness-only upstream changes are exercised verbatim there. lane_adoption makes the state
# explicit and distinct from an unprocessed semantic commit.

E("60557e874decb307c3cec459ca0f49e2756b27c1", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (version-output expectation in run_misc.at)",
  "Missing commit for r5167 - version increase", "Lane adoption; candidate version identity recorded in cobc-rs info",
  lane_adoption=True)

E("c140aafc1568a20ebfd33e681445529603712bf3", "CONFIGURATION_INTEGRATED",
  "None: configure.ac hardening flag is native C build infrastructure",
  "configure.ac: add -fstack-clash-protection to --enable-hardening",
  "Proof: native build flag; candidate equivalents are Rust hardening options; no COBOL surface",
  lane_adoption=True)

E("9d4be36a13eabf0a9c4a48a927eb4b00c151afdb", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (testsuite correction for r5190)",
  "Correction of testsuite for r5190", "Lane adoption", lane_adoption=True)

E("777852c35adf44d44bb615cb5b479115307365ce", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane; the behavior it tests is ported with 303917744 (module constants)",
  "Testcase for r5195 / bugs:#923", "Lane adoption", lane_adoption=True)

E("6e358998b272e6bc82094483102ea634b4ec133f", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (Windows path-difference fix in run_misc.at)",
  "Fix false positives due to path differences in testsuite (run_misc.at) on Windows",
  "Lane adoption", lane_adoption=True)

E("7c60012c019be8b4f42ab9345081f6bd7ae4b1d5", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (typo fix in run_misc.at)",
  "Fixing typo (testsuite)", "Lane adoption", lane_adoption=True)

E("ed789c8a9bc2c66e3c314f82b47cc07c05b12f49", "PLATFORM_BEHAVIOR_INTEGRATED",
  "Adopt Win32-relevant test expectations via the current-upstream lane; the common.c/fileio.c parts are Windows-path fixes",
  "Win32 fixes, mostly testcases",
  "Windows-only runtime parts are NOT_APPLICABLE to the candidate; test expectations adopted by lane",
  lane_adoption=True)

E("1daa3931493ba29cd649cd3ece9ea3621b3226a1", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (portability fix for r5249)",
  "Portability fix for r5249", "Lane adoption", lane_adoption=True)

E("63bd0f81fa4d3033e8b960f8e1ad1ea6ed0d7e8b", "CONFIGURATION_INTEGRATED",
  "None: native macOS build/test flags",
  "Fix macOS testsuite issues (configure.ac dynamic-library flags)",
  "Proof: native build infra; no candidate surface", lane_adoption=True)

E("db0e8067d3e8d2b22285e726a4d3f229b68b72e9", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (compile-error expected results fix)",
  "Fix small error in compile error expected results", "Lane adoption", lane_adoption=True)

E("1b01ffd2398e226e005382850e71d6708eb11f27", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (MSVC test fixes)",
  "Testsuite fixes for MSVC", "Lane adoption", lane_adoption=True)


E("9744112d55609a6f9ac5992c49a4716db3200a29", "CONFIGURATION_INTEGRATED",
  "None: autotools build system update",
  "Build system update", "Proof: native build infra", lane_adoption=True)


E("a234462ff94b5f5feff62f87d2f097b1e7688a69", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (testsuite environment update)",
  "Testsuite environment update", "Lane adoption", lane_adoption=True)

E("111d21f03445f7d6db0e5cfc93e5f49e9fa584ce", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (syn_definition.at updates); the pplex/scanner changes are C89-internal",
  "Minor adjustments (testsuite, ChangeLog entries, C89)", "Lane adoption", lane_adoption=True)


E("710f053fbd7c65a3e8cfa051f8a2fdb92ebeeec8", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (special-cases test updates)",
  "Testsuite update for special cases", "Lane adoption", lane_adoption=True)

E("88937849b8607f9f9b20a605d7f2bf65e9ed7427", "CONFIGURATION_INTEGRATED",
  "Adopt configurable version string / bug-report URL surfaces where the candidate exposes equivalents",
  "New configure options for customized version string / bug report URL",
  "Native configure surface; candidate records its own identity", lane_adoption=True)

E("929b403b68ffedce147598f57eff79dc02590d6f", "PLATFORM_BEHAVIOR_INTEGRATED",
  "None: build_windows PKGVERSION fix is Windows-only",
  "Fix r5349 missing PKGVERSION for build_windows", "Proof: Windows build infra only", lane_adoption=True)

E("ca09f172185ffbfa821a01030c9a873e26d8dff1", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (minor testsuite update)",
  "Minor testsuite update", "Lane adoption", lane_adoption=True)


E("0cc8207d14de2633fb80ebd790dc929e4be76f13", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (skip via atlocal_win)",
  "Follow-up to r5356 - fixed skip via atlocal_win", "Lane adoption", lane_adoption=True)

E("190139b8baee6e0d08f69ef57dc9c303e7d8a47c", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (skip via atlocal_win)",
  "Follow-up to r5356 - fixed skip via atlocal_win", "Lane adoption", lane_adoption=True)

E("a51ca02a68d5adedead306dfada270ff85582a0f", "CONFIGURATION_INTEGRATED",
  "Adopted the copy/gcwindow.cpy copybook asset into crates/gnucobol-rs/copy/ and wired the system-copy search root",
  "Follow-up to r5371 - panel update (refactoring; adds gcwindow.cpy copybook)",
  "Integrated with the config/copy custody batch",
  evidence="BATCH")

E("c2ee239a5209c0a99d5fae8b76ad86d4c7225103", "CONFIGURATION_INTEGRATED",
  "None: configure.ac follow-up is native build infra",
  "Follow-up to r5369 - panel update", "Proof: native build", lane_adoption=True)

E("dda41815fe1fe00f0c58c6ab348f9ff94546017d", "CONFIGURATION_INTEGRATED",
  "None: configure.ac copy+paste fix is native build infra",
  "Fix copy+paste error in r5389", "Proof: native build", lane_adoption=True)

E("a2e4627e6a485aaf61a42f1e68052b6a26d5770f", "CONFIGURATION_INTEGRATED",
  "Adopt init-justify=no for the GCOS-strict dialect (applied by the gcos-strict.conf custody refresh)",
  "[GCOS dialect] Set init-justify to no",
  "Integrated with the GCOS dialect commit (conf synced to pinned HEAD, init_justify knob stored)",
  evidence="7b97303952fe4ce00eb5b062c287e7833ba7db4e")

E("c265f251f14fe3c7e19bc96a49a318fa52d622a6", "CONFIGURATION_INTEGRATED",
  "None: configure.ac Clang flag is native build infra",
  "Fix configure.ac for Clang", "Proof: native build", lane_adoption=True)

E("7824bb9f16e4d98ccacce2952e9aedbac57c5013", "CONFIGURATION_INTEGRATED",
  "None: configure cleanup is native build infra",
  "Configure cleanup", "Proof: native build", lane_adoption=True)

E("aa297a7c6743706ba777dc0c86cd781c5540a899", "CONFIGURATION_INTEGRATED",
  "None: autotools build update",
  "Build update", "Proof: native build infra", lane_adoption=True)

E("0a761c9fa42cbf4cc01aa5127e21c94991b44ef5", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (SIGTERM test flake fix)",
  "Fix SIGTERM test randomly failing in tests/testsuite.src/used_binaries.at",
  "Lane adoption; upstream-known flake classified per suite", lane_adoption=True)

E("f2106ff244e7c6495df0d1fd6060b5e83f5ee937", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (comment-only follow-up)",
  "Follow-up to r5473 - add missing comment", "Lane adoption", lane_adoption=True)


E("26a5cba4eda9d8cee02878a2f745a808b9b47a48", "CONFIGURATION_INTEGRATED",
  "None: iconv.m4 gettext update is native build infra",
  "Missing file in r5552 from gettext infrastructure update", "Proof: native build", lane_adoption=True)




E("a3d9d6435401a2893c862e41b4dbc956796e8c35", "HARNESS_ADOPTED",
  "Adopt the bashism removal in the candidate testsuite harness",
  "Drop bashisms in atlocal.in and pre-inst-env.in", "Lane adoption", lane_adoption=True)

E("f49bf5314302dfa36e0cc260a8e03d9f09ef4214", "CONFIGURATION_INTEGRATED",
  "None: configure adjustments are native build infra",
  "Configure adjustments", "Proof: native build", lane_adoption=True)

E("34efe755f6f4d6a8d3b9f2158609917ad21ece5d", "TEST_IMPORTED",
  "Adopted by the current-upstream suite lane (test update)",
  "Test update", "Lane adoption", lane_adoption=True)

E("1fa8db0d0e6bd4411f0db864511fd3d9bb6963a5", "CONFIGURATION_INTEGRATED",
  "Adopt the alignment/tab normalization: all config/*.conf synced to the pinned upstream head bytes",
  "Fix minor alignment/tab issues in config/*.conf",
  "Integrated with the config/copy custody batch; whitespace-only drift confirmed before sync",
  evidence="BATCH")

# ---- non-semantic harness / doc / asset overrides (defaults refined) ----
E("8dd5b382cf01cf81a52fcebf71b6b3afe117d17d", "HARNESS_ADOPTED",
  "Adopt the builddir quoting adjustments in the candidate testsuite harness",
  "Quoting adjustments for use of builddir in the testsuite harness", "")

E("94c8c561555ad6424a38aebe4c430b13f1ff0103", "HARNESS_ADOPTED",
  "Adopt build-system support for embedded paths in the candidate harness",
  "Fix #1142 build system support for embedded paths", "")

E("da5c185222c78dbb2badb1f9dc209726db2fdb2e", "HARNESS_ADOPTED",
  "Adopt DIFF-override support in the candidate testsuite harness",
  "Testing and overriding the diff command (configure DIFF override)", "")

E("b583a357302ad882fd2ac6565f823d9750b46e37", "HARNESS_ADOPTED",
  "Adopt the build-and-test updates relevant to the candidate harness",
  "Build and test updates", "")

E("7b3047cb2616c6be935be3293e9bfeb377d10a13", "HARNESS_ADOPTED",
  "Adopt the NIST85 run-definition updates in the candidate NIST85 harness",
  "Update for NIST85 (test-runner definitions)", "")

E("97668518028eda1174b2235cbe5c22d61fe9cf8a", "HARNESS_ADOPTED",
  "Adopt checkmanual workflow improvements in the candidate doc harness",
  "Work on make checkmanual", "")

E("d877fb362d20abcbf9914d9709f493791a985d22", "HARNESS_ADOPTED",
  "Adopt perf-record support and quote fixes in the candidate test runner",
  "Test runner: perf record addition and quote-fix", "")

E("808c9be88a5066b2c9a9b3eb9e7d904ee797a187", "HARNESS_ADOPTED",
  "Adopt the current NIST archive URL in the candidate NIST harness",
  "Retrieve archive of NIST test suite from sourceforge instead of an out-dated URL", "")

E("40ea3891c102db606464e3f1b1a49e206f312534", "DOCUMENTATION_TRACKED",
  "Track only: README/docs refresh",
  "Doc update (ChangeLog, DEPENDENCIES, INSTALL, README, TODO)", "")

E("cd346a66aab8791abd8bc821b9cd4e5c248bd7a8", "DOCUMENTATION_TRACKED",
  "Track only: gnucobol.texi updates",
  "Minor doc changes (gnucobol.texi, ABOUT-NLS, DEPENDENCIES)", "")

E("a51ca02a68d5adedead306dfada270ff85582a0f", "CONFIGURATION_INTEGRATED",
  "Adopt the copy/gcwindow.cpy copybook asset (WINDOW screen support) into the candidate copybook tree; THANKS is prose",
  "Follow-up to r5371 - panel update (refactoring; adds gcwindow.cpy copybook)", "")

E("3edbc6d1f70b0a642c07d056d3ecc2c215731665", "NOT_APPLICABLE_WITH_PROOF",
  "None: developer-tooling config (.gitpod.yml)",
  "Update .gitpod.yml", "Proof: developer tooling only, no candidate surface")

E("8b7350a27a7afc4d424dc3279ec4e30a73ed638e", "NOT_APPLICABLE_WITH_PROOF",
  "None: developer-tooling config (.gitpod.yml)",
  "Update .gitpod.yml", "Proof: developer tooling only, no candidate surface")

E("7225e55ddb07b786af5f0496af7fe62a670dd024", "NOT_APPLICABLE_WITH_PROOF",
  "None: developer-tooling config (.gitpod.yml)",
  "Add gitpod configuration", "Proof: developer tooling only, no candidate surface")

E("fabaca953a23f9ebbdd790acec65190de03e4b48", "NOT_APPLICABLE_WITH_PROOF",
  "None: .gitignore update",
  "Update .gitignore", "Proof: repository hygiene only")


def main() -> int:
    entries = {}
    for sha, status, action, behavior, residual, evidence, lane_adoption in C:
        if status not in COURT:
            print(f"FATAL: unknown status {status} for {sha}")
            return 1
        entries[sha] = {
            "status": status,
            "action": action,
            "behavior": behavior,
            "residual": residual,
            "court": COURT[status],
            "superseded_by": None,
            "evidence": evidence,
            "lane_adoption": lane_adoption,
        }
    doc = {
        "schema": "gnurust-atlas-overrides-v1",
        "range_start": "645b417",
        "range_end": "5568b8fc770ff310e5017300d561d8f3deec257c",
        "entries": entries,
    }
    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump(doc, fh, indent=1, sort_keys=False)
        fh.write("\n")
    print(f"wrote {OUT} with {len(entries)} curated entries")
    return 0


if __name__ == "__main__":
    sys.exit(main())

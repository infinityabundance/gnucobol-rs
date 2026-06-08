#!/usr/bin/env python3
"""Generate reports/negative-capabilities.json from the claim-ladder not_proven items + the fixed
NEG.DOC.* documentation-staleness capabilities (TRUST.4.DOCS makes staleness a negative capability)."""
import json, os, re
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
cl = json.load(open(os.path.join(ROOT, "reports/claim-ladder.json")))
def slug(s): return "NEG." + re.sub(r'[^A-Z0-9]+', '-', s.upper()).strip('-')[:40]
seen = {}
for c in cl["courts"]:
    for neg in [n.strip() for n in (c.get("not_proven", "") or "").split(";") if n.strip()]:
        k = slug(neg)
        seen.setdefault(k, {"id": k, "surface": neg, "status": "not_admitted_fail_closed",
                            "guard": "fails closed / outside the sealed subset",
                            "risk_if_guessed": "silent mis-decode of a surface this court does not prove",
                            "owning_future_campaign": None, "evidence": []})
        seen[k]["evidence"].append(c["id"])
DOC = [
 ("NEG.DOC.STALE_VERSION", "an authoritative doc naming a version other than Cargo.toml", "lab/docs/generate.py check", "a reader trusts a stale version/claim"),
 ("NEG.DOC.STALE_CLAIM", "an authoritative doc claim contradicting the current casefiles", "lab/docs/generate.py check", "README says future but STATUS/casefile says composed"),
 ("NEG.DOC.STALE_RECEIPT_LINK", "a doc linking a receipt/casefile that no longer exists", "doc-gate link check", "a dangling evidence pointer"),
 ("NEG.DOC.STALE_NONCLAIM", "a fail-closed list contradicting an admitted casefile", "lab/docs/generate.py check", "a non-claim that is actually now admitted"),
 ("NEG.DOC.MANUAL_EDIT", "a generated authoritative doc edited by hand", "lab/docs/generate.py check (== regenerated)", "hand-written authority masquerading as generated"),
 ("NEG.DOC.LEGACY_NOT_REPLACED", "a legacy doc moved without a generated replacement", "lab/trust4/migrate_reports.py check", "lost authority with no successor"),
 ("NEG.DOC.INFORMATION_LOSS", "a generated doc that is not an information superset of its legacy", "legacy_preservation review", "dropped caveat/claim/context"),
 ("NEG.DOC.FRONTDOOR_CONFLICT", "a front-door doc linking a demoted legacy report as authority", "lab/trust4/migrate_reports.py check", "a reader follows a non-authoritative exhibit"),
]
for (i, s, g, r) in DOC:
    seen[i] = {"id": i, "surface": s, "status": "machine_detected_negative_finding", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["TRUST.4.DOCS"]}

# Banking operating-semantics refusals (registered NOW, before any banking court exists): the dangerous
# class of misuse is treating a clean DECODE as posting/accounting/extraction/business TRUTH. Each is
# refused by default and admitted only by an explicit declared reconciliation profile.
BANK = [
 ("NEG.ACCOUNTING.POLARITY", "debit/credit polarity inferred from sign, CR/DB marker, transaction code, or account type", "ACCOUNTING.PROFILE.1 (future)", "wrong money direction from a numeric sign"),
 ("NEG.NUMERIC.ROLE", "a numeric field assumed to be an amount (vs a rate, identifier, code, or count)", "ACCOUNTING.PROFILE.2 numeric-role (future)", "summing account IDs / branch numbers / rates into a money total"),
 ("NEG.IDENTIFIER.NUMERIC_COERCION", "a PIC 9(n) identifier coerced to a JSON number (leading zeros are material)", "values stay strings by default (serde Serialize-only)", "lost leading zeros on account/branch/routing numbers"),
 ("NEG.DATE.INFERENCE", "a PIC 9(8) field interpreted as a date / format (YYYYMMDD vs Julian vs period vs sentinel)", "DATE.PROFILE.1 (future)", "a modernization date-format bug"),
 ("NEG.CURRENCY.SCALE", "a copybook scale (V99) treated as currency minor-unit correctness", "CURRENCY.PROFILE.1 (future)", "wrong minor units for JPY/0-decimal or basis-point fields"),
 ("NEG.FIGURATIVE.SENTINEL", "LOW-VALUES / HIGH-VALUES / ZEROES / SPACES in raw records given sentinel meaning", "SENTINEL.PROFILE.1 (future); bytes preserved verbatim", "treating an uninitialized/sentinel field as a real value"),
 ("NEG.DB2.NULL.INDICATOR", "a decoded value treated as DB2 null-state truth without its paired null-indicator", "KOBOLD.DB2HOST.1 (future)", "a field decodes cleanly but is semantically NULL at the DB boundary"),
 ("NEG.SQL.PRECOMPILER", "COPY/layout courts claiming Db2 precompiler source transformation (DECLARE SECTION, SQLCA, INCLUDE)", "SQLPRECOMP.ATLAS.1 (future)", "mis-modeling embedded-SQL host structures"),
 ("NEG.CICS.MESSAGE.SCHEMA", "a copybook for CICS COMMAREA/channel data assumed to be a fixed-record FILE schema", "KOBOLD.CICS.ATLAS.1 (future)", "decoding a message payload as if it were a flat file"),
 ("NEG.CICS.HANDLE.CONTROL", "CICS control-flow / error-handling semantics inferred from a data copybook", "out of scope (data court only)", "assuming exceptional-path record behavior"),
 ("NEG.NUMERIC.ENVIRONMENT.UNKNOWN", "arithmetic-transform parity claimed outside the admitted numeric environment (ARITH/TRUNC/sign mode)", "NUMERIC.ENVIRONMENT.1 manifest (future)", "wrong intermediate/truncation results on write-back"),
 ("NEG.SIZE_ERROR.SEMANTICS", "transformed-record write-back without admitted ON SIZE ERROR / truncation / rounded-carry / sign-of-zero semantics", "SIZE.ERROR.ATLAS.1 + KOBOLD.RECON.2 (future)", "silently corrupting money on overflow"),
 ("NEG.REVERSAL.INFERENCE", "a reversal/chargeback/correction inferred from a negative sign alone", "REVERSAL.PROFILE.1 (future)", "mis-pairing a debit as a reversal of an unrelated entry"),
 ("NEG.POSTING.UNIT", "records grouped into a posting / balancing / settlement unit without a declared manifest", "KOBOLD.POSTING.1 (future)", "treating independent records as a balanced bundle (or vice versa)"),
 ("NEG.BATCH.CONTROL.UNDECLARED", "a KOBOLD.FILE.1 record count treated as a banking batch control total", "BATCH.CONTROL.1 header/trailer manifest (future)", "a mechanical count mistaken for a declared control-record total"),
 ("NEG.SINGLE_LAYOUT_ASSUMPTION", "one layout assumed per file when records carry a header/detail/trailer discriminator", "KOBOLD.VARIANT.1 (future)", "decoding every record with the wrong (single) copybook"),
 ("NEG.CURRENTNESS", "decoded extracted bytes treated as the CURRENT live banking state (vs an as-of/EOD extract)", "EXTRACT.ATLAS.1 business_date/as_of (future)", "acting on stale end-of-day data as if live"),
 ("NEG.PUBLIC.CUSTOMER.DATA", "real customer/account/PII data appearing in a public casefile or audit", "PRIVACY.REDACTION.1 (future); corpus is synthetic only", "leaking regulated data into public evidence"),
]
for (i, s, g, r) in BANK:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": g if "future" in g else None,
               "evidence": ["banking-truth-boundaries"]}

# KOBOLD.BANK.1 truth-layer refusals: the court reconciles DECLARED vs OBSERVED control totals but never
# promotes a balanced file into posting/ledger/business truth.
BANK1 = [
 ("NEG.BANKING.POSTING_TRUTH", "a balanced file proving that records form an accepted posting unit", "KOBOLD.BANK.1 control totals only; posting profile (future)", "treating a reconciled batch as a valid posting"),
 ("NEG.BANKING.LEDGER_TRUTH", "a matched trailer proving the batch was accepted into a system of record", "out of scope (no ledger integration)", "treating a balanced file as ledger acceptance"),
 ("NEG.BANKING.BUSINESS_TRUTH", "decoded/reconciled values proving the account/customer state is actually true", "never -- outside the project", "acting on decoded data as business reality"),
 ("NEG.BANKING.BALANCED_FILE_IS_NOT_CORRECT_FILE", "a file whose declared==observed totals assumed to be correct", "KOBOLD.BANK.1 reports balance, not correctness", "a balanced-but-wrong file passed as correct"),
 ("NEG.BANKING.TRAILER_MATCH_IS_NOT_LEDGER_ACCEPTANCE", "trailer reconciliation assumed to be downstream acceptance", "KOBOLD.BANK.1 reconciles totals only", "skipping ledger-acceptance verification"),
 ("NEG.BANKING.KOBOLD_TOTAL_IS_NOT_TRAILER_TOTAL", "KOBOLD-observed totals conflated with the file's declared trailer totals", "KOBOLD.BANK.1 keeps declared and observed separate", "reporting an observed total as the declared control total"),
 ("NEG.BANKING.SIGN_IS_NOT_POLARITY", "debit/credit polarity inferred from a numeric sign instead of the declared DR/CR field", "KOBOLD.BANK.1 polarity from declared indicator only", "wrong posting side from a numeric sign"),
 ("NEG.BANKING.CR_DB_IS_NOT_POSTING_SIDE", "an edited CR/DB presentation marker assumed to be posting side", "ACCOUNTING.PROFILE polarity (declared)", "presentation mistaken for posting polarity"),
 ("NEG.BANKING.UNKNOWN_RECORD_TYPE", "an undeclared record discriminator silently decoded", "KOBOLD.BANK.1 fails closed on unknown variant", "decoding a record with the wrong layout"),
]
for (i, s, g, r) in BANK1:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.BANK.1"]}

# KOBOLD.DB2HOST.1 refusals: a clean decode is not the database semantic value.
DB2 = [
 ("NEG.DB2.HOST_VALUE_NOT_DATABASE_VALUE", "a decoded host-variable value treated as the database value (ignoring its null/truncation indicator)", "KOBOLD.DB2HOST.1 declared indicator manifest", "acting on a value the database boundary marked null/truncated"),
 ("NEG.DB2.SQLCA_NOT_INTERPRETED", "SQLCA / SQLCODE / SQLSTATE interpreted from data", "out of scope (no SQL execution)", "inferring statement outcome from record bytes"),
 ("NEG.DB2.PACKAGE_NOT_CLAIMED", "DBRM / package / collection identity claimed as evidence", "out of scope", "asserting which package produced the data"),
 ("NEG.DB2.DATABASE_TRUTH_NOT_CLAIMED", "decoded/indicator-evaluated values treated as current database truth", "never -- database_truth claimed:false", "treating an extract as live database state"),
]
for (i, s, g, r) in DB2:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.DB2HOST.1"]}

# GNURUST.19 DIVIDE refusals.
ARITH = [
 ("NEG.ARITH.DIVIDE_BY_ZERO_SIZE_ERROR", "divide-by-zero handled (vs fail closed) without an ON SIZE ERROR court", "GNURUST.19 fails closed (ArithError::DivideByZero)", "silently producing a value on n/0"),
 ("NEG.ARITH.REMAINDER_NOT_CLAIMED", "DIVIDE ... REMAINDER", "future court", "wrong remainder bytes"),
 ("NEG.ARITH.COMPUTE_NOT_CLAIMED", "COMPUTE / arithmetic expression evaluation", "out of scope (single-op courts only)", "mis-evaluating an expression"),
 ("NEG.ARITH.PROCEDURE_FLOW_NOT_CLAIMED", "procedure-division control flow / ON SIZE ERROR branch", "out of scope (no execution)", "assuming control-flow side effects"),
 ("NEG.ARITH.FLOAT_NOT_CLAIMED", "floating-point (COMP-1/COMP-2) arithmetic", "out of scope (integer-decimal only)", "float rounding error in money"),
 ("NEG.ARITH.BINARY_RECEIVER_NOT_CLAIMED", "DIVIDE into a binary (COMP/COMP-5) receiver", "future court", "wrong binary receiver bytes"),
 ("NEG.ARITH.BUSINESS_CALCULATION_NOT_CLAIMED", "a divide result treated as a correct business calculation", "byte semantics only", "a plausible but wrong fee/interest/allocation"),
]
for (i, s, g, r) in ARITH:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_fail_closed", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["GNURUST.19"]}

# KOBOLD.RECON.2 transformed-record refusals.
RECON = [
 ("NEG.RECON.PROCEDURE_DIVISION", "Procedure Division execution inferred from a declared transform", "out of scope (no execution; sealed byte courts only)", "assuming program control-flow side effects"),
 ("NEG.RECON.WRITE_BACK_PRODUCTION", "a transformed record treated as a production write-back", "RECON.2 produces evidence, not write-back", "mutating a system of record on transform evidence"),
 ("NEG.RECON.BUSINESS_TRUTH", "transformed values treated as business truth", "business_truth claimed:false", "acting on transformed data as reality"),
 ("NEG.RECON.UNDECLARED_TRANSFORM", "any transform applied that was not explicitly declared", "RECON.2 fails closed on undeclared targets", "an unintended mutation"),
 ("NEG.RECON.SIDE_EFFECTS", "side effects beyond the declared field bytes", "RECON.2 only writes the declared field", "hidden cross-field effects"),
 ("NEG.RECON.FILE_REWRITE_PARITY", "byte parity of a rewritten FILE/container", "RECON.2 is per-record, not file-rewrite", "assuming the whole file rewrites identically"),
 ("NEG.RECON.LEDGER_ACCEPTANCE", "a transformed record treated as accepted into a ledger", "out of scope", "treating a transform as posting/ledger acceptance"),
]
for (i, s, g, r) in RECON:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.RECON.2"]}

# KOBOLD.POSTING.1 posting-unit custody refusals.
POSTING = [
 ("NEG.POSTING.LEDGER_ACCEPTANCE", "a declared posting unit treated as accepted into a ledger", "POSTING.1 custody only; ledger_truth claimed:false", "treating custody evidence as posting acceptance"),
 ("NEG.POSTING.SETTLEMENT_FINALITY", "a posting unit treated as settled/final", "out of scope", "acting on unsettled data as final"),
 ("NEG.POSTING.ACCOUNT_BALANCE_TRUTH", "posting-unit records treated as correct account balances", "out of scope; business_truth claimed:false", "trusting derived balances as truth"),
 ("NEG.POSTING.SEQUENCE_CONTIGUITY_UNDECLARED", "gap detection without a declared-contiguous sequence", "POSTING.1 detects gaps ONLY when sequence_contiguous is declared", "false gap alarms / missed gaps on a non-contiguous sequence"),
 ("NEG.POSTING.DUPLICATE_NOT_BUSINESS_DUPLICATE", "a duplicate sequence/txn-id treated as a business duplicate", "POSTING.1 records duplicate EVIDENCE, not business meaning", "treating a technical duplicate as a real double-post"),
]
for (i, s, g, r) in POSTING:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.POSTING.1"]}

# KOBOLD.EXTRACT.PROFILE.1 / CUSTODY.1 extraction-provenance refusals.
EXTRACT = [
 ("NEG.EXTRACT.EXTRACTION_TRUTH", "decoded extracted bytes treated as extraction truth (how/when/whence)", "EXTRACT.PROFILE.1 records DECLARED provenance only", "trusting an extract's origin/completeness without provenance"),
 ("NEG.EXTRACT.VSAM_NOT_CLAIMED", "VSAM file-organization semantics claimed", "declared, not interpreted", "mis-modeling a VSAM extract"),
 ("NEG.EXTRACT.INDEXED_BACKEND_NOT_CLAIMED", "indexed-file backend (BDB/VBISAM/VSAM) behavior claimed", "out of scope", "assuming index/key semantics"),
 ("NEG.EXTRACT.FILE_STATUS_NOT_INTERPRETED", "COBOL FILE STATUS / VSAM feedback interpreted", "out of scope", "inferring I/O outcome from data"),
 ("NEG.CODESET.FILE_IO_CONVERSION", "field-level cp500 decode treated as COBOL CODE-SET file-I/O conversion parity", "declared code_set_conversion_before_kobold, not executed", "double-converting or mis-attributing a pre-KOBOLD conversion"),
 ("NEG.COPYBOOK.STALE", "an accepted copybook treated as the production-truth copybook", "EXTRACT.PROFILE.1 copybook_freshness claimed:false (hash+provenance evidence only)", "a stale copybook decoding bytes plausibly wrong"),
]
for (i, s, g, r) in EXTRACT:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.EXTRACT.PROFILE.1"]}

# KOBOLD.PRIVACY.REDACTION.1 refusals.
PRIVACY = [
 ("NEG.PRIVACY.NO_REAL_CUSTOMER_DATA_PUBLIC", "real customer/account/PII data in a public casefile/audit", "corpus is synthetic; redaction withholds declared sensitive values", "leaking regulated data publicly"),
 ("NEG.REDACTION.NOT_ANONYMIZATION", "redaction treated as anonymization", "PRIVACY.REDACTION.1 hides declared values; not an anonymization guarantee", "re-identification from residual fields"),
 ("NEG.REDACTION.NOT_REGULATORY_COMPLIANCE", "redaction treated as GDPR/PCI/regulatory compliance", "evidence-preserving redaction only", "a false compliance posture"),
 ("NEG.REDACTION.HASH_NOT_BUSINESS_VALUE", "a preserved value/raw hash treated as the business value", "hashes are for verification, not value recovery", "using a hash as the field value"),
 ("NEG.REDACTION.TOKEN_NOT_IDENTITY", "a deterministic token treated as an identity / reversible", "tokens are scope-stable, never claimed reversible", "joining tokens as if they were the real identity"),
 ("NEG.REDACTION.REVERSIBILITY_NOT_CLAIMED", "redaction/tokenization assumed reversible", "no reversibility (no key management in v1)", "expecting to recover the original from the casefile"),
 ("NEG.REDACTION.UNLISTED_SENSITIVE_FIELD", "an unlisted sensitive field silently shown under a strict policy", "deny-unlisted withholds + fails closed", "leaking an undeclared sensitive field"),
]
for (i, s, g, r) in PRIVACY:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.PRIVACY.REDACTION.1"]}

# KOBOLD.PERF.1 gated-parallelism refusals.
PERF = [
 ("NEG.PERF.PRODUCTION_SLA", "benchmark numbers treated as a production SLA / capacity guarantee", "synthetic micro-benchmark only", "promising throughput a real deployment cannot hold"),
 ("NEG.PERF.AWS_THROUGHPUT", "local rayon speed treated as AWS/Lambda throughput", "no AWS measurement (see LAMBDA.LIVE.1)", "false serverless capacity planning"),
 ("NEG.PERF.SIMD", "SIMD acceleration claimed", "PERF.1 is record-level rayon only; no SIMD", "assuming vectorized speed not measured"),
 ("NEG.PERF.DETERMINISTIC_SCHEDULING", "deterministic thread scheduling claimed beyond identical artifacts", "only the EMITTED artifacts are guaranteed identical, not the schedule", "depending on thread timing/order"),
 ("NEG.PERF.SEMANTIC_CHANGE", "parallelism assumed to change decode/finding/audit results", "rayon output is byte-identical to scalar (gated)", "trusting a faster path that silently differs"),
]
for (i, s, g, r) in PERF:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_fail_closed", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.PERF.1"]}

# KOBOLD.ENTERPRISE.2 signed-attestation refusals.
ENT2 = [
 ("NEG.ENTERPRISE2.NOT_REGULATORY_COMPLIANCE", "a verified signature treated as regulatory compliance (GDPR/PCI/SOX)", "ENTERPRISE.2 proves a key signed a payload, nothing more", "a false compliance posture"),
 ("NEG.ENTERPRISE2.NOT_PRODUCTION_APPROVAL", "a verified signature treated as production go-live approval", "out of scope", "deploying on an attestation, not a decision"),
 ("NEG.ENTERPRISE2.NOT_CUSTOMER_ACCEPTANCE", "a verified signature treated as customer acceptance/sign-off", "out of scope", "claiming acceptance never given"),
 ("NEG.ENTERPRISE2.NO_LONG_TERM_KEY_CUSTODY", "long-term key governance / rotation / revocation claimed", "ENTERPRISE.2 verifies under a CONFIGURED key only; no key lifecycle", "trusting a compromised/rotated key"),
 ("NEG.ENTERPRISE2.NO_IDENTITY_TRUST_BEYOND_KEY", "identity trust beyond the configured key (no PKI/transparency log)", "the key is the only trust root; no identity binding", "believing a signer identity that was never bound"),
 ("NEG.ENTERPRISE2.NO_SUPPLY_CHAIN_COMPLETENESS", "complete supply-chain assurance beyond the listed materials/products", "in-toto subjects/materials are those listed, not exhaustive", "assuming an unlisted artifact is covered"),
]
for (i, s, g, r) in ENT2:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.ENTERPRISE.2"]}

# KOBOLD.BANK.RECONCILE.1 view refusals (a VIEW, never a new truth source).
BANKREC = [
 ("NEG.BANK_RECONCILE.NOT_LEDGER_ACCEPTANCE", "a matched reconciliation view treated as ledger acceptance", "view summarizes; ledger_truth claimed:false", "posting a batch the ledger never accepted"),
 ("NEG.BANK_RECONCILE.NOT_SETTLEMENT_FINALITY", "the view treated as settlement finality", "out of scope; settlement_truth claimed:false", "treating unsettled data as final"),
 ("NEG.BANK_RECONCILE.NOT_ACCOUNT_BALANCE_TRUTH", "the view treated as correct account balances", "out of scope", "trusting derived balances as truth"),
 ("NEG.BANK_RECONCILE.NOT_BUSINESS_APPROVAL", "the view treated as business approval/sign-off", "out of scope", "claiming approval never given"),
 ("NEG.BANK_RECONCILE.VIEW_NOT_NEW_EVIDENCE", "the view treated as new evidence rather than a summary", "introduces_new_evidence:false; every number is from a court struct", "double-counting a summary as fresh proof"),
 ("NEG.BANK_RECONCILE.MATCH_NOT_CORRECTNESS", "a declared-vs-observed MATCH treated as business correctness", "a match proves equality to the DECLARED totals, not correctness", "trusting a balanced file as a correct file"),
 ("NEG.BANK_RECONCILE.SOURCE_CASEFILE_REQUIRED", "a reconciliation view without its named source casefiles", "the view binds BANK.1/2 + POSTING.1 (+ DB2HOST.1/EXTRACT/PRIVACY) source casefiles by sha256", "a view with no provable source evidence"),
 ("NEG.BANK_RECONCILE.SOURCE_HASH_MISMATCH", "a view whose source sha256 no longer matches the on-disk casefile", "source_evidence pins each source by sha; a verifier fails on mismatch", "a stale view over changed evidence"),
]
for (i, s, g, r) in BANKREC:
    seen[i] = {"id": i, "surface": s, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.BANK.RECONCILE.1"]}

# Atlas P-axis (platform/runtime) refusal.
seen["NEG.PLATFORM.RUNTIME_NOT_CLAIMED"] = {"id": "NEG.PLATFORM.RUNTIME_NOT_CLAIMED",
    "surface": "platform/runtime COBOL semantics (z/OS, z/VSE, IBM i, AIX, OpenVMS, NonStop, ...) claimed",
    "status": "not_admitted_fail_closed", "guard": "P-platform-axis lists platforms as a curated axis; no platform runtime court is admitted",
    "risk_if_guessed": "assuming a platform's runtime behavior (file I/O, EBCDIC, sort, sign) without an oracle",
    "owning_future_campaign": None, "evidence": ["archaeology/atlases/P-platform-axis"]}

# KOBOLD.DIFF.1 declared-target comparison refusals.
DIFF = [
 ("NEG.DIFF.MATCH_NOT_BUSINESS_TRUTH", "a diff MATCH treated as business truth/correctness", "a match proves equality to the declared target under selected rules only", "trusting a matched diff as a correct result"),
 ("NEG.DIFF.EXPECTED_OUTPUT_NOT_ORACLE", "a declared expected artifact treated as an oracle", "oracle_status defaults to not_oracle; never an oracle unless declared by a stronger authority", "treating a golden/previous-run as ground truth"),
 ("NEG.DIFF.SYSTEM_OF_RECORD_NOT_VALIDATED", "a system-of-record export treated as validated truth", "system_of_record_export_unvalidated is explicit; not validated", "trusting an unvalidated SoR export"),
 ("NEG.DIFF.NO_LEDGER_ACCEPTANCE", "a diff match treated as ledger acceptance", "out of scope", "posting on a diff match"),
 ("NEG.DIFF.NO_SETTLEMENT_FINALITY", "a diff match treated as settlement finality", "out of scope", "treating a match as final"),
 ("NEG.DIFF.NO_CUSTOMER_APPROVAL", "a diff match treated as customer approval", "out of scope", "claiming approval from a match"),
 ("NEG.DIFF.NO_NEW_EVIDENCE", "the diff report treated as new evidence", "a diff summarizes actual-vs-declared; it creates no new evidence", "double-counting a diff as fresh proof"),
]
for (i, s2, g, r) in DIFF:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.DIFF.1"]}

# KOBOLD.OPERATOR.1 — explain/totals/dirty-mode are VIEWS over the sealed decode; no new truth.
OPER = [
 ("NEG.OPERATOR.NO_NEW_EVIDENCE", "an operator view (explain/totals) treated as new evidence", "it is a view over the sealed-court decode; creates no new evidence", "double-counting a view as proof"),
 ("NEG.OPERATOR.NOT_BUSINESS_TRUTH", "an operator view treated as business truth", "provenance/totals are record-level evidence, not business truth", "acting on a view as reality"),
 ("NEG.OPERATOR.DIRTY_NOT_REPAIRED", "dirty data in dirty-mode treated as repaired/clean", "dirty bytes are preserved as evidence, never coerced", "trusting dirty data as clean"),
]
for (i, s2, g, r) in OPER:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.OPERATOR.1"]}

# TRUST.5 — the anti-ceremony audit's own non-claims (it must not become ceremony itself).
TRUST5 = [
 ("NEG.TRUST5.AUDIT_NOT_CERTIFICATION", "the audit treated as a certification", "it proves each court CAN FAIL; it is not a certificate", "claiming certified status"),
 ("NEG.TRUST5.CLASS_NOT_QUALITY_SCORE", "the A/B/C/D class treated as a quality score", "class is evidence-shape (oracle/composed/view/staged), not quality", "ranking courts by class"),
 ("NEG.TRUST5.SNAPSHOT_NOT_LIVE", "the audit treated as a live state", "snapshot at generation; STATUS/claim-ladder are live", "trusting a stale audit"),
 ("NEG.TRUST5.NO_NEW_TRUTH", "the audit treated as new truth/evidence", "it re-derives from existing evidence; creates nothing", "double-counting the audit as proof"),
]
for (i, s2, g, r) in TRUST5:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["TRUST.5"]}

# NIST-STYLE-FIXTURE-FORMAT.1 — named replayable fixtures; "NIST-style" is a SHAPE, not conformance.
FIX = [
 ("NEG.FIXTURE.NOT_NIST_CONFORMANCE", "a fixture treated as NIST COBOL conformance", "'NIST-style' names the shape (named replayable cases); nist_conformance:false", "claiming NIST conformance"),
 ("NEG.FIXTURE.NOT_LANGUAGE_SUITE", "a fixture set treated as a COBOL language test suite", "fixtures cover sealed DATA courts, not the language", "claiming language-suite parity"),
 ("NEG.FIXTURE.NOT_CERTIFICATION", "a passing fixture treated as certification", "a fixture proves one case replays to expected; not a certificate", "claiming certified status"),
 ("NEG.FIXTURE.EXPECTED_NOT_ORACLE", "the fixture's expected output treated as oracle authority", "expected is the AUTHOR's declaration, not the GnuCOBOL oracle", "trusting expected as oracle truth"),
 ("NEG.FIXTURE.PASS_NOT_BUSINESS_TRUTH", "a passing fixture treated as business truth", "a replay match is not business correctness", "acting on a green fixture as reality"),
 ("NEG.FIXTURE.SYNTHETIC_NOT_CUSTOMER_DATA", "fixture bytes treated as representative customer data", "fixtures are synthetic risk shapes", "assuming customer representativeness"),
]
for (i, s2, g, r) in FIX:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["NIST-STYLE-FIXTURE-FORMAT.1"]}

# Ecosystem refusals (GnuCOBOL industrial-posture deck): keep KOBOLD in the forensic-evidence lane.
ECO = [
 ("NEG.COBOL.OBJECTS_MESSAGES", "COBOL object/message (OO) semantics admitted by a data court", "out of scope; GnuCOBOL itself lists OO/messages as not-yet; KOBOLD refuses", "decoding/inferring object or message state"),
 ("NEG.COBOL.NIST_CONFORMANCE", "NIST COBOL-85 language conformance claimed", "KOBOLD claims selected fixed-record DATA-court conformance under admitted witnesses, not language conformance", "implying NIST/compiler conformance"),
 ("NEG.FFI.C_API_PARITY", "COBOL<->C FFI / API-boundary parity claimed (the SuperBOL/translator lane)", "KOBOLD is a data-evidence layer, not a translator/FFI checker", "assuming generated-C / FFI behavior"),
 ("NEG.IDE.LSP_PARSER", "a full COBOL parser / LSP / IDE navigation claimed (the SuperBOL lane)", "KOBOLD emits evidence artifacts an IDE COULD consume; it is not the parser/editor", "treating KOBOLD as a language server"),
 ("NEG.DIALECT.IMPLICIT", "a court witness called 'COBOL' in the abstract rather than GnuCOBOL 3.2 under a declared dialect", "dialect profile is EVIDENCE, not metadata; every witness names its dialect/profile", "saying 'COBOL says' when the witness is one dialect"),
]
for (i, s2, g, r) in ECO:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_fail_closed", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["archaeology/ATLAS.md"]}

# KOBOLD.SCALE.1 scale-measurement refusals.
SCALE = [
 ("NEG.SCALE.CUSTOMER_WORKLOAD", "synthetic-corpus throughput treated as customer-workload performance", "corpus is declared synthetic mixed fixed-record; not customer data", "promising customer-workload speed"),
 ("NEG.SCALE.PRODUCTION_SLA", "a scale receipt treated as a production SLA/capacity guarantee", "local micro-measurement only", "false capacity/SLA planning"),
 ("NEG.SCALE.AWS_COST", "local timing treated as AWS/serverless cost", "no AWS measurement (see LAMBDA.LIVE.1)", "false cloud cost projection"),
 ("NEG.SCALE.MAINFRAME_EQUIVALENCE", "scale numbers treated as mainframe-equivalent throughput", "no mainframe comparison", "claiming parity with mainframe batch windows"),
 ("NEG.SCALE.UNIVERSAL_THROUGHPUT", "one host's numbers treated as universal throughput", "host/rustc/profile recorded; single host", "generalizing one machine's result"),
 ("NEG.SCALE.RANDOM_BYTES_NOT_BUSINESS_CORPUS", "a synthetic corpus treated as a representative business corpus", "mixed fixed-record synthetic, not real distributions", "assuming real-data shape from synthetic"),
]
for (i, s2, g, r) in SCALE:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_fail_closed", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.SCALE.1"]}

# KOBOLD.PERF.2 multithreaded-pipeline refusals.
PERF2 = [
 ("NEG.PERF.ORDERED_OUTPUT_NOT_OPTIONAL", "record/finding/JSONL order treated as optional under parallelism", "order-sensitive output is byte-identical to scalar; ordering is never relaxed for speed", "reordered records/findings under threads"),
 ("NEG.PERF.POSTING_CHAIN_PARALLEL_NOT_CLAIMED", "the POSTING.1 hash chain computed in parallel", "the chain is sequential over ordered record hashes (h_i=sha(h_{i-1}||r_i)); only record-local work is parallel", "a parallel chain that diverges from the sequential one"),
 ("NEG.PERF.JSON_FAST_PATH_NOT_SEMANTIC_AUTHORITY", "a faster JSON/render path treated as semantic authority", "rendering is a derived artifact; the courts are the authority", "trusting a fast path that differs"),
 ("NEG.PERF.THREAD_SCHEDULE_NOT_EVIDENCE", "thread scheduling/timing treated as evidence", "only the EMITTED bytes are guaranteed; the schedule is not", "depending on thread timing/order"),
 ("NEG.PERF.PARALLEL_TOTALS_FLOAT_NOT_ALLOWED", "floating-point parallel reduction of totals", "totals are i128 cents (associative/deterministic); float reduction is forbidden", "non-deterministic float totals under parallel reduction"),
 ("NEG.PERF.CUSTOMER_SLA_NOT_CLAIMED", "a PERF.2 timing treated as a customer SLA", "micro-measurement only", "false SLA from a benchmark"),
]
for (i, s2, g, r) in PERF2:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_fail_closed", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.PERF.2"]}

# SIZE.ERROR.ATLAS.1 — observed-only; the control flow is explicitly NOT implemented.
SIZEERR = [
 ("NEG.SIZE_ERROR.CONTROL_FLOW", "ON SIZE ERROR control-flow execution", "the atlas OBSERVES oracle behavior; it does not execute control flow", "running a branch KOBOLD never executes"),
 ("NEG.SIZE_ERROR.ON_SIZE_ERROR_NOT_IMPLEMENTED", "ON SIZE ERROR imperative implemented in KOBOLD", "not implemented; observation only", "assuming the imperative ran"),
 ("NEG.SIZE_ERROR.NOT_ON_SIZE_ERROR_NOT_IMPLEMENTED", "NOT ON SIZE ERROR imperative implemented", "not implemented; observation only", "assuming the not-error branch ran"),
 ("NEG.SIZE_ERROR.RECEIVER_WRITE_NOT_INFERRED", "receiver-written/preserved INFERRED rather than observed", "recorded from sentinel before/after oracle bytes, never inferred", "guessing whether a money field changed"),
 ("NEG.SIZE_ERROR.BRANCH_EXECUTION_NOT_CLAIMED", "which branch executes claimed", "out of scope (no Procedure Division execution)", "claiming control-flow outcome"),
 ("NEG.SIZE_ERROR.BUSINESS_ARITHMETIC_NOT_CLAIMED", "business-arithmetic correctness claimed from the atlas", "byte-behavior observation only", "trusting overflow handling as business-correct"),
]
for (i, s2, g, r) in SIZEERR:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_observed_only", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["SIZE.ERROR.ATLAS.1"]}

# KOBOLD.LAYOUT.REDEFINES.2 — overlapping byte VIEWS, never the active view unless declared.
REDEF = [
 ("NEG.REDEFINES.ACTIVE_VIEW_NOT_INFERRED", "which REDEFINES view is semantically active inferred", "active_view stays claimed:false unless a declared discriminator admits it", "promoting a layout-valid overlay into truth"),
 ("NEG.REDEFINES.LAYOUT_VALID_NOT_BUSINESS_MEANING", "a layout-valid view treated as business-meaningful", "byte_view_truth only; business_meaning claimed:false", "trusting a valid overlay as the meaning"),
 ("NEG.REDEFINES.MULTI_VIEW_TRUTH_NOT_CLAIMED", "multiple overlapping views treated as simultaneously true", "views are alternative byte interpretations of the SAME bytes", "double-reading shared storage as multiple records"),
 ("NEG.REDEFINES.DISCRIMINATOR_REQUIRED", "an active view without a declared discriminator", "active view requires a declared discriminator/profile; unknown -> false", "guessing the active view"),
 ("NEG.REDEFINES.WRITE_BACK_NOT_CLAIMED", "a decoded view treated as a write-back", "decode/evidence only", "mutating shared storage on a view"),
]
for (i, s2, g, r) in REDEF:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.LAYOUT.REDEFINES.2"]}

# KOBOLD.SENTINEL.PROFILE.1 — declared markers as evidence; nothing inferred.
SENT = [
 ("NEG.SENTINEL.LOW_VALUES_NOT_NULL", "LOW-VALUES / 0x00 treated as NULL", "a declared marker is evidence only; nullness needs DB2HOST.1/a declared profile", "treating a binary-zero field as database null"),
 ("NEG.SENTINEL.HIGH_VALUES_NOT_MAX_DATE", "HIGH-VALUES / 0xFF treated as a max-date", "marker evidence only; date meaning needs DATE.PROFILE", "reading 0xFF as 9999-12-31"),
 ("NEG.SENTINEL.SPACES_NOT_MISSING", "SPACES treated as missing", "marker evidence only", "treating blanks as absent data"),
 ("NEG.SENTINEL.ZEROES_NOT_ABSENT", "ZEROES treated as absent/zero-meaning", "marker evidence only", "treating zero as no-value"),
 ("NEG.SENTINEL.ZERO_DATE_NOT_DATE", "a zero-date (00000000) treated as a date value", "marker evidence only; date meaning needs DATE.PROFILE", "computing on a zero-date"),
 ("NEG.SENTINEL.MARKER_NOT_BUSINESS_STATUS", "a declared marker treated as a business status/account state", "business_status claimed:false", "deriving account state from a marker"),
 ("NEG.SENTINEL.UNDECLARED_NOT_INFERRED", "an UNDECLARED sentinel-looking value inferred as a marker", "only DECLARED rules match; undeclared_inference:false", "guessing a sentinel that was not declared"),
]
for (i, s2, g, r) in SENT:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.SENTINEL.PROFILE.1"]}

# KOBOLD.DATE.PROFILE.1 — declared date format; no date inference.
DATEP = [
 ("NEG.DATE.PIC9_NOT_DATE", "a PIC 9(n) field treated as a date without a declared profile", "only declared date fields are validated; PIC shape alone gets no date claim", "reading a numeric id as a date"),
 ("NEG.DATE.ZERO_DATE_NOT_NULL", "00000000 treated as null", "sentinels delegated to SENTINEL.PROFILE.1; nullness needs DB2HOST.1", "treating a zero-date as null"),
 ("NEG.DATE.HIGH_DATE_NOT_MAX_DATE", "99999999 treated as a max/open date", "declared sentinel only; no max-date inference", "reading 99999999 as forever-open"),
 ("NEG.DATE.Y2K_WINDOW_NOT_INFERRED", "a 2-digit year windowed to a century", "no Y2K window inference", "guessing 19xx vs 20xx"),
 ("NEG.DATE.BUSINESS_CALENDAR_NOT_CLAIMED", "a valid date treated as a business-calendar date", "format_valid_only; business_calendar claimed:false", "assuming a business day / holiday calendar"),
 ("NEG.DATE.SETTLEMENT_DATE_NOT_CLAIMED", "a date treated as settlement/maturity meaning", "out of scope", "deriving settlement from a date field"),
 ("NEG.DATE.ARITHMETIC_NOT_CLAIMED", "date arithmetic (age, term, days-between) claimed", "format validation only", "computing on dates"),
 ("NEG.DATE.CURRENTNESS_NOT_CLAIMED", "a date treated as current / timezone-resolved", "no currentness/timezone", "assuming as-of/current meaning"),
]
for (i, s2, g, r) in DATEP:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.DATE.PROFILE.1"]}

# KOBOLD.CURRENCY.PROFILE.1 — declared currency/amount profile; no money inference.
CURP = [
 ("NEG.CURRENCY.V99_NOT_MONEY", "a PIC V99 field treated as money without a declared profile", "only declared amount fields are admitted; V99 scale alone is no money claim", "summing a V99 rate/id as money"),
 ("NEG.CURRENCY.MINOR_UNIT_NOT_INFERRED", "the minor-unit scale inferred from PIC", "scale comes from the declared profile, compared to observed", "assuming 2 decimals for a 0/3-decimal currency"),
 ("NEG.CURRENCY.CODE_NOT_LEGAL_TENDER_TRUTH", "a currency-code field treated as legal-tender truth", "the code is preserved as EVIDENCE only; legal_tender:false", "trusting a code as the legal currency"),
 ("NEG.CURRENCY.FX_CONVERSION_NOT_CLAIMED", "FX conversion claimed", "out of scope", "converting across currencies"),
 ("NEG.CURRENCY.ROUNDING_POLICY_NOT_CLAIMED", "a rounding policy claimed", "out of scope", "assuming half-up/banker's rounding"),
 ("NEG.CURRENCY.SIGN_NOT_POLARITY", "a numeric sign treated as debit/credit polarity", "sign is not polarity; BANK.2 owns the side", "deriving DR/CR from a minus sign"),
 ("NEG.CURRENCY.RATE_NOT_AMOUNT", "a rate/percent field admitted as money", "role!=amount -> not admitted as money", "summing a rate into a money total"),
 ("NEG.CURRENCY.BUSINESS_VALUE_NOT_CLAIMED", "a validated amount treated as business value", "amount_scale_evidence only; business_value:false", "trusting a decoded amount as the real value"),
]
for (i, s2, g, r) in CURP:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.CURRENCY.PROFILE.1"]}

# SUPPORT-PACKET.1 — a bundle of existing evidence; creates no new truth.
SUPP = [
 ("NEG.SUPPORT.NO_NEW_TRUTH", "the support packet treated as new evidence/truth", "it gathers EXISTING generated artifacts; introduces nothing", "double-counting a bundle as fresh proof"),
 ("NEG.SUPPORT.NOT_CERTIFICATION", "the packet treated as a certification", "a bundle is not a certificate", "claiming certified status"),
 ("NEG.SUPPORT.NOT_COMPLIANCE", "the packet treated as regulatory compliance", "out of scope", "a false compliance posture"),
 ("NEG.SUPPORT.NOT_PRODUCTION_APPROVAL", "the packet treated as production approval", "out of scope", "deploying on a bundle"),
 ("NEG.SUPPORT.NOT_CUSTOMER_ACCEPTANCE", "the packet treated as customer acceptance", "out of scope", "claiming acceptance"),
 ("NEG.SUPPORT.SNAPSHOT_NOT_LIVE", "the packet treated as a live/current state", "it is a snapshot of the artifacts at generation time; STATUS/claim-ladder are the live authority", "trusting a stale snapshot"),
]
for (i, s2, g, r) in SUPP:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["SUPPORT-PACKET.1"]}

# DIALECT.PROFILE.1 — the witness is GnuCOBOL 3.2 under a declared dialect, never "COBOL".
DIAL = [
 ("NEG.DIALECT.GENERAL_COBOL", "general COBOL behavior claimed from a GnuCOBOL witness", "evidence is GnuCOBOL 3.2.0 under the declared profile, not 'COBOL'", "saying 'COBOL does X' from one compiler"),
 ("NEG.DIALECT.VENDOR_PARITY", "parity with a vendor compiler (IBM/Micro Focus/ACU/...) claimed", "only the admitted GnuCOBOL witness is evidence", "assuming vendor-equivalent behavior"),
 ("NEG.DIALECT.RUNTIME_PORTABILITY", "runtime portability across platforms claimed", "the witness is one build on one host; see NEG.PLATFORM.RUNTIME_NOT_CLAIMED", "assuming portable runtime semantics"),
]
for (i, s2, g, r) in DIAL:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_fail_closed", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["DIALECT.PROFILE.1"]}

# KOBOLD.TOOLING.EXPORT.1 — an export FOR tools; it is not the tool.
TOOL = [
 ("NEG.TOOLING.NOT_LSP", "the export treated as a language server", "it is a static evidence map; not an LSP", "expecting live language-service behavior"),
 ("NEG.TOOLING.NOT_IDE", "the export treated as an IDE/editor integration", "out of scope; an IDE may CONSUME it", "treating KOBOLD as the editor"),
 ("NEG.TOOLING.NOT_FULL_PARSER", "the export treated as a full COBOL parser", "it maps the SEALED-court decode, not a general parse", "assuming full-language parsing"),
 ("NEG.TOOLING.NOT_SOURCE_OF_TRUTH", "the export treated as the source of truth", "STATUS/claim-ladder/casefiles are the authority; this is a downstream map", "trusting the export over the courts"),
 ("NEG.TOOLING.NO_NEW_EVIDENCE", "the export treated as new evidence", "introduces_new_evidence:false; assembled from existing decode/provenance", "double-counting the map as proof"),
 ("NEG.TOOLING.REDACTION_STILL_APPLIES", "a redacted field's cleartext exposed via the export", "the declared redaction policy is honored; no cleartext for a redacted field", "leaking redacted data through tooling"),
]
for (i, s2, g, r) in TOOL:
    seen[i] = {"id": i, "surface": s2, "status": "not_admitted_requires_declared_profile", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["KOBOLD.TOOLING.EXPORT.1"]}
out = {"schema": "kobold-negative-capability-registry-v1",
       "truth_hierarchy": ["bytes", "record truth", "posting truth", "accounting truth", "extraction truth", "business truth"],
       "doctrine": "Negative capability is the trust surface. Banking COBOL data is not merely decoded; it is reconciled under declared control boundaries. This stack preserves the difference between bytes, record truth, posting truth, accounting truth, extraction truth, and business truth -- and refuses to cross those boundaries (NEG.ACCOUNTING.*/NEG.DB2.*/NEG.DATE.*/NEG.POSTING.*/...) unless an explicit declared reconciliation profile admits it. Entries are court non-claims, machine-detectable doc-staleness (NEG.DOC.*), or banking operating-semantics refusals registered before their courts exist.",
       "count": len(seen), "negative_capabilities": sorted(seen.values(), key=lambda x: x["id"])}
json.dump(out, open(os.path.join(ROOT, "reports/negative-capabilities.json"), "w"), indent=2)
print(f"negative-capabilities.json: {len(seen)} surfaces ({len(DOC)} NEG.DOC.*, {len(BANK)} banking refusals)")

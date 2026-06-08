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
out = {"schema": "kobold-negative-capability-registry-v1",
       "truth_hierarchy": ["bytes", "record truth", "posting truth", "accounting truth", "extraction truth", "business truth"],
       "doctrine": "Negative capability is the trust surface. Banking COBOL data is not merely decoded; it is reconciled under declared control boundaries. This stack preserves the difference between bytes, record truth, posting truth, accounting truth, extraction truth, and business truth -- and refuses to cross those boundaries (NEG.ACCOUNTING.*/NEG.DB2.*/NEG.DATE.*/NEG.POSTING.*/...) unless an explicit declared reconciliation profile admits it. Entries are court non-claims, machine-detectable doc-staleness (NEG.DOC.*), or banking operating-semantics refusals registered before their courts exist.",
       "count": len(seen), "negative_capabilities": sorted(seen.values(), key=lambda x: x["id"])}
json.dump(out, open(os.path.join(ROOT, "reports/negative-capabilities.json"), "w"), indent=2)
print(f"negative-capabilities.json: {len(seen)} surfaces ({len(DOC)} NEG.DOC.*, {len(BANK)} banking refusals)")

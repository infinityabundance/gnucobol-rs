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
out = {"schema": "kobold-negative-capability-registry-v1",
       "truth_hierarchy": ["bytes", "record truth", "posting truth", "accounting truth", "extraction truth", "business truth"],
       "doctrine": "Negative capability is the trust surface. Banking COBOL data is not merely decoded; it is reconciled under declared control boundaries. This stack preserves the difference between bytes, record truth, posting truth, accounting truth, extraction truth, and business truth -- and refuses to cross those boundaries (NEG.ACCOUNTING.*/NEG.DB2.*/NEG.DATE.*/NEG.POSTING.*/...) unless an explicit declared reconciliation profile admits it. Entries are court non-claims, machine-detectable doc-staleness (NEG.DOC.*), or banking operating-semantics refusals registered before their courts exist.",
       "count": len(seen), "negative_capabilities": sorted(seen.values(), key=lambda x: x["id"])}
json.dump(out, open(os.path.join(ROOT, "reports/negative-capabilities.json"), "w"), indent=2)
print(f"negative-capabilities.json: {len(seen)} surfaces ({len(DOC)} NEG.DOC.*, {len(BANK)} banking refusals)")

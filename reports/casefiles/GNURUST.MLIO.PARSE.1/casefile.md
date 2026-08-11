<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.MLIO.PARSE.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** cobc XML PARSE ... PROCESSING PROCEDURE (libcob/mlio.c cob_xml_parse)
- **Byte domain(s):** an XML PARSE input field + cross-call state -> the XML-EVENT / XML-CODE / XML-TEXT special-register sequence a PROCESSING PROCEDURE observes
- **Replay:** `bash lab/oracle/ml_parse_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- a from-scratch, dependency-free Rust port of the XML PARSE state machine (mlio.rs cob_xml_parse/xml_parse/xml_free_parse_memory over the modelled XML special registers MlModule/XmlState + set_xml_event/set_xml_text/set_xml_code/get_xml_code/set_xml_exception), reproducing the GnuCOBOL 3.2 observable event stream WITHOUT linking libxml2. Crucial fidelity point: GnuCOBOL 3.2's XML PARSE is itself unimplemented -- even built WITH_XML2, mlio.c:xml_parse advances JUST_STARTED -> HAD_END_OF_INPUT directly (the xmlParseChunk and 'not implemented' branches are unreachable for an in-memory document), so a PROCESSING PROCEDURE observes exactly START-OF-DOCUMENT -> END-OF-INPUT -> END-OF-DOCUMENT, all with XML-CODE 0 and (in the default XMLNSS mode) empty XML-TEXT. ml_parse_sweep 2/0 confirms the native state machine emits that identical sequence
- a 'real' parser would DIVERGE from the authority. Empty/all-spaces items raise XML_INTERNAL_ERROR
- EVENT_EXCEPTION as in the C.

## Negative claims (6) — negative capability is the trust surface
- an actual XML tree/attribute/content parse (GnuCOBOL 3.2 does not parse, so neither claim nor port exists)
- JSON PARSE (no such statement in GnuCOBOL 3.2 mlio.c)
- the COMPAT (non-XMLNSS) full-document XML-TEXT delivery beyond the modelled path
- multi-chunk streaming
- schema VALIDATING against a real schema
- lie prevented: XML PARSE needs libxml2 -- NO: GnuCOBOL 3.2 XML PARSE does not actually parse; its observable behaviour is a fixed START-OF-DOCUMENT -> END-OF-INPUT -> END-OF-DOCUMENT event walk that a native Rust state machine reproduces exactly, no C library linked. Writing a real parser would be LESS faithful, not more.

## Damage if overclaimed
claiming 'XML PARSE works' would imply GnuCOBOL parses XML (it does not); the honest claim is byte-identity of the unimplemented-but-observable event stream

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.MLIO.GENERATE.1 (court-casefile)

**Verdict: PASS** · 5/5 pass, 0 fail · crate `gnucobol-rs` 0.7.67

- **Oracle:** cobc XML GENERATE / JSON GENERATE (libcob/mlio.c via libxml2 + json-c)
- **Byte domain(s):** a cob_ml_tree (names, group/leaf structure, numeric/alphanumeric content) -> the XML / JSON GENERATE output bytes
- **Replay:** `bash lab/oracle/ml_generate_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (7)
- a from-scratch, dependency-free Rust serializer of the cob_ml_tree (mlio.rs generate_xml_from_tree/generate_element/generate_normal_element/generate_hex_element/generate_content/generate_attributes/generate_normal_attribute + generate_json_from_tree + get_xml_name/get_xml_num/get_json_num/get_xml_data/get_trimmed_data + cob_xml_generate_new/cob_json_generate_new), proven to produce byte-identical output to the admitted GnuCOBOL's XML GENERATE / JSON GENERATE (which drives libxml2's xmlTextWriter + json-c) WITHOUT linking those libraries (ml_generate_sweep 2/0). Verified end-to-end against a cobc record with a signed numeric (S9(3) -42 -> -42), a scaled numeric (9(3)V99 12.50 -> 12.50), an alphanumeric needing XML escaping (a<b&c -> a&lt
- b&amp
- c in XML, the raw bytes quoted in JSON), and a nested group: XML <G><NEG>-42</NEG><DEC>12.50</DEC><SPC>a&lt
- b&amp
- c</SPC><GRP><X>hi</X><Y>7</Y></GRP></G>, JSON {"G":{"NEG":-42,"DEC":12.50,"SPC":"a<b&c","GRP":{"X":"hi","Y":7}}}. Numeric formatting strips leading zeros + inserts the decimal point at the scale
- XML content escapes &/</>
- JSON strings are quoted + json-escaped, numbers unquoted

## Negative claims (10) — negative capability is the trust surface
- XML PARSE is sealed separately (GNURUST.MLIO.PARSE.1)
- JSON PARSE does not exist in GnuCOBOL 3.2
- attributes/namespaces beyond the basic name="value" form
- the NATIONAL / UTF-16 content path
- pretty-printing / indentation options
- non-ASCII multibyte content escaping
- the exact bytes of the WITH XML-DECLARATION prolog (only the no-declaration path is oracle-swept)
- CDATA / comments / processing-instructions / DOCTYPE in the output
- SUPPRESS WHEN conditional predicates beyond the unconditional is_suppressed flag
- lie prevented: XML/JSON GENERATE needs libxml2/cJSON -- NO: the byte output is a deterministic walk of the cob_ml_tree (element brackets, &/</> escaping, leading-zero-stripped scaled numerics, quoted JSON strings) that a native Rust serializer reproduces exactly, no C library linked

## Damage if overclaimed
claiming the whole ML subsystem would hide that XML/JSON PARSE still needs libxml2/json-c, and that national encoding / pretty-print / overflow paths are unported

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

# GnuCOBOL dialect configuration (copied verbatim from the admitted GnuCOBOL 3.2 source)

These are the GnuCOBOL `config/` files copied **byte-for-byte** from the admitted GnuCOBOL 3.2 source
tree (LGPL-3.0-or-later, like this crate). They drive dialect behaviour: `default.conf`,
`cobol85/2002/2014.conf`, `ibm/mf/acu/bs2000/gcos/mvs/realia/rm/xopen[-strict].conf`, the matching
`*.words` reserved-word lists, the `*.ttbl` EBCDIC translation tables, and `runtime.cfg`. The Rust
config loader (`common_configload`, the port of `cob_load_config`) parses them natively -- see the
`config_files_parse_natively` test. Kept identical to GnuCOBOL so the port's dialect behaviour matches
the oracle. Do not edit by hand; refresh from the pinned source.

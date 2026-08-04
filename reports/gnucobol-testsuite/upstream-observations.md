# GnuCOBOL-suite upstream observations (baseline side)

The baseline run uses the ADMITTED GnuCOBOL 3.2 in-tree build with a stock configuration (no `-fpermissive`, no compat `-Wno-*` flags — those would leak cc1 warnings into stderr and break the suite's stderr-exact expectations). Any oracle-side failure is an observation about this exact build/environment, NOT a claim about upstream.

Oracle-side failures: 0 (each with a preserved group log under reports/gnucobol-testsuite/raw/).

Oracle-side skips: 9 (the suite's own AT_SKIP_IF conditions in this build).
- 0470: ACUCOBOL 32bit literal size (syn_literals.at:1379)
- 0535: function with variable-length RETURNING item (run_fundamental.at:1417)
- 0844: runtime check: write to internal storage (2) (run_misc.at:14197)
- 0922: INDEXED undeclared keys (run_file.at:7104)
- 0930: EXTFH: operation OP_GETINFO / QUERY-FILE (run_file.at:9665)
- 0935: EXTFH: auto-conversion FCD2 <-> FCD3 on 32bit (run_file.at:10869)
- 1110: UDF with recursion (run_functions.at:4594)
- 1182: run_extensions.at:4205: 1182. System routine CBL_GC_WAITPID (run_extensions.at:4205)
- 1188: System routine x'91' function NN (run_extensions.at:4788)

Oracle-side expected-failures: 31 (suite-marked xfail — the baseline 'failure' is the suite's own expectation).
- 0051: syn_copy.at:125: 51. COPY: relative copybooks (syn_copy.at:125)
- 0107: syn_definition.at:1069: 107. Redefinition of label and variable names (syn_definition.at:1069)
- 0116: syn_definition.at:1335: 116. OSVS register CURRENT-DATE and TIME-OF-DAY (syn_definition.at:1335)
- 0133: syn_definition.at:2571: 133. LIKE clause (syn_definition.at:2571)
- 0226: syn_file.at:2066: 226. ALTERNATE RECORD definition WITH NO DUPLICATES (syn_file.at:2066)
- 0227: syn_file.at:2096: 227. ALTERNATE RECORD definition omitting RECORD (syn_file.at:2096)
- 0323: syn_misc.at:3978: 323. checking prototypes and definitions match (syn_misc.at:3978)
- 0336: syn_misc.at:4675: 336. USE FOR DEBUGGING syntax-checks (2) (syn_misc.at:4675)
- 0350: syn_misc.at:5591: 350. Fall-Through to WHEN OTHER (syn_misc.at:5591)
- 0381: syn_misc.at:7083: 381. conditional directives with lvl 78 (1) (syn_misc.at:7083)
- 0382: syn_misc.at:7118: 382. conditional directives with lvl 78 (2) (syn_misc.at:7118)
- 0599: run_fundamental.at:5501: 599. USE FOR DEBUGGING, time of execution (run_fundamental.at:5501)
- 0676: run_misc.at:221: 676. CURRENCY SIGN WITH PICTURE SYMBOL (run_misc.at:221)
- 0682: run_misc.at:513: 682. EXTERNAL data item size mismatch (run_misc.at:513)
- 0898: run_file.at:5016: 898. EXTFH: LINAGE and LINAGE-COUNTER sample (run_file.at:5016)

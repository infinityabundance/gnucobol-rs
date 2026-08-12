# Diagnostic-unblocked transformations

_transformer gnurust-diag-unblocked-transform-v1 · source revision `stable-3.2` · pristine manifest `1022ce18b3df42267b53d567a243a41cc1804c03bf06c375c28662431e61dafd` · transformed manifest `758aea6317f0fd835d7c1adc2f20fc0ef3e3a55ab489b537671bd0515141f5f7`_

Only expected compiler-diagnostic streams become `ignore`; commands, exit statuses,
COBOL source, runtime output, generated-file expectations, environment, ordering
and skip/xfail semantics are unchanged. Nothing else in the suite is modified.

## transformations that ignore a stream

| source | group | step | line | command | status | stream | reason |
|---|---|---|---|---|---|---|---|
| configuration.at | 0 | 0 | 36 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 0 | 56 | `$COMPILE_ONLY -std=default prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 1 | 59 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 2 | 62 | `$COMPILE_ONLY -std=cobol2002 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 3 | 65 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 4 | 68 | `$COMPILE_ONLY -std=xopen prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 5 | 71 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 6 | 74 | `$COMPILE_ONLY -std=bs2000-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 7 | 77 | `$COMPILE_ONLY -std=ibm-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 8 | 80 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 9 | 83 | `$COMPILE_ONLY -std=rm-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 11 | 87 | `$COMPILE_ONLY -std=mvs-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 1 | 12 | 90 | `$COMPILE_ONLY -std=gcos-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 2 | 0 | 119 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 3 | 0 | 140 | `$COMPILE_ONLY -conf=cobol2014.conf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 5 | 0 | 203 | `$COMPILE_ONLY -conf=test.conf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 6 | 0 | 234 | `$COMPILE_ONLY -std=default -conf=test.conf prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 9 | 0 | 306 | `$COMPILE_ONLY -q \
-fcomment-paragraphsok prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 9 | 1 | 310 | `$COMPILE_ONLY \
-fassign-clause=cobol-2002 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 9 | 3 | 318 | `$COMPILE_ONLY \
-freserved-words=defaults prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 9 | 4 | 324 | `$COMPILE_ONLY \
-fword-length=thirty prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 10 | 0 | 351 | `$COMPILE_ONLY -conf=notthere.conf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 10 | 1 | 355 | `$COMPILE_ONLY -conf=defunc.conf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 10 | 2 | 360 | `$COMPILE_ONLY -conf=defunc2.conf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 11 | 0 | 396 | `$COMPILE_ONLY -conf=defunc.conf prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 22 | 0 | 846 | `$COMPILE_ONLY -fformat=unknown fixed.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 22 | 4 | 868 | `$COMPILE_ONLY -fformat=fixed wide.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 22 | 5 | 874 | `$COMPILE_ONLY -fformat=cobol85 wide.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 23 | 5 | 908 | `$COMPILE -C -febcdic-table=unknown prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 23 | 8 | 953 | `$COMPILE -C -febcdic-table=./invalid.ttbl prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 23 | 9 | 970 | `$COMPILE -C -febcdic-table=./shorter.ttbl prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 23 | 10 | 985 | `$COMPILE -C -febcdic-table=./shorter_longer.ttbl prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| configuration.at | 23 | 11 | 1007 | `$COMPILE -C -febcdic-table=./longer.ttbl prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| listings.at | 0 | 0 | 75 | `$COMPILE_ONLY -t prog.lst -tlines=2 prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| listings.at | 32 | 1 | 5131 | `$COMPILE_LISTING0 -Xref -T- -ftsymbols EDITOR.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 23 | 0 | 1695 | `$COMPILE -std=mf prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 27 | 0 | 1905 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 34 | 0 | 2274 | `$COMPILE_ONLY -fnot-register=TALLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 36 | 1 | 2366 | `$COMPILE_MODULE callee.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 50 | 0 | 3112 | `$COMPILE -std=cobol2002 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 51 | 0 | 3159 | `$COMPILE wrong.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 51 | 1 | 3162 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 78 | 0 | 4823 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 78 | 1 | 4829 | `$COMPILE -fself-call-recursive=error prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 78 | 3 | 4853 | `$COMPILE progc.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 84 | 0 | 5024 | `$COMPILE unknown.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 84 | 1 | 5027 | `$COMPILE lit.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_extensions.at | 88 | 4 | 5207 | `$COMPILE -fformat=terminal -fcomment-paragraphs=ok -fdebugging-line marginberr.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_file.at | 12 | 0 | 744 | `$COMPILE -fassign-clause=external prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_functions.at | 5 | 0 | 213 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_functions.at | 118 | 0 | 4358 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 10 | 0 | 547 | `$COMPILE_ONLY -fdiagnostics-show-option prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 10 | 1 | 551 | `$COMPILE_ONLY -Wstrict-typing -fdiagnostics-show-option prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 10 | 2 | 557 | `$COMPILE_ONLY -Wextra -Wno-strict-typing -fdiagnostics-show-option prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 14 | 0 | 904 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 65 | 0 | 4508 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 67 | 0 | 4608 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 94 | 0 | 6161 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 94 | 2 | 6183 | `$COMPILE -fno-constant-folding prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 97 | 0 | 6308 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 99 | 0 | 6440 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_fundamental.at | 101 | 0 | 6514 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 14 | 0 | 610 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 60 | 0 | 2396 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 85 | 0 | 3626 | `$COMPILE -fdefault-colseq=ascii -DEXPECT-ORDER=ASCII -o ascii prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 133 | 0 | 7375 | `$COMPILE -std=mf prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 134 | 0 | 7487 | `$COMPILE -std=mf -fno-move-non-numeric-lit-to-numeric-is-zero prog.cob cmod.c` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 142 | 0 | 11409 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 143 | 0 | 11445 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 144 | 0 | 11467 | `$COMPILE -o prog prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 144 | 1 | 11470 | `$COMPILE -free -o prog prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 145 | 0 | 11521 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 147 | 0 | 11647 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 150 | 0 | 11928 | `$COMPILE -Wno-unfinished prog.cob cmod.c` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 151 | 0 | 12158 | `$COMPILE -Wno-unfinished prog.cob cmod.c` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 153 | 0 | 12343 | `$COMPILE -frelax-syntax-checks prog.cob ` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 157 | 0 | 12666 | `$COMPILE -farithmetic-osvs prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 158 | 0 | 12742 | `$COMPILE -std=ibm prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 164 | 0 | 13174 | `$COMPILE -std=ibm prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 172 | 0 | 13932 | `$COMPILE -std=mf-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_misc.at | 172 | 1 | 13943 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_reportwriter.at | 12 | 0 | 3009 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_reportwriter.at | 12 | 1 | 3014 | `$COMPILE -std=mf prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_reportwriter.at | 16 | 0 | 3896 | `$COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_reportwriter.at | 19 | 0 | 4310 | `$COMPILE -std=cobol2002 -fassign-ext-dyn=ok progv.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| run_reportwriter.at | 19 | 2 | 4405 | `$COMPILE -std=cobol2002 -fdump=all -fassign-ext-dyn=ok progl.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 4 | 0 | 303 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 4 | 1 | 317 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 4 | 2 | 331 | `$COMPILE_ONLY prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 4 | 3 | 335 | `$COMPILE_ONLY -ffold-copy=lower prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 11 | 2 | 571 | `$COMPILE -fpartial-replace-when-literal-src=ok -o prog prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 11 | 3 | 581 | `$COMPILE_ONLY -fpartial-replace-when-literal-src=error prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 11 | 4 | 616 | `$COMPILE_ONLY -fpartial-replace-when-literal-src=skip prog_err.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 11 | 5 | 620 | `$COMPILE_ONLY -fpartial-replace-when-literal-src=unconformable prog_err.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 14 | 3 | 714 | `$COMPILE_ONLY -fpartial-replace-when-literal-src=ok prog_err.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 14 | 4 | 717 | `$COMPILE_ONLY -fpartial-replace-when-literal-src=skip prog_err.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 14 | 5 | 725 | `$COMPILE_ONLY -fpartial-replace-when-literal-src=ok prog_err2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 14 | 6 | 732 | `$COMPILE_ONLY -fpartial-replace-when-literal-src=ok prog_err3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 17 | 0 | 836 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_copy.at | 21 | 0 | 1027 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 0 | 0 | 30 | `$COMPILE_ONLY short.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 1 | 0 | 49 | `$COMPILE_ONLY SHORT.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 1 | 2 | 70 | `$COMPILE_ONLY SHORT3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 1 | 3 | 82 | `$COMPILE_ONLY SHORT4.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 2 | 0 | 99 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 3 | 0 | 116 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 5 | 0 | 166 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 6 | 0 | 187 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 7 | 0 | 210 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 8 | 0 | 232 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 9 | 0 | 250 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 10 | 0 | 314 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 11 | 1 | 367 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 12 | 0 | 397 | `$COBC -fdiagnostics-plain-output -fsyntax-only prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 12 | 1 | 401 | `$COBC -fdiagnostics-plain-output -fsyntax-only -Wall -Werror=goto-section prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 13 | 1 | 426 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 14 | 0 | 454 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 15 | 0 | 487 | `$COMPILE_ONLY prog2.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 16 | 0 | 507 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 17 | 0 | 527 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 18 | 0 | 548 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 19 | 1 | 625 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 20 | 0 | 653 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 20 | 1 | 657 | `$COMPILE_ONLY -fno-program-name-redefinition prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 21 | 0 | 683 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 22 | 1 | 704 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 23 | 0 | 723 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 24 | 1 | 807 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 24 | 2 | 810 | `$COMPILE_ONLY prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 24 | 3 | 814 | `$COMPILE_ONLY prog4.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 24 | 4 | 817 | `$COMPILE_ONLY prog5.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 26 | 0 | 869 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 27 | 0 | 894 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 28 | 0 | 922 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 30 | 1 | 977 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 31 | 0 | 1009 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 32 | 0 | 1030 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 33 | 0 | 1055 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 34 | 0 | 1095 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 36 | 0 | 1148 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 37 | 0 | 1180 | `$COMPILE_ONLY -frelax-level-hierarchy prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 37 | 1 | 1183 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 38 | 0 | 1205 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 39 | 0 | 1227 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 41 | 0 | 1278 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 42 | 1 | 1325 | `$COMPILE -fnot-register=return-code \
prog1.cob prog2.cob prog3.cob prog4.cob prog5.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 43 | 0 | 1376 | `$COMPILE_ONLY -std=cobol85 prog1.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 43 | 1 | 1380 | `$COMPILE_ONLY -std=ibm-strict prog1.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 43 | 2 | 1384 | `$COMPILE_ONLY -std=mf prog1.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 43 | 3 | 1389 | `$COMPILE_ONLY prog1.cob prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 43 | 4 | 1393 | `$COMPILE_ONLY -fregister=current-date,time-of-day \
  prog1.cob prog2.cob prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 44 | 0 | 1423 | `$COMPILE_ONLY -fincorrect-conf-sec-order=error prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 45 | 0 | 1438 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 47 | 0 | 1479 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 49 | 0 | 1602 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 49 | 1 | 1681 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 50 | 0 | 1768 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 51 | 0 | 1795 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 52 | 0 | 1815 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 54 | 0 | 1866 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 55 | 0 | 2036 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 55 | 1 | 2039 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 55 | 2 | 2043 | `$COMPILE_ONLY -Wno-unfinished prog3.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 56 | 0 | 2110 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 57 | 0 | 2145 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 58 | 1 | 2202 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 58 | 2 | 2209 | `$COMPILE_ONLY badprog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 59 | 1 | 2358 | `$COMPILE_ONLY -std=cobol2002 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 59 | 5 | 2376 | `$COMPILE_ONLY -std=mf-strict progstd.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 59 | 6 | 2394 | `$COMPILE_ONLY badprog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 60 | 1 | 2618 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 61 | 0 | 2721 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 62 | 0 | 2842 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_definition.at | 63 | 0 | 2864 | `$COMPILE_ONLY -std=default prog.cob ` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 0 | 0 | 43 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 1 | 0 | 72 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 2 | 0 | 102 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 3 | 0 | 131 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 7 | 0 | 416 | `$COMPILE_ONLY -fassign-variable=warning -fassign-using-variable=warning -fassign-ext-dyn=warning -fassign-disk-from=warning prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 7 | 1 | 424 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 8 | 0 | 452 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 9 | 0 | 488 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 10 | 0 | 530 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 10 | 1 | 536 | `$COMPILE_ONLY -Werror=obsolete -fdiagnostics-show-option prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 11 | 0 | 575 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 13 | 0 | 683 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 14 | 0 | 779 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 14 | 1 | 783 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 14 | 2 | 787 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 14 | 3 | 793 | `$COMPILE_ONLY -std=cobol2014 prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 14 | 4 | 798 | `$COMPILE_ONLY prog3.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 15 | 0 | 887 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 15 | 1 | 893 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 15 | 2 | 899 | `$COMPILE_ONLY -frecord-contains-depending-clause=error prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 15 | 3 | 903 | `$COMPILE_ONLY -frecord-contains-depending-clause=ok prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 16 | 1 | 964 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 16 | 3 | 976 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 16 | 4 | 986 | `$COMPILE_ONLY -std=cobol2014 -frelax-syntax prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 17 | 0 | 1031 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 19 | 0 | 1091 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 20 | 0 | 1133 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 21 | 0 | 1172 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 22 | 0 | 1226 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 23 | 0 | 1258 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 24 | 0 | 1314 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 25 | 0 | 1340 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 26 | 0 | 1446 | `$COMPILE_ONLY -frecord-delim-with-fixed-recs=warning prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 27 | 0 | 1539 | `$COMPILE_ONLY -fodoslide prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 28 | 1 | 1593 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 28 | 2 | 1596 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 29 | 0 | 1644 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 30 | 0 | 1686 | `$COMPILE_ONLY -Wadditional prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 31 | 0 | 1728 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 31 | 1 | 1738 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 31 | 2 | 1744 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 32 | 0 | 1782 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 33 | 0 | 1803 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 34 | 0 | 1829 | `$COMPILE_ONLY -Wimplicit-define prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 37 | 0 | 1963 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 40 | 0 | 2089 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 41 | 0 | 2118 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 42 | 0 | 2203 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 42 | 1 | 2209 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_file.at | 44 | 0 | 2280 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 0 | 0 | 39 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 0 | 1 | 57 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 3 | 0 | 161 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 4 | 0 | 198 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 5 | 0 | 225 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 6 | 0 | 248 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 7 | 1 | 272 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 8 | 1 | 315 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 9 | 0 | 351 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 10 | 0 | 382 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 11 | 0 | 426 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 12 | 0 | 494 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 13 | 0 | 536 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 14 | 0 | 560 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_functions.at | 15 | 0 | 585 | `$COMPILE -Wno-pending prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 0 | 0 | 569 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 0 | 1 | 575 | `$COMPILE_ONLY -t prog.lst prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 1 | 0 | 787 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 1 | 1 | 792 | `$COMPILE_ONLY -fliteral-length=160 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 1 | 2 | 797 | `$COMPILE_ONLY -free prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 1 | 3 | 804 | `$COMPILE_ONLY -t prog.lst prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 1 | 4 | 810 | `$COMPILE_ONLY -free -t prog2.lst prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 2 | 1 | 978 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 2 | 2 | 986 | `$COMPILE_ONLY prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 3 | 0 | 1060 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 3 | 1 | 1081 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 4 | 0 | 1121 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 4 | 1 | 1129 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 5 | 0 | 1155 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 5 | 1 | 1166 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 6 | 0 | 1194 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 6 | 1 | 1206 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 7 | 0 | 1234 | `$COMPILE_ONLY -std=mf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 7 | 1 | 1242 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 8 | 0 | 1267 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 8 | 1 | 1272 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 9 | 0 | 1298 | `$COMPILE_ONLY -Wno-unfinished -fhp-octal-literals=ok prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 9 | 1 | 1306 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 10 | 0 | 1338 | `$COMPILE_ONLY -std=acu prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 10 | 1 | 1361 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 11 | 0 | 1401 | `$COMPILE_ONLY -std=acu prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 12 | 0 | 1438 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 12 | 1 | 1451 | `$COMPILE_ONLY -fzero-length-literals=error -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 13 | 0 | 1497 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 14 | 0 | 1517 | `$COMPILE_ONLY -w prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 15 | 0 | 1539 | `$COMPILE -febcdic-symbolic-characters prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 15 | 2 | 1554 | `$COMPILE prog2.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_literals.at | 15 | 3 | 1557 | `$COMPILE -febcdic-symbolic-characters -febcdic-table=dummyNotThere prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 0 | 0 | 46 | `$COMPILE_ONLY -Wno-constant-numlit-expression prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 2 | 0 | 292 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 3 | 0 | 420 | `$COMPILE_ONLY -fdiagnostics-show-option prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 4 | 0 | 527 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 5 | 0 | 604 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 6 | 0 | 673 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 8 | 0 | 743 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 8 | 1 | 753 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 9 | 0 | 779 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 9 | 1 | 784 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 10 | 4 | 826 | `$COMPILE_ONLY prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 10 | 5 | 830 | `$COMPILE -fdiagnostics-plain-output -frelax-syntax-checks prog3.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 10 | 6 | 834 | `$COBC -fdiagnostics-plain-output -frelax-syntax-checks prog3.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 11 | 0 | 860 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 12 | 0 | 895 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 13 | 0 | 931 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 14 | 0 | 958 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 15 | 0 | 980 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 16 | 0 | 1009 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 17 | 0 | 1039 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 18 | 0 | 1062 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 19 | 0 | 1091 | `$COMPILE_ONLY -freserved=EXAMINE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 20 | 1 | 1139 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 21 | 0 | 1192 | `$COMPILE_ONLY -Wunreachable prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 22 | 0 | 1236 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 23 | 0 | 1329 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 23 | 1 | 1332 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 23 | 4 | 1337 | `$COMPILE_ONLY prog5.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 24 | 0 | 1429 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 24 | 1 | 1432 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 24 | 2 | 1435 | `$COMPILE_ONLY prog3.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 24 | 3 | 1438 | `$COMPILE_ONLY prog4.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 24 | 4 | 1443 | `$COMPILE_ONLY prog5.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 24 | 5 | 1447 | `$COMPILE_ONLY prog6.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 25 | 0 | 1541 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 25 | 1 | 1568 | `$COMPILE_ONLY -std=acu-strict -fsystem-name=SW1 -fno-relax-syntax-checks prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 26 | 0 | 1601 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 27 | 0 | 1627 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 28 | 0 | 1649 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 29 | 1 | 1679 | `$COMPILE_ONLY -fnot-reserved=COMMAND-LINE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 35 | 0 | 1826 | `$COMPILE_ONLY -Wextra prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 36 | 1 | 1860 | `$COBC -fsyntax-only -fdiagnostics-plain-output -fixed -Wextra prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 36 | 2 | 1866 | `$COBC -fsyntax-only -fdiagnostics-plain-output -free -Wextra prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 36 | 3 | 1873 | `$COBC -fsyntax-only -fdiagnostics-plain-output -F -Wextra prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 36 | 4 | 1881 | `$COBC -fsyntax-only -fdiagnostics-plain-output -F -std=default -Wextra prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 37 | 0 | 1903 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 37 | 1 | 1910 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Wextra -fixed prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 37 | 2 | 1913 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Wextra -free prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 38 | 0 | 1936 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 38 | 1 | 1939 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 39 | 2 | 2013 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 39 | 5 | 2030 | `$COMPILE_ONLY -fmfcomment prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 39 | 10 | 2055 | `$COMPILE_ONLY -free prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 39 | 11 | 2063 | `$COMPILE_ONLY -free -fmfcomment prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 39 | 12 | 2071 | `$COMPILE_ONLY -free -facucomment prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 39 | 13 | 2078 | `$COMPILE_ONLY -fformat=terminal -facucomment prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 39 | 14 | 2085 | `$COMPILE_ONLY -fixed prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 40 | 0 | 2196 | `$COMPILE_ONLY -free -fword-length=31 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 40 | 1 | 2212 | `$COMPILE_ONLY -free -fword-length=45 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 40 | 2 | 2224 | `$COMPILE_ONLY -free -fword-length=60 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 40 | 3 | 2232 | `$COMPILE_ONLY -free -fword-length=45 -frelax-syntax-checks prog2.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 40 | 5 | 2242 | `$COMPILE_ONLY -free -fword-length=31 -frelax-syntax-checks prog2.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 40 | 6 | 2254 | `$COMPILE_ONLY -fword-length=59 prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 41 | 0 | 2307 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 41 | 1 | 2328 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 43 | 2 | 2403 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 44 | 0 | 2477 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 44 | 1 | 2496 | `$COMPILE_ONLY -std=rm-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 45 | 0 | 2572 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 45 | 1 | 2586 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 46 | 0 | 2668 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 47 | 0 | 2715 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 48 | 0 | 2737 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 49 | 0 | 2788 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 50 | 0 | 2820 | `$COMPILE_ONLY -freserved=hello,foo,bars,background-color -fnot-reserved=file prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 52 | 1 | 2887 | `$COMPILE_ONLY -fnot-reserved=DISPLAY -freserved=COMP-1=DISPLAY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 53 | 0 | 2907 | `$COMPILE_ONLY -std=ibm-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 54 | 1 | 2944 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 55 | 0 | 2966 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 56 | 1 | 2989 | `$COMPILE_ONLY -std=acu prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 57 | 1 | 3050 | `$COMPILE_ONLY -std=acu prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 57 | 2 | 3057 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 58 | 0 | 3104 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 58 | 3 | 3109 | `$COMPILE_ONLY prog3.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 59 | 0 | 3171 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 60 | 0 | 3196 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 61 | 0 | 3227 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 62 | 0 | 3257 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 62 | 1 | 3262 | `$COMPILE_ONLY -fformat=cobol85 -fmissing-period=error prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 62 | 2 | 3266 | `$COMPILE -fformat=cobol85 prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 63 | 0 | 3314 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 63 | 1 | 3321 | `$COMPILE -fformat=cobol85 prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 63 | 3 | 3351 | `$COMPILE_ONLY -fformat=cobol85 prog_err.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 64 | 1 | 3387 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 65 | 0 | 3436 | `$COMPILE -std=cobol85 -fmissing-period=warning prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 65 | 2 | 3461 | `$COMPILE -std=gcos prog_err.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 66 | 0 | 3496 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 67 | 0 | 3546 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 68 | 0 | 3617 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 69 | 0 | 3675 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 70 | 0 | 3715 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 71 | 0 | 3774 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 72 | 0 | 3847 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 73 | 0 | 3892 | `$COMPILE -x -std=mf -debug -Wall prog.cob ` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 74 | 0 | 3940 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 75 | 0 | 3969 | `$COMPILE_ONLY -fprogram-prototypes=warning prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 76 | 0 | 4100 | `$COMPILE_ONLY -Wno-unfinished -Wno-pending prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 77 | 0 | 4148 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 78 | 0 | 4185 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 79 | 0 | 4209 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 79 | 1 | 4225 | `$COMPILE_ONLY prog2.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 79 | 2 | 4254 | `$COMPILE_ONLY prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 80 | 0 | 4277 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 80 | 1 | 4280 | `$COMPILE_ONLY -std=cobol85 -freserved=EXTERN,C prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 81 | 0 | 4337 | `$COMPILE_ONLY -Wlinkage prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 82 | 0 | 4360 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 83 | 0 | 4385 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 84 | 0 | 4423 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 85 | 0 | 4471 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 86 | 0 | 4522 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 88 | 1 | 4615 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 88 | 2 | 4619 | `$COMPILE_ONLY -std=acu-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 88 | 4 | 4641 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 88 | 5 | 4653 | `$COMPILE_ONLY -std=cobol85 prog2.cob \
-freserved="debug-item,debug-name,debug-line,debug-contents" \
-freserved="debug-sub-1,debug-sub-2,debug-sub-3" \
` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 89 | 0 | 4722 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 89 | 1 | 4733 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 92 | 0 | 4882 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 93 | 0 | 4909 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 93 | 1 | 4915 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 93 | 2 | 4923 | `$COMPILE_ONLY -std=mf-strict -freserved=CONSTANT prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 93 | 3 | 4929 | `$COMPILE_ONLY -std=mf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 94 | 0 | 4958 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 94 | 1 | 4964 | `$COMPILE_ONLY -std=ibm-strict prog.cob ` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 94 | 2 | 4976 | `$COMPILE_ONLY -std=ibm prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 95 | 0 | 5028 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 96 | 0 | 5068 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 96 | 1 | 5075 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 96 | 2 | 5082 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 98 | 0 | 5260 | `$COMPILE_ONLY prog.cob ` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 102 | 0 | 5564 | `$COMPILE_ONLY -w -fmissing-statement=error prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 102 | 1 | 5576 | `$COMPILE_ONLY -fno-constant-folding -fmissing-statement=warning prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 103 | 0 | 5619 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 103 | 1 | 5623 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 105 | 0 | 5676 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 106 | 0 | 5735 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 107 | 0 | 5760 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 108 | 0 | 5788 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 109 | 0 | 5815 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 109 | 1 | 5820 | `$COMPILE_ONLY -std=acu prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 109 | 2 | 5825 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 111 | 0 | 5874 | `$COMPILE_ONLY valid.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 112 | 0 | 5963 | `$COMPILE_ONLY valid.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 114 | 0 | 6023 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 115 | 0 | 6077 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 116 | 0 | 6257 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 117 | 0 | 6357 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 117 | 1 | 6375 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 118 | 0 | 6423 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 118 | 1 | 6429 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 119 | 0 | 6471 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 119 | 1 | 6483 | `$COMPILE_ONLY -free prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 120 | 0 | 6543 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 121 | 0 | 6601 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 123 | 0 | 6748 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 124 | 0 | 6783 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 128 | 3 | 6936 | `$COMPILE_ONLY -std=cobol2002 prog3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 133 | 0 | 7074 | `$COMPILE_ONLY -D X prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 138 | 1 | 7265 | `$COMPILE_ONLY -Wall -fsection-exit-check prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 141 | 3 | 7731 | `$COMPILE replace.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 142 | 2 | 7774 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 143 | 0 | 7801 | `$COMPILE -fcontrol-division=ok prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 143 | 2 | 7806 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 144 | 1 | 7842 | `$COMPILE -fcontrol-division=ok empty0.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 144 | 4 | 7885 | `$COMPILE -fcontrol-division=ok prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 145 | 0 | 7918 | `$COMPILE -std=gcos-strict prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 146 | 0 | 7944 | `$COMPILE_ONLY prog_extraneous_depending.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 146 | 1 | 7959 | `$COMPILE_ONLY prog_missing_depending.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 146 | 2 | 7976 | `$COMPILE_ONLY prog_value.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 146 | 3 | 7997 | `$COMPILE_ONLY prog_errs.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 146 | 4 | 8018 | `$COMPILE_ONLY -fpicture-l=warning prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 147 | 0 | 8050 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 147 | 1 | 8056 | `$COMPILE_ONLY -std=cobol85 -frelax-syntax-checks prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 148 | 0 | 8089 | `$COMPILE_ONLY -std=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 148 | 1 | 8098 | `$COMPILE_ONLY -std=cobol85 -fmissing-period=ok prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 148 | 2 | 8104 | `$COMPILE_ONLY -fformat=cobol85 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 149 | 0 | 8135 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 149 | 1 | 8158 | `$COMPILE_ONLY -fformat=cobol85 cobol85.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 150 | 0 | 8192 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 150 | 1 | 8199 | `$COMPILE_ONLY -fformat=cobol85 prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 151 | 0 | 8235 | `$COMPILE_ONLY -std=mvs pgm1.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 151 | 1 | 8241 | `$COMPILE_ONLY -std=mvs-strict pgm1.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 152 | 3 | 8286 | `$COMPILE_ONLY -fformat=auto free.cob fixed1.cob fixed2.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 152 | 4 | 8289 | `$COMPILE_ONLY -fformat=auto domfree.cob domfixed.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 152 | 5 | 8297 | `$COMPILE_ONLY -fformat=fixed free.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_misc.at | 152 | 6 | 8307 | `$COMPILE_ONLY -fformat=fixed domfree.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 0 | 0 | 54 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 1 | 0 | 78 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 2 | 0 | 116 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 4 | 0 | 181 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 5 | 0 | 215 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 6 | 0 | 248 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 7 | 0 | 286 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 8 | 0 | 321 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 9 | 0 | 358 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 10 | 0 | 394 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 11 | 0 | 421 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 12 | 0 | 446 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 13 | 1 | 510 | `$COMPILE -fdiagnostics-show-option prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 13 | 2 | 526 | `$COMPILE -fdiagnostics-show-option -Wpossible-overlap prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 14 | 0 | 572 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 15 | 0 | 606 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 16 | 0 | 644 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 17 | 0 | 685 | `$COMPILE_ONLY -std=cobol2002 -freserved=COMP-1:FLOAT prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 17 | 1 | 703 | `$COMPILE_ONLY -std=ibm prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 17 | 2 | 721 | `$COMPILE_ONLY -std=mf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_move.at | 17 | 3 | 736 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_multiply.at | 0 | 0 | 50 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_multiply.at | 1 | 0 | 86 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_multiply.at | 2 | 0 | 123 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 0 | 0 | 45 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 0 | 2 | 52 | `$COMPILE_ONLY -ftop-level-occurs-clause=warning prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 1 | 0 | 96 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 2 | 0 | 114 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 3 | 0 | 135 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 4 | 0 | 215 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 4 | 2 | 221 | `$COMPILE_ONLY -fcomplex-odo prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 4 | 4 | 226 | `$COMPILE_ONLY prog3.cob prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 4 | 5 | 231 | `$COMPILE_ONLY prog4.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 4 | 6 | 236 | `$COMPILE_ONLY -fcomplex-odo prog4.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 4 | 7 | 240 | `$COMPILE_ONLY prog5.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 5 | 2 | 335 | `$COMPILE_ONLY -std=cobol2014  prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 5 | 4 | 341 | `$COMPILE_ONLY bad.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 5 | 5 | 345 | `$COMPILE_ONLY bad2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 5 | 6 | 349 | `$COMPILE_ONLY bad3.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 6 | 0 | 377 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 8 | 0 | 455 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 9 | 0 | 497 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 10 | 0 | 537 | `$COMPILE_ONLY -std=cobol2014 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 10 | 1 | 542 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 10 | 2 | 547 | `$COMPILE_ONLY -frelax-syntax prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 11 | 0 | 585 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_occurs.at | 11 | 1 | 590 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 0 | 0 | 40 | `$COMPILE_ONLY -ffree-redefines-position=error prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 0 | 1 | 44 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 1 | 0 | 68 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 2 | 0 | 90 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 3 | 0 | 112 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 4 | 0 | 133 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 5 | 0 | 163 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 6 | 0 | 189 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 7 | 0 | 212 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 8 | 0 | 243 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 9 | 0 | 271 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 10 | 0 | 297 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 12 | 0 | 381 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 13 | 0 | 411 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 15 | 0 | 483 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_redefines.at | 16 | 0 | 519 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_refmod.at | 0 | 0 | 48 | `$COMPILE -fdiagnostics-show-option prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_refmod.at | 1 | 0 | 71 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_refmod.at | 2 | 0 | 109 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_refmod.at | 3 | 0 | 147 | `$COMPILE_ONLY -fdiagnostics-show-option -Wno-constant-numlit-expression prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 0 | 0 | 115 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 1 | 0 | 167 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 2 | 0 | 211 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 3 | 0 | 253 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 4 | 0 | 303 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 5 | 0 | 408 | `$COMPILE_ONLY -std=mf-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 5 | 1 | 417 | `$COMPILE_ONLY -std=cobol2002 -fassign-ext-dyn=ok prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 6 | 0 | 512 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 7 | 0 | 548 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 8 | 0 | 589 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 8 | 1 | 592 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_reportwriter.at | 10 | 0 | 683 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 0 | 0 | 79 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 1 | 0 | 109 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 2 | 0 | 165 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 3 | 0 | 207 | `$COMPILE_ONLY -faccept-display-extensions=error prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 4 | 0 | 243 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 5 | 0 | 273 | `$COMPILE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 6 | 0 | 305 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 7 | 0 | 334 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 9 | 0 | 393 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 10 | 0 | 432 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 10 | 1 | 446 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 11 | 0 | 496 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 12 | 0 | 544 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 12 | 1 | 549 | `$COMPILE_ONLY prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 12 | 2 | 556 | `$COMPILE_ONLY prog3.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 13 | 0 | 584 | `$COMPILE_ONLY -std=cobol2002 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 14 | 0 | 609 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 15 | 0 | 657 | `$COMPILE_ONLY -fscreen-section-rules=std prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 15 | 1 | 672 | `$COMPILE_ONLY -fscreen-section-rules=acu prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 15 | 2 | 688 | `$COMPILE_ONLY -fscreen-section-rules=mf prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 15 | 3 | 715 | `$COMPILE_ONLY -fscreen-section-rules=rm prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 15 | 4 | 733 | `$COMPILE_ONLY -fscreen-section-rules=xopen prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 15 | 5 | 758 | `$COMPILE_ONLY -fscreen-section-rules=gc prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 16 | 0 | 809 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 17 | 0 | 845 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 17 | 1 | 853 | `$COMPILE_ONLY -fnot-reserved=MESSAGE prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_screen.at | 18 | 0 | 880 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_set.at | 0 | 0 | 45 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_set.at | 1 | 0 | 73 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_subscripts.at | 0 | 0 | 42 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_subscripts.at | 1 | 0 | 81 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_subscripts.at | 1 | 1 | 88 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_subscripts.at | 2 | 0 | 115 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_subscripts.at | 3 | 0 | 143 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_subscripts.at | 4 | 0 | 178 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_subscripts.at | 4 | 1 | 185 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_subscripts.at | 5 | 0 | 217 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 0 | 0 | 54 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 0 | 1 | 62 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 1 | 0 | 140 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 2 | 0 | 179 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 3 | 0 | 205 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 4 | 0 | 233 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 5 | 0 | 261 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 6 | 0 | 288 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 7 | 0 | 315 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 8 | 0 | 343 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 9 | 0 | 369 | `$COMPILE_ONLY -Wno-unfinished prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 10 | 0 | 407 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 10 | 1 | 411 | `$COMPILE_ONLY -std=ibm-strict prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 10 | 2 | 416 | `$COMPILE_ONLY -std=ibm prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 11 | 0 | 441 | `$COMPILE_ONLY prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| syn_value.at | 12 | 0 | 514 | `$COMPILE_ONLY -frelax-syntax-checks prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 1 | 71 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Wadditional prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 2 | 74 | `$COBC -fsyntax-only -fdiagnostics-plain-output -fno-diagnostics-show-option -Wadditional prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 3 | 78 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Wall prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 4 | 83 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Wextra prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 5 | 92 | `$COBC -fsyntax-only -fdiagnostics-plain-output -W prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 7 | 103 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Werror=additional prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 8 | 106 | `$COBC -fsyntax-only -fdiagnostics-plain-output -fno-diagnostics-show-option -Werror=additional prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 11 | 113 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Werror=additional -Wno-error prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 12 | 116 | `$COBC -fsyntax-only -fdiagnostics-plain-output -w -Werror=additional prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 13 | 119 | `$COBC -fsyntax-only -fdiagnostics-plain-output -w -Wadditional prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 14 | 122 | `$COBC -fsyntax-only -fdiagnostics-plain-output -w -Wpossible-truncate prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 15 | 153 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Wall prog2.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 16 | 158 | `$COBC -fsyntax-only -fdiagnostics-plain-output -fmax-errors=0 prog2.cob` | 97 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 17 | 165 | `$COBC -fsyntax-only -fdiagnostics-plain-output -Wfatal-errors prog2.cob` | 97 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 1 | 18 | 172 | `$COBC -q -Wfatal-errors=123 prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 6 | 0 | 367 | `$COMPILE_ONLY prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 7 | 0 | 390 | `TMPDIR="" TMP="notthere" TEMP="" $COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 7 | 2 | 394 | `TMPDIR="" TMP="" TEMP="./prog.cob" $COMPILE prog.cob` | 0 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 10 | 0 | 473 | `$COMPILE -q --thisoptiondoesntexist prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 10 | 1 | 477 | `$COMPILE -q -flagdoesntexist prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 18 | 0 | 808 | `$COMPILE -j prog.cob` | 1 | stderr | compiler step; stderr expectation is purely compiler diagnostic text |
| used_binaries.at | 21 | 1 | 866 | `$COMPILE -jdg prog.cob` | 0 | stdout | compiler step; stdout expectation is purely compiler diagnostic text |

# GnuCOBOL testsuite — wrapper-option unsupported census

**180 first-failure groups** whose primary classification is `WRAPPER_OPTION_UNSUPPORTED`, decomposed by the first unsupported option token in the candidate invocation. Native-code modes and unmodeled flags are rejected honestly (never silently ignored); tests that later require native artifacts retain a typed boundary. Counting unit: **first_failure_group**.

## Tokens

- 35: `-fttitle=GnuCOBOL_V.R.P:`
- 9: `-fnotrunc:`
- 6: `-fodoslide:`
- 4: `-C:`
- 4: `-fcontrol-division=ok:`
- 4: `-frelax-syntax-checks:`
- 3: `-c:`
- 3: `-fassign-clause=external:`
- 3: `-fbinary-size=1--8:`
- 3: `-fcomment-paragraphs=ok:`
- 3: `-fdefault-colseq=ascii:`
- 3: `-fdump=ALL:`
- 3: `-fmissing-statement=ok:`
- 3: `-freserved=EXAMINE:`
- 2: `-faccept-display-extensions=error:`
- 2: `-fbinary-size=1-2-4-8:`
- 2: `-fbinary-size=2-4-8:`
- 2: `-fcallfh=TSTFH:`
- 2: `-fdebugging-line:`
- 2: `-fdefine-constant-directive=ok:`
- 2: `-fdpc-in-data=none:`
- 2: `-febcdic-symbolic-characters:`
- 2: `-fno-fast-compare:`
- 2: `-fno-section-exit-check:`
- 2: `-fnot-intrinsic=substitute:`
- 2: `-fsign=ascii:`
- 2: `-ftraceall:`
- 2: `-jd:`
- 1: `--ffold-call=upper:`
- 1: `--list-reserved:`
- 1: `-P-:`
- 1: `-S:`
- 1: `-b:`
- 1: `-faccept-auto:`
- 1: `-faccept-update:`
- 1: `-facu-literal=ok:`
- 1: `-facu-literals=ok:`
- 1: `-farithmetic-osvs:`
- 1: `-fassign-clause=dynamic:`
- 1: `-fassign-ext-dyn=ok:`
- 1: `-fassign-variable=warning:`
- 1: `-fbinary-comp-1:`
- 1: `-fcallfh=EXTFH:`
- 1: `-fcomplex-odo:`
- 1: `-fconstant-folding:`
- 1: `-fentry-statement=ok:`
- 1: `-fformat=�C:`
- 1: `-ffree-redefines-position=error:`
- 1: `-fhp-octal-literals=ok:`
- 1: `-fincorrect-conf-sec-order=error:`
- 1: `-fintrinsics=all:`
- 1: `"-fintrinsics=pi,e:"`
- 1: `-flarger-redefines=ok:`
- 1: `-fliteral-length=1:`
- 1: `-fmissing-period=warning:`
- 1: `-fmissing-statement=error:`
- 1: `-fmove-ibm:`
- 1: `-fno-areacheck:`
- 1: `-fno-binary-truncate:`
- 1: `-fno-constant-folding:`
- 1: `-fno-ec=BOUND-REF-MOD:`
- 1: `-fno-ec=program-arg-mismatch:`
- 1: `-fno-gen-c-decl-static-call:`
- 1: `-fno-implicit-goback-check:`
- 1: `-fno-move-non-numeric-lit-to-numeric-is-zero:`
- 1: `-fno-program-name-redefinition:`
- 1: `-fno-recursive-check:`
- 1: `-fno-trunc:`
- 1: `-fnot-register=TALLY:`
- 1: `-fnot-register=return-code:`
- 1: `-fnot-reserved=COMMAND-LINE:`
- 1: `-fnot-reserved=ID:`
- 1: `"-fnot-reserved=double,float,new,volatile,xor:"`
- 1: `-fpartial-replace-when-literal-src=ok:`
- 1: `-fpartial-replace-when-literal-src=skip:`
- 1: `-fperform-osvs:`
- 1: `-fpretty-display:`
- 1: `-fprogram-prototypes=warning:`
- 1: `-frecord-delim-with-fixed-recs=warning:`
- 1: `-frelax-level-hierarchy:`
- 1: `-freserved=COMP-1:FLOAT:`
- 1: `-freserved=COMP-1=DISPLAY:`
- 1: `-freserved=FOO=DISPLAY*:`
- 1: `-freserved=XX*=BYTE-LENGTH:`
- 1: `"-freserved=hello,foo,bars,background-color:"`
- 1: `-fscreen-section-rules=std:`
- 1: `-fsign=ebcdic:`
- 1: `-fsticky-linkage:`
- 1: `-fstop-error-statement=ok:`
- 1: `"-fsystem-name=sw1,"`
- 1: `-ftop-level-occurs-clause=ok:`
- 1: `-fusing-optional=skip:`
- 1: `-fword-length=31:`
- 1: `-t:`

## Per-test ledger (counting_unit = first_failure_group)

| id | title | group | token | reason |
|---|---|---|---|---|
| 0001 | used_binaries.at:27: 1. compiler help and information | used_binaries.at:27 | --list-reserved: | cobc-rs rejected an option in: $COBC --list-reserved |
| 0003 | used_binaries.at:179: 3. compiler outputs (general) | used_binaries.at:179 | -C: | cobc-rs rejected an option in: $COBC -C prog.cob |
| 0004 | (no status line in group log) |  | -fformat=�C: | cobc-rs rejected an option in: $COBC -I sub/copy prog.c -o prog.$COB_OBJECT_EXT |
| 0006 | used_binaries.at:329: 6. compiler outputs (assembler) | used_binaries.at:329 | -S: | cobc-rs rejected an option in: $COBC -v -S prog.cob |
| 0017 | used_binaries.at:716: 17. cobcrun -M DSO entry multiple arguments | used_binaries.at:716 | -b: | cobc-rs rejected an option in: $COBC -b ${FLAGS} mainer.cob called.cob |
| 0020 | used_binaries.at:815: 20. run job after compilation | used_binaries.at:815 | -jd: | cobc-rs rejected an option in: $COMPILE -jd prog.cob |
| 0021 | used_binaries.at:832: 21. run job after compilation (path specified) | used_binaries.at:832 | -jd: | "cobc-rs rejected an option in: $COMPILE_MODULE -jd -o $(_return_path ""sub/prog"") prog.cob" |
| 0033 | configuration.at:241: 33. cobc compiler flag on command line | configuration.at:241 | -fcomment-paragraphs=ok: | cobc-rs rejected an option in: $COMPILE_ONLY -fcomment-paragraphs=ok prog.cob |
| 0034 | configuration.at:260: 34. cobc compiler flag on command line (priority) | configuration.at:260 | -fcomment-paragraphs=ok: | cobc-rs rejected an option in: $COMPILE_ONLY \ |
| 0049 | configuration.at:885: 49. cobc configuration: ebcdic-table | configuration.at:885 | -C: | cobc-rs rejected an option in: $COMPILE -C -febcdic-table=default prog.cob |
| 0061 | syn_copy.at:528: 61. COPY: partial replacement BY literal | syn_copy.at:528 | -fpartial-replace-when-literal-src=skip: | cobc-rs rejected an option in: $COMPILE -fpartial-replace-when-literal-src=skip -o prog-skip prog.cob |
| 0064 | syn_copy.at:686: 64. REPLACE: partial replacement BY SPACE | syn_copy.at:686 | -fpartial-replace-when-literal-src=ok: | cobc-rs rejected an option in: $COMPILE_ONLY -fpartial-replace-when-literal-src=ok prog.cob |
| 0070 | syn_copy.at:923: 70. COPY and REPLACE in same file | syn_copy.at:923 | -P-: | cobc-rs rejected an option in: $COMPILE_ONLY -P- prog.cob |
| 0092 | syn_definition.at:556: 92. Redefinition of program-name by other programs | syn_definition.at:556 | --ffold-call=upper: | cobc-rs rejected an option in: $COMPILE_ONLY --ffold-call=upper -fdiagnostics-show-option prog.cob |
| 0110 | syn_definition.at:1158: 110. Non-matching level numbers (extension) | syn_definition.at:1158 | -frelax-level-hierarchy: | cobc-rs rejected an option in: $COMPILE_ONLY -frelax-level-hierarchy prog.cob |
| 0115 | syn_definition.at:1284: 115. RETURNING in STOP RUN / GOBACK / EXIT PROGRAM | syn_definition.at:1284 | -fnot-register=return-code: | cobc-rs rejected an option in: $COMPILE -fnot-register=return-code \ |
| 0117 | syn_definition.at:1405: 117. Invalid ENVIRONMENT DIVISION order | syn_definition.at:1405 | -fincorrect-conf-sec-order=error: | cobc-rs rejected an option in: $COMPILE_ONLY -fincorrect-conf-sec-order=error prog.cob |
| 0156 | syn_redefines.at:28: 156. REDEFINES: not following entry-name | syn_redefines.at:28 | -ffree-redefines-position=error: | cobc-rs rejected an option in: $COMPILE_ONLY -ffree-redefines-position=error prog.cob |
| 0185 | syn_value.at:494: 185. Implicit picture from value | syn_value.at:494 | -frelax-syntax-checks: | cobc-rs rejected an option in: $COMPILE_ONLY -frelax-syntax-checks prog.cob |
| 0193 | syn_file.at:326: 193. ASSIGN to variable | syn_file.at:326 | -fassign-variable=warning: | cobc-rs rejected an option in: $COMPILE_ONLY -fassign-variable=warning -fassign-using-variable=warning -fassign-ext-dyn=warning -fassign-disk-from=warning prog.cob |
| 0212 | syn_file.at:1352: 212. RECORD DELIMITER | syn_file.at:1352 | -frecord-delim-with-fixed-recs=warning: | cobc-rs rejected an option in: $COMPILE_ONLY -frecord-delim-with-fixed-recs=warning prog.cob |
| 0213 | syn_file.at:1460: 213. FILE STATUS | syn_file.at:1460 | -fodoslide: | cobc-rs rejected an option in: $COMPILE_ONLY -fodoslide prog.cob |
| 0221 | syn_file.at:1836: 221. ASSIGN external-name matching filename | syn_file.at:1836 | -fassign-clause=external: | cobc-rs rejected an option in: $COMPILE_ONLY -fassign-clause=external prog.cob |
| 0254 | syn_misc.at:683: 254. Valid conditional expression | syn_misc.at:683 | -fno-constant-folding: | cobc-rs rejected an option in: $COMPILE_ONLY -fno-constant-folding prog.cob |
| 0266 | syn_misc.at:1068: 266. EXAMINE invalid literals | syn_misc.at:1068 | -freserved=EXAMINE: | cobc-rs rejected an option in: $COMPILE_ONLY -freserved=EXAMINE prog.cob |
| 0276 | syn_misc.at:1656: 276. unknown device in dialect | syn_misc.at:1656 | -fnot-reserved=COMMAND-LINE: | cobc-rs rejected an option in: $COMPILE_ONLY -fnot-reserved=COMMAND-LINE prog.cob |
| 0277 | syn_misc.at:1686: 277. ACCEPT WITH ( NO ) UPDATE / DEFAULT | syn_misc.at:1686 | -faccept-update: | cobc-rs rejected an option in: $COMPILE_ONLY -faccept-update prog.cob |
| 0278 | syn_misc.at:1711: 278. ACCEPT WITH AUTO / TAB | syn_misc.at:1711 | -faccept-auto: | cobc-rs rejected an option in: $COMPILE_ONLY -faccept-auto prog.cob |
| 0287 | syn_misc.at:2101: 287. word length | syn_misc.at:2101 | -fword-length=31: | cobc-rs rejected an option in: $COMPILE_ONLY -free -fword-length=31 prog.cob |
| 0297 | syn_misc.at:2800: 297. adding/removing reserved words | syn_misc.at:2800 | "-freserved=hello,foo,bars,background-color:" | "cobc-rs rejected an option in: $COMPILE_ONLY -freserved=hello,foo,bars,background-color -fnot-reserved=file prog.cob" |
| 0298 | syn_misc.at:2830: 298. adding aliases | syn_misc.at:2830 | -freserved=FOO=DISPLAY*: | cobc-rs rejected an option in: $COMPILE_ONLY -freserved=FOO=DISPLAY* -freserved=BARS:FOO prog.cob |
| 0299 | syn_misc.at:2864: 299. overriding default words | syn_misc.at:2864 | -freserved=COMP-1=DISPLAY: | cobc-rs rejected an option in: $COMPILE_ONLY -freserved=COMP-1=DISPLAY prog.cob |
| 0312 | syn_misc.at:3406: 312. pseudotext replacement with text in area A | syn_misc.at:3406 | -fmissing-period=warning: | cobc-rs rejected an option in: $COMPILE -std=cobol85 -fmissing-period=warning prog.cob |
| 0322 | syn_misc.at:3951: 322. use of program-prototype-names | syn_misc.at:3951 | -fprogram-prototypes=warning: | cobc-rs rejected an option in: $COMPILE_ONLY -fprogram-prototypes=warning prog.cob |
| 0337 | syn_misc.at:4748: 337. Empty PERFORM with DEBUGGING MODE | syn_misc.at:4748 | -fmissing-statement=ok: | cobc-rs rejected an option in: $COMPILE_ONLY -fmissing-statement=ok prog.cob |
| 0348 | syn_misc.at:5399: 348. Constant Expressions (5) | syn_misc.at:5399 | -C: | cobc-rs rejected an option in: $COMPILE_ONLY -fdiagnostics-show-option -C -fno-remove-unreachable prog.cob |
| 0349 | syn_misc.at:5499: 349. Missing imperative statements | syn_misc.at:5499 | -fmissing-statement=error: | cobc-rs rejected an option in: $COMPILE_ONLY -w -fmissing-statement=error prog.cob |
| 0387 | syn_misc.at:7423: 387. field-tree via COBC_GEN_DUMP_COMMENTS | syn_misc.at:7423 | -C: | cobc-rs rejected an option in: COBC_GEN_DUMP_COMMENTS=1 \ |
| 0388 | syn_misc.at:7703: 388. CONTROL DIVISION | syn_misc.at:7703 | -fcontrol-division=ok: | cobc-rs rejected an option in: $COMPILE_ONLY -fcontrol-division=ok empty.cob |
| 0389 | syn_misc.at:7755: 389. CONTROL: empty default section | syn_misc.at:7755 | -fcontrol-division=ok: | cobc-rs rejected an option in: $COMPILE -fcontrol-division=ok prog.cob |
| 0390 | syn_misc.at:7781: 390. CONTROL: default section | syn_misc.at:7781 | -fcontrol-division=ok: | cobc-rs rejected an option in: $COMPILE -fcontrol-division=ok prog.cob |
| 0391 | syn_misc.at:7815: 391. CONTROL: substitution & default section | syn_misc.at:7815 | -fcontrol-division=ok: | cobc-rs rejected an option in: $COMPILE -fcontrol-division=ok empties.cob |
| 0400 | syn_misc.at:8323: 400. context sensitive alias | syn_misc.at:8323 | -freserved=XX*=BYTE-LENGTH: | "cobc-rs rejected an option in: $COMPILE -freserved=""XX*=BYTE-LENGTH"" prog.cob" |
| 0418 | syn_move.at:653: 418. MOVE FIGURATIVE to NUMERIC | syn_move.at:653 | -freserved=COMP-1:FLOAT: | cobc-rs rejected an option in: $COMPILE_ONLY -std=cobol2002 -freserved=COMP-1:FLOAT prog.cob |
| 0425 | syn_screen.at:175: 425. ACCEPT/DISPLAY extensions detection | syn_screen.at:175 | -faccept-display-extensions=error: | cobc-rs rejected an option in: $COMPILE_ONLY -faccept-display-extensions=error prog.cob |
| 0437 | syn_screen.at:616: 437. Compiler-specific SCREEN SECTION clause rules | syn_screen.at:616 | -fscreen-section-rules=std: | cobc-rs rejected an option in: $COMPILE_ONLY -fscreen-section-rules=std prog.cob |
| 0451 | syn_functions.at:280: 451. Intrinsic functions: replaced | syn_functions.at:280 | -fnot-intrinsic=substitute: | cobc-rs rejected an option in: $COMPILE_ONLY -fnot-intrinsic=substitute prog.cob |
| 0461 | syn_literals.at:819: 461. numeric literals | syn_literals.at:819 | -fliteral-length=1: | cobc-rs rejected an option in: $COMPILE_ONLY -fliteral-length=1 -fnumeric-literal-length=1 -fword-length=60 prog.cob |
| 0468 | syn_literals.at:1281: 468. HP COBOL octal literals | syn_literals.at:1281 | -fhp-octal-literals=ok: | cobc-rs rejected an option in: $COMPILE_ONLY -Wno-unfinished -fhp-octal-literals=ok prog.cob |
| 0474 | syn_literals.at:1525: 474. GCOS literals with EBCDIC symbols (syntax) | syn_literals.at:1525 | -febcdic-symbolic-characters: | cobc-rs rejected an option in: $COMPILE -febcdic-symbolic-characters prog.cob |
| 0475 | listings.at:21: 475. Minimal lines per listing pages | listings.at:21 | -t: | cobc-rs rejected an option in: $COMPILE_ONLY -t prog.lst -tlines=2 prog.cob |
| 0476 | listings.at:85: 476. COPY within comment | listings.at:85 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- prog.cob |
| 0477 | listings.at:149: 477. Replacement w/o strings | listings.at:149 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0478 | listings.at:205: 478. Partial replacement with literals | listings.at:205 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -fpartial-replace-when-literal-src=skip -t- prog.cob |
| 0479 | listings.at:269: 479. COPY replacement with partial match | listings.at:269 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- prog.cob |
| 0480 | listings.at:318: 480. COPY replacement with multiple partial matches | listings.at:318 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- prog.cob |
| 0481 | listings.at:517: 481. COPY replacement order | listings.at:517 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0482 | listings.at:608: 482. COPY separators | listings.at:608 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0483 | listings.at:667: 483. COPY partial replacement | listings.at:667 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0484 | listings.at:869: 484. COPY LEADING replacement | listings.at:869 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0485 | listings.at:932: 485. COPY TRAILING replacement | listings.at:932 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0486 | listings.at:996: 486. COPY recursive replacement | listings.at:996 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0487 | listings.at:1055: 487. COPY multiple files | listings.at:1055 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols tstcpybk.cob |
| 0488 | listings.at:1269: 488. Error/Warning messages | listings.at:1269 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING -Wimplicit-define -t- prog.cob |
| 0489 | listings.at:1590: 489. Two source files | listings.at:1590 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING -t- prog.cob prog1.cob |
| 0490 | listings.at:1651: 490. Multiple programs in one file | listings.at:1651 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE $LISTING_FLAGS -t prog.lst -ftsymbols prog.cob |
| 0491 | listings.at:1860: 491. Multiple programs in one compilation group | listings.at:1860 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE $LISTING_FLAGS -Wunreachable -t prog.lst -Xref -ftsymbols prog-1.cob prog-2.cob |
| 0492 | listings.at:2038: 492. command line | listings.at:2038 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COBC $LISTING_FLAGS -q -fsyntax-only -t- -fno-theader -ftcmd prog.cob |
| 0493 | listings.at:2102: 493. Wide listing | listings.at:2102 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING -T- prog.cob |
| 0494 | listings.at:2178: 494. Symbols: simple | listings.at:2178 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -fno-tmessages -ftsymbols prog.cob |
| 0495 | listings.at:2320: 495. Symbols: pointer | listings.at:2320 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t prog.lst -ftsymbols prog.cob |
| 0496 | listings.at:2598: 496. Symbols: multiple programs/functions | listings.at:2598 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING -t- -ftsymbols prog.cob |
| 0497 | listings.at:2718: 497. Symbols: OCCURS and REDEFINES | listings.at:2718 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING -fcomplex-odo -t- -fno-tsource -ftsymbols prog.cob |
| 0498 | listings.at:2808: 498. Conditional compilation | listings.at:2808 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING -DACTIVATE2 -t- prog.cob |
| 0499 | listings.at:2907: 499. File descriptions | listings.at:2907 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0500 | listings.at:3245: 500. Invalid PICTURE strings | listings.at:3245 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: diff expected.lst prog.lst |
| 0501 | listings.at:3689: 501. Variable format | listings.at:3689 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- prog.cob |
| 0502 | listings.at:3726: 502. MFCOMMENT | listings.at:3726 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- prog.cob |
| 0503 | listings.at:3788: 503. LISTING directive | listings.at:3788 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0504 | listings.at:3884: 504. LISTING directive free-form reference-format | listings.at:3884 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols -free prog.cob |
| 0505 | listings.at:3980: 505. Listing-directive statements | listings.at:3980 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING -t- -std=ibm prog.cob |
| 0506 | listings.at:4042: 506. Eject page | listings.at:4042 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING -t- prog.cob |
| 0507 | listings.at:4220: 507. Cross reference | listings.at:4220 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -Xref -t- -ftsymbols EDITOR.cob |
| 0508 | listings.at:5716: 508. Report Writer | listings.at:5716 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- -ftsymbols -Xref prog.cob |
| 0509 | listings.at:6018: 509. huge REPLACE | listings.at:6018 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- prog.cob |
| 0510 | listings.at:6928: 510. Long concatenated literal | listings.at:6928 | -fttitle=GnuCOBOL_V.R.P: | cobc-rs rejected an option in: $COMPILE_LISTING0 -t- prog.cob |
| 0526 | run_fundamental.at:958: 526. Overlapping MOVE (IBM) | run_fundamental.at:958 | -fmove-ibm: | cobc-rs rejected an option in: $COMPILE -fmove-ibm prog.cob |
| 0542 | run_fundamental.at:1784: 542. CALL alphanumeric data-name | run_fundamental.at:1784 | -fno-program-name-redefinition: | cobc-rs rejected an option in: $COMPILE -fno-program-name-redefinition prog.cob |
| 0553 | run_fundamental.at:2403: 553. Separate sign positions (2) | run_fundamental.at:2403 | -fpretty-display: | cobc-rs rejected an option in: $COMPILE_MODULE -fpretty-display prog.cob |
| 0588 | run_fundamental.at:5047: 588. debugging lines (-fdebugging-line) | run_fundamental.at:5047 | -fdebugging-line: | cobc-rs rejected an option in: $COMPILE -fdebugging-line prog.cob |
| 0591 | "run_fundamental.at:5119: 591. debugging lines, free format (-fdebugging-line)" | run_fundamental.at:5119 | -fdebugging-line: | cobc-rs rejected an option in: $COMPILE -free -fdebugging-line prog.cob |
| 0597 | run_fundamental.at:5380: 597. USE FOR DEBUGGING ON [ALL] REFERENCES OF field | run_fundamental.at:5380 | -fmissing-statement=ok: | cobc-rs rejected an option in: $COMPILE -fmissing-statement=ok prog.cob |
| 0601 | "run_fundamental.at:5584: 601. USE FOR DEBUGGING, referencing BASED item" | run_fundamental.at:5584 | -frelax-syntax-checks: | cobc-rs rejected an option in: $COMPILE -frelax-syntax-checks prog.cob |
| 0647 | run_refmod.at:313: 647. enable / disable ref-mod check | run_refmod.at:313 | -fno-ec=BOUND-REF-MOD: | cobc-rs rejected an option in: $COMPILE -w -fno-ec=BOUND-REF-MOD prog.cob |
| 0668 | run_initialize.at:655: 668. INITIALIZE to table-format VALUES ARE | run_initialize.at:655 | -fno-binary-truncate: | cobc-rs rejected an option in: $COMPILE -fno-binary-truncate  -fcomplex-odo -frelax-syntax-checks -w prog.cob |
| 0700 | run_misc.at:1172: 700. Dynamic call with static linking | run_misc.at:1172 | -c: | cobc-rs rejected an option in: $COMPILE_MODULE -c callee.cob |
| 0701 | run_misc.at:1201: 701. Static call with static linking | run_misc.at:1201 | -c: | cobc-rs rejected an option in: $COMPILE_MODULE -c callee.cob |
| 0703 | run_misc.at:1264: 703. Static CALL with ON EXCEPTION | run_misc.at:1264 | -c: | cobc-rs rejected an option in: $COMPILE_MODULE -c callee2.cob |
| 0707 | run_misc.at:1464: 707. Recursive CALL with RECURSIVE assumed | run_misc.at:1464 | -fno-recursive-check: | cobc-rs rejected an option in: $COMPILE_MODULE -fno-recursive-check callee.cob |
| 0733 | run_misc.at:2487: 733. EXIT SECTION | run_misc.at:2487 | -fno-section-exit-check: | cobc-rs rejected an option in: $COMPILE -fno-section-exit-check prog.cob |
| 0734 | run_misc.at:2528: 734. implicit GOBACK at end of PROCEDURE DIVISION | run_misc.at:2528 | -fno-implicit-goback-check: | cobc-rs rejected an option in: $COMPILE -fno-implicit-goback-check prog.cob |
| 0736 | run_misc.at:2592: 736. PERFORM inline (1) | run_misc.at:2592 | -fmissing-statement=ok: | cobc-rs rejected an option in: $COMPILE -fmissing-statement=ok prog.cob |
| 0737 | run_misc.at:2618: 737. PERFORM inline (2) | run_misc.at:2618 | -frelax-syntax-checks: | cobc-rs rejected an option in: $COMPILE -frelax-syntax-checks -w prog.cob |
| 0746 | run_misc.at:3045: 746. UNSTRING DELIMITER IN | run_misc.at:3045 | -ftop-level-occurs-clause=ok: | cobc-rs rejected an option in: $COMPILE -ftop-level-occurs-clause=ok prog.cob |
| 0754 | run_misc.at:3609: 754. Alphanum comparison with default COLLATING SEQUENCE | run_misc.at:3609 | -fdefault-colseq=ascii: | cobc-rs rejected an option in: $COMPILE -fdefault-colseq=ascii -DEXPECT-ORDER=ASCII -o ascii prog.cob |
| 0755 | run_misc.at:3637: 755. SORT: table with default COLLATING SEQUENCE | run_misc.at:3637 | -fdefault-colseq=ascii: | cobc-rs rejected an option in: $COMPILE -fdefault-colseq=ascii -DEXPECT-ORDER=ASCII -o ascii prog.cob |
| 0757 | run_misc.at:3927: 757. SEARCH ALL: table with default COLLATING SEQUENCE | run_misc.at:3927 | -fdefault-colseq=ascii: | cobc-rs rejected an option in: $COMPILE -fdefault-colseq=ascii -DEXPECT-ORDER=ASCII -o ascii prog.cob |
| 0759 | run_misc.at:4008: 759. PERFORM type OSVS | run_misc.at:4008 | -fperform-osvs: | cobc-rs rejected an option in: $COMPILE -fperform-osvs prog.cob |
| 0760 | run_misc.at:4043: 760. Sticky LINKAGE | run_misc.at:4043 | -fsticky-linkage: | cobc-rs rejected an option in: $COMPILE_MODULE -fsticky-linkage callee.cob |
| 0787 | run_misc.at:5381: 787. STOP ERROR | run_misc.at:5381 | -fstop-error-statement=ok: | cobc-rs rejected an option in: $COMPILE prog.cob -fstop-error-statement=ok |
| 0793 | run_misc.at:6534: 793. C/C++ reserved words/predefined identifiers | run_misc.at:6534 | "-fnot-reserved=double,float,new,volatile,xor:" | "cobc-rs rejected an option in: $COMPILE_MODULE -fnot-reserved=double,float,new,volatile,xor callee.cob" |
| 0796 | run_misc.at:7099: 796. CALL RETURNING POINTER | run_misc.at:7099 | -fno-gen-c-decl-static-call: | cobc-rs rejected an option in: $COMPILE -fno-gen-c-decl-static-call prog.cob |
| 0799 | run_misc.at:7235: 799. LINE/COLUMN 0 exceptions | run_misc.at:7235 | -faccept-display-extensions=error: | cobc-rs rejected an option in: $COMPILE  -faccept-display-extensions=error prog.cob |
| 0803 | run_misc.at:7400: 803. MF FIGURATIVE to NUMERIC | run_misc.at:7400 | -fno-move-non-numeric-lit-to-numeric-is-zero: | cobc-rs rejected an option in: $COMPILE -std=mf -fno-move-non-numeric-lit-to-numeric-is-zero prog.cob cmod.c |
| 0806 | run_misc.at:7609: 806. READY TRACE / RESET TRACE | run_misc.at:7609 | -ftraceall: | cobc-rs rejected an option in: COB_OLD_TRACE=y \ |
| 0807 | run_misc.at:8074: 807. Trace feature with subroutine | run_misc.at:8074 | -ftraceall: | cobc-rs rejected an option in: $COMPILE -ftraceall prog.cob |
| 0808 | run_misc.at:10199: 808. stack and dump feature | run_misc.at:10199 | -fdump=ALL: | cobc-rs rejected an option in: $COMPILE_MODULE -fdump=ALL prog.cob sub2.cob |
| 0809 | run_misc.at:11016: 809. dump feature with NULL address | run_misc.at:11016 | -fdump=ALL: | cobc-rs rejected an option in: $COMPILE -fdump=ALL prog.cob |
| 0822 | run_misc.at:12256: 822. OCCURS INDEXED ASCENDING | run_misc.at:12256 | -frelax-syntax-checks: | cobc-rs rejected an option in: $COMPILE -frelax-syntax-checks prog.cob  |
| 0826 | run_misc.at:12589: 826. OSVS Arithmetic (1) | run_misc.at:12589 | -farithmetic-osvs: | cobc-rs rejected an option in: $COMPILE -farithmetic-osvs prog.cob |
| 0829 | run_misc.at:12866: 829. DEFINE OVERRIDE | run_misc.at:12866 | -fdefine-constant-directive=ok: | cobc-rs rejected an option in: ENVPONY=WHITE $COMPILE prog.cob -fdefine-constant-directive=ok -DDPONY=Stallone |
| 0830 | run_misc.at:12932: 830. DEFINE Defaults | run_misc.at:12932 | -fdefine-constant-directive=ok: | cobc-rs rejected an option in: $COMPILE prog.cob -fdefine-constant-directive=ok |
| 0840 | run_misc.at:13804: 840. Constant Expressions | run_misc.at:13804 | -fconstant-folding: | cobc-rs rejected an option in: $COMPILE prog.cob -fconstant-folding -fremove-unreachable -w |
| 0843 | run_misc.at:14097: 843. runtime check: write to internal storage (1) | run_misc.at:14097 | -fno-ec=program-arg-mismatch: | cobc-rs rejected an option in: $COMPILE -fno-ec=program-arg-mismatch -fmemory-check=pointer caller.cob |
| 0847 | run_misc.at:14434: 847. compare numeric DISPLAY SPACE with ZERO | run_misc.at:14434 | -fno-fast-compare: | cobc-rs rejected an option in: $COMPILE -Wno-constant-expression -fno-fast-compare prog.cob |
| 0859 | run_file.at:671: 859. ASSIGN DYNAMIC and EXTERNAL | run_file.at:671 | -fassign-clause=external: | cobc-rs rejected an option in: $COMPILE -fassign-clause=external prog.cob |
| 0860 | run_file.at:719: 860. ASSIGN EXTERNAL parsing | run_file.at:719 | -fassign-clause=external: | cobc-rs rejected an option in: $COMPILE -fassign-clause=external prog.cob |
| 0861 | run_file.at:753: 861. ASSIGN directive | run_file.at:753 | -fassign-clause=dynamic: | cobc-rs rejected an option in: $COMPILE -fassign-clause=dynamic prog.cob |
| 0884 | run_file.at:3586: 884. DECLARATIVES procedure referencing (multiple) | run_file.at:3586 | -fno-section-exit-check: | cobc-rs rejected an option in: $COMPILE -fno-section-exit-check prog.cob |
| 0924 | run_file.at:7373: 924. EXTFH: using ISAM callback | run_file.at:7373 | -fcallfh=TSTFH: | cobc-rs rejected an option in: $COMPILE -fcallfh=TSTFH prog.cob cmod.c |
| 0926 | run_file.at:8686: 926. EXTFH: SEQUENTIAL files | run_file.at:8686 | -fcallfh=TSTFH: | cobc-rs rejected an option in: $COMPILE -fcallfh=TSTFH prog.cob cmod.c |
| 0927 | "run_file.at:9096: 927. EXTFH: LINE SEQUENTIAL files, direct EXTFH" | run_file.at:9096 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -fnotrunc prog.cob |
| 0928 | run_file.at:9275: 928. EXTFH: LINE SEQUENTIAL files (2) | run_file.at:9275 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -fnotrunc prog.cob |
| 0929 | run_file.at:9468: 929. EXTFH: FIXED SEQUENTIAL | run_file.at:9468 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -fnotrunc prog.cob |
| 0931 | run_file.at:9847: 931. EXTFH: changing record address | run_file.at:9847 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -fnotrunc progl.cob |
| 0932 | run_file.at:10168: 932. EXTFH: INDEXED with multiple keys | run_file.at:10168 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -fnotrunc -fodoslide prog.cob progs.cob |
| 0933 | run_file.at:10499: 933. EXTFH: RELATIVE files | run_file.at:10499 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -fnotrunc prog.cob |
| 0947 | "run_file.at:13267: 947. EXTFH: File SORT, LINE SEQUENTIAL variable records" | run_file.at:13267 | -fcallfh=EXTFH: | cobc-rs rejected an option in: $COMPILE -fcallfh=EXTFH prog.cob |
| 0973 | run_reportwriter.at:4217: 973. Report CODE and LIMIT COLUMNS | run_reportwriter.at:4217 | -fassign-ext-dyn=ok: | cobc-rs rejected an option in: $COMPILE -std=cobol2002 -fassign-ext-dyn=ok progv.cob |
| 0974 | run_reportwriter.at:4435: 974. Test Report dump DECLARATIVES | run_reportwriter.at:4435 | -fdump=ALL: | cobc-rs rejected an option in: $COMPILE -debug -fdump=ALL prog.cob |
| 1105 | run_functions.at:4413: 1105. Intrinsics without FUNCTION keyword (1) | run_functions.at:4413 | -fintrinsics=all: | cobc-rs rejected an option in: $COMPILE -fintrinsics=all prog.cob |
| 1106 | run_functions.at:4434: 1106. Intrinsics without FUNCTION keyword (2) | run_functions.at:4434 | "-fintrinsics=pi,e:" | "cobc-rs rejected an option in: $COMPILE -fintrinsics=pi,e prog.cob" |
| 1109 | run_functions.at:4551: 1109. UDF replacing intrinsic function | run_functions.at:4551 | -fnot-intrinsic=substitute: | cobc-rs rejected an option in: $COMPILE -fnot-intrinsic=substitute prog.cob |
| 1113 | run_extensions.at:101: 1113. ACUCOBOL literals | run_extensions.at:101 | -facu-literals=ok: | cobc-rs rejected an option in: $COMPILE -facu-literals=ok prog.cob |
| 1123 | run_extensions.at:609: 1123. Complex OCCURS DEPENDING ON (3) | run_extensions.at:609 | -fodoslide: | cobc-rs rejected an option in: $COMPILE -fodoslide prog.cob |
| 1124 | run_extensions.at:673: 1124. Complex OCCURS DEPENDING ON (4) | run_extensions.at:673 | -fcomplex-odo: | cobc-rs rejected an option in: $COMPILE -fcomplex-odo prog.cob |
| 1125 | run_extensions.at:739: 1125. Complex OCCURS DEPENDING ON (5) | run_extensions.at:739 | -fodoslide: | cobc-rs rejected an option in: $COMPILE -fodoslide prog.cob |
| 1126 | run_extensions.at:805: 1126. Complex OCCURS DEPENDING ON (6) | run_extensions.at:805 | -fodoslide: | cobc-rs rejected an option in: $COMPILE -fodoslide prog.cob |
| 1130 | run_extensions.at:1151: 1130. INITIALIZE OCCURS ODOSLIDE | run_extensions.at:1151 | -fodoslide: | cobc-rs rejected an option in: $COMPILE -fodoslide prog.cob |
| 1131 | run_extensions.at:1248: 1131. DEPENDING ON with ODOSLIDE | run_extensions.at:1248 | -fodoslide: | cobc-rs rejected an option in: $COBC -x -fodoslide prog.cob |
| 1144 | run_extensions.at:2172: 1144. NUMBER-OF-CALL-PARAMETERS | run_extensions.at:2172 | -fusing-optional=skip: | cobc-rs rejected an option in: $COMPILE_MODULE -fusing-optional=skip callee.cob |
| 1145 | run_extensions.at:2253: 1145. TALLY register | run_extensions.at:2253 | -fnot-register=TALLY: | cobc-rs rejected an option in: $COMPILE_ONLY -fnot-register=TALLY prog.cob |
| 1151 | run_extensions.at:2603: 1151. ENTRY | run_extensions.at:2603 | -fentry-statement=ok: | cobc-rs rejected an option in: $COMPILE_MODULE -fentry-statement=ok hello.cob |
| 1158 | run_extensions.at:2893: 1158. SWITCHES with non-standard names | run_extensions.at:2893 | "-fsystem-name=sw1," | "cobc-rs rejected an option in: $COMPILE -fsystem-name=""sw1, SwItCh\ b, SWITCH\ 25"" \" |
| 1159 | run_extensions.at:3006: 1159. Larger REDEFINES lengths | run_extensions.at:3006 | -flarger-redefines=ok: | cobc-rs rejected an option in: $COMPILE -flarger-redefines=ok -w prog.cob |
| 1198 | run_extensions.at:5113: 1198. X/Open free-form format | run_extensions.at:5113 | -fno-areacheck: | cobc-rs rejected an option in: $COMPILE -fformat=xopen -fno-areacheck prog.cob |
| 1199 | run_extensions.at:5162: 1199. TERMINAL format | run_extensions.at:5162 | -fcomment-paragraphs=ok: | cobc-rs rejected an option in: $COMPILE -fformat=terminal -fcomment-paragraphs=ok prog.cob |
| 1201 | run_extensions.at:5296: 1201. Binary COMP-1 (1) | run_extensions.at:5296 | -fbinary-comp-1: | cobc-rs rejected an option in: $COMPILE -fbinary-comp-1 prog.cob |
| 1204 | run_extensions.at:5393: 1204. Bit Operations | run_extensions.at:5393 | -facu-literal=ok: | cobc-rs rejected an option in: $COMPILE -facu-literal=ok -fno-trunc prog.cob |
| 1205 | run_extensions.at:5496: 1205. Bit Shift Operations | run_extensions.at:5496 | -fno-trunc: | cobc-rs rejected an option in: $COMPILE -std=mf -fno-trunc prog.cob |
| 1215 | run_extensions.at:6159: 1215. EXAMINE TALLYING | run_extensions.at:6159 | -freserved=EXAMINE: | cobc-rs rejected an option in: $COMPILE -freserved=EXAMINE prog.cob |
| 1216 | run_extensions.at:6216: 1216. EXAMINE REPLACING | run_extensions.at:6216 | -freserved=EXAMINE: | cobc-rs rejected an option in: $COMPILE -freserved=EXAMINE prog.cob |
| 1217 | run_extensions.at:6252: 1217. GCOS literals with EBCDIC symbols (run) | run_extensions.at:6252 | -febcdic-symbolic-characters: | cobc-rs rejected an option in: $COMPILE -febcdic-symbolic-characters -febcdic-table=ebcdic500_latin1 prog.cob |
| 1218 | run_ml.at:19: 1218. XML GENERATE general | run_ml.at:19 | -fnot-reserved=ID: | cobc-rs rejected an option in: $COMPILE -fnot-reserved=ID prog.cob |
| 1224 | run_ml.at:488: 1224. XML dpc-in-data config option | run_ml.at:488 | -fdpc-in-data=none: | cobc-rs rejected an option in: $COMPILE -fdpc-in-data=none prog.cob |
| 1231 | run_ml.at:868: 1231. JSON dpc-in-data config option | run_ml.at:868 | -fdpc-in-data=none: | cobc-rs rejected an option in: $COMPILE -fdpc-in-data=none prog.cob |
| 1232 | data_binary.at:23: 1232. BINARY: 2-4-8 big-endian | data_binary.at:23 | -fbinary-size=2-4-8: | cobc-rs rejected an option in: $COMPILE -fbinary-size=2-4-8 \ |
| 1233 | data_binary.at:205: 1233. BINARY: 2-4-8 native | data_binary.at:205 | -fbinary-size=2-4-8: | cobc-rs rejected an option in: $COMPILE -fbinary-size=2-4-8 \ |
| 1234 | data_binary.at:393: 1234. BINARY: 1-2-4-8 big-endian | data_binary.at:393 | -fbinary-size=1-2-4-8: | cobc-rs rejected an option in: $COMPILE -fbinary-size=1-2-4-8 \ |
| 1235 | data_binary.at:575: 1235. BINARY: 1-2-4-8 native | data_binary.at:575 | -fbinary-size=1-2-4-8: | cobc-rs rejected an option in: $COMPILE -fbinary-size=1-2-4-8 \ |
| 1236 | data_binary.at:763: 1236. BINARY: 1--8 big-endian | data_binary.at:763 | -fbinary-size=1--8: | cobc-rs rejected an option in: $COMPILE -fbinary-size=1--8 \ |
| 1237 | data_binary.at:945: 1237. BINARY: 1--8 native | data_binary.at:945 | -fbinary-size=1--8: | cobc-rs rejected an option in: $COMPILE -fbinary-size=1--8 \ |
| 1238 | data_binary.at:1133: 1238. BINARY: full-print | data_binary.at:1133 | -fbinary-size=1--8: | cobc-rs rejected an option in: $COMPILE -fbinary-size=1--8 \ |
| 1240 | data_binary.at:1215: 1240. BINARY: 64bit unsigned arithmetic notrunc | data_binary.at:1215 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -fnotrunc prog.cob |
| 1241 | data_binary.at:1242: 1241. BINARY: 64bit signed negative constant range | data_binary.at:1242 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -fnotrunc prog.cob |
| 1243 | data_binary.at:1323: 1243. COMP-4 No Truncate | data_binary.at:1323 | -fnotrunc: | cobc-rs rejected an option in: $COMPILE -w -fnotrunc prog.cob |
| 1250 | data_display.at:22: 1250. DISPLAY: Sign ASCII | data_display.at:22 | -fsign=ascii: | cobc-rs rejected an option in: $COMPILE -fsign=ascii prog.cob |
| 1251 | data_display.at:81: 1251. DISPLAY: Sign ASCII (2) | data_display.at:81 | -fsign=ascii: | cobc-rs rejected an option in: $COMPILE -fsign=ascii prog.cob |
| 1252 | data_display.at:127: 1252. DISPLAY: Sign EBCDIC | data_display.at:127 | -fsign=ebcdic: | cobc-rs rejected an option in: $COMPILE -fsign=ebcdic prog.cob |
| 1271 | data_packed.at:1432: 1271. COMP-6 used with MOVE | data_packed.at:1432 | -fno-fast-compare: | cobc-rs rejected an option in: $COMPILE -fno-fast-compare -C -o progalt.c prog.cob |

180 rows; generated by `gnucobol-rs-testsuite option-census generate` — do not edit by hand.

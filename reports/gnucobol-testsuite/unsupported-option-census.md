# GnuCOBOL testsuite — unsupported wrapper-option census

180 tests classified WRAPPER_OPTION_UNSUPPORTED, decomposed by the actual option
and its required behavior (native-artifact vs adapter-compatible vs diagnostic-only).

## Categories

- dialect/extension flag: 82
- other: 53
- listing: 35
- native-code mode: 5
- long option: 2
- job: 2
- embedded build: 1

## Top tokens

- 35: `-fttitle=GnuCOBOL_V.R.P:`
- 9: `-fnotrunc:`
- 6: `-fodoslide:`
- 4: `-C:`
- 4: `-frelax-syntax-checks:`
- 4: `-fcontrol-division=ok:`
- 3: `-fcomment-paragraphs=ok:`
- 3: `-fassign-clause=external:`
- 3: `-freserved=EXAMINE:`
- 3: `-fmissing-statement=ok:`
- 3: `-c:`
- 3: `-fdefault-colseq=ascii:`
- 3: `-fdump=ALL:`
- 3: `-fbinary-size=1--8:`
- 2: `-jd:`
- 2: `-faccept-display-extensions=error:`
- 2: `-fnot-intrinsic=substitute:`
- 2: `-febcdic-symbolic-characters:`
- 2: `-fdebugging-line:`
- 2: `-fno-section-exit-check:`
- 2: `-ftraceall:`
- 2: `-fdefine-constant-directive=ok:`
- 2: `-fno-fast-compare:`
- 2: `-fcallfh=TSTFH:`
- 2: `-fdpc-in-data=none:`

## Per-test ledger

| id | title | token | category | failing command |
|---|---|---|---|---|
| 0001 | used_binaries.at:27: 1. compiler help and informat | `--list-reserved:` | long option | $COBC --list-reserved |
| 0003 | used_binaries.at:179: 3. compiler outputs (general | `-C:` | native-code mode | $COBC -C prog.cob |
| 0004 | (no status line in group log) | `-fformat=�C:` | dialect/extension flag | $COBC -I sub/copy prog.c -o prog.$COB_OBJECT_EXT |
| 0006 | used_binaries.at:329: 6. compiler outputs (assembl | `-S:` | native-code mode | $COBC -v -S prog.cob |
| 0017 | used_binaries.at:716: 17. cobcrun -M DSO entry mul | `-b:` | embedded build | $COBC -b ${FLAGS} mainer.cob called.cob |
| 0020 | used_binaries.at:815: 20. run job after compilatio | `-jd:` | job | $COMPILE -jd prog.cob |
| 0021 | used_binaries.at:832: 21. run job after compilatio | `-jd:` | job | $COMPILE_MODULE -jd -o $(_return_path "sub/prog") prog. |
| 0033 | configuration.at:241: 33. cobc compiler flag on co | `-fcomment-paragraphs=ok:` | dialect/extension flag | $COMPILE_ONLY -fcomment-paragraphs=ok prog.cob |
| 0034 | configuration.at:260: 34. cobc compiler flag on co | `-fcomment-paragraphs=ok:` | dialect/extension flag | $COMPILE_ONLY \ |
| 0049 | configuration.at:885: 49. cobc configuration: ebcd | `-C:` | native-code mode | $COMPILE -C -febcdic-table=default prog.cob |
| 0061 | syn_copy.at:528: 61. COPY: partial replacement BY  | `-fpartial-replace-when-literal-src=skip:` | dialect/extension flag | $COMPILE -fpartial-replace-when-literal-src=skip -o pro |
| 0064 | syn_copy.at:686: 64. REPLACE: partial replacement  | `-fpartial-replace-when-literal-src=ok:` | dialect/extension flag | $COMPILE_ONLY -fpartial-replace-when-literal-src=ok pro |
| 0070 | syn_copy.at:923: 70. COPY and REPLACE in same file | `-P-:` | other | $COMPILE_ONLY -P- prog.cob |
| 0092 | syn_definition.at:556: 92. Redefinition of program | `--ffold-call=upper:` | long option | $COMPILE_ONLY --ffold-call=upper -fdiagnostics-show-opt |
| 0110 | syn_definition.at:1158: 110. Non-matching level nu | `-frelax-level-hierarchy:` | other | $COMPILE_ONLY -frelax-level-hierarchy prog.cob |
| 0115 | syn_definition.at:1284: 115. RETURNING in STOP RUN | `-fnot-register=return-code:` | dialect/extension flag | $COMPILE -fnot-register=return-code \ |
| 0117 | syn_definition.at:1405: 117. Invalid ENVIRONMENT D | `-fincorrect-conf-sec-order=error:` | dialect/extension flag | $COMPILE_ONLY -fincorrect-conf-sec-order=error prog.cob |
| 0156 | syn_redefines.at:28: 156. REDEFINES: not following | `-ffree-redefines-position=error:` | dialect/extension flag | $COMPILE_ONLY -ffree-redefines-position=error prog.cob |
| 0185 | syn_value.at:494: 185. Implicit picture from value | `-frelax-syntax-checks:` | other | $COMPILE_ONLY -frelax-syntax-checks prog.cob |
| 0193 | syn_file.at:326: 193. ASSIGN to variable | `-fassign-variable=warning:` | dialect/extension flag | $COMPILE_ONLY -fassign-variable=warning -fassign-using- |
| 0212 | syn_file.at:1352: 212. RECORD DELIMITER | `-frecord-delim-with-fixed-recs=warning:` | dialect/extension flag | $COMPILE_ONLY -frecord-delim-with-fixed-recs=warning pr |
| 0213 | syn_file.at:1460: 213. FILE STATUS | `-fodoslide:` | other | $COMPILE_ONLY -fodoslide prog.cob |
| 0221 | syn_file.at:1836: 221. ASSIGN external-name matchi | `-fassign-clause=external:` | dialect/extension flag | $COMPILE_ONLY -fassign-clause=external prog.cob |
| 0254 | syn_misc.at:683: 254. Valid conditional expression | `-fno-constant-folding:` | other | $COMPILE_ONLY -fno-constant-folding prog.cob |
| 0266 | syn_misc.at:1068: 266. EXAMINE invalid literals | `-freserved=EXAMINE:` | dialect/extension flag | $COMPILE_ONLY -freserved=EXAMINE prog.cob |
| 0276 | syn_misc.at:1656: 276. unknown device in dialect | `-fnot-reserved=COMMAND-LINE:` | dialect/extension flag | $COMPILE_ONLY -fnot-reserved=COMMAND-LINE prog.cob |
| 0277 | syn_misc.at:1686: 277. ACCEPT WITH ( NO ) UPDATE / | `-faccept-update:` | other | $COMPILE_ONLY -faccept-update prog.cob |
| 0278 | syn_misc.at:1711: 278. ACCEPT WITH AUTO / TAB | `-faccept-auto:` | other | $COMPILE_ONLY -faccept-auto prog.cob |
| 0287 | syn_misc.at:2101: 287. word length | `-fword-length=31:` | dialect/extension flag | $COMPILE_ONLY -free -fword-length=31 prog.cob |
| 0297 | syn_misc.at:2800: 297. adding/removing reserved wo | `-freserved=hello,foo,bars,background-color:` | dialect/extension flag | $COMPILE_ONLY -freserved=hello,foo,bars,background-colo |
| 0298 | syn_misc.at:2830: 298. adding aliases | `-freserved=FOO=DISPLAY*:` | dialect/extension flag | $COMPILE_ONLY -freserved=FOO=DISPLAY* -freserved=BARS:F |
| 0299 | syn_misc.at:2864: 299. overriding default words | `-freserved=COMP-1=DISPLAY:` | dialect/extension flag | $COMPILE_ONLY -freserved=COMP-1=DISPLAY prog.cob |
| 0312 | syn_misc.at:3406: 312. pseudotext replacement with | `-fmissing-period=warning:` | dialect/extension flag | $COMPILE -std=cobol85 -fmissing-period=warning prog.cob |
| 0322 | syn_misc.at:3951: 322. use of program-prototype-na | `-fprogram-prototypes=warning:` | dialect/extension flag | $COMPILE_ONLY -fprogram-prototypes=warning prog.cob |
| 0337 | syn_misc.at:4748: 337. Empty PERFORM with DEBUGGIN | `-fmissing-statement=ok:` | dialect/extension flag | $COMPILE_ONLY -fmissing-statement=ok prog.cob |
| 0348 | syn_misc.at:5399: 348. Constant Expressions (5) | `-C:` | native-code mode | $COMPILE_ONLY -fdiagnostics-show-option -C -fno-remove- |
| 0349 | syn_misc.at:5499: 349. Missing imperative statemen | `-fmissing-statement=error:` | dialect/extension flag | $COMPILE_ONLY -w -fmissing-statement=error prog.cob |
| 0387 | syn_misc.at:7423: 387. field-tree via COBC_GEN_DUM | `-C:` | native-code mode | COBC_GEN_DUMP_COMMENTS=1 \ |
| 0388 | syn_misc.at:7703: 388. CONTROL DIVISION | `-fcontrol-division=ok:` | dialect/extension flag | $COMPILE_ONLY -fcontrol-division=ok empty.cob |
| 0389 | syn_misc.at:7755: 389. CONTROL: empty default sect | `-fcontrol-division=ok:` | dialect/extension flag | $COMPILE -fcontrol-division=ok prog.cob |
| 0390 | syn_misc.at:7781: 390. CONTROL: default section | `-fcontrol-division=ok:` | dialect/extension flag | $COMPILE -fcontrol-division=ok prog.cob |
| 0391 | syn_misc.at:7815: 391. CONTROL: substitution & def | `-fcontrol-division=ok:` | dialect/extension flag | $COMPILE -fcontrol-division=ok empties.cob |
| 0400 | syn_misc.at:8323: 400. context sensitive alias | `-freserved=XX*=BYTE-LENGTH:` | dialect/extension flag | $COMPILE -freserved="XX*=BYTE-LENGTH" prog.cob |
| 0418 | syn_move.at:653: 418. MOVE FIGURATIVE to NUMERIC | `-freserved=COMP-1:FLOAT:` | dialect/extension flag | $COMPILE_ONLY -std=cobol2002 -freserved=COMP-1:FLOAT pr |
| 0425 | syn_screen.at:175: 425. ACCEPT/DISPLAY extensions  | `-faccept-display-extensions=error:` | dialect/extension flag | $COMPILE_ONLY -faccept-display-extensions=error prog.co |
| 0437 | syn_screen.at:616: 437. Compiler-specific SCREEN S | `-fscreen-section-rules=std:` | dialect/extension flag | $COMPILE_ONLY -fscreen-section-rules=std prog.cob |
| 0451 | syn_functions.at:280: 451. Intrinsic functions: re | `-fnot-intrinsic=substitute:` | dialect/extension flag | $COMPILE_ONLY -fnot-intrinsic=substitute prog.cob |
| 0461 | syn_literals.at:819: 461. numeric literals | `-fliteral-length=1:` | dialect/extension flag | $COMPILE_ONLY -fliteral-length=1 -fnumeric-literal-leng |
| 0468 | syn_literals.at:1281: 468. HP COBOL octal literals | `-fhp-octal-literals=ok:` | dialect/extension flag | $COMPILE_ONLY -Wno-unfinished -fhp-octal-literals=ok pr |
| 0474 | syn_literals.at:1525: 474. GCOS literals with EBCD | `-febcdic-symbolic-characters:` | other | $COMPILE -febcdic-symbolic-characters prog.cob |
| 0475 | listings.at:21: 475. Minimal lines per listing pag | `-t:` | listing | $COMPILE_ONLY -t prog.lst -tlines=2 prog.cob |
| 0476 | listings.at:85: 476. COPY within comment | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- prog.cob |
| 0477 | listings.at:149: 477. Replacement w/o strings | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0478 | listings.at:205: 478. Partial replacement with lit | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -fpartial-replace-when-literal-src=sk |
| 0479 | listings.at:269: 479. COPY replacement with partia | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- prog.cob |
| 0480 | listings.at:318: 480. COPY replacement with multip | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- prog.cob |
| 0481 | listings.at:517: 481. COPY replacement order | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0482 | listings.at:608: 482. COPY separators | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0483 | listings.at:667: 483. COPY partial replacement | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0484 | listings.at:869: 484. COPY LEADING replacement | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0485 | listings.at:932: 485. COPY TRAILING replacement | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0486 | listings.at:996: 486. COPY recursive replacement | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0487 | listings.at:1055: 487. COPY multiple files | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols tstcpybk.cob |
| 0488 | listings.at:1269: 488. Error/Warning messages | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING -Wimplicit-define -t- prog.cob |
| 0489 | listings.at:1590: 489. Two source files | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING -t- prog.cob prog1.cob |
| 0490 | listings.at:1651: 490. Multiple programs in one fi | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE $LISTING_FLAGS -t prog.lst -ftsymbols prog.cob |
| 0491 | listings.at:1860: 491. Multiple programs in one co | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE $LISTING_FLAGS -Wunreachable -t prog.lst -Xref |
| 0492 | listings.at:2038: 492. command line | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COBC $LISTING_FLAGS -q -fsyntax-only -t- -fno-theader  |
| 0493 | listings.at:2102: 493. Wide listing | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING -T- prog.cob |
| 0494 | listings.at:2178: 494. Symbols: simple | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -fno-tmessages -ftsymbols prog.co |
| 0495 | listings.at:2320: 495. Symbols: pointer | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t prog.lst -ftsymbols prog.cob |
| 0496 | listings.at:2598: 496. Symbols: multiple programs/ | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING -t- -ftsymbols prog.cob |
| 0497 | listings.at:2718: 497. Symbols: OCCURS and REDEFIN | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING -fcomplex-odo -t- -fno-tsource -ftsymb |
| 0498 | listings.at:2808: 498. Conditional compilation | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING -DACTIVATE2 -t- prog.cob |
| 0499 | listings.at:2907: 499. File descriptions | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0500 | listings.at:3245: 500. Invalid PICTURE strings | `-fttitle=GnuCOBOL_V.R.P:` | dialect/extension flag | diff expected.lst prog.lst |
| 0501 | listings.at:3689: 501. Variable format | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- prog.cob |
| 0502 | listings.at:3726: 502. MFCOMMENT | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- prog.cob |
| 0503 | listings.at:3788: 503. LISTING directive | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols prog.cob |
| 0504 | listings.at:3884: 504. LISTING directive free-form | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols -free prog.cob |
| 0505 | listings.at:3980: 505. Listing-directive statement | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING -t- -std=ibm prog.cob |
| 0506 | listings.at:4042: 506. Eject page | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING -t- prog.cob |
| 0507 | listings.at:4220: 507. Cross reference | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -Xref -t- -ftsymbols EDITOR.cob |
| 0508 | listings.at:5716: 508. Report Writer | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- -ftsymbols -Xref prog.cob |
| 0509 | listings.at:6018: 509. huge REPLACE | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- prog.cob |
| 0510 | listings.at:6928: 510. Long concatenated literal | `-fttitle=GnuCOBOL_V.R.P:` | listing | $COMPILE_LISTING0 -t- prog.cob |
| 0526 | run_fundamental.at:958: 526. Overlapping MOVE (IBM | `-fmove-ibm:` | other | $COMPILE -fmove-ibm prog.cob |
| 0542 | run_fundamental.at:1784: 542. CALL alphanumeric da | `-fno-program-name-redefinition:` | other | $COMPILE -fno-program-name-redefinition prog.cob |
| 0553 | run_fundamental.at:2403: 553. Separate sign positi | `-fpretty-display:` | other | $COMPILE_MODULE -fpretty-display prog.cob |
| 0588 | run_fundamental.at:5047: 588. debugging lines (-fd | `-fdebugging-line:` | other | $COMPILE -fdebugging-line prog.cob |
| 0591 | run_fundamental.at:5119: 591. debugging lines, fre | `-fdebugging-line:` | other | $COMPILE -free -fdebugging-line prog.cob |
| 0597 | run_fundamental.at:5380: 597. USE FOR DEBUGGING ON | `-fmissing-statement=ok:` | dialect/extension flag | $COMPILE -fmissing-statement=ok prog.cob |
| 0601 | run_fundamental.at:5584: 601. USE FOR DEBUGGING, r | `-frelax-syntax-checks:` | other | $COMPILE -frelax-syntax-checks prog.cob |
| 0647 | run_refmod.at:313: 647. enable / disable ref-mod c | `-fno-ec=BOUND-REF-MOD:` | dialect/extension flag | $COMPILE -w -fno-ec=BOUND-REF-MOD prog.cob |
| 0668 | run_initialize.at:655: 668. INITIALIZE to table-fo | `-fno-binary-truncate:` | other | $COMPILE -fno-binary-truncate  -fcomplex-odo -frelax-sy |
| 0700 | run_misc.at:1172: 700. Dynamic call with static li | `-c:` | other | $COMPILE_MODULE -c callee.cob |
| 0701 | run_misc.at:1201: 701. Static call with static lin | `-c:` | other | $COMPILE_MODULE -c callee.cob |
| 0703 | run_misc.at:1264: 703. Static CALL with ON EXCEPTI | `-c:` | other | $COMPILE_MODULE -c callee2.cob |
| 0707 | run_misc.at:1464: 707. Recursive CALL with RECURSI | `-fno-recursive-check:` | other | $COMPILE_MODULE -fno-recursive-check callee.cob |
| 0733 | run_misc.at:2487: 733. EXIT SECTION | `-fno-section-exit-check:` | other | $COMPILE -fno-section-exit-check prog.cob |
| 0734 | run_misc.at:2528: 734. implicit GOBACK at end of P | `-fno-implicit-goback-check:` | other | $COMPILE -fno-implicit-goback-check prog.cob |
| 0736 | run_misc.at:2592: 736. PERFORM inline (1) | `-fmissing-statement=ok:` | dialect/extension flag | $COMPILE -fmissing-statement=ok prog.cob |
| 0737 | run_misc.at:2618: 737. PERFORM inline (2) | `-frelax-syntax-checks:` | other | $COMPILE -frelax-syntax-checks -w prog.cob |
| 0746 | run_misc.at:3045: 746. UNSTRING DELIMITER IN | `-ftop-level-occurs-clause=ok:` | dialect/extension flag | $COMPILE -ftop-level-occurs-clause=ok prog.cob |
| 0754 | run_misc.at:3609: 754. Alphanum comparison with de | `-fdefault-colseq=ascii:` | dialect/extension flag | $COMPILE -fdefault-colseq=ascii -DEXPECT-ORDER=ASCII -o |
| 0755 | run_misc.at:3637: 755. SORT: table with default CO | `-fdefault-colseq=ascii:` | dialect/extension flag | $COMPILE -fdefault-colseq=ascii -DEXPECT-ORDER=ASCII -o |
| 0757 | run_misc.at:3927: 757. SEARCH ALL: table with defa | `-fdefault-colseq=ascii:` | dialect/extension flag | $COMPILE -fdefault-colseq=ascii -DEXPECT-ORDER=ASCII -o |
| 0759 | run_misc.at:4008: 759. PERFORM type OSVS | `-fperform-osvs:` | other | $COMPILE -fperform-osvs prog.cob |
| 0760 | run_misc.at:4043: 760. Sticky LINKAGE | `-fsticky-linkage:` | other | $COMPILE_MODULE -fsticky-linkage callee.cob |
| 0787 | run_misc.at:5381: 787. STOP ERROR | `-fstop-error-statement=ok:` | dialect/extension flag | $COMPILE prog.cob -fstop-error-statement=ok |
| 0793 | run_misc.at:6534: 793. C/C++ reserved words/predef | `-fnot-reserved=double,float,new,volatile,xor:` | dialect/extension flag | $COMPILE_MODULE -fnot-reserved=double,float,new,volatil |
| 0796 | run_misc.at:7099: 796. CALL RETURNING POINTER | `-fno-gen-c-decl-static-call:` | other | $COMPILE -fno-gen-c-decl-static-call prog.cob |
| 0799 | run_misc.at:7235: 799. LINE/COLUMN 0 exceptions | `-faccept-display-extensions=error:` | dialect/extension flag | $COMPILE  -faccept-display-extensions=error prog.cob |
| 0803 | run_misc.at:7400: 803. MF FIGURATIVE to NUMERIC | `-fno-move-non-numeric-lit-to-numeric-is-zero:` | other | $COMPILE -std=mf -fno-move-non-numeric-lit-to-numeric-i |
| 0806 | run_misc.at:7609: 806. READY TRACE / RESET TRACE | `-ftraceall:` | other | COB_OLD_TRACE=y \ |
| 0807 | run_misc.at:8074: 807. Trace feature with subrouti | `-ftraceall:` | other | $COMPILE -ftraceall prog.cob |
| 0808 | run_misc.at:10199: 808. stack and dump feature | `-fdump=ALL:` | dialect/extension flag | $COMPILE_MODULE -fdump=ALL prog.cob sub2.cob |
| 0809 | run_misc.at:11016: 809. dump feature with NULL add | `-fdump=ALL:` | dialect/extension flag | $COMPILE -fdump=ALL prog.cob |
| 0822 | run_misc.at:12256: 822. OCCURS INDEXED ASCENDING | `-frelax-syntax-checks:` | other | $COMPILE -frelax-syntax-checks prog.cob  |
| 0826 | run_misc.at:12589: 826. OSVS Arithmetic (1) | `-farithmetic-osvs:` | other | $COMPILE -farithmetic-osvs prog.cob |
| 0829 | run_misc.at:12866: 829. DEFINE OVERRIDE | `-fdefine-constant-directive=ok:` | dialect/extension flag | ENVPONY=WHITE $COMPILE prog.cob -fdefine-constant-direc |
| 0830 | run_misc.at:12932: 830. DEFINE Defaults | `-fdefine-constant-directive=ok:` | dialect/extension flag | $COMPILE prog.cob -fdefine-constant-directive=ok |
| 0840 | run_misc.at:13804: 840. Constant Expressions | `-fconstant-folding:` | other | $COMPILE prog.cob -fconstant-folding -fremove-unreachab |
| 0843 | run_misc.at:14097: 843. runtime check: write to in | `-fno-ec=program-arg-mismatch:` | dialect/extension flag | $COMPILE -fno-ec=program-arg-mismatch -fmemory-check=po |
| 0847 | run_misc.at:14434: 847. compare numeric DISPLAY SP | `-fno-fast-compare:` | other | $COMPILE -Wno-constant-expression -fno-fast-compare pro |
| 0859 | run_file.at:671: 859. ASSIGN DYNAMIC and EXTERNAL | `-fassign-clause=external:` | dialect/extension flag | $COMPILE -fassign-clause=external prog.cob |
| 0860 | run_file.at:719: 860. ASSIGN EXTERNAL parsing | `-fassign-clause=external:` | dialect/extension flag | $COMPILE -fassign-clause=external prog.cob |
| 0861 | run_file.at:753: 861. ASSIGN directive | `-fassign-clause=dynamic:` | dialect/extension flag | $COMPILE -fassign-clause=dynamic prog.cob |
| 0884 | run_file.at:3586: 884. DECLARATIVES procedure refe | `-fno-section-exit-check:` | other | $COMPILE -fno-section-exit-check prog.cob |
| 0924 | run_file.at:7373: 924. EXTFH: using ISAM callback | `-fcallfh=TSTFH:` | dialect/extension flag | $COMPILE -fcallfh=TSTFH prog.cob cmod.c |
| 0926 | run_file.at:8686: 926. EXTFH: SEQUENTIAL files | `-fcallfh=TSTFH:` | dialect/extension flag | $COMPILE -fcallfh=TSTFH prog.cob cmod.c |
| 0927 | run_file.at:9096: 927. EXTFH: LINE SEQUENTIAL file | `-fnotrunc:` | other | $COMPILE -fnotrunc prog.cob |
| 0928 | run_file.at:9275: 928. EXTFH: LINE SEQUENTIAL file | `-fnotrunc:` | other | $COMPILE -fnotrunc prog.cob |
| 0929 | run_file.at:9468: 929. EXTFH: FIXED SEQUENTIAL | `-fnotrunc:` | other | $COMPILE -fnotrunc prog.cob |
| 0931 | run_file.at:9847: 931. EXTFH: changing record addr | `-fnotrunc:` | other | $COMPILE -fnotrunc progl.cob |
| 0932 | run_file.at:10168: 932. EXTFH: INDEXED with multip | `-fnotrunc:` | other | $COMPILE -fnotrunc -fodoslide prog.cob progs.cob |
| 0933 | run_file.at:10499: 933. EXTFH: RELATIVE files | `-fnotrunc:` | other | $COMPILE -fnotrunc prog.cob |
| 0947 | run_file.at:13267: 947. EXTFH: File SORT, LINE SEQ | `-fcallfh=EXTFH:` | dialect/extension flag | $COMPILE -fcallfh=EXTFH prog.cob |
| 0973 | run_reportwriter.at:4217: 973. Report CODE and LIM | `-fassign-ext-dyn=ok:` | dialect/extension flag | $COMPILE -std=cobol2002 -fassign-ext-dyn=ok progv.cob |
| 0974 | run_reportwriter.at:4435: 974. Test Report dump DE | `-fdump=ALL:` | dialect/extension flag | $COMPILE -debug -fdump=ALL prog.cob |
| 1105 | run_functions.at:4413: 1105. Intrinsics without FU | `-fintrinsics=all:` | dialect/extension flag | $COMPILE -fintrinsics=all prog.cob |
| 1106 | run_functions.at:4434: 1106. Intrinsics without FU | `-fintrinsics=pi,e:` | dialect/extension flag | $COMPILE -fintrinsics=pi,e prog.cob |
| 1109 | run_functions.at:4551: 1109. UDF replacing intrins | `-fnot-intrinsic=substitute:` | dialect/extension flag | $COMPILE -fnot-intrinsic=substitute prog.cob |
| 1113 | run_extensions.at:101: 1113. ACUCOBOL literals | `-facu-literals=ok:` | dialect/extension flag | $COMPILE -facu-literals=ok prog.cob |
| 1123 | run_extensions.at:609: 1123. Complex OCCURS DEPEND | `-fodoslide:` | other | $COMPILE -fodoslide prog.cob |
| 1124 | run_extensions.at:673: 1124. Complex OCCURS DEPEND | `-fcomplex-odo:` | other | $COMPILE -fcomplex-odo prog.cob |
| 1125 | run_extensions.at:739: 1125. Complex OCCURS DEPEND | `-fodoslide:` | other | $COMPILE -fodoslide prog.cob |
| 1126 | run_extensions.at:805: 1126. Complex OCCURS DEPEND | `-fodoslide:` | other | $COMPILE -fodoslide prog.cob |
| 1130 | run_extensions.at:1151: 1130. INITIALIZE OCCURS OD | `-fodoslide:` | other | $COMPILE -fodoslide prog.cob |
| 1131 | run_extensions.at:1248: 1131. DEPENDING ON with OD | `-fodoslide:` | other | $COBC -x -fodoslide prog.cob |
| 1144 | run_extensions.at:2172: 1144. NUMBER-OF-CALL-PARAM | `-fusing-optional=skip:` | dialect/extension flag | $COMPILE_MODULE -fusing-optional=skip callee.cob |
| 1145 | run_extensions.at:2253: 1145. TALLY register | `-fnot-register=TALLY:` | dialect/extension flag | $COMPILE_ONLY -fnot-register=TALLY prog.cob |
| 1151 | run_extensions.at:2603: 1151. ENTRY | `-fentry-statement=ok:` | dialect/extension flag | $COMPILE_MODULE -fentry-statement=ok hello.cob |
| 1158 | run_extensions.at:2893: 1158. SWITCHES with non-st | `-fsystem-name=sw1,` | dialect/extension flag | $COMPILE -fsystem-name="sw1, SwItCh\ b, SWITCH\ 25" \ |
| 1159 | run_extensions.at:3006: 1159. Larger REDEFINES len | `-flarger-redefines=ok:` | dialect/extension flag | $COMPILE -flarger-redefines=ok -w prog.cob |
| 1198 | run_extensions.at:5113: 1198. X/Open free-form for | `-fno-areacheck:` | other | $COMPILE -fformat=xopen -fno-areacheck prog.cob |
| 1199 | run_extensions.at:5162: 1199. TERMINAL format | `-fcomment-paragraphs=ok:` | dialect/extension flag | $COMPILE -fformat=terminal -fcomment-paragraphs=ok prog |
| 1201 | run_extensions.at:5296: 1201. Binary COMP-1 (1) | `-fbinary-comp-1:` | other | $COMPILE -fbinary-comp-1 prog.cob |
| 1204 | run_extensions.at:5393: 1204. Bit Operations | `-facu-literal=ok:` | dialect/extension flag | $COMPILE -facu-literal=ok -fno-trunc prog.cob |
| 1205 | run_extensions.at:5496: 1205. Bit Shift Operations | `-fno-trunc:` | other | $COMPILE -std=mf -fno-trunc prog.cob |
| 1215 | run_extensions.at:6159: 1215. EXAMINE TALLYING | `-freserved=EXAMINE:` | dialect/extension flag | $COMPILE -freserved=EXAMINE prog.cob |
| 1216 | run_extensions.at:6216: 1216. EXAMINE REPLACING | `-freserved=EXAMINE:` | dialect/extension flag | $COMPILE -freserved=EXAMINE prog.cob |
| 1217 | run_extensions.at:6252: 1217. GCOS literals with E | `-febcdic-symbolic-characters:` | other | $COMPILE -febcdic-symbolic-characters -febcdic-table=eb |
| 1218 | run_ml.at:19: 1218. XML GENERATE general | `-fnot-reserved=ID:` | dialect/extension flag | $COMPILE -fnot-reserved=ID prog.cob |
| 1224 | run_ml.at:488: 1224. XML dpc-in-data config option | `-fdpc-in-data=none:` | dialect/extension flag | $COMPILE -fdpc-in-data=none prog.cob |
| 1231 | run_ml.at:868: 1231. JSON dpc-in-data config optio | `-fdpc-in-data=none:` | dialect/extension flag | $COMPILE -fdpc-in-data=none prog.cob |
| 1232 | data_binary.at:23: 1232. BINARY: 2-4-8 big-endian | `-fbinary-size=2-4-8:` | dialect/extension flag | $COMPILE -fbinary-size=2-4-8 \ |
| 1233 | data_binary.at:205: 1233. BINARY: 2-4-8 native | `-fbinary-size=2-4-8:` | dialect/extension flag | $COMPILE -fbinary-size=2-4-8 \ |
| 1234 | data_binary.at:393: 1234. BINARY: 1-2-4-8 big-endi | `-fbinary-size=1-2-4-8:` | dialect/extension flag | $COMPILE -fbinary-size=1-2-4-8 \ |
| 1235 | data_binary.at:575: 1235. BINARY: 1-2-4-8 native | `-fbinary-size=1-2-4-8:` | dialect/extension flag | $COMPILE -fbinary-size=1-2-4-8 \ |
| 1236 | data_binary.at:763: 1236. BINARY: 1--8 big-endian | `-fbinary-size=1--8:` | dialect/extension flag | $COMPILE -fbinary-size=1--8 \ |
| 1237 | data_binary.at:945: 1237. BINARY: 1--8 native | `-fbinary-size=1--8:` | dialect/extension flag | $COMPILE -fbinary-size=1--8 \ |
| 1238 | data_binary.at:1133: 1238. BINARY: full-print | `-fbinary-size=1--8:` | dialect/extension flag | $COMPILE -fbinary-size=1--8 \ |
| 1240 | data_binary.at:1215: 1240. BINARY: 64bit unsigned  | `-fnotrunc:` | other | $COMPILE -fnotrunc prog.cob |
| 1241 | data_binary.at:1242: 1241. BINARY: 64bit signed ne | `-fnotrunc:` | other | $COMPILE -fnotrunc prog.cob |
| 1243 | data_binary.at:1323: 1243. COMP-4 No Truncate | `-fnotrunc:` | other | $COMPILE -w -fnotrunc prog.cob |
| 1250 | data_display.at:22: 1250. DISPLAY: Sign ASCII | `-fsign=ascii:` | dialect/extension flag | $COMPILE -fsign=ascii prog.cob |
| 1251 | data_display.at:81: 1251. DISPLAY: Sign ASCII (2) | `-fsign=ascii:` | dialect/extension flag | $COMPILE -fsign=ascii prog.cob |
| 1252 | data_display.at:127: 1252. DISPLAY: Sign EBCDIC | `-fsign=ebcdic:` | dialect/extension flag | $COMPILE -fsign=ebcdic prog.cob |
| 1271 | data_packed.at:1432: 1271. COMP-6 used with MOVE | `-fno-fast-compare:` | other | $COMPILE -fno-fast-compare -C -o progalt.c prog.cob |

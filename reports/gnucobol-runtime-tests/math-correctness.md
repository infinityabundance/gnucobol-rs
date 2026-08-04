# GnuCOBOL runtime/mathematics — correctness classification

323 math tests (of 1282 suite tests), classified from the SAME differential results as every
other test — no favorable selection. Correctness is the suite's own AT_CHECK assertion
outcome; performance is reported separately and only for tests passing on both sides.

## Totals by classification

- CANDIDATE_CHECK_REJECT: 52
- CANDIDATE_MODULE_MODEL_UNSUPPORTED: 147
- CANDIDATE_UNSUPPORTED: 1
- OBSERVABLE_MATCH: 97
- ORACLE_SKIP: 3
- ORACLE_XFAIL: 1
- WRAPPER_OPTION_UNSUPPORTED: 22

## By .at source

| source | category | tests |
|---|---|---|
| `data_binary.at` | binary arithmetic (COMP-5/binary fields) | 18 |
| `data_display.at` | DISPLAY/zoned-decimal arithmetic | 9 |
| `data_packed.at` | PACKED-DECIMAL (COMP-3) arithmetic | 23 |
| `data_pointer.at` | POINTER/USAGE POINTER | 1 |
| `run_fundamental.at` | fundamental arithmetic (ADD/SUBTRACT/MULTIPLY/DIVIDE/COMPUTE) | 114 |
| `run_functions.at` | intrinsic mathematical functions | 126 |
| `syn_multiply.at` | MULTIPLY syntax | 3 |
| `syn_value.at` | VALUE clauses / numeric literals | 13 |
| `syn_literals.at` | literal forms | 16 |

## Per-test ledger

| id | title | category | oracle | candidate | classification |
|---|---|---|---|---|---|
| 0173 | syn_value.at:28: 173. bad VALUES / VALUES ARE in format-1 | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0174 | syn_value.at:76: 174. OCCURS too many VALUEs | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0175 | syn_value.at:162: 175. Numeric item (integer) | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0176 | syn_value.at:189: 176. Numeric item (non-integer) | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0177 | syn_value.at:213: 177. Numeric item with picture P | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0178 | syn_value.at:245: 178. Signed numeric literal | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0179 | syn_value.at:271: 179. Alphabetic item | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0180 | syn_value.at:299: 180. Alphanumeric item | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0181 | syn_value.at:325: 181. Alphanumeric group item | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0182 | syn_value.at:352: 182. National item | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0183 | syn_value.at:389: 183. Numeric-edited item | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0184 | syn_value.at:425: 184. Alphanumeric-edited item | syn_value | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0185 | syn_value.at:494: 185. Implicit picture from value | syn_value | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0419 | syn_multiply.at:28: 419. Category check of Format 1 | syn_multiply | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0420 | syn_multiply.at:64: 420. Category check of Format 2 | syn_multiply | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0421 | syn_multiply.at:102: 421. Category check of literals | syn_multiply | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0459 | syn_literals.at:25: 459. continuation Indicator - too many lines | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0460 | syn_literals.at:583: 460. literal too long | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0461 | syn_literals.at:819: 461. numeric literals | syn_literals | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0462 | syn_literals.at:998: 462. floating-point literals | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0463 | syn_literals.at:1105: 463. X literals | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0464 | syn_literals.at:1140: 464. national literals | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0465 | syn_literals.at:1178: 465. NX literals | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0466 | syn_literals.at:1216: 466. binary literals | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0467 | syn_literals.at:1252: 467. binary-hexadecimal literals | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0468 | syn_literals.at:1281: 468. HP COBOL octal literals | syn_literals | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0469 | syn_literals.at:1317: 469. ACUCOBOL literals | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0470 | ACUCOBOL 32bit literal size | syn_literals | ORACLE_SKIP | - | ORACLE_SKIP |
| 0471 | syn_literals.at:1412: 471. zero-length literals | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0472 | syn_literals.at:1472: 472. long literal in error message | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0473 | syn_literals.at:1504: 473. literal missing terminating character | syn_literals | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0474 | syn_literals.at:1525: 474. GCOS literals with EBCDIC symbols (syntax) | syn_literals | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0511 | run_fundamental.at:25: 511. DISPLAY literals | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0512 | run_fundamental.at:72: 512. DISPLAY literals, DECIMAL-POINT is COMMA | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0513 | run_fundamental.at:105: 513. Hexadecimal literal | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0514 | DISPLAY data items with VALUE clause | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0515 | DISPLAY data items with MOVE statement | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0516 | run_fundamental.at:247: 516. MOVE to edited item (1) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0517 | run_fundamental.at:296: 517. MOVE to edited item (2) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0518 | run_fundamental.at:345: 518. MOVE to edited item (3) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0519 | run_fundamental.at:450: 519. MOVE to item with simple and floating insertion | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0520 | run_fundamental.at:485: 520. MOVE to JUSTIFIED item | run_fundamental | ORACLE_PASS | - | CANDIDATE_UNSUPPORTED |
| 0521 | run_fundamental.at:530: 521. MOVE integer literal to alphanumeric | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0522 | run_fundamental.at:567: 522. Compare FLOAT-LONG with floating-point literal | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0523 | run_fundamental.at:620: 523. equality of FLOAT-SHORT / FLOAT-LONG | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0524 | run_fundamental.at:743: 524. equality of FLOAT-SHORT / FLOAT-EXTENDED | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0525 | run_fundamental.at:874: 525. Overlapping MOVE (GnuCOBOL) | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0526 | run_fundamental.at:958: 526. Overlapping MOVE (IBM) | run_fundamental | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0527 | run_fundamental.at:1039: 527. ALPHABETIC test | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0528 | run_fundamental.at:1071: 528. ALPHABETIC-UPPER test | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0529 | run_fundamental.at:1103: 529. ALPHABETIC-LOWER test | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0530 | run_fundamental.at:1135: 530. GLOBAL at same level | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0531 | run_fundamental.at:1184: 531. GLOBAL at lower level | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0532 | run_fundamental.at:1233: 532. GLOBAL CONSTANT | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0533 | run_fundamental.at:1317: 533. GLOBAL identifiers from ENVIRONMENT DIVISION | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0534 | GLOBAL REDEFINES | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0535 | function with variable-length RETURNING item | run_fundamental | ORACLE_SKIP | - | ORACLE_SKIP |
| 0536 | run_fundamental.at:1500: 536. Entry point visibility (1) | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0537 | run_fundamental.at:1532: 537. Entry point visibility (2) | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0538 | run_fundamental.at:1570: 538. Contained program visibility (1) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0539 | run_fundamental.at:1625: 539. Contained program visibility (2) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0540 | run_fundamental.at:1678: 540. Contained program visibility (3) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0541 | run_fundamental.at:1729: 541. Contained program visibility (4) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0542 | run_fundamental.at:1784: 542. CALL alphanumeric data-name | run_fundamental | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0543 | run_fundamental.at:1872: 543. CALL program-pointer | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0544 | run_fundamental.at:1968: 544. CALL/CANCEL/SET ADDRESS program-prototype-name | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0545 | GLOBAL FD (1) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0546 | GLOBAL FD (2) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0547 | run_fundamental.at:2175: 547. GLOBAL FD (3) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0548 | run_fundamental.at:2225: 548. GLOBAL FD (4) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0549 | run_fundamental.at:2277: 549. CANCEL test (1) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0550 | run_fundamental.at:2304: 550. CANCEL test (2) | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0551 | run_fundamental.at:2341: 551. CANCEL test (3) | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0552 | Separate sign positions (1) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0553 | run_fundamental.at:2403: 553. Separate sign positions (2) | run_fundamental | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0554 | run_fundamental.at:2436: 554. Context sensitive words (1) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0555 | run_fundamental.at:2459: 555. Context sensitive words (2) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0556 | run_fundamental.at:2483: 556. Context sensitive words (3) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0557 | Context sensitive words (4) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0558 | Context sensitive words (5) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0559 | Context sensitive words (6) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0560 | run_fundamental.at:2577: 560. Context sensitive words (7) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0561 | Context sensitive words (8) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0562 | ROUNDED AWAY-FROM-ZERO | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0563 | ROUNDED NEAREST-AWAY-FROM-ZERO | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0564 | ROUNDED NEAREST-EVEN | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0565 | ROUNDED NEAREST-TOWARD-ZERO | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0566 | ROUNDED TOWARD-GREATER | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0567 | ROUNDED TOWARD-LESSER | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0568 | ROUNDED TRUNCATION | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0569 | Numeric operations (1) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0570 | run_fundamental.at:3108: 570. Numeric operations (2) DISPLAY | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0571 | run_fundamental.at:3344: 571. Numeric operations (3) PACKED-DECIMAL | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0572 | run_fundamental.at:3648: 572. Numeric operations (4) BINARY | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0573 | run_fundamental.at:3883: 573. Numeric operations (5) COMP-5 | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0574 | run_fundamental.at:4118: 574. Numeric operations (6) | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0575 | Numeric operations (7) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0576 | run_fundamental.at:4468: 576. Numeric operations (8) | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0577 | run_fundamental.at:4518: 577. ADD CORRESPONDING | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0578 | run_fundamental.at:4566: 578. ADD CORRESPONDING no match | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0579 | run_fundamental.at:4616: 579. SYNC in OCCURS | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0580 | 88 level with THRU | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0581 | run_fundamental.at:4751: 581. 88 level with FILLER | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0582 | run_fundamental.at:4780: 582. 88 level with FALSE IS clause | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0583 | BLANK WHEN ZERO | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0584 | MULTIPLY BY literal in INITIAL program | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0585 | run_fundamental.at:4867: 585. DIVIDE complex | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0586 | run_fundamental.at:4988: 586. COMPUTE with decimal constants | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0587 | run_fundamental.at:5024: 587. debugging lines (not active) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0588 | run_fundamental.at:5047: 588. debugging lines (-fdebugging-line) | run_fundamental | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0589 | debugging lines (WITH DEBUGGING MODE) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0590 | debugging lines, free format (not active) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0591 | run_fundamental.at:5119: 591. debugging lines, free format (-fdebugging-line) | run_fundamental | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0592 | USE FOR DEBUGGING (no DEBUGGING MODE) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0593 | USE FOR DEBUGGING (COB_SET_DEBUG deactivated) | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0594 | run_fundamental.at:5232: 594. USE FOR DEBUGGING ON ALL PROCEDURES | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0595 | run_fundamental.at:5283: 595. USE FOR DEBUGGING ON procedure | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0596 | run_fundamental.at:5331: 596. USE FOR DEBUGGING (COB_SET_DEBUG switched) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0597 | run_fundamental.at:5380: 597. USE FOR DEBUGGING ON [ALL] REFERENCES OF field | run_fundamental | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0598 | run_fundamental.at:5460: 598. USE FOR DEBUGGING, reference within DEBUGGING | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0599 | run_fundamental.at:5501: 599. USE FOR DEBUGGING, time of execution | run_fundamental | ORACLE_XFAIL | expected-failure | ORACLE_XFAIL |
| 0600 | run_fundamental.at:5551: 600. USE FOR DEBUGGING, reference with OCCURS | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0601 | run_fundamental.at:5584: 601. USE FOR DEBUGGING, referencing BASED item | run_fundamental | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 0602 | run_fundamental.at:5639: 602. USE FOR DEBUGGING file | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0603 | run_fundamental.at:5682: 603. Simple Expressions with figurative constants | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0604 | run_fundamental.at:6025: 604. Expression numeric vs. DISPLAY | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0605 | run_fundamental.at:6080: 605. Abbreviated Expressions | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0606 | run_fundamental.at:6192: 606. integer arithmetic on floating-point var | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0607 | run_fundamental.at:6234: 607. TYPEDEF application | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0608 | run_fundamental.at:6286: 608. Alphanumeric VALUE longer than PIC | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0609 | run_fundamental.at:6318: 609. DISPLAY with P fields | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0610 | run_fundamental.at:6404: 610. condition IS ZERO AND | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0611 | abbreviated conditions with multiple words operators | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0612 | run_fundamental.at:6493: 612. abbreviated conditions with multiple words operators | run_fundamental | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0613 | MOVE with JUSTIFIED clause | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0614 | MOVE with PICTURE P | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0615 | run_fundamental.at:6770: 615. MOVE with de-editting to USAGE DISPLAY | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0616 | run_fundamental.at:6822: 616. MOVE with de-editting to DECIMAL IS COMMA | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0617 | run_fundamental.at:6880: 617. MOVE with de-editting to BINARY | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0618 | run_fundamental.at:6931: 618. MOVE with de-editting to COMP-3 | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0619 | run_fundamental.at:6983: 619. MOVE with de-editting to COMP-5 | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0620 | run_fundamental.at:7035: 620. MOVE with de-editting to NUMERIC DISPLAY (2) | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0621 | run_fundamental.at:7164: 621. MOVE misc. edited | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0622 | run_fundamental.at:7238: 622. MOVE between USAGEs | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0623 | MOVE to editted ZERO | run_fundamental | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0624 | run_fundamental.at:9066: 624. SPECIAL-NAMES CLASS | run_fundamental | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0985 | FUNCTION ABS | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0986 | FUNCTION ACOS | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0987 | FUNCTION ANNUITY | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0988 | FUNCTION ASIN | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0989 | FUNCTION ATAN | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0990 | run_functions.at:165: 990. FUNCTION BYTE-LENGTH | run_functions | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 0991 | run_functions.at:221: 991. FUNCTION CHAR | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0992 | FUNCTION COMBINED-DATETIME | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 0993 | run_functions.at:284: 993. FUNCTION CONCAT / CONCATENATE | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0994 | run_functions.at:331: 994. FUNCTION CONCATENATE with reference modding | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0995 | run_functions.at:359: 995. FUNCTION BIT-OF and BIT-TO-CHAR | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0996 | run_functions.at:416: 996. FUNCTION HEX-OF and HEX-TO-CHAR | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0997 | run_functions.at:547: 997. FUNCTION CONTENT-LENGTH | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0998 | run_functions.at:581: 998. FUNCTION CONTENT-OF | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0999 | run_functions.at:656: 999. FUNCTION as CALL parameter BY CONTENT | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1000 | FUNCTION COS | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1001 | FUNCTION CURRENCY-SYMBOL | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1002 | run_functions.at:739: 1002. FUNCTION CURRENT-DATE | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1003 | FUNCTION DATE-OF-INTEGER | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1004 | run_functions.at:835: 1004. FUNCTION DATE-TO-YYYYMMDD | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1005 | FUNCTION DAY-OF-INTEGER | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1006 | run_functions.at:885: 1006. FUNCTION DAY-TO-YYYYDDD | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1007 | FUNCTION E | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1008 | FUNCTION EXCEPTION-FILE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1009 | run_functions.at:969: 1009. FUNCTION EXCEPTION-LOCATION | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1010 | run_functions.at:1057: 1010. FUNCTION EXCEPTION-STATEMENT | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1011 | run_functions.at:1092: 1011. FUNCTION EXCEPTION-STATUS | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1012 | FUNCTION EXP | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1013 | FUNCTION EXP10 | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1014 | FUNCTION FACTORIAL | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1015 | FUNCTION FORMATTED-CURRENT-DATE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1016 | FUNCTION FORMATTED-DATE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1017 | run_functions.at:1292: 1017. FUNCTION FORMATTED-DATE with ref modding | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1018 | run_functions.at:1317: 1018. FUNCTION FORMATTED-DATETIME | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1019 | run_functions.at:1374: 1019. FUNCTION FORMATTED-DATETIME with ref modding | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1020 | run_functions.at:1400: 1020. FUNCTION FORMATTED-TIME | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1021 | run_functions.at:1487: 1021. FUNCTION FORMATTED-TIME DP.COMMA | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1022 | run_functions.at:1518: 1022. FUNCTION FORMATTED-TIME with ref modding | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1023 | FUNCTION FRACTION-PART | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1024 | run_functions.at:1574: 1024. FUNCTION HIGHEST-ALGEBRAIC | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1025 | run_functions.at:1642: 1025. FUNCTION INTEGER | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1026 | FUNCTION INTEGER-OF-DATE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1027 | FUNCTION INTEGER-OF-DAY | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1028 | FUNCTION INTEGER-OF-FORMATTED-DATE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1029 | FUNCTION INTEGER-PART | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1030 | run_functions.at:1801: 1030. FUNCTION LENGTH | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1031 | FUNCTION LOCALE-COMPARE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1032 | FUNCTION LOCALE-DATE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1033 | FUNCTION LOCALE-TIME | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1034 | run_functions.at:1965: 1034. FUNCTION LOCALE-TIME-FROM-SECONDS | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1035 | FUNCTION LOG | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1036 | FUNCTION LOG10 | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1037 | run_functions.at:2039: 1037. FUNCTION LOWER-CASE | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1038 | FUNCTION LOWER-CASE with reference modding | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1039 | run_functions.at:2093: 1039. FUNCTION LOWEST-ALGEBRAIC | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1040 | FUNCTION MAX | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1041 | FUNCTION MEAN | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1042 | FUNCTION MEDIAN | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1043 | FUNCTION MIDRANGE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1044 | FUNCTION MIN | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1045 | FUNCTION MOD (valid) | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1046 | run_functions.at:2286: 1046. FUNCTION MOD (invalid) | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1047 | run_functions.at:2317: 1047. FUNCTION MODULE-CALLER-ID | run_functions | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1048 | FUNCTION MODULE-DATE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1049 | FUNCTION MODULE-FORMATTED-DATE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1050 | run_functions.at:2401: 1050. FUNCTION MODULE-ID | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1051 | run_functions.at:2422: 1051. FUNCTION MODULE-PATH | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1052 | FUNCTION MODULE-SOURCE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1053 | run_functions.at:2468: 1053. FUNCTION MODULE-TIME | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1054 | run_functions.at:2493: 1054. FUNCTION MONETARY-DECIMAL-POINT | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1055 | run_functions.at:2516: 1055. FUNCTION MONETARY-THOUSANDS-SEPARATOR | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1056 | run_functions.at:2539: 1056. FUNCTION NUMERIC-DECIMAL-POINT | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1057 | run_functions.at:2562: 1057. FUNCTION NUMERIC-THOUSANDS-SEPARATOR | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1058 | run_functions.at:2585: 1058. FUNCTION NUMVAL | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1059 | run_functions.at:2658: 1059. FUNCTION NUMVAL-C | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1060 | run_functions.at:2719: 1060. FUNCTION NUMVAL-C DP.COMMA | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1061 | FUNCTION NUMVAL-F | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1062 | FUNCTION ORD | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1063 | FUNCTION ORD-MAX | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1064 | FUNCTION ORD-MIN | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1065 | FUNCTION PI | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1066 | FUNCTION PRESENT-VALUE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1067 | run_functions.at:2888: 1067. FUNCTION RANDOM | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1068 | run_functions.at:2910: 1068. FUNCTION RANGE | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1069 | run_functions.at:2932: 1069. FUNCTION REM (valid) | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1070 | run_functions.at:2954: 1070. FUNCTION REM (invalid) | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1071 | FUNCTION REVERSE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1072 | run_functions.at:3004: 1072. FUNCTION REVERSE with reference modding | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1073 | run_functions.at:3027: 1073. FUNCTION SECONDS-FROM-FORMATTED-TIME | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1074 | FUNCTION SECONDS-PAST-MIDNIGHT | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1075 | run_functions.at:3104: 1075. FUNCTION SIGN | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1076 | FUNCTION SIN | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1077 | FUNCTION SQRT | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1078 | FUNCTION STANDARD-DEVIATION | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1079 | FUNCTION STORED-CHAR-LENGTH | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1080 | run_functions.at:3238: 1080. FUNCTION SUBSTITUTE | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1081 | run_functions.at:3265: 1081. FUNCTION SUBSTITUTE with reference modding | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1082 | FUNCTION SUBSTITUTE-CASE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1083 | run_functions.at:3316: 1083. FUNCTION SUBSTITUTE-CASE with reference mod | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1084 | FUNCTION SUM | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1085 | FUNCTION TAN | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1086 | FUNCTION TEST-DATE-YYYYMMDD | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1087 | FUNCTION TEST-DAY-YYYYDDD | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1088 | FUNCTION TEST-FORMATTED-DATETIME with dates | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1089 | run_functions.at:3531: 1089. FUNCTION TEST-FORMATTED-DATETIME with times | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1090 | run_functions.at:3612: 1090. FUNCTION TEST-FORMATTED-DATETIME with datetimes | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1091 | run_functions.at:3665: 1091. FUNCTION TEST-FORMATTED-DATETIME DP.COMMA | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1092 | FUNCTION TEST-NUMVAL | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1093 | run_functions.at:3857: 1093. FUNCTION TEST-NUMVAL-C | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1094 | FUNCTION TEST-NUMVAL-F | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1095 | FUNCTION TRIM | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1096 | run_functions.at:4083: 1096. FUNCTION TRIM with reference modding | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1097 | FUNCTION TRIM zero length | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1098 | FUNCTION UPPER-CASE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1099 | run_functions.at:4161: 1099. FUNCTION UPPER-CASE with reference modding | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1100 | FUNCTION VARIANCE | run_functions | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1101 | run_functions.at:4216: 1101. FUNCTION WHEN-COMPILED | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1102 | run_functions.at:4270: 1102. FUNCTION YEAR-TO-YYYY | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1103 | run_functions.at:4294: 1103. Formatted funcs w/ invalid variable format | run_functions | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1104 | run_functions.at:4375: 1104. FORMATTED-(DATE)TIME with SYSTEM-OFFSET | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1105 | run_functions.at:4413: 1105. Intrinsics without FUNCTION keyword (1) | run_functions | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1106 | run_functions.at:4434: 1106. Intrinsics without FUNCTION keyword (2) | run_functions | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1107 | run_functions.at:4457: 1107. User-Defined FUNCTION with/without parameter | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1108 | run_functions.at:4508: 1108. UDF in COMPUTE | run_functions | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1109 | run_functions.at:4551: 1109. UDF replacing intrinsic function | run_functions | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1110 | UDF with recursion | run_functions | ORACLE_SKIP | - | ORACLE_SKIP |
| 1232 | data_binary.at:23: 1232. BINARY: 2-4-8 big-endian | data_binary | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1233 | data_binary.at:205: 1233. BINARY: 2-4-8 native | data_binary | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1234 | data_binary.at:393: 1234. BINARY: 1-2-4-8 big-endian | data_binary | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1235 | data_binary.at:575: 1235. BINARY: 1-2-4-8 native | data_binary | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1236 | data_binary.at:763: 1236. BINARY: 1--8 big-endian | data_binary | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1237 | data_binary.at:945: 1237. BINARY: 1--8 native | data_binary | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1238 | data_binary.at:1133: 1238. BINARY: full-print | data_binary | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1239 | data_binary.at:1185: 1239. BINARY: 64bit unsigned compare | data_binary | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1240 | data_binary.at:1215: 1240. BINARY: 64bit unsigned arithmetic notrunc | data_binary | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1241 | data_binary.at:1242: 1241. BINARY: 64bit signed negative constant range | data_binary | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1242 | COMP-4 Truncate | data_binary | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1243 | data_binary.at:1323: 1243. COMP-4 No Truncate | data_binary | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1244 | data_binary.at:1409: 1244. MOVE DISPLAY to BINARY | data_binary | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1245 | data_binary.at:1595: 1245. MOVE PACKED-DECIMAL to BINARY | data_binary | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1246 | data_binary.at:1781: 1246. MOVE BINARY to PACKED-DECIMAL | data_binary | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1247 | data_binary.at:1961: 1247. MOVE BINARY to BINARY | data_binary | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1248 | data_binary.at:2143: 1248. PPP COMP-5 | data_binary | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1249 | arithmetic truncation with USAGE BINARY | data_binary | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1250 | data_display.at:22: 1250. DISPLAY: Sign ASCII | data_display | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1251 | data_display.at:81: 1251. DISPLAY: Sign ASCII (2) | data_display | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1252 | data_display.at:127: 1252. DISPLAY: Sign EBCDIC | data_display | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1253 | data_display.at:172: 1253. DISPLAY: unsigned | data_display | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1254 | data_display.at:226: 1254. MOVE DISPLAY to DISPLAY | data_display | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1255 | data_display.at:408: 1255. PPP DISPLAY | data_display | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1256 | arithmetic truncation with USAGE DISPLAY | data_display | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1257 | data_display.at:533: 1257. DISPLAY: ADD and SUBTRACT w/o SIZE ERROR | data_display | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1258 | data_display.at:16940: 1258. DISPLAY: ADD and SUBTRACT, all ROUNDED MODEs | data_display | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1259 | data_packed.at:27: 1259. PACKED-DECIMAL dump | data_packed | ORACLE_PASS | - | CANDIDATE_CHECK_REJECT |
| 1260 | PACKED-DECIMAL used with DISPLAY | data_packed | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1261 | data_packed.at:221: 1261. PACKED-DECIMAL used with MOVE | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1262 | data_packed.at:459: 1262. MOVE PACKED-DECIMAL to PACKED-DECIMAL | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1263 | data_packed.at:639: 1263. MOVE PACKED-DECIMAL to DISPLAY | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1264 | data_packed.at:825: 1264. MOVE DISPLAY to PACKED-DECIMAL | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1265 | PACKED-DECIMAL used with INITIALIZE | data_packed | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1266 | data_packed.at:1063: 1266. PACKED-DECIMAL arithmetic | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1267 | data_packed.at:1187: 1267. PACKED-DECIMAL comparison | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1268 | data_packed.at:1269: 1268. PACKED-DECIMAL numeric test (1) | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1269 | data_packed.at:1333: 1269. PACKED-DECIMAL numeric test (2) | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1270 | COMP-6 used with DISPLAY | data_packed | ORACLE_PASS | ok | OBSERVABLE_MATCH |
| 1271 | data_packed.at:1432: 1271. COMP-6 used with MOVE | data_packed | ORACLE_PASS | - | WRAPPER_OPTION_UNSUPPORTED |
| 1272 | data_packed.at:1474: 1272. COMP-6 arithmetic | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1273 | data_packed.at:1563: 1273. COMP-6 numeric | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1274 | data_packed.at:1609: 1274. COMP-6 comparison | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1275 | data_packed.at:1671: 1275. COMP-3 vs. COMP-6 - BCD comparison | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1276 | data_packed.at:1971: 1276. PPP COMP-3 | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1277 | data_packed.at:2083: 1277. PPP COMP-6 | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1278 | data_packed.at:2174: 1278. arithmetic truncation with USAGE PACKED-DECIMAL | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1279 | data_packed.at:2236: 1279. MOVE between several BCD fields | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1280 | data_packed.at:12326: 1280. BCD ADD and SUBTRACT w/o SIZE ERROR | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1281 | data_packed.at:28746: 1281. BCD ADD and SUBTRACT, all ROUNDED MODEs | data_packed | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1282 | data_pointer.at:21: 1282. POINTER: display | data_pointer | ORACLE_PASS | - | CANDIDATE_MODULE_MODEL_UNSUPPORTED |

# GnuCOBOL testsuite — module-lifecycle census (forensic)

456 module-related tests, grouped into the ACTUAL module patterns observed in the
real suite (not one undifferentiated 407-test feature). Derived from the invocation census +
per-group transcripts + the classification ledger.

## Patterns

- COBCRUN_DIRECT ./prog (launcher run): 417
- other: 30
- cobcrun runtime-config: 6
- cobc -m module build: 3

## Per-test ledger

| id | title | pattern | failing command | classification |
|---|---|---|---|---|
| 0001 | used_binaries.at:27: 1. compiler help and information | other | $COBC --list-reserved | WRAPPER_OPTION_UNSUPPORTED |
| 0004 | (no status line in group log) | other | $COBC -I sub/copy prog.c -o prog.$COB_OBJECT_EXT | WRAPPER_OPTION_UNSUPPORTED |
| 0010 | used_binaries.at:427: 10. C Compiler optimizations | other | $COBCRUN -M sub/ prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0013 | used_binaries.at:500: 13. cobcrun validation | cobc -m module build | $COMPILE_MODULE callee.cob | CANDIDATE_CHECK_REJECT |
| 0014 | used_binaries.at:540: 14. cobcrun -M DSO entry argument | other | $COBCRUN -M "" nope | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0015 | used_binaries.at:626: 15. cobcrun -M directory/ default | cobc -m module build | $COMPILE_MODULE callee.cob | CANDIDATE_CHECK_REJECT |
| 0016 | used_binaries.at:665: 16. cobcrun -M directory/dso alternate | cobc -m module build | $COMPILE_MODULE callee.cob | CANDIDATE_CHECK_REJECT |
| 0017 | used_binaries.at:716: 17. cobcrun -M DSO entry multiple argu | other | $COBC -b ${FLAGS} mainer.cob called.cob | WRAPPER_OPTION_UNSUPPORTED |
| 0018 | used_binaries.at:762: 18. cobcrun error messages | other | $COBCRUN -q | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0039 | configuration.at:546: 39. runtime configuration | cobcrun runtime-config | $COBCRUN --runtime-conf | tr -d '\n ' | \ | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0040 | configuration.at:565: 40. runtime configuration file | cobcrun runtime-config | $COBCRUN -c test2.cfg --runtime-conf | \ | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0041 | configuration.at:608: 41. runtime configuration: recursive i | cobcrun runtime-config | $COBCRUN -c test.cfg -r | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0043 | configuration.at:650: 43. runtime configuration: entries | cobcrun runtime-config | echo "$PATHSEP" | CANDIDATE_UNSUPPORTED |
| 0044 | configuration.at:731: 44. runtime configuration: conf missin | cobcrun runtime-config | $COBCRUN -c notthere.cfg --runtime-conf | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0046 | configuration.at:773: 46. runtime configuration: strings and | cobcrun runtime-config | unset greet name ; \ | CANDIDATE_UNSUPPORTED |
| 0286 | syn_misc.at:1947: 286. line and floating comments | other | $COMPILE_ONLY prog2.cob | CANDIDATE_CHECK_REJECT |
| 0511 | run_fundamental.at:25: 511. DISPLAY literals | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0512 | run_fundamental.at:72: 512. DISPLAY literals, DECIMAL-POINT  | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0516 | run_fundamental.at:247: 516. MOVE to edited item (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0517 | run_fundamental.at:296: 517. MOVE to edited item (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0518 | run_fundamental.at:345: 518. MOVE to edited item (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0519 | run_fundamental.at:450: 519. MOVE to item with simple and fl | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0520 | run_fundamental.at:485: 520. MOVE to JUSTIFIED item | other |  | CANDIDATE_UNSUPPORTED |
| 0522 | run_fundamental.at:567: 522. Compare FLOAT-LONG with floatin | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0523 | run_fundamental.at:620: 523. equality of FLOAT-SHORT / FLOAT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0524 | run_fundamental.at:743: 524. equality of FLOAT-SHORT / FLOAT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0527 | run_fundamental.at:1039: 527. ALPHABETIC test | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0528 | run_fundamental.at:1071: 528. ALPHABETIC-UPPER test | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0529 | run_fundamental.at:1103: 529. ALPHABETIC-LOWER test | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0530 | run_fundamental.at:1135: 530. GLOBAL at same level | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0531 | run_fundamental.at:1184: 531. GLOBAL at lower level | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0532 | run_fundamental.at:1233: 532. GLOBAL CONSTANT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0533 | run_fundamental.at:1317: 533. GLOBAL identifiers from ENVIRO | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0538 | run_fundamental.at:1570: 538. Contained program visibility ( | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0539 | run_fundamental.at:1625: 539. Contained program visibility ( | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0540 | run_fundamental.at:1678: 540. Contained program visibility ( | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0541 | run_fundamental.at:1729: 541. Contained program visibility ( | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0543 | run_fundamental.at:1872: 543. CALL program-pointer | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0544 | run_fundamental.at:1968: 544. CALL/CANCEL/SET ADDRESS progra | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0547 | run_fundamental.at:2175: 547. GLOBAL FD (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0548 | run_fundamental.at:2225: 548. GLOBAL FD (4) | COBCRUN_DIRECT ./prog (launcher run) | COB_DISABLE_WARNINGS=1 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0549 | run_fundamental.at:2277: 549. CANCEL test (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0554 | run_fundamental.at:2436: 554. Context sensitive words (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0555 | run_fundamental.at:2459: 555. Context sensitive words (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0556 | run_fundamental.at:2483: 556. Context sensitive words (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0560 | run_fundamental.at:2577: 560. Context sensitive words (7) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0570 | run_fundamental.at:3108: 570. Numeric operations (2) DISPLAY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0571 | run_fundamental.at:3344: 571. Numeric operations (3) PACKED- | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0572 | run_fundamental.at:3648: 572. Numeric operations (4) BINARY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0573 | run_fundamental.at:3883: 573. Numeric operations (5) COMP-5 | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0577 | run_fundamental.at:4518: 577. ADD CORRESPONDING | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0579 | run_fundamental.at:4616: 579. SYNC in OCCURS | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0581 | run_fundamental.at:4751: 581. 88 level with FILLER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0582 | run_fundamental.at:4780: 582. 88 level with FALSE IS clause | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0585 | run_fundamental.at:4867: 585. DIVIDE complex | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0586 | run_fundamental.at:4988: 586. COMPUTE with decimal constants | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0587 | run_fundamental.at:5024: 587. debugging lines (not active) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0594 | run_fundamental.at:5232: 594. USE FOR DEBUGGING ON ALL PROCE | COBCRUN_DIRECT ./prog (launcher run) | COB_SET_DEBUG=1 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0595 | run_fundamental.at:5283: 595. USE FOR DEBUGGING ON procedure | COBCRUN_DIRECT ./prog (launcher run) | COB_SET_DEBUG=1 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0596 | run_fundamental.at:5331: 596. USE FOR DEBUGGING (COB_SET_DEB | COBCRUN_DIRECT ./prog (launcher run) | COB_SET_DEBUG=1 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0598 | run_fundamental.at:5460: 598. USE FOR DEBUGGING, reference w | COBCRUN_DIRECT ./prog (launcher run) | COB_SET_DEBUG=1 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0599 | run_fundamental.at:5501: 599. USE FOR DEBUGGING, time of exe | COBCRUN_DIRECT ./prog (launcher run) | COB_SET_DEBUG=1 $COBCRUN_DIRECT ./prog | ORACLE_XFAIL |
| 0600 | run_fundamental.at:5551: 600. USE FOR DEBUGGING, reference w | COBCRUN_DIRECT ./prog (launcher run) | COB_SET_DEBUG=1 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0602 | run_fundamental.at:5639: 602. USE FOR DEBUGGING file | COBCRUN_DIRECT ./prog (launcher run) | COB_SET_DEBUG=1 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0603 | run_fundamental.at:5682: 603. Simple Expressions with figura | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0604 | run_fundamental.at:6025: 604. Expression numeric vs. DISPLAY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0606 | run_fundamental.at:6192: 606. integer arithmetic on floating | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0609 | run_fundamental.at:6318: 609. DISPLAY with P fields | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0615 | run_fundamental.at:6770: 615. MOVE with de-editting to USAGE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0616 | run_fundamental.at:6822: 616. MOVE with de-editting to DECIM | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0617 | run_fundamental.at:6880: 617. MOVE with de-editting to BINAR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0618 | run_fundamental.at:6931: 618. MOVE with de-editting to COMP- | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0619 | run_fundamental.at:6983: 619. MOVE with de-editting to COMP- | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0620 | run_fundamental.at:7035: 620. MOVE with de-editting to NUMER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0621 | run_fundamental.at:7164: 621. MOVE misc. edited | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0622 | run_fundamental.at:7238: 622. MOVE between USAGEs | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0624 | run_fundamental.at:9066: 624. SPECIAL-NAMES CLASS | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0625 | run_subscripts.at:26: 625. Subscript out of bounds | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0626 | run_subscripts.at:71: 626. Value of DEPENDING ON N out of bo | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0627 | run_subscripts.at:122: 627. Subscript bounds with OCCURS DEP | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0628 | run_subscripts.at:148: 628. Subscript bounds with OCCURS DEP | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0629 | run_subscripts.at:176: 629. Subscript bounds with OCCURS DEP | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0630 | run_subscripts.at:211: 630. Subscript by arithmetic expressi | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0631 | run_subscripts.at:242: 631. length of ODO w/- reference-modi | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0632 | run_subscripts.at:298: 632. SEARCH ALL with OCCURS DEPENDING | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0633 | run_subscripts.at:351: 633. enable / disable subscript check | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./badprog1 | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0634 | run_subscripts.at:396: 634. enable / disable subscript check | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./badprog1 | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0635 | run_subscripts.at:437: 635. BOUND and NOBOUND directives | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0636 | run_subscripts.at:506: 636. SSRANGE and NOSSRANGE directives | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0637 | run_subscripts.at:542: 637. CALL with OCCURS DEPENDING ON | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0640 | run_refmod.at:94: 640. Offset underflow | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0641 | run_refmod.at:118: 641. Offset overflow | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0642 | run_refmod.at:145: 642. Length underflow | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0643 | run_refmod.at:189: 643. Length overflow | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0644 | run_refmod.at:231: 644. Length overflow with offset (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0645 | run_refmod.at:254: 645. Length overflow with offset (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0646 | run_refmod.at:280: 646. Length overflow with offset (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0648 | run_refmod.at:388: 648. MF SSRANGE and NOSSRANGE directives | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0649 | run_accept.at:29: 649. ACCEPT OMITTED (simple) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog < input.txt | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0650 | run_accept.at:51: 650. ACCEPT FROM TIME / DATE / DAY / DAY-O | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0651 | run_accept.at:141: 651. ACCEPT FROM TIME / DATE / DAY / DAY- | other | COB_CURRENT_DATE='2015/04/05 18:45:22.123400056' \ | CANDIDATE_UNSUPPORTED |
| 0652 | run_accept.at:283: 652. ACCEPT DATE / DAY and intrinsic func | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0653 | run_accept.at:320: 653. ACCEPT DATE / DAY and intrinsic func | other | COB_CURRENT_DATE='2020/06/12 18:45:22' \ | CANDIDATE_UNSUPPORTED |
| 0654 | run_accept.at:367: 654. ACCEPT OMITTED (SCREEN) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog < input.txt | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0655 | run_initialize.at:29: 655. INITIALIZE group entry with OCCUR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0657 | run_initialize.at:90: 657. INITIALIZE OCCURS with SIGN LEADI | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0662 | run_initialize.at:298: 662. INITIALIZE group item | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0663 | run_initialize.at:415: 663. INITIALIZE with REDEFINES | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0664 | run_initialize.at:442: 664. INITIALIZE with FILLER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0666 | run_initialize.at:560: 666. INITIALIZE with reference-modifi | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0667 | run_initialize.at:596: 667. INITIALIZE big table with VALUE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0669 | run_misc.at:23: 669. Comma separator without space | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0670 | run_misc.at:44: 670. DECIMAL-POINT is COMMA (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0672 | run_misc.at:100: 672. DECIMAL-POINT is COMMA (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0673 | run_misc.at:128: 673. DECIMAL-POINT is COMMA (4) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0674 | run_misc.at:156: 674. DECIMAL-POINT is COMMA (5) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0675 | run_misc.at:190: 675. CURRENCY SIGN | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0676 | run_misc.at:221: 676. CURRENCY SIGN WITH PICTURE SYMBOL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | ORACLE_XFAIL |
| 0689 | run_misc.at:746: 689. MOVE Z'literal' | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0690 | run_misc.at:788: 690. Floating continuation indicator | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0691 | run_misc.at:810: 691. Fixed continuation indicator | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0692 | run_misc.at:852: 692. Concatenation operator | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0693 | run_misc.at:876: 693. SOURCE FIXED/FREE directives | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0694 | run_misc.at:912: 694. TURN directive | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0695 | run_misc.at:953: 695. OCCURS on level 01 | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0697 | run_misc.at:1060: 697. Index and parenthesized expression | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0698 | run_misc.at:1082: 698. Alphanumeric and binary numeric | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0699 | run_misc.at:1106: 699. Non-numeric data in numeric items | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./unchecked_prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0713 | run_misc.at:1726: 713. TRANSFORM statement | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0714 | run_misc.at:1759: 714. INSPECT CONVERTING alphabet | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0715 | run_misc.at:1808: 715. INSPECT CONVERTING TO figurative cons | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0716 | run_misc.at:1830: 716. INSPECT CONVERTING NULL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0717 | run_misc.at:1852: 717. INSPECT CONVERTING TO NULL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0718 | run_misc.at:1874: 718. INSPECT CONVERTING complex | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0719 | run_misc.at:1903: 719. INSPECT numeric signed | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0722 | run_misc.at:2110: 722. INSPECT TALLYING BEFORE and AFTER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0723 | run_misc.at:2146: 723. INSPECT TALLYING REPLACING BEFORE and | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0724 | run_misc.at:2183: 724. INSPECT REPLACING figurative constant | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0728 | run_misc.at:2337: 728. PERFORM VARYING Float | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0732 | run_misc.at:2453: 732. EXIT PARAGRAPH | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0735 | run_misc.at:2551: 735. PERFORM FOREVER / PERFORM UNTIL EXIT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0741 | run_misc.at:2712: 741. STRING WITH POINTER ON OVERFLOW with  | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0744 | run_misc.at:2944: 744. UNSTRING DELIMITED ALL SPACE-2 | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0748 | run_misc.at:3201: 748. UNSTRING with FUNCTION / literal | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0749 | run_misc.at:3271: 749. SORT: table | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0750 | run_misc.at:3305: 750. SORT: table (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0751 | run_misc.at:3430: 751. SORT: table (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0752 | run_misc.at:3522: 752. SORT: table (toplevel) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0753 | run_misc.at:3544: 753. SORT: EBCDIC table | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0758 | run_misc.at:3967: 758. PIC ZZZ-, ZZZ+ | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0763 | run_misc.at:4182: 763. Lookup ENTRY from main executable | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0765 | run_misc.at:4249: 765. ALLOCATE / FREE with BASED item (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0766 | run_misc.at:4275: 766. ALLOCATE / FREE with BASED item (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0767 | run_misc.at:4319: 767. ALLOCATE CHARACTERS INITIALIZED (TO) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0772 | run_misc.at:4641: 772. CALL in from C, cob_call_params expli | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./caller | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0773 | run_misc.at:4700: 773. CALL in from C, cob_call_params unkno | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./caller | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0774 | run_misc.at:4753: 774. CALL C with callback, PROCEDURE DIVIS | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0775 | run_misc.at:4823: 775. CALL C with callback, ENTRY-CONVENTIO | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0776 | run_misc.at:4974: 776. CALL in from C with init missing / im | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./caller | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0777 | run_misc.at:5022: 777. CALL STATIC C from COBOL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./caller | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0780 | run_misc.at:5157: 780. ANY LENGTH (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0781 | run_misc.at:5200: 781. ANY LENGTH (4) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0783 | run_misc.at:5268: 783. access to BASED item without allocati | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0786 | run_misc.at:5363: 786. STOP RUN WITH ERROR STATUS | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0788 | run_misc.at:5406: 788. SYMBOLIC clause | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0789 | run_misc.at:5439: 789. OCCURS clause with 1 entry | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0790 | run_misc.at:5480: 790. Computing of different USAGEs w/o dec | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0791 | run_misc.at:6005: 791. Computing of different USAGEs w/- dec | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0795 | run_misc.at:7063: 795. POINTER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0797 | run_misc.at:7169: 797. ON EXCEPTION clause of DISPLAY | COBCRUN_DIRECT ./prog (launcher run) | COB_EXIT_WAIT=0 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0798 | run_misc.at:7194: 798. EC-SCREEN-LINE-NUMBER and -STARTING-C | COBCRUN_DIRECT ./prog (launcher run) | COB_EXIT_WAIT=0 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0800 | run_misc.at:7273: 800. SET LAST EXCEPTION TO OFF | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0805 | run_misc.at:7574: 805. void PROCEDURE, NOTHING return | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0815 | run_misc.at:11549: 815. Alphanumeric MOVE with truncation | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0821 | run_misc.at:12226: 821. OPTIONS paragraph, DEFAULT ROUNDED M | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0823 | run_misc.at:12357: 823. ZERO unsigned and negative binary su | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0824 | run_misc.at:12434: 824. Default Arithmetic (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0828 | run_misc.at:12763: 828. SET CONSTANT directive | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0831 | run_misc.at:12998: 831. 78 VALUE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0832 | run_misc.at:13066: 832. 01 CONSTANT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0834 | run_misc.at:13248: 834. FLOAT-DECIMAL w/o SIZE ERROR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0835 | run_misc.at:13422: 835. FLOAT-SHORT / FLOAT-LONG w/o SIZE ER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0836 | run_misc.at:13621: 836. FLOAT-SHORT with SIZE ERROR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0837 | run_misc.at:13672: 837. FLOAT-LONG with SIZE ERROR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0838 | run_misc.at:13730: 838. EC-SIZE-ZERO-DIVIDE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0839 | run_misc.at:13773: 839. EC-SIZE-OVERFLOW | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0842 | run_misc.at:13983: 842. runtime checks within conditions | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0845 | run_misc.at:14292: 845. libcob version check | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0846 | run_misc.at:14397: 846. assorted math | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0848 | run_file.at:23: 848. OPEN EXTEND and CLOSE, SEQUENTIAL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0849 | run_file.at:89: 849. variable-length SEQUENTIAL data integri | COBCRUN_DIRECT ./prog (launcher run) | DD_DATA=TEST1 $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0850 | run_file.at:154: 850. DELETE FILE, SEQUENTIAL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0851 | run_file.at:224: 851. OUTPUT on SEQUENTIAL file to missing d | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0852 | run_file.at:261: 852. OPEN EXTEND and CLOSE, INDEXED | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0853 | run_file.at:333: 853. DELETE FILE, INDEXED | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0854 | run_file.at:406: 854. OUTPUT on INDEXED file to missing dire | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0858 | run_file.at:601: 858. REWRITE a RELATIVE file with RANDOM ac | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0862 | run_file.at:796: 862. ASSIGN filename expansion | other | test -f "./FILENAME" && rm -f "./FILENAME" | CANDIDATE_RUNTIME_FAIL |
| 0863 | run_file.at:823: 863. ASSIGN filename mapping | other | test -f "FILENAME2" | CANDIDATE_UNSUPPORTED |
| 0864 | run_file.at:999: 864. ASSIGN with COB_FILE_PATH | other | test -f "tstdir/FILENAMEX" && rm -f "tstdir/FILENAMEX" | CANDIDATE_UNSUPPORTED |
| 0865 | run_file.at:1036: 865. ASSIGN DYNAMIC with LOCAL-STORAGE ite | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0866 | run_file.at:1073: 866. ASSIGN DYNAMIC with LOCAL-STORAGE ite | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0867 | run_file.at:1113: 867. ASSIGN DYNAMIC with BASED data item | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog X | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0868 | run_file.at:1199: 868. ASSIGN DYNAMIC with data item in LINK | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0869 | run_file.at:1371: 869. ASSIGN DYNAMIC with empty data item | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0870 | run_file.at:1411: 870. ASSIGN DYNAMIC with unset implicit da | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0871 | run_file.at:1443: 871. INDEXED file key-name | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0872 | run_file.at:1485: 872. INDEXED file sparse/split keys | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0873 | run_file.at:2005: 873. INDEXED file split keys WITH DUPLICAT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0874 | run_file.at:2171: 874. INDEXED file variable length record | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0875 | run_file.at:2447: 875. INDEXED sample | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0876 | run_file.at:2948: 876. WRITE + REWRITE FILE name | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0877 | run_file.at:3109: 877. START RELATIVE (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0878 | run_file.at:3158: 878. START RELATIVE (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0879 | run_file.at:3234: 879. START RELATIVE (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0880 | run_file.at:3310: 880. READ on OPTIONAL missing RELATIVE / S | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0881 | run_file.at:3432: 881. READ on OPTIONAL missing INDEXED file | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0883 | run_file.at:3546: 883. DECLARATIVES procedure referencing | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0885 | run_file.at:3630: 885. System routines for directories (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0886 | run_file.at:3672: 886. System routines for directories (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0887 | run_file.at:3769: 887. System routines for files | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0888 | run_file.at:3966: 888. System routines for files - filename  | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog notthere | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0889 | run_file.at:4083: 889. System routine CBL_COPY_FILE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog2 | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0891 | run_file.at:4192: 891. SEQUENTIAL basic I/O | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0892 | run_file.at:4241: 892. LINE SEQUENTIAL basic I/O | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0893 | run_file.at:4307: 893. LINE SEQUENTIAL WRITE AFTER | other | cat TEST-FILE | CANDIDATE_UNSUPPORTED |
| 0894 | run_file.at:4353: 894. LINE SEQUENTIAL record truncation (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0895 | run_file.at:4457: 895. LINE SEQUENTIAL record truncation (2) | other |  | CANDIDATE_UNSUPPORTED |
| 0896 | run_file.at:4593: 896. LINE SEQUENTIAL standard record overf | other |  | CANDIDATE_UNSUPPORTED |
| 0897 | run_file.at:4729: 897. LINAGE and LINAGE-COUNTER sample | COBCRUN_DIRECT ./prog (launcher run) | COB_CURRENT_DATE="2015/02/06 16:40:52" $COBCRUN_DIRECT ./pro | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0899 | run_file.at:5306: 899. SEQUENTIAL file I/O with variable rec | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0900 | run_file.at:5381: 900. LINE SEQUENTIAL file I/O with variabl | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0901 | run_file.at:5455: 901. SEQUENTIAL file REWRITE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0902 | run_file.at:5567: 902. SEQUENTIAL file with LOCK MODE EXCLUS | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0903 | run_file.at:5637: 903. SEQUENTIAL file with OPEN WITH LOCK | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0904 | run_file.at:5705: 904. SEQUENTIAL file with SHARING NO | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0905 | run_file.at:5775: 905. SEQUENTIAL file with SHARING READ ONL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0906 | run_file.at:5850: 906. SEQUENTIAL file with blocked lock | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0907 | run_file.at:5923: 907. RELATIVE SEQUENTIAL basic I/O | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0908 | run_file.at:5957: 908. RELATIVE RANDOM basic I/O | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0909 | run_file.at:6007: 909. RELATIVE SEQUENTIAL with variable rec | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0910 | run_file.at:6081: 910. INDEXED SEQUENTIAL basic I/O | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0911 | run_file.at:6119: 911. INDEXED SEQUENTIAL with variable reco | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0912 | run_file.at:6201: 912. INDEXED file with LOCK MODE EXCLUSIVE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0913 | run_file.at:6280: 913. INDEXED file with OPEN WITH LOCK | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0914 | run_file.at:6358: 914. INDEXED file with SHARING NO | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0915 | run_file.at:6437: 915. INDEXED file with SHARING READ ONLY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0916 | run_file.at:6523: 916. INDEXED file with blocked lock | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0917 | run_file.at:6605: 917. INDEXED file with LOCK AUTOMATIC (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0918 | run_file.at:6696: 918. INDEXED file with LOCK AUTOMATIC (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0919 | run_file.at:6788: 919. INDEXED file with LOCK MANUAL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | ORACLE_XFAIL |
| 0920 | run_file.at:6878: 920. START INDEXED | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0921 | run_file.at:6957: 921. INDEXED partial keys | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0923 | run_file.at:7275: 923. READ INPUT pipe & WRITE OUTPUT pipe | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | ORACLE_XFAIL |
| 0936 | run_file.at:11235: 936. RELATIVE Multi-Record | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0937 | run_file.at:11485: 937. INDEXED File READ/DELETE/READ | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog 1>prog.out | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0938 | run_file.at:12291: 938. TURN EC-I-O | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | ORACLE_XFAIL |
| 0939 | run_file.at:12388: 939. LINE SEQUENTIAL REWRITE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0940 | run_file.at:12660: 940. LINE SEQUENTIAL data | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0941 | run_file.at:12817: 941. Concatenated Files | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0943 | run_file.at:13011: 943. File SORT, SEQUENTIAL variable recor | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0944 | run_file.at:13097: 944. File SORT, LINE SEQUENTIAL | other | cat result.txt | CANDIDATE_UNSUPPORTED |
| 0945 | run_file.at:13156: 945. File SORT, LINE SEQUENTIAL same file | other | cat test.txt | CANDIDATE_UNSUPPORTED |
| 0946 | run_file.at:13203: 946. File SORT, LINE SEQUENTIAL variable  | other | cat file2 | CANDIDATE_UNSUPPORTED |
| 0948 | run_file.at:13334: 948. File MERGE, LINE SEQUENTIAL variable | other | cat file3 | CANDIDATE_UNSUPPORTED |
| 0950 | run_file.at:13460: 950. SORT with INPUT/OUTPUT PROCEDUREs | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0951 | run_file.at:13550: 951. SORT with key1 ASCENDING, key2 DESCE | other | cat file2 | CANDIDATE_UNSUPPORTED |
| 0952 | run_file.at:13617: 952. Scope of FD GLOBAL in nested program | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0953 | run_file.at:13774: 953. OPEN / CLOSE with multiple filenames | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 > prog1.txt | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0954 | run_reportwriter.at:23: 954. Report Line Order | COBCRUN_DIRECT ./prog (launcher run) | DD_PRINTOUT="./report.txt" $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0955 | run_reportwriter.at:120: 955. REPORT COL PLUS | other | diff reference report.txt | CANDIDATE_UNSUPPORTED |
| 0956 | run_reportwriter.at:189: 956. Report Overlapping Fields | other | diff reference report.txt | CANDIDATE_UNSUPPORTED |
| 0957 | run_reportwriter.at:258: 957. EMPTY REPORT | COBCRUN_DIRECT ./prog (launcher run) | DD_PRINTOUT="./report.txt" $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0958 | run_reportwriter.at:327: 958. PAGE LIMIT REPORT | COBCRUN_DIRECT ./prog (launcher run) | DD_PRINTOUT=./report.txt $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0959 | run_reportwriter.at:388: 959. PAGE LIMIT REPORT 2 | other | diff reference report.txt | CANDIDATE_UNSUPPORTED |
| 0960 | run_reportwriter.at:460: 960. Sample Customer Report | COBCRUN_DIRECT ./prog (launcher run) | DD_DATAIN="./inp_data" DD_SYSPRINT="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0961 | run_reportwriter.at:775: 961. Sample Charge Report | COBCRUN_DIRECT ./prog (launcher run) | DD_DATAIN="./inp_data" DD_SYSPRINT="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0962 | run_reportwriter.at:1128: 962. Sample Charge Report 2 | COBCRUN_DIRECT ./prog (launcher run) | DD_DATAIN="./inp_data" DD_SYSPRINT="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0963 | run_reportwriter.at:1498: 963. Sample Charge Report 3 | COBCRUN_DIRECT ./prog (launcher run) | DD_DATAIN="./inp_data" DD_SYSPRINT="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0964 | run_reportwriter.at:1798: 964. Sample Charge Report 4 | COBCRUN_DIRECT ./prog (launcher run) | DD_DATAIN="./inp_data" DD_SYSPRINT="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0965 | run_reportwriter.at:2214: 965. Sample Payroll Report | COBCRUN_DIRECT ./prog (launcher run) | DD_DATAIN="./inp_data" DD_SYSPRINT="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0967 | run_reportwriter.at:3063: 967. STUDENT REPORT with INITIAL | COBCRUN_DIRECT ./prog (launcher run) | DD_STUDENT="./inp_data" DD_REPORT1="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0968 | run_reportwriter.at:3215: 968. ORDER REPORT; Test substring | COBCRUN_DIRECT ./prog (launcher run) | DD_CUSTORD="./inp_data" DD_REPORT2="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0969 | run_reportwriter.at:3563: 969. Sample Control Break | COBCRUN_DIRECT ./prog (launcher run) | DD_STUDREC="./inp_data" DD_REPORT3="./report.txt" $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0971 | run_reportwriter.at:3985: 971. Duplicate Detail Line | other | diff reference report.txt | CANDIDATE_UNSUPPORTED |
| 0972 | run_reportwriter.at:4113: 972. Report with OCCURS | COBCRUN_DIRECT ./prog (launcher run) | DD_PRINTOUT=./report.txt $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0975 | run_reportwriter.at:4580: 975. Duplicate INITIATE | COBCRUN_DIRECT ./prog (launcher run) | DD_PRINTOUT=./report.txt $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0976 | run_reportwriter.at:4646: 976. Missing INITIATE and GENERATE | COBCRUN_DIRECT ./prog (launcher run) | DD_PRINTOUT=./report.txt $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0977 | run_reportwriter.at:4706: 977. Missing INITIATE and TERMINAT | COBCRUN_DIRECT ./prog (launcher run) | DD_PRINTOUT=./report.txt $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0978 | run_reportwriter.at:4760: 978. Next Group Next Page | COBCRUN_DIRECT ./prog (launcher run) | DD_TEMPFILE=./inp_data DD_REPORTFILE=./report.txt $COBCRUN_D | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0979 | run_reportwriter.at:9083: 979. Report PRESENT AFTER | other | DD_INFILE=./inp_data DD_OREPORT=./report.txt \ | CANDIDATE_RUNTIME_FAIL |
| 0980 | run_reportwriter.at:9345: 980. Varying and Nested OCCURS | other | PRINTOUT=tstdmrp.txt \ | ORACLE_XFAIL |
| 0981 | run_reportwriter.at:9519: 981. BEFORE REPORTING | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0991 | run_functions.at:221: 991. FUNCTION CHAR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0993 | run_functions.at:284: 993. FUNCTION CONCAT / CONCATENATE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0994 | run_functions.at:331: 994. FUNCTION CONCATENATE with referen | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0995 | run_functions.at:359: 995. FUNCTION BIT-OF and BIT-TO-CHAR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0996 | run_functions.at:416: 996. FUNCTION HEX-OF and HEX-TO-CHAR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0997 | run_functions.at:547: 997. FUNCTION CONTENT-LENGTH | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0998 | run_functions.at:581: 998. FUNCTION CONTENT-OF | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 0999 | run_functions.at:656: 999. FUNCTION as CALL parameter BY CON | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1002 | run_functions.at:739: 1002. FUNCTION CURRENT-DATE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1004 | run_functions.at:835: 1004. FUNCTION DATE-TO-YYYYMMDD | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1006 | run_functions.at:885: 1006. FUNCTION DAY-TO-YYYYDDD | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1009 | run_functions.at:969: 1009. FUNCTION EXCEPTION-LOCATION | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1010 | run_functions.at:1057: 1010. FUNCTION EXCEPTION-STATEMENT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1011 | run_functions.at:1092: 1011. FUNCTION EXCEPTION-STATUS | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1017 | run_functions.at:1292: 1017. FUNCTION FORMATTED-DATE with re | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1018 | run_functions.at:1317: 1018. FUNCTION FORMATTED-DATETIME | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1019 | run_functions.at:1374: 1019. FUNCTION FORMATTED-DATETIME wit | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1020 | run_functions.at:1400: 1020. FUNCTION FORMATTED-TIME | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1021 | run_functions.at:1487: 1021. FUNCTION FORMATTED-TIME DP.COMM | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1022 | run_functions.at:1518: 1022. FUNCTION FORMATTED-TIME with re | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1024 | run_functions.at:1574: 1024. FUNCTION HIGHEST-ALGEBRAIC | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1025 | run_functions.at:1642: 1025. FUNCTION INTEGER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1030 | run_functions.at:1801: 1030. FUNCTION LENGTH | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1034 | run_functions.at:1965: 1034. FUNCTION LOCALE-TIME-FROM-SECON | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1037 | run_functions.at:2039: 1037. FUNCTION LOWER-CASE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1039 | run_functions.at:2093: 1039. FUNCTION LOWEST-ALGEBRAIC | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1046 | run_functions.at:2286: 1046. FUNCTION MOD (invalid) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1050 | run_functions.at:2401: 1050. FUNCTION MODULE-ID | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1051 | run_functions.at:2422: 1051. FUNCTION MODULE-PATH | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1053 | run_functions.at:2468: 1053. FUNCTION MODULE-TIME | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1054 | run_functions.at:2493: 1054. FUNCTION MONETARY-DECIMAL-POINT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1055 | run_functions.at:2516: 1055. FUNCTION MONETARY-THOUSANDS-SEP | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1056 | run_functions.at:2539: 1056. FUNCTION NUMERIC-DECIMAL-POINT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1057 | run_functions.at:2562: 1057. FUNCTION NUMERIC-THOUSANDS-SEPA | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1058 | run_functions.at:2585: 1058. FUNCTION NUMVAL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1059 | run_functions.at:2658: 1059. FUNCTION NUMVAL-C | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1060 | run_functions.at:2719: 1060. FUNCTION NUMVAL-C DP.COMMA | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1067 | run_functions.at:2888: 1067. FUNCTION RANDOM | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1068 | run_functions.at:2910: 1068. FUNCTION RANGE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1069 | run_functions.at:2932: 1069. FUNCTION REM (valid) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1070 | run_functions.at:2954: 1070. FUNCTION REM (invalid) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1072 | run_functions.at:3004: 1072. FUNCTION REVERSE with reference | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1073 | run_functions.at:3027: 1073. FUNCTION SECONDS-FROM-FORMATTED | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1075 | run_functions.at:3104: 1075. FUNCTION SIGN | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1080 | run_functions.at:3238: 1080. FUNCTION SUBSTITUTE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1081 | run_functions.at:3265: 1081. FUNCTION SUBSTITUTE with refere | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1083 | run_functions.at:3316: 1083. FUNCTION SUBSTITUTE-CASE with r | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1089 | run_functions.at:3531: 1089. FUNCTION TEST-FORMATTED-DATETIM | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1090 | run_functions.at:3612: 1090. FUNCTION TEST-FORMATTED-DATETIM | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1091 | run_functions.at:3665: 1091. FUNCTION TEST-FORMATTED-DATETIM | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1093 | run_functions.at:3857: 1093. FUNCTION TEST-NUMVAL-C | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1096 | run_functions.at:4083: 1096. FUNCTION TRIM with reference mo | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1099 | run_functions.at:4161: 1099. FUNCTION UPPER-CASE with refere | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1101 | run_functions.at:4216: 1101. FUNCTION WHEN-COMPILED | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1102 | run_functions.at:4270: 1102. FUNCTION YEAR-TO-YYYY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1104 | run_functions.at:4375: 1104. FORMATTED-(DATE)TIME with SYSTE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1107 | run_functions.at:4457: 1107. User-Defined FUNCTION with/with | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1108 | run_functions.at:4508: 1108. UDF in COMPUTE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1112 | run_extensions.at:72: 1112. Numeric Boolean literals | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1115 | run_extensions.at:188: 1115. Hexadecimal numeric literals | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1116 | run_extensions.at:217: 1116. Semi-parenthesized condition | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1117 | run_extensions.at:237: 1117. ADDRESS OF | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1118 | run_extensions.at:287: 1118. LENGTH OF | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1119 | run_extensions.at:451: 1119. SET TO SIZE OF | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1120 | run_extensions.at:488: 1120. WHEN-COMPILED | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1121 | run_extensions.at:517: 1121. Complex OCCURS DEPENDING ON (1) | other |  | CANDIDATE_UNSUPPORTED |
| 1122 | run_extensions.at:546: 1122. Complex OCCURS DEPENDING ON (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1127 | run_extensions.at:846: 1127. OCCURS UNBOUNDED (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1128 | run_extensions.at:908: 1128. OCCURS UNBOUNDED (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./ALLOC | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1129 | run_extensions.at:1048: 1129. INITIALIZE OCCURS UNBOUNDED | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1132 | run_extensions.at:1463: 1132. DEPENDING ON with ODOSLIDE for | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1133 | run_extensions.at:1568: 1133. INITIALIZE level 01 OCCURS | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1139 | run_extensions.at:1914: 1139. CALL BY VALUE numeric literal  | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1142 | run_extensions.at:2126: 1142. Quoted PROGRAM-ID | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1143 | run_extensions.at:2149: 1143. PROGRAM-ID AS clause | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1148 | run_extensions.at:2374: 1148. PROCEDURE DIVISION CHAINING | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog X ABCD | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1149 | run_extensions.at:2497: 1149. STOP RUN RETURNING/GIVING | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog1 | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1150 | run_extensions.at:2553: 1150. GOBACK/EXIT PROGRAM RETURNING/ | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1152 | run_extensions.at:2654: 1152. ASSIGN to KEYBOARD/DISPLAY | COBCRUN_DIRECT ./prog (launcher run) | cat TEST-FILE | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1153 | run_extensions.at:2712: 1153. SORT ASSIGN KEYBOARD to ASSIGN | COBCRUN_DIRECT ./prog (launcher run) | cat TEST-FILE | $COBCRUN_DIRECT ./prog | ORACLE_XFAIL |
| 1154 | run_extensions.at:2774: 1154. Environment/Argument variable | COBCRUN_DIRECT ./prog (launcher run) | TEST_ENV=OK $COBCRUN_DIRECT ./prog CHECKPAR | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1155 | run_extensions.at:2820: 1155. 78 Level (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1156 | run_extensions.at:2843: 1156. 78 Level (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1157 | run_extensions.at:2869: 1157. 78 Level (3) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1166 | run_extensions.at:3335: 1166. System routine C$JUSTIFY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1167 | run_extensions.at:3360: 1167. System routine C$PRINTABLE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1168 | run_extensions.at:3389: 1168. System routine C$MAKEDIR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1169 | run_extensions.at:3410: 1169. System routine C$GETPID | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1170 | run_extensions.at:3435: 1170. System routine C$TOUPPER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1171 | run_extensions.at:3460: 1171. System routine C$TOLOWER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1172 | run_extensions.at:3485: 1172. System routine CBL_OR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1173 | run_extensions.at:3512: 1173. System routine CBL_NOR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1174 | run_extensions.at:3539: 1174. System routine CBL_AND | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1175 | run_extensions.at:3566: 1175. System routine CBL_XOR | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1176 | run_extensions.at:3593: 1176. System routine CBL_IMP | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1177 | run_extensions.at:3620: 1177. System routine CBL_NIMP | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1178 | run_extensions.at:3647: 1178. System routine CBL_NOT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1179 | run_extensions.at:3673: 1179. System routine CBL_EQ | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1180 | run_extensions.at:3700: 1180. System routine CBL_GC_GETOPT | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog --version --verbose -jkl | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1181 | run_extensions.at:4137: 1181. System routine CBL_GC_FORK | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1182 | run_extensions.at:4205: 1182. System routine CBL_GC_WAITPID | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | ORACLE_SKIP |
| 1184 | run_extensions.at:4387: 1184. System routine SYSTEM, paramet | COBCRUN_DIRECT ./prog (launcher run) | PATH=.:$PATH $COBCRUN_DIRECT prog "start" | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1185 | run_extensions.at:4498: 1185. System routine CBL_EXIT_PROC | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1186 | run_extensions.at:4608: 1186. System routine CBL_ERROR_PROC  | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1187 | run_extensions.at:4707: 1187. System routine CBL_ERROR_PROC  | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1193 | run_extensions.at:4952: 1193. Conditional / define directive | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1194 | run_extensions.at:4980: 1194. Conditional / define directive | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1196 | run_extensions.at:5035: 1196. Variable format | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1197 | run_extensions.at:5074: 1197. COBOLX format | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1200 | run_extensions.at:5246: 1200. MF FREE format (X/Open) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog-free | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1202 | run_extensions.at:5327: 1202. Binary COMP-1 (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1203 | run_extensions.at:5363: 1203. EXHIBIT statement | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1206 | run_extensions.at:5571: 1206. GCOS floating-point usages | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1207 | run_extensions.at:5610: 1207. PICTURE L (basic) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1208 | run_extensions.at:5740: 1208. PICTURE L (under/over shoot) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./under | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1209 | run_extensions.at:5795: 1209. PICTURE L (MOVE CORRESPONDING) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./corr | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1210 | run_extensions.at:5873: 1210. PICTURE L (OCCURS ... PIC L) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./nested | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1211 | run_extensions.at:5942: 1211. PICTURE L (REDEFINES) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./redefines | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1212 | run_extensions.at:6030: 1212. INSPECT TRAILING | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1213 | run_extensions.at:6100: 1213. INSPECT REPLACING TRAILING ZER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1214 | run_extensions.at:6122: 1214. INSPECT REPLACING complex | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1219 | run_ml.at:127: 1219. XML GENERATE SUPPRESS | other |  | CANDIDATE_UNSUPPORTED |
| 1220 | run_ml.at:204: 1220. XML GENERATE exceptions | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1221 | run_ml.at:316: 1221. XML GENERATE record selection | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1222 | run_ml.at:360: 1222. XML GENERATE trimming | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1223 | run_ml.at:457: 1223. XML DPC-IN-DATA directive | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1225 | run_ml.at:532: 1225. JSON GENERATE general | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1226 | run_ml.at:589: 1226. JSON GENERATE SUPPRESS | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1227 | run_ml.at:628: 1227. JSON GENERATE exceptions | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1228 | run_ml.at:693: 1228. JSON GENERATE record selection | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1229 | run_ml.at:737: 1229. JSON GENERATE trimming | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1230 | run_ml.at:837: 1230. JSON DPC-IN-DATA directive | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1239 | data_binary.at:1185: 1239. BINARY: 64bit unsigned compare | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1244 | data_binary.at:1409: 1244. MOVE DISPLAY to BINARY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1245 | data_binary.at:1595: 1245. MOVE PACKED-DECIMAL to BINARY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1246 | data_binary.at:1781: 1246. MOVE BINARY to PACKED-DECIMAL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1247 | data_binary.at:1961: 1247. MOVE BINARY to BINARY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1248 | data_binary.at:2143: 1248. PPP COMP-5 | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1253 | data_display.at:172: 1253. DISPLAY: unsigned | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1254 | data_display.at:226: 1254. MOVE DISPLAY to DISPLAY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1255 | data_display.at:408: 1255. PPP DISPLAY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1257 | data_display.at:533: 1257. DISPLAY: ADD and SUBTRACT w/o SIZ | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1258 | data_display.at:16940: 1258. DISPLAY: ADD and SUBTRACT, all  | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1261 | data_packed.at:221: 1261. PACKED-DECIMAL used with MOVE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1262 | data_packed.at:459: 1262. MOVE PACKED-DECIMAL to PACKED-DECI | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1263 | data_packed.at:639: 1263. MOVE PACKED-DECIMAL to DISPLAY | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1264 | data_packed.at:825: 1264. MOVE DISPLAY to PACKED-DECIMAL | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1266 | data_packed.at:1063: 1266. PACKED-DECIMAL arithmetic | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1267 | data_packed.at:1187: 1267. PACKED-DECIMAL comparison | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1268 | data_packed.at:1269: 1268. PACKED-DECIMAL numeric test (1) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1269 | data_packed.at:1333: 1269. PACKED-DECIMAL numeric test (2) | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1271 | data_packed.at:1432: 1271. COMP-6 used with MOVE | other | $COMPILE -fno-fast-compare -C -o progalt.c prog.cob | WRAPPER_OPTION_UNSUPPORTED |
| 1272 | data_packed.at:1474: 1272. COMP-6 arithmetic | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1273 | data_packed.at:1563: 1273. COMP-6 numeric | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1274 | data_packed.at:1609: 1274. COMP-6 comparison | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1275 | data_packed.at:1671: 1275. COMP-3 vs. COMP-6 - BCD compariso | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1276 | data_packed.at:1971: 1276. PPP COMP-3 | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1277 | data_packed.at:2083: 1277. PPP COMP-6 | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1278 | data_packed.at:2174: 1278. arithmetic truncation with USAGE  | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1279 | data_packed.at:2236: 1279. MOVE between several BCD fields | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1280 | data_packed.at:12326: 1280. BCD ADD and SUBTRACT w/o SIZE ER | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1281 | data_packed.at:28746: 1281. BCD ADD and SUBTRACT, all ROUNDE | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |
| 1282 | data_pointer.at:21: 1282. POINTER: display | COBCRUN_DIRECT ./prog (launcher run) | $COBCRUN_DIRECT ./prog | CANDIDATE_MODULE_MODEL_UNSUPPORTED |

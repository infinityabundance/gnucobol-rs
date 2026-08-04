
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  tab1.
           03  tab1-entry    OCCURS 5 PIC S999.
       01  tab2.
           03  tab2-entry    OCCURS 7 PIC S99.
           03  tab2b-entry   OCCURS 7.
               05  x         PIC S99.
       01  tab3.
           03  sub-tab-3  OCCURS 2.
               05 tab3-entry OCCURS 5 PIC S999.

       PROCEDURE       DIVISION.
           ADD      TABLE tab1-entry TO   tab2-entry.
           SUBTRACT TABLE tab2-entry FROM tab1-entry.
           ADD      TABLE tab1-entry TO   tab3-entry (1).
           SUBTRACT TABLE tab2-entry FROM tab3-entry (2).
           ADD      TABLE tab1-entry TO   x.
           SUBTRACT TABLE x          FROM tab1-entry.
           ADD      TABLE tab1-entry TO   tab2b-entry.
           GOBACK.

      * Fixed-format literal continuation (the CCVS85 corpus idiom): a col-7 `-` continuation of a
      * NONNUMERIC literal joins the previous line FLUSH -- the literal runs to column 72 of the
      * first line, the continuation resumes at column 12, and the QUOTE at column 12 is the
      * continuation marker (not part of the value). Lines taken VERBATIM from the corpus (IC101A's
      * HYPHEN-LINE / CCVS-C-1, sequence area included): the front-end previously emitted the
      * continuation as a separate line (breaking the VALUE); oracle-verified byte-identical.
      * @format: fixed
000100 IDENTIFICATION DIVISION.                                            IC1014.2
000200 PROGRAM-ID. P193.                                                   IC1014.2
000300 DATA DIVISION.                                                      IC1014.2
000400 WORKING-STORAGE SECTION.                                            IC1014.2
018500 01  HYPHEN-LINE.                                                 IC1014.2
018600     02 FILLER  PIC IS X VALUE IS SPACE.                          IC1014.2
018700     02 FILLER  PIC IS X(65)    VALUE IS "************************IC1014.2
018800-    "*****************************************".                 IC1014.2
018900     02 FILLER  PIC IS X(54)    VALUE IS "************************IC1014.2
019000-    "******************************".                            IC1014.2
009300 01  CCVS-C-1.                                                    IC1014.2
009400     02 FILLER  PIC IS X(99)    VALUE IS " FEATURE              PAIC1014.2
009500-    "SS  PARAGRAPH-NAME                                          IC1014.2
009600-    "       REMARKS".                                            IC1014.2
009700     02 FILLER                     PIC X(20)    VALUE SPACE.      IC1014.2
001600 PROCEDURE DIVISION.                                                IC1014.2
001700     DISPLAY HYPHEN-LINE.                                           IC1014.2
001800     DISPLAY CCVS-C-1.                                              IC1014.2
001900     STOP RUN.                                                      IC1014.2

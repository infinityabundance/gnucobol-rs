
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       78  POS0      VALUE NEXT.
       01  MYREC.
          05  FLD1   PIC 9(2).
          05  FLD2   PIC X(7).
             78  POS3    VALUE NEXT.
          05  FLD3   PIC X(2) OCCURS 5 TIMES.
             78  POS4    VALUE NEXT.
          05  FLD4   PIC X(4).
             78  POS-NEXT   VALUE NEXT.
       77  MYREC2 PIC X.
       01  MYREC3 EXTERNAL.
          05  FLD5   PIC X(4).
             78  POS5    VALUE NEXT.
          05  FLD6   PIC X(4).
       01  PICX      PIC XXX VALUE 'Abc'.
       78  HUN       VALUE 10 * (10 + LENGTH OF PICX) + 12.35-2+3.
       78  HUN2      VALUE HUN * (10 + LENGTH
                                       OF PICX) -4.
       78  DIV1      VALUE 100 / 3.
       78  STRT4     VALUE START OF FLD4.
       LINKAGE SECTION.
       01  XMYREC.
          05  XFLD1   PIC 9(2).
          05  XFLD2   PIC X(7).
             78  XPOS3    VALUE NEXT.
          05  XFLD3   PIC X(2) OCCURS 5 TIMES.
             78  XPOS4    VALUE NEXT.
          05  XFLD4   PIC X(4).
          05  XFLD5   PIC X(4).
       78  XSTRT4     VALUE START OF XFLD4.
      *
       PROCEDURE DIVISION.
           STOP RUN.

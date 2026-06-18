      *> 66-level RENAMES (start THRU end) -- a regrouping alias over a contiguous range of sibling fields,
      *> modelled as a Group so reads concat and a MOVE distributes across the renamed leaves. Identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P86.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 REC.
          05 A PIC X(2) VALUE "AA".
          05 B PIC X(2) VALUE "BB".
          05 C PIC X(2) VALUE "CC".
       66 NAB RENAMES A THRU B.
       66 NALL RENAMES A THRU C.
       PROCEDURE DIVISION.
           DISPLAY "NAB=[" NAB "]".
           DISPLAY "NALL=[" NALL "]".
           MOVE "XYZW" TO NAB.
           DISPLAY "A=[" A "] B=[" B "] C=[" C "]".
           STOP RUN.

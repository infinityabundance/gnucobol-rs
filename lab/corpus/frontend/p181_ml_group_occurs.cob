      *> JSON/XML GENERATE over a source containing a flat group-OCCURS table. cobc 3.2 renders only the
      *> FIRST occurrence's children as a single object/element ({"ROW":{"K":..,"V":..}}) -- a documented
      *> -Wpending behaviour, reproduced byte-for-byte here.
      *> @no312: JSON/XML GENERATE over a group-OCCURS evolved between 3.1.2 and 3.2; port targets 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P181.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SRC.
          05 HDR PIC X(3) VALUE "hdr".
          05 ROW OCCURS 2.
             10 K PIC 9(2) VALUE 7.
             10 V PIC X(2) VALUE "ab".
       01 DJ PIC X(200).
       01 DX PIC X(200).
       01 LN PIC 9(4).
       PROCEDURE DIVISION.
           JSON GENERATE DJ FROM SRC COUNT LN.
           DISPLAY "json: " DJ(1:LN).
           XML  GENERATE DX FROM SRC COUNT LN.
           DISPLAY "xml:  " DX(1:LN).
           STOP RUN.

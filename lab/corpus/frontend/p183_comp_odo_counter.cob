      *> An OCCURS DEPENDING ON counter held in a BINARY (COMP) field, value >= 16 so the binary byte
      *> (0x12 = 18) is NOT recoverable from its low nibble alone -- this is the shape that exposed a
      *> resolve_int bug where a COMP counter was misread nibble-by-nibble as zoned display (18 -> 2).
      *> FUNCTION LENGTH(T) is the live DEPENDING length; the group image is that many bytes. Identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P183.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N   PIC 9(4) COMP VALUE 18.
       01 T.
          05 E PIC X OCCURS 1 TO 20 DEPENDING ON N.
       PROCEDURE DIVISION.
           MOVE ALL "z" TO T.
           DISPLAY "LEN=" FUNCTION LENGTH(T).
           DISPLAY "GRP=" T.
           STOP RUN.


       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       78  I   VALUE 20.
       78  J   VALUE 5000.
       78  M   VALUE 5.
       01  SETUP-REC.
           05  FL1       PIC X(04).
           05  FL2       PIC ZZZZZ.
           05  FL3       PIC 9(04).
           05  FL4       PIC 9(08) COMP.
           05  FL5       PIC 9(04) COMP-4.
           05  FL6       PIC Z,ZZZ.99.
           05  FL7       PIC S9(05) SIGN LEADING SEPARATE.
           05  FL8       PIC X(04).
           05  FL9 REDEFINES FL8 PIC 9(04).
           05  FLA.
               10  FLB OCCURS I TIMES.
                   15  FLC PIC X(02).
               10  FLD   PIC X(20).
           05  FLD1      PIC X(100).
           05  FLD2 OCCURS M TO J TIMES DEPENDING ON FL5.
               10  FILLER PIC X(01).
           05  FLD3      PIC X(3).
           05  FLD4      PIC X(4).
       LINKAGE          SECTION.
       01  O1  PIC X OCCURS UNBOUNDED DEPENDING ON FL4.
       01  O2.
           05  O-START   PIC X(42).
           05  FILLER    OCCURS UNBOUNDED DEPENDING ON FL9.
             07  O-ENTRY.
               10  O-E1  PIC 9(24).
               10  O-E2  PIC XXX.
       PROCEDURE        DIVISION.
           STOP RUN.

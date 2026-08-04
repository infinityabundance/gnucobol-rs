
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
      *> Verify that we report conflict markers correctly
<<<<<<< HEAD
       01  one-1      PIC X.
======= there may be something here
       01  one-2      PIC X.
>>>>>>> some note
      *> Verify that we don't have an issue with unmatched conflict markers
       77  var        PIC 9.
<<<<<<< HEAD
       01  var2       PIC X.
=======
<<<<<<< HEAD
      *> Verify that we only report conflict markers at the start of lines.
 <<<<<<< HEAD
       01  one        PIC X.
 =======
       01  two        PIC 9.
 >>>>>>> some note
       PROCEDURE DIVISION.
           GOBACK.

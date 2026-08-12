
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       PROCEDURE DIVISION.
       MAIN SECTION.
       MAIN-00.
           PERFORM SUB1
           PERFORM SUB1 THRU SUB1-EX
           PERFORM SUB1-00 THRU SUB1-EX
           PERFORM SUB1 THRU SUB1
           PERFORM SUB1 THRU SUB2
           GOBACK.
       MAIN-EX. EXIT.
      *
       SUB1 SECTION.
       SUB1-00.
           PERFORM SUB1-01 THRU SUB1-02.
       SUB1-01.
           CONTINUE.
       SUB1-02.
           CONTINUE.
       SUB1-EX. EXIT.
      *
       SUB2 SECTION.
       SUB2-00.
           PERFORM SUB1-02 THRU SUB1-01.
       SUB2-01.
           CONTINUE.
       SUB2-02.
           CONTINUE.
       SUB2-EX. EXIT.

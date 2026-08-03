*> @format: free
*> The CCVS85 report idiom (found in the NC/SQ/IC units): `PERFORM X THRU X-EXIT` whose body
*> conditionally `GO TO X-EXIT` -- jumping to the LAST paragraph of the performed range. Control
*> must stay INSIDE the performed range and return to the statement AFTER the PERFORM. The front-end
*> previously propagated the jump to the body level and re-ran the following section forever
*> (an unbounded loop the 1e7-jump guard only caught later); the second PERFORM exercises the
*> fall-through path with a non-space subject. Oracle-verified byte-identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P192.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 CORRECT-A PIC X(5) VALUE SPACES.
       01 N PIC 9 VALUE 0.
       PROCEDURE DIVISION.
           PERFORM BAIL-OUT THRU BAIL-OUT-EX.
           ADD 1 TO N.
           DISPLAY "AFTER " N.
           MOVE "X" TO CORRECT-A.
           PERFORM BAIL-OUT THRU BAIL-OUT-EX.
           ADD 1 TO N.
           DISPLAY "AFTER " N.
           STOP RUN.
       BAIL-OUT.
           IF CORRECT-A EQUAL TO SPACE GO TO BAIL-OUT-EX.
           DISPLAY "BAIL WRITE".
       BAIL-OUT-EX.
           EXIT.

      *> COMPUTE: unary minus glued to an operand / paren (-A, -(A-B), -(-(-3))) and RIGHT-associative
      *> exponentiation (2 ** 3 ** 2 = 512). Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P154.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9 VALUE 2.
       01 B PIC 9 VALUE 3.
       01 R PIC S9(7)V9(4) VALUE 0.
       PROCEDURE DIVISION.
           COMPUTE R = -A + B. DISPLAY "R1=[" R "]".
           COMPUTE R = -(A - B). DISPLAY "R2=[" R "]".
           COMPUTE R = 2 ** 3 ** 2. DISPLAY "R3=[" R "]".
           COMPUTE R = -(-(-3)). DISPLAY "R4=[" R "]".
           COMPUTE R = (-A) ** 2. DISPLAY "R5=[" R "]".
           COMPUTE R = A * -B. DISPLAY "R6=[" R "]".
           STOP RUN.

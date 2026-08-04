
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-LINE-NUMBER         PIC 9(3) VALUE ZERO.
       01 WS-NUMBER              PIC 9(3)V99 VALUE ZERO.
       01 WS-TEXT                PIC X(5) VALUE 'CAT'.
       01 PIC-9-SIGNED           PIC S9(2) VALUE 5.
       01 PIC-9-SIGNED-DECIMAL   PIC S9(2)V99 VALUE 5.
       01 PIC-9-NOT-SIGNED       PIC 9(3) VALUE 5.
       01 PIC-9-NOT-DECIMAL      PIC 9(3) VALUE 5.
       01 XX                     PIC 9(2) VALUE 2.
       01 PIC-9-DECIMAL          PIC 9(3)V9 VALUE 5.
       01 COMPUTE-1              PIC 999V9999 VALUE 654.1873.
       01 GROUP-ITEM-X6.
          05 FILLER              PIC X(6) VALUE 'CAT'.
       78 CONST1                 VALUE 'CAT     '.
       01 CONST2                 CONSTANT AS 00000001234.
       01 IF-D16                 PIC PP99 VALUE .0012.
       PROCEDURE DIVISION.
       MAIN.
           IF GROUP-ITEM-X6 = '1234567'   CONTINUE.
           IF PIC-9-NOT-DECIMAL = 1.1     CONTINUE.
           IF PIC-9-NOT-DECIMAL = 1.0
               CONTINUE.
           IF PIC-9-DECIMAL = 1.01
               CONTINUE.
           IF PIC-9-DECIMAL = 1.100
               CONTINUE.
           IF PIC-9-SIGNED NOT = 11.0
               CONTINUE.
           IF PIC-9-NOT-SIGNED < 0
               CONTINUE.
           IF PIC-9-NOT-SIGNED < ZERO
               CONTINUE.
           IF PIC-9-NOT-SIGNED < (25 - 50)
               CONTINUE.
           IF PIC-9-DECIMAL = (2.24 / 2)
               CONTINUE.
           IF PIC-9-NOT-DECIMAL = "123"
               CONTINUE.
           IF PIC-9-NOT-DECIMAL = "1B0"       *> field is numeric
               CONTINUE.
           IF PIC-9-NOT-DECIMAL (1:3) = "1B0" *> refmod is always alphanumeric
               CONTINUE.
           IF PIC-9-NOT-DECIMAL (1:3) NOT = "Hot Doggy"
               CONTINUE.
           IF PIC-9-NOT-DECIMAL (1:XX) NOT = "Hi"
               CONTINUE.
           IF WS-LINE-NUMBER > '123'
              CONTINUE.
           IF WS-TEXT > 'DOGGY'
              CONTINUE.
           IF WS-NUMBER > 123.999
              CONTINUE.
           IF WS-LINE-NUMBER > 2345
              CONTINUE.
           IF WS-LINE-NUMBER <= 1234
              CONTINUE.
           IF WS-LINE-NUMBER > '1234'
              CONTINUE.
           IF 5432 < WS-LINE-NUMBER
              CONTINUE.
           IF 7855 >= WS-LINE-NUMBER
              CONTINUE.
           IF 1234 < WS-LINE-NUMBER
              CONTINUE.
           IF 5432 >= WS-LINE-NUMBER
              CONTINUE.
           IF WS-TEXT > 'DOGGY++'
              CONTINUE.
           IF WS-TEXT > 3141596
              CONTINUE.
           IF WS-TEXT > 3.141596
              CONTINUE.
           IF WS-TEXT = 3.141596
              CONTINUE.
           IF 'DOG+CAT' NOT = WS-TEXT
              CONTINUE.
           IF WS-TEXT = 'CAT+DOG'
              CONTINUE.
           EVALUATE TRUE
              WHEN 'DOG+CAT' = WS-TEXT
                 CONTINUE
              WHEN CONST1 = 'CAT+DOG'
                 CONTINUE
              WHEN CONST1 = 'CAT'
                 CONTINUE
              WHEN CONST2 = 1234
                 CONTINUE
           END-EVALUATE
           IF WS-LINE-NUMBER > 0000234
              CONTINUE.
           IF WS-TEXT = 'CAT         '
              CONTINUE.
           IF ( COMPUTE-1 < 654.20038) AND
              ( COMPUTE-1 > 654.17422) THEN
              CONTINUE.
           IF ( COMPUTE-1 < 5654.20) CONTINUE.
           IF ( COMPUTE-1 > 5654.20) CONTINUE.
           IF   COMPUTE-1 < 05654.20 CONTINUE.
           IF   COMPUTE-1 > 05654.20 CONTINUE.
           IF ( 5654.20 > COMPUTE-1) CONTINUE.
           IF ( 5654.20 < COMPUTE-1) CONTINUE.
           IF  05654.20 > COMPUTE-1  CONTINUE.
           IF  05654.20 < COMPUTE-1  CONTINUE.
           IF IF-D16 POSITIVE
              CONTINUE.
           IF IF-D16 NOT POSITIVE
              CONTINUE.
           IF IF-D16 NEGATIVE
              CONTINUE.
           IF IF-D16 NOT NEGATIVE
              CONTINUE.
           IF PIC-9-NOT-SIGNED > (25 - 50)
               CONTINUE.
           IF PIC-9-NOT-SIGNED >= -1  CONTINUE.
           IF PIC-9-NOT-SIGNED >= -.1 CONTINUE.
           IF PIC-9-NOT-SIGNED > 0.0  CONTINUE.
           IF PIC-9-NOT-SIGNED >  .0  CONTINUE.
           IF PIC-9-NOT-SIGNED > ZERO
               CONTINUE.
           IF PIC-9-NOT-SIGNED >= 0.0 CONTINUE.
           IF PIC-9-NOT-SIGNED >=  .0 CONTINUE.
           IF PIC-9-NOT-SIGNED >= ZERO
               CONTINUE.
           IF GROUP-ITEM-X6 (1:6) = '123456'
               CONTINUE.
      * Both have correct error check verified in syn_refmod.at
      *    IF GROUP-ITEM-X6 (2:6) = '123456'
      *        CONTINUE.
      *    IF GROUP-ITEM-X6 (WS-LINE-NUMBER:7) = '123456'
      *        CONTINUE.
           IF GROUP-ITEM-X6 (1:5) = '123456'
               CONTINUE.
           IF GROUP-ITEM-X6 (3:) = '12345'
               CONTINUE.
           IF GROUP-ITEM-X6 (3:WS-LINE-NUMBER) = '12345'
               CONTINUE.
           IF GROUP-ITEM-X6 (WS-LINE-NUMBER:3) = '12345'
               CONTINUE.
           IF GROUP-ITEM-X6 (WS-LINE-NUMBER:WS-LINE-NUMBER) = '12345'
               CONTINUE.
           IF GROUP-ITEM-X6 (WS-LINE-NUMBER:) = '12345'
               CONTINUE.
           IF PIC-9-NOT-DECIMAL    >     9     CONTINUE.
           IF PIC-9-NOT-DECIMAL    >   009     CONTINUE.
           IF PIC-9-NOT-DECIMAL    >   900     CONTINUE.
           IF PIC-9-NOT-DECIMAL    >   909     CONTINUE.
           IF PIC-9-NOT-DECIMAL    >   999     CONTINUE.
           IF PIC-9-NOT-DECIMAL    >  0000999  CONTINUE.
           IF PIC-9-DECIMAL        >   999     CONTINUE.
           IF PIC-9-DECIMAL        >   990.9   CONTINUE.
           IF PIC-9-DECIMAL        >   999.9   CONTINUE.
           IF PIC-9-DECIMAL        >  0999.90  CONTINUE.
           IF PIC-9-DECIMAL        > -0999.90  CONTINUE.
           IF PIC-9-SIGNED-DECIMAL >    99.99  CONTINUE.
           IF PIC-9-SIGNED-DECIMAL >=   99.99  CONTINUE.
           IF PIC-9-SIGNED-DECIMAL <   -99.99  CONTINUE.
           IF PIC-9-SIGNED-DECIMAL <= -099.990 CONTINUE.
           IF PIC-9-SIGNED-DECIMAL <= -099.991 CONTINUE.
           IF 99                   > XX CONTINUE.
           IF XX               NOT < 99 CONTINUE.
           IF NOT XX               < 99 CONTINUE.

           STOP RUN.

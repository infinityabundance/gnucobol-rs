
       ENVIRONMENT    DIVISION.
       INPUT-OUTPUT   SECTION.
       FILE-CONTROL.
           SELECT testfile
               ASSIGN TO filename
               ORGANIZATION RELATIVE
               ACCESS IS sequentia
               STATUS IS stat.

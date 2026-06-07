       IDENTIFICATION DIVISION.
       PROGRAM-ID. PROG7.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 CUSTOMER.
       COPY TPL
              REPLACING ==:PFX:== BY ==CC==
                        ==AMOUNT== BY ==MONEY==.
       PROCEDURE DIVISION.
           STOP RUN.

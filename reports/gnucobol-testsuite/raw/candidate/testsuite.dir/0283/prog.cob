
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.                                           																																																																																																																																		
       DATA             DIVISION.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  
       WORKING-STORAGE  SECTION.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   *> This is a real comment
       PROCEDURE        DIVISION.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  This is commentary only
           CONTINUE.                                              *> comment after column 72
      *    This is a very long comment that exceeds column 72 but doesn't exceed 512 bytes, therefore not leading to a line overflow. As it is a comment line there is no "Source text after column 72" warning
           CONTINUE.                                                    CONTINUE.
           CONTINUE.                                                                        
           STOP RUN.


       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 TAB-1.
           05 TAB-1-ENTRY OCCURS 50
                          ASCENDING KEY IS TAB-1-KEY
                          INDEXED BY TAB-1-INDEX.
             10 TAB-1-KEY.
               15 T1K-SUB1     PIC X(03).
               15 T1K-SUB2     PIC 9(03).
             10 TAB-1-DATA     PIC X(50).
       01 TAB-2.
           05 TAB-2-ENTRY OCCURS 50
                          INDEXED BY TAB-2-INDEX.
             10 TAB-2-KEY.
               15 T2K-SUB1     PIC X(03).
               15 T2K-SUB2     PIC 9(03).
             10 TAB-2-DATA     PIC X(50).

       PROCEDURE DIVISION.
       TESTING SECTION.
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN TAB-1-KEY = ALL ZERO
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN TAB-1-KEY (TAB-1-INDEX) = ALL ZERO
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN "ABC123" = TAB-1-KEY (TAB-1-INDEX)
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN TAB-1-KEY (TAB-2-INDEX) = ALL ZERO
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              WHEN TAB-1-KEY (TAB-1-INDEX) = ALL ZERO
                 CONTINUE
              AT END
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN TAB-1-KEY (TAB-1-INDEX) = ALL ZERO
              WHEN TAB-1-KEY (TAB-1-INDEX) = "ZZZ999"
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN TESTING
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN T1K-SUB1 (TAB-1-INDEX) = "ZZZ"
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN TAB-2-KEY (TAB-1-INDEX) = ALL ZERO
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN TAB-2-KEY (TAB-2-INDEX) = ALL ZERO
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-1-ENTRY
              AT END
                 CONTINUE
              WHEN "AA" = "BB"
                 CONTINUE
           END-SEARCH
           SEARCH ALL TAB-2-ENTRY
              AT END
                 CONTINUE
              WHEN TAB-2-KEY (TAB-2-INDEX) = ALL ZERO
                 CONTINUE
           END-SEARCH
           .

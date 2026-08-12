
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE SECTION.
       77  myhandle      usage handle.
       77  mythread      usage handle of thread.
       77  WINDOW-HANDLE usage handle of window.
       PROCEDURE DIVISION.
       MAIN.
           DISPLAY WINDOW, LINE 1, COLUMN 29, SIZE 51, LINES 12,
               ERASE SCREEN BOXED SHADOW, POP-UP AREA IS WINDOW-HANDLE
           END-DISPLAY
           DISPLAY "THIS IS TEXT IN A WINDOW"
           DISPLAY FLOATING WINDOW UPON WINDOW-HANDLE,
               LINE 5, COLUMN 10, SIZE 20, LINES 2,
               ERASE SCREEN, POP-UP AREA IS myhandle
           END-DISPLAY
           DISPLAY "Some text for the floating window"
           ACCEPT OMITTED
           CLOSE WINDOW myhandle
           DISPLAY FLOATING WINDOW,
               LINE 5, COLUMN 10, SIZE 20, LINES 2,
               ERASE SCREEN, POP-UP AREA mythread
           END-DISPLAY
           ACCEPT OMITTED
           DESTROY WINDOW-HANDLE
           DISPLAY WINDOW AT 1020 SIZE 36 LINES 15 BOXED
              FOREGROUND-COLOR IS 7
              BACKGROUND-COLOR IS 0
              TOP CENTERED TITLE IS 'SOME TITLE'
              pop-up area = WINDOW-HANDLE.
           DISPLAY SUBWINDOW UPON WINDOW-HANDLE SHADOW
              AT 0505 SIZE 25 LINES 10
              BOTTOM LEFT TITLE = 'buttom left'
              BACKGROUND-COLOR IS 10
              FOREGROUND-COLOR IS 5.
           DISPLAY WINDOW UPON WINDOW-HANDLE SHADOW
              AT 0808 SIZE 18 LINES 5
              RIGHT TITLE 'top right'
              BACKGROUND-COLOR 1
              FOREGROUND-COLOR 14.
           ACCEPT OMITTED
           DESTROY WINDOW-HANDLE
      *
           STOP RUN.

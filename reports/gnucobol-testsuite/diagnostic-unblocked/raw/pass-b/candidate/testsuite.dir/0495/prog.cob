
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       77  WS-Where           pic x(512).

       01  ws-mysql.
           02  ws-mysql-cid                        usage pointer.
           02  ws-mysql-result                     usage pointer.
           02  ws-mysql-result-2                   usage pointer.
           02  ws-mysql-result-3                   usage pointer.
           02  ws-mysql-count-rows                 pic s9(9) comp.
           02  ws-mysql-error-number               pic x(4).
           02  ws-mysql-error-message              pic x(80).
           02  ws-mysql-host-name                  pic x(64).
           02  ws-mysql-implementation             pic x(64).
           02  ws-mysql-password                   pic x(64).
           02  ws-mysql-base-name                  pic x(64).
           02  ws-mysql-port-number                pic x(4).
           02  ws-mysql-socket                     pic x(64).
           02  ws-mysql-command                    pic x(4096).

       01  ws-No-Paragraph pic 9(4).
       local-storage section.
       01  subscripts usage comp-5.
           12 J                    pic s9(4).
           12 K                    pic s9(4).
           12 L                    pic s9(4).

       SCREEN           SECTION.
       01  Display-Message-1 foreground-color 2.
           03  value "WS-Where=" line 23 col  1.
           03  from WS-Where (1:J) pic x(69) col 10.
       01  Display-Message-2 foreground-color 2.
           03  value "ST004 SQL Err No.=" line 4 col  1.
           03  using ws-mysql-error-number pic x(4) col 19.
           03  value " Para=" col 23.
           03  using WS-No-Paragraph pic 9(3) col 29.
           03  value " SQL Cmd=" col 32.
           03  using ws-mysql-command pic x(199) col 41.
           03  value "SQL Err Msg=" line 7 col  1.
           03  using ws-mysql-error-message pic x(67) col 13.
       PROCEDURE        DIVISION.
           STOP RUN.

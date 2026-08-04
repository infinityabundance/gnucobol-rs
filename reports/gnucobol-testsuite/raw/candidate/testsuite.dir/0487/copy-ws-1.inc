
       01  TEXTFILE-1-NAME          PIC X(080)     VALUE "TEXTFILE.1".
       01  TEXTFILE-1-OCFG          PIC X(001)     VALUE "C".
           88 TEXTFILE-1-NOTOPEN    VALUE "C".
           88 TEXTFILE-1-IS-OPEN    VALUE "I", "O", "U".
       01  TEXTFILE-1-SIZE          PIC 9(004) EXTERNAL.
      *

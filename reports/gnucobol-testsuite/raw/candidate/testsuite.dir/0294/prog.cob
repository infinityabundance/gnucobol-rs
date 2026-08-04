
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE SECTION.
       77  my-text             pic x(10) value "TEXT".
       78  MB-OK               VALUE 1.
       78  MB-YES-NO           VALUE 2.
       78  MB-OK-CANCEL        VALUE 3.
       78  MB-YES-NO-CANCEL    VALUE 4.
       78  MB-YES              VALUE 1.
       78  MB-NO               VALUE 2.
       78  MB-CANCEL           VALUE 3.
       78  MB-DEFAULT-ICON     VALUE 1.
       78  MB-WARNING-ICON     VALUE 2.
       78  MB-ERROR-ICON       VALUE 3.
       PROCEDURE DIVISION.
           DISPLAY MESSAGE "Important"
                   TITLE "Very important"
                   TYPE = MB-OK
                   ICON IS MB-WARNING-ICON
           DISPLAY MESSAGE "This is" space "my" space my-text
           DISPLAY MESSAGE BOX "More messages?"
                   TYPE  MB-YES-NO
                   TITLE = "box title"
                   DEFAULT IS MB-YES
                   RETURNING RETURN-CODE
      *
           STOP RUN.

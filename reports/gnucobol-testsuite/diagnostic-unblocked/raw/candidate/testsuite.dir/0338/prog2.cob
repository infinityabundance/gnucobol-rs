
       ID,;DIVISION;,.,;
           author,.;tester.
       PROGRAM-ID,;.;,prog2;,.;,
           REMARKS;. Should work.,,
       ENVIRONMENT,;DIVISION;,.,;
       CONFIGURATION;;,,SECTION;;,,.
       SOURCE-COMPUTER;;.,,whatever;;DEBUGGING,,MODE;,.

      DDATA;DIVISION,.
      DWORKING-STORAGE,SECTION;.
       01;i,PIC;9;.

       PROCEDURE;DIVISION,.;
           IF;,i;,GREATER,;THAN;,OR,;EQUAL ,;TO;;5;
           ,,,THEN;;;GOBACK.
           STOP,RUN;.,

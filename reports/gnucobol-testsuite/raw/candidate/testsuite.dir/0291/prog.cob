
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE SECTION.
       78  listdir-open                value 1.
       78  listdir-next                value 2.
       78  listdir-close               value 3.
       77  pattern           pic x(5) value "*.cob".
       77  directory         pic x(5) value ".".
       77  filename          pic x(256).
       77  mydir             usage handle.

       77  mythread          usage handle of thread.
       77  unused-thread           handle    thread.

       77  mywindow          usage handle of window.

       77  nor-a-handle      usage handle bananas.
       77  neither-a-handle  usage handle of apes.
       77  control-handle    usage handle of label.

       PROCEDURE DIVISION.
       MAIN.
      * Call LISTDIR-OPEN to get a directory handle.
           call "C$LIST-DIRECTORY"
              using listdir-open, directory, pattern.
           move return-code to mydir.
      * Call LISTDIR-NEXT to get the names of the files.
      * Repeat this operation until a filename containing only
      * spaces is returned.  The filenames are not necessarily
      * returned in any particular order.  Filenames may be
      * sorted on some machines and not on others.
           perform thread with test after until filename = spaces
             handle in mywindow
              call "C$LIST-DIRECTORY"
                 using listdir-next, mydir, filename
           end-perform.
           stop thread mywindow
      * Call LISTDIR-CLOSE to close the directory and deallocate
      * memory. Omitting this call will result in memory leaks.
           call "C$LIST-DIRECTORY" using listdir-close, mydir.
      *
           CALL IN THREAD 'NOTHERE'
                HANDLE IN mywindow
                USING  'STUFF'
              NOT ON EXCEPTION  DISPLAY 'called in THREAD'
           END-CALL
      *
      *    Just to check that the handles are still recognized and usable:
           destroy neither-a-handle, control-handle
      *
      *    check for invalid use
           add  neither-a-handle to control-handle
           compute mywindow = 0
           string mydir delimited by size into filename
      *
           STOP RUN.

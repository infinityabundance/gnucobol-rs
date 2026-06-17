      *> @env: COB_DISPLAY_PRINT_FILE=/tmp/gnucobol_rs_sweep_printer.out
      *> DISPLAY ... UPON PRINTER with COB_DISPLAY_PRINT_FILE set: the printer line is diverted to the
      *> file, so stdout shows only the plain DISPLAYs -- identical under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P35.
       PROCEDURE DIVISION.
           DISPLAY "ON-STDOUT-1".
           DISPLAY "ON-PRINTER" UPON PRINTER.
           DISPLAY "ON-STDOUT-2".
           STOP RUN.

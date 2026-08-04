
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE SECTION.
       01  GNUCOBOL-URL IS EXTERNAL-FORM
                IDENTIFIED BY "http://www.gnu.org/software/gnucobol/".
       01  WEB-PAGE-1 EXTERNAL-FORM,
                IDENTIFIED "cgipage1".
       01  SIMPLE-FORM IS EXTERNAL-FORM.
           03  SIMPLE-FORM-VAR1  PIC X(10).
           03  SIMPLE-FORM-VAR2  PIC 9(5).
       01  MY-FORM EXTERNAL-FORM.
           03  CGI-VAR1 PIC X(20) IDENTIFIED "Name".
           03  CGI-VAR2 PIC X(50) IDENTIFIED BY CGI-VAR1.

       PROCEDURE DIVISION.
       MAIN.
      *>   CGI display of static content (full URL)
           DISPLAY GNUCOBOL-URL
      *>   CGI display of static content (current URL + "cgipage1" + ".html")
           DISPLAY WEB-PAGE-1
      *>   CGI display of output form
           DISPLAY MY-FORM
      *>   CGI display of input  form (docs say: used for debugging purposes)
           DISPLAY SIMPLE-FORM
      *>   CGI accept
           ACCEPT SIMPLE-FORM
      *>   normal accept
           ACCEPT SIMPLE-FORM-VAR1
      *>   CGI accept with first var (with cgi identifier Name)
      *>   setting the cgi identifier of the second
           ACCEPT MY-FORM
      *
           STOP RUN.

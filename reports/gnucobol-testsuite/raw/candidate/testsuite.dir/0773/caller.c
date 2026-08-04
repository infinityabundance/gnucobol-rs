
#include <stdio.h>
#include <libcob.h>

int callee (char *, char *);

int
main (int argc, char **argv)
{
   /* for storing COBOL return code */
   int cob_ret;

   /* initialize parameters */
   char *p1 = "A";
   char *p2 = "FROM C";

   /* initialize the COBOL run-time library */
   cob_init (argc, argv);

   /* call COBOL program */
   cob_ret = callee (p1, p2);

   /* Clean up and terminate - This does not return */
   cob_stop_run (cob_ret);
}


#include <stdio.h>
#include <libcob.h>

#ifndef NULL
#define NULL (void*)0
#endif

int
main (int argc, char **argv)
{
   /* for storing COBOL return code */
   int cob_ret;

   /* initialize parameters */
   void *cob_argv[2];

   cob_argv[0] = argv[2];
   cob_argv[1] = NULL;

   /* initialize the COBOL run-time library */
   cob_init(argc, argv);

   /* call COBOL program */
   cob_ret = cob_call (argv[1], 2, cob_argv);

   cob_runtime_hint("program exited normally, "
       "without STOP RUN with status %d", cob_ret);

   /* Clean up and terminate - This does not return */
   cob_stop_run (cob_ret);
}

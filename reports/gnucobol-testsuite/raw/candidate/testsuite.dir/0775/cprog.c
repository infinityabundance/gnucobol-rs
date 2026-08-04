
#include <stdio.h>
#include <libcob.h>

COB_EXT_EXPORT int
cprog (void *cb)
{
   char *p1;
   int  p2 = 42;
   char *p3 = "CALLBACK";

   p1 = p3;
   ((int (*)(char *, int, char *))cb)(p1, p2, p3);
   return 0;
}

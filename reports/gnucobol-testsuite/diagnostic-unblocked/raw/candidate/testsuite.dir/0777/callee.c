
#include <string.h>

int
callee (char *p1, char *p2)
{
   if (p1[0] == 'A') {
      p1[0] = 'B';
   }
   memcpy (p2, "FROM C", 6);

   return 3;
}

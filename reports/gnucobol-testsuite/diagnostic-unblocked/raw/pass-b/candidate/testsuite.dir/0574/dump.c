
#include <stdio.h>
#include <libcob.h>

COB_EXT_EXPORT int
dump (char *p)
{
  printf ("%c%c", p[0], p[1]);
  return 0;
}

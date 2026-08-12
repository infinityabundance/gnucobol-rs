
#include <stdio.h>
#include <libcob.h>

COB_EXT_EXPORT int
dump (unsigned char *data, int *p)
{
  int i;
  if ( *p == 1 ) {
     for (i = 0; i < 4; i++)
       printf ("%02x", data[i]);
  } else {
       printf ("%8.8d", *((int *)data));
  }
  puts ("");
  return 0;
}

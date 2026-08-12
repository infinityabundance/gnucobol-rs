
#include <stdio.h>
#include <libcob.h>

COB_EXT_EXPORT int
dump (unsigned char *data)
{
  int i;
  for (i = 0; i < 8; i++)
    printf ("%02x", data[i]);
  puts ("");
  return 0;
}

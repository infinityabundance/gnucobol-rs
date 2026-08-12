
#include <stdio.h>
#include <libcob.h>

COB_EXT_EXPORT int
dump (unsigned char *data)
{
  printf ("%02x%02x%02x", data[0], data[1], data[2]);
  return 0;
}

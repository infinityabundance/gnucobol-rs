
#include <stdio.h>
#include <string.h>
#include <libcob.h>

COB_EXT_EXPORT int
setfilename (cob_file *f, unsigned char *name)
{
  memcpy (f->assign->data, name, strlen ((char *)name));
  return 0;
}

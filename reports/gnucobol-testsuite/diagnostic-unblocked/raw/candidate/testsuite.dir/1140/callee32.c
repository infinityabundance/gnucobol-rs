
#include <stdio.h>
#include <libcob.h>

COB_EXT_EXPORT int
callee32(cob_s32_t val)
{
  printf("VAL received: %d\n", val);
  return 0;
}

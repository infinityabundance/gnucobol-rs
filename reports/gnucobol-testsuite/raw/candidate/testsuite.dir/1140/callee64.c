
#include <stdio.h>
#include <libcob.h>

COB_EXT_EXPORT int
callee64(cob_s64_t val)
{
  printf("VAL received: " CB_FMT_LLD "\n", val);
  return 0;
}

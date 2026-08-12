
#include <libcob.h>

/* wrapper function as C functions are not
   accessible without explicit loading on all systems */
COB_EXT_EXPORT char *
calldyn (unsigned char *env_name)
{
  return cob_getenv (env_name);
}

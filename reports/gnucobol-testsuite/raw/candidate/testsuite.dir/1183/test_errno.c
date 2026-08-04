
#include <errno.h>
#include <stdio.h>

#include <libcob.h>

COB_EXT_EXPORT int
test_errno(void)
{
    FILE *fail;
    fail = fopen("file-not-to-be-found", "r");
    if (errno != 2) {
        printf("BAD ERRNO %d", errno);
    } else {
        if (fail) fclose(fail);
    }
    return 0;
}

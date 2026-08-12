
#include <stdio.h>
#include <libcob.h>

static char *txtOpCode(int opCode);

/*********************************************************
 *  TSTFH - External File Handler entry point.
*********************************************************/

COB_EXT_EXPORT int
TSTFH (unsigned char *opCodep, FCD3 *fcd)
{
   unsigned int   opCode;

   if (*opCodep == 0xfa)
      opCode = 0xfa00 + opCodep[1];
   else
      opCode = opCodep[1];

   if (fcd->fileOrg == ORG_LINE_SEQ
    || fcd->fileOrg == ORG_SEQ
    || fcd->fileOrg == ORG_INDEXED
    || fcd->fileOrg == ORG_RELATIVE) {
      switch (opCode) {
      case OP_OPEN_OUTPUT:
      case OP_OPEN_IO:
      case OP_OPEN_EXTEND:
      case OP_OPEN_OUTPUT_NOREWIND:
         return EXTFH(opCodep, fcd);
         break;

      case OP_OPEN_INPUT:
      case OP_OPEN_INPUT_NOREWIND:
      case OP_OPEN_INPUT_REVERSED:
         return EXTFH(opCodep, fcd);
         break;

      default:
         break;
      }

   }

   if (opCode == OP_CLOSE
    && (fcd->openMode & OPEN_NOT_OPEN) ) {
      return 0;
   }

   return EXTFH(opCodep, fcd);
}

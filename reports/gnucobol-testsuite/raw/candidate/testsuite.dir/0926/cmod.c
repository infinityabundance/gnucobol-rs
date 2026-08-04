
#include <stdio.h>
#include <libcob.h>

static char *txtOpCode(int opCode);

static int
doOpenFile(
   unsigned char  *opCodep,
   FCD3  *fcd,
   char  *opmsg)
{
   int      sts;

   sts = EXTFH( opCodep, fcd );
   printf("EXFTH did %s; Status=%c%c; File now %s\n",
       opmsg, fcd->fileStatus[0], fcd->fileStatus[1],
       (fcd->openMode & OPEN_NOT_OPEN) ? "Closed" : "Open");
   return sts;
}

/*********************************************************
 *  TSTFH - External File Handler entry point.
*********************************************************/

COB_EXT_EXPORT int
TSTFH (unsigned char *opCodep, FCD3 *fcd)
{
   unsigned int   opCode;
   int      sts;

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
         return doOpenFile( opCodep, fcd, txtOpCode(opCode));
         break;

      case OP_OPEN_INPUT:
      case OP_OPEN_INPUT_NOREWIND:
      case OP_OPEN_INPUT_REVERSED:
         return doOpenFile( opCodep, fcd, txtOpCode(opCode));
         break;

      case OP_CLOSE:
         return doOpenFile( opCodep, fcd, txtOpCode(opCode));
         break;

      default:
         break;
      }

   }

   if (opCode == OP_CLOSE
    && (fcd->openMode & OPEN_NOT_OPEN) ) {
      return 0;
   }

   sts = EXTFH(opCodep, fcd);
   printf("EXFTH did %s; Status=%c%c\n", txtOpCode(opCode),
       fcd->fileStatus[0], fcd->fileStatus[1]);
   return sts;
}

static char *           /* Return Text name of function */
txtOpCode(int opCode)
{
   static char tmp[32];
   switch (opCode) {
   case OP_OPEN_INPUT:     return "OPEN_IN";
   case OP_OPEN_OUTPUT:       return "OPEN_OUT";
   case OP_OPEN_IO:     return "OPEN_IO";
   case OP_OPEN_EXTEND:       return "OPEN_EXT";
   case OP_OPEN_INPUT_NOREWIND:  return "OPEN_IN_NOREW";
   case OP_OPEN_OUTPUT_NOREWIND: return "OPEN_OUT_NOREW";
   case OP_OPEN_INPUT_REVERSED:  return "OPEN_IN_REV";
   case OP_CLOSE:          return "CLOSE";
   case OP_CLOSE_LOCK:     return "CLOSE_LOCK";
   case OP_CLOSE_NOREWIND:    return "CLOSE_NORED";
   case OP_CLOSE_REEL:     return "CLOSE_REEL";
   case OP_CLOSE_REMOVE:      return "CLOSE_REMOVE";
   case OP_CLOSE_NO_REWIND:   return "CLOSE_NO_REW";
   case OP_START_EQ:       return "START_EQ";
   case OP_START_EQ_ANY:      return "START_EQ_ANY";
   case OP_START_GT:       return "START_GT";
   case OP_START_GE:       return "START_GE";
   case OP_START_LT:       return "START_LT";
   case OP_START_LE:       return "START_LE";
   case OP_READ_SEQ_NO_LOCK:  return "READ_SEQ_NO_LK";
   case OP_READ_SEQ:       return "READ_SEQ";
   case OP_READ_SEQ_LOCK:     return "READ_SEQ_LK";
   case OP_READ_SEQ_KEPT_LOCK:   return "READ_SEQ_KEPT_LK";
   case OP_READ_PREV_NO_LOCK:    return "READ_PREV_NO_LK";
   case OP_READ_PREV:      return "READ_PREV";
   case OP_READ_PREV_LOCK:    return "READ_PREV_LK";
   case OP_READ_PREV_KEPT_LOCK:  return "READ_PREV_KEPT_LK";
   case OP_READ_RAN:       return "READ_RAN";
   case OP_READ_RAN_NO_LOCK:  return "READ_RAN_NO_LK";
   case OP_READ_RAN_KEPT_LOCK:   return "READ_RAN_KEPT_LK";
   case OP_READ_RAN_LOCK:     return "READ_RAN_LK";
   case OP_READ_DIR:       return "READ_DIR";
   case OP_READ_DIR_NO_LOCK:  return "READ_DIR_NO_LK";
   case OP_READ_DIR_KEPT_LOCK:   return "READ_DIR_KEPT_LK";
   case OP_READ_DIR_LOCK:     return "READ_DIR_LK";
   case OP_READ_POSITION:     return "READ_POSITION";
   case OP_WRITE:          return "WRITE";
   case OP_REWRITE:     return "REWRITE";
   case OP_DELETE:      return "DELETE";
   case OP_DELETE_FILE:       return "DELETE_FILE";
   case OP_UNLOCK:      return "UNLOCK";
   case OP_ROLLBACK:       return "ROLLBACK";
   case OP_COMMIT:      return "COMMIT";
   case OP_WRITE_BEFORE:      return "WRITE_BEFORE";
   case OP_WRITE_BEFORE_TAB:  return "WRITE_BEFORE_TAB";
   case OP_WRITE_BEFORE_PAGE:    return "WRITE_BEFORE_PAGE";
   case OP_WRITE_AFTER:       return "WRITE_AFTER";
   case OP_WRITE_AFTER_TAB:   return "WRITE_AFTER_TAB";
   case OP_WRITE_AFTER_PAGE:  return "WRITE_AFTER_PAGE";
   }
   sprintf(tmp, "Func 0x%02X:", opCode);
   return tmp;
}

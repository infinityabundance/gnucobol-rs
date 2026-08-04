
#include <stdio.h>
#include <string.h>
#include <libcob.h>

static char *
getType (int type, int byvalue)
{
   static char wrk[24];
   switch (type) {
#if 1
   case COB_TYPE_GROUP:           return "Group";
   case COB_TYPE_NUMERIC_COMP5:
       /* fall through as the test will have different results
          on big endian systems otherwise
        return "COMP-5"; */
        COB_UNUSED (byvalue);
   case COB_TYPE_NUMERIC_BINARY:  return "BINARY";
   case COB_TYPE_NUMERIC_PACKED:  return "COMP-3";
   case COB_TYPE_NUMERIC_FLOAT:   return "COMP-1";
   case COB_TYPE_NUMERIC_DOUBLE:  return "COMP-2";
   case COB_TYPE_NUMERIC_DISPLAY: return "DISPLAY";
   case COB_TYPE_ALPHANUMERIC:    return "X";
   case COB_TYPE_NUMERIC_EDITED:  return "EDITED";
   case COB_TYPE_NATIONAL:        return "N";
#else
   case COB_TYPE_GROUP:           return "Group";
   case COB_TYPE_NUMERIC_COMP5:
        return byvalue == 2 ? "COMP-4" : "COMP-5";
   case COB_TYPE_NUMERIC_BINARY:  return "COMP-4";
   case COB_TYPE_NUMERIC_PACKED:  return "COMP-3";
   case COB_TYPE_NUMERIC_FLOAT:   return "COMP-1";
   case COB_TYPE_NUMERIC_DOUBLE:  return "COMP-2";
   case COB_TYPE_NUMERIC_DISPLAY: return "DISPLAY";
   case COB_TYPE_ALPHANUMERIC:    return "X";
   case COB_TYPE_NUMERIC_EDITED:  return "EDITED";
   case COB_TYPE_NATIONAL:        return "N";
#endif
   }
   sprintf (wrk,"Type %04X",type);
   return wrk;
}

COB_EXT_EXPORT int
CAPI (void *p1, ...)
{
   int      k,nargs,type,digits,scale,size,sign,byvalue;
   cob_s64_t   val = 0;
   char     *str;
   char     wrk[80],pic[30];	/* note: maximum _theoretical_ size */

   nargs = cob_get_num_params();
   printf ("CAPI called with %d parameters\n",nargs);
   for (k=1; k <= nargs; k++) {
      type   = cob_get_param_type (k);
      digits = cob_get_param_digits (k);
      scale  = cob_get_param_scale (k);
      size   = cob_get_param_size (k);
      sign   = cob_get_param_sign (k);
      byvalue = cob_get_param_constant(k);
      printf (" %d: %-8s ", k, getType (type, byvalue));
      if (byvalue) {
         printf ("BY VALUE     ");
      } else {
         printf ("BY REFERENCE ");
      }
      if (type == COB_TYPE_ALPHANUMERIC) {
         sprintf (pic, "X(%d)", size);
         str = cob_get_picx_param (k, NULL, 0);
         printf ("%-11s '%s'", pic, str);
         cob_free ((void*)str);
         cob_put_picx_param (k, "Bye!");
      } else if (type == COB_TYPE_NATIONAL) {
         sprintf (pic, "N(%d)", size); /* FIXME */
         printf ("exchange of national data is not supported yet");
      } else if (type == COB_TYPE_GROUP) {
         sprintf (pic, "(%d)", size);
         str = cob_get_grp_param (k, NULL, 0);
         printf ("%-11s '%.*s'", pic, size, str);
         cob_free ((void*)str);
         memset (wrk,' ',sizeof(wrk));
         memcpy (wrk,"Bye-Bye Birdie!",15);
         cob_put_grp_param (k, wrk, sizeof(wrk));
      } else if (type == COB_TYPE_NUMERIC_EDITED) {
         if (scale > 0) {
            sprintf (pic,"%s9(%d)V9(%d)",sign?"S":"",digits-scale,scale);
         } else {
            sprintf (pic,"%s9(%d)",sign?"S":"",digits-scale);
         }
         val = cob_get_s64_param (k);
         printf ("%-11s %lld ",pic,val);
         val = val + 130;
         val = -val;
         cob_put_s64_param (k, val);
         cob_get_grp_param (k, wrk, sizeof(wrk));
         printf (" to %.*s",size,wrk);
      } else {
         if(scale > 0) {
            sprintf (pic,"%s9(%d)V9(%d)",sign?"S":"",digits-scale,scale);
         } else {
            sprintf (pic,"%s9(%d)",sign?"S":"",digits-scale);
         }
         val = cob_get_s64_param (k);
         printf ("%-11s %lld", pic, val);
         cob_put_s64_param (k, val + 3);
      }
      printf (";\n");
      fflush (stdout);
   }
   if (nargs > 2) {
      cob_put_s64_param (7, val + 3);
   }
   return 0;
}

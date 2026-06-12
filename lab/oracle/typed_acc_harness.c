/* Oracle harness for move.c typed accessors. Row: "label variant signed len value".
 * variant: 0=compx 1=comp5 2=comp3 3=comp6 4=pic9. Calls cob_put_* then cob_get_*; prints
 * "label <put_hex> <get_value>". */
#include <libcob.h>
#include <stdio.h>
#include <string.h>
static cob_module ms; static cob_global *cg;
int main(int argc,char**argv){
  cob_init(argc,argv);
  memset(&ms,0,sizeof ms); ms.decimal_point='.'; ms.numeric_separator=','; ms.ebcdic_sign=0; ms.currency_symbol='$';
  cob_module *mp=&ms; cob_module_global_enter(&mp,&cg,0,0,0);
  char line[512],label[128]; int variant,signd,len; long long value;
  while(fgets(line,sizeof line,stdin)){
    if(line[0]=='#'||line[0]=='\n')continue;
    if(sscanf(line,"%127s %d %d %d %lld",label,&variant,&signd,&len,&value)!=5)continue;
    unsigned char m[64]; memset(m,0,sizeof m);
    long long got=0;
    if(signd){ long long v=value;
      switch(variant){case 0:cob_put_s64_compx(v,m,len);got=cob_get_s64_compx(m,len);break;
        case 1:cob_put_s64_comp5(v,m,len);got=cob_get_s64_comp5(m,len);break;
        case 2:cob_put_s64_comp3(v,m,len);got=cob_get_s64_comp3(m,len);break;
        case 4:cob_put_s64_pic9(v,m,len);got=cob_get_s64_pic9(m,len);break;}
    } else { unsigned long long v=(unsigned long long)value;
      switch(variant){case 0:cob_put_u64_compx(v,m,len);got=(long long)cob_get_u64_compx(m,len);break;
        case 1:cob_put_u64_comp5(v,m,len);got=(long long)cob_get_u64_comp5(m,len);break;
        case 2:cob_put_u64_comp3(v,m,len);got=(long long)cob_get_u64_comp3(m,len);break;
        case 3:cob_put_u64_comp6(v,m,len);got=(long long)cob_get_u64_comp6(m,len);break;
        case 4:cob_put_u64_pic9(v,m,len);got=(long long)cob_get_u64_pic9(m,len);break;}
    }
    printf("%s ",label); for(int i=0;i<len;i++)printf("%02x",m[i]); printf(" %lld\n",got);
  }
  return 0;
}

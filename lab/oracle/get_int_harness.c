/* Oracle harness for move.c's cob_get_int / cob_get_llint accessors. Reads a field spec + bytes,
 * calls the REAL libcob accessors, prints "label <int> <llint>". */
#include <libcob.h>
#include <stdio.h>
#include <string.h>
static int hexval(int c){ if(c>='0'&&c<='9')return c-'0'; if(c>='a'&&c<='f')return c-'a'+10; if(c>='A'&&c<='F')return c-'A'+10; return 0; }
static int parse_hex(const char*s,unsigned char*b,size_t n){ for(size_t i=0;i<n;i++){ if(!s[2*i]||!s[2*i+1])return 1; b[i]=(hexval((unsigned char)s[2*i])<<4)|hexval((unsigned char)s[2*i+1]); } return 0; }
static cob_module module_storage; static cob_global *cobglob;
int main(int argc,char**argv){
  cob_init(argc,argv);
  memset(&module_storage,0,sizeof(module_storage));
  module_storage.decimal_point='.'; module_storage.currency_symbol='$'; module_storage.numeric_separator=','; module_storage.ebcdic_sign=0; module_storage.flag_binary_truncate=1;
  cob_module *mp=&module_storage; cob_module_global_enter(&mp,&cobglob,0,0,0);
  char line[4096];
  while(fgets(line,sizeof line,stdin)){
    char label[256],hex[2050]; unsigned int type,dig,flags,size; int scale;
    if(line[0]=='#'||line[0]=='\n')continue;
    if(sscanf(line,"%255s %u %u %d %u %u %2049s",label,&type,&dig,&scale,&flags,&size,hex)!=7){ continue; }
    unsigned char data[1024]; if(size>sizeof data||parse_hex(hex,data,size))continue;
    cob_field_attr a; cob_field f; a.type=type;a.digits=dig;a.scale=scale;a.flags=flags;a.pic=NULL; f.size=size;f.data=data;f.attr=&a;
    int vi=cob_get_int(&f); long long vl=(long long)cob_get_llint(&f);
    printf("%s %d %lld\n",label,vi,vl);
  }
  return 0;
}

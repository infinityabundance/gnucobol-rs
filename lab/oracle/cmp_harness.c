/* cmp_harness.c -- oracle for gnucobol-rs numeric comparison (GNURUST.NUMCMP.1).
 * Calls the REAL cob_numeric_cmp on two cob_fields, prints the sign (-1/0/1).
 * Row: label t1 d1 sc1 fl1 sz1 hex1 t2 d2 sc2 fl2 sz2 hex2
 * types: 16 DISPLAY, 18 PACKED, 17 BINARY; flags: 1 HAVE_SIGN. */
#include <libcob.h>
#include <stdio.h>
#include <string.h>
static cob_module module_storage; static cob_global *cobglob;
static int hv(int c){ if(c>='0'&&c<='9')return c-'0'; if(c>='a'&&c<='f')return c-'a'+10; if(c>='A'&&c<='F')return c-'A'+10; return -1; }
static int ph(const char*s,unsigned char*b,size_t n){ for(size_t i=0;i<n;i++){int h=hv((unsigned char)s[2*i]),l=hv((unsigned char)s[2*i+1]); if(h<0||l<0)return -1; b[i]=(h<<4)|l;} return 0; }
int main(int argc,char**argv){
  char line[4096]; cob_init(argc,argv);
  memset(&module_storage,0,sizeof(module_storage));
  module_storage.decimal_point='.'; module_storage.currency_symbol='$'; module_storage.numeric_separator=','; module_storage.ebcdic_sign=0; module_storage.flag_binary_truncate=1;
  { cob_module*mp=&module_storage; cob_module_global_enter(&mp,&cobglob,0,0,0); }
  while(fgets(line,sizeof(line),stdin)){
    char label[256],h1[2050],h2[2050];
    unsigned int t1,d1,fl1,sz1,t2,d2,fl2,sz2; int sc1,sc2;
    unsigned char b1[1024],b2[1024]; cob_field_attr a1,a2; cob_field f1,f2;
    if(line[0]=='#'||line[0]=='\n')continue;
    if(sscanf(line,"%255s %u %u %d %u %u %2049s %u %u %d %u %u %2049s",
        label,&t1,&d1,&sc1,&fl1,&sz1,h1,&t2,&d2,&sc2,&fl2,&sz2,h2)!=13){ fprintf(stderr,"bad: %s",line); continue; }
    if(sz1>sizeof(b1)||sz2>sizeof(b2))continue;
    if(ph(h1,b1,sz1)||ph(h2,b2,sz2))continue;
    a1.type=t1;a1.digits=d1;a1.scale=sc1;a1.flags=fl1;a1.pic=NULL; f1.size=sz1;f1.data=b1;f1.attr=&a1;
    a2.type=t2;a2.digits=d2;a2.scale=sc2;a2.flags=fl2;a2.pic=NULL; f2.size=sz2;f2.data=b2;f2.attr=&a2;
    int r=cob_numeric_cmp(&f1,&f2);
    printf("%s %d\n",label,(r<0)?-1:(r>0)?1:0);
  }
  return 0;
}

//! Rust mirror of the typed-accessor oracle. Row: `label variant signed len value` -> `label put_hex get`.
use gnucobol_rs::accessors::*;
use std::io::{self,BufRead,Write};
fn main(){let so=io::stdout();let mut o=so.lock();
  for line in io::stdin().lock().lines().map_while(Result::ok){
    let f:Vec<&str>=line.split_whitespace().collect(); if f.len()!=5{continue}
    let (var,sg,len,v)=(f[1].parse::<u32>().unwrap(),f[2].parse::<u32>().unwrap(),f[3].parse::<usize>().unwrap(),f[4].parse::<i64>().unwrap());
    let mut m=vec![0u8;len];
    let got:i64 = if sg==1 { match var {
        0=>{cob_put_s64_compx(v,&mut m,len);cob_get_s64_compx(&m,len)},
        1=>{cob_put_s64_comp5(v,&mut m,len);cob_get_s64_comp5(&m,len)},
        2=>{cob_put_s64_comp3(v,&mut m,len);cob_get_s64_comp3(&m,len)},
        4=>{cob_put_s64_pic9(v,&mut m,len,false);cob_get_s64_pic9(&m,len,false)},
        _=>0 } } else { let u=v as u64; match var {
        0=>{cob_put_u64_compx(u,&mut m,len);cob_get_u64_compx(&m,len) as i64},
        1=>{cob_put_u64_comp5(u,&mut m,len);cob_get_u64_comp5(&m,len) as i64},
        2=>{cob_put_u64_comp3(u,&mut m,len);cob_get_u64_comp3(&m,len) as i64},
        3=>{cob_put_u64_comp6(u,&mut m,len);cob_get_u64_comp6(&m,len) as i64},
        4=>{cob_put_u64_pic9(u,&mut m,len);cob_get_u64_pic9(&m,len) as i64},
        _=>0 } };
    let hx:String=m.iter().map(|b|format!("{b:02x}")).collect();
    let _=writeln!(o,"{} {} {}",f[0],hx,got);
  }
}

//! Rust mirror of the cob_get_int/cob_get_llint oracle: reads the same field rows, calls the accessors,
//! prints "label <int> <llint>". Test infrastructure.
use gnucobol_rs::accessors::{cob_get_int, cob_get_llint};
use gnucobol_rs::FieldAttr;
use std::io::{self,BufRead,Write};
fn ph(s:&str,n:usize)->Option<Vec<u8>>{let b=s.as_bytes();if b.len()<n*2{return None}(0..n).map(|i|{let h=(b[2*i]as char).to_digit(16)?;let l=(b[2*i+1]as char).to_digit(16)?;Some(((h<<4)|l)as u8)}).collect()}
fn main(){let so=io::stdout();let mut o=so.lock();
  for line in io::stdin().lock().lines().map_while(Result::ok){
    let f:Vec<&str>=line.split_whitespace().collect(); if f.len()!=7{continue}
    let n=|i:usize|f[i].parse::<i64>().ok();
    let (Some(t),Some(dg),Some(sc),Some(fl),Some(sz))=(n(1),n(2),n(3),n(4),n(5)) else{continue};
    let Some(data)=ph(f[6],sz as usize) else{continue};
    let a=FieldAttr{field_type:t as u16,digits:dg as u16,scale:sc as i16,flags:fl as u16};
    let _=writeln!(o,"{} {} {}",f[0],cob_get_int(&data,&a),cob_get_llint(&data,&a));
  }
}

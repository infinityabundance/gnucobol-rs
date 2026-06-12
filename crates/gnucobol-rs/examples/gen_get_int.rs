//! Generator for the cob_get_int/cob_get_llint accessor differential (move.c). Emits DISPLAY / PACKED /
//! BINARY source fields with varied digits/scale/sign/value; identical bytes feed the oracle harness and
//! the Rust port. Row: `label type digits scale flags size hex`.
use gnucobol_rs::FieldAttr;
const DISP:u16=0x10; const PACK:u16=0x12; const BIN:u16=0x11;
const HS:u16=0x0001; const RB:u16=0x0040;
fn hex(b:&[u8])->String{b.iter().map(|x|format!("{x:02x}")).collect()}
fn disp_img(dig:usize,v:i64,signed:bool)->Vec<u8>{let neg=v<0;let mut s:Vec<u8>=format!("{:0w$}",v.unsigned_abs(),w=dig).into_bytes();s.truncate(dig);while s.len()<dig{s.insert(0,b'0');}if signed&&neg{let l=s.len()-1;s[l]|=0x40;}s}
fn main(){let mut id=0u64;
  let vals:&[i64]=&[0,1,7,42,-42,123,-123,9999,-9999,12345,-12345,99999,-99999];
  // DISPLAY
  for &(dg,sc,signed) in &[(5u16,0i16,true),(5,2,true),(7,3,true),(4,0,false),(6,2,true)]{
    let a=FieldAttr{field_type:DISP,digits:dg,scale:sc,flags:if signed{HS}else{0}};
    for &v in vals{ if !signed&&v<0{continue} if v.unsigned_abs()>=10u64.pow(dg as u32){continue}
      let img=disp_img(dg as usize,v,signed);
      println!("d{id} {DISP} {dg} {sc} {} {} {}",a.flags,img.len(),hex(&img)); id+=1; }
  }
  // PACKED (COMP-3)
  for &(dg,sc) in &[(5u16,0i16),(7,2),(9,0),(6,3)]{
    let a=FieldAttr{field_type:PACK,digits:dg,scale:sc,flags:HS};
    let len=dg as usize/2+1;
    for &v in vals{ if v.unsigned_abs()>=10u64.pow(dg as u32){continue}
      let mut img=vec![0u8;len]; gnucobol_rs::packed::cob_set_packed_int(&mut img,&a,v as i32);
      println!("p{id} {PACK} {dg} {sc} {} {} {}",HS,len,hex(&img)); id+=1; }
  }
  // BINARY COMP-5 (native little-endian, two's-complement)
  for &(dg,sc,bytes) in &[(9u16,0i16,4usize),(9,2,4),(18,0,8),(4,0,2)]{
    for &v in vals{ if v.unsigned_abs()>=10u64.pow(dg as u32){continue}
      let le=v.to_le_bytes(); let img=&le[..bytes];
      println!("b{id} {BIN} {dg} {sc} {} {} {}",HS|RB,bytes,hex(img)); id+=1; }
  }
}

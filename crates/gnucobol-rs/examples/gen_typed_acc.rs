//! Generator for the typed-accessor differential (move.c cob_put/get_<u64|s64>_<compx|comp5|comp3|
//! comp6|pic9>). Row: `label variant signed len value`. variant 0=compx 1=comp5 2=comp3 3=comp6 4=pic9.
fn main(){let mut id=0u64;
  let vals:&[i64]=&[0,1,7,42,-42,127,-128,255,1000,-1000,12345,-12345,65535,-65535];
  for variant in 0..=4u32 {
    for &signed in &[0u32,1] {
      if variant==3 && signed==1 { continue; } // comp6 unsigned-only
      let lens:&[usize] = if variant==2 {&[2,3,4,5,8]} else if variant==4 {&[4,6,8,10]} else {&[1,2,3,4,5,6,7,8]};
      for &len in lens {
        for &v in vals {
          if signed==0 && v<0 { continue; }
          // capacity guard
          let cap:i128 = match variant {
            2 => 10i128.pow((len*2-1) as u32),          // comp3 digits
            3 => 10i128.pow((len*2) as u32),            // comp6 digits
            4 => 10i128.pow((if signed==1 {len-0} else {len}) as u32), // pic9 digits (signed uses a digit slot)
            _ => if len>=8 {i128::MAX} else { if signed==1 {1i128<<(len*8-1)} else {1i128<<(len*8)} },
          };
          if (v.unsigned_abs() as i128) >= cap { continue; }
          println!("t{id} {variant} {signed} {len} {v}"); id+=1;
        }
      }
    }
  }
}

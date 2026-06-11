//! Generate OCCURS DEPENDING ON cases (`GNURUST.ODO.1`): `label|type|N|i|contenthex`. Structure: REC = N
//! PIC 9 (prefix 1) + E OCCURS 1 TO 5 DEPENDING ON N PIC X(3). type ∈ len (LENGTH OF REC) / elem (E(i)).
fn main() {
    let mut id = 0u32;
    for n in [1usize, 2, 3, 4, 5] {
        println!("o{id}|len|{n}||");
        id += 1;
    }
    // element access at N=5 over "5AAABBBCCCDDDEEE" (digit '5' prefix + 5 x 3-byte elems).
    let content = b"5AAABBBCCCDDDEEE";
    let h: String = content.iter().map(|b| format!("{b:02x}")).collect();
    for i in [1usize, 2, 3, 4, 5] {
        println!("o{id}|elem|5|{i}|{h}");
        id += 1;
    }
}

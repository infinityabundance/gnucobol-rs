//! Generate table-subscript cases (`GNURUST.SUBSCRIPT.1`): `label|shape|fieldhex|i|j`. shape 1d = E OCCURS
//! 5 PIC X(3); shape 2d = C OCCURS 3 of (OCCURS 4 PIC X(2)). All in-bounds (out-of-bounds is unit-tested).
fn main() {
    let one = b"ABCDEFGHIJKLMNO";
    let two = b"AABBCCDDEEFFGGHHIIJJKKLL";
    let h = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let (oh, th) = (h(one), h(two));
    let mut id = 0u32;
    for i in [1usize, 2, 3, 4, 5] {
        println!("s{id}|1d|{oh}|{i}|");
        id += 1;
    }
    for i in [1usize, 2, 3] {
        for j in [1usize, 2, 3, 4] {
            println!("s{id}|2d|{th}|{i}|{j}");
            id += 1;
        }
    }
}

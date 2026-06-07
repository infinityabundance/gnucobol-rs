//! Deterministic generator of ODO physical-max layout cases (`GNURUST.10`). Each case is a record
//! whose LAST item is `OCCURS min TO max TIMES DEPENDING ON <ctrl>` (elementary or group). Output:
//! a `#CASE <label>` line then `name<TAB>decl` item lines (the layout-harness format), blank-line
//! separated. The oracle compares the record's **physical** allocation (`b_REC[size]`). Test infra.

fn main() {
    let mut id = 0u64;
    let mut case = |lines: &[String]| {
        println!("#CASE o{id}");
        for l in lines {
            println!("{l}");
        }
        println!();
        id += 1;
    };

    let ctrl = "CNT\t05 CNT PIC 9(3)".to_string();
    // Elementary ODO of varying element PIC / usage / bounds.
    let elems: &[(&str, &str)] = &[
        ("X", ""),
        ("X(4)", ""),
        ("9(3)", ""),
        ("S9(5)V99", " USAGE COMP-3"),
        ("9(7)", " USAGE COMP-3"),
    ];
    for (pic, usage) in elems {
        for (min, max) in [(0u32, 5u32), (1, 4), (0, 10), (2, 3), (1, 2)] {
            let rec = vec![
                "REC\t01 REC".to_string(),
                ctrl.clone(),
                format!(
                    "ITEM\t05 ITEM PIC {pic}{usage} OCCURS {min} TO {max} TIMES DEPENDING ON CNT"
                ),
            ];
            case(&rec);
        }
    }

    // Group ODO: the last item is a group that OCCURS DEPENDING ON, with elementary children.
    for (min, max) in [(1u32, 4u32), (0, 6), (2, 8)] {
        let rec = vec![
            "REC\t01 REC".to_string(),
            "HDR\t05 HDR PIC X(2)".to_string(),
            ctrl.clone(),
            format!("GRP\t05 GRP OCCURS {min} TO {max} TIMES DEPENDING ON CNT"),
            "A\t10 A PIC 9(3)".to_string(),
            "B\t10 B PIC S9(4) USAGE COMP-3".to_string(),
            "C\t10 C PIC X(2)".to_string(),
        ];
        case(&rec);
    }

    // A record with several fixed fields before the ODO (offsets must still line up).
    for (min, max) in [(0u32, 5u32), (3, 7)] {
        let rec = vec![
            "REC\t01 REC".to_string(),
            "F1\t05 F1 PIC 9(4)".to_string(),
            "F2\t05 F2 PIC X(3)".to_string(),
            ctrl.clone(),
            format!(
                "ITEM\t05 ITEM PIC S9(3) USAGE COMP-3 OCCURS {min} TO {max} TIMES DEPENDING ON CNT"
            ),
        ];
        case(&rec);
    }
}

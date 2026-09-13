//! `key_tool info <key> [kind]` prints what the explorer derives from a pasted key;
//! `key_tool open <cm_hex> <envelope_json> <kind> <key>` opens one envelope with it.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = match a.get(1).map(String::as_str) {
        Some("info") => randscan_viewing::key_info(&a[2], a.get(3).map(String::as_str).unwrap_or("spend")),
        Some("open") => randscan_viewing::open_note(&a[2], &a[3], &a[4], &a[5]),
        _ => Err("usage: key_tool info <key> [kind] | open <cm> <envelope_json> <kind> <key>".into()),
    };
    match out { Ok(s) => println!("{s}"), Err(e) => { eprintln!("error: {e}"); std::process::exit(1) } }
}

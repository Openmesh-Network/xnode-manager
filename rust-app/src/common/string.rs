use std::fmt::Write;

/// Convert bytes to utf8 string lossless by escaping invalid bytes
/// Source: https://doc.rust-lang.org/std/primitive.slice.html#method.utf8_chunks
pub fn escaped_utf8_from_bytes(bytes: &[u8]) -> String {
    let mut repr = String::new();
    repr.push_str("c\"");
    for chunk in bytes.utf8_chunks() {
        for ch in chunk.valid().chars() {
            // Escapes \0, \t, \r, \n, \\, \', \", and uses \u{...} for non-printable characters.
            write!(repr, "{}", ch.escape_debug()).unwrap();
        }
        for byte in chunk.invalid() {
            write!(repr, "\\x{:02X}", byte).unwrap();
        }
    }
    repr.push('"');
    repr
}

pub fn between<'a>(str: &'a str, start: &'a str, end: &'a str) -> Option<&'a str> {
    if let Some(start_index) = str.find(start) {
        let start_end = start_index + start.len();
        if let Some(end_offset) = str[start_end..].find(end) {
            return Some(&str[start_end..(start_end + end_offset)]);
        }
    }

    None
}

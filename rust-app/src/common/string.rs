/// Convert bytes to utf8 string lossless by escaping invalid bytes
/// Source: https://doc.rust-lang.org/std/primitive.slice.html#method.utf8_chunks
pub fn escaped_utf8_from_bytes(bytes: impl AsRef<[u8]>) -> String {
    let bytes = bytes.as_ref();
    let mut repr = String::new();
    for chunk in bytes.utf8_chunks() {
        for ch in chunk.valid().chars() {
            match ch {
                '\\' => repr.push_str("\\\\"),
                _ => repr.push(ch),
            }
        }
        for byte in chunk.invalid() {
            repr.push_str(&format!("\\x{byte:02X}"));
        }
    }
    repr
}

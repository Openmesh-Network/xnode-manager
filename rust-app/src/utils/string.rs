pub fn between<'a>(str: &'a str, start: &'a str, end: &'a str) -> Option<&'a str> {
    if let Some(start_index) = str.find(start) {
        let start_end = start_index + start.len();
        if let Some(end_offset) = str[start_end..].find(end) {
            return Some(&str[start_end..(start_end + end_offset)]);
        }
    }

    None
}

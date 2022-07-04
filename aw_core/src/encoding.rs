pub fn latin1_to_string(s: &[u8]) -> String {
    s.iter()
        .map(|&c| c as char)
        .collect::<String>()
        .trim_end_matches('\0') // Strip off any null terminator
        .to_string()
}

pub fn string_to_latin1(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u8).collect()
}

pub(crate) fn printable_char(n: u8) -> Result<char, &'static str> {
    if n < 95 {
        Ok((n + 32) as char)
    } else {
        Err("Invalid base96 number")
    }
}

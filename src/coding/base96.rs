pub fn decode(ch: char) -> Result<u8, &'static str> {
    if matches!(ch, ' '..='~') {
        Ok(ch as u8 - 32)
    } else {
        Err("Invalid base96 char")
    }
}

pub fn encode(n: u8) -> Result<char, &'static str> {
    if n < 95 {
        Ok((n + 32) as char)
    } else {
        Err("Invalid base96 number")
    }
}

pub fn to_decimal(data: &[u8; 4]) -> u32 {
    data.iter()
        .enumerate()
        .map(|(i, &n)| n as u32 * 96_u32.pow(i as u32))
        .sum()
}

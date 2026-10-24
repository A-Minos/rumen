pub fn decode(ch: char) -> Result<u8, &'static str> {
    Ok(match ch {
        'A'..='Z' => ch as u8 - b'A',
        'a'..='z' => ch as u8 - b'a' + 26,
        '0'..='9' => ch as u8 - b'0' + 52,
        '+' => 62,
        '/' => 63,
        _ => return Err("Invalid base64 char"),
    })
}

pub fn encode(n: u8) -> Result<char, &'static str> {
    Ok(match n {
        0..=25 => (n + b'A') as char,
        26..=51 => (n - 26 + b'a') as char,
        52..=61 => (n - 52 + b'0') as char,
        62 => '+',
        63 => '/',
        _ => return Err("Invalid base64 number"),
    })
}

pub fn to_decimal(data: &[u8]) -> u32 {
    data.iter()
        .enumerate()
        .map(|(i, &n)| n as u32 * 64_u32.pow(i as u32))
        .sum()
}

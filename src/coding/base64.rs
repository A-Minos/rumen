const DECODE: [u8; 128] = {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut table = [u8::MAX; 128];
    let mut digit = 0;
    while digit < alphabet.len() {
        table[alphabet[digit] as usize] = digit as u8;
        digit += 1;
    }
    table
};

pub fn decode(ch: char) -> Result<u8, &'static str> {
    DECODE
        .get(ch as usize)
        .copied()
        .filter(|&digit| digit < 64)
        .ok_or("Invalid base64 char")
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

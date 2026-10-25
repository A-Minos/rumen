use std::char::DecodeUtf16Error;

/// Decodes JavaScript `unescape` syntax without decoding an escape twice.
/// Unpaired UTF-16 surrogates cannot be represented by a Rust `String`.
/// Decoded bytes never outgrow their escapes, so the input buffer is reused.
pub fn decode(input: String) -> Result<String, DecodeUtf16Error> {
    let Some(mut read) = input.find('%') else {
        return Ok(input);
    };
    let mut bytes = input.into_bytes();
    let mut write = read;
    // write <= read: replacements cannot overwrite bytes still to be decoded.
    while read < bytes.len() {
        if let Some((unit, mut consumed)) = escape(&bytes[read..]) {
            let character = if let Some(character) = char::from_u32(unit as u32) {
                character
            } else {
                let next = escape(&bytes[read + consumed..]);
                let units = std::iter::once(unit).chain(next.map(|(unit, _)| unit));
                let character = char::decode_utf16(units).next().unwrap()?;
                consumed += next.map_or(0, |(_, length)| length);
                character
            };
            let mut encoded = [0; 4];
            let encoded = character.encode_utf8(&mut encoded).as_bytes();
            bytes[write..write + encoded.len()].copy_from_slice(encoded);
            write += encoded.len();
            read += consumed;
        } else {
            bytes[write] = bytes[read];
            write += 1;
            read += 1;
        }
    }
    bytes.truncate(write);
    bytes.shrink_to_fit();
    // Raw UTF-8 bytes are unchanged; every replacement is an encoded scalar.
    Ok(String::from_utf8(bytes).expect("unescape preserves UTF-8"))
}

fn escape(input: &[u8]) -> Option<(u16, usize)> {
    let digits = input.strip_prefix(b"%")?;
    let (digits, length, consumed) = if let Some(digits) = digits.strip_prefix(b"u") {
        (digits, 4, 6)
    } else {
        (digits, 2, 3)
    };
    let mut value = 0;
    for &digit in digits.get(..length)? {
        value = value * 16 + (digit as char).to_digit(16)? as u16;
    }
    Some((value, consumed))
}

/// Encodes text using the JavaScript `escape` alphabet used by fumen comments.
pub fn encode(input: &str) -> String {
    input
        .encode_utf16()
        .map(|unit| {
            if unit <= 0x7f
                && ((unit as u8).is_ascii_alphanumeric()
                    || matches!(unit as u8, b'@' | b'*' | b'_' | b'+' | b'-' | b'.' | b'/'))
            {
                char::from(unit as u8).to_string()
            } else if unit <= 0xff {
                format!("%{unit:02X}")
            } else {
                format!("%u{unit:04X}")
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_ascii() {
        assert_eq!(decode("hello%20world".to_owned()).unwrap(), "hello world");
        assert_eq!(
            decode("%21%40%23%24%25%5E%26%2A%28%29".to_owned()).unwrap(),
            "!@#$%^&*()"
        );
    }

    #[test]
    fn test_decode_unicode() {
        assert_eq!(decode("%u4F60%u597D".to_owned()).unwrap(), "你好");
        assert_eq!(decode("%uD83D%uDE00".to_owned()).unwrap(), "\u{1f600}");
        assert_eq!(decode("%E9%FF".to_owned()).unwrap(), "éÿ");
        assert_eq!(decode("你好".to_owned()).unwrap(), "你好");
    }

    #[test]
    fn test_decode_mixed() {
        assert_eq!(
            decode("hello%20%u4F60%u597D%20world".to_owned()).unwrap(),
            "hello 你好 world"
        );
        assert_eq!(
            decode("%21%uD83D%uDE00%40".to_owned()).unwrap(),
            "!\u{1f600}@"
        );
        assert_eq!(
            decode("\u{1f600}%uD83d%ude00%41é".to_owned()).unwrap(),
            "\u{1f600}\u{1f600}Aé"
        );
    }

    #[test]
    fn test_encode_ascii() {
        assert_eq!(encode("hello world"), "hello%20world");
        assert_eq!(encode("!@#$%^&*()"), "%21@%23%24%25%5E%26*%28%29");
    }

    #[test]
    fn test_encode_unicode() {
        assert_eq!(encode("你好"), "%u4F60%u597D");
        assert_eq!(encode("\u{1f600}"), "%uD83D%uDE00");
        assert_eq!(encode("éÿ"), "%E9%FF");
    }

    #[test]
    fn test_encode_mixed() {
        assert_eq!(encode("hello 你好 world"), "hello%20%u4F60%u597D%20world");
        assert_eq!(encode("!\u{1f600}@"), "%21%uD83D%uDE00@");
    }

    #[test]
    fn test_decode_invalid() {
        for literal in ["%", "%u4F6", "%uZZZZ", "%xy", "%U0041", "%４１", "%éA"] {
            assert_eq!(decode(literal.to_owned()).unwrap(), literal);
        }
        for (input, expected) in [
            ("%u%41", "%uA"),
            ("%4%31", "%41"),
            ("%u12%34", "%u124"),
            ("%u\u{1f600}%41", "%u\u{1f600}A"),
        ] {
            assert_eq!(decode(input.to_owned()).unwrap(), expected);
        }
        for (surrogate, unpaired) in [
            ("%uD83D%uDE0", 0xD83D),
            ("%uD800", 0xD800),
            ("%uDC00", 0xDC00),
            ("%uD800\u{1f600}", 0xD800),
            ("\u{1f600}%uDC00", 0xDC00),
            ("%uD800%41%uDC00", 0xD800),
            ("%uD800%uD800", 0xD800),
        ] {
            assert_eq!(
                decode(surrogate.to_owned())
                    .unwrap_err()
                    .unpaired_surrogate(),
                unpaired,
                "{surrogate}"
            );
        }
    }

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode(""), "");
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode(String::new()).unwrap(), "");
    }

    #[test]
    fn escaped_percent_is_not_reinterpreted() {
        assert_eq!(decode("%25u4F60".to_owned()).unwrap(), "%u4F60");
        for text in [
            "%u4F60",
            "%20",
            "%25uD800",
            "a \t\n",
            "\0",
            "é你好\u{1f600}",
        ] {
            assert_eq!(decode(encode(text)).unwrap(), text);
        }
    }
}

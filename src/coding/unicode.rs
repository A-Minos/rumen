use std::string::FromUtf16Error;

/// Decodes JavaScript `unescape` syntax without decoding an escape twice.
/// Unpaired UTF-16 surrogates cannot be represented by a Rust `String`.
pub fn decode(input: &str) -> Result<String, FromUtf16Error> {
    let input: Vec<u16> = input.encode_utf16().collect();
    let mut output = Vec::with_capacity(input.len());
    let mut position = 0;
    while position < input.len() {
        if input[position] == b'%' as u16 {
            let (prefix, length) = if input.get(position + 1) == Some(&(b'u' as u16)) {
                (2, 4)
            } else {
                (1, 2)
            };
            if let Some(digits) = input.get(position + prefix..position + prefix + length) {
                let value = digits.iter().try_fold(0_u16, |value, &digit| {
                    char::from_u32(digit as u32)
                        .and_then(|digit| digit.to_digit(16))
                        .map(|digit| value * 16 + digit as u16)
                });
                if let Some(value) = value {
                    output.push(value);
                    position += prefix + length;
                    continue;
                }
            }
        }
        output.push(input[position]);
        position += 1;
    }
    String::from_utf16(&output)
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
        assert_eq!(decode("hello%20world").unwrap(), "hello world");
        assert_eq!(
            decode("%21%40%23%24%25%5E%26%2A%28%29").unwrap(),
            "!@#$%^&*()"
        );
    }

    #[test]
    fn test_decode_unicode() {
        assert_eq!(decode("%u4F60%u597D").unwrap(), "你好");
        assert_eq!(decode("%uD83D%uDE00").unwrap(), "\u{1f600}");
        assert_eq!(decode("%E9%FF").unwrap(), "éÿ");
        assert_eq!(decode("你好").unwrap(), "你好");
    }

    #[test]
    fn test_decode_mixed() {
        assert_eq!(
            decode("hello%20%u4F60%u597D%20world").unwrap(),
            "hello 你好 world"
        );
        assert_eq!(decode("%21%uD83D%uDE00%40").unwrap(), "!\u{1f600}@");
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
        for literal in ["%", "%u4F6", "%uZZZZ", "%xy"] {
            assert_eq!(decode(literal).unwrap(), literal);
        }
        for surrogate in ["%uD83D%uDE0", "%uD800", "%uDC00"] {
            assert!(decode(surrogate).is_err(), "{surrogate}");
        }
    }

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode(""), "");
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode("").unwrap(), "");
    }

    #[test]
    fn escaped_percent_is_not_reinterpreted() {
        assert_eq!(decode("%25u4F60").unwrap(), "%u4F60");
        for text in [
            "%u4F60",
            "%20",
            "%25uD800",
            "a \t\n",
            "\0",
            "é你好\u{1f600}",
        ] {
            assert_eq!(decode(&encode(text)).unwrap(), text);
        }
    }
}

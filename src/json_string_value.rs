use rust_extensions::StrOrString;

const DOUBLE_QUOTE: char = '"';
const BACK_SLASH: char = '\\';
const BACKSPACE: char = '\x08';
const TAB: char = '\x09';
const NEWLINE: char = '\x0A';
const FORM_FEED: char = '\x0C';
const CARRIAGE_RETURN: char = '\x0D';
const CONTROL_CHAR_THRESHOLD: u8 = 0x20;

fn has_to_escape(src: &[u8]) -> bool {
    for i in 0..src.len() {
        let byte = src[i];
        // Check for characters that need escaping: ", \, and control characters (0x00-0x1F)
        if byte == DOUBLE_QUOTE as u8 || byte == BACK_SLASH as u8 || byte < CONTROL_CHAR_THRESHOLD {
            return true;
        }
    }

    false
}

pub fn write_escaped_json_string_value(src: &str, out: &mut String) {
    for c in src.chars() {
        match c {
            DOUBLE_QUOTE => {
                out.push('\\');
                out.push('"');
            }
            BACK_SLASH => {
                out.push('\\');
                out.push('\\');
            }
            BACKSPACE => {
                // backspace
                out.push_str("\\b");
            }
            TAB => {
                // tab
                out.push_str("\\t");
            }
            NEWLINE => {
                // newline
                out.push_str("\\n");
            }
            FORM_FEED => {
                // form feed
                out.push_str("\\f");
            }
            CARRIAGE_RETURN => {
                // carriage return
                out.push_str("\\r");
            }
            c if (c as u32) < CONTROL_CHAR_THRESHOLD as u32 => {
                // Escape other control characters as \uXXXX
                let code = c as u32;
                out.push_str(&format!("\\u{:04x}", code));
            }
            _ => {
                out.push(c);
            }
        }
    }
}

pub fn escape_json_string_value<'s>(src: &'s str) -> StrOrString<'s> {
    if !has_to_escape(src.as_bytes()) {
        return src.into();
    }

    let mut result = String::new();

    for c in src.chars() {
        match c {
            DOUBLE_QUOTE => {
                result.push_str("\\\"");
            }
            BACK_SLASH => {
                result.push_str("\\\\");
            }
            BACKSPACE => {
                // backspace
                result.push_str("\\b");
            }
            TAB => {
                // tab
                result.push_str("\\t");
            }
            NEWLINE => {
                // newline
                result.push_str("\\n");
            }
            FORM_FEED => {
                // form feed
                result.push_str("\\f");
            }
            CARRIAGE_RETURN => {
                // carriage return
                result.push_str("\\r");
            }
            c if (c as u32) < CONTROL_CHAR_THRESHOLD as u32 => {
                // Escape other control characters as \uXXXX
                let code = c as u32;
                result.push_str(&format!("\\u{:04x}", code));
            }
            _ => {
                result.push(c);
            }
        }
    }

    result.into()
}

/// Reads exactly 4 hex digits from `chars` and returns their value, or `None` if fewer than 4
/// remain or any is not a hex digit (consuming what it read - the caller clones the iterator
/// first when it needs a non-destructive look-ahead).
fn parse_unicode_escape(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<u32> {
    let mut code: u32 = 0;
    for _ in 0..4 {
        let digit = chars.next()?.to_digit(16)?;
        code = code * 16 + digit;
    }
    Some(code)
}

pub fn de_escape_json_string_value<'s>(src: &'s str) -> StrOrString<'s> {
    if !has_to_escape(src.as_bytes()) {
        return src.into();
    }

    let mut result = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('b') => result.push(BACKSPACE), // backspace
                Some('f') => result.push(FORM_FEED), // form feed
                Some('n') => result.push(NEWLINE),   // newline
                Some('r') => result.push(CARRIAGE_RETURN), // carriage return
                Some('t') => result.push(TAB),       // tab
                Some('u') => {
                    // Handle \uXXXX unicode escape, including surrogate pairs
                    // (a high surrogate \uD800-\uDBFF followed by a low surrogate \uDC00-\uDFFF).
                    match parse_unicode_escape(&mut chars) {
                        Some(code) if (0xD800..=0xDBFF).contains(&code) => {
                            // High surrogate: try to combine with a following low surrogate.
                            let mut lookahead = chars.clone();
                            let combined =
                                if lookahead.next() == Some('\\') && lookahead.next() == Some('u') {
                                    match parse_unicode_escape(&mut lookahead) {
                                        Some(low) if (0xDC00..=0xDFFF).contains(&low) => Some(
                                            0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00),
                                        ),
                                        _ => None,
                                    }
                                } else {
                                    None
                                };

                            match combined {
                                Some(cp) => {
                                    // consume the low surrogate escape we peeked
                                    chars = lookahead;
                                    result.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                                }
                                // unpaired high surrogate -> replacement character
                                None => result.push('\u{FFFD}'),
                            }
                        }
                        // lone low surrogate -> replacement character
                        Some(code) if (0xDC00..=0xDFFF).contains(&code) => {
                            result.push('\u{FFFD}')
                        }
                        Some(code) => {
                            result.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        }
                        None => {
                            // Invalid / truncated \uXXXX escape - keep the marker literally.
                            result.push_str("\\u");
                        }
                    }
                }
                Some(other) => {
                    // Unknown escape sequence, push backslash and the character
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    // Backslash at end of string, push it
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }

    result.into()
}

#[cfg(test)]
mod test {

    #[test]
    pub fn test_basic_escape() {
        let src = "Test String \\ \"MyData\" '";

        let escaped = super::escape_json_string_value(src);

        assert_eq!("Test String \\\\ \\\"MyData\\\" '", escaped.as_str());

        let result = super::de_escape_json_string_value(escaped.as_str());

        assert_eq!(src, result.as_str());
    }

    #[test]
    pub fn test_escape_other_case() {
        let src = "Air Suspension Smoker's";

        let escaped = super::escape_json_string_value(src);

        let de_escaped = super::de_escape_json_string_value(escaped.as_str());

        assert_eq!("Air Suspension Smoker's", escaped.as_str());

        assert_eq!("Air Suspension Smoker's", de_escaped.as_str());
    }

    #[test]
    pub fn test_escape_control_characters() {
        // Test newline
        let src = "Line1\nLine2";
        let escaped = super::escape_json_string_value(src);
        assert_eq!("Line1\\nLine2", escaped.as_str());
        let de_escaped = super::de_escape_json_string_value(escaped.as_str());
        assert_eq!(src, de_escaped.as_str());

        // Test tab
        let src = "Column1\tColumn2";
        let escaped = super::escape_json_string_value(src);
        assert_eq!("Column1\\tColumn2", escaped.as_str());
        let de_escaped = super::de_escape_json_string_value(escaped.as_str());
        assert_eq!(src, de_escaped.as_str());

        // Test carriage return
        let src = "Line1\rLine2";
        let escaped = super::escape_json_string_value(src);
        assert_eq!("Line1\\rLine2", escaped.as_str());
        let de_escaped = super::de_escape_json_string_value(escaped.as_str());
        assert_eq!(src, de_escaped.as_str());

        // Test backspace
        let src = "Text\x08Back";
        let escaped = super::escape_json_string_value(src);
        assert_eq!("Text\\bBack", escaped.as_str());
        let de_escaped = super::de_escape_json_string_value(escaped.as_str());
        assert_eq!(src, de_escaped.as_str());

        // Test form feed
        let src = "Text\x0CFeed";
        let escaped = super::escape_json_string_value(src);
        assert_eq!("Text\\fFeed", escaped.as_str());
        let de_escaped = super::de_escape_json_string_value(escaped.as_str());
        assert_eq!(src, de_escaped.as_str());
    }

    #[test]
    pub fn test_escape_other_control_chars() {
        // Test null character (0x00)
        let src = "Text\x00Null";
        let escaped = super::escape_json_string_value(src);
        assert_eq!("Text\\u0000Null", escaped.as_str());
        let de_escaped = super::de_escape_json_string_value(escaped.as_str());
        assert_eq!(src, de_escaped.as_str());

        // Test bell character (0x07)
        let src = "Text\x07Bell";
        let escaped = super::escape_json_string_value(src);
        assert_eq!("Text\\u0007Bell", escaped.as_str());
        let de_escaped = super::de_escape_json_string_value(escaped.as_str());
        assert_eq!(src, de_escaped.as_str());
    }

    #[test]
    pub fn test_escape_multiple_control_chars() {
        let src = "Line1\nLine2\tTabbed\rReturned";
        let escaped = super::escape_json_string_value(src);
        assert_eq!("Line1\\nLine2\\tTabbed\\rReturned", escaped.as_str());
        let de_escaped = super::de_escape_json_string_value(escaped.as_str());
        assert_eq!(src, de_escaped.as_str());
    }

    #[test]
    pub fn test_unicode_escape_roundtrip() {
        // Test that unicode escapes are properly handled
        let src = "Text\\u0041Text"; // \u0041 is 'A'
        let de_escaped = super::de_escape_json_string_value(src);
        assert_eq!("TextAText", de_escaped.as_str());
    }

    #[test]
    pub fn test_surrogate_pair_is_decoded() {
        // the surrogate pair D83D DE00 decodes to U+1F600
        let src = "a\\uD83D\\uDE00b";
        let de_escaped = super::de_escape_json_string_value(src);
        assert_eq!("a\u{1F600}b", de_escaped.as_str());
    }

    #[test]
    pub fn test_basic_multilingual_plane_escape() {
        // 4F60 597D decodes to two BMP chars (no surrogates)
        let src = "\\u4F60\\u597D";
        let de_escaped = super::de_escape_json_string_value(src);
        assert_eq!("你好", de_escaped.as_str());
    }

    #[test]
    pub fn test_unpaired_high_surrogate_does_not_panic() {
        // A high surrogate not followed by a low surrogate becomes the replacement character.
        let src = "x\\uD83Dy";
        let de_escaped = super::de_escape_json_string_value(src);
        assert_eq!("x\u{FFFD}y", de_escaped.as_str());

        // Truncated \u escape (fewer than 4 hex digits) degrades to a literal "\u", no panic.
        let src = "x\\uD8";
        let de_escaped = super::de_escape_json_string_value(src);
        assert_eq!("x\\u", de_escaped.as_str());
    }

    #[test]
    pub fn test_all_simple_escapes() {
        let src = "\\n\\t\\r\\\"\\\\\\/\\b\\f";
        let de_escaped = super::de_escape_json_string_value(src);
        assert_eq!("\n\t\r\"\\/\u{08}\u{0C}", de_escaped.as_str());
    }
}

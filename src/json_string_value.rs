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
                    // Handle \uXXXX unicode escape
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        if let Some(hex_char) = chars.next() {
                            hex.push(hex_char);
                        } else {
                            // Invalid unicode escape, push as-is
                            result.push_str("\\u");
                            result.push_str(&hex);
                            break;
                        }
                    }
                    if hex.len() == 4 {
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(unicode_char) = char::from_u32(code) {
                                result.push(unicode_char);
                            } else {
                                // Invalid unicode code point, push as-is
                                result.push_str("\\u");
                                result.push_str(&hex);
                            }
                        } else {
                            // Invalid hex, push as-is
                            result.push_str("\\u");
                            result.push_str(&hex);
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
}

use rust_extensions::StrOrString;

const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';
const BACK_SLASH: char = '\\';

fn has_to_escape(src: &[u8]) -> bool {
    for i in 0..src.len() {
        if src[i] == '"' as u8 || src[i] == '\\' as u8 || src[i] == '\'' as u8 {
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
            SINGLE_QUOTE => {
                out.push('\\');
                out.push('\'');
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
            SINGLE_QUOTE => {
                result.push_str("\\\'");
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

    let mut escape_mode = false;
    for c in src.chars() {
        if escape_mode {
            result.push(c);
            escape_mode = false;
        } else {
            if c == '\\' {
                escape_mode = true;
            } else {
                result.push(c);
                escape_mode = false;
            }
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

        assert_eq!("Test String \\\\ \\\"MyData\\\" \\'", escaped.as_str());

        let result = super::de_escape_json_string_value(escaped.as_str());

        assert_eq!(src, result.as_str());
    }
}

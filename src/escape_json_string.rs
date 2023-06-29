const SINGLE_QUOTE: u8 = '\'' as u8;
const DOUBLE_QUOTE: u8 = '"' as u8;
const BACK_SLASH: u8 = '\\' as u8;

pub struct EscapedJsonString<'s> {
    as_slice: Option<&'s str>,
    as_string: Option<String>,
}

impl<'s> EscapedJsonString<'s> {
    pub fn new(src: &'s str) -> Self {
        return Self::from_bytes(src.as_bytes());
    }

    pub fn from_bytes(src: &'s [u8]) -> Self {
        if !has_to_escape(src) {
            return Self {
                as_slice: Some(std::str::from_utf8(src).unwrap()),
                as_string: None,
            };
        }

        let mut as_string = String::new();

        for b in src {
            match *b {
                DOUBLE_QUOTE => {
                    as_string.push_str("\\\"");
                }
                BACK_SLASH => {
                    as_string.push_str("\\\\");
                }
                SINGLE_QUOTE => {
                    as_string.push_str("\\\'");
                }
                _ => {
                    as_string.push(*b as char);
                }
            }
        }

        Self {
            as_slice: None,
            as_string: Some(as_string),
        }
    }

    pub fn as_str(&'s self) -> &'s str {
        if let Some(as_slice) = self.as_slice {
            return as_slice;
        }

        if let Some(as_string) = &self.as_string {
            return as_string;
        }

        panic!("EscapedJsonString is not initialized properly");
    }

    pub fn as_slice(&'s self) -> &'s [u8] {
        if let Some(as_slice) = self.as_slice {
            return as_slice.as_bytes();
        }

        if let Some(as_string) = &self.as_string {
            return as_string.as_bytes();
        }

        panic!("EscapedJsonString is not initialized properly");
    }
}

fn has_to_escape(src: &[u8]) -> bool {
    for i in 0..src.len() {
        if src[i] == '"' as u8 || src[i] == '\\' as u8 || src[i] == '\'' as u8 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod test {
    use crate::escape_json_string::*;

    #[test]
    pub fn test_basic_escape() {
        let src = "Test String \\ \"MyData\" '";

        let escaped = EscapedJsonString::new(src);

        println!("{}", escaped.as_str());

        assert_eq!("Test String \\\\ \\\"MyData\\\" \\'", escaped.as_str());

        assert_eq!(src.len(), escaped.as_str().len() - 4);
    }
}

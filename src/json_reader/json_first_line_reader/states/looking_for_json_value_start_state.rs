use rust_extensions::StringBuilder;

use rust_extensions::array_of_bytes_iterator::*;

use super::super::super::consts;
use super::super::super::JsonParseError;

pub struct LookingForJsonValueStartState;

impl LookingForJsonValueStartState {
    pub fn read_next(
        &self,
        src: &mut impl ArrayOfBytesIterator,
    ) -> Result<NextValue, JsonParseError> {
        let start_pos = src.get_pos();
        while let Some(next_value) = src.get_next() {
            if super::utils::is_space(next_value.value) {
                continue;
            }

            if is_open_value(next_value.value) {
                return Ok(next_value);
            } else {
                return Err(JsonParseError::new(format!(
                    "Invalid token '{}' is found at position {}. Expected token is {}",
                    next_value.value as char,
                    start_pos,
                    expected_token()
                )));
            }
        }

        return Err(JsonParseError::new(format!(
            "Could not find json token {} starting from position {}",
            expected_token(),
            start_pos
        )));
    }
}

const OPEN_VALUE_TOKEN: &'static [u8] = &[
    consts::OPEN_BRACKET,
    consts::OPEN_ARRAY,
    consts::DOUBLE_QUOTE,
    '0' as u8,
    '1' as u8,
    '2' as u8,
    '3' as u8,
    '4' as u8,
    '5' as u8,
    '6' as u8,
    '7' as u8,
    '8' as u8,
    '9' as u8,
    'T' as u8,
    't' as u8,
    'f' as u8,
    'F' as u8,
    'n' as u8,
    'N' as u8,
    '-' as u8,
];

fn is_open_value(c: u8) -> bool {
    return super::utils::is_start_of_digit(c)
        || super::utils::is_start_of_bool_or_null(c)
        || c == consts::DOUBLE_QUOTE
        || c == consts::OPEN_BRACKET
        || c == consts::OPEN_ARRAY;
}

fn expected_token() -> String {
    let mut sb = StringBuilder::new();

    for v in OPEN_VALUE_TOKEN {
        sb.append_byte(*v);
        sb.append_char(',');
    }

    return sb.to_string_utf8();
}

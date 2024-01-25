use rust_extensions::array_of_bytes_iterator::*;

use super::super::super::consts;
use super::super::super::JsonParseError;

pub struct ReadingNonStringValueState;

impl ReadingNonStringValueState {
    pub fn read_next(
        &self,
        src: &mut impl ArrayOfBytesIterator,
    ) -> Result<NextValue, JsonParseError> {
        let start_pos = src.get_pos();
        while let Some(next_value) = src.peek_value() {
            if is_non_string_value_char(next_value.value) {
                src.get_next();
                continue;
            }

            if next_value.value == consts::COMMA
                || super::utils::is_space(next_value.value)
                || next_value.value == consts::CLOSE_BRACKET
            {
                return Ok(next_value);
            }

            return Err(JsonParseError::new(format!(
                "Error reading non string value. Start {}, current pos {}",
                start_pos, next_value.pos
            )));
        }

        return Err(JsonParseError::new(format!(
            "Error reading non string value. Start {}. We reached the end of the payload",
            start_pos
        )));
    }
}

fn is_non_string_value_char(b: u8) -> bool {
    return super::utils::is_number(b) || super::utils::is_latin_letter(b) || b == '.' as u8;
}

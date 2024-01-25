use rust_extensions::array_of_bytes_iterator::*;

use super::super::super::consts;
use super::super::super::JsonParseError;

pub struct LookingForNextKeyStartState;

impl LookingForNextKeyStartState {
    pub fn read_next(
        &self,
        src: &mut impl ArrayOfBytesIterator,
    ) -> Result<Option<usize>, JsonParseError> {
        let start_pos = src.get_pos();

        while let Some(next_value) = src.get_next() {
            if super::utils::is_space(next_value.value) || next_value.value == consts::COMMA {
                continue;
            }

            if next_value.value == consts::DOUBLE_QUOTE {
                return Ok(Some(next_value.pos));
            }

            if next_value.value == consts::CLOSE_BRACKET {
                return Ok(None);
            }

            return Err(JsonParseError::new(format!(
                "We are expecting '\"' or '{}' sign. But we have {} at pos {}. Started looking from pos {}",
                consts::CLOSE_BRACKET,
                next_value.value as char,
                next_value.pos,start_pos
            )));
        }

        return Err(JsonParseError::new(format!(
            "We are expecting '\"' or '{}' sign. But we have not found it at all. Started looking from pos {}",
            consts::CLOSE_BRACKET,start_pos
        )));
    }
}

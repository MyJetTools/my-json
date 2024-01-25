use rust_extensions::array_of_bytes_iterator::*;

use super::super::super::byte_of_array_reader::FoundResult;
use super::super::super::JsonParseError;
pub struct LookingForJsonTokenState {
    pub token: u8,
}

impl LookingForJsonTokenState {
    pub fn new(token: u8) -> Self {
        Self { token }
    }
    pub fn read_next(
        &self,
        src: &mut impl ArrayOfBytesIterator,
    ) -> Result<NextValue, JsonParseError> {
        let start_pos = src.get_pos();
        let result = super::super::super::byte_of_array_reader::next_token_must_be(src, self.token);
        match result {
            FoundResult::Ok(pos) => Ok(pos),
            FoundResult::EndOfJson => Err(JsonParseError::new(format!(
                "We started looking for a token {} at pos {} and did not found",
                self.token, start_pos
            ))),
            FoundResult::InvalidTokenFound(next_value) => Err(JsonParseError::new(format!(
                "We started looking for a token {} at pos {} but we found a token '{}' at pos {}",
                self.token as char, start_pos, next_value.value as char, next_value.pos
            ))),
        }
    }
}

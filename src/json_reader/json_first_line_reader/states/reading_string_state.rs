use rust_extensions::array_of_bytes_iterator::*;

use super::super::super::JsonParseError;

pub struct ReadingStringState;

impl ReadingStringState {
    pub fn read_next(
        &self,
        src: &mut impl ArrayOfBytesIterator,
    ) -> Result<NextValue, JsonParseError> {
        super::super::super::byte_of_array_reader::find_the_end_of_the_string(src)
    }
}

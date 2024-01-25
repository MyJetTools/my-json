use rust_extensions::array_of_bytes_iterator::*;

use super::super::JsonParseError;

use super::states::{
    LookingForJsonTokenState, LookingForJsonValueStartState, LookingForNextKeyStartState,
    ReadingNonStringValueState, ReadingObjectValueState, ReadingStringState,
};

use super::super::consts;

pub enum ReadMode {
    LookingForOpenJson(LookingForJsonTokenState),
    LookingForJsonKeyStart(LookingForJsonTokenState),
    ReadingKey(ReadingStringState),
    LookingForKeyValueSeparator(LookingForJsonTokenState),
    LookingForValueStart(LookingForJsonValueStartState),
    ReadingStringValue(ReadingStringState),
    ReadingNonStringValue(ReadingNonStringValueState),
    ReadingObjectValue(ReadingObjectValueState),
    LookingForNextKeyStart(LookingForNextKeyStartState),
}

pub enum ReadResult {
    OpenJsonFound,
    KeyStartFound(usize),
    KeyEndFound(usize),
    KeyValueSeparatorFound,
    FoundStringValueStart(usize),
    FoundNonStringValueStart(usize),
    FoundObjectOrArrayValueStart(usize),
    ValueEndFound(usize),
    EndOfJson,
}

impl ReadMode {
    pub fn read_next(
        &self,
        src: &mut impl ArrayOfBytesIterator,
    ) -> Result<ReadResult, JsonParseError> {
        match self {
            ReadMode::LookingForOpenJson(state) => {
                state.read_next(src)?;
                Ok(ReadResult::OpenJsonFound)
            }
            ReadMode::LookingForJsonKeyStart(state) => {
                let next_value = state.read_next(src)?;
                Ok(ReadResult::KeyStartFound(next_value.pos))
            }
            ReadMode::ReadingKey(state) => {
                let next_value = state.read_next(src)?;
                Ok(ReadResult::KeyEndFound(next_value.pos))
            }
            ReadMode::LookingForKeyValueSeparator(state) => {
                state.read_next(src)?;
                Ok(ReadResult::KeyValueSeparatorFound)
            }
            ReadMode::LookingForValueStart(state) => {
                let next_value = state.read_next(src)?;

                match next_value.value {
                    consts::DOUBLE_QUOTE => Ok(ReadResult::FoundStringValueStart(next_value.pos)),
                    consts::OPEN_BRACKET => {
                        Ok(ReadResult::FoundObjectOrArrayValueStart(next_value.pos))
                    }
                    consts::OPEN_ARRAY => {
                        Ok(ReadResult::FoundObjectOrArrayValueStart(next_value.pos))
                    }
                    _ => Ok(ReadResult::FoundNonStringValueStart(next_value.pos)),
                }
            }
            ReadMode::ReadingStringValue(state) => {
                let next_value = state.read_next(src)?;
                return Ok(ReadResult::ValueEndFound(next_value.pos));
            }
            ReadMode::ReadingNonStringValue(state) => {
                let next_value = state.read_next(src)?;
                return Ok(ReadResult::ValueEndFound(next_value.pos - 1));
            }
            ReadMode::ReadingObjectValue(state) => {
                let next_value = state.read_next(src)?;
                return Ok(ReadResult::ValueEndFound(next_value.pos));
            }
            ReadMode::LookingForNextKeyStart(state) => {
                let pos = state.read_next(src)?;

                match pos {
                    Some(pos) => Ok(ReadResult::KeyStartFound(pos)),
                    None => Ok(ReadResult::EndOfJson),
                }
            }
        }
    }
}

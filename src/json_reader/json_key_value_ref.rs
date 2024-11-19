use crate::json_reader::JsonValueRef;

use super::{JsonContentOffset, JsonFieldNameRef, JsonValue};

pub struct JsonKeyValueRef<'s> {
    pub name: JsonFieldNameRef<'s>,
    pub value: JsonValueRef<'s>,
}

impl<'s> JsonKeyValueRef<'s> {
    pub fn new(name: JsonContentOffset, value: JsonValue, slice: &'s [u8]) -> Self {
        Self {
            name: JsonFieldNameRef::new(name, slice),
            value: JsonValueRef::new(value, slice),
        }
    }
}

use super::{super::JsonValue, JsonFieldName};

pub struct JsonKeyValue {
    pub name: JsonFieldName,
    pub value: JsonValue,
}

impl JsonKeyValue {
    pub fn new(key_start: usize, key_end: usize, value_start: usize, value_end: usize) -> Self {
        JsonKeyValue {
            name: JsonFieldName {
                start: key_start,
                end: key_end,
            },
            value: JsonValue {
                start: value_start,
                end: value_end,
            },
        }
    }
}

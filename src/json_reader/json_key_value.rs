use crate::json_reader::AsJsonSlice;

use super::{JsonFieldName, JsonKeyValueRef, JsonValue};

pub struct JsonKeyValue {
    pub name: JsonFieldName,
    pub value: JsonValue,
}

impl std::fmt::Debug for JsonKeyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonKeyValue")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
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

    pub fn as_ref<'s>(&self, json: &'s impl AsJsonSlice) -> JsonKeyValueRef<'s> {
        JsonKeyValueRef::new(self.name.clone(), self.value.clone(), json.as_slice())
    }
}

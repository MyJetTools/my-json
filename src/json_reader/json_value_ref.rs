use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

use super::{
    json_value::UnwrappedJsonValue, JsonArrayIterator, JsonFirstLineIterator, JsonParseError,
    JsonValue,
};

#[derive(Clone, Debug)]
pub struct JsonValueRef<'s> {
    pub data: JsonValue,
    pub json_slice: &'s [u8],
}

impl<'s> JsonValueRef<'s> {
    pub fn new(src: JsonValue, json_slice: &'s [u8]) -> Self {
        return Self {
            data: src,
            json_slice,
        };
    }

    pub fn unwrap_value(&'s self) -> Result<UnwrappedJsonValue<'s>, JsonParseError> {
        self.data.unwrap_value(&self.json_slice)
    }

    pub fn is_null(&'s self) -> bool {
        self.data.is_null(&self.json_slice)
    }

    pub fn unwrap_as_number(&'s self) -> Result<Option<i64>, JsonParseError> {
        self.data.unwrap_as_number(&self.json_slice)
    }

    pub fn is_object(&self) -> bool {
        self.data.is_object(&self.json_slice)
    }

    pub fn unwrap_as_object(&'s self) -> Result<JsonFirstLineIterator<'s>, JsonParseError> {
        self.data.unwrap_as_object(&self.json_slice)
    }

    pub fn unwrap_as_bool(&'s self) -> Option<bool> {
        self.data.unwrap_as_bool(&self.json_slice)
    }

    pub fn is_bool(&'s self) -> bool {
        self.data.is_bool(&self.json_slice)
    }

    pub fn is_string(&'s self) -> bool {
        self.data.is_string(&self.json_slice)
    }

    pub fn is_number(&'s self) -> bool {
        self.data.is_number(&self.json_slice)
    }

    pub fn is_double(&self) -> bool {
        self.data.is_double(&self.json_slice)
    }

    pub fn unwrap_as_double(&self) -> Result<Option<f64>, JsonParseError> {
        self.data.unwrap_as_double(&self.json_slice)
    }

    pub fn is_array(&'s self) -> bool {
        self.data.is_array(&self.json_slice)
    }

    pub fn as_bytes(&'s self) -> &'s [u8] {
        self.data.as_bytes(&self.json_slice)
    }

    pub fn unwrap_as_array(&'s self) -> Result<JsonArrayIterator<'s>, JsonParseError> {
        self.data.unwrap_as_array(&self.json_slice)
    }

    pub fn as_str(&'s self) -> Option<StrOrString<'s>> {
        self.data.as_str(&self.json_slice)
    }

    pub fn as_unescaped_str(&'s self) -> Option<&'s str> {
        self.data.as_unescaped_str(&self.json_slice)
    }

    pub fn as_raw_str(&'s self) -> Option<&'s str> {
        self.data.as_raw_str(&self.json_slice)
    }

    pub fn as_slice(&'s self) -> &'s [u8] {
        &self.json_slice[self.data.start..self.data.end]
    }

    pub fn as_date_time(&'s self) -> Option<DateTimeAsMicroseconds> {
        self.data.as_date_time(&self.json_slice)
    }
}

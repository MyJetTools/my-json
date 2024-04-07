use rust_extensions::{
    array_of_bytes_iterator::SliceIterator, date_time::DateTimeAsMicroseconds, StrOrString,
};

use super::{
    array_iterator::JsonArrayIterator, json_value::UnwrappedValue, JsonFirstLineReader,
    JsonParseError, JsonValue,
};

#[derive(Clone, Debug)]
pub struct JsonValueRef<'s> {
    pub src: JsonValue,
    pub json_slice: &'s [u8],
}

impl<'s> JsonValueRef<'s> {
    pub fn new(src: JsonValue, json_slice: &'s [u8]) -> Self {
        return Self { src, json_slice };
    }

    pub fn unwrap_value(&'s self) -> Result<UnwrappedValue<'s>, JsonParseError> {
        self.src.unwrap_value(&self.json_slice)
    }

    pub fn is_null(&'s self) -> bool {
        self.src.is_null(&self.json_slice)
    }

    pub fn unwrap_as_number(&'s self) -> Result<Option<i64>, JsonParseError> {
        self.src.unwrap_as_number(&self.json_slice)
    }

    pub fn is_object(&self) -> bool {
        self.src.is_object(&self.json_slice)
    }

    pub fn unwrap_as_object(
        &'s self,
    ) -> Result<JsonFirstLineReader<SliceIterator<'s>>, JsonParseError> {
        self.src.unwrap_as_object(&self.json_slice)
    }

    pub fn unwrap_as_bool(&'s self) -> Option<bool> {
        self.src.unwrap_as_bool(&self.json_slice)
    }

    pub fn is_bool(&'s self) -> bool {
        self.src.is_bool(&self.json_slice)
    }

    pub fn is_string(&'s self) -> bool {
        self.src.is_string(&self.json_slice)
    }

    pub fn is_number(&'s self) -> bool {
        self.src.is_number(&self.json_slice)
    }

    pub fn is_double(&self) -> bool {
        self.src.is_double(&self.json_slice)
    }

    pub fn unwrap_as_double(&self) -> Result<Option<f64>, JsonParseError> {
        self.src.unwrap_as_double(&self.json_slice)
    }

    pub fn is_array(&'s self) -> bool {
        self.src.is_array(&self.json_slice)
    }

    pub fn as_bytes(&'s self) -> &'s [u8] {
        self.src.as_bytes(&self.json_slice)
    }

    pub fn unwrap_as_array(
        &'s self,
    ) -> Result<JsonArrayIterator<SliceIterator<'s>>, JsonParseError> {
        self.src.unwrap_as_array(&self.json_slice)
    }

    pub fn as_str(&'s self) -> Option<StrOrString<'s>> {
        self.src.as_str(&self.json_slice)
    }

    pub fn as_unescaped_str(&'s self) -> Option<&'s str> {
        self.src.as_unescaped_str(&self.json_slice)
    }

    pub fn as_raw_str(&'s self) -> Option<&'s str> {
        self.src.as_raw_str(&self.json_slice)
    }

    pub fn as_date_time(&'s self) -> Option<DateTimeAsMicroseconds> {
        self.src.as_date_time(&self.json_slice)
    }
}

#[cfg(test)]
use std::str::Utf8Error;

use rust_extensions::StrOrString;

use crate::json_reader::json_value::AsJsonSlice;

use super::super::{JsonParseError, JsonValue};

pub struct JsonKeyValue {
    pub name_start: usize,
    pub name_end: usize,
    pub value_start: usize,
    pub value_end: usize,
}

impl JsonKeyValue {
    pub fn get_raw_name<'s>(&self, json: &'s impl AsJsonSlice) -> Result<&'s str, JsonParseError> {
        let name = json.as_slice(self.name_start, self.name_end);
        match std::str::from_utf8(name) {
            Ok(result) => Ok(result),
            Err(err) => Err(JsonParseError {
                msg: format!("Can not parse name: {:?}", err),
            }),
        }
    }

    pub fn get_name<'s>(
        &self,
        json: &'s impl AsJsonSlice,
    ) -> Result<StrOrString<'s>, JsonParseError> {
        let name = json.as_slice(self.name_start, self.name_end);

        if let Some(name) = crate::json_utils::try_get_string_value(name) {
            return Ok(name);
        }

        Err(JsonParseError {
            msg: format!("Can not parse name: {}-{}", self.name_start, self.name_end),
        })
    }

    pub fn get_unescaped_name<'s>(
        &self,
        json: &'s impl AsJsonSlice,
    ) -> Result<&'s str, JsonParseError> {
        let name = json.as_slice(self.name_start, self.name_end);

        if let Some(name) = std::str::from_utf8(name).ok() {
            return Ok(name);
        }

        Err(JsonParseError {
            msg: format!("Can not parse name: {}-{}", self.name_start, self.name_end),
        })
    }

    #[cfg(test)]
    pub fn get_raw_value<'s>(&self, json: &'s impl AsJsonSlice) -> Result<&'s str, Utf8Error> {
        let slice = json.as_slice(self.value_start, self.value_end);
        return std::str::from_utf8(slice);
    }

    pub fn get_value(&self) -> JsonValue {
        JsonValue::new(self.value_start, self.value_end)
    }
}

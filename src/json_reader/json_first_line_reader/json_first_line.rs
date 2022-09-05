#[cfg(test)]
use std::str::Utf8Error;

use super::super::consts;
use super::super::{JsonParseError, JsonValue};

pub struct JsonFirstLine<'t> {
    pub name_start: usize,
    pub name_end: usize,
    pub value_start: usize,
    pub value_end: usize,
    pub data: &'t [u8],
}

impl<'t> JsonFirstLine<'t> {
    #[cfg(test)]
    pub fn get_raw_name(&self) -> Result<&'t str, Utf8Error> {
        let name = &self.data[self.name_start..self.name_end];
        return std::str::from_utf8(name);
    }

    pub fn get_name(&self) -> Result<&'t str, JsonParseError> {
        let name = &self.data[self.name_start + 1..self.name_end - 1];

        if name.len() == 0 {
            return Err(JsonParseError::new(format!(
                "Invalid name len: {}",
                name.len()
            )));
        }

        let result = std::str::from_utf8(name);
        match result {
            Ok(str) => Ok(str),
            Err(err) => Err(JsonParseError::new(format!(
                "Can convert name to utf8 string. Err {}",
                err
            ))),
        }
    }

    #[cfg(test)]
    pub fn get_raw_value(&self) -> Result<&'t str, Utf8Error> {
        let value = &self.data[self.value_start..self.value_end];
        return std::str::from_utf8(value);
    }

    pub fn get_value(&self) -> Result<JsonValue<'t>, JsonParseError> {
        let value = &self.data[self.value_start..self.value_end];

        if crate::json_utils::is_null(value) {
            return Ok(JsonValue::Null);
        }

        if let Some(value) = crate::json_utils::is_bool(value) {
            return Ok(JsonValue::Boolean(value));
        }

        if crate::json_utils::is_number(value) {
            return Ok(JsonValue::Number(convert_to_utf8(value)?));
        }

        if value[0] == consts::OPEN_ARRAY {
            return Ok(JsonValue::Array(value));
        }

        if value[0] == consts::OPEN_BRACKET {
            return Ok(JsonValue::Object(value));
        }

        return Ok(JsonValue::String(convert_to_utf8(value)?));
    }

    /*
    pub fn get_value_as_date_time(&self) -> Option<DateTimeAsMicroseconds> {

    }
    */
}

fn convert_to_utf8(src: &[u8]) -> Result<&str, JsonParseError> {
    match std::str::from_utf8(src) {
        Ok(str) => Ok(str),
        Err(err) => Err(JsonParseError::new(format!(
            "Can convert value to utf8 string. Err {}",
            err
        ))),
    }
}

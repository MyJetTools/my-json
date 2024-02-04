#[cfg(test)]
use std::str::Utf8Error;

use super::super::{JsonParseError, JsonValue};

pub struct JsonFirstLine<'t> {
    pub name_start: usize,
    pub name_end: usize,
    pub value_start: usize,
    pub value_end: usize,
    pub data: &'t [u8],
}

impl<'t> JsonFirstLine<'t> {
    pub fn get_raw_name(&'t self) -> Result<&'t str, JsonParseError> {
        let name = &self.data[self.name_start..self.name_end];
        match std::str::from_utf8(name) {
            Ok(result) => Ok(result),
            Err(err) => Err(JsonParseError {
                msg: format!("Can not parse name: {:?}", err),
            }),
        }
    }

    pub fn get_name(&'t self) -> Result<&'t str, JsonParseError> {
        if self.name_end - self.name_start <= 2 {
            return Ok("");
        }
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
    pub fn get_raw_value(&'t self) -> Result<&'t str, Utf8Error> {
        let value = &self.data[self.value_start..self.value_end];
        return std::str::from_utf8(value);
    }

    pub fn get_value(&'t self) -> Result<JsonValue<'t>, JsonParseError> {
        let value = &self.data[self.value_start..self.value_end];
        JsonValue::new(value)
    }

    /*
    pub fn get_value_as_date_time(&self) -> Option<DateTimeAsMicroseconds> {

    }
    */
}

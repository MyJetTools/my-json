#[cfg(test)]
use std::str::Utf8Error;

use rust_extensions::array_of_bytes_iterator::SliceIterator;

use super::{
    super::{JsonParseError, JsonValue},
    JsonFirstLineReader,
};

pub struct JsonFirstLine {
    pub name_start: usize,
    pub name_end: usize,
    pub value_start: usize,
    pub value_end: usize,
}

impl JsonFirstLine {
    pub fn get_raw_name<'s>(
        &self,
        json_first_line_reader: &'s JsonFirstLineReader<SliceIterator>,
    ) -> Result<&'s str, JsonParseError> {
        let name = &json_first_line_reader.get_src_slice()[self.name_start..self.name_end];
        match std::str::from_utf8(name) {
            Ok(result) => Ok(result),
            Err(err) => Err(JsonParseError {
                msg: format!("Can not parse name: {:?}", err),
            }),
        }
    }

    pub fn get_name<'s>(
        &self,
        json_first_line_reader: &'s JsonFirstLineReader<SliceIterator>,
    ) -> Result<&'s str, JsonParseError> {
        if self.name_end - self.name_start <= 2 {
            return Ok("");
        }
        let name = &json_first_line_reader.get_src_slice()[self.name_start + 1..self.name_end - 1];

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
    pub fn get_raw_value<'s>(
        &self,
        json_first_line_reader: &'s JsonFirstLineReader<SliceIterator>,
    ) -> Result<&'s str, Utf8Error> {
        let value = &json_first_line_reader.get_src_slice()[self.value_start..self.value_end];
        return std::str::from_utf8(value);
    }

    pub fn get_value<'s>(
        &self,
        json_first_line_reader: &'s JsonFirstLineReader<SliceIterator>,
    ) -> Result<JsonValue<'s>, JsonParseError> {
        let value = &json_first_line_reader.get_src_slice()[self.value_start..self.value_end];
        JsonValue::new(value)
    }

    /*
    pub fn get_value_as_date_time(&self) -> Option<DateTimeAsMicroseconds> {

    }
    */
}

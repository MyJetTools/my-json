use rust_extensions::StrOrString;

use crate::json_reader::{json_value::AsJsonSlice, JsonParseError};

pub struct JsonFieldName {
    pub start: usize,
    pub end: usize,
}

impl JsonFieldName {
    pub fn as_raw_str<'s>(
        &self,
        json: &'s impl AsJsonSlice<'s>,
    ) -> Result<&'s str, JsonParseError> {
        let name = json.as_slice(self.start, self.end);
        match std::str::from_utf8(name) {
            Ok(result) => Ok(result),
            Err(err) => Err(JsonParseError {
                msg: format!("Can not parse name: {:?}", err),
            }),
        }
    }

    pub fn as_str<'s>(
        &self,
        json: &'s impl AsJsonSlice<'s>,
    ) -> Result<StrOrString<'s>, JsonParseError> {
        let name = json.as_slice(self.start, self.end);

        if let Some(name) = crate::json_utils::try_get_string_value(name) {
            return Ok(name);
        }

        Err(JsonParseError {
            msg: format!("Can not parse name: {}-{}", self.start, self.end),
        })
    }

    pub fn as_unescaped_name<'s>(
        &self,
        json: &'s impl AsJsonSlice<'s>,
    ) -> Result<&'s str, JsonParseError> {
        let name = json.as_slice(self.start, self.end);

        if let Some(name) = std::str::from_utf8(name).ok() {
            return Ok(name);
        }

        Err(JsonParseError {
            msg: format!("Can not parse name: {}-{}", self.start, self.end),
        })
    }
}

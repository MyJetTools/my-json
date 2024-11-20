use rust_extensions::StrOrString;

use crate::json_reader::{json_value::AsJsonSlice, JsonParseError};

use super::JsonFieldNameRef;

#[derive(Debug, Clone)]
pub struct JsonContentOffset {
    pub start: usize,
    pub end: usize,
}

impl JsonContentOffset {
    pub fn as_ref<'s>(&self, json: &'s impl AsJsonSlice) -> JsonFieldNameRef<'s> {
        JsonFieldNameRef::new(self.clone(), json.as_slice()[self.start..self.end].as_ref())
    }

    pub fn as_raw_str<'s>(&self, json: &'s impl AsJsonSlice) -> Result<&'s str, JsonParseError> {
        let slice = json.as_slice()[self.start..self.end].as_ref();

        match std::str::from_utf8(slice) {
            Ok(result) => Ok(result),
            Err(err) => Err(JsonParseError::new(format!(
                "Can not parse name: {:?}",
                err
            ))),
        }
    }

    pub fn as_str<'s>(
        &self,
        json: &'s impl AsJsonSlice,
    ) -> Result<StrOrString<'s>, JsonParseError> {
        let slice = json.as_slice()[self.start..self.end].as_ref();

        if let Some(name) = crate::json_utils::try_get_string_value(slice) {
            return Ok(name);
        }

        Err(JsonParseError::new(format!(
            "Can not parse name: {}-{}",
            self.start, self.end
        )))
    }

    pub fn as_unescaped_str<'s>(
        &self,
        json: &'s impl AsJsonSlice,
    ) -> Result<&'s str, JsonParseError> {
        let slice = json.as_slice()[self.start + 1..self.end - 1].as_ref();

        if let Some(name) = std::str::from_utf8(slice).ok() {
            return Ok(name);
        }

        Err(JsonParseError::new(format!(
            "Can not parse name: {}-{}",
            self.start, self.end
        )))
    }
}

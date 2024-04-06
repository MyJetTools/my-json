use rust_extensions::{
    array_of_bytes_iterator::SliceIterator, date_time::DateTimeAsMicroseconds, StrOrString,
};

use super::{array_iterator::JsonArrayIterator, JsonFirstLineReader, JsonParseError};

pub trait AsJsonSlice {
    fn as_slice(&self, start_index: usize, end_index: usize) -> &[u8];
}

pub struct JsonValue {
    start: usize,
    end: usize,
    /*
    String(&'s str),
    Number(&'s str),
    Double(&'s str),
    Boolean(bool),
    Array(&'s [u8]),
    Object(&'s [u8]),
     */
}

impl JsonValue {
    pub fn new(start: usize, end: usize) -> Self {
        return Self { start, end };
    }

    pub fn unwrap_value<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<UnwrappedValue<'s>, JsonParseError> {
        let slice = as_json_slice.as_slice(self.start, self.end);

        if crate::json_utils::is_null(slice) {
            return Ok(UnwrappedValue::Null);
        }

        if let Some(value) = crate::json_utils::as_bool_value(slice) {
            return Ok(UnwrappedValue::Boolean(value));
        }

        match crate::json_utils::is_number(slice) {
            crate::json_utils::NumberType::NaN => {}
            crate::json_utils::NumberType::Number => {
                return Ok(UnwrappedValue::Number(convert_to_i64(slice)?));
            }
            crate::json_utils::NumberType::Double => {
                return Ok(UnwrappedValue::Double(convert_to_f64(slice)?));
            }
        }

        if crate::json_utils::is_array(slice) {
            let slice_iterator = SliceIterator::new(slice);
            return Ok(UnwrappedValue::Array(JsonArrayIterator::new(
                slice_iterator,
            )));
        }

        if crate::json_utils::is_object(slice) {
            let slice_iterator = SliceIterator::new(slice);
            return Ok(UnwrappedValue::Object(JsonFirstLineReader::new(
                slice_iterator,
            )));
        }

        return Ok(UnwrappedValue::String(convert_to_utf8(slice)?));
    }

    /*
       pub fn new(value: &'s [u8]) -> Result<JsonValue<'s>, JsonParseError> {
           if crate::json_utils::is_null(value) {
               return Ok(JsonValue::Null);
           }

           if let Some(value) = crate::json_utils::is_bool(value) {
               return Ok(JsonValue::Boolean(value));
           }

           match crate::json_utils::is_number(value) {
               crate::json_utils::NumberType::NaN => {}
               crate::json_utils::NumberType::Number => {
                   return Ok(JsonValue::Number(convert_to_utf8(value)?));
               }
               crate::json_utils::NumberType::Double => {
                   return Ok(JsonValue::Double(convert_to_utf8(value)?));
               }
           }

           if value[0] == consts::OPEN_ARRAY {
               return Ok(JsonValue::Array(value));
           }

           if value[0] == consts::OPEN_BRACKET {
               return Ok(JsonValue::Object(value));
           }

           return Ok(JsonValue::String(convert_to_utf8(value)?));
       }
    */

    pub fn is_null(&self, as_json_slice: &impl AsJsonSlice) -> bool {
        let slice = as_json_slice.as_slice(self.start, self.end);
        crate::json_utils::is_null(slice)
    }

    pub fn unwrap_as_number(
        &self,
        as_json_slice: &impl AsJsonSlice,
    ) -> Result<Option<i64>, JsonParseError> {
        let slice = as_json_slice.as_slice(self.start, self.end);
        match crate::json_utils::is_number(slice) {
            crate::json_utils::NumberType::NaN => {}
            crate::json_utils::NumberType::Number => {
                return Ok(Some(convert_to_i64(slice)?));
            }
            crate::json_utils::NumberType::Double => {}
        }

        panic!("Json value is not a number")
    }

    /*
           pub fn get_value(&self) -> UnwrappedValue {
               match self {
                   JsonValue::Null => UnwrappedValue::Null,
                   JsonValue::String(value) => UnwrappedValue::String(value),
                   JsonValue::Number(value) => UnwrappedValue::Number(value.parse().unwrap()),
                   JsonValue::Double(value) => UnwrappedValue::Double(value.parse().unwrap()),
                   JsonValue::Boolean(value) => UnwrappedValue::Boolean(*value),
                   JsonValue::Array(slice) => {
                       let slice_iterator = SliceIterator::new(slice);
                       UnwrappedValue::Array(JsonArrayIterator::new(slice_iterator))
                   }
                   JsonValue::Object(slice) => {
                       let slice_iterator = SliceIterator::new(slice);
                       UnwrappedValue::Object(JsonFirstLineReader::new(slice_iterator))
                   }
               }
           }

           pub fn unwrap_as_double(&self) -> Option<f64> {
               match self {
                   JsonValue::Double(src) => {
                       let value = src.parse().unwrap();
                       Some(value)
                   }
                   JsonValue::Number(src) => {
                       let value = src.parse().unwrap();
                       Some(value)
                   }
                   JsonValue::Null => None,
                   _ => {
                       panic!("Json value is not double")
                   }
               }
           }

        pub fn unwrap_as_bool(&self) -> Option<bool> {
            match self {
                JsonValue::Boolean(src) => Some(*src),
                JsonValue::Null => None,
                _ => {
                    panic!("Json value is not boolean")
                }
            }
        }

        pub fn unwrap_as_array(&self) -> Option<JsonArrayIterator<SliceIterator>> {
            match self {
                JsonValue::Array(src) => Some(JsonArrayIterator::new(SliceIterator::new(src))),
                JsonValue::Null => None,
                _ => {
                    panic!("Json value is not array")
                }
            }
        }

        pub fn unwrap_as_object(&self) -> Option<JsonFirstLineReader<SliceIterator>> {
            match self {
                JsonValue::Object(src) => Some(JsonFirstLineReader::new(SliceIterator::new(src))),
                JsonValue::Null => None,
                _ => {
                    panic!("Json value is not array")
                }
            }
        }
    */
    pub fn is_object(&self, as_json_slice: &impl AsJsonSlice) -> bool {
        crate::json_utils::is_object(as_json_slice.as_slice(self.start, self.end))
    }

    pub fn unwrap_as_object<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<JsonFirstLineReader<SliceIterator<'s>>, JsonParseError> {
        let slice = as_json_slice.as_slice(self.start, self.end);

        if crate::json_utils::is_object(slice) {
            let slice_iterator = SliceIterator::new(slice);
            return Ok(JsonFirstLineReader::new(slice_iterator));
        }

        Err(JsonParseError::new(
            "Json value is not an object".to_string(),
        ))
    }

    pub fn unwrap_as_bool(&self, as_json_slice: &impl AsJsonSlice) -> Option<bool> {
        crate::json_utils::as_bool_value(as_json_slice.as_slice(self.start, self.end))
    }

    pub fn is_bool(&self, as_json_slice: &impl AsJsonSlice) -> bool {
        crate::json_utils::is_bool(as_json_slice.as_slice(self.start, self.end))
    }

    pub fn is_string(&self, as_json_slice: &impl AsJsonSlice) -> bool {
        crate::json_utils::is_string(as_json_slice.as_slice(self.start, self.end))
    }

    pub fn is_number(&self, as_json_slice: &impl AsJsonSlice) -> bool {
        match crate::json_utils::is_number(as_json_slice.as_slice(self.start, self.end)) {
            crate::json_utils::NumberType::NaN => {}
            crate::json_utils::NumberType::Number => return true,
            crate::json_utils::NumberType::Double => {}
        }
        false
    }

    pub fn is_double(&self, as_json_slice: &impl AsJsonSlice) -> bool {
        match crate::json_utils::is_number(as_json_slice.as_slice(self.start, self.end)) {
            crate::json_utils::NumberType::NaN => {}
            crate::json_utils::NumberType::Number => {}
            crate::json_utils::NumberType::Double => return true,
        }
        false
    }

    pub fn unwrap_as_double<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<Option<f64>, JsonParseError> {
        let slice = as_json_slice.as_slice(self.start, self.end);

        if crate::json_utils::is_null(slice) {
            return Ok(None);
        }

        match crate::json_utils::is_number(slice) {
            crate::json_utils::NumberType::NaN => {
                return Ok(None);
            }
            crate::json_utils::NumberType::Number => {
                let value = convert_to_f64(slice)?;
                return Ok(Some(value));
            }
            crate::json_utils::NumberType::Double => {
                let value = convert_to_f64(slice)?;
                return Ok(Some(value));
            }
        }
    }

    pub fn is_array(&self, as_json_slice: &impl AsJsonSlice) -> bool {
        crate::json_utils::is_array(as_json_slice.as_slice(self.start, self.end))
    }

    pub fn unwrap_as_array<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<JsonArrayIterator<SliceIterator<'s>>, JsonParseError> {
        let slice = as_json_slice.as_slice(self.start, self.end);

        if crate::json_utils::is_array(slice) {
            let slice_iterator = SliceIterator::new(slice);
            return Ok(JsonArrayIterator::new(slice_iterator));
        }

        Err(JsonParseError::new(
            "Json value is not an array".to_string(),
        ))
    }

    pub fn as_str<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> Option<StrOrString<'s>> {
        let json_slice = as_json_slice.as_slice(self.start, self.end);

        if let Some(value) = crate::json_utils::try_get_string_value(json_slice) {
            return Some(value);
        }

        if crate::json_utils::is_null(json_slice) {
            return None;
        }

        let result = std::str::from_utf8(json_slice).unwrap();

        Some(result.into())
    }

    pub fn as_raw_str<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> Option<&'s str> {
        let json_slice = as_json_slice.as_slice(self.start, self.end);

        if crate::json_utils::is_null(json_slice) {
            return None;
        }

        let result = std::str::from_utf8(json_slice).unwrap();

        Some(result.into())
    }

    pub fn as_date_time<'s>(
        &self,
        as_json_slice: &impl AsJsonSlice,
    ) -> Option<DateTimeAsMicroseconds> {
        let as_str = self.as_str(as_json_slice)?;
        DateTimeAsMicroseconds::from_str(as_str.as_str())
    }
}

pub enum UnwrappedValue<'s> {
    Null,
    String(&'s str),
    Number(i64),
    Double(f64),
    Boolean(bool),
    Array(JsonArrayIterator<SliceIterator<'s>>),
    Object(JsonFirstLineReader<SliceIterator<'s>>),
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

fn convert_to_i64(src: &[u8]) -> Result<i64, JsonParseError> {
    let src = convert_to_utf8(src)?;
    match src.parse() {
        Ok(value) => Ok(value),
        Err(err) => Err(JsonParseError::new(format!(
            "Can convert value to i64. Err {}",
            err
        ))),
    }
}

fn convert_to_f64(src: &[u8]) -> Result<f64, JsonParseError> {
    let src = convert_to_utf8(src)?;
    match src.parse() {
        Ok(value) => Ok(value),
        Err(err) => Err(JsonParseError::new(format!(
            "Can convert value to f64. Err {}",
            err
        ))),
    }
}

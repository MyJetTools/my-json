use rust_extensions::{array_of_bytes_iterator::SliceIterator, date_time::DateTimeAsMicroseconds};

use super::{array_iterator::JsonArrayIterator, consts, JsonFirstLineReader, JsonParseError};

pub enum JsonValue<'s> {
    Null,
    String(&'s str),
    Number(&'s str),
    Double(&'s str),
    Boolean(bool),
    Array(&'s [u8]),
    Object(&'s [u8]),
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

impl<'s> JsonValue<'s> {
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

    pub fn as_date_time(&self) -> Option<DateTimeAsMicroseconds> {
        DateTimeAsMicroseconds::from_str(self.as_str()?)
    }

    pub fn is_null(&self) -> bool {
        match self {
            JsonValue::Null => true,
            _ => false,
        }
    }

    pub fn unwrap_as_number(&self) -> Option<i64> {
        match self {
            JsonValue::Number(src) => {
                let value = src.parse().unwrap();
                Some(value)
            }
            JsonValue::Null => None,
            _ => {
                panic!("Json value is not number")
            }
        }
    }

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

    pub fn as_str(&self) -> Option<&'s str> {
        match self {
            JsonValue::Null => None,
            JsonValue::String(src) => Some(src[1..src.len() - 1].as_ref()),
            JsonValue::Number(src) => Some(src),
            JsonValue::Double(src) => Some(src),
            JsonValue::Boolean(src) => match src {
                true => Some("true"),
                false => Some("false"),
            },
            JsonValue::Array(array_object) => Some(std::str::from_utf8(array_object).unwrap()),
            JsonValue::Object(json_object) => Some(std::str::from_utf8(json_object).unwrap()),
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

    pub fn as_raw_str(&self) -> Option<&'s str> {
        let value = self.as_bytes()?;
        Some(std::str::from_utf8(value).unwrap())
    }
    pub fn as_bytes(&self) -> Option<&'s [u8]> {
        match self {
            JsonValue::Null => None,
            JsonValue::String(src) => Some(src.as_bytes()),
            JsonValue::Number(src) => Some(src.as_bytes()),
            JsonValue::Double(src) => Some(src.as_bytes()),
            JsonValue::Boolean(src) => match src {
                true => Some("true".as_bytes()),
                false => Some("false".as_bytes()),
            },
            JsonValue::Array(src) => Some(src),
            JsonValue::Object(src) => Some(src),
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            JsonValue::Object(_) => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            JsonValue::Boolean(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            JsonValue::String(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            JsonValue::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_double(&self) -> bool {
        match self {
            JsonValue::Double(_) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            JsonValue::Array(_) => true,
            _ => false,
        }
    }
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

use std::fmt::Debug;

use super::json_value::AsJsonSlice;
use super::JsonParseError;
use super::{bytes_of_array_reader::*, JsonValue};

use rust_extensions::{array_of_bytes_iterator::*, UnsafeValue};

pub struct JsonArrayIteratorInner<TArrayOfBytesIterator: ArrayOfBytesIterator> {
    data: TArrayOfBytesIterator,
    initialized: UnsafeValue<bool>,
}

impl Debug for JsonArrayIteratorInner<SliceIterator<'_>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonArrayIterator")
            .field("Position", &self.data.get_pos())
            .field("initialized", &self.initialized)
            .finish()
    }
}

impl<TArrayOfBytesIterator: ArrayOfBytesIterator> JsonArrayIteratorInner<TArrayOfBytesIterator> {
    pub fn new(mut data: TArrayOfBytesIterator) -> Result<Self, JsonParseError> {
        let result = sync_reader::skip_white_spaces(&mut data);

        match result {
            Ok(_) => Ok(Self {
                data,
                initialized: UnsafeValue::new(false),
            }),
            Err(result) => Err(JsonParseError::CanNotFineStartOfTheArrayObject(
                result.into_string(),
            )),
        }
    }

    pub fn get_src_slice(&self) -> &[u8] {
        self.data.get_src_slice()
    }

    fn init(&self) -> Result<(), JsonParseError> {
        let result = sync_reader::next_token_must_be(&self.data, crate::consts::OPEN_ARRAY);

        match result {
            FoundResult::Ok(_) => {
                self.initialized.set_value(true);
                return Ok(());
            }
            FoundResult::EndOfJson => {
                return Err(JsonParseError::new(format!(
                    "Can not find start of the array token"
                )));
            }
            FoundResult::InvalidTokenFound(value) => {
                return Err(JsonParseError::new(format!(
                    "We were looking start of array token but found '{}' at position {}",
                    value.value as char, value.pos
                )));
            }
        }
    }

    pub fn get_next(&self) -> Option<Result<JsonValue, JsonParseError>> {
        let start_value = if !self.initialized.get_value() {
            match self.init() {
                Ok(_) => match sync_reader::skip_white_spaces(&self.data) {
                    Ok(value) => value,
                    Err(err) => return Some(Err(err)),
                },
                Err(err) => return Some(Err(err)),
            }
        } else {
            let next_pos = match sync_reader::skip_white_spaces_and_get_next(&self.data) {
                Ok(value) => value,
                Err(err) => {
                    return Some(Err(JsonParseError::CanNotFineStartOfTheArrayObject(
                        err.into_string(),
                    )))
                }
            };

            match next_pos.value {
                crate::consts::CLOSE_ARRAY => {
                    return None;
                }
                crate::consts::COMMA => match sync_reader::skip_white_spaces(&self.data) {
                    Ok(value) => value,
                    Err(err) => return Some(Err(err)),
                },
                _ => {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found ['{}'] at position {}",
                        next_pos.value as char, next_pos.pos
                    ))));
                }
            }
        };

        match start_value.value {
            crate::consts::CLOSE_ARRAY => {
                return None;
            }
            crate::consts::DOUBLE_QUOTE => {
                match sync_reader::find_the_end_of_the_string(&self.data) {
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }

            crate::consts::OPEN_ARRAY => match sync_reader::find_the_end_of_array(&self.data) {
                Ok(_) => {}
                Err(err) => return Some(Err(err)),
            },

            crate::consts::OPEN_BRACKET => match sync_reader::find_the_end_of_json(&self.data) {
                Ok(_) => {}
                Err(err) => return Some(Err(err)),
            },
            crate::consts::START_OF_NULL_UPPER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&self.data, "null") {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_NULL_LOWER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&self.data, "null") {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_TRUE_UPPER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&self.data, "true") {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_TRUE_LOWER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&self.data, "true") {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_FALSE_UPPER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&self.data, "false") {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_FALSE_LOWER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&self.data, "false") {
                    return Some(Err(err));
                }
            }

            _ => {
                if sync_reader::is_number(start_value.value) {
                    match sync_reader::find_the_end_of_the_number(&self.data) {
                        Ok(_) => {}
                        Err(err) => return Some(Err(err)),
                    }
                } else {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found '{}' at position {}",
                        start_value.value as char, start_value.pos
                    ))));
                }
            }
        };

        //let result = self.data.get_slice_to_current_pos(start_value.pos);
        return Some(Ok(JsonValue::new(start_value.pos, self.data.get_pos())));
    }
}

impl<'s> TryInto<JsonArrayIteratorInner<SliceIterator<'s>>> for &'s [u8] {
    type Error = JsonParseError;
    fn try_into(self) -> Result<JsonArrayIteratorInner<SliceIterator<'s>>, Self::Error> {
        let slice_iterator = SliceIterator::new(self);
        JsonArrayIteratorInner::new(slice_iterator)
    }
}

impl<'s> TryInto<JsonArrayIteratorInner<SliceIterator<'s>>> for &'s str {
    type Error = JsonParseError;
    fn try_into(self) -> Result<JsonArrayIteratorInner<SliceIterator<'s>>, Self::Error> {
        let slice_iterator = SliceIterator::new(self.as_bytes());
        JsonArrayIteratorInner::new(slice_iterator)
    }
}

impl<TArrayOfBytesIterator: ArrayOfBytesIterator> AsJsonSlice
    for JsonArrayIteratorInner<TArrayOfBytesIterator>
{
    fn as_slice(&self) -> &[u8] {
        self.data.get_src_slice()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_basic_json_array_split() {
        let json = r###"[{"id":1},{"id":2},{"id":3}]"###;

        println!("{}", json);

        let mut i = 0;

        let slice_iterator = SliceIterator::from_str(json);

        let json_array_iterator = JsonArrayIteratorInner::new(slice_iterator).unwrap();

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();
            i += 1;
            println!("{}", i);

            assert_eq!(
                format!("{{\"id\":{}}}", i),
                sub_json.as_str(&json_array_iterator).unwrap().as_str()
            );

            assert!(sub_json.is_object(&json_array_iterator));
        }
    }

    #[test]
    pub fn test_basic_json_array_split_case_2() {
        let json = r###"[{"id":1} , {"id":2} , {"id":3}]"###;

        println!("{}", json);

        let mut i = 0;

        let slice_iterator = SliceIterator::from_str(json);

        let json_array_iterator = JsonArrayIteratorInner::new(slice_iterator).unwrap();

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();
            i += 1;

            assert_eq!(
                format!("{{\"id\":{}}}", i),
                sub_json.as_str(&json_array_iterator).unwrap().as_str()
            );

            assert!(sub_json.is_object(&json_array_iterator));
        }
    }

    #[test]
    pub fn test_basic_json_array_split_case_3() {
        let json = r###"[{"id":1}, {"id":2} ,{"id":3}]"###;

        println!("{}", json);

        let mut i = 0;

        let slice_iterator = SliceIterator::from_str(json);
        let json_array_iterator = JsonArrayIteratorInner::new(slice_iterator).unwrap();

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();
            i += 1;

            assert_eq!(
                format!("{{\"id\":{}}}", i),
                sub_json.as_str(&json_array_iterator).unwrap().as_str()
            );
        }
    }

    #[test]
    pub fn parse_empty_array() {
        let json = r###"[]"###;

        let mut i = 0;
        let slice_iterator = SliceIterator::from_str(json);
        let json_array_iterator = JsonArrayIteratorInner::new(slice_iterator).unwrap();

        while let Some(_) = json_array_iterator.get_next() {
            i += 1;
        }

        assert_eq!(0, i);
    }

    #[test]
    pub fn parse_array_with_different_objects() {
        let json = r###"["chat message",123,{"name":"chat"}, true, null]"###;

        let slice_iterator = SliceIterator::from_str(json);
        let json_array_iterator = JsonArrayIteratorInner::new(slice_iterator).unwrap();

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert_eq!(
            "\"chat message\"",
            value.as_raw_str(&json_array_iterator).unwrap()
        );
        assert_eq!(
            "chat message",
            value.as_str(&json_array_iterator).unwrap().as_str()
        );
        assert!(value.is_string(&json_array_iterator));

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert_eq!(
            123,
            value
                .unwrap_as_number(&json_array_iterator)
                .unwrap()
                .unwrap()
        );

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert!(value.is_object(&json_array_iterator));
        assert_eq!(
            "{\"name\":\"chat\"}",
            value.as_str(&json_array_iterator).unwrap().as_str()
        );

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert!(value.is_bool(&json_array_iterator));
        assert_eq!(true, value.unwrap_as_bool(&json_array_iterator).unwrap());

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert!(value.is_null(&json_array_iterator));

        let value = json_array_iterator.get_next();
        assert!(value.is_none());
    }

    #[test]
    pub fn parse_array_inside_array() {
        let json = "[[19313.0,2.7731]]";

        let slice_iterator = SliceIterator::from_str(json);
        let json_array_iterator = JsonArrayIteratorInner::new(slice_iterator).unwrap();

        let next_value = json_array_iterator.get_next().unwrap().unwrap();

        let sub_array = next_value.unwrap_as_array(&json_array_iterator).unwrap();

        let value = sub_array.get_next().unwrap().unwrap();
        assert_eq!(19313.0, value.unwrap_as_double().unwrap().unwrap());

        let value = sub_array.get_next().unwrap().unwrap();
        assert_eq!(2.7731, value.unwrap_as_double().unwrap().unwrap());
    }

    #[test]
    fn test_from_real_world() {
        let src = r#"[
            {
              "Id": "YourFin",
              "BaseDomain": "your_fin.tech",
              "DomainsPool": [
                "your_fin.tech"
              ],
              "CakeRegistrationId": "9",
              "TimeStamp": "2022-11-22T16:00:52.7472",
              "Expires": null
            }]"#
        .as_bytes();

        let slice_iterator = SliceIterator::new(src);
        let json_array_iterator = JsonArrayIteratorInner::new(slice_iterator).unwrap();

        let value = json_array_iterator.get_next().unwrap().unwrap();

        let object = value.unwrap_as_object(&json_array_iterator).unwrap();

        let (name, value) = object.get_next().unwrap().unwrap();
        assert_eq!("Id", name.as_str().unwrap().as_str());

        assert_eq!("YourFin", value.as_str().unwrap().as_str());

        let (name, value) = object.get_next().unwrap().unwrap();
        assert_eq!("BaseDomain", name.as_str().unwrap().as_str());

        assert_eq!("your_fin.tech", value.as_str().unwrap().as_str());

        let (name, value) = object.get_next().unwrap().unwrap();
        assert_eq!("DomainsPool", name.as_str().unwrap().as_str());

        assert_eq!(true, value.is_array());

        let (name, value) = object.get_next().unwrap().unwrap();
        assert_eq!("CakeRegistrationId", name.as_str().unwrap().as_str());

        assert_eq!("9", value.as_str().unwrap().as_str());

        let (name, value) = object.get_next().unwrap().unwrap();
        assert_eq!("TimeStamp", name.as_str().unwrap().as_str());

        println!("{}", value.as_date_time().unwrap().to_rfc3339());
    }

    #[test]
    fn test_from_file() {
        let src = std::fs::read_to_string("test.json").unwrap();

        let slice_iterator = SliceIterator::from_str(src.as_str());
        let json_array_iterator = JsonArrayIteratorInner::new(slice_iterator).unwrap();

        while let Some(itm) = json_array_iterator.get_next() {
            let itm = itm.unwrap();

            println!("-----");
            println!("{}", itm.as_str(&json_array_iterator).unwrap().as_str());
        }
    }
}

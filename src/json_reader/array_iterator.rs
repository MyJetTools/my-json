use super::{bytes_of_array_reader::*, JsonValue};
use super::{consts, JsonParseError};
pub use consts::*;
use rust_extensions::array_of_bytes_iterator::*;

pub struct JsonArrayIterator<TArrayOfBytesIterator: ArrayOfBytesIterator> {
    data: TArrayOfBytesIterator,
    initialized: bool,
}

impl<TArrayOfBytesIterator: ArrayOfBytesIterator> JsonArrayIterator<TArrayOfBytesIterator> {
    pub fn new(mut data: TArrayOfBytesIterator) -> Self {
        sync_reader::skip_white_spaces(&mut data).unwrap();
        Self {
            data,
            initialized: false,
        }
    }

    pub fn get_src_slice(&self) -> &[u8] {
        self.data.get_src_slice()
    }

    fn init(&mut self) -> Result<(), JsonParseError> {
        let result = sync_reader::next_token_must_be(&mut self.data, consts::OPEN_ARRAY);

        match result {
            FoundResult::Ok(_) => {
                self.initialized = true;
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

    pub fn get_next<'s>(&'s mut self) -> Option<Result<JsonValue<'s>, JsonParseError>> {
        let start_value = if !self.initialized {
            match self.init() {
                Ok(_) => match sync_reader::skip_white_spaces(&mut self.data) {
                    Ok(value) => value,
                    Err(err) => return Some(Err(err)),
                },
                Err(err) => return Some(Err(err)),
            }
        } else {
            let next_pos = match sync_reader::skip_white_spaces_and_get_next(&mut self.data) {
                Ok(value) => value,
                Err(err) => return Some(Err(err)),
            };

            match next_pos.value {
                consts::CLOSE_ARRAY => {
                    return None;
                }
                consts::COMMA => match sync_reader::skip_white_spaces(&mut self.data) {
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
            consts::CLOSE_ARRAY => {
                return None;
            }
            consts::DOUBLE_QUOTE => match sync_reader::find_the_end_of_the_string(&mut self.data) {
                Ok(_) => {}
                Err(err) => return Some(Err(err)),
            },

            consts::OPEN_ARRAY => match sync_reader::find_the_end_of_array(&mut self.data) {
                Ok(_) => {}
                Err(err) => return Some(Err(err)),
            },

            consts::OPEN_BRACKET => match sync_reader::find_the_end_of_json(&mut self.data) {
                Ok(_) => {}
                Err(err) => return Some(Err(err)),
            },
            consts::START_OF_NULL_UPPER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&mut self.data, "null") {
                    return Some(Err(err));
                }
            }

            consts::START_OF_NULL_LOWER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&mut self.data, "null") {
                    return Some(Err(err));
                }
            }

            consts::START_OF_TRUE_UPPER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&mut self.data, "true") {
                    return Some(Err(err));
                }
            }

            consts::START_OF_TRUE_LOWER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&mut self.data, "true") {
                    return Some(Err(err));
                }
            }

            consts::START_OF_FALSE_UPPER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&mut self.data, "false") {
                    return Some(Err(err));
                }
            }

            consts::START_OF_FALSE_LOWER_CASE => {
                if let Err(err) = sync_reader::check_json_symbol(&mut self.data, "false") {
                    return Some(Err(err));
                }
            }

            _ => {
                if sync_reader::is_number(start_value.value) {
                    match sync_reader::find_the_end_of_the_number(&mut self.data) {
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

        let result = self.data.get_slice_to_current_pos(start_value.pos);
        return Some(JsonValue::new(result));
    }
}

impl<'s> Into<JsonArrayIterator<SliceIterator<'s>>> for &'s [u8] {
    fn into(self) -> JsonArrayIterator<SliceIterator<'s>> {
        let slice_iterator = SliceIterator::new(self);
        JsonArrayIterator::new(slice_iterator)
    }
}

impl<'s> Into<JsonArrayIterator<SliceIterator<'s>>> for &'s str {
    fn into(self) -> JsonArrayIterator<SliceIterator<'s>> {
        let slice_iterator = SliceIterator::new(self.as_bytes());
        JsonArrayIterator::new(slice_iterator)
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

        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();
            i += 1;
            println!("{}", i);

            assert_eq!(
                format!("{{\"id\":{}}}", i),
                std::str::from_utf8(sub_json.as_bytes().unwrap()).unwrap()
            );

            assert!(sub_json.is_object());
        }
    }

    #[test]
    pub fn test_basic_json_array_split_case_2() {
        let json = r###"[{"id":1} , {"id":2} , {"id":3}]"###;

        println!("{}", json);

        let mut i = 0;

        let slice_iterator = SliceIterator::from_str(json);

        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();
            i += 1;

            assert_eq!(
                format!("{{\"id\":{}}}", i),
                std::str::from_utf8(sub_json.as_bytes().unwrap()).unwrap()
            );

            assert!(sub_json.is_object());
        }
    }

    #[test]
    pub fn test_basic_json_array_split_case_3() {
        let json = r###"[{"id":1}, {"id":2} ,{"id":3}]"###;

        println!("{}", json);

        let mut i = 0;

        let slice_iterator = SliceIterator::from_str(json);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();
            i += 1;

            assert_eq!(
                format!("{{\"id\":{}}}", i),
                std::str::from_utf8(sub_json.as_bytes().unwrap()).unwrap()
            );
        }
    }

    #[test]
    pub fn parse_empty_array() {
        let json = r###"[]"###;

        let mut i = 0;
        let slice_iterator = SliceIterator::from_str(json);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(_) = json_array_iterator.get_next() {
            i += 1;
        }

        assert_eq!(0, i);
    }

    #[test]
    pub fn parse_array_with_different_objects() {
        let json = r###"["chat message",123,{"name":"chat"}, true, null]"###;

        let slice_iterator = SliceIterator::from_str(json);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert_eq!(
            "\"chat message\"",
            std::str::from_utf8(value.as_bytes().unwrap()).unwrap()
        );
        assert_eq!("chat message", value.as_str().unwrap());
        assert!(value.is_string());

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert_eq!(123, value.unwrap_as_number().unwrap());

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert!(value.is_object());
        assert_eq!("{\"name\":\"chat\"}", value.as_raw_str().unwrap());

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert!(value.is_bool());
        assert_eq!(true, value.unwrap_as_bool().unwrap());

        let value = json_array_iterator.get_next().unwrap().unwrap();
        assert!(value.is_null());

        let value = json_array_iterator.get_next();
        assert!(value.is_none());
    }

    #[test]
    pub fn parse_array_inside_array() {
        let json = "[[19313.0,2.7731]]";

        let slice_iterator = SliceIterator::from_str(json);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        let next_value = json_array_iterator.get_next().unwrap().unwrap();

        let mut sub_array = next_value.unwrap_as_array().unwrap();

        let value = sub_array.get_next().unwrap().unwrap();
        assert_eq!(19313.0, value.unwrap_as_double().unwrap());

        let value = sub_array.get_next().unwrap().unwrap();
        assert_eq!(2.7731, value.unwrap_as_double().unwrap());
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
            }]"#;

        let slice_iterator = SliceIterator::from_str(src);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        let value = json_array_iterator.get_next().unwrap().unwrap();

        let mut object = value.unwrap_as_object().unwrap();

        let param = object.get_next().unwrap().unwrap();
        assert_eq!("Id", param.get_name().unwrap());

        let value = param.get_value().unwrap();
        assert_eq!("YourFin", value.as_str().unwrap());

        let param = object.get_next().unwrap().unwrap();
        assert_eq!("BaseDomain", param.get_name().unwrap());

        let value = param.get_value().unwrap();
        assert_eq!("your_fin.tech", value.as_str().unwrap());

        let param = object.get_next().unwrap().unwrap();
        assert_eq!("DomainsPool", param.get_name().unwrap());

        let value = param.get_value().unwrap();
        assert_eq!(true, value.is_array());

        let param = object.get_next().unwrap().unwrap();
        assert_eq!("CakeRegistrationId", param.get_name().unwrap());

        let value = param.get_value().unwrap();
        assert_eq!("9", value.as_str().unwrap());

        let param = object.get_next().unwrap().unwrap();
        assert_eq!("TimeStamp", param.get_name().unwrap());

        let value = param.get_value().unwrap();
        println!("{}", value.as_date_time().unwrap().to_rfc3339());
    }

    #[test]
    fn test_from_file() {
        let src = std::fs::read_to_string("test.json").unwrap();

        let slice_iterator = SliceIterator::from_str(src.as_str());
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(itm) = json_array_iterator.get_next() {
            let itm = itm.unwrap();

            println!("-----");
            println!("{}", itm.as_raw_str().unwrap());
        }
    }
}

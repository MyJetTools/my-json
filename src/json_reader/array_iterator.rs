use super::{byte_of_array_reader::FoundResult, consts, JsonParseError};
pub use consts::*;
use rust_extensions::array_of_bytes_iterator::*;

pub struct JsonArrayIterator<TArrayOfBytesIterator: ArrayOfBytesIterator> {
    data: TArrayOfBytesIterator,
    initialized: bool,
}

impl<TArrayOfBytesIterator: ArrayOfBytesIterator> JsonArrayIterator<TArrayOfBytesIterator> {
    pub fn new(mut data: TArrayOfBytesIterator) -> Self {
        super::byte_of_array_reader::skip_white_spaces(&mut data).unwrap();
        Self {
            data,
            initialized: false,
        }
    }

    fn init(&mut self) -> Result<(), JsonParseError> {
        let result =
            super::byte_of_array_reader::next_token_must_be(&mut self.data, consts::OPEN_ARRAY);

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

    pub fn get_next<'s>(&'s mut self) -> Option<Result<&'s [u8], JsonParseError>> {
        let start_value = if !self.initialized {
            match self.init() {
                Ok(_) => match super::byte_of_array_reader::skip_white_spaces(&mut self.data) {
                    Ok(value) => value,
                    Err(err) => return Some(Err(err)),
                },
                Err(err) => return Some(Err(err)),
            }
        } else {
            let next_pos = match super::byte_of_array_reader::skip_white_spaces_and_extract_value(
                &mut self.data,
            ) {
                Ok(value) => value,
                Err(err) => return Some(Err(err)),
            };

            match next_pos.value {
                consts::CLOSE_ARRAY => {
                    return None;
                }
                consts::COMMA => {
                    match super::byte_of_array_reader::skip_white_spaces(&mut self.data) {
                        Ok(value) => value,
                        Err(err) => return Some(Err(err)),
                    }
                }
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
            consts::DOUBLE_QUOTE => {
                match super::byte_of_array_reader::find_the_end_of_the_string(&mut self.data) {
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }

            consts::OPEN_ARRAY => {
                match super::byte_of_array_reader::find_the_end_of_json_object_or_array(
                    &mut self.data,
                ) {
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }

            consts::OPEN_BRACKET => {
                match super::byte_of_array_reader::find_the_end_of_json_object_or_array(
                    &mut self.data,
                ) {
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }
            consts::START_OF_NULL_UPPER_CASE => {
                if let Err(err) =
                    super::byte_of_array_reader::check_json_symbol(&mut self.data, "null")
                {
                    return Some(Err(err));
                }
            }

            consts::START_OF_NULL_LOWER_CASE => {
                if let Err(err) =
                    super::byte_of_array_reader::check_json_symbol(&mut self.data, "null")
                {
                    return Some(Err(err));
                }
            }

            consts::START_OF_TRUE_UPPER_CASE => {
                if let Err(err) =
                    super::byte_of_array_reader::check_json_symbol(&mut self.data, "true")
                {
                    return Some(Err(err));
                }
            }

            consts::START_OF_TRUE_LOWER_CASE => {
                if let Err(err) =
                    super::byte_of_array_reader::check_json_symbol(&mut self.data, "true")
                {
                    return Some(Err(err));
                }
            }

            consts::START_OF_FALSE_UPPER_CASE => {
                if let Err(err) =
                    super::byte_of_array_reader::check_json_symbol(&mut self.data, "false")
                {
                    return Some(Err(err));
                }
            }

            consts::START_OF_FALSE_LOWER_CASE => {
                if let Err(err) =
                    super::byte_of_array_reader::check_json_symbol(&mut self.data, "false")
                {
                    return Some(Err(err));
                }
            }

            _ => {
                if super::byte_of_array_reader::is_number(start_value.value) {
                    match super::byte_of_array_reader::find_the_end_of_the_number(&mut self.data) {
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
        return Some(Ok(result));
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
            println!("{}", std::str::from_utf8(sub_json).unwrap());

            assert_eq!(
                format!("{{\"id\":{}}}", i),
                std::str::from_utf8(sub_json).unwrap()
            );
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
                std::str::from_utf8(sub_json).unwrap()
            );
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
                std::str::from_utf8(sub_json).unwrap()
            );
        }
    }

    #[test]
    pub fn parse_empty_array() {
        let json = r###"[]"###;

        let mut i = 0;
        let slice_iterator = SliceIterator::from_str(json);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();

            println!("{}", sub_json.len());
            i += 1;
        }

        assert_eq!(0, i);
    }

    #[test]
    pub fn parse_array_with_different_objects() {
        let json = r###"["chat message",123,{"name":"chat"}, true, null]"###;

        let mut result = Vec::new();

        let slice_iterator = SliceIterator::from_str(json);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();

            let value = String::from_utf8(sub_json.to_vec()).unwrap();
            println!("{:?}", value);
            result.push(value);
        }

        assert_eq!("\"chat message\"", result.get(0).unwrap());
        assert_eq!("123", result.get(1).unwrap());

        assert_eq!("{\"name\":\"chat\"}", result.get(2).unwrap());
        assert_eq!("true", result.get(3).unwrap());
        assert_eq!("null", result.get(4).unwrap());
    }

    #[test]
    pub fn parse_array_inside_array() {
        let json = "[[19313.0,2.7731]]";

        let slice_iterator = SliceIterator::from_str(json);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(itm) = json_array_iterator.get_next() {
            let itm = itm.unwrap();
            let mut i = 0;

            let slice_iterator = SliceIterator::new(itm);
            let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

            while let Some(itm) = json_array_iterator.get_next() {
                let itm = itm.unwrap();

                let result = std::str::from_utf8(itm).unwrap();

                if i == 0 {
                    assert_eq!("19313.0", result)
                } else if i == 1 {
                    assert_eq!("2.7731", result)
                } else {
                    panic!("Invalid index");
                }

                i += 1;
            }
        }
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
              "CakeFtdId": "36",
              "CakeDepositId": "37",
              "CakeTradeId": "38",
              "CakeStatusId": "35",
              "PartitionKey": "brand",
              "RowKey": "YourFin",
              "TimeStamp": "2022-11-22T16:00:52.7472",
              "Expires": null
            }]"#;

        let slice_iterator = SliceIterator::from_str(src);
        let mut json_array_iterator = JsonArrayIterator::new(slice_iterator);

        while let Some(object) = json_array_iterator.get_next() {
            let object = object.unwrap();
            let object = std::str::from_utf8(object).unwrap();
            println!("{}", object);
        }
    }
}

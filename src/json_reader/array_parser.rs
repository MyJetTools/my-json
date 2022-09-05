pub use consts::*;

use super::{consts, read_json_object::FoundResult, JsonParseError};

pub struct JsonArrayIterator<'t> {
    data: &'t [u8],
    pos: usize,
}

impl<'t> JsonArrayIterator<'t> {
    pub fn new(data: &'t [u8]) -> Self {
        let result = Self { data, pos: 0 };

        result
    }

    fn init(&mut self) -> Result<(), JsonParseError> {
        if self.pos > 0 {
            return Ok(());
        }

        let result = super::read_json_object::next_token_must_be(self.data, 0, consts::OPEN_ARRAY);

        match result {
            FoundResult::Ok(pos) => {
                self.pos = pos;
                return Ok(());
            }
            FoundResult::EndOfJson => {
                return Err(JsonParseError::new(format!(
                    "Can not find start of the array token"
                )));
            }
            FoundResult::InvalidTokenFound { found_token, pos } => {
                return Err(JsonParseError::new(format!(
                    "We were looking start of array token but found '{}' at position {}",
                    found_token as char, pos
                )));
            }
        }
    }
}

impl<'t> Iterator for JsonArrayIterator<'t> {
    type Item = Result<&'t [u8], JsonParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == 0 {
            if let Err(err) = self.init() {
                return Some(Err(err));
            }
        } else {
            self.pos = match super::read_json_object::skip_whitespaces(self.data, self.pos + 1) {
                Ok(value) => value,
                Err(err) => return Some(Err(err)),
            };

            let b = self.data[self.pos];

            match b {
                consts::CLOSE_ARRAY => {
                    return None;
                }
                consts::COMMA => {}
                _ => {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found ['{}'] at position {}",
                        b as char, self.pos
                    ))));
                }
            }
        }

        let start_pos = match super::read_json_object::skip_whitespaces(self.data, self.pos + 1) {
            Ok(value) => value,
            Err(err) => return Some(Err(err)),
        };
        let b = self.data[start_pos];

        let end_of_item = match b {
            consts::CLOSE_ARRAY => {
                return None;
            }
            consts::DOUBLE_QUOTE => {
                match super::read_json_object::find_the_end_of_the_string(self.data, start_pos + 1)
                {
                    Ok(value) => value,
                    Err(err) => return Some(Err(err)),
                }
            }

            consts::OPEN_BRACKET => {
                match super::read_json_object::read_json_object(self.data, start_pos) {
                    Ok(value) => value,
                    Err(err) => return Some(Err(err)),
                }
            }
            consts::START_OF_NULL_UPPER_CASE => {
                let end = start_pos + 3;
                if end >= self.data.len() {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found ['{}'] at position {}",
                        b as char, start_pos
                    ))));
                }

                end
            }

            consts::START_OF_NULL_LOWER_CASE => {
                let end = start_pos + 3;
                if end >= self.data.len() {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found ['{}'] at position {}",
                        b as char, start_pos
                    ))));
                }

                end
            }

            consts::START_OF_TRUE_UPPER_CASE => {
                let end = start_pos + 3;
                if end >= self.data.len() {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found ['{}'] at position {}",
                        b as char, start_pos
                    ))));
                }

                end
            }

            consts::START_OF_TRUE_LOWER_CASE => {
                let end = start_pos + 3;
                if end >= self.data.len() {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found ['{}'] at position {}",
                        b as char, start_pos
                    ))));
                }

                end
            }

            consts::START_OF_FALSE_UPPER_CASE => {
                let end = start_pos + 4;
                if end >= self.data.len() {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found ['{}'] at position {}",
                        b as char, start_pos
                    ))));
                }

                end
            }

            consts::START_OF_FALSE_LOWER_CASE => {
                let end = start_pos + 4;
                if end >= self.data.len() {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found ['{}'] at position {}",
                        b as char, start_pos
                    ))));
                }

                end
            }

            _ => {
                if super::read_json_object::is_number(b) {
                    match super::read_json_object::find_the_end_of_the_number(self.data, start_pos)
                    {
                        Ok(value) => value,
                        Err(err) => return Some(Err(err)),
                    }
                } else {
                    return Some(Err(JsonParseError::new(format!(
                        "Invalid token found '{}' at position {}",
                        b as char, start_pos
                    ))));
                }
            }
        };

        self.pos = end_of_item;
        let result = &self.data[start_pos..end_of_item + 1];
        return Some(Ok(result));
    }
}

pub trait ArrayToJsonObjectsSplitter<'t> {
    fn split_array_json_to_objects(self) -> JsonArrayIterator<'t>;
}

impl<'t> ArrayToJsonObjectsSplitter<'t> for &'t [u8] {
    fn split_array_json_to_objects(self) -> JsonArrayIterator<'t> {
        return JsonArrayIterator::new(self);
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
        for sub_json in json.as_bytes().split_array_json_to_objects() {
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
        for sub_json in json.as_bytes().split_array_json_to_objects() {
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
        for sub_json in json.as_bytes().split_array_json_to_objects() {
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
        for sub_json in json.as_bytes().split_array_json_to_objects() {
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

        for sub_json in json.as_bytes().split_array_json_to_objects() {
            let sub_json = sub_json.unwrap();

            result.push(String::from_utf8(sub_json.to_vec()).unwrap());
        }

        assert_eq!("\"chat message\"", result.get(0).unwrap());
        assert_eq!("123", result.get(1).unwrap());

        assert_eq!("{\"name\":\"chat\"}", result.get(2).unwrap());
        assert_eq!("true", result.get(3).unwrap());
        assert_eq!("null", result.get(4).unwrap());
    }
}

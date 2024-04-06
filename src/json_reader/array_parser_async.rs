use super::bytes_of_array_reader::*;
use super::JsonParseError;

use rust_extensions::array_of_bytes_iterator::*;

pub struct JsonArrayIteratorAsync<TArrayOfBytesIterator: ArrayOfBytesIteratorAsync> {
    data: TArrayOfBytesIterator,
    initialized: bool,
}

impl<TArrayOfBytesIterator: ArrayOfBytesIteratorAsync>
    JsonArrayIteratorAsync<TArrayOfBytesIterator>
{
    pub async fn new(mut data: TArrayOfBytesIterator) -> Self {
        async_reader::skip_white_spaces(&mut data).await.unwrap();
        Self {
            data,
            initialized: false,
        }
    }

    async fn init(&mut self) -> Result<(), JsonParseError> {
        let result =
            async_reader::next_token_must_be(&mut self.data, crate::consts::OPEN_ARRAY).await;

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

    pub async fn get_next(&mut self) -> Option<Result<Vec<u8>, JsonParseError>> {
        let start_value = if !self.initialized {
            match self.init().await {
                Ok(_) => match async_reader::skip_white_spaces(&mut self.data).await {
                    Ok(value) => value,
                    Err(err) => return Some(Err(err)),
                },
                Err(err) => return Some(Err(err)),
            }
        } else {
            let next_pos = match async_reader::skip_white_spaces_and_get_next(&mut self.data).await
            {
                Ok(value) => value,
                Err(err) => return Some(Err(err)),
            };

            match next_pos.value {
                crate::consts::CLOSE_ARRAY => {
                    return None;
                }
                crate::consts::COMMA => match async_reader::skip_white_spaces(&mut self.data).await
                {
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
                match async_reader::find_the_end_of_the_string(&mut self.data).await {
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }

            crate::consts::OPEN_ARRAY => {
                match async_reader::find_the_end_of_array(&mut self.data).await {
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }

            crate::consts::OPEN_BRACKET => {
                match async_reader::find_the_end_of_json(&mut self.data).await {
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }
            crate::consts::START_OF_NULL_UPPER_CASE => {
                if let Err(err) = async_reader::check_json_symbol(&mut self.data, "null").await {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_NULL_LOWER_CASE => {
                if let Err(err) = async_reader::check_json_symbol(&mut self.data, "null").await {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_TRUE_UPPER_CASE => {
                if let Err(err) = async_reader::check_json_symbol(&mut self.data, "true").await {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_TRUE_LOWER_CASE => {
                if let Err(err) = async_reader::check_json_symbol(&mut self.data, "true").await {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_FALSE_UPPER_CASE => {
                if let Err(err) = async_reader::check_json_symbol(&mut self.data, "false").await {
                    return Some(Err(err));
                }
            }

            crate::consts::START_OF_FALSE_LOWER_CASE => {
                if let Err(err) = async_reader::check_json_symbol(&mut self.data, "false").await {
                    return Some(Err(err));
                }
            }

            _ => {
                if async_reader::is_number(start_value.value) {
                    match async_reader::find_the_end_of_the_number(&mut self.data).await {
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

        let result = self
            .data
            .get_slice_to_current_pos(start_value.pos)
            .await
            .unwrap();
        return Some(Ok(result));
    }
}

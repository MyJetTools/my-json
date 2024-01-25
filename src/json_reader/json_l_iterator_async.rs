use super::{consts, JsonParseError};
use rust_extensions::array_of_bytes_iterator::*;

pub struct JsonLIteratorAsync<TArrayOfBytesIterator: ArrayOfBytesIteratorAsync> {
    data: TArrayOfBytesIterator,
}

impl<TArrayOfBytesIterator: ArrayOfBytesIteratorAsync> JsonLIteratorAsync<TArrayOfBytesIterator> {
    pub async fn new(mut data: TArrayOfBytesIterator) -> Self {
        super::byte_of_array_reader_async::skip_white_spaces(&mut data)
            .await
            .unwrap();
        Self { data }
    }

    pub async fn get_next<'s>(&'s mut self) -> Option<Result<Vec<u8>, JsonParseError>> {
        let start_value =
            match super::byte_of_array_reader_async::skip_white_spaces(&mut self.data).await {
                Ok(value) => value,
                Err(_) => return None,
            };

        match start_value.value {
            consts::OPEN_BRACKET => {
                match super::byte_of_array_reader_async::find_the_end_of_json_object_or_array(
                    &mut self.data,
                )
                .await
                {
                    Ok(_) => {}
                    Err(err) => return Some(Err(err)),
                }
            }

            _ => {
                return Some(Err(JsonParseError::new(format!(
                    "Error reading value as object. Start {}. We reached the end of the payload",
                    start_value.pos
                ))));
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

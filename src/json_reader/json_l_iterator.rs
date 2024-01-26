use super::{bytes_of_array_reader::*, consts, JsonParseError};
use rust_extensions::array_of_bytes_iterator::*;

pub struct JsonLIterator<TArrayOfBytesIterator: ArrayOfBytesIterator> {
    data: TArrayOfBytesIterator,
}

impl<TArrayOfBytesIterator: ArrayOfBytesIterator> JsonLIterator<TArrayOfBytesIterator> {
    pub fn new(mut data: TArrayOfBytesIterator) -> Self {
        sync_reader::skip_white_spaces(&mut data).unwrap();
        Self { data }
    }

    pub fn get_next<'s>(&'s mut self) -> Option<Result<&'s [u8], JsonParseError>> {
        let start_value = match sync_reader::skip_white_spaces(&mut self.data) {
            Ok(value) => value,
            Err(_) => return None,
        };

        match start_value.value {
            consts::OPEN_BRACKET => match sync_reader::find_the_end_of_json(&mut self.data) {
                Ok(_) => {}
                Err(err) => return Some(Err(err)),
            },

            _ => {
                return Some(Err(JsonParseError::new(format!(
                    "Error reading value as object. Start {}. We reached the end of the payload",
                    start_value.pos
                ))));
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
    pub fn test_basic_json_l_split() {
        let json = r###"{"id":1}
        {"id":2}{"id":3}"###;

        println!("{}", json);

        let mut i = 0;

        let slice_iterator = SliceIterator::from_str(json);

        let mut json_array_iterator = JsonLIterator::new(slice_iterator);

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
    pub fn test_with_array_inside_json_l_split() {
        let json = r###"{
            "names": [
                {
                    "company_name": "Company \"A\""
                },
                {
                    "company_name": "Company \"B\""
                },
                {
                    "company_name": "Company \"C\""
                }
            ],
            "registered_address": "Addr",
            "retrieved_at": "2010-02-23"
        }"###;

        println!("{}", json);

        let mut i = 0;

        let slice_iterator = SliceIterator::from_str(json);

        let mut json_array_iterator = JsonLIterator::new(slice_iterator);

        while let Some(sub_json) = json_array_iterator.get_next() {
            let sub_json = sub_json.unwrap();
            i += 1;
            println!("{}", i);
            println!("{}", std::str::from_utf8(sub_json).unwrap());
        }
    }
}

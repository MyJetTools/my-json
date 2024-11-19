use rust_extensions::array_of_bytes_iterator::SliceIterator;

use super::{array_iterator::JsonArrayIterator, JsonParseError, JsonValueRef};

pub struct JsonArrayIteratorFromSlice<'s> {
    slice: JsonArrayIterator<SliceIterator<'s>>,
}

impl<'s> JsonArrayIteratorFromSlice<'s> {
    pub fn new(slice: &'s [u8]) -> Result<Self, JsonParseError> {
        let slice = JsonArrayIterator::new(SliceIterator::new(slice))?;
        let result = Self { slice };

        Ok(result)
    }

    pub fn get_next(&self) -> Option<Result<JsonValueRef, JsonParseError>> {
        let result = self.slice.get_next()?;

        match result {
            Ok(value) => {
                let result = JsonValueRef::new(value, self.slice.get_src_slice());
                Some(Ok(result))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'s> Into<JsonArrayIteratorFromSlice<'s>> for &'s [u8] {
    fn into(self) -> JsonArrayIteratorFromSlice<'s> {
        JsonArrayIteratorFromSlice::new(self).unwrap()
    }
}

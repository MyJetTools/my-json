use rust_extensions::array_of_bytes_iterator::SliceIterator;

use crate::json_reader::{AsJsonSlice, JsonFieldNameRef, JsonParseError, JsonValueRef};

use super::JsonFirstLineReader;

pub struct FirstLineReaderFromSlice<'s> {
    reader: JsonFirstLineReader<SliceIterator<'s>>,
}

impl<'s> FirstLineReaderFromSlice<'s> {
    pub fn new(slice: &'s [u8]) -> Self {
        let slice_iterator = SliceIterator::new(slice);
        let reader = JsonFirstLineReader::new(slice_iterator);
        Self { reader }
    }

    pub fn get_next(
        &'s self,
    ) -> Option<Result<(JsonFieldNameRef<'s>, JsonValueRef<'s>), JsonParseError>> {
        let result = self.reader.get_next()?;

        match result {
            Ok(item) => {
                let key = JsonFieldNameRef::new(item.name, self.reader.as_slice());
                let value = JsonValueRef::new(item.value, self.reader.as_slice());
                Some(Ok((key, value)))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'s> Into<FirstLineReaderFromSlice<'s>> for JsonFirstLineReader<SliceIterator<'s>> {
    fn into(self) -> FirstLineReaderFromSlice<'s> {
        FirstLineReaderFromSlice { reader: self }
    }
}

#[cfg(test)]
mod tests {
    use crate::json_reader::FirstLineReaderFromSlice;

    #[test]
    fn test_basic_case() {
        let json = r#"{"key1": "value1", "key2": "value2", "object": { "key":"value" }}"#;

        let reader = super::FirstLineReaderFromSlice::new(json.as_bytes());

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key1");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value1");

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key2");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value2");

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "object");

        let value: FirstLineReaderFromSlice<'_> = item.1.unwrap_as_object().unwrap().into();

        let item = value.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value");

        assert!(value.get_next().is_none());
    }
}

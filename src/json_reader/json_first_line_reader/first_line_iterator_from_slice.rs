use rust_extensions::array_of_bytes_iterator::SliceIterator;

use crate::json_reader::{AsJsonSlice, JsonFieldNameRef, JsonParseError, JsonValueRef};

use super::JsonFirstLineReader;

pub struct JsonFirstLineIteratorFromSlice<'s> {
    reader: JsonFirstLineReader<SliceIterator<'s>>,
}

impl<'s> JsonFirstLineIteratorFromSlice<'s> {
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

    pub fn as_str(&'s self) -> &'s str {
        std::str::from_utf8(self.reader.as_slice()).unwrap()
    }
}

impl<'s> Into<JsonFirstLineIteratorFromSlice<'s>> for JsonFirstLineReader<SliceIterator<'s>> {
    fn into(self) -> JsonFirstLineIteratorFromSlice<'s> {
        JsonFirstLineIteratorFromSlice { reader: self }
    }
}

impl<'s> Into<JsonFirstLineIteratorFromSlice<'s>> for &'s str {
    fn into(self) -> JsonFirstLineIteratorFromSlice<'s> {
        JsonFirstLineIteratorFromSlice::new(self.as_bytes())
    }
}

impl<'s> Into<JsonFirstLineIteratorFromSlice<'s>> for &'s [u8] {
    fn into(self) -> JsonFirstLineIteratorFromSlice<'s> {
        JsonFirstLineIteratorFromSlice::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::json_reader::JsonFirstLineIteratorFromSlice;

    #[test]
    fn test_basic_case() {
        let json = r#"{"key1": "value1", "key2": "value2", "object": { "key":"value" }}"#;

        let reader = super::JsonFirstLineIteratorFromSlice::new(json.as_bytes());

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key1");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value1");

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key2");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value2");

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "object");

        let value: JsonFirstLineIteratorFromSlice<'_> = item.1.unwrap_as_object().unwrap().into();

        let item = value.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value");

        assert!(value.get_next().is_none());
    }
}

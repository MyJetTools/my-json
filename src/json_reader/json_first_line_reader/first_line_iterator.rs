use rust_extensions::array_of_bytes_iterator::SliceIterator;

use crate::json_reader::{AsJsonSlice, JsonFieldNameRef, JsonParseError, JsonValueRef};

use super::reader_inner::JsonFirstLineReaderInner;

pub struct JsonFirstLineIterator<'s> {
    inner: JsonFirstLineReaderInner<SliceIterator<'s>>,
}

impl<'s> JsonFirstLineIterator<'s> {
    pub fn new(slice: &'s [u8]) -> Self {
        let slice_iterator = SliceIterator::new(slice);
        let reader = JsonFirstLineReaderInner::new(slice_iterator);
        Self { inner: reader }
    }

    pub fn get_next(
        &'s self,
    ) -> Option<Result<(JsonFieldNameRef<'s>, JsonValueRef<'s>), JsonParseError>> {
        let result = self.inner.get_next()?;

        match result {
            Ok(item) => {
                let key = JsonFieldNameRef::new(item.name, self.inner.as_slice());
                let value = JsonValueRef::new(item.value, self.inner.as_slice());
                Some(Ok((key, value)))
            }
            Err(err) => Some(Err(err)),
        }
    }

    pub fn as_str(&'s self) -> &'s str {
        std::str::from_utf8(self.inner.as_slice()).unwrap()
    }
}

impl<'s> Into<JsonFirstLineIterator<'s>> for JsonFirstLineReaderInner<SliceIterator<'s>> {
    fn into(self) -> JsonFirstLineIterator<'s> {
        JsonFirstLineIterator { inner: self }
    }
}

impl<'s> Into<JsonFirstLineIterator<'s>> for &'s str {
    fn into(self) -> JsonFirstLineIterator<'s> {
        JsonFirstLineIterator::new(self.as_bytes())
    }
}

impl<'s> Into<JsonFirstLineIterator<'s>> for &'s [u8] {
    fn into(self) -> JsonFirstLineIterator<'s> {
        JsonFirstLineIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::json_reader::JsonFirstLineIterator;

    #[test]
    fn test_basic_case() {
        let json = r#"{"key1": "value1", "key2": "value2", "object": { "key":"value" }}"#;

        let reader = super::JsonFirstLineIterator::new(json.as_bytes());

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key1");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value1");

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key2");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value2");

        let item = reader.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "object");

        let value: JsonFirstLineIterator<'_> = item.1.unwrap_as_object().unwrap().into();

        let item = value.get_next().unwrap().unwrap();

        assert_eq!(item.0.as_str().unwrap().as_str(), "key");
        assert_eq!(item.1.as_str().unwrap().as_str(), "value");

        assert!(value.get_next().is_none());
    }
}

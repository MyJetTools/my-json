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

    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
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

    #[test]
    fn tests() {
        let json = r#"{"PartitionKey":"t","RowKey":"fab16435","TimeStamp":"","TraderId":"284daf07","Expires":"2024-12-24T11:46:51.249725+00:00","Claims":null}"#;

        let json_first_line_iterator = JsonFirstLineIterator::new(json.as_bytes());

        while let Some(next) = json_first_line_iterator.get_next() {
            let (key, value) = next.unwrap();
            println!(
                "{}: {}",
                key.as_raw_str().unwrap(),
                value.as_raw_str().unwrap()
            );
        }
    }

    #[test]
    pub fn test_empty_object() {
        let json = "{}";

        let json_array_iterator = JsonFirstLineIterator::new(json.as_bytes());

        let mut items = 0;

        while let Some(sub_json) = json_array_iterator.get_next() {
            let _ = sub_json.unwrap();

            items += 1;
        }

        assert_eq!(items, 0);
    }

    #[test]
    pub fn test_empty_object_with_spaces_inside() {
        let json = "{  }";

        let json_array_iterator = JsonFirstLineIterator::new(json.as_bytes());

        let mut items = 0;

        while let Some(sub_json) = json_array_iterator.get_next() {
            let _ = sub_json.unwrap();

            items += 1;
        }

        assert_eq!(items, 0);
    }

    #[test]
    pub fn test_empty_object_with_spaces_everywhere() {
        let json = "  {  } ";

        let json_array_iterator = JsonFirstLineIterator::new(json.as_bytes());

        let mut items = 0;

        while let Some(sub_json) = json_array_iterator.get_next() {
            let _ = sub_json.unwrap();

            items += 1;
        }

        assert_eq!(items, 0);
    }
}

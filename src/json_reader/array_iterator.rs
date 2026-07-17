use rust_extensions::array_of_bytes_iterator::SliceIterator;

use super::{array_iterator_inner::JsonArrayIteratorInner, JsonParseError, JsonValueRef};

pub struct JsonArrayIterator<'s> {
    iterator: JsonArrayIteratorInner<SliceIterator<'s>>,
}

impl<'s> JsonArrayIterator<'s> {
    pub fn new(slice: &'s [u8]) -> Result<Self, JsonParseError> {
        let slice = JsonArrayIteratorInner::new(SliceIterator::new(slice))?;
        let result = Self { iterator: slice };

        Ok(result)
    }

    pub fn get_next(&'s self) -> Option<Result<JsonValueRef<'s>, JsonParseError>> {
        let result = self.iterator.get_next()?;

        match result {
            Ok(value) => {
                let result = JsonValueRef::new(value, self.iterator.get_src_slice());
                Some(Ok(result))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'s> Into<JsonArrayIterator<'s>> for &'s [u8] {
    fn into(self) -> JsonArrayIterator<'s> {
        JsonArrayIterator::new(self).unwrap()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_number() {
        let src = "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]";

        let iter: super::JsonArrayIterator = src.as_bytes().into();

        let item = iter.get_next().unwrap().unwrap();

        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 1);
        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 2);

        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 3);

        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 4);

        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 5);

        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 6);

        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 7);

        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 8);

        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 9);

        let item = iter.get_next().unwrap().unwrap();
        assert_eq!(item.unwrap_as_number().unwrap().unwrap(), 10);

        let item = iter.get_next();
        assert!(item.is_none());
    }

    /// Regression: `find_the_end_of_array` used to demand a value straight after the `[`, so an
    /// empty array *nested inside* an array was unreadable - `[[],[1]]` failed at the `]` of the
    /// inner empty one. A top level `[]` always worked (the iterator checks for the close token
    /// itself), which is why this went unnoticed; the writer emits `[[],[1]]` for a
    /// `Vec<Vec<T>>` whose first element is empty.
    #[test]
    fn test_empty_array_nested_inside_an_array() {
        for (json, expected) in [
            ("[[]]", vec!["[]"]),
            ("[[],[1]]", vec!["[]", "[1]"]),
            ("[[1],[]]", vec!["[1]", "[]"]),
            ("[[],[]]", vec!["[]", "[]"]),
            ("[[], [ ] ]", vec!["[]", "[ ]"]),
            ("[[[]],[]]", vec!["[[]]", "[]"]),
            ("[{},[]]", vec!["{}", "[]"]),
        ] {
            let iter: super::JsonArrayIterator = json.as_bytes().into();

            let mut found = Vec::new();
            while let Some(item) = iter.get_next() {
                let item = item.unwrap_or_else(|err| {
                    panic!("failed to iterate {}: {}", json, err.to_string())
                });
                found.push(std::str::from_utf8(item.as_slice()).unwrap().to_string());
            }

            assert_eq!(found, expected, "iterating {}", json);
        }
    }

    /// The empty-array check must not loosen the grammar: a trailing comma is still an error.
    #[test]
    fn test_nested_array_with_trailing_comma_is_still_rejected() {
        let iter: super::JsonArrayIterator = "[[1,],[2]]".as_bytes().into();
        assert!(iter.get_next().unwrap().is_err());
    }

    #[test]
    fn test_objects() {
        let src =
            r#"[{"PartitionKey":"pk1", "RowKey":"rk1"}, {"PartitionKey":"pk2", "RowKey":"rk2"}]"#;

        let array_iter: super::JsonArrayIterator = src.as_bytes().into();

        let next_element = array_iter.get_next().unwrap().unwrap();

        let obj = next_element.unwrap_as_object().unwrap();

        let (name, value) = obj.get_next().unwrap().unwrap();

        assert_eq!(name.as_str().unwrap().as_str(), "PartitionKey");
        assert_eq!(value.as_str().unwrap().as_str(), "pk1");

        let (name, value) = obj.get_next().unwrap().unwrap();
        assert_eq!(name.as_str().unwrap().as_str(), "RowKey");
        assert_eq!(value.as_str().unwrap().as_str(), "rk1");

        let next_element = array_iter.get_next().unwrap().unwrap();

        let obj = next_element.unwrap_as_object().unwrap();

        let (name, value) = obj.get_next().unwrap().unwrap();

        assert_eq!(name.as_str().unwrap().as_str(), "PartitionKey");
        assert_eq!(value.as_str().unwrap().as_str(), "pk2");

        let (name, value) = obj.get_next().unwrap().unwrap();
        assert_eq!(name.as_str().unwrap().as_str(), "RowKey");
        assert_eq!(value.as_str().unwrap().as_str(), "rk2");

        assert!(array_iter.get_next().is_none())
    }
}

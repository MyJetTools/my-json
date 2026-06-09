use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

use super::{
    json_value::UnwrappedJsonValue, JsonArrayIterator, JsonFirstLineIterator, JsonParseError,
    JsonValue,
};

#[derive(Clone, Debug)]
pub struct JsonValueRef<'s> {
    pub data: JsonValue,
    pub json_slice: &'s [u8],
}

impl<'s> JsonValueRef<'s> {
    pub fn new(src: JsonValue, json_slice: &'s [u8]) -> Self {
        return Self {
            data: src,
            json_slice,
        };
    }

    pub fn unwrap_value(&'s self) -> Result<UnwrappedJsonValue<'s>, JsonParseError> {
        self.data.unwrap_value(&self.json_slice)
    }

    pub fn is_null(&'s self) -> bool {
        self.data.is_null(&self.json_slice)
    }

    pub fn unwrap_as_number(&'s self) -> Result<Option<i64>, JsonParseError> {
        self.data.unwrap_as_number(&self.json_slice)
    }

    pub fn is_object(&'s self) -> bool {
        self.data.is_object(&self.json_slice)
    }

    pub fn unwrap_as_object(&'s self) -> Result<JsonFirstLineIterator<'s>, JsonParseError> {
        self.data.unwrap_as_object(&self.json_slice)
    }

    pub fn unwrap_as_bool(&'s self) -> Option<bool> {
        self.data.unwrap_as_bool(&self.json_slice)
    }

    pub fn is_bool(&'s self) -> bool {
        self.data.is_bool(&self.json_slice)
    }

    pub fn is_string(&'s self) -> bool {
        self.data.is_string(&self.json_slice)
    }

    pub fn is_number(&'s self) -> bool {
        self.data.is_number(&self.json_slice)
    }

    pub fn is_double(&'s self) -> bool {
        self.data.is_double(&self.json_slice)
    }

    pub fn unwrap_as_double(&'s self) -> Result<Option<f64>, JsonParseError> {
        self.data.unwrap_as_double(&self.json_slice)
    }

    pub fn is_array(&'s self) -> bool {
        self.data.is_array(&self.json_slice)
    }

    pub fn as_bytes(&'s self) -> &'s [u8] {
        self.data.as_bytes(&self.json_slice)
    }

    pub fn unwrap_as_array(&'s self) -> Result<JsonArrayIterator<'s>, JsonParseError> {
        self.data.unwrap_as_array(&self.json_slice)
    }

    pub fn as_str(&'s self) -> Option<StrOrString<'s>> {
        self.data.as_str(&self.json_slice)
    }

    pub fn as_unescaped_str(&'s self) -> Option<&'s str> {
        self.data.as_unescaped_str(&self.json_slice)
    }

    pub fn as_raw_str(&'s self) -> Option<&'s str> {
        self.data.as_raw_str(&self.json_slice)
    }

    pub fn as_slice(&'s self) -> &'s [u8] {
        &self.json_slice[self.data.start..self.data.end]
    }

    pub fn as_date_time(&'s self) -> Option<DateTimeAsMicroseconds> {
        self.data.as_date_time(&self.json_slice)
    }

    pub fn j_query_as_value(
        &'s self,
        path: &str,
    ) -> Result<Option<JsonValueRef<'s>>, JsonParseError> {
        let data = &self.json_slice[self.data.start..self.data.end];
        let result = crate::j_path::get_value(data, path)?;
        return Ok(result.map(|mut ref_| {
            ref_.data.start += self.data.start;
            ref_.data.end += self.data.start;
            ref_.json_slice = self.json_slice;
            ref_
        }));
    }

    pub fn j_query_as_vec(&'s self, path: &str) -> Result<Vec<JsonValueRef<'s>>, JsonParseError> {
        if path == "[]" && self.is_array() {
            let data = &self.json_slice[self.data.start..self.data.end];
            let reader = JsonArrayIterator::new(data)?;
            let mut result = vec![];
            while let Some(item) = reader.get_next() {
                let value = item?;
                result.push(JsonValueRef {
                    data: JsonValue::new(
                        value.data.start + self.data.start,
                        value.data.end + self.data.start,
                    ),
                    json_slice: self.json_slice,
                });
            }
            return Ok(result);
        }

        let data = &self.json_slice[self.data.start..self.data.end];
        let result = crate::j_path::get_value_as_vec(data, path)?;
        return Ok(result.into_iter().map(|mut ref_| {
            ref_.data.start += self.data.start;
            ref_.data.end += self.data.start;
            ref_.json_slice = self.json_slice;
            ref_
        }).collect());
    }
}

#[cfg(test)]
mod tests {
    use crate::j_path;

    #[test]
    fn test_j_query_as_value_nested_object() {
        let json = br#"{"user":{"name":"John","age":30},"status":"active"}"#;
        let user = j_path::get_value(json, "user").unwrap().unwrap();
        assert!(user.is_object());

        let name = user.j_query_as_value("name").unwrap().unwrap();
        assert!(name.is_string());
        assert_eq!(name.as_str().unwrap().to_string(), "John");
    }

    #[test]
    fn test_j_query_as_value_returns_correct_slice() {
        let json = br#"{"user":{"name":"Alice"}}"#;
        let user = j_path::get_value(json, "user").unwrap().unwrap();
        let name = user.j_query_as_value("name").unwrap().unwrap();

        let slice = name.as_slice();
        assert_eq!(slice, br#""Alice""#);
    }

    #[test]
    fn test_j_query_as_vec_multiple_matches() {
        let json = br#"{"items":[{"id":1},{"id":2},{"id":3}]}"#;
        let items = j_path::get_value(json, "items").unwrap().unwrap();

        let ids = items.j_query_as_vec("[]").unwrap();
        assert_eq!(ids.len(), 3);

        for id_ref in ids {
            assert!(id_ref.is_object());
        }
    }

    #[test]
    fn test_j_query_nested_with_array() {
        let json = br#"{"data":{"users":[{"name":"Bob"},{"name":"Carol"}]}}"#;
        let data = j_path::get_value(json, "data").unwrap().unwrap();
        let users = data.j_query_as_value("users").unwrap().unwrap();

        let user_objs = users.j_query_as_vec("[]").unwrap();
        assert_eq!(user_objs.len(), 2);

        let names = user_objs.iter()
            .filter_map(|u| u.j_query_as_value("name").ok().flatten())
            .collect::<Vec<_>>();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_lifetime_annotations_consistency() {
        let json = br#"{"value":42.5}"#;
        let value = j_path::get_value(json, "value").unwrap().unwrap();
        assert!(value.is_double());
        assert!(!value.is_bool());

        let num = value.unwrap_as_double().unwrap().unwrap();
        assert_eq!(num, 42.5);
    }

    #[test]
    fn test_j_query_as_value_empty_result() {
        let json = br#"{"user":{"name":"test"}}"#;
        let user = j_path::get_value(json, "user").unwrap().unwrap();

        let result = user.j_query_as_value("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_j_query_preserves_slice_validity() {
        let json = br#"{"outer":{"inner":{"value":"test"}}}"#;
        let outer = j_path::get_value(json, "outer").unwrap().unwrap();
        let inner = outer.j_query_as_value("inner").unwrap().unwrap();
        let value = inner.j_query_as_value("value").unwrap().unwrap();

        let slice = value.as_slice();
        assert_eq!(slice, br#""test""#);

        let str_val = value.as_str().unwrap().to_string();
        assert_eq!(str_val, "test");
    }

    #[test]
    fn test_j_query_three_levels_deep() {
        let json = br#"{"a":{"b":{"c":"deep"}}}"#;
        let a = j_path::get_value(json, "a").unwrap().unwrap();
        let b = a.j_query_as_value("b").unwrap().unwrap();
        let c = b.j_query_as_value("c").unwrap().unwrap();

        assert!(c.is_string());
        assert_eq!(c.as_str().unwrap().to_string(), "deep");
    }
}

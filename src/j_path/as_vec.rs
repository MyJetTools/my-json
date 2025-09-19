use rust_extensions::StrOrString;

use crate::json_reader::{JsonArrayIterator, JsonFirstLineIterator, JsonParseError, JsonValueRef};

pub fn get_value_as_vec<'s, 'd>(
    json: &'s [u8],
    path: impl Into<StrOrString<'d>>,
) -> Result<Vec<JsonValueRef<'s>>, JsonParseError> {
    let path: StrOrString = path.into();

    as_array_internal(json, path.as_str())
}
fn as_array_internal<'s>(
    json: &'s [u8],
    path: &str,
) -> Result<Vec<JsonValueRef<'s>>, JsonParseError> {
    if path.is_empty() {
        return Ok(vec![]);
    }
    let reader = JsonFirstLineIterator::new(json);

    let j_path_reader = crate::j_path::JPathReader::new(path);

    let j_prop_name = j_path_reader.get_prop_name();

    match j_prop_name {
        super::JPropName::Name(j_prop_name) => {
            while let Some(next) = reader.get_next() {
                let (key, value) = next?;

                let key = key.as_str()?;

                if key.as_str() == j_prop_name {
                    match j_path_reader.get_next_level_path() {
                        Some(next_level_path) => {
                            let data = &json[value.data.start..value.data.end];
                            return as_array_internal(data, next_level_path);
                        }
                        None => {
                            return Ok(vec![JsonValueRef {
                                data: value.data,
                                json_slice: json,
                            }]);
                        }
                    }
                }
            }
        }
        super::JPropName::ArrayAndIndex { j_prop_name, index } => {
            while let Some(next) = reader.get_next() {
                let (key, value) = next?;

                let key = key.as_str()?;

                if key.as_str() == j_prop_name {
                    match j_path_reader.get_next_level_path() {
                        Some(next_level_path) => {
                            let data = &json[value.data.start..value.data.end];
                            match super::find_object_from_array(data, next_level_path, index)? {
                                Some(result) => return Ok(vec![result]),
                                None => return Ok(vec![]),
                            }
                        }
                        None => {
                            let data = &json[value.data.start..value.data.end];
                            match super::find_object_from_array(data, "", index)? {
                                Some(result) => return Ok(vec![result]),
                                None => return Ok(vec![]),
                            }
                        }
                    }
                }
            }
        }
        super::JPropName::Array(j_prop_name) => {
            while let Some(next) = reader.get_next() {
                let (key, value) = next?;

                let key = key.as_str()?;

                if key.as_str() == j_prop_name {
                    match j_path_reader.get_next_level_path() {
                        Some(next_level_path) => {
                            if value.is_array() {
                                let data = &json[value.data.start..value.data.end];
                                return iterate_items(data, next_level_path);
                            } else {
                                let data = &json[value.data.start..value.data.end];
                                return as_array_internal(data, next_level_path);
                            }
                        }
                        None => {
                            if value.is_array() {
                                let data = &json[value.data.start..value.data.end];
                                return iterate_last_items(data);
                            } else {
                                return Ok(vec![JsonValueRef {
                                    data: value.data,
                                    json_slice: json,
                                }]);
                            }
                        }
                    }
                }
            }

            return Ok(vec![]);
        }
    }

    Ok(vec![])
}

fn iterate_last_items<'s>(json: &'s [u8]) -> Result<Vec<JsonValueRef<'s>>, JsonParseError> {
    let json_array_iterator = JsonArrayIterator::new(json)?;

    let mut result = vec![];

    while let Some(item) = json_array_iterator.get_next() {
        let value = item?;
        result.push(JsonValueRef {
            data: value.data,
            json_slice: json,
        });
    }

    Ok(result)
}

fn iterate_items<'s>(json: &'s [u8], path: &str) -> Result<Vec<JsonValueRef<'s>>, JsonParseError> {
    let json_array_iterator = JsonArrayIterator::new(json)?;

    let mut result = vec![];

    while let Some(item) = json_array_iterator.get_next() {
        let value = item?;
        let data = &json[value.data.start..value.data.end];

        let item = super::get_value(data, path)?;
        if let Some(item) = item {
            result.push(item);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_array_with_multiple_items() {
        // Test JSON with an array containing multiple items
        let json = r#"{
            "users": [
                {"id": 1, "name": "Alice", "active": true},
                {"id": 2, "name": "Bob", "active": false},
                {"id": 3, "name": "Charlie", "active": true}
            ]
        }"#;

        // Test getting all users from the array
        let result = get_value_as_vec(json.as_bytes(), "users[]").unwrap();

        // Should return all 3 users
        assert_eq!(result.len(), 3);

        assert_eq!(
            r#"{"id": 1, "name": "Alice", "active": true}"#,
            result.get(0).unwrap().as_raw_str().unwrap(),
        );

        assert_eq!(
            r#"{"id": 2, "name": "Bob", "active": false}"#,
            result.get(1).unwrap().as_raw_str().unwrap(),
        );

        assert_eq!(
            r#"{"id": 3, "name": "Charlie", "active": true}"#,
            result.get(2).unwrap().as_raw_str().unwrap(),
        );
    }

    #[test]
    fn test_as_array_with_multiple_items_and_exact_value() {
        // Test JSON with an array containing multiple items
        let json = r#"{
            "users": [
                {"id": 1, "name": "Alice", "active": true},
                {"id": 2, "name": "Bob", "active": false},
                {"id": 3, "name": "Charlie", "active": true}
            ]
        }"#;

        // Test getting all users from the array
        let result = get_value_as_vec(json.as_bytes(), "users[].name").unwrap();

        // Should return all 3 users
        assert_eq!(result.len(), 3);

        assert_eq!("Alice", result.get(0).unwrap().as_str().unwrap().as_str(),);

        assert_eq!("Bob", result.get(1).unwrap().as_str().unwrap().as_str(),);

        assert_eq!("Charlie", result.get(2).unwrap().as_str().unwrap().as_str(),);
    }
}

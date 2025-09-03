use crate::json_reader::{JsonArrayIterator, JsonFirstLineIterator, JsonParseError, JsonValueRef};

pub fn j_path<'s>(json: &'s [u8], path: &str) -> Result<Option<JsonValueRef<'s>>, JsonParseError> {
    if path.is_empty() {
        return Ok(None);
    }
    let reader = JsonFirstLineIterator::new(json);

    let j_path_reader = crate::j_path::JPathReader::new(path);

    let j_prop_name = j_path_reader.get_prop_name();

    match j_prop_name {
        super::JPropName::Name(j_prop_name) => {
            while let Some(next) = reader.get_next() {
                let (key, value) = next.unwrap();

                let key = key.as_str()?;

                if key.as_str() == j_prop_name {
                    match j_path_reader.get_next_level_path() {
                        Some(next_level_path) => {
                            let data = &json[value.data.start..value.data.end];
                            return j_path(data, next_level_path);
                        }
                        None => {
                            return Ok(Some(JsonValueRef {
                                data: value.data,
                                json_slice: json,
                            }));
                        }
                    }
                }
            }
        }
        super::JPropName::Array { j_prop_name, index } => {
            while let Some(next) = reader.get_next() {
                let (key, value) = next.unwrap();

                let key = key.as_str()?;

                if key.as_str() == j_prop_name {
                    match j_path_reader.get_next_level_path() {
                        Some(next_level_path) => {
                            let data = &json[value.data.start..value.data.end];
                            return find_object_from_array(data, next_level_path, index);
                        }
                        None => {
                            let data = &json[value.data.start..value.data.end];
                            return find_object_from_array(data, "", index);
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

fn find_object_from_array<'s>(
    json: &'s [u8],
    next_path: &str,
    index: usize,
) -> Result<Option<JsonValueRef<'s>>, JsonParseError> {
    let reader = JsonArrayIterator::new(json)?;

    let mut i = 0;
    while let Some(array_item) = reader.get_next() {
        let array_item = array_item?;

        if i == index {
            if next_path.is_empty() {
                return Ok(Some(JsonValueRef {
                    data: array_item.data,
                    json_slice: json,
                }));
            }

            let json = &json[array_item.data.start..array_item.data.end];
            return j_path(json, next_path);
        }

        i += 1;
    }

    Ok(None)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic_case() {
        let json = r#"{"key1": "value1", "key2": "value2", "object": { "key":"value" }}"#;

        let result = super::j_path(json.as_bytes(), "key1").unwrap().unwrap();

        assert_eq!(result.as_str().unwrap().as_str(), "value1");

        let result = super::j_path(json.as_bytes(), "key2").unwrap().unwrap();

        assert_eq!(result.as_str().unwrap().as_str(), "value2");

        let result = super::j_path(json.as_bytes(), "object").unwrap().unwrap();

        assert!(result.is_object());
    }

    #[test]
    fn test_basic_case_inside_object() {
        let json = r#"{"key1": "value1", "key2": "value2", "object": { "key":"value" }}"#;

        let result = super::j_path(json.as_bytes(), "object.key")
            .unwrap()
            .unwrap();

        assert_eq!(result.as_str().unwrap().as_str(), "value");
    }

    #[test]
    fn test_basic_case_inside_twp_objects() {
        let json =
            r#"{"key1": "value1", "key2": "value2", "object": { "key":{ "key2":"value2" } }}"#;

        let result = super::j_path(json.as_bytes(), "object.key.key2")
            .unwrap()
            .unwrap();

        assert_eq!(result.as_str().unwrap().as_str(), "value2");
    }

    #[test]
    fn test_with_array() {
        let json = r#"{ "key1":"value1", "key2":"value2", "array":[{"key":"a0"},{"key":"a1"},{"key":"a2"}] }"#;

        let result = super::j_path(json.as_bytes(), "array[1].key")
            .unwrap()
            .unwrap();

        assert_eq!(result.as_str().unwrap().as_str(), "a1");

        let result = super::j_path(json.as_bytes(), "array[2].key")
            .unwrap()
            .unwrap();

        assert_eq!(result.as_str().unwrap().as_str(), "a2");
    }

    // Additional comprehensive tests
    #[test]
    fn test_empty_path() {
        let json = r#"{"key": "value"}"#;
        let result = super::j_path(json.as_bytes(), "").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_nonexistent_path() {
        let json = r#"{"key": "value"}"#;
        let result = super::j_path(json.as_bytes(), "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_nonexistent_nested_path() {
        let json = r#"{"user": {"name": "Alice"}}"#;
        let result = super::j_path(json.as_bytes(), "user.age").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_nonexistent_array_index() {
        let json = r#"{"array": [1, 2, 3]}"#;
        let result = super::j_path(json.as_bytes(), "array[5]").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_array_without_index() {
        let json = r#"{"array": [1, 2, 3]}"#;
        let result = super::j_path(json.as_bytes(), "array").unwrap().unwrap();
        assert!(result.is_array());
    }

    #[test]
    fn test_deep_nesting() {
        let json = r#"{"level1": {"level2": {"level3": {"level4": {"level5": "deep_value"}}}}}"#;
        let result = super::j_path(json.as_bytes(), "level1.level2.level3.level4.level5")
            .unwrap()
            .unwrap();
        assert_eq!(result.as_str().unwrap().as_str(), "deep_value");
    }

    #[test]
    fn test_array_in_object() {
        let json = r#"{"user": {"hobbies": ["reading", "gaming", "coding"]}}"#;
        let result = super::j_path(json.as_bytes(), "user.hobbies[1]")
            .unwrap()
            .unwrap();
        assert_eq!(result.as_str().unwrap().as_str(), "gaming");
    }

    #[test]
    fn test_object_in_array() {
        let json = r#"{"users": [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 30}]}"#;
        let result = super::j_path(json.as_bytes(), "users[0].name")
            .unwrap()
            .unwrap();
        assert_eq!(result.as_str().unwrap().as_str(), "Alice");

        let result = super::j_path(json.as_bytes(), "users[1].age")
            .unwrap()
            .unwrap();
        assert!(result.is_number());
    }

    #[test]
    fn test_mixed_types() {
        let json = r#"{
            "string": "hello",
            "number": 42,
            "boolean": true,
            "null_value": null,
            "array": [1, 2, 3],
            "object": {"key": "value"}
        }"#;

        let string_val = super::j_path(json.as_bytes(), "string").unwrap().unwrap();
        assert!(string_val.is_string());
        assert_eq!(string_val.as_str().unwrap().as_str(), "hello");

        let number_val = super::j_path(json.as_bytes(), "number").unwrap().unwrap();
        assert!(number_val.is_number());

        let bool_val = super::j_path(json.as_bytes(), "boolean").unwrap().unwrap();
        assert!(bool_val.is_bool());

        let null_val = super::j_path(json.as_bytes(), "null_value")
            .unwrap()
            .unwrap();
        assert!(null_val.is_null());

        let array_val = super::j_path(json.as_bytes(), "array").unwrap().unwrap();
        assert!(array_val.is_array());

        let object_val = super::j_path(json.as_bytes(), "object").unwrap().unwrap();
        assert!(object_val.is_object());
    }

    #[test]
    fn test_complex_nested_structure() {
        let json = r#"{
            "company": {
                "name": "TechCorp",
                "departments": [
                    {
                        "name": "Engineering",
                        "employees": [
                            {"id": 1, "name": "John", "skills": ["Rust", "Python"]},
                            {"id": 2, "name": "Jane", "skills": ["Java", "Go"]}
                        ]
                    },
                    {
                        "name": "Marketing",
                        "employees": [
                            {"id": 3, "name": "Bob", "skills": ["SEO", "Social Media"]}
                        ]
                    }
                ]
            }
        }"#;

        // Test deep nested access
        let company_name = super::j_path(json.as_bytes(), "company.name")
            .unwrap()
            .unwrap();
        assert_eq!(company_name.as_str().unwrap().as_str(), "TechCorp");

        let dept_name = super::j_path(json.as_bytes(), "company.departments[0].name")
            .unwrap()
            .unwrap();
        assert_eq!(dept_name.as_str().unwrap().as_str(), "Engineering");

        let employee_name =
            super::j_path(json.as_bytes(), "company.departments[0].employees[1].name")
                .unwrap()
                .unwrap();
        assert_eq!(employee_name.as_str().unwrap().as_str(), "Jane");

        let skills = super::j_path(
            json.as_bytes(),
            "company.departments[0].employees[0].skills[0]",
        )
        .unwrap()
        .unwrap();
        assert_eq!(skills.as_str().unwrap().as_str(), "Rust");
    }

    #[test]
    fn test_unicode_and_special_chars() {
        let json = r#"{"user": {"name": "José", "email": "test@example.com", "message": "Hello, World! 你好"}}"#;

        let name = super::j_path(json.as_bytes(), "user.name")
            .unwrap()
            .unwrap();
        assert_eq!(name.as_str().unwrap().as_str(), "José");

        let email = super::j_path(json.as_bytes(), "user.email")
            .unwrap()
            .unwrap();
        assert_eq!(email.as_str().unwrap().as_str(), "test@example.com");

        let message = super::j_path(json.as_bytes(), "user.message")
            .unwrap()
            .unwrap();
        assert_eq!(message.as_str().unwrap().as_str(), "Hello, World! 你好");
    }

    #[test]
    fn test_whitespace_in_json() {
        let json = r#"{
            "key": "value",
            "nested": {
                "deep": "nested_value"
            }
        }"#;

        let result = super::j_path(json.as_bytes(), "nested.deep")
            .unwrap()
            .unwrap();
        assert_eq!(result.as_str().unwrap().as_str(), "nested_value");
    }

    #[test]
    fn test_empty_arrays_and_objects() {
        let json = r#"{"empty_array": [], "empty_object": {}}"#;

        let empty_array = super::j_path(json.as_bytes(), "empty_array")
            .unwrap()
            .unwrap();
        assert!(empty_array.is_array());

        let empty_object = super::j_path(json.as_bytes(), "empty_object")
            .unwrap()
            .unwrap();
        assert!(empty_object.is_object());
    }

    #[test]
    fn test_numeric_keys() {
        let json = r#"{"123": "numeric_key", "0": "zero_key"}"#;

        let numeric_key = super::j_path(json.as_bytes(), "123").unwrap().unwrap();
        assert_eq!(numeric_key.as_str().unwrap().as_str(), "numeric_key");

        let zero_key = super::j_path(json.as_bytes(), "0").unwrap().unwrap();
        assert_eq!(zero_key.as_str().unwrap().as_str(), "zero_key");
    }

    #[test]
    fn test_boolean_values() {
        let json = r#"{"true_val": true, "false_val": false}"#;

        let true_val = super::j_path(json.as_bytes(), "true_val").unwrap().unwrap();
        assert!(true_val.unwrap_as_bool().unwrap());

        let false_val = super::j_path(json.as_bytes(), "false_val")
            .unwrap()
            .unwrap();
        assert!(!false_val.unwrap_as_bool().unwrap());
    }

    #[test]
    fn test_null_values() {
        let json = r#"{"null_field": null}"#;

        let null_field = super::j_path(json.as_bytes(), "null_field")
            .unwrap()
            .unwrap();
        assert!(null_field.is_null());
    }

    #[test]
    fn test_large_numbers() {
        let json = r#"{"large_number": 1234567890123456789, "small_number": 0.001}"#;

        let large_number = super::j_path(json.as_bytes(), "large_number")
            .unwrap()
            .unwrap();
        assert!(large_number.is_number());

        let small_number = super::j_path(json.as_bytes(), "small_number")
            .unwrap()
            .unwrap();
        assert!(small_number.is_double());
    }

    #[test]
    fn test_very_large_json() {
        // Create a large JSON with many nested levels
        let json = r#"{"level0": {"level1": {"level2": {"level3": {"level4": {"level5": {"level6": {"level7": {"level8": {"level9": {"level10": "deep_value"}}}}}}}}}}}"#;

        // Test deep nesting
        let result = super::j_path(
            json.as_bytes(),
            "level0.level1.level2.level3.level4.level5.level6.level7.level8.level9.level10",
        )
        .unwrap()
        .unwrap();
        assert_eq!(result.as_str().unwrap().as_str(), "deep_value");
    }

    #[test]
    fn test_array_boundary_conditions() {
        let json = r#"{"array": [1, 2, 3]}"#;

        // Test first element
        let first = super::j_path(json.as_bytes(), "array[0]").unwrap().unwrap();
        assert!(first.is_number());

        // Test last element
        let last = super::j_path(json.as_bytes(), "array[2]").unwrap().unwrap();
        assert!(last.is_number());

        // Test out of bounds
        let out_of_bounds = super::j_path(json.as_bytes(), "array[3]").unwrap();
        assert!(out_of_bounds.is_none());
    }

    #[test]
    fn test_mixed_array_types() {
        let json = r#"{"mixed": [1, "string", true, null, {"key": "value"}, [1, 2, 3]]}"#;

        let number = super::j_path(json.as_bytes(), "mixed[0]").unwrap().unwrap();
        assert!(number.is_number());

        let string = super::j_path(json.as_bytes(), "mixed[1]").unwrap().unwrap();
        assert!(string.is_string());

        let boolean = super::j_path(json.as_bytes(), "mixed[2]").unwrap().unwrap();
        assert!(boolean.is_bool());

        let null_val = super::j_path(json.as_bytes(), "mixed[3]").unwrap().unwrap();
        assert!(null_val.is_null());

        let object = super::j_path(json.as_bytes(), "mixed[4]").unwrap().unwrap();
        assert!(object.is_object());

        let array = super::j_path(json.as_bytes(), "mixed[5]").unwrap().unwrap();
        assert!(array.is_array());
    }
}

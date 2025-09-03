use std::cell::RefCell;

use crate::json_reader::*;

pub struct SchemeDetection<'s> {
    src: &'s [u8],
    json_readers: RefCell<Vec<(usize, JsonValue, String, JsonFirstLineIterator<'s>)>>,
    array_items_to_yield: RefCell<Vec<String>>,
}

impl<'s> SchemeDetection<'s> {
    pub fn new(src: &'s [u8]) -> Self {
        Self {
            src,
            json_readers: RefCell::new(vec![(
                0,
                JsonValue {
                    start: 0,
                    end: src.len(),
                },
                String::new(),
                JsonFirstLineIterator::new(src),
            )]),

            array_items_to_yield: RefCell::new(vec![]),
        }
    }

    pub fn get_next(&'s self) -> Result<Option<String>, JsonParseError> {
        loop {
            {
                let mut array_items_to_yield = self.array_items_to_yield.borrow_mut();
                if array_items_to_yield.len() > 0 {
                    return Ok(Some(array_items_to_yield.remove(0)));
                }
            }

            let mut json_readers_access = self.json_readers.borrow_mut();

            let json_reader = json_readers_access.last();

            let Some(json_reader) = json_reader else {
                return Ok(None);
            };

            let (offset, src, src_prefix, json_reader) = json_reader;

            let next_item = json_reader.get_next();

            let Some(next_item) = next_item else {
                json_readers_access.pop();
                continue;
            };

            let next_item = next_item?;

            let item_key = next_item.0;
            let item_key = item_key.as_str()?;

            if next_item.1.is_object() {
                let src_offset = *offset;
                let src = src.clone();
                let json_pos = next_item.1.data;
                let new_prefix = format!("{}{}.", src_prefix, item_key.as_str());

                let sub_slice = &self.src[src.start + offset..src.end + offset];
                println!("sub_slice_pre: {}", String::from_utf8_lossy(sub_slice));
                let sub_slice = &sub_slice[json_pos.start..json_pos.end];

                println!("sub_slice: {}", String::from_utf8_lossy(sub_slice));

                json_readers_access.push((
                    src_offset + src.start,
                    json_pos,
                    new_prefix,
                    JsonFirstLineIterator::new(sub_slice),
                ));
                continue;
            }

            if next_item.1.is_array() {
                let array_iterator = next_item.1.unwrap_as_array()?;

                if let Some(first_item) = array_iterator.get_next() {
                    let first_item = first_item?;

                    if first_item.is_object() {
                        let sd = SchemeDetection::new(first_item.as_slice());

                        while let Some(next) = sd.get_next()? {
                            let item_to_push =
                                format!("{}{}[].{}", src_prefix, item_key.as_str(), next);
                            self.array_items_to_yield.borrow_mut().push(item_to_push);
                        }
                    } else {
                        let item_to_push = format!("{}{}[]", src_prefix, item_key.as_str());
                        self.array_items_to_yield.borrow_mut().push(item_to_push);
                    }
                }

                continue;
            }

            return Ok(Some(format!("{}{}", src_prefix, item_key.as_str())));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::j_path::SchemeDetection;

    #[test]
    fn test_basic_case() {
        let json =
            r#"{"key1": "value1", "key2": "value2", "object": { "key":"value", "key2":"value2" }}"#;
        let scheme_detection = SchemeDetection::new(json.as_bytes());
        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("key1".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("key2".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("object.key".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("object.key2".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_nested_objects() {
        let json = r#"{
            "level1_key1": "value1",
            "level1_object": {
                "level2_key1": "value2",
                "level2_object": {
                    "level3_key1": "value3",
                    "level3_key2": "value4"
                },
                "level2_key2": "value5"
            },
            "level1_key2": "value6"
        }"#;

        let scheme_detection = SchemeDetection::new(json.as_bytes());

        // Level 1 keys
        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("level1_key1".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("level1_object.level2_key1".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(
            result,
            Some("level1_object.level2_object.level3_key1".to_string())
        );

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(
            result,
            Some("level1_object.level2_object.level3_key2".to_string())
        );

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("level1_object.level2_key2".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("level1_key2".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_complex_nested_structure() {
        let json = r#"{
            "user": {
                "profile": {
                    "personal": {
                        "name": "John",
                        "age": 30
                    },
                    "contact": {
                        "email": "john@example.com",
                        "phone": "123-456-7890"
                    }
                },
                "settings": {
                    "theme": "dark",
                    "language": "en"
                }
            },
            "metadata": {
                "version": "1.0",
                "created": "2024-01-01"
            }
        }"#;

        let scheme_detection = SchemeDetection::new(json.as_bytes());

        // User profile personal
        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("user.profile.personal.name".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("user.profile.personal.age".to_string()));

        // User profile contact
        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("user.profile.contact.email".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("user.profile.contact.phone".to_string()));

        // User settings
        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("user.settings.theme".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("user.settings.language".to_string()));

        // Metadata
        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("metadata.version".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("metadata.created".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_array_of_objects() {
        let json = r#"{
            "users": [
                {
                    "id": 1,
                    "name": "Alice",
                    "profile": {
                        "age": 25,
                        "city": "New York"
                    }
                },
                {
                    "id": 2,
                    "name": "Bob",
                    "profile": {
                        "age": 30,
                        "city": "Los Angeles"
                    }
                }
            ],
            "metadata": {
                "total": 2,
                "version": "1.0"
            }
        }"#;

        let scheme_detection = SchemeDetection::new(json.as_bytes());

        // First user object fields
        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("users[].id".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("users[].name".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("users[].profile.age".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("users[].profile.city".to_string()));

        // Metadata fields
        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("metadata.total".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("metadata.version".to_string()));

        let result = scheme_detection.get_next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_array_of_values() {
        let json = r#"
        {
          "key":{"array": [1, 2, 3]}
        }"#;

        let scheme_detection = SchemeDetection::new(json.as_bytes());

        let result = scheme_detection.get_next().unwrap();
        assert_eq!(result, Some("key.array[]".to_string()));
    }
}

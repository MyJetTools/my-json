pub struct JPathReader<'s> {
    path: &'s str,
    index: Option<usize>,
}

impl<'s> JPathReader<'s> {
    pub fn new(path: &'s str) -> Self {
        let index = path.find('.');

        Self { path, index }
    }

    pub fn get_prop_name(&'s self) -> JPropName<'s> {
        match self.index {
            Some(index) => JPropName::new(&self.path[..index]),
            None => JPropName::new(self.path),
        }
    }

    pub fn get_next_level_path(&'s self) -> Option<&'s str> {
        let index = self.index?;

        let result = &self.path[index + 1..];
        Some(result)
    }
}

pub enum JPropName<'s> {
    Name(&'s str),
    ArrayAndIndex { j_prop_name: &'s str, index: usize },
    Array(&'s str),
}

impl<'s> JPropName<'s> {
    pub fn new(name: &'s str) -> Self {
        if name.ends_with(']') {
            // A trailing ']' only denotes an array segment when there is a matching '['. A key
            // that merely contains ']' (or an index that is not a number) must NOT panic - it is
            // treated as a literal property name instead.
            if let Some(open) = name.rfind('[') {
                if open == name.len() - 2 {
                    // "name[]" -> whole-array fan-out
                    return Self::Array(&name[..name.len() - 2]);
                }

                if let Ok(index) = name[open + 1..name.len() - 1].parse::<usize>() {
                    return Self::ArrayAndIndex {
                        j_prop_name: &name[..open],
                        index,
                    };
                }
            }

            // malformed segment: no '[' or a non-numeric index -> literal key
            return Self::Name(name);
        }

        Self::Name(name)
    }

    pub fn as_str(&self) -> &str {
        match self {
            JPropName::Name(name) => name,
            JPropName::ArrayAndIndex {
                j_prop_name,
                index: _,
            } => j_prop_name,
            JPropName::Array(name) => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_name_without_index() {
        let prop_name = JPropName::new("array[]");

        match prop_name {
            JPropName::Array(name) => assert_eq!(name, "array"),
            _ => panic!("Expected Array variant"),
        }
    }

    #[test]
    fn test_j_path_reader_simple_path() {
        let reader = JPathReader::new("simple");
        assert_eq!(reader.path, "simple");
        assert_eq!(reader.index, None);

        let prop_name = reader.get_prop_name();
        match prop_name {
            JPropName::Name(name) => assert_eq!(name, "simple"),
            _ => panic!("Expected Name variant"),
        }

        assert!(reader.get_next_level_path().is_none());
    }

    #[test]
    fn test_j_path_reader_nested_path() {
        let reader = JPathReader::new("parent.child");
        assert_eq!(reader.path, "parent.child");
        assert_eq!(reader.index, Some(6));

        let prop_name = reader.get_prop_name();
        match prop_name {
            JPropName::Name(name) => assert_eq!(name, "parent"),
            _ => panic!("Expected Name variant"),
        }

        let next_level = reader.get_next_level_path();
        assert_eq!(next_level, Some("child"));
    }

    #[test]
    fn test_j_path_reader_deep_nested_path() {
        let reader = JPathReader::new("level1.level2.level3");
        assert_eq!(reader.path, "level1.level2.level3");
        assert_eq!(reader.index, Some(6));

        let prop_name = reader.get_prop_name();
        match prop_name {
            JPropName::Name(name) => assert_eq!(name, "level1"),
            _ => panic!("Expected Name variant"),
        }

        let next_level = reader.get_next_level_path();
        assert_eq!(next_level, Some("level2.level3"));
    }

    #[test]
    fn test_j_prop_name_simple() {
        let prop_name = JPropName::new("simple");
        match prop_name {
            JPropName::Name(name) => assert_eq!(name, "simple"),
            _ => panic!("Expected Name variant"),
        }
    }

    #[test]
    fn test_j_prop_name_array() {
        let prop_name = JPropName::new("array[0]");
        match prop_name {
            JPropName::ArrayAndIndex { j_prop_name, index } => {
                assert_eq!(j_prop_name, "array");
                assert_eq!(index, 0);
            }
            _ => panic!("Expected Array variant"),
        }
    }

    #[test]
    fn test_j_prop_name_array_large_index() {
        let prop_name = JPropName::new("items[999]");
        match prop_name {
            JPropName::ArrayAndIndex { j_prop_name, index } => {
                assert_eq!(j_prop_name, "items");
                assert_eq!(index, 999);
            }
            _ => panic!("Expected Array variant"),
        }
    }

    #[test]
    fn test_j_prop_name_array_zero_index() {
        let prop_name = JPropName::new("list[0]");
        match prop_name {
            JPropName::ArrayAndIndex { j_prop_name, index } => {
                assert_eq!(j_prop_name, "list");
                assert_eq!(index, 0);
            }
            _ => panic!("Expected Array variant"),
        }
    }

    #[test]
    fn test_j_prop_name_bracket_without_open_is_literal_not_panic() {
        // ']' without a matching '[' must not panic; the whole thing is a literal key.
        let prop_name = JPropName::new("weird]key]");
        match prop_name {
            JPropName::Name(name) => assert_eq!(name, "weird]key]"),
            _ => panic!("Expected Name variant"),
        }
    }

    #[test]
    fn test_j_prop_name_non_numeric_index_is_literal_not_panic() {
        // A non-numeric index must not panic; treated as a literal key.
        let prop_name = JPropName::new("items[abc]");
        match prop_name {
            JPropName::Name(name) => assert_eq!(name, "items[abc]"),
            _ => panic!("Expected Name variant"),
        }

        // negative index (not a usize) is also literal, not a panic
        let prop_name = JPropName::new("items[-1]");
        match prop_name {
            JPropName::Name(name) => assert_eq!(name, "items[-1]"),
            _ => panic!("Expected Name variant"),
        }
    }

    #[test]
    fn test_get_value_with_bracket_in_key_does_not_panic() {
        // End-to-end: a hostile path with a stray ']' resolves without panicking.
        let json = br#"{"a":1}"#;
        let result = crate::j_path::get_value(json, "weird]key");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_j_path_reader_edge_cases() {
        // Test with empty string
        let reader = JPathReader::new("");
        assert_eq!(reader.path, "");
        assert_eq!(reader.index, None);

        // Test with single dot
        let reader = JPathReader::new(".");
        assert_eq!(reader.path, ".");
        assert_eq!(reader.index, Some(0));

        // Test with multiple consecutive dots
        let reader = JPathReader::new("a..b");
        assert_eq!(reader.path, "a..b");
        assert_eq!(reader.index, Some(1));

        let prop_name = reader.get_prop_name();
        match prop_name {
            JPropName::Name(name) => assert_eq!(name, "a"),
            _ => panic!("Expected Name variant"),
        }

        let next_level = reader.get_next_level_path();
        assert_eq!(next_level, Some(".b"));
    }
}

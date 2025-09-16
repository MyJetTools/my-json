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
            let index = name.rfind('[').unwrap();

            if index == name.len() - 2 {
                return Self::Array(&name[..name.len() - 2]);
            }

            let j_prop_name = &name[..index];

            let index = name[index + 1..name.len() - 1].parse::<usize>().unwrap();

            return Self::ArrayAndIndex { j_prop_name, index };
        }

        Self::Name(name)
    }

    pub fn as_str(&self) -> &str {
        match self {
            JPropName::Name(name) => name,
            JPropName::ArrayAndIndex { j_prop_name, index } => j_prop_name,
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

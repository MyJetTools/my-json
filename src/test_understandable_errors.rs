use crate::json_reader::{JsonArrayIterator, JsonFirstLineIterator};
#[test]
fn test_empty_string_as_json() {
    let json = "";

    let first_line_reader = JsonFirstLineIterator::new(json.as_bytes());

    let item = first_line_reader.get_next().unwrap();
    assert_eq!(item.is_err(), true);
}

#[test]
fn test_empty_string_as_array_iterator() {
    let json = "";

    let array_iterator = JsonArrayIterator::new(json.as_bytes());

    assert!(array_iterator.is_err())
}

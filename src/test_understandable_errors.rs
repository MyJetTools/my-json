use crate::json_reader::array_iterator::JsonArrayIterator;
use rust_extensions::array_of_bytes_iterator::SliceIterator;
#[test]
fn test_empty_string_as_json() {
    use crate::json_reader::JsonFirstLineReader;

    let json = "";

    let slice_iterator = SliceIterator::new(json.as_bytes());

    let first_line_reader = JsonFirstLineReader::new(slice_iterator);

    while let Some(item) = first_line_reader.get_next() {
        println!("{:?}", item);

        if item.is_err() {
            break;
        }
    }
}

#[test]
fn test_empty_string_as_array_iterator() {
    let json = "";

    let slice_iterator = SliceIterator::new(json.as_bytes());

    let array_iterator = JsonArrayIterator::new(slice_iterator);

    println!("Array iterator: {:?}", array_iterator);
}

use my_json::json_reader::JsonArrayIterator;

#[test]
fn probe_unwrap_as_number_panics_on_double() {
    let json = br#"{"v":1.5}"#;
    let v = my_json::j_path::get_value(json, "v").unwrap().unwrap();
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = v.unwrap_as_number();
    }));
    assert!(r.is_err(), "expected panic on Double via unwrap_as_number");
}

#[test]
fn probe_unwrap_as_number_panics_on_null() {
    let json = br#"{"v":null}"#;
    let v = my_json::j_path::get_value(json, "v").unwrap().unwrap();
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = v.unwrap_as_number();
    }));
    assert!(r.is_err(), "expected panic on null via unwrap_as_number");
}

#[test]
fn probe_into_array_iterator_panics_on_empty() {
    let r = std::panic::catch_unwind(|| {
        let _it: JsonArrayIterator = b"".as_slice().into();
    });
    assert!(r.is_err(), "expected panic building JsonArrayIterator from empty bytes");
}

#[test]
fn probe_into_array_iterator_panics_on_whitespace() {
    let r = std::panic::catch_unwind(|| {
        let _it: JsonArrayIterator = b"   ".as_slice().into();
    });
    assert!(r.is_err(), "expected panic building JsonArrayIterator from whitespace");
}

#[test]
fn probe_j_update_panics_on_malformed() {
    let r = std::panic::catch_unwind(|| {
        let _ = my_json::j_path::j_update("{\"a\"", "x", "y");
    });
    assert!(r.is_err(), "expected panic in j_update on malformed json (next.unwrap())");
}

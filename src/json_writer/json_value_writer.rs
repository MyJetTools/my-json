use std::collections::HashMap;
use std::fmt::Write;

use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

use super::JsonValueWriter;

// Integer writers: format straight into the destination buffer via `write!` instead of allocating
// an intermediate `String` through `to_string()` on every value.
macro_rules! impl_json_value_writer_for_integer {
    ($($t:ty),* $(,)?) => {
        $(
            impl JsonValueWriter for $t {
                const IS_ARRAY: bool = false;
                fn write(&self, dest: &mut String) {
                    let _ = write!(dest, "{}", self);
                }
            }

            impl JsonValueWriter for Option<$t> {
                const IS_ARRAY: bool = false;
                fn write(&self, dest: &mut String) {
                    match self {
                        Some(v) => {
                            let _ = write!(dest, "{}", v);
                        }
                        None => dest.push_str("null"),
                    }
                }
            }
        )*
    };
}

// `i128` / `u128` are here so the writer covers every integer the reader can read back: JSON has
// no integer width limit, and a `u128` renders as plain digits like any other. Without them a
// `u128` field would have no wire form at all, while `JsonValueReader` happily reads one.
impl_json_value_writer_for_integer!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
);

// Float writers: format straight into the buffer, and emit `null` for the non-finite values
// (`NaN`, `+Infinity`, `-Infinity`) - those have no JSON representation and `to_string()` would
// otherwise produce the invalid tokens `NaN` / `inf`. This matches `serde_json`.
macro_rules! impl_json_value_writer_for_float {
    ($($t:ty),* $(,)?) => {
        $(
            impl JsonValueWriter for $t {
                const IS_ARRAY: bool = false;
                fn write(&self, dest: &mut String) {
                    if self.is_finite() {
                        let _ = write!(dest, "{}", self);
                    } else {
                        dest.push_str("null");
                    }
                }
            }

            impl JsonValueWriter for Option<$t> {
                const IS_ARRAY: bool = false;
                fn write(&self, dest: &mut String) {
                    match self {
                        Some(v) if v.is_finite() => {
                            let _ = write!(dest, "{}", v);
                        }
                        // both `None` and a non-finite value serialise as `null`
                        _ => dest.push_str("null"),
                    }
                }
            }
        )*
    };
}

impl_json_value_writer_for_float!(f32, f64);

#[cfg(feature = "decimal")]
impl JsonValueWriter for rust_decimal::Decimal {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        let _ = write!(dest, "{}", self);
    }
}

// Mirrors `Option<DateTimeAsMicroseconds>`: without this a `Option<Decimal>` field has no wire
// form, even though the reader can read one back.
#[cfg(feature = "decimal")]
impl JsonValueWriter for Option<rust_decimal::Decimal> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        match self {
            Some(v) => {
                let _ = write!(dest, "{}", v);
            }
            None => dest.push_str("null"),
        }
    }
}

impl JsonValueWriter for bool {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        if *self {
            dest.push_str("true");
        } else {
            dest.push_str("false");
        }
    }
}

impl JsonValueWriter for Option<bool> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        match self {
            Some(v) => bool::write(v, dest),
            None => dest.push_str("null"),
        }
    }
}

// A JSON date-time is written as an RFC-3339 string in the canonical UTC form
// (e.g. "2021-04-25T17:30:03.000000Z") - the same rendering `DateTimeAsMicroseconds`'s `Serialize`
// produces, so this type has exactly one spelling on the wire regardless of who writes it.
//
// `to_rfc3339_utc()` (not `to_rfc3339()`): the `Z` suffix plus fixed 6-digit microseconds give a
// constant width, so lexicographic string order matches chronological order. `to_rfc3339()` renders
// the zero offset as `+00:00` and varies the fractional width (0/3/6/9 digits), losing that.
//
// This is the format the read side resolves through `JsonValueRef::as_date_time` /
// `try_get_date_time` (the string branch), so a written value round-trips back to the same
// `DateTimeAsMicroseconds`. RFC-3339 (rather than a raw microseconds number) keeps the wire value
// human-readable and unambiguous - the numeric read path guesses the unit from magnitude.
impl JsonValueWriter for DateTimeAsMicroseconds {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        write_string(dest, self.to_rfc3339_utc().as_str());
    }
}

impl JsonValueWriter for Option<DateTimeAsMicroseconds> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        match self {
            Some(v) => write_string(dest, v.to_rfc3339_utc().as_str()),
            None => dest.push_str("null"),
        }
    }
}

impl JsonValueWriter for String {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push('"');
        crate::json_string_value::write_escaped_json_string_value(self, dest);
        dest.push('"');
    }
}

impl<'s> JsonValueWriter for StrOrString<'s> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push('"');
        crate::json_string_value::write_escaped_json_string_value(self.as_str(), dest);
        dest.push('"');
    }
}

impl<'s> JsonValueWriter for &'s StrOrString<'s> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push('"');
        crate::json_string_value::write_escaped_json_string_value(self.as_str(), dest);
        dest.push('"');
    }
}

impl JsonValueWriter for Option<String> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        match self {
            Some(v) => write_string(dest, v),
            None => dest.push_str("null"),
        }
    }
}

impl<'s> JsonValueWriter for &'s str {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        write_string(dest, self)
    }
}

impl<'s> JsonValueWriter for Option<&'s str> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        match self {
            Some(v) => write_string(dest, v),
            None => dest.push_str("null"),
        }
    }
}

impl<'s> JsonValueWriter for &'s String {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        write_string(dest, self)
    }
}

impl<'s> JsonValueWriter for Option<&'s String> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        match self {
            Some(v) => write_string(dest, v),
            None => dest.push_str("null"),
        }
    }
}

pub enum RawJsonObject<'s> {
    AsString(String),
    AsStr(&'s str),
}

impl<'s> RawJsonObject<'s> {
    pub fn new(value: String) -> Self {
        RawJsonObject::AsString(value)
    }

    pub fn as_str(&'s self) -> &'s str {
        match self {
            RawJsonObject::AsString(vec) => vec,
            RawJsonObject::AsStr(slice) => slice,
        }
    }
}

impl<'s> Into<RawJsonObject<'s>> for Vec<u8> {
    fn into(self) -> RawJsonObject<'s> {
        RawJsonObject::AsString(String::from_utf8(self).unwrap())
    }
}

impl<'s> Into<RawJsonObject<'s>> for String {
    fn into(self) -> RawJsonObject<'s> {
        RawJsonObject::AsString(self)
    }
}

impl<'s> JsonValueWriter for RawJsonObject<'s> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.as_str());
    }
}

pub struct JsonNullValue;

impl JsonValueWriter for JsonNullValue {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str("null");
    }
}

pub struct EmptyJsonArray;

impl JsonValueWriter for EmptyJsonArray {
    const IS_ARRAY: bool = true;
    fn write(&self, dest: &mut String) {
        dest.push_str("");
    }
}

// Writes the comma-separated body of an array of `T`. Each element that is itself an array
// (`T::IS_ARRAY`) is wrapped in its own `[` `]` - without this a `Vec<Vec<T>>` (or a `Vec` of any
// array-valued element) would collapse into a single flat array, e.g. `[[1,2],[3]]` -> `[1,2,3]`.
fn write_array_body<T: JsonValueWriter>(dest: &mut String, items: &[T]) {
    for (no, itm) in items.iter().enumerate() {
        if no > 0 {
            dest.push(',');
        }
        if T::IS_ARRAY {
            dest.push('[');
        }
        itm.write(dest);
        if T::IS_ARRAY {
            dest.push(']');
        }
    }
}

impl<T: JsonValueWriter> JsonValueWriter for Vec<T> {
    const IS_ARRAY: bool = true;
    fn write(&self, dest: &mut String) {
        write_array_body(dest, self.as_slice());
    }
}

impl<'s, T: JsonValueWriter> JsonValueWriter for &'s [T] {
    const IS_ARRAY: bool = true;
    fn write(&self, dest: &mut String) {
        write_array_body(dest, *self);
    }
}

impl<'s, T: JsonValueWriter> JsonValueWriter for &'s Vec<T> {
    const IS_ARRAY: bool = true;
    fn write(&self, dest: &mut String) {
        write_array_body(dest, self.as_slice());
    }
}

// A `HashMap<String, V>` serialises as a JSON object - each entry becomes a `"key":value` pair.
// `IS_ARRAY` is false because, like `JsonObjectWriter`, `write` emits its own surrounding `{` `}`,
// so `JsonObjectWriter::write` must not wrap it again. A value that is itself an array
// (`V::IS_ARRAY`) is wrapped in `[` `]`, mirroring `JsonObjectWriter::write`, so a
// `HashMap<String, Vec<T>>` keeps each value as its own nested array rather than a flat run.
// This impl is what lets derived writers (e.g. `MyHttpObjectStructure`) accept `HashMap` fields.
// Entry order follows `HashMap` iteration order and is therefore not deterministic.
impl<V: JsonValueWriter> JsonValueWriter for HashMap<String, V> {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push('{');
        for (no, (key, value)) in self.iter().enumerate() {
            if no > 0 {
                dest.push(',');
            }
            write_string(dest, key);
            dest.push(':');
            if V::IS_ARRAY {
                dest.push('[');
            }
            value.write(dest);
            if V::IS_ARRAY {
                dest.push(']');
            }
        }
        dest.push('}');
    }
}

fn write_string(out: &mut String, value: &str) {
    out.push('"');
    crate::json_string_value::write_escaped_json_string_value(value, out);
    out.push('"');
}

#[cfg(test)]
mod test {
    use crate::json_writer::{JsonArrayWriter, JsonObjectWriter};

    #[test]
    fn test_array_of_numbers() {
        let a = vec![1, 2, 3];

        let array = JsonArrayWriter::new();
        let result = array.write(a).build();

        assert_eq!(result, "[1,2,3]");
    }

    #[test]
    fn test_array_of_strings() {
        let a = vec!["1", "2", "3"];

        let array = JsonArrayWriter::new();
        let result = array.write(a).build();

        assert_eq!(result, r#"["1","2","3"]"#);
    }

    #[test]
    fn test_write_array_to_object() {
        let a = vec!["1", "2", "3"];
        let result = JsonObjectWriter::new().write("test", &a).build();

        assert_eq!(result, r#"{"test":["1","2","3"]}"#);
    }

    #[test]
    fn test_write_array_to_object_as_iterator() {
        let result = JsonObjectWriter::new()
            .write_iter("test", ["1", "2", "3"].into_iter())
            .build();

        assert_eq!(result, r#"{"test":["1","2","3"]}"#);
    }

    #[test]
    fn test_nested_vec_of_vec_keeps_inner_brackets() {
        // Regression: a Vec<Vec<T>> must NOT collapse into a single flat array.
        let nested = vec![vec![1, 2], vec![3]];
        let result = JsonObjectWriter::new().write("m", nested).build();
        assert_eq!(result, r#"{"m":[[1,2],[3]]}"#);
    }

    #[test]
    fn test_nested_vec_via_array_writer() {
        let nested = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        let result = JsonArrayWriter::new().write(nested).build();
        assert_eq!(result, "[[1,2],[3,4],[5,6]]");
    }

    #[test]
    fn test_triple_nested_vec() {
        let nested = vec![vec![vec![1], vec![2, 3]], vec![vec![4]]];
        let result = JsonObjectWriter::new().write("m", nested).build();
        assert_eq!(result, r#"{"m":[[[1],[2,3]],[[4]]]}"#);
    }

    #[test]
    fn test_non_finite_floats_become_null() {
        let result = JsonObjectWriter::new()
            .write("nan", f64::NAN)
            .write("pinf", f64::INFINITY)
            .write("ninf", f64::NEG_INFINITY)
            .write("nan32", f32::NAN)
            .write("inf32", f32::INFINITY)
            .build();

        assert_eq!(
            result,
            r#"{"nan":null,"pinf":null,"ninf":null,"nan32":null,"inf32":null}"#
        );
    }

    #[test]
    fn test_finite_floats_are_written() {
        let result = JsonObjectWriter::new()
            .write("a", 4.5f64)
            .write("b", -1.25f32)
            .write("c", 0.0f64)
            .build();

        assert_eq!(result, r#"{"a":4.5,"b":-1.25,"c":0}"#);
    }

    #[test]
    fn test_option_non_finite_float_is_null() {
        let some_nan: Option<f64> = Some(f64::NAN);
        let some_val: Option<f64> = Some(2.5);
        let none_val: Option<f64> = None;

        let result = JsonObjectWriter::new()
            .write("a", some_nan)
            .write("b", some_val)
            .write("c", none_val)
            .build();

        assert_eq!(result, r#"{"a":null,"b":2.5,"c":null}"#);
    }

    #[test]
    fn test_date_time_writes_rfc3339_and_round_trips() {
        use rust_extensions::date_time::DateTimeAsMicroseconds;

        let dt = DateTimeAsMicroseconds::from_str("2021-04-25T17:30:03.000Z").unwrap();

        let json = JsonObjectWriter::new().write("ts", dt).build();

        // canonical UTC form: `Z` suffix, fixed 6-digit microseconds (NOT `+00:00`)
        assert_eq!(json, r#"{"ts":"2021-04-25T17:30:03.000000Z"}"#);

        // read it back through the value reader - the wire format must match the read path
        let read = crate::j_path::get_value(json.as_bytes(), "ts")
            .unwrap()
            .unwrap();
        let back = read.try_get_date_time().unwrap();

        assert_eq!(back.unix_microseconds, dt.unix_microseconds);
    }

    /// The wire form is pinned exactly: canonical UTC RFC-3339, `Z` suffix, always 6 fractional
    /// digits. This is the same spelling `DateTimeAsMicroseconds`'s `Serialize` produces, so the
    /// type has one representation on the wire whoever writes it.
    #[test]
    fn test_date_time_wire_format_is_exact() {
        use rust_extensions::date_time::DateTimeAsMicroseconds;

        for (raw, expected) in [
            // real microseconds are kept
            ("2024-01-02T03:04:05.123456Z", r#"{"ts":"2024-01-02T03:04:05.123456Z"}"#),
            // a whole second is padded to 6 digits (not dropped, as AutoSi would)
            ("2024-01-02T03:04:05.000Z", r#"{"ts":"2024-01-02T03:04:05.000000Z"}"#),
            // milliseconds are padded to 6 digits (not left at 3)
            ("2021-04-25T17:30:03.500Z", r#"{"ts":"2021-04-25T17:30:03.500000Z"}"#),
            // epoch
            ("1970-01-01T00:00:00.000Z", r#"{"ts":"1970-01-01T00:00:00.000000Z"}"#),
        ] {
            let dt = DateTimeAsMicroseconds::from_str(raw).unwrap();
            let json = JsonObjectWriter::new().write("ts", dt).build();

            assert_eq!(json, expected, "wire format for {}", raw);
        }
    }

    #[test]
    fn test_option_date_time_wire_format_is_exact() {
        use rust_extensions::date_time::DateTimeAsMicroseconds;

        let dt = DateTimeAsMicroseconds::from_str("2024-01-02T03:04:05.123456Z").unwrap();
        let some: Option<DateTimeAsMicroseconds> = Some(dt);
        let none: Option<DateTimeAsMicroseconds> = None;

        let json = JsonObjectWriter::new().write("a", some).write("b", none).build();

        assert_eq!(
            json,
            r#"{"a":"2024-01-02T03:04:05.123456Z","b":null}"#
        );
    }

    #[test]
    fn test_date_time_keeps_real_microseconds() {
        use rust_extensions::date_time::DateTimeAsMicroseconds;

        let dt = DateTimeAsMicroseconds::from_str("2024-01-02T03:04:05.123456Z").unwrap();
        let json = JsonObjectWriter::new().write("ts", dt).build();

        assert_eq!(json, r#"{"ts":"2024-01-02T03:04:05.123456Z"}"#);

        // and the microseconds survive a full round-trip
        let back = crate::j_path::get_value(json.as_bytes(), "ts")
            .unwrap()
            .unwrap()
            .try_get_date_time()
            .unwrap();
        assert_eq!(back.unix_microseconds, dt.unix_microseconds);
    }

    /// Fixed width + `Z` => lexicographic string order == chronological order.
    /// This is what the floating-width `to_rfc3339()` (AutoSi) could not guarantee.
    #[test]
    fn test_date_time_strings_sort_chronologically() {
        use rust_extensions::date_time::DateTimeAsMicroseconds;

        // deliberately mixed fractional magnitudes - these are exactly the values whose
        // rendered width used to vary (0 / 3 / 6 digits)
        let mut moments: Vec<DateTimeAsMicroseconds> = [
            "2024-01-02T03:04:05.000000Z",
            "2024-01-02T03:04:05.000001Z",
            "2024-01-02T03:04:05.001Z",
            "2024-01-02T03:04:05.500Z",
            "2024-01-02T03:04:06.000000Z",
            "2023-12-31T23:59:59.999999Z",
            "1970-01-01T00:00:00.000000Z",
        ]
        .iter()
        .map(|s| DateTimeAsMicroseconds::from_str(s).unwrap())
        .collect();

        moments.sort_by_key(|m| m.unix_microseconds);
        let by_time: Vec<String> = moments.iter().map(|m| m.to_rfc3339_utc()).collect();

        let mut by_string = by_time.clone();
        by_string.sort();

        assert_eq!(
            by_time, by_string,
            "lexicographic order of the written strings must match chronological order"
        );
    }

    #[test]
    fn test_hash_map_single_entry_renders_as_object() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("key".to_string(), "value");

        let result = JsonObjectWriter::new().write("m", map).build();
        assert_eq!(result, r#"{"m":{"key":"value"}}"#);
    }

    #[test]
    fn test_hash_map_key_is_escaped() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(r#"he said "hi""#.to_string(), 1);

        let result = JsonObjectWriter::new().write("m", map).build();
        assert_eq!(result, r#"{"m":{"he said \"hi\"":1}}"#);
    }

    #[test]
    fn test_hash_map_of_arrays_keeps_nested_brackets() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("nums".to_string(), vec![1, 2, 3]);

        let result = JsonObjectWriter::new().write("m", map).build();
        assert_eq!(result, r#"{"m":{"nums":[1,2,3]}}"#);
    }

    #[test]
    fn test_hash_map_multi_entry_round_trips() {
        use std::collections::HashMap;

        // Order is not deterministic, so read each value back by path instead of
        // asserting on the raw string.
        let mut map = HashMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        map.insert("c".to_string(), 3);

        let json = JsonObjectWriter::new().write("m", map).build();

        for (key, expected) in [("a", 1), ("b", 2), ("c", 3)] {
            let read = crate::j_path::get_value(json.as_bytes(), format!("m.{}", key).as_str())
                .unwrap()
                .unwrap();
            assert_eq!(read.unwrap_as_number().unwrap(), Some(expected));
        }
    }

    #[test]
    fn test_empty_hash_map_renders_empty_object() {
        use std::collections::HashMap;

        let map: HashMap<String, i32> = HashMap::new();
        let result = JsonObjectWriter::new().write("m", map).build();
        assert_eq!(result, r#"{"m":{}}"#);
    }

    #[test]
    fn test_option_date_time() {
        use rust_extensions::date_time::DateTimeAsMicroseconds;

        let dt = DateTimeAsMicroseconds::from_str("2021-04-25T17:30:03.000Z").unwrap();
        let some: Option<DateTimeAsMicroseconds> = Some(dt);
        let none: Option<DateTimeAsMicroseconds> = None;

        let result = JsonObjectWriter::new()
            .write("a", some)
            .write("b", none)
            .build();

        assert!(result.contains(r#""b":null"#));

        let read = crate::j_path::get_value(result.as_bytes(), "a")
            .unwrap()
            .unwrap();
        assert_eq!(
            read.try_get_date_time().unwrap().unix_microseconds,
            dt.unix_microseconds
        );
    }
}

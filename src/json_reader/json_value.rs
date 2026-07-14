use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

use super::{JsonArrayIterator, JsonFirstLineIterator, JsonParseError};

pub trait AsJsonSlice {
    fn as_slice(&self) -> &[u8];
}

impl<'s> AsJsonSlice for &'s [u8] {
    fn as_slice(&self) -> &[u8] {
        self
    }
}

impl<'s> AsJsonSlice for &'s str {
    fn as_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}
#[derive(Clone, Debug)]
pub struct JsonValue {
    pub start: usize,
    pub end: usize,
}

impl JsonValue {
    pub fn new(start: usize, end: usize) -> Self {
        return Self { start, end };
    }

    pub fn unwrap_value<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<UnwrappedJsonValue<'s>, JsonParseError> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();

        if crate::json_utils::is_null(slice) {
            return Ok(UnwrappedJsonValue::Null);
        }

        if let Some(value) = crate::json_utils::as_bool_value(slice) {
            return Ok(UnwrappedJsonValue::Boolean(value));
        }

        match crate::json_utils::is_number(slice) {
            crate::json_utils::NumberType::NaN => {}
            crate::json_utils::NumberType::Number => {
                return Ok(UnwrappedJsonValue::Number(convert_to_i64(slice)?));
            }
            crate::json_utils::NumberType::Double => {
                return Ok(UnwrappedJsonValue::Double(convert_to_f64(slice)?));
            }
        }

        if crate::json_utils::is_array(slice) {
            let json_array_iterator = JsonArrayIterator::new(slice)?;
            return Ok(UnwrappedJsonValue::Array(json_array_iterator));
        }

        if crate::json_utils::is_object(slice) {
            return Ok(UnwrappedJsonValue::Object(JsonFirstLineIterator::new(
                slice,
            )));
        }

        return Ok(UnwrappedJsonValue::String(convert_to_utf8(slice)?));
    }

    pub fn clone_with_offset(&self, offset: usize) -> Self {
        Self {
            start: self.start + offset,
            end: self.end + offset,
        }
    }
    /*
       pub fn new(value: &'s [u8]) -> Result<JsonValue<'s>, JsonParseError> {
           if crate::json_utils::is_null(value) {
               return Ok(JsonValue::Null);
           }

           if let Some(value) = crate::json_utils::is_bool(value) {
               return Ok(JsonValue::Boolean(value));
           }

           match crate::json_utils::is_number(value) {
               crate::json_utils::NumberType::NaN => {}
               crate::json_utils::NumberType::Number => {
                   return Ok(JsonValue::Number(convert_to_utf8(value)?));
               }
               crate::json_utils::NumberType::Double => {
                   return Ok(JsonValue::Double(convert_to_utf8(value)?));
               }
           }

           if value[0] == consts::OPEN_ARRAY {
               return Ok(JsonValue::Array(value));
           }

           if value[0] == consts::OPEN_BRACKET {
               return Ok(JsonValue::Object(value));
           }

           return Ok(JsonValue::String(convert_to_utf8(value)?));
       }
    */

    pub fn is_null<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> bool {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        crate::json_utils::is_null(slice)
    }

    pub fn unwrap_as_number<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<Option<i64>, JsonParseError> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        match crate::json_utils::is_number(slice) {
            crate::json_utils::NumberType::NaN => {}
            crate::json_utils::NumberType::Number => {
                return Ok(Some(convert_to_i64(slice)?));
            }
            crate::json_utils::NumberType::Double => {}
        }

        panic!("Json value is not a number")
    }

    /*
           pub fn get_value(&self) -> UnwrappedValue {
               match self {
                   JsonValue::Null => UnwrappedValue::Null,
                   JsonValue::String(value) => UnwrappedValue::String(value),
                   JsonValue::Number(value) => UnwrappedValue::Number(value.parse().unwrap()),
                   JsonValue::Double(value) => UnwrappedValue::Double(value.parse().unwrap()),
                   JsonValue::Boolean(value) => UnwrappedValue::Boolean(*value),
                   JsonValue::Array(slice) => {
                       let slice_iterator = SliceIterator::new(slice);
                       UnwrappedValue::Array(JsonArrayIterator::new(slice_iterator))
                   }
                   JsonValue::Object(slice) => {
                       let slice_iterator = SliceIterator::new(slice);
                       UnwrappedValue::Object(JsonFirstLineReader::new(slice_iterator))
                   }
               }
           }

           pub fn unwrap_as_double(&self) -> Option<f64> {
               match self {
                   JsonValue::Double(src) => {
                       let value = src.parse().unwrap();
                       Some(value)
                   }
                   JsonValue::Number(src) => {
                       let value = src.parse().unwrap();
                       Some(value)
                   }
                   JsonValue::Null => None,
                   _ => {
                       panic!("Json value is not double")
                   }
               }
           }

        pub fn unwrap_as_bool(&self) -> Option<bool> {
            match self {
                JsonValue::Boolean(src) => Some(*src),
                JsonValue::Null => None,
                _ => {
                    panic!("Json value is not boolean")
                }
            }
        }

        pub fn unwrap_as_array(&self) -> Option<JsonArrayIterator<SliceIterator>> {
            match self {
                JsonValue::Array(src) => Some(JsonArrayIterator::new(SliceIterator::new(src))),
                JsonValue::Null => None,
                _ => {
                    panic!("Json value is not array")
                }
            }
        }

        pub fn unwrap_as_object(&self) -> Option<JsonFirstLineReader<SliceIterator>> {
            match self {
                JsonValue::Object(src) => Some(JsonFirstLineReader::new(SliceIterator::new(src))),
                JsonValue::Null => None,
                _ => {
                    panic!("Json value is not array")
                }
            }
        }
    */
    pub fn is_object<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> bool {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        crate::json_utils::is_object(slice)
    }

    pub fn unwrap_as_object<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<JsonFirstLineIterator<'s>, JsonParseError> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();

        if crate::json_utils::is_object(slice) {
            return Ok(JsonFirstLineIterator::new(slice));
        }

        Err(JsonParseError::new(
            "Json value is not an object".to_string(),
        ))
    }

    pub fn unwrap_as_bool<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> Option<bool> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        crate::json_utils::as_bool_value(slice)
    }

    pub fn is_bool<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> bool {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        crate::json_utils::is_bool(slice)
    }

    pub fn is_string<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> bool {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        crate::json_utils::is_string(slice)
    }

    pub fn is_number<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> bool {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        match crate::json_utils::is_number(slice) {
            crate::json_utils::NumberType::NaN => {}
            crate::json_utils::NumberType::Number => return true,
            crate::json_utils::NumberType::Double => {}
        }
        false
    }

    pub fn is_double<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> bool {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        match crate::json_utils::is_number(slice) {
            crate::json_utils::NumberType::NaN => {}
            crate::json_utils::NumberType::Number => {}
            crate::json_utils::NumberType::Double => return true,
        }
        false
    }

    pub fn unwrap_as_double<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<Option<f64>, JsonParseError> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();

        if crate::json_utils::is_null(slice) {
            return Ok(None);
        }

        match crate::json_utils::is_number(slice) {
            crate::json_utils::NumberType::NaN => {
                return Ok(None);
            }
            crate::json_utils::NumberType::Number => {
                let value = convert_to_f64(slice)?;
                return Ok(Some(value));
            }
            crate::json_utils::NumberType::Double => {
                let value = convert_to_f64(slice)?;
                return Ok(Some(value));
            }
        }
    }

    pub fn is_array<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> bool {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();
        crate::json_utils::is_array(slice)
    }

    pub fn as_bytes<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> &'s [u8] {
        as_json_slice.as_slice()[self.start..self.end].as_ref()
    }

    pub fn unwrap_as_array<'s>(
        &self,
        as_json_slice: &'s impl AsJsonSlice,
    ) -> Result<JsonArrayIterator<'s>, JsonParseError> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();

        if crate::json_utils::is_array(slice) {
            return JsonArrayIterator::new(slice);
        }

        Err(JsonParseError::new(
            "Json value is not an array".to_string(),
        ))
    }

    /// Returns the value as a string, fully **resolving** JSON escapes (`\n`, `\t`, `\"`, `\\`,
    /// `\/`, `\uXXXX` including surrogate pairs) for a quoted string, and yielding `""` for an
    /// empty JSON string `""`. A non-string scalar (e.g. a number) is returned verbatim.
    ///
    /// Returns `None` for JSON `null` or when the underlying bytes are not valid UTF-8 - it never
    /// panics on hostile input.
    pub fn as_str<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> Option<StrOrString<'s>> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();

        if crate::json_utils::is_null(slice) {
            return None;
        }

        if let Some(value) = crate::json_utils::try_get_string_value(slice) {
            return Some(value);
        }

        // Not a quoted string (a number / bare token) - return it verbatim, but never panic on
        // non-UTF-8 bytes coming from an untrusted payload.
        let result = std::str::from_utf8(slice).ok()?;

        Some(result.into())
    }

    /// Returns the value with a single pair of matching surrounding quotes stripped, WITHOUT
    /// resolving JSON escape sequences (the result borrows from the input, so it can not allocate
    /// a de-escaped copy). For a value that needs escapes resolved use [`Self::as_str`].
    ///
    /// An empty JSON string `""` yields `Some("")`. Returns `None` for JSON `null` or non-UTF-8
    /// bytes; it never panics on hostile input.
    pub fn as_unescaped_str<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> Option<&'s str> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();

        if crate::json_utils::is_null(slice) {
            return None;
        }

        // Strip a single pair of matching surrounding quotes when both are present; this also
        // yields an empty slice (and thus `""`) for the empty string `""`.
        let has_quotes = slice.len() >= 2
            && ((slice[0] == crate::consts::DOUBLE_QUOTE
                && slice[slice.len() - 1] == crate::consts::DOUBLE_QUOTE)
                || (slice[0] == crate::consts::SINGLE_QUOTE
                    && slice[slice.len() - 1] == crate::consts::SINGLE_QUOTE));

        let inner = if has_quotes {
            slice[1..slice.len() - 1].as_ref()
        } else {
            slice
        };

        std::str::from_utf8(inner).ok()
    }

    pub fn as_raw_str<'s>(&self, as_json_slice: &'s impl AsJsonSlice) -> Option<&'s str> {
        let slice = as_json_slice.as_slice()[self.start..self.end].as_ref();

        // Never panic on non-UTF-8 bytes from an untrusted payload.
        std::str::from_utf8(slice).ok()
    }

    pub fn as_date_time<'s>(
        &self,
        as_json_slice: &impl AsJsonSlice,
    ) -> Option<DateTimeAsMicroseconds> {
        let as_str = self.as_str(as_json_slice)?;
        DateTimeAsMicroseconds::from_str(as_str.as_str())
    }
}

pub enum UnwrappedJsonValue<'s> {
    Null,
    String(&'s str),
    Number(i64),
    Double(f64),
    Boolean(bool),
    Array(JsonArrayIterator<'s>),
    Object(JsonFirstLineIterator<'s>),
}

impl<'s> UnwrappedJsonValue<'s> {
    pub fn get_type_name(&self) -> &'static str {
        match self {
            UnwrappedJsonValue::Null => "null",
            UnwrappedJsonValue::String(_) => "string",
            UnwrappedJsonValue::Number(_) => "number",
            UnwrappedJsonValue::Double(_) => "double",
            UnwrappedJsonValue::Boolean(_) => "boolean",
            UnwrappedJsonValue::Array(_) => "array",
            UnwrappedJsonValue::Object(_) => "object",
        }
    }
}

// Conversions from an unwrapped JSON value into the system numeric types.
//
// Semantics - STRICT, no silent data loss:
//   * integer target  - only `Number(i64)` is accepted, range-checked via `TryFrom<i64>`; a
//                        `Double` is rejected (a fractional JSON number can not become an integer
//                        without loss), and every non-numeric variant is rejected.
//   * float target     - both `Number` and `Double` are accepted, but ONLY when the value is
//                        representable exactly: the conversion must round-trip back to the original,
//                        otherwise (mantissa precision loss, or overflow to +/-Infinity) it is an
//                        `Err`. E.g. `0.1 -> f32`, `1e40 -> f32`, `9007199254740993 -> f64` all fail.
//   * `String`, `Boolean`, `Null`, `Array`, `Object` never convert to a number.
macro_rules! impl_try_from_unwrapped_for_integer {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'s> TryFrom<UnwrappedJsonValue<'s>> for $t {
                type Error = JsonParseError;

                fn try_from(value: UnwrappedJsonValue<'s>) -> Result<Self, Self::Error> {
                    match value {
                        UnwrappedJsonValue::Number(number) => {
                            <$t>::try_from(number).map_err(|_| {
                                JsonParseError::new(format!(
                                    "JSON number {} is out of range for type {}",
                                    number,
                                    stringify!($t)
                                ))
                            })
                        }
                        UnwrappedJsonValue::Double(number) => Err(JsonParseError::new(format!(
                            "JSON value is a floating point number ({}) and can not be converted to integer type {} without loss",
                            number,
                            stringify!($t)
                        ))),
                        other => Err(JsonParseError::new(format!(
                            "JSON value of type '{}' can not be converted to integer type {}",
                            other.get_type_name(),
                            stringify!($t)
                        ))),
                    }
                }
            }
        )*
    };
}

// Float target: STRICT / lossless. A `Number` or `Double` is accepted only if the converted
// value round-trips back to the original exactly; anything that would lose precision or overflow
// to +/-Infinity is rejected. `String`/`Boolean`/`Null`/`Array`/`Object` never convert.
macro_rules! impl_try_from_unwrapped_for_float {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'s> TryFrom<UnwrappedJsonValue<'s>> for $t {
                type Error = JsonParseError;

                #[allow(clippy::float_cmp)] // exact round-trip comparison is intentional
                fn try_from(value: UnwrappedJsonValue<'s>) -> Result<Self, Self::Error> {
                    match value {
                        UnwrappedJsonValue::Number(number) => {
                            let converted = number as $t;
                            // i128 holds any i64 and any finite f32/f64-of-an-i64 without saturation,
                            // so this comparison is exact (detects lost integer precision).
                            if converted as i128 == number as i128 {
                                Ok(converted)
                            } else {
                                Err(JsonParseError::new(format!(
                                    "JSON number {} can not be represented exactly as {} (would lose precision)",
                                    number,
                                    stringify!($t)
                                )))
                            }
                        }
                        UnwrappedJsonValue::Double(number) => {
                            let converted = number as $t;
                            // Widening back to f64 must reproduce the original bit-for-bit; catches
                            // both mantissa precision loss and saturation to +/-Infinity.
                            if converted as f64 == number {
                                Ok(converted)
                            } else {
                                Err(JsonParseError::new(format!(
                                    "JSON value {} can not be represented exactly as {} (would lose precision or overflow)",
                                    number,
                                    stringify!($t)
                                )))
                            }
                        }
                        other => Err(JsonParseError::new(format!(
                            "JSON value of type '{}' can not be converted to float type {}",
                            other.get_type_name(),
                            stringify!($t)
                        ))),
                    }
                }
            }
        )*
    };
}

// `Option<T>` variant of the conversions: a JSON `null` becomes `Ok(None)`; every other
// value goes through the underlying `TryFrom<UnwrappedJsonValue> for T`, so a type mismatch
// or out-of-range value is still an `Err` (only `null` maps to `None`, nothing is swallowed).
//
// A blanket `impl<T> TryFrom<UnwrappedJsonValue> for Option<T>` is forbidden by the orphan
// rules (uncovered `T` before the first local type, E0210), hence the per-type macro.
macro_rules! impl_try_from_unwrapped_for_option {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'s> TryFrom<UnwrappedJsonValue<'s>> for Option<$t> {
                type Error = JsonParseError;

                fn try_from(value: UnwrappedJsonValue<'s>) -> Result<Self, Self::Error> {
                    match value {
                        UnwrappedJsonValue::Null => Ok(None),
                        other => Ok(Some(<$t>::try_from(other)?)),
                    }
                }
            }
        )*
    };
}

impl_try_from_unwrapped_for_integer!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_try_from_unwrapped_for_float!(f32, f64);

// Strips a single pair of matching surrounding quotes (`"` or `'`) if present.
// `UnwrappedJsonValue::String` keeps the raw slice (quotes included), so we unquote here.
fn strip_json_quotes(src: &str) -> &str {
    let bytes = src.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == crate::consts::DOUBLE_QUOTE && last == crate::consts::DOUBLE_QUOTE)
            || (first == crate::consts::SINGLE_QUOTE && last == crate::consts::SINGLE_QUOTE)
        {
            return &src[1..src.len() - 1];
        }
    }
    src
}

impl<'s> TryFrom<UnwrappedJsonValue<'s>> for DateTimeAsMicroseconds {
    type Error = JsonParseError;

    fn try_from(value: UnwrappedJsonValue<'s>) -> Result<Self, Self::Error> {
        match value {
            UnwrappedJsonValue::String(src) => {
                let src = strip_json_quotes(src);
                DateTimeAsMicroseconds::from_str(src).ok_or_else(|| {
                    JsonParseError::new(format!(
                        "JSON string '{}' can not be parsed as a date time",
                        src
                    ))
                })
            }
            // Unit (sec/ms/us/ns) is auto-detected by magnitude inside `From<i64>`.
            UnwrappedJsonValue::Number(number) => Ok(number.into()),
            UnwrappedJsonValue::Double(number) => Err(JsonParseError::new(format!(
                "JSON value is a floating point number ({}) and can not be converted to DateTimeAsMicroseconds",
                number
            ))),
            other => Err(JsonParseError::new(format!(
                "JSON value of type '{}' can not be converted to DateTimeAsMicroseconds",
                other.get_type_name()
            ))),
        }
    }
}

impl<'s> TryFrom<UnwrappedJsonValue<'s>> for bool {
    type Error = JsonParseError;

    fn try_from(value: UnwrappedJsonValue<'s>) -> Result<Self, Self::Error> {
        match value {
            UnwrappedJsonValue::Boolean(value) => Ok(value),
            other => Err(JsonParseError::new(format!(
                "JSON value of type '{}' can not be converted to bool",
                other.get_type_name()
            ))),
        }
    }
}

impl_try_from_unwrapped_for_option!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool,
    DateTimeAsMicroseconds
);

fn convert_to_utf8(src: &[u8]) -> Result<&str, JsonParseError> {
    match std::str::from_utf8(src) {
        Ok(str) => Ok(str),
        Err(err) => Err(JsonParseError::new(format!(
            "Can convert value to utf8 string. Err {}",
            err
        ))),
    }
}

fn convert_to_i64(src: &[u8]) -> Result<i64, JsonParseError> {
    let src = convert_to_utf8(src)?;
    match src.parse() {
        Ok(value) => Ok(value),
        Err(err) => Err(JsonParseError::new(format!(
            "Can convert value to i64. Err {}",
            err
        ))),
    }
}

fn convert_to_f64(src: &[u8]) -> Result<f64, JsonParseError> {
    let src = convert_to_utf8(src)?;
    match src.parse() {
        Ok(value) => Ok(value),
        Err(err) => Err(JsonParseError::new(format!(
            "Can convert value to f64. Err {}",
            err
        ))),
    }
}

#[cfg(test)]
mod try_from_tests {
    use crate::j_path;
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    #[test]
    fn number_converts_to_every_integer_type() {
        let json = br#"{"v":42}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        assert_eq!(v.try_get_number::<i8>().unwrap(), 42i8);
        assert_eq!(v.try_get_number::<u8>().unwrap(), 42u8);
        assert_eq!(v.try_get_number::<i16>().unwrap(), 42i16);
        assert_eq!(v.try_get_number::<u16>().unwrap(), 42u16);
        assert_eq!(v.try_get_number::<i32>().unwrap(), 42i32);
        assert_eq!(v.try_get_number::<u32>().unwrap(), 42u32);
        assert_eq!(v.try_get_number::<i64>().unwrap(), 42i64);
        assert_eq!(v.try_get_number::<u64>().unwrap(), 42u64);
        assert_eq!(v.try_get_number::<i128>().unwrap(), 42i128);
        assert_eq!(v.try_get_number::<u128>().unwrap(), 42u128);
        assert_eq!(v.try_get_number::<isize>().unwrap(), 42isize);
        assert_eq!(v.try_get_number::<usize>().unwrap(), 42usize);
    }

    #[test]
    fn number_converts_to_float_types() {
        let json = br#"{"v":42}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        assert_eq!(v.try_get_number::<f32>().unwrap(), 42f32);
        assert_eq!(v.try_get_number::<f64>().unwrap(), 42f64);
    }

    #[test]
    fn try_into_from_unwrapped_directly() {
        let json = br#"{"v":7}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        let n: u16 = v.unwrap_value().unwrap().try_into().unwrap();
        assert_eq!(n, 7u16);
    }

    #[test]
    fn out_of_range_number_fails() {
        let json = br#"{"v":300}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        assert!(v.try_get_number::<u8>().is_err());
        assert!(v.try_get_number::<i8>().is_err());
        // but fits into wider types
        assert_eq!(v.try_get_number::<u16>().unwrap(), 300u16);
    }

    #[test]
    fn negative_number_into_unsigned_fails() {
        let json = br#"{"v":-5}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        assert!(v.try_get_number::<u8>().is_err());
        assert_eq!(v.try_get_number::<i8>().unwrap(), -5i8);
    }

    #[test]
    fn double_into_integer_is_rejected() {
        let json = br#"{"v":42.0}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        // even an integral-looking double is rejected for integer targets
        assert!(v.try_get_number::<u8>().is_err());
        assert!(v.try_get_number::<i64>().is_err());
        // but a float target accepts it
        assert_eq!(v.try_get_number::<f64>().unwrap(), 42.0f64);
    }

    #[test]
    fn double_into_float_works() {
        let json = br#"{"v":42.5}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        // 42.5 is exactly representable in both f32 and f64
        assert_eq!(v.try_get_number::<f64>().unwrap(), 42.5f64);
        assert_eq!(v.try_get_number::<f32>().unwrap(), 42.5f32);
    }

    #[test]
    fn float_strict_rejects_precision_loss() {
        // 0.1 is exact in f64 (as parsed) but NOT in f32
        let v = j_path::get_value(br#"{"v":0.1}"#, "v").unwrap().unwrap();
        assert_eq!(v.try_get_number::<f64>().unwrap(), 0.1f64);
        assert!(v.try_get_number::<f32>().is_err());

        // 1e40 overflows f32 to +Infinity -> rejected; fits f64 and round-trips.
        // (an exponent - upper or lower case, with or without a dot - classifies as a Double)
        let v = j_path::get_value(br#"{"v":1.0E40}"#, "v").unwrap().unwrap();
        assert!(v.try_get_number::<f32>().is_err());
        assert!(v.try_get_number::<f64>().is_ok());

        // the same value with a lowercase exponent and no dot is also a Double now
        let v = j_path::get_value(br#"{"v":1e40}"#, "v").unwrap().unwrap();
        assert!(v.try_get_number::<f32>().is_err());
        assert!(v.try_get_number::<f64>().is_ok());

        // 2^53 + 1: an integer that f64 can not hold exactly
        let v = j_path::get_value(br#"{"v":9007199254740993}"#, "v").unwrap().unwrap();
        assert!(v.try_get_number::<f64>().is_err());

        // 2^24 + 1: an integer that f32 can not hold exactly, but f64 can
        let v = j_path::get_value(br#"{"v":16777217}"#, "v").unwrap().unwrap();
        assert!(v.try_get_number::<f32>().is_err());
        assert_eq!(v.try_get_number::<f64>().unwrap(), 16777217f64);

        // small integers convert cleanly to both
        let v = j_path::get_value(br#"{"v":42}"#, "v").unwrap().unwrap();
        assert_eq!(v.try_get_number::<f32>().unwrap(), 42f32);
        assert_eq!(v.try_get_number::<f64>().unwrap(), 42f64);
    }

    #[test]
    fn float_strict_applies_to_option_variant() {
        // a lossy non-null value is an Err, NOT silently None
        let v = j_path::get_value(br#"{"v":1.0E40}"#, "v").unwrap().unwrap();
        assert!(v.try_get_number_opt::<f32>().is_err());
        assert!(v.try_get_number_opt::<f64>().unwrap().is_some());

        // null is still None
        let v = j_path::get_value(br#"{"v":null}"#, "v").unwrap().unwrap();
        assert_eq!(v.try_get_number_opt::<f32>().unwrap(), None);
    }

    #[test]
    fn string_number_is_rejected() {
        let json = br#"{"v":"42"}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        assert!(v.try_get_number::<u8>().is_err());
        assert!(v.try_get_number::<f64>().is_err());
    }

    #[test]
    fn bool_and_null_are_rejected() {
        let json = br#"{"b":true,"n":null}"#;
        let b = j_path::get_value(json, "b").unwrap().unwrap();
        let n = j_path::get_value(json, "n").unwrap().unwrap();

        assert!(b.try_get_number::<u8>().is_err());
        assert!(n.try_get_number::<u8>().is_err());
    }

    #[test]
    fn date_time_from_iso_string() {
        let raw = "2021-04-25T17:30:03.000Z";
        let json = format!(r#"{{"v":"{}"}}"#, raw);
        let v = j_path::get_value(json.as_bytes(), "v").unwrap().unwrap();

        let dt = v.try_get_date_time().unwrap();
        let expected = DateTimeAsMicroseconds::from_str(raw).unwrap();
        assert_eq!(dt.unix_microseconds, expected.unix_microseconds);

        // and directly through TryInto on the unwrapped value
        let dt2: DateTimeAsMicroseconds = v.unwrap_value().unwrap().try_into().unwrap();
        assert_eq!(dt2.unix_microseconds, expected.unix_microseconds);
    }

    #[test]
    fn date_time_from_numeric_string() {
        let json = br#"{"v":"1619371803"}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        let dt = v.try_get_date_time().unwrap();
        // seconds magnitude -> multiplied to microseconds
        assert_eq!(dt.unix_microseconds, 1619371803 * 1_000_000);
    }

    #[test]
    fn date_time_from_number() {
        let json = br#"{"v":1619371803}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        let dt = v.try_get_date_time().unwrap();
        assert_eq!(dt.unix_microseconds, 1619371803 * 1_000_000);
    }

    #[test]
    fn date_time_from_double_is_rejected() {
        let json = br#"{"v":1.5}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();
        assert!(v.try_get_date_time().is_err());
    }

    #[test]
    fn date_time_from_garbage_string_is_rejected() {
        let json = br#"{"v":"not a date"}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();
        assert!(v.try_get_date_time().is_err());
    }

    #[test]
    fn date_time_from_bool_is_rejected() {
        let json = br#"{"v":true}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();
        assert!(v.try_get_date_time().is_err());
    }

    #[test]
    fn option_null_becomes_none() {
        let json = br#"{"v":null}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        assert_eq!(v.try_get_number_opt::<u8>().unwrap(), None);
        assert_eq!(v.try_get_number_opt::<i64>().unwrap(), None);
        assert_eq!(v.try_get_number_opt::<f64>().unwrap(), None);
        assert_eq!(v.try_get_date_time_opt().unwrap(), None);

        // directly via TryInto on the unwrapped value
        let n: Option<u32> = v.unwrap_value().unwrap().try_into().unwrap();
        assert_eq!(n, None);
    }

    #[test]
    fn option_present_number_becomes_some() {
        let json = br#"{"v":42}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        assert_eq!(v.try_get_number_opt::<u8>().unwrap(), Some(42u8));
        assert_eq!(v.try_get_number_opt::<i32>().unwrap(), Some(42i32));
        assert_eq!(v.try_get_number_opt::<f64>().unwrap(), Some(42f64));

        let n: Option<u16> = v.unwrap_value().unwrap().try_into().unwrap();
        assert_eq!(n, Some(42u16));
    }

    #[test]
    fn option_out_of_range_is_still_err_not_none() {
        // a non-null value that can not be converted must NOT be silently swallowed to None
        let json = br#"{"v":300}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();
        assert!(v.try_get_number_opt::<u8>().is_err());
        assert_eq!(v.try_get_number_opt::<u16>().unwrap(), Some(300u16));
    }

    #[test]
    fn option_double_into_int_is_still_err() {
        let json = br#"{"v":42.0}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();
        assert!(v.try_get_number_opt::<u8>().is_err());
        assert_eq!(v.try_get_number_opt::<f64>().unwrap(), Some(42.0));
    }

    #[test]
    fn option_string_number_is_still_err() {
        let json = br#"{"v":"42"}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();
        assert!(v.try_get_number_opt::<u8>().is_err());
    }

    #[test]
    fn option_date_time_from_string_and_null() {
        let json = br#"{"a":"2021-04-25T17:30:03.000Z","b":null}"#;
        let a = j_path::get_value(json, "a").unwrap().unwrap();
        let b = j_path::get_value(json, "b").unwrap().unwrap();

        assert!(a.try_get_date_time_opt().unwrap().is_some());
        assert_eq!(b.try_get_date_time_opt().unwrap(), None);

        // garbage string still errors even for the Option variant
        let json = br#"{"v":"not a date"}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();
        assert!(v.try_get_date_time_opt().is_err());
    }

    #[test]
    fn option_works_while_iterating_array_with_nulls() {
        let src = "[1, null, 3]";
        let iter: crate::json_reader::JsonArrayIterator = src.as_bytes().into();

        let mut collected: Vec<Option<u8>> = vec![];
        while let Some(item) = iter.get_next() {
            let item = item.unwrap();
            collected.push(item.try_get_number_opt::<u8>().unwrap());
        }

        assert_eq!(collected, vec![Some(1u8), None, Some(3u8)]);
    }

    // Mirrors the downstream generic wrapper: it must be possible to name the bound using only
    // the crate's PUBLIC re-exports (`my_json::json_reader::{UnwrappedJsonValue, JsonParseError}`).
    fn parse_generic<'s, T>(
        v: &'s crate::json_reader::JsonValueRef<'s>,
    ) -> Result<T, crate::json_reader::JsonParseError>
    where
        T: TryFrom<crate::json_reader::UnwrappedJsonValue<'s>, Error = crate::json_reader::JsonParseError>,
    {
        v.try_get_number::<T>()
    }

    #[test]
    fn generic_wrapper_over_public_reexports() {
        let json = br#"{"v":7}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        assert_eq!(parse_generic::<u8>(&v).unwrap(), 7u8);
        assert_eq!(parse_generic::<i64>(&v).unwrap(), 7i64);
        assert_eq!(parse_generic::<f64>(&v).unwrap(), 7f64);
    }

    #[test]
    fn bool_converts_and_rejects() {
        let json = br#"{"t":true,"f":false,"n":1,"nil":null}"#;
        let t = j_path::get_value(json, "t").unwrap().unwrap();
        let f = j_path::get_value(json, "f").unwrap().unwrap();
        let n = j_path::get_value(json, "n").unwrap().unwrap();
        let nil = j_path::get_value(json, "nil").unwrap().unwrap();

        assert_eq!(t.try_get_number::<bool>().unwrap(), true);
        assert_eq!(f.try_get_number::<bool>().unwrap(), false);
        // a number is NOT a bool
        assert!(n.try_get_number::<bool>().is_err());

        // Option variant: null -> None, present -> Some, mismatch -> Err
        assert_eq!(t.try_get_number_opt::<bool>().unwrap(), Some(true));
        assert_eq!(nil.try_get_number_opt::<bool>().unwrap(), None);
        assert!(n.try_get_number_opt::<bool>().is_err());

        // and via TryInto directly on the unwrapped value
        let b: bool = t.unwrap_value().unwrap().try_into().unwrap();
        assert!(b);
    }

    #[test]
    fn array_and_object_are_rejected_by_every_family() {
        let json = br#"{"arr":[1,2,3],"obj":{"a":1}}"#;
        let arr = j_path::get_value(json, "arr").unwrap().unwrap();
        let obj = j_path::get_value(json, "obj").unwrap().unwrap();

        for v in [&arr, &obj] {
            assert!(v.try_get_number::<u8>().is_err());
            assert!(v.try_get_number::<f64>().is_err());
            assert!(v.try_get_number::<bool>().is_err());
            assert!(v.try_get_date_time().is_err());
            // Option variants: a non-null structural value is still an Err, never None
            assert!(v.try_get_number_opt::<u8>().is_err());
            assert!(v.try_get_date_time_opt().is_err());
        }
    }

    #[test]
    fn option_wide_integer_types() {
        let json = br#"{"v":42,"nil":null}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();
        let nil = j_path::get_value(json, "nil").unwrap().unwrap();

        assert_eq!(v.try_get_number_opt::<i128>().unwrap(), Some(42i128));
        assert_eq!(v.try_get_number_opt::<u128>().unwrap(), Some(42u128));
        assert_eq!(v.try_get_number_opt::<isize>().unwrap(), Some(42isize));
        assert_eq!(v.try_get_number_opt::<usize>().unwrap(), Some(42usize));

        assert_eq!(nil.try_get_number_opt::<i128>().unwrap(), None);
        assert_eq!(nil.try_get_number_opt::<usize>().unwrap(), None);
    }

    #[test]
    fn option_negative_into_unsigned_is_err_not_none() {
        let json = br#"{"v":-5}"#;
        let v = j_path::get_value(json, "v").unwrap().unwrap();

        // negative must NOT be silently swallowed to None
        assert!(v.try_get_number_opt::<u8>().is_err());
        assert_eq!(v.try_get_number_opt::<i8>().unwrap(), Some(-5i8));
    }

    #[test]
    fn works_while_iterating_array() {
        let src = "[1, 2, 3]";
        let iter: crate::json_reader::JsonArrayIterator = src.as_bytes().into();

        let mut collected = vec![];
        while let Some(item) = iter.get_next() {
            let item = item.unwrap();
            collected.push(item.try_get_number::<u8>().unwrap());
        }

        assert_eq!(collected, vec![1u8, 2u8, 3u8]);
    }
}

#[cfg(test)]
mod string_accessor_tests {
    use crate::j_path;

    #[test]
    fn as_str_de_escapes_and_handles_empty_string() {
        let v = j_path::get_value(br#"{"v":""}"#, "v").unwrap().unwrap();
        // an empty JSON string is "", NOT `""`
        assert_eq!(v.as_str().unwrap().as_str(), "");

        let v = j_path::get_value(br#"{"v":"a\nb\t\"c\""}"#, "v").unwrap().unwrap();
        assert_eq!(v.as_str().unwrap().as_str(), "a\nb\t\"c\"");
    }

    #[test]
    fn as_unescaped_str_strips_quotes_and_handles_empty() {
        let v = j_path::get_value(br#"{"v":""}"#, "v").unwrap().unwrap();
        assert_eq!(v.as_unescaped_str().unwrap(), "");

        let v = j_path::get_value(br#"{"v":"hello"}"#, "v").unwrap().unwrap();
        assert_eq!(v.as_unescaped_str().unwrap(), "hello");
    }

    #[test]
    fn invalid_utf8_inside_string_does_not_panic() {
        use crate::json_reader::JsonValue;

        // A quoted "string" value whose inner bytes are not valid UTF-8: the accessors must NOT
        // panic (the old `from_utf8(..).unwrap()` was a DoS on hostile bytes) - they return None.
        let bytes: &[u8] = &[b'"', 0xFF, b'"'];
        let value = JsonValue::new(0, bytes.len());

        assert!(value.as_str(&bytes).is_none());
        assert!(value.as_unescaped_str(&bytes).is_none());
        assert!(value.as_raw_str(&bytes).is_none());
    }
}

use std::collections::HashMap;

use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

use super::{JsonArrayIterator, JsonFirstLineIterator, JsonParseError, JsonValueRef};

/// The read half of the `JsonValueWriter` contract: whatever a `JsonValueWriter` puts on the
/// wire, the matching `JsonValueReader` takes back off it, unchanged.
///
/// The two traits are mirrors on purpose. When only the writer existed, a nested object was
/// written by one contract and read by another (serde), and every disagreement between them -
/// field naming, date-time spelling, enum spelling - surfaced as a separate production bug.
/// One trait pair, generated from one set of metadata, is what removes that whole class.
///
/// # Reading is verbatim
///
/// Every implementation parses the value's **raw source text** rather than routing through
/// [`JsonValueRef::try_get_number`], which normalises through `i64` / `f64` first. That
/// normalisation is exactly where symmetry dies:
///
/// * `u64::MAX` (`18446744073709551615`) does not fit `i64`, so the normalised path fails on a
///   value the writer is perfectly happy to write.
/// * `0.1f32` writes as `0.1`; read as `f64` and narrowed back to `f32` it no longer compares
///   equal to the `f64`, so a strict narrowing check rejects it.
/// * `100.00` as a `Decimal` keeps its scale only if the digits are read as written.
///
/// Reading the text as the *target* type sidesteps all three: it is both lossless and simpler.
///
/// # Strictness
///
/// A type mismatch is always an `Err`, never a silent coercion and never a panic. Only an
/// explicit `null` (or an absent key, see [`Self::from_absent_json_value`]) maps to `None`, and
/// only for `Option<T>`.
pub trait JsonValueReader<'s>: Sized {
    /// Reads the value. Returns `Err` if it does not hold this type.
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError>;

    /// Produces the value for a key that is **absent** from the object.
    ///
    /// This is the mirror of `JsonObjectWriter::write_if_some`, which omits the key entirely
    /// for `None`: since the writer can leave nothing behind, the reader has to be able to read
    /// nothing back. The default is an error - a missing key is missing data - and `Option<T>`
    /// overrides it to `Ok(None)`, so an omitted key and an explicit `null` agree.
    fn from_absent_json_value(field_name: &str) -> Result<Self, JsonParseError> {
        Err(JsonParseError::new(format!(
            "Field '{}' is missing and type {} has no value for an absent field",
            field_name,
            std::any::type_name::<Self>()
        )))
    }
}

/// The value's raw source text, borrowed for the **full** `'s` and not for the (shorter) borrow
/// of `value` - which is what lets the structural readers below recurse. See
/// [`JsonValueRef::as_slice`].
fn raw_slice<'s>(value: &JsonValueRef<'s>) -> &'s [u8] {
    value.as_slice()
}

fn raw_str<'s>(value: &JsonValueRef<'s>) -> Result<&'s str, JsonParseError> {
    let slice = raw_slice(value);
    std::str::from_utf8(slice).map_err(|err| {
        JsonParseError::new(format!("JSON value is not a valid utf8 string. {}", err))
    })
}

/// Names the JSON type of a raw token, for error messages only. Every `is_*` helper that indexes
/// `src[0]` is guarded by the empty check first, so this never panics on hostile input.
fn json_type_name(raw: &[u8]) -> &'static str {
    if raw.is_empty() {
        return "empty";
    }

    if crate::json_utils::is_null(raw) {
        return "null";
    }

    if crate::json_utils::is_bool(raw) {
        return "boolean";
    }

    if crate::json_utils::is_array(raw) {
        return "array";
    }

    if crate::json_utils::is_object(raw) {
        return "object";
    }

    if crate::json_utils::is_string(raw) {
        return "string";
    }

    match crate::json_utils::is_number(raw) {
        crate::json_utils::NumberType::Number => "number",
        crate::json_utils::NumberType::Double => "double",
        crate::json_utils::NumberType::NaN => "unknown",
    }
}

/// A bounded, lossy rendering of a token for an error message - an error about a 10 MB array
/// must not carry the array.
fn preview(raw: &[u8]) -> String {
    const MAX: usize = 48;

    let as_str = String::from_utf8_lossy(raw);

    if as_str.chars().count() <= MAX {
        return as_str.into_owned();
    }

    let truncated: String = as_str.chars().take(MAX).collect();
    format!("{}...", truncated)
}

fn type_mismatch(raw: &[u8], target: &str) -> JsonParseError {
    JsonParseError::new(format!(
        "JSON value of type '{}' can not be read as {}. Value: {}",
        json_type_name(raw),
        target,
        preview(raw)
    ))
}

// Integers: the digits are parsed straight into the target type, so the full width of every type
// is available - `u64::MAX` and the 128 bit types included. Routing through `i64` (as
// `try_get_number` does) would cap the readable range at `i64` for all of them.
macro_rules! impl_json_value_reader_for_integer {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'s> JsonValueReader<'s> for $t {
                fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
                    let raw = raw_slice(value);

                    match crate::json_utils::is_number(raw) {
                        crate::json_utils::NumberType::Number => {
                            let as_str = raw_str(value)?;
                            as_str.parse::<$t>().map_err(|_| {
                                JsonParseError::new(format!(
                                    "JSON number {} is out of range for type {}",
                                    as_str,
                                    stringify!($t)
                                ))
                            })
                        }
                        crate::json_utils::NumberType::Double => Err(JsonParseError::new(format!(
                            "JSON value is a floating point number ({}) and can not be read as integer type {} without loss",
                            preview(raw),
                            stringify!($t)
                        ))),
                        crate::json_utils::NumberType::NaN => {
                            Err(type_mismatch(raw, stringify!($t)))
                        }
                    }
                }
            }
        )*
    };
}

impl_json_value_reader_for_integer!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

// Floats: parsed as the target type directly, which is what makes the round trip exact. Rust's
// `Display` emits the shortest text that reads back as the same value, and `FromStr` is correctly
// rounded, so `write -> read` is the identity for every finite `f32` / `f64`.
//
// Note this is *more* permissive than `try_get_number::<f32>()`, which reads through `f64` and
// rejects the narrowing - and would therefore reject `0.1f32`, a value the writer emits happily.
//
// An integer-classified token is accepted too: whole floats render without a dot (`0.0` -> `0`).
macro_rules! impl_json_value_reader_for_float {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'s> JsonValueReader<'s> for $t {
                fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
                    let raw = raw_slice(value);

                    match crate::json_utils::is_number(raw) {
                        crate::json_utils::NumberType::Number
                        | crate::json_utils::NumberType::Double => {
                            let as_str = raw_str(value)?;

                            let result = as_str.parse::<$t>().map_err(|_| {
                                JsonParseError::new(format!(
                                    "JSON number {} can not be read as {}",
                                    as_str,
                                    stringify!($t)
                                ))
                            })?;

                            // A finite decimal that reads back as +/-Infinity overflowed the
                            // target. The writer never emits such a token for this type, so this
                            // only ever fires on a foreign payload - where silently yielding
                            // `inf` would be exactly the data loss this trait exists to prevent.
                            if !result.is_finite() {
                                return Err(JsonParseError::new(format!(
                                    "JSON number {} overflows {}",
                                    as_str,
                                    stringify!($t)
                                )));
                            }

                            Ok(result)
                        }
                        crate::json_utils::NumberType::NaN => {
                            Err(type_mismatch(raw, stringify!($t)))
                        }
                    }
                }
            }
        )*
    };
}

impl_json_value_reader_for_float!(f32, f64);

impl<'s> JsonValueReader<'s> for bool {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        let raw = raw_slice(value);

        match crate::json_utils::as_bool_value(raw) {
            Some(result) => Ok(result),
            None => Err(type_mismatch(raw, "bool")),
        }
    }
}

// The mirror of the `DateTimeAsMicroseconds` writer, which emits `to_rfc3339_utc()`.
//
// The spelling is not decided here. `from_json_value_str` in rust-extensions is the single
// place that knows every form this type accepts on the wire, and it is pinned to serde's
// `Deserialize` by a test there (`serde_and_from_json_value_str_agree`). Re-implementing the
// dispatch in this crate is how the two halves drifted apart in the first place - so this
// hands the raw token over verbatim, quotes and all, and lets that function decide.
//
// Microseconds survive: `to_rfc3339_utc()` always renders 6 fractional digits and the parser
// reads all 6 back.
impl<'s> JsonValueReader<'s> for DateTimeAsMicroseconds {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        let raw = raw_str(value)?;

        DateTimeAsMicroseconds::from_json_value_str(raw).ok_or_else(|| {
            JsonParseError::new(format!(
                "JSON value {} can not be read as DateTimeAsMicroseconds",
                preview(raw.as_bytes())
            ))
        })
    }
}

// Decimal is written unquoted via `Display`, so the digits on the wire are the digits of the
// value - scale included. `from_str_exact` reads them back without rounding, which is what keeps
// `100.00` at scale 2 instead of collapsing to `100` through an f64 detour.
#[cfg(feature = "decimal")]
impl<'s> JsonValueReader<'s> for rust_decimal::Decimal {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        let raw = raw_slice(value);

        if crate::json_utils::is_number(raw).is_nan() {
            return Err(type_mismatch(raw, "rust_decimal::Decimal"));
        }

        let as_str = raw_str(value)?;

        rust_decimal::Decimal::from_str_exact(as_str).map_err(|err| {
            JsonParseError::new(format!(
                "JSON number {} can not be read as rust_decimal::Decimal. {}",
                as_str, err
            ))
        })
    }
}

// The string readers.
//
// `StrOrString` - not `&'s str` - is the counterpart of the `&str` writers, and it has to be:
// a JSON string carrying an escape (`"a\"b"`) has no de-escaped form anywhere inside the source
// bytes, so no `&'s str` could point at one. `StrOrString` borrows when the token is escape-free
// and allocates only when it is not, which is why the crate already uses it for exactly this.
impl<'s> JsonValueReader<'s> for StrOrString<'s> {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        let raw = raw_slice(value);

        if raw.is_empty() || !crate::json_utils::is_string(raw) {
            return Err(type_mismatch(raw, "string"));
        }

        crate::json_utils::try_get_string_value(raw).ok_or_else(|| {
            JsonParseError::new(format!(
                "JSON string {} is not a valid utf8 string",
                preview(raw)
            ))
        })
    }
}

impl<'s> JsonValueReader<'s> for String {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        let result: StrOrString<'s> = JsonValueReader::from_json_value(value)?;
        Ok(result.to_string())
    }
}

// `null` -> `None`, an absent key -> `None` (see `from_absent_json_value`), anything else is
// delegated - so a present-but-wrong value stays an `Err` and is never swallowed into `None`.
impl<'s, T: JsonValueReader<'s>> JsonValueReader<'s> for Option<T> {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        let raw = raw_slice(value);

        if raw.is_empty() || crate::json_utils::is_null(raw) {
            return Ok(None);
        }

        Ok(Some(T::from_json_value(value)?))
    }

    fn from_absent_json_value(_field_name: &str) -> Result<Self, JsonParseError> {
        Ok(None)
    }
}

impl<'s, T: JsonValueReader<'s>> JsonValueReader<'s> for Vec<T> {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        // Borrowed for the full `'s`, so every element ref built below carries the document
        // lifetime rather than the lifetime of this call.
        let inner: &'s [u8] = raw_slice(value);

        if inner.is_empty() || !crate::json_utils::is_array(inner) {
            return Err(type_mismatch(inner, "array"));
        }

        let iterator = JsonArrayIterator::new(inner)?;

        let mut result = Vec::new();

        while let Some(item) = iterator.get_next() {
            let item = item?;

            // The iterator hands back a ref borrowed for its own (local) lifetime. `data` is a
            // pair of offsets into `inner` and carries no lifetime, so rebuilding the ref against
            // `inner` re-widens it to `'s` without copying anything.
            let item = JsonValueRef::new(item.data.clone(), inner);

            result.push(T::from_json_value(&item)?);
        }

        Ok(result)
    }
}

// Mirrors the `HashMap<String, V>` writer, which renders each entry as a `"key":value` pair.
// Entry order is not part of the contract on either side.
impl<'s, V: JsonValueReader<'s>> JsonValueReader<'s> for HashMap<String, V> {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        let inner: &'s [u8] = raw_slice(value);

        if inner.is_empty() || !crate::json_utils::is_object(inner) {
            return Err(type_mismatch(inner, "object"));
        }

        let iterator = JsonFirstLineIterator::new(inner);

        let mut result = HashMap::new();

        while let Some(item) = iterator.get_next() {
            let (key, item_value) = item?;

            let key = key.as_str()?.to_string();

            let item_value = JsonValueRef::new(item_value.data.clone(), inner);

            result.insert(key, V::from_json_value(&item_value)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod symmetry_tests {
    use super::*;
    use crate::json_writer::{JsonObjectWriter, JsonValueWriter};

    /// The property under test, for every supported type:
    ///
    ///   `read(write(v)) == v`
    ///
    /// Both halves go through the real public API - the writer builds an object, the reader
    /// resolves the value back out of it by path - so this exercises the exact path a generated
    /// `#[http_body]` model would take.
    fn write_value<T: JsonValueWriter>(value: T) -> String {
        JsonObjectWriter::new().write("v", value).build()
    }

    fn read_value<'s, T: JsonValueReader<'s>>(json: &'s str) -> Result<T, JsonParseError> {
        let value = crate::j_path::get_value(json.as_bytes(), "v")?
            .expect("the writer always writes key 'v'");
        T::from_json_value(&value)
    }

    /// Deterministic xorshift64*. A fixed seed keeps a failure reproducible and keeps the crate
    /// free of a property-testing dependency; the generated domain is what matters here, not the
    /// generator.
    struct Rng(u64);

    impl Rng {
        fn new(seed: u64) -> Self {
            Self(seed)
        }

        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn next_u128(&mut self) -> u128 {
            ((self.next_u64() as u128) << 64) | self.next_u64() as u128
        }
    }

    // ----- integers ---------------------------------------------------------------------------

    // Every integer type, over its own full width: both extremes, the values around zero, and a
    // spread of random bit patterns truncated into range.
    macro_rules! assert_integer_symmetry {
        ($($t:ty),* $(,)?) => {
            $({
                let mut rng = Rng::new(0x2545F4914F6CDD1D);

                let mut values: Vec<$t> = vec![0 as $t, 1 as $t, <$t>::MIN, <$t>::MAX];

                if <$t>::MIN != 0 {
                    values.push(<$t>::MIN + 1);
                }
                values.push(<$t>::MAX - 1);

                for _ in 0..500 {
                    values.push(rng.next_u128() as $t);
                }

                for value in values {
                    let json = write_value(value);

                    let back: $t = read_value(&json).unwrap_or_else(|err| {
                        panic!(
                            "{} failed to read back {} from {}: {}",
                            stringify!($t), value, json, err.to_string()
                        )
                    });

                    assert_eq!(
                        back, value,
                        "{} lost its value through {}", stringify!($t), json
                    );
                }
            })*
        };
    }

    #[test]
    fn property_every_integer_type_round_trips() {
        assert_integer_symmetry!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
    }

    /// The headline regression: these are the values a `i64`-normalised read path can not return.
    /// `u64::MAX` does not fit `i64` at all, and the 128 bit extremes are far outside it - yet the
    /// writer emits all of them as plain digits.
    #[test]
    fn wide_numbers_keep_full_precision() {
        let json = write_value(u64::MAX);
        assert_eq!(json, r#"{"v":18446744073709551615}"#);
        assert_eq!(read_value::<u64>(&json).unwrap(), u64::MAX);

        let json = write_value(u128::MAX);
        assert_eq!(json, r#"{"v":340282366920938463463374607431768211455}"#);
        assert_eq!(read_value::<u128>(&json).unwrap(), u128::MAX);

        let json = write_value(i128::MIN);
        assert_eq!(json, r#"{"v":-170141183460469231731687303715884105728}"#);
        assert_eq!(read_value::<i128>(&json).unwrap(), i128::MIN);

        // 2^53 + 1 - the first integer f64 can not hold. Reading the digits as an integer keeps
        // it; a detour through f64 would silently return 9007199254740992.
        let json = write_value(9007199254740993i64);
        assert_eq!(read_value::<i64>(&json).unwrap(), 9007199254740993i64);

        // The same value the pre-existing `try_get_number` path rejects outright, proving the two
        // readers differ on purpose.
        let value = crate::j_path::get_value(json.as_bytes(), "v")
            .unwrap()
            .unwrap();
        assert!(value.try_get_number::<f64>().is_err());
    }

    // ----- floats -----------------------------------------------------------------------------

    #[test]
    fn property_f64_round_trips() {
        let mut rng = Rng::new(0x9E3779B97F4A7C15);

        let mut values: Vec<f64> = vec![
            0.0,
            -0.0,
            1.0,
            -1.0,
            0.1,
            100.0,
            4.5,
            -1.25,
            f64::MIN,
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::EPSILON,
            1e300,
            1e-300,
        ];

        for _ in 0..2000 {
            let candidate = f64::from_bits(rng.next_u64());
            // The writer maps every non-finite to `null` by design (it has no JSON spelling), so
            // those are outside the symmetric domain - `Option<f64>` covers them below.
            if candidate.is_finite() {
                values.push(candidate);
            }
        }

        for value in values {
            let json = write_value(value);

            let back: f64 = read_value(&json)
                .unwrap_or_else(|err| panic!("failed to read {}: {}", json, err.to_string()));

            assert_eq!(
                back.to_bits(),
                value.to_bits(),
                "f64 lost its value through {}",
                json
            );
        }
    }

    /// `0.1f32` is the value that makes the case for verbatim reading: it writes as `0.1`, and any
    /// reader that parses that text as `f64` first can not hand back the original `f32` - the
    /// pre-existing strict `try_get_number::<f32>` rejects it outright rather than lie.
    #[test]
    fn property_f32_round_trips_including_values_a_f64_detour_would_reject() {
        let mut rng = Rng::new(0xD1B54A32D192ED03);

        let mut values: Vec<f32> = vec![
            0.0,
            -0.0,
            0.1,
            -1.25,
            f32::MIN,
            f32::MAX,
            f32::MIN_POSITIVE,
            f32::EPSILON,
        ];

        for _ in 0..2000 {
            let candidate = f32::from_bits(rng.next_u64() as u32);
            if candidate.is_finite() {
                values.push(candidate);
            }
        }

        for value in values {
            let json = write_value(value);

            let back: f32 = read_value(&json)
                .unwrap_or_else(|err| panic!("failed to read {}: {}", json, err.to_string()));

            assert_eq!(
                back.to_bits(),
                value.to_bits(),
                "f32 lost its value through {}",
                json
            );
        }

        // Pin the specific asymmetry this trait fixes.
        let json = write_value(0.1f32);
        assert_eq!(json, r#"{"v":0.1}"#);
        assert_eq!(read_value::<f32>(&json).unwrap(), 0.1f32);

        let value = crate::j_path::get_value(json.as_bytes(), "v")
            .unwrap()
            .unwrap();
        assert!(
            value.try_get_number::<f32>().is_err(),
            "try_get_number is the normalised path that can not round trip 0.1f32"
        );
    }

    #[test]
    fn non_finite_floats_are_a_documented_one_way_street() {
        // The writer turns them into `null` (matching serde_json), so a bare f64 read fails and
        // an Option<f64> reads None. Neither panics.
        let json = write_value(f64::NAN);
        assert_eq!(json, r#"{"v":null}"#);
        assert!(read_value::<f64>(&json).is_err());
        assert_eq!(read_value::<Option<f64>>(&json).unwrap(), None);
    }

    // ----- bool / strings ---------------------------------------------------------------------

    #[test]
    fn property_bool_and_strings_round_trip() {
        for value in [true, false] {
            let json = write_value(value);
            assert_eq!(read_value::<bool>(&json).unwrap(), value);
        }

        let mut rng = Rng::new(0xA24BAED4963EE407);

        let mut values: Vec<String> = vec![
            String::new(),
            "hello".to_string(),
            // every escape the writer emits
            "a\nb\tc\"d\\e".to_string(),
            "he said \"hi\"".to_string(),
            "юникод и эмодзи 🎉".to_string(),
            "\u{0}\u{1}\u{1f}".to_string(),
            "/slash".to_string(),
        ];

        for _ in 0..500 {
            let len = (rng.next_u64() % 24) as usize;
            let value: String = (0..len)
                .map(|_| {
                    // Deliberately biased towards the characters that need escaping.
                    let alphabet = ['a', 'Я', '"', '\\', '\n', '\t', '🎉', '/', '\u{7}', ' '];
                    alphabet[(rng.next_u64() % alphabet.len() as u64) as usize]
                })
                .collect();
            values.push(value);
        }

        for value in values {
            let json = write_value(value.as_str());

            let back: String = read_value(&json)
                .unwrap_or_else(|err| panic!("failed to read {}: {}", json, err.to_string()));
            assert_eq!(back, value, "String lost its value through {}", json);

            // StrOrString is the borrowing counterpart and must agree.
            let back: StrOrString = read_value(&json).unwrap();
            assert_eq!(back.as_str(), value.as_str());
        }
    }

    // ----- date time --------------------------------------------------------------------------

    /// Real microseconds, not a whole second - the digits must survive verbatim.
    #[test]
    fn date_time_keeps_real_microseconds() {
        let dt = DateTimeAsMicroseconds::from_str("2024-01-02T03:04:05.123456Z").unwrap();

        let json = write_value(dt);
        assert_eq!(json, r#"{"v":"2024-01-02T03:04:05.123456Z"}"#);

        let back: DateTimeAsMicroseconds = read_value(&json).unwrap();
        assert_eq!(back.unix_microseconds, dt.unix_microseconds);
    }

    #[test]
    fn property_date_time_round_trips() {
        let mut rng = Rng::new(0x14057B7EF767814F);

        let mut values = vec![
            DateTimeAsMicroseconds::from_str("2024-01-02T03:04:05.123456Z").unwrap(),
            DateTimeAsMicroseconds::from_str("1970-01-01T00:00:00.000000Z").unwrap(),
            DateTimeAsMicroseconds::from_str("2021-04-25T17:30:03.000000Z").unwrap(),
            DateTimeAsMicroseconds::from_str("2023-12-31T23:59:59.999999Z").unwrap(),
            // Pre-epoch: `unix_microseconds` goes negative, which is a different branch of
            // `to_chrono_utc` and the only place the sub-second part is borrowed from the second
            // below. Both the tick before the epoch and the tick after it are included.
            DateTimeAsMicroseconds::new(-1),
            DateTimeAsMicroseconds::new(1),
            DateTimeAsMicroseconds::new(0),
            DateTimeAsMicroseconds::from_str("1900-01-01T00:00:00.000000Z").unwrap(),
            DateTimeAsMicroseconds::from_str("1969-12-31T23:59:59.999999Z").unwrap(),
        ];

        for _ in 0..500 {
            // A spread across roughly 1900..2100, at microsecond resolution - straddling the
            // epoch so the negative range is covered, not just the modern one.
            const RANGE: u64 = 6_311_433_600_000_000;
            const EPOCH_OFFSET: i64 = 2_208_988_800_000_000; // 1900-01-01

            let micros = (rng.next_u64() % RANGE) as i64 - EPOCH_OFFSET;
            values.push(DateTimeAsMicroseconds::new(micros));
        }

        for value in values {
            let json = write_value(value);

            let back: DateTimeAsMicroseconds = read_value(&json)
                .unwrap_or_else(|err| panic!("failed to read {}: {}", json, err.to_string()));

            assert_eq!(
                back.unix_microseconds, value.unix_microseconds,
                "date time lost microseconds through {}",
                json
            );
        }
    }

    /// The reader delegates the spelling to rust-extensions, so every form serde accepts reads
    /// here too - including the legacy numeric payloads that predate the RFC 3339 writer.
    #[test]
    fn date_time_reads_every_wire_spelling_rust_extensions_accepts() {
        for (raw, expected) in [
            (
                r#"{"v":"2021-04-25T17:30:03.000000Z"}"#,
                "2021-04-25T17:30:03.000000Z",
            ),
            (
                r#"{"v":"2021-04-25T17:30:03+00:00"}"#,
                "2021-04-25T17:30:03.000000Z",
            ),
            (r#"{"v":1619371803000000}"#, "2021-04-25T17:30:03.000000Z"),
            (r#"{"v":"1619371803000000"}"#, "2021-04-25T17:30:03.000000Z"),
        ] {
            let back: DateTimeAsMicroseconds = read_value(raw)
                .unwrap_or_else(|err| panic!("failed to read {}: {}", raw, err.to_string()));

            assert_eq!(back.to_rfc3339_utc(), expected, "reading {}", raw);
        }
    }

    // ----- decimal ----------------------------------------------------------------------------

    #[cfg(feature = "decimal")]
    #[test]
    fn decimal_keeps_scale_and_round_trips() {
        use rust_decimal::Decimal;

        // The scale is the whole point: `100.00` must not come back as `100`. rust_decimal
        // compares by numeric value, so equality alone would not catch a lost scale - the string
        // form is what pins it.
        let value = Decimal::from_str_exact("100.00").unwrap();

        let json = write_value(value);
        assert_eq!(json, r#"{"v":100.00}"#);

        let back: Decimal = read_value(&json).unwrap();
        assert_eq!(back, value);
        assert_eq!(back.to_string(), "100.00", "the scale must survive");
        assert_eq!(back.scale(), 2);

        for raw in [
            "0",
            "100.00",
            "-1.5",
            "0.000000001",
            "79228162514264337593543950335", // Decimal::MAX
            "-79228162514264337593543950335",
        ] {
            let value = Decimal::from_str_exact(raw).unwrap();

            let json = write_value(value);
            let back: Decimal = read_value(&json)
                .unwrap_or_else(|err| panic!("failed to read {}: {}", json, err.to_string()));

            assert_eq!(back, value);
            assert_eq!(
                back.to_string(),
                value.to_string(),
                "scale lost through {}",
                json
            );
        }

        let json = write_value(Some(Decimal::from_str_exact("1.10").unwrap()));
        assert_eq!(
            read_value::<Option<Decimal>>(&json)
                .unwrap()
                .unwrap()
                .to_string(),
            "1.10"
        );

        let json = write_value(Option::<Decimal>::None);
        assert_eq!(json, r#"{"v":null}"#);
        assert_eq!(read_value::<Option<Decimal>>(&json).unwrap(), None);
    }

    // ----- Option -----------------------------------------------------------------------------

    #[test]
    fn option_reads_null_and_absent_key_as_none() {
        // explicit null
        let json = write_value(Option::<u64>::None);
        assert_eq!(json, r#"{"v":null}"#);
        assert_eq!(read_value::<Option<u64>>(&json).unwrap(), None);

        // present
        let json = write_value(Some(42u64));
        assert_eq!(read_value::<Option<u64>>(&json).unwrap(), Some(42));

        // absent: `write_if_some` omits the key entirely, which is the case `from_absent_json_value`
        // exists for - an omitted key must read the same as an explicit null.
        let json = JsonObjectWriter::new()
            .write_if_some("v", Option::<u64>::None)
            .build();
        assert_eq!(json, "{}");

        let absent = crate::j_path::get_value(json.as_bytes(), "v").unwrap();
        assert!(absent.is_none());

        assert_eq!(Option::<u64>::from_absent_json_value("v").unwrap(), None);

        // ...while a required field says so instead of silently defaulting.
        let err = u64::from_absent_json_value("v").unwrap_err();
        assert!(
            err.to_string().contains("missing"),
            "got: {}",
            err.to_string()
        );
    }

    #[test]
    fn option_does_not_swallow_a_bad_value_into_none() {
        // Only `null` is None. A present-but-wrong value stays an error.
        let json = r#"{"v":"not a number"}"#;
        assert!(read_value::<Option<u64>>(json).is_err());

        let json = r#"{"v":300}"#;
        assert!(read_value::<Option<u8>>(json).is_err());
        assert_eq!(read_value::<Option<u16>>(json).unwrap(), Some(300));
    }

    // ----- nesting ----------------------------------------------------------------------------

    #[test]
    fn nested_vec_of_vec_round_trips() {
        let value = vec![vec![1i32, 2], vec![3], vec![]];

        let json = write_value(value.clone());
        assert_eq!(json, r#"{"v":[[1,2],[3],[]]}"#);

        assert_eq!(read_value::<Vec<Vec<i32>>>(&json).unwrap(), value);

        // three levels deep
        let value = vec![vec![vec![1i32], vec![2, 3]], vec![vec![4]]];
        let json = write_value(value.clone());
        assert_eq!(json, r#"{"v":[[[1],[2,3]],[[4]]]}"#);
        assert_eq!(read_value::<Vec<Vec<Vec<i32>>>>(&json).unwrap(), value);
    }

    #[test]
    fn vec_of_strings_and_date_times_round_trips() {
        let value = vec!["a\"b".to_string(), String::new(), "🎉".to_string()];
        let json = write_value(value.clone());
        assert_eq!(read_value::<Vec<String>>(&json).unwrap(), value);

        let value = vec![
            DateTimeAsMicroseconds::from_str("2024-01-02T03:04:05.123456Z").unwrap(),
            DateTimeAsMicroseconds::from_str("1970-01-01T00:00:00.000000Z").unwrap(),
        ];
        let json = write_value(value.clone());
        let back: Vec<DateTimeAsMicroseconds> = read_value(&json).unwrap();
        assert_eq!(
            back.iter().map(|v| v.unix_microseconds).collect::<Vec<_>>(),
            value
                .iter()
                .map(|v| v.unix_microseconds)
                .collect::<Vec<_>>()
        );
    }

    /// `Option<Vec<T>>` reads, though the writer has no impl to produce one: a generic
    /// `Option<T>` writer is not expressible under the current `IS_ARRAY` design, because
    /// `JsonObjectWriter` would wrap the `null` of a `None` in the array's brackets and emit
    /// `[null]`. The read side is what the acceptance criteria need, and it works against every
    /// form the wire can carry.
    #[test]
    fn option_of_vec_reads_null_present_and_absent() {
        assert_eq!(
            read_value::<Option<Vec<i32>>>(r#"{"v":null}"#).unwrap(),
            None
        );

        assert_eq!(
            read_value::<Option<Vec<i32>>>(r#"{"v":[1,2,3]}"#).unwrap(),
            Some(vec![1, 2, 3])
        );

        assert_eq!(
            read_value::<Option<Vec<i32>>>(r#"{"v":[]}"#).unwrap(),
            Some(vec![])
        );

        assert_eq!(
            Option::<Vec<i32>>::from_absent_json_value("v").unwrap(),
            None
        );

        // an inner element of the wrong type still errors through the Option
        assert!(read_value::<Option<Vec<i32>>>(r#"{"v":[1,"two"]}"#).is_err());
    }

    #[test]
    fn hash_map_round_trips_including_map_of_arrays() {
        let mut value: HashMap<String, i32> = HashMap::new();
        value.insert("a".to_string(), 1);
        value.insert("b".to_string(), 2);
        value.insert(r#"he said "hi""#.to_string(), 3);

        let json = write_value(value.clone());
        assert_eq!(read_value::<HashMap<String, i32>>(&json).unwrap(), value);

        // HashMap<String, Vec<T>>
        let mut value: HashMap<String, Vec<i32>> = HashMap::new();
        value.insert("nums".to_string(), vec![1, 2, 3]);
        value.insert("empty".to_string(), vec![]);

        let json = write_value(value.clone());
        assert_eq!(
            read_value::<HashMap<String, Vec<i32>>>(&json).unwrap(),
            value
        );

        // empty map
        let value: HashMap<String, i32> = HashMap::new();
        let json = write_value(value.clone());
        assert_eq!(json, r#"{"v":{}}"#);
        assert_eq!(read_value::<HashMap<String, i32>>(&json).unwrap(), value);
    }

    #[test]
    fn vec_of_hash_maps_round_trips() {
        let mut first: HashMap<String, i32> = HashMap::new();
        first.insert("a".to_string(), 1);
        let mut second: HashMap<String, i32> = HashMap::new();
        second.insert("b".to_string(), 2);

        let value = vec![first, second];
        let json = write_value(value.clone());

        assert_eq!(
            read_value::<Vec<HashMap<String, i32>>>(&json).unwrap(),
            value
        );
    }

    // ----- errors -----------------------------------------------------------------------------

    /// Every mismatch is a `JsonParseError` naming both sides. None of these panic - the whole
    /// grid is driven from hostile-ish input.
    #[test]
    fn type_mismatch_is_a_clear_error_never_a_panic() {
        let cases: Vec<(&str, &str)> = vec![
            (r#"{"v":"text"}"#, "string"),
            (r#"{"v":true}"#, "boolean"),
            (r#"{"v":null}"#, "null"),
            (r#"{"v":[1,2]}"#, "array"),
            (r#"{"v":{"a":1}}"#, "object"),
        ];

        for (json, expected_type) in cases {
            let err = read_value::<u64>(json)
                .unwrap_err_or_panic_message(&format!("u64 must reject {}", json));

            let message = err.to_string();
            assert!(
                message.contains(expected_type) && message.contains("u64"),
                "error for {} should name both '{}' and 'u64', got: {}",
                json,
                expected_type,
                message
            );
        }

        // a float is not an integer
        let err = read_value::<u64>(r#"{"v":1.5}"#).unwrap_err();
        assert!(
            err.to_string().contains("floating point"),
            "got: {}",
            err.to_string()
        );

        // and the structural readers reject scalars
        assert!(read_value::<Vec<i32>>(r#"{"v":5}"#).is_err());
        assert!(read_value::<HashMap<String, i32>>(r#"{"v":[1]}"#).is_err());
        assert!(read_value::<String>(r#"{"v":5}"#).is_err());
        assert!(read_value::<bool>(r#"{"v":1}"#).is_err());
        assert!(read_value::<DateTimeAsMicroseconds>(r#"{"v":"not a date"}"#).is_err());
    }

    trait UnwrapErrOrPanic<T> {
        fn unwrap_err_or_panic_message(self, message: &str) -> JsonParseError;
    }

    impl<T: std::fmt::Debug> UnwrapErrOrPanic<T> for Result<T, JsonParseError> {
        fn unwrap_err_or_panic_message(self, message: &str) -> JsonParseError {
            match self {
                Ok(value) => panic!("{}, but it returned {:?}", message, value),
                Err(err) => err,
            }
        }
    }

    #[test]
    fn out_of_range_integer_names_the_type() {
        let json = write_value(300u16);
        let err = read_value::<u8>(&json).unwrap_err();
        assert!(
            err.to_string().contains("out of range") && err.to_string().contains("u8"),
            "got: {}",
            err.to_string()
        );

        // negative into unsigned
        let json = write_value(-5i32);
        assert!(read_value::<u32>(&json).is_err());
        assert_eq!(read_value::<i32>(&json).unwrap(), -5);

        // u64::MAX does not fit i64
        let json = write_value(u64::MAX);
        assert!(read_value::<i64>(&json).is_err());
    }

    #[test]
    fn hostile_input_does_not_panic() {
        use crate::json_reader::JsonValue;

        // A quoted "string" whose inner bytes are not valid UTF-8. The value ref is built directly
        // (as `invalid_utf8_inside_string_does_not_panic` does) because the path reader rejects
        // such a payload before a reader would ever see it - this pins the readers themselves.
        let bytes: &[u8] = &[b'"', 0xFF, b'"'];
        let value = JsonValueRef::new(JsonValue::new(0, bytes.len()), bytes);

        assert!(String::from_json_value(&value).is_err());
        assert!(StrOrString::from_json_value(&value).is_err());
        assert!(u64::from_json_value(&value).is_err());
        assert!(f64::from_json_value(&value).is_err());
        assert!(bool::from_json_value(&value).is_err());
        assert!(DateTimeAsMicroseconds::from_json_value(&value).is_err());
        assert!(Vec::<u8>::from_json_value(&value).is_err());
        assert!(HashMap::<String, u8>::from_json_value(&value).is_err());
        assert!(Option::<u64>::from_json_value(&value).is_err());

        // An empty value ref must not index out of bounds either.
        let empty: &[u8] = &[];
        let value = JsonValueRef::new(JsonValue::new(0, 0), empty);

        assert!(u64::from_json_value(&value).is_err());
        assert!(String::from_json_value(&value).is_err());
        assert!(bool::from_json_value(&value).is_err());
        assert!(Vec::<u8>::from_json_value(&value).is_err());
        assert!(HashMap::<String, u8>::from_json_value(&value).is_err());
        // An empty token carries no value, so the optional reader treats it as absent.
        assert_eq!(Option::<u64>::from_json_value(&value).unwrap(), None);
    }

    // Mirrors the downstream generator: the trait must be nameable and usable through the crate's
    // public re-exports alone.
    fn parse_generic<'s, T: crate::json_reader::JsonValueReader<'s>>(
        value: &crate::json_reader::JsonValueRef<'s>,
    ) -> Result<T, crate::json_reader::JsonParseError> {
        T::from_json_value(value)
    }

    #[test]
    fn generic_wrapper_over_public_reexports() {
        let json = r#"{"v":7}"#;
        let value = crate::j_path::get_value(json.as_bytes(), "v")
            .unwrap()
            .unwrap();

        assert_eq!(parse_generic::<u8>(&value).unwrap(), 7u8);
        assert_eq!(parse_generic::<i128>(&value).unwrap(), 7i128);
        assert_eq!(parse_generic::<f64>(&value).unwrap(), 7f64);
        assert_eq!(parse_generic::<Option<u8>>(&value).unwrap(), Some(7u8));
    }
}

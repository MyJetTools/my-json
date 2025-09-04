use super::JsonValueWriter;

impl JsonValueWriter for u8 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for i8 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for u16 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for i16 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for u32 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for i32 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for u64 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for i64 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for usize {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for isize {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for f64 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonValueWriter for f32 {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

#[cfg(feature = "decimal")]
impl JsonValueWriter for rust_decimal::Decimal {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
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

impl JsonValueWriter for String {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push('"');
        crate::json_string_value::write_escaped_json_string_value(self, dest);
        dest.push('"');
    }
}

impl<'s> JsonValueWriter for &'s str {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push('"');
        crate::json_string_value::write_escaped_json_string_value(self, dest);
        dest.push('"');
    }
}

impl<'s> JsonValueWriter for &'s String {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        dest.push('"');
        crate::json_string_value::write_escaped_json_string_value(self, dest);
        dest.push('"');
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

impl<T: JsonValueWriter> JsonValueWriter for Vec<T> {
    const IS_ARRAY: bool = true;
    fn write(&self, dest: &mut String) {
        for (no, itm) in self.iter().enumerate() {
            if no > 0 {
                dest.push(',');
            }
            itm.write(dest);
        }
    }
}

impl<'s, T: JsonValueWriter> JsonValueWriter for &'s [T] {
    const IS_ARRAY: bool = true;
    fn write(&self, dest: &mut String) {
        for (no, itm) in self.iter().enumerate() {
            if no > 0 {
                dest.push(',');
            }
            itm.write(dest);
        }
    }
}

impl<'s, T: JsonValueWriter> JsonValueWriter for &'s Vec<T> {
    const IS_ARRAY: bool = true;
    fn write(&self, dest: &mut String) {
        for (no, itm) in self.iter().enumerate() {
            if no > 0 {
                dest.push(',');
            }
            itm.write(dest);
        }
    }
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
        let result = JsonObjectWriter::new().write("test", a).build();

        assert_eq!(result, r#"{"test":["1","2","3"]}"#);
    }
}

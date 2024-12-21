use super::JsonObject;

impl JsonObject for u8 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for i8 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for u16 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for i16 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for u32 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for i32 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for u64 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for i64 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for usize {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for isize {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for f64 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for f32 {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.to_string().as_str());
    }
}

impl JsonObject for bool {
    fn write_into(&self, dest: &mut String) {
        if *self {
            dest.push_str("true");
        } else {
            dest.push_str("false");
        }
    }
}

impl JsonObject for String {
    fn write_into(&self, dest: &mut String) {
        dest.push('"');
        crate::json_string_value::write_escaped_json_string_value(self, dest);
        dest.push('"');
    }
}

impl<'s> JsonObject for &'s str {
    fn write_into(&self, dest: &mut String) {
        dest.push('"');
        crate::json_string_value::write_escaped_json_string_value(self, dest);
        dest.push('"');
    }
}

impl<'s> JsonObject for &'s String {
    fn write_into(&self, dest: &mut String) {
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

impl<'s> JsonObject for RawJsonObject<'s> {
    fn write_into(&self, dest: &mut String) {
        dest.push_str(self.as_str());
    }
}

pub struct JsonNullValue;

impl JsonObject for JsonNullValue {
    fn write_into(&self, dest: &mut String) {
        dest.push_str("null");
    }
}

pub struct EmptyJsonArray;

impl JsonObject for EmptyJsonArray {
    fn write_into(&self, dest: &mut String) {
        dest.push_str("[]");
    }
}

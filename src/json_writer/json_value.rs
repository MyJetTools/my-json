use super::JsonObject;

impl JsonObject for u8 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for i8 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for u16 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for i16 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for u32 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for i32 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for u64 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for i64 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for usize {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for isize {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for f64 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for f32 {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonObject for bool {
    fn write_into(&self, dest: &mut Vec<u8>) {
        if *self {
            dest.extend_from_slice("true".as_bytes());
        } else {
            dest.extend_from_slice("false".as_bytes());
        }
    }
}

impl JsonObject for String {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.push('"' as u8);
        let escaped_json = crate::EscapedJsonString::new(self);
        dest.extend_from_slice(escaped_json.as_slice());
        dest.push('"' as u8);
    }
}

impl<'s> JsonObject for &'s str {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.push('"' as u8);
        let escaped_json = crate::EscapedJsonString::new(self);
        dest.extend_from_slice(escaped_json.as_slice());
        dest.push('"' as u8);
    }
}

impl<'s> JsonObject for &'s String {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.push('"' as u8);
        let escaped_json = crate::EscapedJsonString::new(self);
        dest.extend_from_slice(escaped_json.as_slice());
        dest.push('"' as u8);
    }
}

pub enum RawJsonObject<'s> {
    AsVec(Vec<u8>),
    AsSlice(&'s [u8]),
}

impl<'s> RawJsonObject<'s> {
    pub fn new(value: Vec<u8>) -> Self {
        RawJsonObject::AsVec(value)
    }

    pub fn as_slice(&'s self) -> &'s [u8] {
        match self {
            RawJsonObject::AsVec(vec) => vec.as_slice(),
            RawJsonObject::AsSlice(slice) => slice,
        }
    }
}

impl<'s> Into<RawJsonObject<'s>> for Vec<u8> {
    fn into(self) -> RawJsonObject<'s> {
        RawJsonObject::AsVec(self)
    }
}

impl<'s> JsonObject for RawJsonObject<'s> {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.as_slice());
    }
}

pub struct JsonNullValue;

impl JsonObject for JsonNullValue {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice("null".as_bytes());
    }
}

pub struct EmptyJsonArray;

impl JsonObject for EmptyJsonArray {
    fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice("[]".as_bytes());
    }
}

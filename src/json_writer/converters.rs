pub trait JsonValue {
    fn write_value(&self, dest: &mut Vec<u8>);
}

impl JsonValue for u8 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for i8 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for u16 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for i16 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for u32 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for i32 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for u64 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for i64 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for usize {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for isize {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for f64 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for f32 {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.to_string().as_bytes());
    }
}

impl JsonValue for bool {
    fn write_value(&self, dest: &mut Vec<u8>) {
        if *self {
            dest.extend_from_slice("true".as_bytes());
        } else {
            dest.extend_from_slice("false".as_bytes());
        }
    }
}

impl JsonValue for String {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.push('"' as u8);
        dest.extend_from_slice(self.to_string().as_bytes());
        dest.push('"' as u8);
    }
}

impl<'s> JsonValue for &'s str {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.push('"' as u8);
        dest.extend_from_slice(self.to_string().as_bytes());
        dest.push('"' as u8);
    }
}

impl<'s> JsonValue for &'s String {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.push('"' as u8);
        dest.extend_from_slice(self.to_string().as_bytes());
        dest.push('"' as u8);
    }
}

pub struct JsonNullValue;

impl JsonValue for JsonNullValue {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice("null".as_bytes());
    }
}

pub struct EmptyJsonArray;

impl JsonValue for EmptyJsonArray {
    fn write_value(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice("[]".as_bytes());
    }
}

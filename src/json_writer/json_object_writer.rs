use crate::EscapedJsonString;

use super::JsonArrayWriter;

pub struct JsonObjectWriter<'s> {
    first_element: bool,
    raw: &'s mut Vec<u8>,
}

impl<'s> JsonObjectWriter<'s> {
    pub fn new(raw: &'s mut Vec<u8>) -> Self {
        raw.push('{' as u8);
        Self {
            raw,
            first_element: true,
        }
    }

    pub fn has_written(&self) -> bool {
        !self.first_element
    }

    fn add_delimetr(&mut self) {
        if self.first_element {
            self.first_element = false;
        } else {
            self.raw.push(',' as u8);
        }
    }

    fn write_key(&mut self, key: &str) {
        self.raw.push('"' as u8);
        self.raw
            .extend_from_slice(EscapedJsonString::new(key).as_slice());
        self.raw.extend_from_slice("\":".as_bytes());
    }

    pub fn write_empty_array(&mut self, key: &str) {
        self.add_delimetr();
        self.write_key(key);
        self.raw.extend_from_slice("[]".as_bytes());
    }

    pub fn write_null_value(&mut self, key: &str) {
        self.add_delimetr();
        self.write_key(key);
        self.raw.extend_from_slice("null".as_bytes());
    }

    pub fn write_string_value(&mut self, key: &str, value: &str) {
        self.add_delimetr();

        self.write_key(key);

        self.raw.push('"' as u8);
        self.raw
            .extend_from_slice(EscapedJsonString::new(value).as_slice());
        self.raw.push('"' as u8);
    }

    pub fn write_bool_value(&mut self, key: &str, value: bool) {
        self.add_delimetr();
        self.write_key(key);

        if value {
            self.raw.extend_from_slice("true".as_bytes())
        } else {
            self.raw.extend_from_slice("false".as_bytes())
        }
    }

    pub fn write_raw_value(&mut self, key: &str, raw: &[u8]) {
        self.add_delimetr();

        self.write_key(key);

        self.raw.extend(raw);
    }

    pub fn start_writing_object(&'s mut self, key: &str) -> JsonObjectWriter<'s> {
        self.add_delimetr();
        self.write_key(key);
        JsonObjectWriter::new(self.raw)
    }

    pub fn start_writing_array(&'s mut self, key: &str) -> JsonArrayWriter<'s> {
        self.add_delimetr();
        self.write_key(key);
        JsonArrayWriter::new(self.raw)
    }
}

impl<'s> Drop for JsonObjectWriter<'s> {
    fn drop(&mut self) {
        self.raw.push('}' as u8)
    }
}

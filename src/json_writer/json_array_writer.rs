use crate::EscapedJsonString;

use super::JsonObjectWriter;

pub struct JsonArrayWriter<'s> {
    raw: &'s mut Vec<u8>,
    first_element: bool,
}

impl<'s> JsonArrayWriter<'s> {
    pub fn new(raw: &'s mut Vec<u8>) -> Self {
        raw.push('[' as u8);

        Self {
            raw,
            first_element: true,
        }
    }

    fn add_delimetr(&mut self) {
        if self.first_element {
            self.first_element = false;
        } else {
            self.raw.push(',' as u8);
        }
    }

    pub fn write_string_element(&mut self, value: &str) {
        self.add_delimetr();
        self.raw.push('"' as u8);
        self.raw
            .extend_from_slice(EscapedJsonString::new(value).as_slice());
        self.raw.push('"' as u8);
    }

    pub fn write_null_element(&mut self) {
        self.add_delimetr();
        self.raw.extend_from_slice("null".as_bytes());
    }

    pub fn write_number_element(&mut self, number: &str) {
        self.add_delimetr();
        self.raw.extend_from_slice(number.as_bytes());
    }

    pub fn write_raw_element(&mut self, raw: &[u8]) {
        self.add_delimetr();
        self.raw.extend_from_slice(raw);
    }

    pub fn start_writing_object(&'s mut self) -> JsonObjectWriter<'s> {
        self.add_delimetr();
        JsonObjectWriter::new(self.raw)
    }

    pub fn start_writing_array(&'s mut self) -> JsonArrayWriter<'s> {
        self.add_delimetr();
        JsonArrayWriter::new(self.raw)
    }
}

impl<'s> Drop for JsonArrayWriter<'s> {
    fn drop(&mut self) {
        self.raw.push(']' as u8);
    }
}

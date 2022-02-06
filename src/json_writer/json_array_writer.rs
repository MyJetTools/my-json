use crate::EscapedJsonString;

use super::JsonBuilder;

pub struct JsonArrayWriter {
    raw: Vec<u8>,
    first_element: bool,
}

impl JsonArrayWriter {
    pub fn new() -> Self {
        let mut raw = Vec::new();
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

    pub fn write_object<TJsonBuilder: JsonBuilder>(&mut self, object: TJsonBuilder) {
        self.add_delimetr();
        self.raw.extend(object.build());
    }
}

impl JsonBuilder for JsonArrayWriter {
    fn build(mut self) -> Vec<u8> {
        self.raw.push(']' as u8);
        self.raw
    }
}

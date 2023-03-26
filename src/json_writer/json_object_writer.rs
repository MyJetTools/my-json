use crate::EscapedJsonString;

use super::{JsonBuilder, JsonValue};

pub struct JsonObjectWriter {
    first_element: bool,
    raw: Vec<u8>,
}

impl JsonObjectWriter {
    pub fn new() -> Self {
        let mut raw = Vec::new();
        raw.push('{' as u8);
        Self {
            raw,
            first_element: true,
        }
    }

    pub fn has_written(&self) -> bool {
        !self.first_element
    }

    fn add_delimiter(&mut self) {
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

    pub fn write_value(&mut self, key: &str, value: impl JsonValue) {
        self.add_delimiter();
        self.write_key(key);

        value.write_value(&mut self.raw);
    }

    pub fn write_raw_value(&mut self, key: &str, raw: &[u8]) {
        self.add_delimiter();

        self.write_key(key);

        self.raw.extend(raw);
    }

    pub fn write_object<TJsonBuilder: JsonBuilder>(&mut self, key: &str, value: TJsonBuilder) {
        self.add_delimiter();

        self.write_key(key);

        self.raw.extend(value.build());
    }

    pub fn build(mut self) -> Vec<u8> {
        self.raw.push('}' as u8);
        self.raw
    }

    pub fn build_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.raw.as_slice());
        dest.push('}' as u8);
    }
}

impl JsonBuilder for JsonObjectWriter {
    fn build(self) -> Vec<u8> {
        self.build()
    }

    fn build_into(&self, dest: &mut Vec<u8>) {
        self.build_into(dest)
    }
}

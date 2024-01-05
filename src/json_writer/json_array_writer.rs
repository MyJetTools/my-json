use rust_extensions::StrOrString;

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

    fn add_delimiter(&mut self) {
        if self.first_element {
            self.first_element = false;
        } else {
            self.raw.push(',' as u8);
        }
    }

    pub fn write_string_element<'s>(&mut self, value: impl Into<StrOrString<'s>>) {
        self.add_delimiter();
        self.raw.push('"' as u8);
        self.raw
            .extend_from_slice(EscapedJsonString::new(value.into().as_str()).as_slice());
        self.raw.push('"' as u8);
    }

    pub fn write_null_element(&mut self) {
        self.add_delimiter();
        self.raw.extend_from_slice("null".as_bytes());
    }

    pub fn write_number_element(&mut self, number: &str) {
        self.add_delimiter();
        self.raw.extend_from_slice(number.as_bytes());
    }

    pub fn write_raw_element(&mut self, raw: &[u8]) {
        self.add_delimiter();
        self.raw.extend_from_slice(raw);
    }

    pub fn get_mut_to_write_raw_element(&mut self) -> &mut Vec<u8> {
        self.add_delimiter();
        &mut self.raw
    }

    pub fn build(mut self) -> Vec<u8> {
        self.raw.push(']' as u8);
        self.raw
    }

    pub fn write_object<TJsonBuilder: JsonBuilder>(&mut self, value: TJsonBuilder) {
        self.add_delimiter();
        self.raw.extend(value.build());
    }

    pub fn build_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.raw.as_slice());
        dest.push(']' as u8);
    }
}

impl JsonBuilder for JsonArrayWriter {
    fn build(self) -> Vec<u8> {
        self.build()
    }

    fn build_into(&self, dest: &mut Vec<u8>) {
        self.build_into(dest)
    }
}

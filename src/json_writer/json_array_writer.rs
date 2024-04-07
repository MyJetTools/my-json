use super::JsonObject;

pub struct JsonArrayWriter {
    raw: Vec<u8>,
    first_element: bool,
}

impl JsonArrayWriter {
    pub fn new() -> Self {
        let mut raw = Vec::new();
        raw.push(b'[');

        Self {
            raw,
            first_element: true,
        }
    }

    fn add_delimiter(&mut self) {
        if self.first_element {
            self.first_element = false;
        } else {
            self.raw.push(b',');
        }
    }

    pub fn write_null_element(&mut self) {
        self.add_delimiter();
        self.raw.extend_from_slice("null".as_bytes());
    }

    pub fn build(mut self) -> Vec<u8> {
        self.raw.push(b']');
        self.raw
    }

    pub fn write(&mut self, value: impl JsonObject) {
        self.add_delimiter();
        value.write_into(&mut self.raw);
    }

    pub fn build_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(&self.raw);
        dest.push(b']');
    }
}

impl JsonObject for JsonArrayWriter {
    fn write_into(&self, dest: &mut Vec<u8>) {
        self.build_into(dest)
    }
}

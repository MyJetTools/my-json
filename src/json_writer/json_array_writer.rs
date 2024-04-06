use super::JsonObject;

pub struct JsonArrayWriter {
    raw: String,
    first_element: bool,
}

impl JsonArrayWriter {
    pub fn new() -> Self {
        let mut raw = String::new();
        raw.push('[');

        Self {
            raw,
            first_element: true,
        }
    }

    fn add_delimiter(&mut self) {
        if self.first_element {
            self.first_element = false;
        } else {
            self.raw.push(',');
        }
    }

    pub fn write_null_element(&mut self) {
        self.add_delimiter();
        self.raw.push_str("null");
    }

    pub fn build(mut self) -> String {
        self.raw.push(']');
        self.raw
    }

    pub fn write(&mut self, value: impl JsonObject) {
        self.add_delimiter();
        value.write_into(&mut self.raw);
    }

    pub fn build_into(&self, dest: &mut String) {
        dest.push_str(&self.raw);
        dest.push(']');
    }
}

impl JsonObject for JsonArrayWriter {
    fn write_into(&self, dest: &mut String) {
        self.build_into(dest)
    }
}

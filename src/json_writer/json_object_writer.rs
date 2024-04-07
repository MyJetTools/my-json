use super::JsonObject;

pub struct JsonObjectWriter {
    first_element: bool,
    raw: Vec<u8>,
}

impl JsonObjectWriter {
    pub fn new() -> Self {
        let mut raw = Vec::new();
        raw.push(b'{');
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
            self.raw.push(b',');
        }
    }

    fn write_key(&mut self, key: &str) {
        self.raw.push(b'"');

        crate::json_string_value::write_escaped_json_string_value(key, &mut self.raw);
        self.raw.extend_from_slice("\":".as_bytes());
    }

    pub fn write(&mut self, key: &str, value: impl JsonObject) {
        self.add_delimiter();
        self.write_key(key);
        value.write_into(&mut self.raw);
    }

    pub fn build(mut self) -> Vec<u8> {
        self.raw.push(b'}');
        self.raw
    }

    pub fn build_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(&self.raw);
        dest.push(b'}');
    }
}

impl JsonObject for JsonObjectWriter {
    fn write_into(&self, dest: &mut Vec<u8>) {
        self.build_into(dest)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn basic_test() {
        let mut json_write = super::JsonObjectWriter::new();

        json_write.write("key", "value");

        json_write.write("key2", "'value'");

        let result = json_write.build();

        assert_eq!(
            "{\"key\":\"value\",\"key2\":\"\\'value\\'\"}",
            std::str::from_utf8(&result).unwrap()
        );
    }
}

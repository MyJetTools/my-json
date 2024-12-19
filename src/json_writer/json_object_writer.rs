use super::{JsonArrayWriter, JsonObject};

pub struct JsonObjectWriter {
    first_element: bool,
    raw: Option<Vec<u8>>,
}

impl JsonObjectWriter {
    pub fn new() -> Self {
        let mut raw = Vec::new();
        raw.push(b'{');

        Self {
            first_element: true,
            raw: Some(raw),
        }
    }

    pub fn from_vec(mut raw: Vec<u8>) -> Self {
        raw.push(b'{');
        Self {
            first_element: true,
            raw: Some(raw),
        }
    }

    pub fn has_written(&self) -> bool {
        !self.first_element
    }

    fn add_delimiter(&mut self) {
        if self.first_element {
            self.first_element = false;
        } else {
            self.raw.as_mut().unwrap().push(b',');
        }
    }

    fn write_key(&mut self, key: &str) {
        let raw = self.raw.as_mut().unwrap();
        raw.push(b'"');

        crate::json_string_value::write_escaped_json_string_value(key, raw);
        raw.extend_from_slice("\":".as_bytes());
    }

    pub fn write_json_object(
        &mut self,
        key: &str,
        write_object: impl Fn(&mut JsonObjectWriter) -> (),
    ) {
        self.add_delimiter();
        self.write_key(key);

        let raw = self.raw.take().unwrap();

        let mut json_object_writer = Self::from_vec(raw);

        write_object(&mut json_object_writer);

        let raw = json_object_writer.build();
        self.raw = Some(raw);
    }

    pub fn write_json_array(
        &mut self,
        key: &str,
        write_array: impl Fn(&mut JsonArrayWriter) -> (),
    ) {
        self.add_delimiter();
        self.write_key(key);

        let raw = self.raw.take().unwrap();

        let mut json_array_writer = JsonArrayWriter::from_vec(raw);

        write_array(&mut json_array_writer);

        let raw = json_array_writer.build();
        self.raw = Some(raw);
    }

    pub fn write(&mut self, key: &str, value: impl JsonObject) {
        self.add_delimiter();
        self.write_key(key);

        value.write_into(self.raw.as_mut().unwrap());
    }

    pub fn build(mut self) -> Vec<u8> {
        let mut raw = self.raw.take().unwrap();
        raw.push(b'}');
        raw
    }

    pub fn build_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.raw.as_ref().unwrap());
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

    #[test]
    fn test_nested_object_write() {
        let mut json_writer = super::JsonObjectWriter::new();

        json_writer.write("key1", "value1");

        json_writer.write_json_object("key2", |json_writer| {
            json_writer.write("key3", "value3");
            json_writer.write("key4", 54);
        });

        let result = json_writer.build();

        let result = String::from_utf8(result).unwrap();

        println!("{}", result);
    }

    #[test]
    fn test_nested_array_write() {
        let mut json_writer = super::JsonObjectWriter::new();

        json_writer.write("key1", "value1");

        json_writer.write_json_array("key2", |json_array_writer| {
            json_array_writer.write_json_object(|json_object| {
                json_object.write("key3", "value3");
                json_object.write("key4", 54);
            });

            json_array_writer.write_json_object(|json_object| {
                json_object.write("key3", "value5");
                json_object.write("key4", 55);
            });
        });

        let result = json_writer.build();

        let result = String::from_utf8(result).unwrap();

        assert_eq!("{\"key1\":\"value1\",\"key2\":[{\"key3\":\"value3\",\"key4\":54},{\"key3\":\"value5\",\"key4\":55}]}", result.as_str());
    }
}

use super::{JsonObject, JsonObjectWriter};

pub struct JsonArrayWriter {
    raw: Option<String>,
    first_element: bool,
}

impl JsonArrayWriter {
    pub fn new() -> Self {
        let mut raw = String::new();
        raw.push('[');

        Self {
            raw: Some(raw),
            first_element: true,
        }
    }

    pub fn from_string(mut raw: String) -> Self {
        raw.push('[');
        Self {
            first_element: true,
            raw: Some(raw),
        }
    }

    fn add_delimiter(&mut self) {
        if self.first_element {
            self.first_element = false;
        } else {
            self.raw.as_mut().unwrap().push(',');
        }
    }

    pub fn write_null_element(&mut self) {
        self.add_delimiter();
        self.raw.as_mut().unwrap().push_str("null");
    }

    pub fn build(mut self) -> String {
        let mut raw = self.raw.take().unwrap();
        raw.push(']');
        raw
    }

    pub fn write(&mut self, value: impl JsonObject) {
        self.add_delimiter();
        let raw = self.raw.as_mut().unwrap();
        value.write_into(raw);
    }

    pub fn write_json_object(&mut self, write_object: impl Fn(&mut JsonObjectWriter) -> ()) {
        self.add_delimiter();

        let raw = self.raw.take().unwrap();

        let mut json_object_writer = JsonObjectWriter::from_string(raw);

        write_object(&mut json_object_writer);

        let raw = json_object_writer.build();
        self.raw = Some(raw);
    }

    pub fn build_into(&self, dest: &mut String) {
        dest.push_str(self.raw.as_ref().unwrap());
        dest.push(']');
    }

    pub fn write_into(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.raw.as_ref().unwrap().as_bytes());
        dest.push(b']');
    }
}

impl JsonObject for JsonArrayWriter {
    fn write_into(&self, dest: &mut String) {
        self.build_into(dest)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_array_write() {
        let mut json_array_writer = super::JsonArrayWriter::new();

        json_array_writer.write_json_object(|json_object| {
            json_object.write("key1", "value1");
            json_object.write("key2", "value2");
        });

        json_array_writer.write_json_object(|json_object| {
            json_object.write("key1", "value1");
            json_object.write("key2", "value2");
        });

        let result = json_array_writer.build();

        assert_eq!(
            result,
            "[{\"key1\":\"value1\",\"key2\":\"value2\"},{\"key1\":\"value1\",\"key2\":\"value2\"}]"
        );
    }
}

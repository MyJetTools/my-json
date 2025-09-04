use super::*;

pub struct JsonObjectWriter {
    first_element: bool,
    raw: Option<String>,
}

impl JsonObjectWriter {
    pub fn new() -> Self {
        let mut raw = String::new();
        raw.push('{');

        Self {
            first_element: true,
            raw: Some(raw),
        }
    }

    pub fn from_string(mut raw: String) -> Self {
        raw.push('{');
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
            self.raw.as_mut().unwrap().push(',');
        }
    }

    fn write_key(&mut self, key: &str) {
        let raw = self.raw.as_mut().unwrap();
        raw.push('"');

        crate::json_string_value::write_escaped_json_string_value(key, raw);
        raw.push_str("\":");
    }

    pub fn write_json_object(
        mut self,
        key: &str,
        write_object: impl Fn(JsonObjectWriter) -> JsonObjectWriter,
    ) -> Self {
        self.add_delimiter();
        self.write_key(key);

        let raw = self.raw.take().unwrap();

        let mut json_object_writer = Self::from_string(raw);

        json_object_writer = write_object(json_object_writer);

        let raw = json_object_writer.build();
        self.raw = Some(raw);

        self
    }

    pub fn write_json_array(
        mut self,
        key: &str,
        write_array: impl Fn(JsonArrayWriter) -> JsonArrayWriter,
    ) -> Self {
        self.add_delimiter();
        self.write_key(key);

        let raw = self.raw.take().unwrap();

        let json_array_writer = write_array(JsonArrayWriter::from_string(raw));

        let raw = json_array_writer.build();
        self.raw = Some(raw);
        self
    }

    pub fn write<TJsonValue: JsonValueWriter>(mut self, key: &str, value: TJsonValue) -> Self {
        self.add_delimiter();
        self.write_key(key);

        let raw = self.raw.as_mut().unwrap();

        if TJsonValue::IS_ARRAY {
            raw.push('[');
        }

        value.write(raw);

        if TJsonValue::IS_ARRAY {
            raw.push(']');
        }

        self
    }

    pub fn write_ref<TJsonValue: JsonValueWriter>(mut self, key: &str, value: &TJsonValue) -> Self {
        self.add_delimiter();
        self.write_key(key);

        let raw = self.raw.as_mut().unwrap();

        if TJsonValue::IS_ARRAY {
            raw.push('[');
        }

        value.write(raw);

        if TJsonValue::IS_ARRAY {
            raw.push(']');
        }

        self
    }

    pub fn write_if_some<TJsonValue: JsonValueWriter>(
        mut self,
        key: &str,
        value: Option<TJsonValue>,
    ) -> Self {
        let Some(value) = value else {
            return self;
        };

        self.add_delimiter();
        self.write_key(key);

        let raw = self.raw.as_mut().unwrap();

        if TJsonValue::IS_ARRAY {
            raw.push('[');
        }

        value.write(raw);

        if TJsonValue::IS_ARRAY {
            raw.push(']');
        }

        self
    }

    pub fn write_if_some_ref<TJsonValue: JsonValueWriter>(
        mut self,
        key: &str,
        value: Option<&TJsonValue>,
    ) -> Self {
        let Some(value) = value else {
            return self;
        };

        self.add_delimiter();
        self.write_key(key);

        let raw = self.raw.as_mut().unwrap();

        if TJsonValue::IS_ARRAY {
            raw.push('[');
        }

        value.write(raw);

        if TJsonValue::IS_ARRAY {
            raw.push(']');
        }

        self
    }

    pub fn build(mut self) -> String {
        let mut raw = self.raw.take().unwrap();
        raw.push('}');
        raw
    }

    pub fn build_into(&self, dest: &mut String) {
        dest.push_str(self.raw.as_ref().unwrap());
        dest.push('}');
    }

    pub fn write_into_vec(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.raw.as_ref().unwrap().as_bytes());
        dest.push(b'}');
    }
}

impl JsonValueWriter for JsonObjectWriter {
    const IS_ARRAY: bool = false;
    fn write(&self, dest: &mut String) {
        self.build_into(dest)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn basic_test() {
        let result = super::JsonObjectWriter::new()
            .write("key", "value")
            .write("key2", "'value'")
            .build();

        assert_eq!("{\"key\":\"value\",\"key2\":\"\'value\'\"}", result);
    }

    #[test]
    fn test_nested_object_write() {
        let json_writer = super::JsonObjectWriter::new()
            .write("key1", "value1")
            .write_json_object("key2", |json_writer| {
                json_writer.write("key3", "value3").write("key4", 54)
            });

        let result = json_writer.build();

        println!("{}", result);
    }

    #[test]
    fn test_nested_array_write() {
        let result = super::JsonObjectWriter::new()
            .write("key1", "value1")
            .write_json_array("key2", |json_array_writer| {
                json_array_writer
                    .write_json_object(|json_object| {
                        json_object.write("key3", "value3").write("key4", 54)
                    })
                    .write_json_object(|json_object| {
                        json_object.write("key3", "value5").write("key4", 55)
                    })
            })
            .build();

        assert_eq!("{\"key1\":\"value1\",\"key2\":[{\"key3\":\"value3\",\"key4\":54},{\"key3\":\"value5\",\"key4\":55}]}", result.as_str());
    }
}

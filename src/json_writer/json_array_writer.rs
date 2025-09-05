use super::*;

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

    pub fn write_null_element(mut self) -> Self {
        self.add_delimiter();
        self.raw.as_mut().unwrap().push_str("null");
        self
    }

    pub fn build(mut self) -> String {
        let mut raw = self.raw.take().unwrap();
        raw.push(']');
        raw
    }

    pub fn write(mut self, value: impl JsonValueWriter) -> Self {
        self.add_delimiter();
        let raw = self.raw.as_mut().unwrap();
        value.write(raw);
        self
    }

    pub fn write_ref(mut self, value: &impl JsonValueWriter) -> Self {
        self.add_delimiter();
        let raw = self.raw.as_mut().unwrap();
        value.write(raw);
        self
    }

    pub fn write_iter<TJsonValueWriter: JsonValueWriter>(
        mut self,
        values: impl Iterator<Item = TJsonValueWriter>,
    ) -> Self {
        self.add_delimiter();
        let raw = self.raw.as_mut().unwrap();

        for itm in values {
            itm.write(raw);
        }

        self
    }

    pub fn write_slice<TJsonValueWriter: JsonValueWriter>(
        mut self,
        values: &[TJsonValueWriter],
    ) -> Self {
        self.add_delimiter();
        let raw = self.raw.as_mut().unwrap();

        for itm in values {
            itm.write(raw);
        }

        self
    }

    pub fn write_json_object(
        mut self,
        write_object: impl Fn(JsonObjectWriter) -> JsonObjectWriter,
    ) -> Self {
        self.add_delimiter();

        let raw = self.raw.take().unwrap();

        let json_object_writer = write_object(JsonObjectWriter::from_string(raw));

        let raw = json_object_writer.build();
        self.raw = Some(raw);

        self
    }

    pub fn build_into(&self, dest: &mut String) {
        dest.push_str(self.raw.as_ref().unwrap());
        dest.push(']');
    }

    pub fn write_into_vec(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.raw.as_ref().unwrap().as_bytes());
        dest.push(b']');
    }
}

impl JsonValueWriter for JsonArrayWriter {
    const IS_ARRAY: bool = true;
    fn write(&self, dest: &mut String) {
        self.build_into(dest)
    }
}

#[cfg(test)]
mod tests {
    use crate::json_writer::EmptyJsonArray;

    #[test]
    fn test_array_write() {
        let result = super::JsonArrayWriter::new()
            .write_json_object(|json_object| {
                json_object.write("key1", "value1").write("key2", "value2")
            })
            .write_json_object(|json_object| {
                json_object.write("key1", "value1").write("key2", "value2")
            })
            .build();

        assert_eq!(
            result,
            "[{\"key1\":\"value1\",\"key2\":\"value2\"},{\"key1\":\"value1\",\"key2\":\"value2\"}]"
        );
    }

    #[test]
    fn test_array_write_to_object() {
        let result = super::JsonObjectWriter::new()
            .write_json_array("array", |array_writer| {
                array_writer.write_json_object(|json_object| {
                    json_object.write("key1", "value1").write("key2", "value2")
                })
            })
            .build();

        assert_eq!(r#"{"array":[{"key1":"value1","key2":"value2"}]}"#, result);
    }

    #[test]
    fn test_array_write_to_object_2() {
        let json_object = super::JsonObjectWriter::new()
            .write("key1", "value1")
            .write("key2", "value2");
        let result = super::JsonObjectWriter::new()
            .write_iter("array", [json_object].into_iter())
            .build();

        assert_eq!(r#"{"array":[{"key1":"value1","key2":"value2"}]}"#, result);
    }

    #[test]
    fn test_empty_array() {
        let result = super::JsonObjectWriter::new()
            .write("array", EmptyJsonArray)
            .build();

        assert_eq!(r#"{"array":[]}"#, result);
    }
}

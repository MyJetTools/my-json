use crate::EscapedJsonString;

pub trait JsonBuilder {
    fn build(self) -> Vec<u8>;
}

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

    fn add_delimetr(&mut self) {
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
        self.raw.push('"' as u8);
    }

    pub fn write_empty_array(&mut self, key: &str) {
        self.add_delimetr();
        self.write_key(key);
        self.raw.extend_from_slice(":[]".as_bytes());
    }

    pub fn write_null_value(&mut self, key: &str) {
        self.add_delimetr();
        self.write_key(key);
        self.raw.extend_from_slice(":null".as_bytes());
    }

    pub fn write_array_object_element(&mut self, object: JsonObjectWriter) {
        self.add_delimetr();
        self.raw.extend(object.build());
    }

    pub fn write_string_value(&mut self, key: &str, value: &str) {
        self.add_delimetr();
        let data_to_add = format!(
            "\"{}\":\"{}\"",
            EscapedJsonString::new(key).as_str(),
            EscapedJsonString::new(value).as_str()
        );
        self.raw.extend(data_to_add.into_bytes());
    }

    pub fn write_bool_value(&mut self, key: &str, value: bool) {
        self.add_delimetr();
        let data_to_add = format!("\"{}\":{}", EscapedJsonString::new(key).as_str(), value);
        self.raw.extend(data_to_add.into_bytes());
    }

    pub fn write_object<TJsonBuilder: JsonBuilder>(&mut self, key: &str, object: TJsonBuilder) {
        self.add_delimetr();
        let data_to_add = format!("\"{}\":", EscapedJsonString::new(key).as_str());

        self.raw.extend(data_to_add.into_bytes());
        self.raw.extend(object.build());
    }

    pub fn build(mut self) -> Vec<u8> {
        self.raw.push('}' as u8);
        self.raw
    }
}

impl JsonBuilder for JsonObjectWriter {
    fn build(self) -> Vec<u8> {
        self.build()
    }
}

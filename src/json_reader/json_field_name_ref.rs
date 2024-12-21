use rust_extensions::StrOrString;

use crate::json_reader::JsonParseError;

use super::JsonContentOffset;

pub struct JsonFieldNameRef<'s> {
    pub data: JsonContentOffset,
    src_json: &'s [u8],
}

impl<'s> JsonFieldNameRef<'s> {
    pub fn new(data: JsonContentOffset, src_json: &'s [u8]) -> Self {
        Self { data, src_json }
    }
    pub fn as_raw_str(&'s self) -> Result<&'s str, JsonParseError> {
        self.data.as_raw_str(&self.src_json)
    }

    pub fn as_str(&'s self) -> Result<StrOrString<'s>, JsonParseError> {
        self.data.as_str(&self.src_json)
    }

    pub fn as_slice(&'s self) -> &'s [u8] {
        &self.src_json[self.data.start..self.data.end]
    }

    pub fn as_unescaped_str(&'s self) -> Result<&'s str, JsonParseError> {
        self.data.as_unescaped_str(&self.src_json)
    }
}

#[cfg(test)]
mod tests {

    use crate::json_reader::*;

    #[test]
    pub fn test_simple_parse() {
        let src_data = "{\"name1\":\"123\", \"name2\":true,       \"name3\":null, \"name4\":0.12, \"name5\":{\"a\":\"b\"}}".as_bytes();

        let parser = JsonFirstLineIterator::new(src_data.as_slice());

        let (name, value) = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name1\"", name.as_raw_str().unwrap());
        assert_eq!("\"123\"", value.as_raw_str().unwrap());

        let (name, value) = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name2\"", name.as_raw_str().unwrap());
        assert_eq!("true", value.as_raw_str().unwrap());

        let (name, value) = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name3\"", name.as_raw_str().unwrap());
        assert_eq!("null", value.as_raw_str().unwrap());

        let (name, value) = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name4\"", name.as_raw_str().unwrap());
        assert_eq!("0.12", value.as_raw_str().unwrap());

        let (name, value) = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name5\"", name.as_raw_str().unwrap());
        assert_eq!("{\"a\":\"b\"}", value.as_raw_str().unwrap());

        let item = parser.get_next();

        assert_eq!(true, item.is_none());
    }

    #[test]
    fn test_json_first_line() {
        let fist_line = r#"{"processId":"8269e2ac-fa3b-419a-8e65-1a606ba07942","sellAmount":0.4,"buyAmount":null,"sellAsset":"ETH","buyAsset":"USDT"}"#.as_bytes();

        let parser = JsonFirstLineIterator::new(fist_line.as_slice());

        let (name, value) = parser.get_next().unwrap().unwrap();

        assert_eq!("processId", name.as_str().unwrap().as_str());
        assert_eq!(
            "8269e2ac-fa3b-419a-8e65-1a606ba07942",
            value.as_str().unwrap().as_str()
        );

        let (name, value) = parser.get_next().unwrap().unwrap();

        assert_eq!("sellAmount", name.as_str().unwrap().as_str());
        assert_eq!("0.4", value.as_str().unwrap().as_str());

        let (name, value) = parser.get_next().unwrap().unwrap();

        assert_eq!("buyAmount", name.as_str().unwrap().as_str());

        assert!(value.is_null());
    }

    #[test]
    pub fn test_with_array_inside_json_l_split() {
        let json = r###"{"names":[{"company_name": "Company \"A\""},
                {
                    "company_name": "Company \"B\""
                },
                {
                    "company_name": "Company \"C\""
                }
            ],
            "registered_address": "Addr",
            "retrieved_at": "2010-02-23"
        }"###
            .as_bytes();

        let first_line_reader = JsonFirstLineIterator::new(json.as_slice());

        while let Some(sub_json) = first_line_reader.get_next() {
            let (name, value) = sub_json.unwrap();
            println!("{}", name.as_str().unwrap().as_str(),);
            println!("{:?}", value.as_raw_str());
        }
    }

    #[test]
    pub fn read_first_line_with_empty_value() {
        let json = r###"{
            "": true,
            "AD": false,
            "CD": false,
            "DK": false,
            "HD": false,
            "Note:": true,
            "SI": false,
            "UT": false,
            "VÃ": false
        }"###
            .as_bytes();

        let first_line_reader = JsonFirstLineIterator::new(json.as_ref());

        while let Some(sub_json) = first_line_reader.get_next() {
            let (name, value) = sub_json.unwrap();
            println!(
                "{}:{}",
                name.as_raw_str().unwrap(),
                value.as_raw_str().unwrap()
            );
        }
    }
}

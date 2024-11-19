use rust_extensions::StrOrString;

use crate::json_reader::JsonParseError;

use super::JsonFieldName;

pub struct JsonFieldNameRef<'s> {
    src: JsonFieldName,
    field_name_slice: &'s [u8],
}

impl<'s> JsonFieldNameRef<'s> {
    pub fn new(src: JsonFieldName, field_name_slice: &'s [u8]) -> Self {
        Self {
            src,
            field_name_slice,
        }
    }
    pub fn as_raw_str(&'s self) -> Result<&'s str, JsonParseError> {
        self.src.as_raw_str(&self.field_name_slice)
    }

    pub fn as_str(&'s self) -> Result<StrOrString<'s>, JsonParseError> {
        self.src.as_str(&self.field_name_slice)
    }

    pub fn as_unescaped_name(&'s self) -> Result<&'s str, JsonParseError> {
        self.src.as_unescaped_name(&self.field_name_slice)
    }
}

#[cfg(test)]
mod tests {
    use rust_extensions::array_of_bytes_iterator::*;

    use crate::json_reader::*;

    #[test]
    pub fn test_simple_parse() {
        let src_data = "{\"name1\":\"123\", \"name2\":true,       \"name3\":null, \"name4\":0.12, \"name5\":{\"a\":\"b\"}}".as_bytes();

        let slice_iterator = SliceIterator::new(src_data);
        let parser = JsonFirstLineReader::new(slice_iterator);

        let item = parser.get_next().unwrap().unwrap().as_ref(&parser);

        assert_eq!("\"name1\"", item.name.as_raw_str().unwrap());
        assert_eq!("\"123\"", item.value.as_raw_str().unwrap());

        let item = parser.get_next().unwrap().unwrap().as_ref(&parser);

        assert_eq!("\"name2\"", item.name.as_raw_str().unwrap());
        assert_eq!("true", item.value.as_raw_str().unwrap());

        let item = parser.get_next().unwrap().unwrap().as_ref(&parser);

        assert_eq!("\"name3\"", item.name.as_raw_str().unwrap());
        assert_eq!("null", item.value.as_raw_str().unwrap());

        let item = parser.get_next().unwrap().unwrap().as_ref(&parser);

        assert_eq!("\"name4\"", item.name.as_raw_str().unwrap());
        assert_eq!("0.12", item.value.as_raw_str().unwrap());

        let item = parser.get_next().unwrap().unwrap().as_ref(&parser);

        assert_eq!("\"name5\"", item.name.as_raw_str().unwrap());
        assert_eq!("{\"a\":\"b\"}", item.value.as_raw_str().unwrap());

        let item = parser.get_next();

        assert_eq!(true, item.is_none());
    }

    #[test]
    fn test_json_first_line() {
        let fist_line = r#"{"processId":"8269e2ac-fa3b-419a-8e65-1a606ba07942","sellAmount":0.4,"buyAmount":null,"sellAsset":"ETH","buyAsset":"USDT"}"#.as_bytes();

        let slice_iterator = SliceIterator::new(fist_line);

        let parser = JsonFirstLineReader::new(slice_iterator);

        let item = parser.get_next().unwrap().unwrap().as_ref(&parser);

        assert_eq!("processId", item.name.as_str().unwrap().as_str());
        assert_eq!(
            "8269e2ac-fa3b-419a-8e65-1a606ba07942",
            item.value.as_str().unwrap().as_str()
        );

        let item = parser.get_next().unwrap().unwrap().as_ref(&parser);

        assert_eq!("sellAmount", item.name.as_str().unwrap().as_str());
        assert_eq!("0.4", item.value.as_str().unwrap().as_str());

        let item = parser.get_next().unwrap().unwrap().as_ref(&parser);

        assert_eq!("buyAmount", item.name.as_str().unwrap().as_str());

        assert!(item.value.is_null());
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

        let slice_iterator = SliceIterator::new(json);

        let first_line_reader = JsonFirstLineReader::new(slice_iterator);

        while let Some(sub_json) = first_line_reader.get_next() {
            let sub_json = sub_json.unwrap().as_ref(&first_line_reader);
            println!("{}", sub_json.name.as_str().unwrap().as_str(),);
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

        let slice_iterator = SliceIterator::new(json);

        let first_line_reader = JsonFirstLineReader::new(slice_iterator);

        while let Some(sub_json) = first_line_reader.get_next() {
            let sub_json = sub_json.unwrap().as_ref(&first_line_reader);
            println!(
                "{}:{}",
                sub_json.name.as_raw_str().unwrap(),
                sub_json.value.as_raw_str().unwrap()
            );
        }
    }
}

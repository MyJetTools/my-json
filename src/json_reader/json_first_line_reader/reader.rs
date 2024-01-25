use rust_extensions::array_of_bytes_iterator::*;

use super::super::json_first_line_reader::states::{
    LookingForNextKeyStartState, ReadingNonStringValueState,
};
use super::{
    super::consts,
    states::{LookingForJsonValueStartState, ReadingObjectValueState},
    JsonFirstLine,
};

use super::super::JsonParseError;
use super::{
    read_mode::ReadMode,
    states::{LookingForJsonTokenState, ReadingStringState},
};

pub struct JsonFirstLineReader<TArrayOfBytesIterator: ArrayOfBytesIterator> {
    raw: TArrayOfBytesIterator,
    read_mode: ReadMode,
    start_pos: usize,
}

impl<TArrayOfBytesIterator: ArrayOfBytesIterator> JsonFirstLineReader<TArrayOfBytesIterator> {
    pub fn new(raw: TArrayOfBytesIterator) -> Self {
        let start_pos = raw.get_pos();
        Self {
            raw,
            read_mode: ReadMode::LookingForOpenJson(LookingForJsonTokenState::new(
                consts::OPEN_BRACKET,
            )),
            start_pos,
        }
    }

    pub fn get_next<'s>(&'s mut self) -> Option<Result<JsonFirstLine<'s>, JsonParseError>> {
        let mut key_start = None;
        let mut key_end = None;
        let mut value_start = None;
        loop {
            let result = self.read_mode.read_next(&mut self.raw);

            if let Err(err) = result {
                return Some(Err(err));
            }

            match result.unwrap() {
                super::ReadResult::OpenJsonFound => {
                    self.read_mode = ReadMode::LookingForJsonKeyStart(
                        LookingForJsonTokenState::new(consts::DOUBLE_QUOTE),
                    );
                }
                super::ReadResult::KeyStartFound(pos) => {
                    key_start = Some(pos);
                    self.read_mode = ReadMode::ReadingKey(ReadingStringState);
                }
                super::ReadResult::KeyEndFound(pos) => {
                    key_end = Some(pos);
                    self.read_mode = ReadMode::LookingForKeyValueSeparator(
                        LookingForJsonTokenState::new(consts::DOUBLE_COLUMN),
                    );
                }
                super::ReadResult::KeyValueSeparatorFound => {
                    self.read_mode = ReadMode::LookingForValueStart(LookingForJsonValueStartState);
                }
                super::ReadResult::FoundStringValueStart(pos) => {
                    value_start = Some(pos);
                    self.read_mode = ReadMode::ReadingStringValue(ReadingStringState);
                }
                super::ReadResult::FoundNonStringValueStart(pos) => {
                    value_start = Some(pos);
                    self.read_mode = ReadMode::ReadingNonStringValue(ReadingNonStringValueState);
                }
                super::ReadResult::FoundObjectOrArrayValueStart(pos) => {
                    value_start = Some(pos);
                    self.read_mode = ReadMode::ReadingObjectValue(ReadingObjectValueState);
                }
                super::ReadResult::ValueEndFound(pos) => {
                    self.read_mode = ReadMode::LookingForNextKeyStart(LookingForNextKeyStartState);

                    return Some(Ok(JsonFirstLine {
                        data: self.raw.get_slice_to_current_pos(self.start_pos),
                        name_start: key_start.unwrap(),
                        name_end: key_end.unwrap() + 1,
                        value_start: value_start.unwrap(),
                        value_end: pos + 1,
                    }));
                }
                super::ReadResult::EndOfJson => {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_extensions::array_of_bytes_iterator::*;

    use super::*;

    #[test]
    pub fn test_simple_parse() {
        let src_data = "{\"name1\":\"123\", \"name2\":true,       \"name3\":null, \"name4\":0.12}";

        let slice_iterator = SliceIterator::from_str(src_data);
        let mut parser = JsonFirstLineReader::new(slice_iterator);

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name1\"", item.get_raw_name().unwrap());
        assert_eq!("\"123\"", item.get_raw_value().unwrap());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name2\"", item.get_raw_name().unwrap());
        assert_eq!("true", item.get_raw_value().unwrap());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name3\"", item.get_raw_name().unwrap());
        assert_eq!("null", item.get_raw_value().unwrap());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name4\"", item.get_raw_name().unwrap());
        assert_eq!("0.12", item.get_raw_value().unwrap());

        let item = parser.get_next();

        assert_eq!(true, item.is_none());
    }

    #[test]
    fn test_json_first_line() {
        let fist_line = r#"{"processId":"8269e2ac-fa3b-419a-8e65-1a606ba07942","sellAmount":0.4,"buyAmount":null,"sellAsset":"ETH","buyAsset":"USDT"}"#;

        let slice_iterator = SliceIterator::from_str(fist_line);

        let mut parser = JsonFirstLineReader::new(slice_iterator);

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("processId", item.get_name().unwrap());
        assert_eq!(
            "8269e2ac-fa3b-419a-8e65-1a606ba07942",
            item.get_value().unwrap().as_str().unwrap()
        );

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("sellAmount", item.get_name().unwrap());
        assert_eq!("0.4", item.get_value().unwrap().as_str().unwrap());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("buyAmount", item.get_name().unwrap());

        let value = item.get_value().unwrap();
        assert!(value.is_null());
    }
}

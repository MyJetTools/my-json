use rust_extensions::array_of_bytes_iterator::*;

use crate::json_reader::JsonParseError;

use super::*;

pub fn find_the_end_of_the_object_value(
    src: &mut impl ArrayOfBytesIterator,
    value_start: u8,
) -> Result<usize, JsonParseError> {
    if is_number(value_start) {
        let result = find_the_end_of_the_number(src)?;
        return Ok(result.pos);
    }

    match value_start {
        crate::consts::START_OF_TRUE_LOWER_CASE => {
            check_json_symbol(src, "true")?;
            return Ok(src.get_pos());
        }
        crate::consts::START_OF_TRUE_UPPER_CASE => {
            check_json_symbol(src, "true")?;
            return Ok(src.get_pos());
        }

        crate::consts::START_OF_FALSE_LOWER_CASE => {
            check_json_symbol(src, "false")?;
            return Ok(src.get_pos());
        }
        crate::consts::START_OF_FALSE_UPPER_CASE => {
            check_json_symbol(src, "false")?;
            return Ok(src.get_pos());
        }

        crate::consts::START_OF_NULL_LOWER_CASE => {
            check_json_symbol(src, "null")?;
            return Ok(src.get_pos());
        }
        crate::consts::START_OF_NULL_UPPER_CASE => {
            check_json_symbol(src, "null")?;
            return Ok(src.get_pos());
        }

        crate::consts::DOUBLE_QUOTE => {
            let result = find_the_end_of_the_string(src)?;
            return Ok(result.pos + 1);
        }

        crate::consts::OPEN_BRACKET => {
            let pos = find_the_end_of_json_object_or_array(src)?;
            return Ok(pos.pos + 1);
        }

        crate::consts::OPEN_ARRAY => {
            let result = find_the_end_of_json_object_or_array(src)?;
            return Ok(result.pos + 1);
        }
        _ => {
            panic!("Somehow we are getting here")
        }
    }
}

pub fn find_the_end_of_json_object_or_array(
    src: &mut impl ArrayOfBytesIterator,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();

    let next_value = src.get_next();

    if next_value.is_none() {
        return Err(JsonParseError::new(format!(
            "Error reading value as object. Start {}. We reached the end of the payload",
            start_pos
        )));
    }

    let next_value = next_value.unwrap();

    let mut brackets = Vec::new();

    let open_open_bracket = if next_value.value == crate::consts::OPEN_BRACKET
        || next_value.value == crate::consts::OPEN_ARRAY
    {
        next_value.value
    } else {
        panic!(
            "Bug... It has to be {} or {} symbol",
            crate::consts::OPEN_BRACKET as char,
            crate::consts::OPEN_ARRAY as char
        )
    };

    brackets.push(open_open_bracket);

    while let Some(next_value) = src.get_next() {
        match next_value.value {
            crate::consts::DOUBLE_QUOTE => {
                skip_to_the_end_of_the_string(src)?;
            }
            crate::consts::OPEN_ARRAY => {
                brackets.push(next_value.value);
            }
            crate::consts::OPEN_BRACKET => {
                brackets.push(next_value.value);
            }

            crate::consts::CLOSE_BRACKET => {
                let open_bracket = brackets[brackets.len() - 1];
                if open_bracket == crate::consts::OPEN_BRACKET {
                    brackets.remove(brackets.len() - 1);
                    if brackets.len() == 0 {
                        return Ok(next_value);
                    }
                } else {
                    return Err(JsonParseError::new(format!(
                        "Error reading value as object. Start {}. Error pos {}. Open bracket '{}' does not match close bracket '{}'",
                        start_pos, next_value.pos, open_bracket as u8,  next_value.value as u8
                    )));
                }
            }

            crate::consts::CLOSE_ARRAY => {
                let open_bracket = brackets[brackets.len() - 1];
                if open_bracket == crate::consts::OPEN_ARRAY {
                    brackets.remove(brackets.len() - 1);
                    if brackets.len() == 0 {
                        return Ok(next_value);
                    }
                } else {
                    return Err(JsonParseError::new(format!(
                        "Error reading value as object. Start {}. Error pos {}. Open bracket '{}' does not match close bracket '{}'",
                        start_pos, next_value.pos, open_bracket as u8,  next_value.value as u8
                    )));
                }
            }

            _ => {}
        }
    }

    return Err(JsonParseError::new(format!(
        "Error reading value as object. Start {}. We reached the end of the payload",
        start_pos
    )));
}

pub fn find_the_end_of_json(src: &mut impl ArrayOfBytesIterator) -> Result<usize, JsonParseError> {
    skip_white_spaces_and_get_expected_token(src, ExpectedOpenJsonObjectToken)?;
    loop {
        //let start = src.get_pos();

        skip_white_spaces_and_peek_expected_token(src, ExpectedJsonObjectKeyStart)?;
        find_the_end_of_the_string(src)?;

        //println!(
        //    "{}",
        //    std::str::from_utf8(src.get_slice_to_current_pos(start)).unwrap()
        //);
        skip_white_spaces_and_get_expected_token(src, ExpectedJsonObjectKeyValueSeparator)?;

        let value_start = skip_white_spaces_and_peek_expected_token(src, ExpectedJsonValueStart)?;
        find_the_end_of_the_object_value(src, value_start.value)?;

        //println!(
        //    "{}",
        //    std::str::from_utf8(src.get_slice_to_current_pos(value_start.pos)).unwrap()
        //);

        let token = skip_white_spaces_and_get_expected_token(
            src,
            ExpectedTokenJsonObjectSeparatorOrCloseBracket,
        )?;

        if token.value == crate::consts::CLOSE_BRACKET {
            return Ok(token.pos);
        }
    }
}

pub fn find_the_end_of_array(
    src: &mut impl ArrayOfBytesIterator,
) -> Result<NextValue, JsonParseError> {
    let _open_array_token = src.get_next().unwrap();

    /*
       println!(
           "Found Open Array Token: {} at {}",
           open_array_token.value as char, open_array_token.pos
       );
    */
    loop {
        let value_start = skip_white_spaces_and_peek_expected_token(src, ExpectedJsonValueStart)?;
        find_the_end_of_the_object_value(src, value_start.value)?;

        let separator_or_end =
            skip_white_spaces_and_get_expected_token(src, ExpectedEndOfArrayOrComma)?;

        if separator_or_end.value == crate::consts::CLOSE_ARRAY {
            return Ok(separator_or_end);
        }
    }
}

pub fn next_token_must_be(raw: &mut impl ArrayOfBytesIterator, token: u8) -> FoundResult {
    while let Some(next_value) = raw.get_next() {
        if is_space(next_value.value) {
            continue;
        }

        if next_value.value == token {
            return FoundResult::Ok(next_value);
        } else {
            return FoundResult::InvalidTokenFound(next_value);
        }
    }

    return FoundResult::EndOfJson;
}

pub fn skip_white_spaces(src: &mut impl ArrayOfBytesIterator) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();
    while let Some(next_value) = src.peek_value() {
        if next_value.value > 32 {
            return Ok(next_value);
        }

        src.get_next();
    }

    Err(JsonParseError::new(format!(
        "Error skipping white spaces. Started skipping at position {}. We reached the end of the payload at {}",
        start_pos,
        src.get_pos()
    )))
}

pub fn skip_white_spaces_and_get_next(
    src: &mut impl ArrayOfBytesIterator,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();
    while let Some(next_value) = src.get_next() {
        if next_value.value > 32 {
            return Ok(next_value);
        }
    }

    Err(JsonParseError::new(format!(
        "Error skipping white spaces. Start {}. We reached the end of the payload",
        start_pos
    )))
}

pub fn skip_white_spaces_and_peek_expected_token(
    src: &mut impl ArrayOfBytesIterator,
    expected_token: impl ExpectedToken,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();
    while let Some(next_value) = src.peek_value() {
        if next_value.value > 32 {
            match expected_token.we_are_expecting_token(next_value.value) {
                Ok(_) => {
                    return Ok(next_value);
                }
                Err(err) => {
                    return Err(JsonParseError::new(format!(
                        "Expected token is not found at Pos: {}. Found token is: {}. Ex expecting one of tokens [{}]",
                        next_value.pos,
                        next_value.value as char,
                        err
                    )));
                }
            }
        }

        src.get_next();
    }

    Err(JsonParseError::new(format!(
        "Error skipping white spaces. Start {}. We reached the end of the payload",
        start_pos
    )))
}

pub fn skip_white_spaces_and_get_expected_token(
    src: &mut impl ArrayOfBytesIterator,
    expected_token: impl ExpectedToken,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();
    while let Some(next_value) = src.get_next() {
        if next_value.value > 32 {
            match expected_token.we_are_expecting_token(next_value.value) {
                Ok(_) => {
                    return Ok(next_value);
                }
                Err(err) => {
                    return Err(JsonParseError::new(format!(
                        "Expected token is not found at Pos: {}. Found token is: {}. Ex expecting one of tokens [{}]",
                        next_value.pos,
                        next_value.value as char,
                        err
                    )));
                }
            }
        }
    }

    Err(JsonParseError::new(format!(
        "Error skipping white spaces. Start {}. We reached the end of the payload",
        start_pos
    )))
}

pub fn find_the_end_of_the_string(
    src: &mut impl ArrayOfBytesIterator,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();

    src.get_next();

    while let Some(next_value) = src.get_next() {
        if next_value.value == '\\' as u8 {
            src.get_next();
            continue;
        }

        if next_value.value >= 0xF0 {
            src.get_next();
            src.get_next();
            src.get_next();
            continue;
        }
        if next_value.value >= 0xE0 {
            src.get_next();
            src.get_next();
            continue;
        }

        if next_value.value >= 0xC0 {
            src.get_next();
            continue;
        }

        if next_value.value == crate::consts::DOUBLE_QUOTE {
            return Ok(next_value);
        }
    }

    Err(JsonParseError::new(format!(
        "Error reading the end of the string. Start {}. We reached the end of the payload",
        start_pos
    )))
}

pub fn skip_to_the_end_of_the_string(
    src: &mut impl ArrayOfBytesIterator,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();

    while let Some(next_value) = src.get_next() {
        if next_value.value == '\\' as u8 {
            src.get_next();
            continue;
        }

        if next_value.value >= 0xF0 {
            src.get_next();
            src.get_next();
            src.get_next();
            continue;
        }
        if next_value.value >= 0xE0 {
            src.get_next();
            src.get_next();
            continue;
        }

        if next_value.value >= 0xC0 {
            src.get_next();
            continue;
        }

        if next_value.value == crate::consts::DOUBLE_QUOTE {
            return Ok(next_value);
        }
    }

    Err(JsonParseError::new(format!(
        "Error reading the end of the string. Start {}. We reached the end of the payload",
        start_pos
    )))
}

pub fn find_the_end_of_the_number(
    src: &mut impl ArrayOfBytesIterator,
) -> Result<NextValue, JsonParseError> {
    let pos = src.get_pos();
    while let Some(next_value) = src.peek_value() {
        if !is_number(next_value.value) {
            return Ok(next_value);
        }

        src.get_next();
    }

    Err(JsonParseError::new(format!(
        "Error reading the end of the number. Start {}. We reached the end of the payload",
        pos
    )))
}

pub fn is_number(c: u8) -> bool {
    if c >= 48 && c <= 57 {
        return true;
    }

    if c == '-' as u8 {
        return true;
    }

    if c == '+' as u8 {
        return true;
    }

    if c == '.' as u8 {
        return true;
    }

    if c == 'E' as u8 {
        return true;
    }

    if c == 'e' as u8 {
        return true;
    }

    false
}

pub fn is_space(c: u8) -> bool {
    c <= 32
}

pub fn check_json_symbol(
    src: &mut impl ArrayOfBytesIterator,
    symbol: &str,
) -> Result<(), JsonParseError> {
    let pos = src.get_pos();
    let value = src.advance(symbol.len());

    match value {
        Some(value) => {
            let value = std::str::from_utf8(value).unwrap();
            if !rust_extensions::str_utils::compare_strings_case_insensitive(value, symbol) {
                return Err(JsonParseError::new(format!(
                    "Error Parsing {symbol} value. Invalid token found ['{}'] at position {}",
                    value, pos
                )));
            }
        }
        None => {
            return Err(JsonParseError::new(format!(
                "Error Parsing {symbol}. Invalid token found ['{}'] at position {}",
                std::str::from_utf8(src.get_slice_to_end(pos)).unwrap(),
                pos
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rust_extensions::array_of_bytes_iterator::*;

    #[test]
    pub fn detect_end_of_the_string() {
        let str = " \"test\" ";
        let mut slice_iterator = SliceIterator::from_str(str);
        super::skip_white_spaces(&mut slice_iterator).unwrap();
        let result = super::find_the_end_of_the_string(&mut slice_iterator).unwrap();
        assert_eq!(6, result.pos);
    }

    #[test]
    pub fn detect_end_of_the_number() {
        let str = " 123, ";

        let mut slice_iterator = SliceIterator::from_str(str);
        super::skip_white_spaces(&mut slice_iterator).unwrap();
        let result = super::find_the_end_of_the_number(&mut slice_iterator).unwrap();
        assert_eq!(4, result.pos);
    }
}

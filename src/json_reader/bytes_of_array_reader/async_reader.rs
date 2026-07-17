use rust_extensions::array_of_bytes_iterator::*;

use crate::json_reader::JsonParseError;

use super::*;

pub async fn find_the_end_of_the_object_value(
    src: &mut impl ArrayOfBytesIteratorAsync,
    value_start: u8,
) -> Result<usize, JsonParseError> {
    if is_number(value_start) {
        let result = find_the_end_of_the_number(src).await?;
        return Ok(result.pos);
    }

    match value_start {
        crate::consts::START_OF_TRUE_LOWER_CASE => {
            check_json_symbol(src, "true").await?;
            return Ok(src.get_pos());
        }
        crate::consts::START_OF_TRUE_UPPER_CASE => {
            check_json_symbol(src, "true").await?;
            return Ok(src.get_pos());
        }

        crate::consts::START_OF_FALSE_LOWER_CASE => {
            check_json_symbol(src, "false").await?;
            return Ok(src.get_pos());
        }
        crate::consts::START_OF_FALSE_UPPER_CASE => {
            check_json_symbol(src, "false").await?;
            return Ok(src.get_pos());
        }

        crate::consts::START_OF_NULL_LOWER_CASE => {
            check_json_symbol(src, "null").await?;
            return Ok(src.get_pos());
        }
        crate::consts::START_OF_NULL_UPPER_CASE => {
            check_json_symbol(src, "null").await?;
            return Ok(src.get_pos());
        }

        crate::consts::DOUBLE_QUOTE => {
            let result = find_the_end_of_the_string(src).await?;
            return Ok(result.pos + 1);
        }

        crate::consts::OPEN_BRACKET => {
            let pos = find_the_end_of_json_object_or_array(src).await?;
            return Ok(pos.pos + 1);
        }

        crate::consts::OPEN_ARRAY => {
            let result = find_the_end_of_json_object_or_array(src).await?;
            return Ok(result.pos + 1);
        }
        _ => {
            panic!("Somehow we are getting here")
        }
    }
}

pub async fn find_the_end_of_json_object_or_array(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();

    let next_value = src.get_next().await.unwrap();

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
    //println!(
    //    "First Entrance {}: {:?}",
    //    next_value.pos,
    //    brackets.iter().map(|x| *x as char).collect::<Vec<char>>()
    //);

    while let Some(next_value) = src.get_next().await.unwrap() {
        match next_value.value {
            crate::consts::DOUBLE_QUOTE => {
                //let start = next_value.pos;
                skip_to_the_end_of_the_string(src).await?;

                // println!(
                //     "{}",
                //     String::from_utf8(src.get_slice_to_current_pos(start).await.unwrap()).unwrap()
                // );
            }
            crate::consts::OPEN_ARRAY => {
                brackets.push(next_value.value);

                //   println!(
                //       "Open Array {}: {:?}",
                //       next_value.pos,
                //       brackets.iter().map(|x| *x as char).collect::<Vec<char>>()
                //   );
            }
            crate::consts::OPEN_BRACKET => {
                brackets.push(next_value.value);
                // println!(
                //   "Open Object {}: {:?}",
                //    next_value.pos,
                //    brackets.iter().map(|x| *x as char).collect::<Vec<char>>()
                //);
            }

            crate::consts::CLOSE_BRACKET => {
                let open_bracket = brackets.remove(brackets.len() - 1);
                //println!(
                //    "Close Object {}: {:?}",
                //    next_value.pos,
                //    brackets.iter().map(|x| *x as char).collect::<Vec<char>>()
                //);

                if open_bracket == crate::consts::OPEN_BRACKET {
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
                let open_bracket = brackets.remove(brackets.len() - 1);
                // println!(
                //     "Close Array {}: {:?}",
                //     next_value.pos,
                //    brackets.iter().map(|x| *x as char).collect::<Vec<char>>()
                //);
                if open_bracket == crate::consts::OPEN_ARRAY {
                    if brackets.len() == 0 {
                        return Ok(next_value);
                    }
                } else {
                    let json_start =
                        String::from_utf8(src.get_slice_to_current_pos(start_pos).await.unwrap())
                            .unwrap();

                    //  std::fs::write("/Users/amigin/Downloads/bug.jsonl", json_start.as_str())
                    //     .unwrap();
                    return Err(JsonParseError::new(format!(
                        "Error reading value as array. Start {}. Error pos {}. Open bracket '{}' does not match close bracket '{}'. Json: {}",
                        start_pos, next_value.pos, open_bracket as u8,  next_value.value as u8,
                        &json_start[..256]
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

pub async fn find_the_end_of_json(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<usize, JsonParseError> {
    skip_white_spaces_and_get_expected_token(src, ExpectedOpenJsonObjectToken).await?;
    loop {
        // Peek, don't consume: `find_the_end_of_the_string` eats the opening quote itself. This
        // used to consume it here as well, so the quote was counted twice - harmless for a key of
        // one character or more, but an empty key (`{"":1}`) had its *closing* quote swallowed and
        // the scan then ran on into the rest of the payload. The sync twin has always peeked.
        let key_start =
            skip_white_spaces_and_peek_expected_token(src, ExpectedJsonObjectKeyStart).await?;

        // An empty object has no key to read. Mirrors the sync `find_the_end_of_json`; the `}` is
        // consumed so the position ends up past it, exactly as the non-empty path below leaves it.
        if key_start.value == crate::consts::CLOSE_BRACKET {
            src.get_next().await.unwrap();
            return Ok(key_start.pos);
        }

        find_the_end_of_the_string(src).await?;

        skip_white_spaces_and_get_expected_token(src, ExpectedJsonObjectKeyValueSeparator).await?;

        let value_start =
            skip_white_spaces_and_peek_expected_token(src, ExpectedJsonValueStart).await?;
        find_the_end_of_the_object_value(src, value_start.value).await?;

        let token = skip_white_spaces_and_get_expected_token(
            src,
            ExpectedTokenJsonObjectSeparatorOrCloseBracket,
        )
        .await?;

        if token.value == crate::consts::CLOSE_BRACKET {
            return Ok(token.pos);
        }
    }
}

pub async fn find_the_end_of_array(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<NextValue, JsonParseError> {
    let _open_array_token = src.get_next().await.unwrap().unwrap();

    // Empty array: `]` can legally come first, and peeking straight for a value start rejects it.
    // Kept in step with the sync `find_the_end_of_array`, which had the same bug.
    let first_token =
        skip_white_spaces_and_peek_expected_token(src, ExpectedJsonValueStartOrEndOfArray).await?;

    if first_token.value == crate::consts::CLOSE_ARRAY {
        // Consume it, leaving the position past the end of this array. The peek above already
        // proved the token is buffered, so this mirrors the unwrap the rest of this file uses.
        let close_token = src.get_next().await.unwrap().unwrap();
        return Ok(close_token);
    }

    loop {
        let value_start =
            skip_white_spaces_and_peek_expected_token(src, ExpectedJsonValueStart).await?;
        find_the_end_of_the_object_value(src, value_start.value).await?;

        let separator_or_end =
            skip_white_spaces_and_get_expected_token(src, ExpectedEndOfArrayOrComma).await?;

        if separator_or_end.value == crate::consts::CLOSE_ARRAY {
            return Ok(separator_or_end);
        }
    }
}

pub async fn next_token_must_be(
    raw: &mut impl ArrayOfBytesIteratorAsync,
    token: u8,
) -> FoundResult {
    while let Some(next_value) = raw.get_next().await.unwrap() {
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

pub async fn skip_white_spaces(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();
    while let Some(next_value) = src.peek_value() {
        if next_value.value > 32 {
            return Ok(next_value);
        }

        src.get_next().await.unwrap();
    }

    Err(JsonParseError::new(format!(
        "Error skipping white spaces. Start {}. We reached the end of the payload",
        start_pos
    )))
}

pub async fn skip_white_spaces_and_get_next(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();
    while let Some(next_value) = src.get_next().await.unwrap() {
        if next_value.value > 32 {
            return Ok(next_value);
        }
    }

    Err(JsonParseError::new(format!(
        "Error skipping white spaces. Start {}. We reached the end of the payload",
        start_pos
    )))
}

pub async fn skip_white_spaces_and_peek_expected_token(
    src: &mut impl ArrayOfBytesIteratorAsync,
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

        src.get_next().await.unwrap();
    }

    Err(JsonParseError::new(format!(
        "Error skipping white spaces. Start {}. We reached the end of the payload",
        start_pos
    )))
}

pub async fn skip_white_spaces_and_get_expected_token(
    src: &mut impl ArrayOfBytesIteratorAsync,
    expected_token: impl ExpectedToken,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();
    while let Some(next_value) = src.get_next().await.unwrap() {
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

pub async fn find_the_end_of_the_string(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();

    src.get_next().await.unwrap();

    while let Some(next_value) = src.get_next().await.unwrap() {
        if next_value.value == '\\' as u8 {
            src.get_next().await.unwrap();
            continue;
        }

        if next_value.value >= 0xF0 {
            src.get_next().await.unwrap();
            src.get_next().await.unwrap();
            src.get_next().await.unwrap();
            continue;
        }
        if next_value.value >= 0xE0 {
            src.get_next().await.unwrap();
            src.get_next().await.unwrap();
            continue;
        }

        if next_value.value >= 0xC0 {
            src.get_next().await.unwrap();
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

pub async fn skip_to_the_end_of_the_string(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();

    while let Some(next_value) = src.get_next().await.unwrap() {
        if next_value.value == '\\' as u8 {
            src.get_next().await.unwrap();
            continue;
        }

        if next_value.value >= 0xF0 {
            src.get_next().await.unwrap();
            src.get_next().await.unwrap();
            src.get_next().await.unwrap();
            continue;
        }
        if next_value.value >= 0xE0 {
            src.get_next().await.unwrap();
            src.get_next().await.unwrap();
            continue;
        }

        if next_value.value >= 0xC0 {
            src.get_next().await.unwrap();
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

pub async fn find_the_end_of_the_number(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<NextValue, JsonParseError> {
    let pos = src.get_pos();
    while let Some(next_value) = src.peek_value() {
        if !is_number(next_value.value) {
            return Ok(next_value);
        }

        src.get_next().await.unwrap();
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

    if c == '.' as u8 {
        return true;
    }

    false
}

pub fn is_space(c: u8) -> bool {
    c <= 32
}

pub async fn check_json_symbol(
    src: &mut impl ArrayOfBytesIteratorAsync,
    symbol: &str,
) -> Result<(), JsonParseError> {
    let pos = src.get_pos();
    let value = src.advance(symbol.len()).await.unwrap();

    match value {
        Some(value) => {
            // Untrusted bytes can be non-UTF-8 - that is a parse error, never a panic.
            match std::str::from_utf8(value.as_slice()) {
                Ok(value) => {
                    if !rust_extensions::str_utils::compare_strings_case_insensitive(value, symbol)
                    {
                        return Err(JsonParseError::new(format!(
                            "Error Parsing {symbol} value. Invalid token found ['{}'] at position {}",
                            value, pos
                        )));
                    }
                }
                Err(_) => {
                    return Err(JsonParseError::new(format!(
                        "Error Parsing {symbol} value. Non-UTF8 token found at position {}",
                        pos
                    )));
                }
            }
        }
        None => {
            return Err(JsonParseError::new(format!(
                "Error Parsing {symbol}. Invalid token found ['{}'] at position {}",
                String::from_utf8_lossy(src.get_slice_to_end(pos).await.unwrap().as_slice()),
                pos
            )));
        }
    }

    Ok(())
}

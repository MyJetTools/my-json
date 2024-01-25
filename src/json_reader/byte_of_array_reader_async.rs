use rust_extensions::array_of_bytes_iterator::*;

use super::{byte_of_array_reader::FoundResult, consts, JsonParseError};

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

    let open_open_bracket =
        if next_value.value == consts::OPEN_BRACKET || next_value.value == consts::OPEN_ARRAY {
            next_value.value
        } else {
            panic!(
                "Bug... It has to be {} or {} symbol",
                consts::OPEN_BRACKET as char,
                consts::OPEN_ARRAY as char
            )
        };

    brackets.push(open_open_bracket);

    while let Some(next_value) = src.get_next().await.unwrap() {
        match next_value.value {
            consts::DOUBLE_QUOTE => {
                find_the_end_of_the_string(src).await?;
                /*
                Ok(end_string_pos) => {
                    pos = end_string_pos;
                }
                None => {
                    return Err(JsonParseError::new(format!(
                        "Error reading value as object. Start {}. Error pos {}",
                        start_pos, pos
                    )));
                }
                 */
            }
            consts::OPEN_ARRAY => {
                brackets.push(consts::OPEN_ARRAY);
            }
            consts::OPEN_BRACKET => {
                brackets.push(consts::OPEN_ARRAY);
            }

            consts::CLOSE_BRACKET => {
                let open_bracket = brackets[brackets.len() - 1];
                if open_bracket == consts::OPEN_BRACKET {
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

            consts::CLOSE_ARRAY => {
                let open_bracket = brackets[brackets.len() - 1];
                if open_bracket == consts::OPEN_ARRAY {
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

/*
pub fn read_string(src: &impl ArrayOfBytesIterator) -> Option<usize> {
    let mut esc_mode = false;
    let mut pos = start_pos + 1;
    while pos < raw.len() {
        let b = raw[pos];

        if esc_mode {
            esc_mode = false;
        } else {
            match b {
                consts::ESC_SYMBOL => {
                    esc_mode = true;
                }
                consts::DOUBLE_QUOTE => {
                    return Some(pos);
                }
                _ => {}
            }
        }

        pos += 1;
    }

    return None;
}
 */

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

pub async fn skip_white_spaces_and_extract_value(
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

pub async fn find_the_end_of_the_string(
    src: &mut impl ArrayOfBytesIteratorAsync,
) -> Result<NextValue, JsonParseError> {
    let start_pos = src.get_pos();

    src.get_next().await.unwrap();

    while let Some(next_value) = src.get_next().await.unwrap() {
        if next_value.value == '/' as u8 {
            src.get_next().await.unwrap();
            continue;
        }

        if next_value.value == consts::DOUBLE_QUOTE {
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
            let value = std::str::from_utf8(value.as_slice()).unwrap();
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
                String::from_utf8(src.get_slice_to_end(pos).await.unwrap()).unwrap(),
                pos
            )));
        }
    }

    Ok(())
}

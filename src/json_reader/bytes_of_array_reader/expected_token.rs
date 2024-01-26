use crate::json_reader::consts;

use super::sync_reader::is_number;

pub trait ExpectedToken {
    fn we_are_expecting_token(&self, token: u8) -> Result<(), String>;
}

pub struct ExpectedOpenJsonObjectToken;

impl ExpectedToken for ExpectedOpenJsonObjectToken {
    fn we_are_expecting_token(&self, token: u8) -> Result<(), String> {
        if token == consts::OPEN_BRACKET {
            return Ok(());
        }

        Err(format!("{}", consts::OPEN_BRACKET as char,))
    }
}

pub struct ExpectedTokenJsonObjectSeparatorOrCloseBracket;

impl ExpectedToken for ExpectedTokenJsonObjectSeparatorOrCloseBracket {
    fn we_are_expecting_token(&self, token: u8) -> Result<(), String> {
        if token == consts::CLOSE_BRACKET || token == consts::COMMA {
            return Ok(());
        }

        Err(format!(
            "{} or {}",
            consts::COMMA as char,
            consts::CLOSE_BRACKET as char,
        ))
    }
}

pub struct ExpectedEndOfArrayOrComma;

impl ExpectedToken for ExpectedEndOfArrayOrComma {
    fn we_are_expecting_token(&self, token: u8) -> Result<(), String> {
        if token == consts::CLOSE_ARRAY || token == consts::COMMA {
            return Ok(());
        }

        Err(format!(
            "{} or {}",
            consts::COMMA as char,
            consts::CLOSE_ARRAY as char,
        ))
    }
}

pub struct ExpectedJsonObjectKeyStart;

impl ExpectedToken for ExpectedJsonObjectKeyStart {
    fn we_are_expecting_token(&self, token: u8) -> Result<(), String> {
        if token == consts::DOUBLE_QUOTE {
            return Ok(());
        }

        Err(format!("{}", consts::DOUBLE_QUOTE as char,))
    }
}

pub struct ExpectedJsonObjectKeyValueSeparator;

impl ExpectedToken for ExpectedJsonObjectKeyValueSeparator {
    fn we_are_expecting_token(&self, token: u8) -> Result<(), String> {
        if token == consts::DOUBLE_COLUMN {
            return Ok(());
        }

        Err(format!("{}", consts::DOUBLE_COLUMN as char,))
    }
}

pub struct ExpectedJsonValueStart;

impl ExpectedToken for ExpectedJsonValueStart {
    fn we_are_expecting_token(&self, token: u8) -> Result<(), String> {
        if token == consts::DOUBLE_QUOTE
            || token == consts::OPEN_BRACKET
            || token == consts::OPEN_ARRAY
            || token == consts::START_OF_FALSE_LOWER_CASE
            || token == consts::START_OF_FALSE_UPPER_CASE
            || token == consts::START_OF_TRUE_LOWER_CASE
            || token == consts::START_OF_TRUE_UPPER_CASE
            || token == consts::START_OF_NULL_LOWER_CASE
            || token == consts::START_OF_NULL_UPPER_CASE
            || is_number(token)
        {
            return Ok(());
        }

        Err(format!("Start of Volume"))
    }
}

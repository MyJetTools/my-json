pub const ESC_SYMBOL: u8 = '\\' as u8;
pub const DOUBLE_QUOTE: u8 = '"' as u8;
pub const SINGLE_QUOTE: u8 = '\'' as u8;
pub const OPEN_BRACKET: u8 = '{' as u8;
pub const CLOSE_BRACKET: u8 = '}' as u8;
pub const DOUBLE_COLUMN: u8 = ':' as u8;

pub const OPEN_ARRAY: u8 = '[' as u8;
pub const CLOSE_ARRAY: u8 = ']' as u8;
pub const COMMA: u8 = ',' as u8;

pub const START_OF_TRUE_UPPER_CASE: u8 = 'T' as u8;
pub const START_OF_TRUE_LOWER_CASE: u8 = 't' as u8;

pub const START_OF_FALSE_UPPER_CASE: u8 = 'F' as u8;
pub const START_OF_FALSE_LOWER_CASE: u8 = 'f' as u8;

pub const START_OF_NULL_UPPER_CASE: u8 = 'N' as u8;
pub const START_OF_NULL_LOWER_CASE: u8 = 'n' as u8;

pub static EMPTY_ARRAY: &'static [u8] = &[OPEN_ARRAY, CLOSE_ARRAY];

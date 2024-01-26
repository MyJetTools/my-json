pub mod async_reader;
mod expected_token;
pub mod sync_reader;
pub use expected_token::*;
use rust_extensions::array_of_bytes_iterator::NextValue;
pub enum FoundResult {
    Ok(NextValue),
    EndOfJson,
    InvalidTokenFound(NextValue),
}

use super::JsonParseError;

/// What reading a JSON out of a stream of bytes fails with.
#[derive(Debug)]
pub enum JsonStreamError<TError> {
    /// The stream failed to give its next chunk. The error is the stream's own, as it is.
    Stream(TError),
    /// The bytes the stream gave are not the JSON that was expected.
    Json(JsonParseError),
}

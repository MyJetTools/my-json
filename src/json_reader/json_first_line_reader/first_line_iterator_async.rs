use rust_extensions::AsyncBytesStream;

use super::JsonFirstLineReaderInner;
use crate::json_reader::json_stream_scanner::JsonStreamScanner;
use crate::json_reader::{JsonFieldNameRef, JsonStreamError, JsonValueRef};

/// Reads the keys of a JSON object and their values out of a stream of bytes, as the bytes
/// arrive - the object is never held in memory as a whole.
///
/// A key and its value are read by the code `JsonFirstLineIterator` runs and handed out the way
/// that one hands them out - here into the buffer the stream is read into, with no copy made of
/// them. How the chunks of the stream are put together is up to `JsonStreamScanner`.
pub struct JsonFirstLineIteratorAsync<TError, TStream: AsyncBytesStream<TError>> {
    scanner: JsonStreamScanner<TError, TStream>,
}

impl<TError, TStream: AsyncBytesStream<TError>> JsonFirstLineIteratorAsync<TError, TStream> {
    /// Nothing is read until `get_next()` is called.
    pub fn new(stream: TStream) -> Self {
        Self {
            scanner: JsonStreamScanner::new(stream),
        }
    }

    /// The next key of the object and its value, `None` once the object is closed. The value
    /// comes once all of it is read.
    ///
    /// - The key and the value are borrowed from the buffer of the iterator, and the next call
    ///   drops them from there.
    /// - A stream error is returned as it is. Nothing read so far is lost, and the next call asks
    ///   the stream again.
    /// - A JSON error ends the reading: every call after it returns `None`.
    /// - Once the `}` of the object is read, the stream is not asked for more.
    pub async fn get_next(
        &mut self,
    ) -> Option<Result<(JsonFieldNameRef<'_>, JsonValueRef<'_>), JsonStreamError<TError>>> {
        let item = self
            .scanner
            .next_item(|not_parsed, initialized| {
                JsonFirstLineReaderInner::resume(not_parsed, initialized).get_next()
            })
            .await?;

        Some(item.map(|(item, json)| {
            (
                JsonFieldNameRef::new(item.name, json),
                JsonValueRef::new(item.value, json),
            )
        }))
    }
}

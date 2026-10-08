use rust_extensions::AsyncBytesStream;

use super::array_iterator_inner::JsonArrayIteratorInner;
use super::json_stream_scanner::JsonStreamScanner;
use super::{JsonStreamError, JsonValueRef};

/// Reads a JSON array out of a stream of bytes item by item, as the bytes arrive - the array is
/// never held in memory as a whole.
///
/// An item is read by the code `JsonArrayIterator` runs and handed out the way that one hands it
/// out, as a `JsonValueRef` - here into the buffer the stream is read into, with no copy made of
/// it. How the chunks of the stream are put together is up to `JsonStreamScanner`.
pub struct JsonArrayIteratorAsync<TError, TStream: AsyncBytesStream<TError>> {
    scanner: JsonStreamScanner<TError, TStream>,
}

impl<TError, TStream: AsyncBytesStream<TError>> JsonArrayIteratorAsync<TError, TStream> {
    /// Nothing is read until `get_next()` is called.
    pub fn new(stream: TStream) -> Self {
        Self {
            scanner: JsonStreamScanner::new(stream),
        }
    }

    /// The next item of the array, `None` once the array is closed.
    ///
    /// - The item is borrowed from the buffer of the iterator, and the next call drops it from
    ///   there. A copy is `item.as_slice().to_vec()`.
    /// - A stream error is returned as it is. Nothing read so far is lost, and the next call asks
    ///   the stream again.
    /// - A JSON error ends the reading: every call after it returns `None`.
    /// - Once the `]` of the array is read, the stream is not asked for more.
    pub async fn get_next(&mut self) -> Option<Result<JsonValueRef<'_>, JsonStreamError<TError>>> {
        let item = self
            .scanner
            .next_item(|not_parsed, initialized| {
                JsonArrayIteratorInner::resume(not_parsed, initialized).get_next()
            })
            .await?;

        Some(item.map(|(item, json)| JsonValueRef::new(item, json)))
    }
}

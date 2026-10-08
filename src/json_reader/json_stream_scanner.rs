use std::cell::Cell;

use rust_extensions::array_of_bytes_iterator::{ArrayOfBytesIterator, NextValue};
use rust_extensions::{AsyncBytesStream, BufferedReader};

use super::{JsonKeyValue, JsonParseError, JsonStreamError, JsonValue};

/// Reads a JSON out of a stream of bytes one item at a time - an item of an array, a key and its
/// value of an object - as the bytes arrive, so the JSON is never held in memory as a whole.
///
/// The stream is cut into chunks wherever it happens to be, so an item may begin in one chunk and
/// end in another. A `BufferedReader` keeps the bytes that are read but not parsed yet as one
/// slice, and that slice is scanned by the sync reader of the JSON - the very code the sync
/// iterators run, so both read a JSON the same way and fail on it with the same error. When the
/// scan runs out of bytes before the item ends, more of the stream is read in and the item is
/// scanned again from its start.
///
/// An item is handed out as positions in the buffer of the reader, not as a copy. It stays in the
/// buffer while it is borrowed and is dropped from it on the next call. So the buffer holds the
/// item and the rest of the chunk it ends in, and its room is reused for the items after it.
pub(crate) struct JsonStreamScanner<TError, TStream: AsyncBytesStream<TError>> {
    reader: BufferedReader<TError, TStream>,
    // Position in the stream of the first byte that is not parsed yet
    pos: usize,
    // The size of the item handed out last - it is parsed once the next one is asked for
    handed_out: usize,
    // The bracket that opens the JSON is behind
    initialized: bool,
    // Nothing is left to read: the JSON is closed, or it turned out not to be the one expected
    finished: bool,
    // `false` - the last scan ran out of bytes, and what was read since can not have made the
    // item complete
    item_may_be_complete: bool,
    item_end: ItemEndTracker,
}

impl<TError, TStream: AsyncBytesStream<TError>> JsonStreamScanner<TError, TStream> {
    /// Nothing is read until an item is asked for.
    pub fn new(stream: TStream) -> Self {
        Self {
            reader: BufferedReader::new(stream),
            pos: 0,
            handed_out: 0,
            initialized: false,
            finished: false,
            item_may_be_complete: true,
            item_end: ItemEndTracker::default(),
        }
    }

    /// The next item with the slice its positions are in, `None` once the JSON is closed.
    ///
    /// `scan` is the sync reader of the JSON: it is given the bytes that are not parsed yet and
    /// whether the opening bracket is behind, and reads one item out of them.
    ///
    /// - A stream error is returned as it is. Nothing read so far is lost, and the next call asks
    ///   the stream again.
    /// - A JSON error ends the reading: every call after it returns `None`.
    /// - Once the end of the JSON is read, the stream is not asked for more.
    pub async fn next_item<TItem: StreamItem>(
        &mut self,
        scan: impl Fn(NotParsedBytes<'_>, bool) -> Option<Result<TItem, JsonParseError>>,
    ) -> Option<Result<(TItem, &[u8]), JsonStreamError<TError>>> {
        // The item handed out last is not borrowed any more
        self.reader.mark_as_read(self.handed_out);
        self.pos += self.handed_out;
        self.handed_out = 0;

        if self.finished {
            return None;
        }

        loop {
            if self.item_may_be_complete {
                match self.scan(&scan) {
                    Scan::Item(item) => {
                        self.handed_out = item.get_end();
                        self.initialized = true;
                        self.item_end = ItemEndTracker::default();

                        return Some(Ok((item, self.reader.as_slice())));
                    }
                    Scan::End => {
                        self.finished = true;
                        return None;
                    }
                    Scan::Error(err) => {
                        self.finished = true;
                        return Some(Err(JsonStreamError::Json(err)));
                    }
                    Scan::NotEnoughBytes => {
                        self.item_may_be_complete = false;
                        self.item_end.scanned_in_vain = self.reader.as_slice().len();
                    }
                }
            }

            // What `get_next()` returns shares the buffer of the reader - it is dropped right
            // here, so the next chunk is put into that buffer in place.
            if let Err(err) = self.reader.get_next().await {
                return Some(Err(JsonStreamError::Stream(err)));
            }

            // Before its opening bracket is read, the JSON itself is one of the brackets that are
            // open around the item
            let top_level = if self.initialized { 0 } else { 1 };

            // Once the stream is over there is nothing to wait for: the scan tells what is wrong
            self.item_may_be_complete = !self.reader.read_mode()
                || self
                    .item_end
                    .may_be_complete(self.reader.as_slice(), top_level);
        }
    }

    /// Looks for the next item in the bytes that are read already. Changes nothing, so a scan
    /// that ran out of bytes is simply repeated once there are more of them.
    fn scan<TItem: StreamItem>(
        &self,
        scan: &impl Fn(NotParsedBytes<'_>, bool) -> Option<Result<TItem, JsonParseError>>,
    ) -> Scan<TItem> {
        let out_of_data = Cell::new(false);
        let not_parsed = NotParsedBytes::new(self.reader.as_slice(), self.pos, &out_of_data);

        let item = scan(not_parsed, self.initialized);

        // Whatever the scan came to, it has not seen the whole of what it was reading, and the
        // stream may still have the rest. Once the stream is over the result is final: it is what
        // the sync reader makes of a JSON that is cut short.
        if out_of_data.get() && self.reader.read_mode() {
            return Scan::NotEnoughBytes;
        }

        match item {
            Some(Ok(mut item)) => {
                item.shift_back(self.pos);
                Scan::Item(item)
            }
            Some(Err(err)) => Scan::Error(err),
            None => Scan::End,
        }
    }
}

/// What a scan of the not parsed bytes came to.
enum Scan<TItem> {
    /// With its positions in the not parsed bytes
    Item(TItem),
    End,
    Error(JsonParseError),
    /// The scan asked for bytes the stream has not given yet
    NotEnoughBytes,
}

/// What the sync readers make of an item: positions of its parts in the bytes they read.
pub(crate) trait StreamItem {
    /// Where the item ends - the bytes up to there are parsed once it is handed out.
    fn get_end(&self) -> usize;

    /// Moves the positions from the stream to the not parsed bytes, which begin at `offset`.
    fn shift_back(&mut self, offset: usize);
}

impl StreamItem for JsonValue {
    fn get_end(&self) -> usize {
        self.end
    }

    fn shift_back(&mut self, offset: usize) {
        self.start -= offset;
        self.end -= offset;
    }
}

impl StreamItem for JsonKeyValue {
    // The value goes after the key
    fn get_end(&self) -> usize {
        self.value.end
    }

    fn shift_back(&mut self, offset: usize) {
        self.name.start -= offset;
        self.name.end -= offset;
        self.value.shift_back(offset);
    }
}

/// Tells when the item at the start of the not parsed bytes is worth scanning for again.
///
/// A scan starts from the first of those bytes every time. Repeated on every chunk of an item
/// that spans many of them, it costs the square of the item's size. This follows the brackets and
/// the strings instead - each byte once, however the chunks are cut - and the scan is repeated
/// only when the item can be complete: what was opened since its start is closed.
///
/// It decides when to scan, never what an item is. A string ends where the sync reader ends it
/// (`find_the_end_of_the_string`), so on a valid JSON an item is scanned for as soon as its last
/// byte is read. A broken JSON may never close what it opened, so the scan is also repeated every
/// time the buffer doubles - that is as late as an error can come.
#[derive(Default)]
struct ItemEndTracker {
    // How many of the not parsed bytes are looked at
    looked_at: usize,
    // Brackets that are open
    depth: usize,
    in_string: bool,
    // Bytes of a string to pass by: what follows a `\` or the first byte of a UTF-8 char
    to_skip: u8,
    // How many not parsed bytes there were when the last scan ran out of them
    scanned_in_vain: usize,
}

impl ItemEndTracker {
    /// `top_level` is how many brackets are open around the item.
    fn may_be_complete(&mut self, not_parsed: &[u8], top_level: usize) -> bool {
        // Doubled - the scan is due whatever is in the bytes. They are not looked at here, which
        // keeps this out of the way of items that end in the chunk after the one they begin in.
        if not_parsed.len() >= self.scanned_in_vain * 2 {
            return true;
        }

        let mut result = false;

        for value in &not_parsed[self.looked_at..] {
            if self.to_skip > 0 {
                self.to_skip -= 1;
                continue;
            }

            if self.in_string {
                match *value {
                    b'\\' => self.to_skip = 1,
                    0xF0..=0xFF => self.to_skip = 3,
                    0xE0..=0xEF => self.to_skip = 2,
                    0xC0..=0xDF => self.to_skip = 1,
                    crate::consts::DOUBLE_QUOTE => self.in_string = false,
                    _ => {}
                }
            } else {
                match *value {
                    crate::consts::DOUBLE_QUOTE => self.in_string = true,
                    crate::consts::OPEN_BRACKET | crate::consts::OPEN_ARRAY => self.depth += 1,
                    crate::consts::CLOSE_BRACKET | crate::consts::CLOSE_ARRAY => {
                        self.depth = self.depth.saturating_sub(1)
                    }
                    _ => {}
                }
            }

            // Outside of everything the item could have opened: it may have ended right here
            if !self.in_string && self.depth <= top_level {
                result = true;
            }
        }

        self.looked_at = not_parsed.len();

        result
    }
}

/// The bytes of a stream that are read but not parsed yet, for the sync reader to walk through.
///
/// Positions are the ones of the stream, not of what is left of it in the buffer, so an error
/// names the place in the stream - the very place the sync iterator names for the same JSON.
/// `get_src_slice()` is the one thing positions do not index: it is the buffer as it is.
///
/// `out_of_data` is raised once a byte behind the last one is asked for. Whatever the reader came
/// to after that, it has not seen the whole of what it was reading.
pub(crate) struct NotParsedBytes<'s> {
    slice: &'s [u8],
    // Position in the stream of the first byte of the slice
    offset: usize,
    pos: Cell<usize>,
    out_of_data: &'s Cell<bool>,
}

impl<'s> NotParsedBytes<'s> {
    fn new(slice: &'s [u8], offset: usize, out_of_data: &'s Cell<bool>) -> Self {
        Self {
            slice,
            offset,
            pos: Cell::new(offset),
            out_of_data,
        }
    }

    /// `size` bytes of the stream from `pos`. `None` - not all of them are read yet.
    fn get_bytes(&self, pos: usize, size: usize) -> Option<&[u8]> {
        let from = pos - self.offset;
        let result = self.slice.get(from..from + size);

        if result.is_none() {
            self.out_of_data.set(true);
        }

        result
    }
}

impl ArrayOfBytesIterator for NotParsedBytes<'_> {
    fn get_pos(&self) -> usize {
        self.pos.get()
    }

    fn set_pos(&self, pos: usize) {
        self.pos.set(pos);
    }

    fn get_src_slice(&self) -> &[u8] {
        self.slice
    }

    fn peek_value(&self) -> Option<NextValue> {
        let pos = self.pos.get();
        let value = self.get_bytes(pos, 1)?[0];

        Some(NextValue { pos, value })
    }

    fn get_next(&self) -> Option<NextValue> {
        let result = self.peek_value()?;
        self.pos.set(result.pos + 1);

        Some(result)
    }

    fn get_slice_to_current_pos(&self, from_pos: usize) -> &[u8] {
        &self.slice[from_pos - self.offset..self.pos.get() - self.offset]
    }

    fn get_slice_to_end(&self, from_pos: usize) -> &[u8] {
        &self.slice[from_pos - self.offset..]
    }

    fn peek_sequence(&self, size: usize, sub_seq: impl Fn(&[u8]) -> bool) -> bool {
        match self.get_bytes(self.pos.get(), size) {
            Some(sequence) => sub_seq(sequence),
            None => false,
        }
    }

    fn peek_and_find_sequence_pos(&self, mut pos: usize, sequence_to_find: &[u8]) -> Option<usize> {
        // Running out of bytes raises `out_of_data` as well: the sequence may begin in the bytes
        // that are here and end in the ones that are not.
        while let Some(sequence) = self.get_bytes(pos, sequence_to_find.len()) {
            if sequence == sequence_to_find {
                return Some(pos);
            }

            pos += 1;
        }

        None
    }

    fn advance(&self, amount: usize) -> Option<&[u8]> {
        let pos = self.pos.get();
        let result = self.get_bytes(pos, amount)?;
        self.pos.set(pos + amount);

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use rust_extensions::array_of_bytes_iterator::{ArrayOfBytesIterator, SliceIterator};

    use super::{ItemEndTracker, NotParsedBytes, StreamItem};
    use crate::json_reader::bytes_of_array_reader::sync_reader;
    use crate::json_reader::{JsonKeyValue, JsonValue};

    /// A tracker for which the buffer never doubles - it tells what the bytes themselves say.
    fn tracker() -> ItemEndTracker {
        ItemEndTracker {
            scanned_in_vain: usize::MAX / 2,
            ..Default::default()
        }
    }

    /// The sizes of `src` at which the item may be complete, `src` coming in chunks of
    /// `chunk_size` bytes.
    fn may_be_complete_at(src: &[u8], chunk_size: usize, top_level: usize) -> Vec<usize> {
        let mut tracker = tracker();

        (1..=src.len())
            .filter(|size| size % chunk_size == 0 || *size == src.len())
            .filter(|size| tracker.may_be_complete(&src[..*size], top_level))
            .collect()
    }

    #[test]
    fn an_item_in_brackets_may_be_complete_once_they_are_closed() {
        // The `,` before the item is outside of it - nothing is known to be open there yet
        assert_eq!(may_be_complete_at(br#",{"a":1}"#, 1, 0), [1, 8]);
        assert_eq!(may_be_complete_at(br#",[1,[2,{"a":[]}],3]"#, 1, 0), [1, 19]);
        assert_eq!(may_be_complete_at(br#",{}"#, 1, 0), [1, 3]);

        // The array itself is open around its first item
        assert_eq!(may_be_complete_at(br#"[{"a":1}"#, 1, 1), [1, 8]);
        assert_eq!(may_be_complete_at(br#" [ {"a":[1]}"#, 1, 1), [1, 2, 3, 12]);
    }

    /// A key and its value: the key closes nothing, so a value that is a string or has brackets
    /// may be complete at its last byte - and at the bytes between the two.
    #[test]
    fn a_key_and_its_value_may_be_complete_once_the_value_is_closed() {
        assert_eq!(may_be_complete_at(br#","a":{"b":1}"#, 1, 0), [1, 4, 5, 12]);
        assert_eq!(may_be_complete_at(br#","a":"}""#, 1, 0), [1, 4, 5, 8]);

        // The object itself is open around its first key
        assert_eq!(may_be_complete_at(br#"{"a":[1]}"#, 1, 1), [1, 4, 5, 8, 9]);
    }

    #[test]
    fn brackets_and_quotes_inside_a_string_close_nothing() {
        assert_eq!(may_be_complete_at(br#",{"a":"}]"}"#, 1, 0), [1, 11]);
        assert_eq!(may_be_complete_at(br#",{"a":"\"}"}"#, 1, 0), [1, 12]);
        assert_eq!(may_be_complete_at(br#",{"a":"\\"}"#, 1, 0), [1, 11]);
        assert_eq!(may_be_complete_at(br#",{"}":["]"]}"#, 1, 0), [1, 12]);
    }

    #[test]
    fn an_item_with_no_brackets_may_be_complete_at_any_byte_outside_a_string() {
        assert_eq!(may_be_complete_at(b",12,", 1, 0), [1, 2, 3, 4]);
        assert_eq!(may_be_complete_at(b",true]", 1, 0), [1, 2, 3, 4, 5, 6]);
        assert_eq!(may_be_complete_at(br#","a]b","#, 1, 0), [1, 6, 7]);
    }

    #[test]
    fn it_does_not_matter_where_the_chunks_are_cut() {
        let src = r#",{"a":[1,{"b":"}]\""}],"c":"\\","d":"привет"}"#.as_bytes();

        let by_a_byte = may_be_complete_at(src, 1, 0);
        assert_eq!(by_a_byte, [1, src.len()]);

        for chunk_size in 2..=src.len() {
            // A chunk tells what its bytes do
            let expected: Vec<usize> = (1..=src.len())
                .filter(|size| size % chunk_size == 0 || *size == src.len())
                .filter(|size| {
                    let chunk_start = (size - 1) / chunk_size * chunk_size;
                    by_a_byte.iter().any(|at| chunk_start < *at && at <= size)
                })
                .collect();

            assert_eq!(
                may_be_complete_at(src, chunk_size, 0),
                expected,
                "chunks of {} bytes",
                chunk_size
            );
        }
    }

    /// The scan decides where a string ends, and it passes by what follows a `\` or the first
    /// byte of a UTF-8 char whatever that is - a quote too. The tracker must end the string at
    /// the very same byte, or it would wait for an item the scan takes for complete.
    #[test]
    fn a_string_ends_where_the_sync_reader_ends_it() {
        for src in [
            &b"\"abc\""[..],
            b"\"a\\\"b\"",
            b"\"a\\\\\"",
            b"\"\xD0\xBF\xD1\x80\"",
            b"\"\xF0\x9F\x98\x80\"",
            // Not UTF-8: the byte after the first one of a char is passed by, the quote as well
            b"\"\xC3\"x\"",
            b"\"\xE2\"\"x\"",
            b"\"\xF0\"\"\"x\"",
            b"\"\xC3\\\"",
        ] {
            let closing_quote = sync_reader::find_the_end_of_the_string(&SliceIterator::new(src))
                .unwrap()
                .pos;

            assert_eq!(
                may_be_complete_at(src, 1, 0).first(),
                Some(&(closing_quote + 1)),
                "reading {:?}",
                String::from_utf8_lossy(src)
            );
        }
    }

    #[test]
    fn the_item_may_be_complete_every_time_the_buffer_doubles() {
        let src = br#",{"a":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#;

        let mut tracker = ItemEndTracker::default();
        let mut result = Vec::new();

        for size in 1..=src.len() {
            if tracker.may_be_complete(&src[..size], 0) {
                result.push(size);
                // The scan runs out of bytes again - the item is not complete yet
                tracker.scanned_in_vain = size;
            }
        }

        // 1 and 2 double what there was. 3 is the `,` the buffer begins with: it is looked at for
        // the first time, a doubling looks at nothing. Then the doublings go on - and whatever
        // they have passed by, the end of the item is found where it is.
        assert_eq!(result, [1, 2, 3, 6, 12, 24, src.len()]);
        assert_eq!(src.len(), 39);
    }

    #[test]
    fn an_item_is_moved_from_the_stream_to_the_not_parsed_bytes() {
        let mut value = JsonValue::new(105, 108);
        value.shift_back(100);
        assert_eq!((value.start, value.end, value.get_end()), (5, 8, 8));

        let mut key_value = JsonKeyValue::new(105, 108, 109, 112);
        key_value.shift_back(100);
        assert_eq!((key_value.name.start, key_value.name.end), (5, 8));
        assert_eq!((key_value.value.start, key_value.value.end), (9, 12));
        assert_eq!(key_value.get_end(), 12);
    }

    /// Whether `read` asks for a byte behind the last one of `cdef` - what is left in the buffer
    /// of a stream whose first 100 bytes are parsed already - reading from `pos`.
    fn is_out_of_data(pos: usize, read: impl Fn(&NotParsedBytes)) -> bool {
        let out_of_data = Cell::new(false);
        let src = NotParsedBytes::new(b"cdef", 100, &out_of_data);
        src.set_pos(pos);

        read(&src);

        out_of_data.get()
    }

    #[test]
    fn positions_are_the_ones_of_the_stream() {
        let out_of_data = is_out_of_data(100, |src| {
            assert_eq!(src.get_pos(), 100);

            let next = src.peek_value().unwrap();
            assert_eq!((next.pos, next.value), (100, b'c'));

            let next = src.get_next().unwrap();
            assert_eq!((next.pos, next.value), (100, b'c'));
            assert_eq!(src.get_pos(), 101);

            assert_eq!(src.advance(2), Some(b"de".as_slice()));
            assert_eq!(src.get_pos(), 103);

            assert_eq!(src.get_slice_to_current_pos(101), b"de");
            assert_eq!(src.get_slice_to_end(101), b"def");

            assert!(src.peek_sequence(1, |sequence| sequence == b"f"));
            assert_eq!(src.peek_and_find_sequence_pos(100, b"ef"), Some(102));
            assert_eq!(src.peek_and_find_sequence_pos_from_current_pos(b"f"), Some(103));
        });

        // All of it was inside the buffer
        assert!(!out_of_data);
    }

    #[test]
    fn the_last_byte_is_not_out_of_data() {
        let out_of_data = is_out_of_data(100, |src| {
            assert_eq!(src.advance(3), Some(b"cde".as_slice()));
            assert_eq!(src.get_next().unwrap().value, b'f');
            assert_eq!(src.get_pos(), 104);
        });

        assert!(!out_of_data);
    }

    #[test]
    fn a_byte_behind_the_last_one_is_out_of_data() {
        assert!(is_out_of_data(104, |src| assert!(src.peek_value().is_none())));

        assert!(is_out_of_data(104, |src| {
            assert!(src.get_next().is_none());
            assert_eq!(src.get_pos(), 104);
        }));

        // Fewer bytes than asked for - the position stays, as the trait has it
        assert!(is_out_of_data(102, |src| {
            assert_eq!(src.advance(3), None);
            assert_eq!(src.get_pos(), 102);
        }));

        assert!(is_out_of_data(102, |src| {
            assert!(!src.peek_sequence(3, |_| true))
        }));

        // `fg` may be there once the next chunk is read in
        assert!(is_out_of_data(100, |src| {
            assert_eq!(src.peek_and_find_sequence_pos(100, b"fg"), None)
        }));
    }
}

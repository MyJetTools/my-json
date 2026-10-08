//! The async array iterator reads a stream that is cut into chunks wherever it happens to be, so
//! an item may begin in one chunk and end in another.
//!
//! It scans with the code the sync iterator has, and these tests hold it to that: the same payload
//! must produce the same items, or the same error, whichever path reads it and however the chunks
//! are cut.

mod common;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use common::{block_on, Chunks};
use my_json::json_reader::{JsonArrayIteratorAsync, JsonStreamError, JsonValueRef};
use rust_extensions::{AsyncBytesStream, DoubleBufferError};

/// Items as strings, or the error - the shape both paths are compared on.
fn as_comparable<TError: std::fmt::Display>(
    item: Result<JsonValueRef<'_>, JsonStreamError<TError>>,
) -> Result<String, String> {
    match item {
        Ok(item) => Ok(String::from_utf8_lossy(item.as_slice()).to_string()),
        Err(JsonStreamError::Json(err)) => Err(err.to_string()),
        Err(JsonStreamError::Stream(err)) => Err(format!("Stream: {}", err)),
    }
}

fn read_async(stream: Chunks) -> Vec<Result<String, String>> {
    read_from::<String, _>(stream)
}

/// Reads until the iterator says the array is over - an error does not stop the loop, the
/// iterator has to.
fn read_from<TError: std::fmt::Display, TStream: AsyncBytesStream<TError>>(
    stream: TStream,
) -> Vec<Result<String, String>> {
    block_on(async {
        let mut iterator = JsonArrayIteratorAsync::new(stream);

        let mut result = Vec::new();

        while let Some(item) = iterator.get_next().await {
            result.push(as_comparable(item));
        }

        result
    })
}

fn read_sync(json: &[u8]) -> Vec<Result<String, String>> {
    let iterator = my_json::json_reader::JsonArrayIterator::new(json).unwrap();

    let mut result = Vec::new();

    while let Some(item) = iterator.get_next() {
        match item {
            Ok(value) => result.push(Ok(String::from_utf8_lossy(value.as_slice()).to_string())),
            Err(err) => {
                result.push(Err(err.to_string()));
                break;
            }
        }
    }

    result
}

/// The async iterator must read `json` the way the sync one does wherever the chunks are cut:
/// every chunk size from a single byte up to the whole payload.
fn assert_async_agrees_with_sync(json: &[u8]) {
    let expected = read_sync(json);

    for chunk_size in 1..=json.len() {
        assert_eq!(
            expected,
            read_async(Chunks::cut(json, chunk_size)),
            "sync and async disagree on {} cut into chunks of {} bytes",
            String::from_utf8_lossy(json),
            chunk_size
        );
    }
}

#[test]
fn async_and_sync_agree_on_objects_and_arrays() {
    for json in [
        r#"[{"a":1}]"#,
        r#"[{"a":1},{"b":2,"c":3}]"#,
        r#"[1,2,3]"#,
        r#"["a","b"]"#,
        r#"[true,false,null]"#,
        r#"[{"a":{"b":1}}]"#,
        r#"[[1,2],[3]]"#,
        r#"  [ {"a" : 1} , [ 1 , 2 ] , "str" , 12.5 , true , null ]  "#,
        r#"["chat message",123,{"name":"chat"}, true, null]"#,
    ] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// What looks like the end of an item inside a string is not one, wherever the string is cut.
#[test]
fn async_and_sync_agree_on_strings() {
    for json in [
        r#"["]","}",","]"#,
        r#"[{"a":"}]"},{"b":"[{"}]"#,
        r#"["a\"b","\\","\\\"",""]"#,
        r#"[{"k\"ey":"va\\lue"}]"#,
        r#"["привет",{"ключ":"значение"},"😀"]"#,
    ] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// A number ends where the byte after it says so - which may be in the next chunk. The exponent
/// forms are the ones the old async path did not take for numbers at all.
#[test]
fn async_and_sync_agree_on_numbers() {
    for json in [
        "[1,22,333]",
        "[-1,0.5,-2.25]",
        "[1e5,-2.5E-3,1E+2]",
        "[ 12345 ]",
        "[[19313.0,2.7731]]",
    ] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// Regression: `find_the_end_of_array` demanded a value straight after the `[`, so an empty array
/// nested in an array was unreadable.
#[test]
fn async_and_sync_agree_on_empty_arrays() {
    for json in ["[]", "[ ]", "[[]]", "[[],[1]]", "[[1],[]]", "[[],[]]", "[[], [ ] ]"] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// Regression: async `find_the_end_of_json` never checked for the `}` of an empty object, so
/// `[{}]` failed on the async path while the sync path read it fine.
#[test]
fn async_and_sync_agree_on_empty_objects() {
    for json in ["[{}]", "[{},{\"a\":1}]", "[{},{}]", "[ {  } ]", "[{},[]]"] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// Regression: async consumed the key's opening quote and then let `find_the_end_of_the_string`
/// consume it again, which swallowed the closing quote of an empty key.
#[test]
fn async_and_sync_agree_on_empty_and_short_keys() {
    for json in [
        r#"[{"":1}]"#,
        r#"[{"a":1}]"#,
        r#"[{"":1,"a":2}]"#,
        r#"[{"":""}]"#,
    ] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// The payload the sync iterator is tested on from a file, exponents and line breaks included.
#[test]
fn async_and_sync_agree_on_the_payload_from_the_file() {
    let json = std::fs::read("test.json").unwrap();

    assert!(!read_sync(&json).is_empty());
    assert_async_agrees_with_sync(&json);
}

/// A JSON that is broken, or cut short by the end of the stream, is the same error on both paths.
/// The position in it is the one in the stream - not in what is left in the buffer after the
/// items before it are dropped.
#[test]
fn async_and_sync_agree_on_malformed_json() {
    for json in [
        r#"{"a":1}"#,
        r#"[{"a":1},{"c" 3}]"#,
        r#"[{"a":1},{"b":{]}]"#,
        r#"[{"a":1},[1,[2}]]"#,
        r#"[1,2,tru]"#,
        r#"[1,2,nul"#,
        r#"[1,2,x]"#,
        r#"[1 2]"#,
        r#"[{"a":1} {"b":2}]"#,
        r#"[{"a":1},{"b":2}"#,
        r#"[{"a":1},{"b":"#,
        r#"[1, 2"#,
        r#"[1,"abc"#,
        r#"["#,
        r#"[1,"#,
    ] {
        let sync_result = read_sync(json.as_bytes());

        assert!(
            sync_result.last().unwrap().is_err(),
            "sync should report an error on {}, got {:?}",
            json,
            sync_result
        );

        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// Regression: a debug `println!` in the async object parser did `from_utf8(..).unwrap()` on the
/// key, turning a non-UTF-8 payload into a panic (a DoS on hostile input) where the sync path
/// returns a plain error. Reading must never panic, whichever path does it.
#[test]
fn non_utf8_key_is_an_error_on_both_paths_not_a_panic() {
    let mut json = Vec::new();
    json.extend_from_slice(b"[{\"");
    json.push(0xFF);
    json.extend_from_slice(b"\":1}]");

    let sync_result = read_sync(&json);

    assert!(
        sync_result.iter().any(|item| item.is_err()),
        "sync should report an error, got {:?}",
        sync_result
    );

    assert_async_agrees_with_sync(&json);
}

#[test]
fn an_item_cut_between_chunks_is_returned_whole() {
    let stream = Chunks::of(&["[{\"id\":1},{\"i", "d\":2", "},{\"id\"", ":3}]"]);

    assert_eq!(
        read_async(stream),
        [
            Ok(r#"{"id":1}"#.to_string()),
            Ok(r#"{"id":2}"#.to_string()),
            Ok(r#"{"id":3}"#.to_string())
        ]
    );
}

/// An item is not copied out of the buffer of the reader: the items that came in one chunk are
/// where they are in it, one after another. They are read further with the sync API as they are.
#[test]
fn an_item_is_not_copied() {
    let json = br#"[{"id":1},"str",[1,2]]"#;

    block_on(async {
        let mut iterator = JsonArrayIteratorAsync::new(Chunks::cut(json, json.len()));

        let mut items = Vec::new();

        while let Some(item) = iterator.get_next().await {
            let item = item.unwrap();

            if item.is_object() {
                let object = item.unwrap_as_object().unwrap();
                let (key, value) = object.get_next().unwrap().unwrap();

                assert_eq!(key.as_str().unwrap().as_str(), "id");
                assert_eq!(value.as_slice(), b"1");
            }

            let text = String::from_utf8_lossy(item.as_slice()).to_string();
            items.push((item.as_slice().as_ptr(), text));
        }

        let texts: Vec<_> = items.iter().map(|(_, text)| text.as_str()).collect();
        assert_eq!(texts, [r#"{"id":1}"#, r#""str""#, "[1,2]"]);

        // An item begins a byte - the `,` - after the end of the one before it
        for pair in items.windows(2) {
            let (before, before_text) = &pair[0];
            assert_eq!(before.wrapping_add(before_text.len() + 1), pair[1].0);
        }
    });
}

/// A `DoubleBuffer` is the stream the readers are made for: a socket is read into one of its two
/// buffers while the other is parsed. A buffer is read into again only once its chunk is dropped,
/// and the iterator holds none of them - however the items are cut by the buffers.
#[test]
fn async_and_sync_agree_over_a_double_buffer() {
    let json = std::fs::read("test.json").unwrap();
    let expected = read_sync(&json);

    for buffer_size in [1, 2, 3, 7, 64, 4096] {
        assert_eq!(
            expected,
            read_from::<DoubleBufferError, _>(common::double_buffer(&json, buffer_size)),
            "buffers of {} bytes",
            buffer_size
        );
    }
}

/// An item is handed over as soon as its last byte is read - not once the stream has more to give.
/// A stream that waits for the item to be answered before it sends the next one never has.
///
/// The scan is not repeated on every chunk of an item, which is where an item could get held
/// back. With a byte per chunk, what the stream was asked for is exactly what was read.
#[test]
fn an_item_is_returned_as_soon_as_its_last_byte_is_read() {
    let from_the_file = std::fs::read_to_string("test.json").unwrap();

    for json in [
        r#"[{"a":1},{"b":[1,2,{"c":"}]"}]},"str",[1,[2,3]],true,null,false,{},[]]"#,
        r#"[1,22,{"a":1},333,"4",-5.5e3]"#,
        r#"["a\"]b",{"k":"\\"},"привет",{"ключ":["😀","}"]},"\\\""]"#,
        "  [ \n  {\"a\" : 1} ,\n\n  [ 1 , 2 ] ,   \"str\"  ,  null  ]  ",
        r#"[{"id":1,"name":"a name long enough for the buffer to double more than once on the way","tags":["a","b","c"]},{"id":2}]"#,
        from_the_file.as_str(),
    ] {
        let json = json.as_bytes();

        // Where the items end, by the sync iterator
        let sync_iterator = my_json::json_reader::JsonArrayIterator::new(json).unwrap();
        let mut item_ends = Vec::new();

        while let Some(item) = sync_iterator.get_next() {
            item_ends.push(item.unwrap().data.end);
        }

        assert!(!item_ends.is_empty());

        let stream = Chunks::cut(json, 1);
        let requests = stream.requests.clone();

        block_on(async {
            let mut iterator = JsonArrayIteratorAsync::new(stream);

            for item_end in item_ends {
                let item = iterator.get_next().await.unwrap().unwrap();
                let item = item.as_slice();

                // A number is the one item that ends by the byte which goes after it
                let is_number = item[0] == b'-' || item[0].is_ascii_digit();
                let expected = if is_number { item_end + 1 } else { item_end };

                assert_eq!(
                    requests.load(Ordering::Relaxed),
                    expected,
                    "reading {} of {}",
                    String::from_utf8_lossy(item),
                    String::from_utf8_lossy(json)
                );
            }

            assert!(iterator.get_next().await.is_none());
        });
    }
}

/// Nothing is there to scan before the first chunk, and nothing ever comes - that is a JSON which
/// is not an array, not a reason to panic.
#[test]
fn a_stream_with_no_array_in_it_is_an_error_not_a_panic() {
    for chunks in [&[][..], &[""][..], &["  ", " \n"][..]] {
        let result = read_async(Chunks::of(chunks));

        assert_eq!(result.len(), 1, "reading {:?}: {:?}", chunks, result);
        assert!(result[0].is_err(), "reading {:?}: {:?}", chunks, result);
    }
}

/// A stream that breaks is not a JSON error and not a panic: the error is handed over as it is.
/// Nothing that was read is lost, so the reading goes on if the stream does.
#[test]
fn a_stream_error_is_returned_as_it_is_and_the_reading_goes_on() {
    let stream = Chunks::new(vec![
        Ok(b"[{\"id\":1},{\"i".to_vec()),
        Err("no connection".to_string()),
        Ok(b"d\":2}]".to_vec()),
    ]);

    block_on(async {
        let mut iterator = JsonArrayIteratorAsync::new(stream);

        let item = iterator.get_next().await.unwrap().unwrap();
        assert_eq!(item.as_slice(), br#"{"id":1}"#);

        match iterator.get_next().await {
            Some(Err(JsonStreamError::Stream(err))) => assert_eq!(err, "no connection"),
            other => panic!("expected the error of the stream, got {:?}", other),
        }

        let item = iterator.get_next().await.unwrap().unwrap();
        assert_eq!(item.as_slice(), br#"{"id":2}"#);

        assert!(iterator.get_next().await.is_none());
    });
}

/// The `]` ends the array: what goes after it is not asked for - it may never come.
#[test]
fn nothing_is_read_past_the_end_of_the_array() {
    let stream = Chunks::of(&["[1,2]", "what goes after the array"]);
    let requests = stream.requests.clone();

    block_on(async {
        let mut iterator = JsonArrayIteratorAsync::new(stream);

        // Nothing is read until it is asked for
        assert_eq!(requests.load(Ordering::Relaxed), 0);

        assert_eq!(iterator.get_next().await.unwrap().unwrap().as_slice(), b"1");
        assert_eq!(iterator.get_next().await.unwrap().unwrap().as_slice(), b"2");
        assert!(iterator.get_next().await.is_none());

        // The array is over, and it stays over
        assert!(iterator.get_next().await.is_none());
        assert_eq!(requests.load(Ordering::Relaxed), 1);
    });
}

/// There is no way back into a JSON after an error in it - the reading is over, and the stream is
/// left alone.
#[test]
fn a_json_error_ends_the_reading() {
    let stream = Chunks::of(&["[1,x,", "3]"]);
    let requests = stream.requests.clone();

    block_on(async {
        let mut iterator = JsonArrayIteratorAsync::new(stream);

        assert_eq!(iterator.get_next().await.unwrap().unwrap().as_slice(), b"1");

        match iterator.get_next().await {
            Some(Err(JsonStreamError::Json(_))) => {}
            other => panic!("expected a JSON error, got {:?}", other),
        }

        assert!(iterator.get_next().await.is_none());
        assert!(iterator.get_next().await.is_none());
        assert_eq!(requests.load(Ordering::Relaxed), 1);
    });
}

/// The iterator and the future of `get_next()` are `Send` - it is read from a spawned task, and
/// the stream may come as an `Arc<dyn AsyncBytesStream + Send + Sync>`.
#[test]
fn the_iterator_is_read_from_another_thread() {
    fn assert_send<T: Send>(value: T) -> T {
        value
    }

    let stream: Arc<dyn AsyncBytesStream<String, Chunk = Vec<u8>> + Send + Sync + 'static> =
        Arc::new(Chunks::of(&["[1,", "2]"]));

    let mut iterator = assert_send(JsonArrayIteratorAsync::new(stream));

    let result = std::thread::spawn(move || {
        block_on(async {
            let mut result = Vec::new();

            while let Some(item) = assert_send(iterator.get_next()).await {
                result.push(as_comparable(item));
            }

            result
        })
    })
    .join()
    .unwrap();

    assert_eq!(result, [Ok("1".to_string()), Ok("2".to_string())]);
}

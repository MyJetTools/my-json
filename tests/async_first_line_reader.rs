//! The async first-line reader reads the keys of an object and their values out of a stream that
//! is cut into chunks wherever it happens to be, so a key or a value may begin in one chunk and end
//! in another.
//!
//! It scans with the code `JsonFirstLineIterator` has, and these tests hold it to that: the same
//! payload must produce the same keys and values, or the same error, whichever path reads it and
//! however the chunks are cut.

mod common;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use common::{block_on, Chunks};
use my_json::json_reader::{
    JsonFieldNameRef, JsonFirstLineIterator, JsonFirstLineIteratorAsync, JsonStreamError,
    JsonValueRef,
};
use rust_extensions::{AsyncBytesStream, DoubleBufferError};

/// A key and its value as they are in the JSON, or the error - the shape both paths are compared
/// on.
type Comparable = Result<(String, String), String>;

fn as_comparable<TError: std::fmt::Display>(
    item: Result<(JsonFieldNameRef<'_>, JsonValueRef<'_>), JsonStreamError<TError>>,
) -> Comparable {
    match item {
        Ok((key, value)) => Ok((
            String::from_utf8_lossy(key.as_slice()).to_string(),
            String::from_utf8_lossy(value.as_slice()).to_string(),
        )),
        Err(JsonStreamError::Json(err)) => Err(err.to_string()),
        Err(JsonStreamError::Stream(err)) => Err(format!("Stream: {}", err)),
    }
}

fn read_async(stream: Chunks) -> Vec<Comparable> {
    read_from::<String, _>(stream)
}

/// Reads until the reader says the object is over - an error does not stop the loop, the reader
/// has to.
fn read_from<TError: std::fmt::Display, TStream: AsyncBytesStream<TError>>(
    stream: TStream,
) -> Vec<Comparable> {
    block_on(async {
        let mut reader = JsonFirstLineIteratorAsync::new(stream);

        let mut result = Vec::new();

        while let Some(item) = reader.get_next().await {
            result.push(as_comparable(item));
        }

        result
    })
}

fn read_sync(json: &[u8]) -> Vec<Comparable> {
    let reader = JsonFirstLineIterator::new(json);

    let mut result = Vec::new();

    while let Some(item) = reader.get_next() {
        match item {
            Ok((key, value)) => result.push(Ok((
                String::from_utf8_lossy(key.as_slice()).to_string(),
                String::from_utf8_lossy(value.as_slice()).to_string(),
            ))),
            Err(err) => {
                result.push(Err(err.to_string()));
                break;
            }
        }
    }

    result
}

/// The async reader must read `json` the way the sync one does wherever the chunks are cut: every
/// chunk size from a single byte up to the whole payload.
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
fn async_and_sync_agree_on_keys_and_values() {
    for json in [
        r#"{"a":1}"#,
        r#"{"a":1,"b":"text","c":[1,2],"d":{"e":null},"f":true,"g":false,"h":null}"#,
        r#"  { "a" : 1 ,  "b" : [ 1 , 2 ] , "c" : { "d" : "e" } }  "#,
        r#"{"a":{"b":{"c":[1,[2,{"d":[]}]]}}}"#,
        r#"{"a":TRUE,"b":False,"c":NULL}"#,
        r#"{"a":1,}"#,
    ] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// What looks like the end of a key or a value inside a string is not one, wherever it is cut.
#[test]
fn async_and_sync_agree_on_strings() {
    for json in [
        r#"{"}":"}","]":"]",",":",",":":":"}"#,
        r#"{"a\"b":"c\"d","\\":"\\\""}"#,
        r#"{"ключ":"значение","emoji":"😀","mixed":"при\"вет"}"#,
        r#"{"a":["}",{"b":"]"}]}"#,
    ] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// A number ends where the byte after it says so - which may be in the next chunk.
#[test]
fn async_and_sync_agree_on_numbers() {
    for json in [
        r#"{"a":1,"b":22,"c":333}"#,
        r#"{"a":-1,"b":0.5,"c":-2.25}"#,
        r#"{"a":1e5,"b":-2.5E-3,"c":1E+2}"#,
        r#"{"a":12345}"#,
        r#"{ "a" : 12345 }"#,
    ] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

#[test]
fn async_and_sync_agree_on_empty_objects_and_keys() {
    for json in [
        "{}",
        "{ }",
        r#"{"":1}"#,
        r#"{"":""}"#,
        r#"{"a":{}}"#,
        r#"{"a":[]}"#,
        r#"{"a":{},"b":[]}"#,
    ] {
        assert_async_agrees_with_sync(json.as_bytes());
    }
}

/// A value that is a whole array - the payload the sync array iterator is tested on.
#[test]
fn async_and_sync_agree_on_a_big_value() {
    let from_the_file = std::fs::read_to_string("test.json").unwrap();
    let json = format!(r#"{{"before":1,"items":{},"after":"end"}}"#, from_the_file);

    assert_eq!(read_sync(json.as_bytes()).len(), 3);
    assert_async_agrees_with_sync(json.as_bytes());
}

/// A JSON that is broken, or cut short by the end of the stream, is the same error on both paths.
/// The position in it is the one in the stream - not in what is left in the buffer after the keys
/// before it are dropped.
#[test]
fn async_and_sync_agree_on_malformed_json() {
    for json in [
        r#"[1,2]"#,
        r#"x"#,
        r#"{"a" 1}"#,
        r#"{"a":1 "b":2}"#,
        r#"{"a":}"#,
        r#"{"a":tru}"#,
        r#"{"a":[1,2}"#,
        r#"{"a":{"b":1]}"#,
        r#"{"a":1"#,
        r#"{"a":"te"#,
        r#"{"a":1,"b"#,
        r#"{"a"#,
        r#"{"#,
        r#"{"a":1,"#,
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

/// Untrusted bytes are an error on both paths, never a panic.
#[test]
fn non_utf8_bytes_are_an_error_on_both_paths_not_a_panic() {
    for json in [&b"{\"\xFF\":1}"[..], b"{\"a\":\"\xFF\"}", b"{\"a\":1,\"\xC3\":2}"] {
        let sync_result = read_sync(json);

        assert!(
            sync_result.iter().any(|item| item.is_err()),
            "sync should report an error on {:?}, got {:?}",
            String::from_utf8_lossy(json),
            sync_result
        );

        assert_async_agrees_with_sync(json);
    }
}

#[test]
fn a_key_and_a_value_cut_between_chunks_are_returned_whole() {
    let stream = Chunks::of(&["{\"na", "me\":\"Jo", "hn\",\"tags\":[\"a\",", "\"b\"]", "}"]);

    assert_eq!(
        read_async(stream),
        [
            Ok((r#""name""#.to_string(), r#""John""#.to_string())),
            Ok((r#""tags""#.to_string(), r#"["a","b"]"#.to_string()))
        ]
    );
}

/// A key and its value are not copied out of the buffer of the reader: what came in one chunk is
/// where it is in it - a value a byte, the `:`, after its key, and a key a byte, the `,`, after
/// the value before it. They are read further with the sync API as they are.
#[test]
fn a_key_and_a_value_are_not_copied() {
    let json = br#"{"user":{"name":"John"},"tags":["a","b"]}"#;

    block_on(async {
        let mut reader = JsonFirstLineIteratorAsync::new(Chunks::cut(json, json.len()));

        let (key, value) = reader.get_next().await.unwrap().unwrap();
        let (user_key, user_value) = (key.as_slice().as_ptr(), value.as_slice().as_ptr());
        assert_eq!(user_key.wrapping_add(key.as_slice().len() + 1), user_value);
        let user_value_end = user_value.wrapping_add(value.as_slice().len());

        assert_eq!(key.as_str().unwrap().as_str(), "user");
        let user = value.unwrap_as_object().unwrap();
        let (key, value) = user.get_next().unwrap().unwrap();
        assert_eq!(key.as_str().unwrap().as_str(), "name");
        assert_eq!(value.as_str().unwrap().as_str(), "John");

        let (key, value) = reader.get_next().await.unwrap().unwrap();
        assert_eq!(user_value_end.wrapping_add(1), key.as_slice().as_ptr());
        assert_eq!(
            key.as_slice().as_ptr().wrapping_add(key.as_slice().len() + 1),
            value.as_slice().as_ptr()
        );

        assert_eq!(key.as_str().unwrap().as_str(), "tags");
        let tags = value.unwrap_as_array().unwrap();
        assert_eq!(tags.get_next().unwrap().unwrap().as_str().unwrap().as_str(), "a");
        assert_eq!(tags.get_next().unwrap().unwrap().as_str().unwrap().as_str(), "b");
        assert!(tags.get_next().is_none());

        assert!(reader.get_next().await.is_none());
    });
}

/// A `DoubleBuffer` is the stream the readers are made for: a socket is read into one of its two
/// buffers while the other is parsed. A buffer is read into again only once its chunk is dropped,
/// and the reader holds none of them - however the keys and values are cut by the buffers.
#[test]
fn async_and_sync_agree_over_a_double_buffer() {
    let from_the_file = std::fs::read_to_string("test.json").unwrap();
    let json = format!(r#"{{"before":1,"items":{},"after":"end"}}"#, from_the_file);
    let expected = read_sync(json.as_bytes());

    for buffer_size in [1, 2, 3, 7, 64, 4096] {
        assert_eq!(
            expected,
            read_from::<DoubleBufferError, _>(common::double_buffer(json.as_bytes(), buffer_size)),
            "buffers of {} bytes",
            buffer_size
        );
    }
}

/// A key and its value are handed over as soon as the last byte of the value is read - not once
/// the stream has more to give. A stream that waits for an answer before it sends more never has.
///
/// With a byte per chunk, what the stream was asked for is exactly what was read.
#[test]
fn a_value_is_returned_as_soon_as_its_last_byte_is_read() {
    let from_the_file = std::fs::read_to_string("test.json").unwrap();
    let big_value = format!(r#"{{"before":1,"items":{},"after":"end"}}"#, from_the_file);

    for json in [
        r#"{"a":{"b":[1,2,{"c":"}]"}]},"s":"str","n":[1,[2,3]],"t":true,"nu":null,"f":false,"e":{},"ea":[]}"#,
        r#"{"a":1,"b":22,"c":{"d":1},"e":333,"f":"4","g":-5.5e3}"#,
        r#"{"a\"]b":"x","k":"\\","ru":"привет","obj":{"ключ":["😀","}"]},"q":"\\\""}"#,
        "  { \n  \"a\" : 1 ,\n\n  \"b\" : [ 1 , 2 ] ,   \"c\"  :  \"str\"  ,  \"d\" :  null  }  ",
        big_value.as_str(),
    ] {
        let json = json.as_bytes();

        // Where the values end, by the sync reader
        let sync_reader = JsonFirstLineIterator::new(json);
        let mut value_ends = Vec::new();

        while let Some(item) = sync_reader.get_next() {
            value_ends.push(item.unwrap().1.data.end);
        }

        assert!(!value_ends.is_empty());

        let stream = Chunks::cut(json, 1);
        let requests = stream.requests.clone();

        block_on(async {
            let mut reader = JsonFirstLineIteratorAsync::new(stream);

            for value_end in value_ends {
                let (key, value) = reader.get_next().await.unwrap().unwrap();
                let value = value.as_slice();

                // A number is the one value that ends by the byte which goes after it
                let is_number = value[0] == b'-' || value[0].is_ascii_digit();
                let expected = if is_number { value_end + 1 } else { value_end };

                assert_eq!(
                    requests.load(Ordering::Relaxed),
                    expected,
                    "reading {} of {}",
                    String::from_utf8_lossy(key.as_slice()),
                    String::from_utf8_lossy(json)
                );
            }

            assert!(reader.get_next().await.is_none());
        });
    }
}

/// Nothing is there to scan before the first chunk, and nothing ever comes - that is a JSON which
/// is not an object, not a reason to panic.
#[test]
fn a_stream_with_no_object_in_it_is_an_error_not_a_panic() {
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
        Ok(b"{\"a\":1,\"b\":\"te".to_vec()),
        Err("no connection".to_string()),
        Ok(b"xt\"}".to_vec()),
    ]);

    block_on(async {
        let mut reader = JsonFirstLineIteratorAsync::new(stream);

        let (key, value) = reader.get_next().await.unwrap().unwrap();
        assert_eq!((key.as_slice(), value.as_slice()), (&b"\"a\""[..], &b"1"[..]));

        match reader.get_next().await {
            Some(Err(JsonStreamError::Stream(err))) => assert_eq!(err, "no connection"),
            other => panic!("expected the error of the stream, got {:?}", other.is_some()),
        }

        let (key, value) = reader.get_next().await.unwrap().unwrap();
        assert_eq!((key.as_slice(), value.as_slice()), (&b"\"b\""[..], &b"\"text\""[..]));

        assert!(reader.get_next().await.is_none());
    });
}

/// The `}` ends the object: what goes after it is not asked for - it may never come.
#[test]
fn nothing_is_read_past_the_end_of_the_object() {
    let stream = Chunks::of(&["{\"a\":1,\"b\":2}", "what goes after the object"]);
    let requests = stream.requests.clone();

    block_on(async {
        let mut reader = JsonFirstLineIteratorAsync::new(stream);

        // Nothing is read until it is asked for
        assert_eq!(requests.load(Ordering::Relaxed), 0);

        assert_eq!(reader.get_next().await.unwrap().unwrap().1.as_slice(), b"1");
        assert_eq!(reader.get_next().await.unwrap().unwrap().1.as_slice(), b"2");
        assert!(reader.get_next().await.is_none());

        // The object is over, and it stays over
        assert!(reader.get_next().await.is_none());
        assert_eq!(requests.load(Ordering::Relaxed), 1);
    });
}

/// There is no way back into a JSON after an error in it - the reading is over, and the stream is
/// left alone.
#[test]
fn a_json_error_ends_the_reading() {
    let stream = Chunks::of(&["{\"a\":1,x", "\"b\":2}"]);
    let requests = stream.requests.clone();

    block_on(async {
        let mut reader = JsonFirstLineIteratorAsync::new(stream);

        assert_eq!(reader.get_next().await.unwrap().unwrap().1.as_slice(), b"1");

        match reader.get_next().await {
            Some(Err(JsonStreamError::Json(_))) => {}
            other => panic!("expected a JSON error, got {:?}", other.is_some()),
        }

        assert!(reader.get_next().await.is_none());
        assert!(reader.get_next().await.is_none());
        assert_eq!(requests.load(Ordering::Relaxed), 1);
    });
}

/// The reader and the future of `get_next()` are `Send` - it is read from a spawned task, and the
/// stream may come as an `Arc<dyn AsyncBytesStream + Send + Sync>`.
#[test]
fn the_reader_is_read_from_another_thread() {
    fn assert_send<T: Send>(value: T) -> T {
        value
    }

    let stream: Arc<dyn AsyncBytesStream<String, Chunk = Vec<u8>> + Send + Sync + 'static> =
        Arc::new(Chunks::of(&["{\"a\":", "1}"]));

    let mut reader = assert_send(JsonFirstLineIteratorAsync::new(stream));

    let result = std::thread::spawn(move || {
        block_on(async {
            let mut result = Vec::new();

            while let Some(item) = assert_send(reader.get_next()).await {
                result.push(as_comparable(item));
            }

            result
        })
    })
    .join()
    .unwrap();

    assert_eq!(result, [Ok((r#""a""#.to_string(), "1".to_string()))]);
}

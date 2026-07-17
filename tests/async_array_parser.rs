//! The async array parser had no test coverage at all, which is how a debug `println!` (that
//! panicked on non-UTF-8 input) and an empty-object bug both survived in it.
//!
//! These tests drive the async parser against the sync one and require the two to agree: the same
//! payload must produce the same elements, or the same error, whichever path reads it.

use my_json::json_reader::JsonArrayIteratorAsync;
use rust_extensions::array_of_bytes_iterator::{ArrayOfBytesIteratorAsync, NextValue};

/// A slice-backed async iterator whose operations all resolve immediately, so the futures never
/// pend and can be driven by the trivial executor below without pulling in a runtime.
struct MockAsyncIterator {
    slice: Vec<u8>,
    pos: usize,
}

impl MockAsyncIterator {
    fn new(src: &[u8]) -> Self {
        Self {
            slice: src.to_vec(),
            pos: 0,
        }
    }
}

#[async_trait::async_trait]
impl ArrayOfBytesIteratorAsync for MockAsyncIterator {
    fn peek_value(&self) -> Option<NextValue> {
        if self.pos < self.slice.len() {
            Some(NextValue {
                pos: self.pos,
                value: self.slice[self.pos],
            })
        } else {
            None
        }
    }

    async fn get_next(&mut self) -> std::io::Result<Option<NextValue>> {
        if self.pos < self.slice.len() {
            let result = NextValue {
                pos: self.pos,
                value: self.slice[self.pos],
            };
            self.pos += 1;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    fn get_pos(&self) -> usize {
        self.pos
    }

    async fn get_slice_to_current_pos(&self, from_pos: usize) -> std::io::Result<Vec<u8>> {
        Ok(self.slice[from_pos..self.pos].to_vec())
    }

    async fn get_slice_to_end(&self, from_pos: usize) -> std::io::Result<Vec<u8>> {
        Ok(self.slice[from_pos..].to_vec())
    }

    async fn advance(&mut self, amount: usize) -> std::io::Result<Option<Vec<u8>>> {
        if self.pos + amount <= self.slice.len() {
            let result = self.slice[self.pos..self.pos + amount].to_vec();
            self.pos += amount;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

/// The mock's futures are ready on the first poll, so a real runtime is unnecessary.
fn block_on<F: std::future::Future>(mut fut: F) -> F::Output {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);

    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    let mut fut = unsafe { std::pin::Pin::new_unchecked(&mut fut) };

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => panic!("the mock iterator never pends"),
        }
    }
}

/// Elements as strings, or the first error - the shape both paths are compared on.
fn read_async(json: &[u8]) -> Vec<Result<String, String>> {
    block_on(async {
        let mut iterator = JsonArrayIteratorAsync::new(MockAsyncIterator::new(json)).await;

        let mut result = Vec::new();

        while let Some(item) = iterator.get_next().await {
            match item {
                Ok(bytes) => result.push(Ok(String::from_utf8_lossy(&bytes).to_string())),
                Err(err) => {
                    result.push(Err(err.to_string()));
                    break;
                }
            }
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
    ] {
        assert_eq!(
            read_sync(json.as_bytes()),
            read_async(json.as_bytes()),
            "sync and async disagree on {}",
            json
        );
    }
}

/// Regression: `find_the_end_of_array` demanded a value straight after the `[`, so an empty array
/// nested in an array was unreadable.
#[test]
fn async_and_sync_agree_on_empty_arrays() {
    for json in ["[[]]", "[[],[1]]", "[[1],[]]", "[[],[]]", "[[], [ ] ]"] {
        assert_eq!(
            read_sync(json.as_bytes()),
            read_async(json.as_bytes()),
            "sync and async disagree on {}",
            json
        );
    }
}

/// Regression: async `find_the_end_of_json` never checked for the `}` of an empty object, so
/// `[{}]` failed on the async path while the sync path read it fine.
#[test]
fn async_and_sync_agree_on_empty_objects() {
    for json in ["[{}]", "[{},{\"a\":1}]", "[{},{}]", "[ {  } ]", "[{},[]]"] {
        assert_eq!(
            read_sync(json.as_bytes()),
            read_async(json.as_bytes()),
            "sync and async disagree on {}",
            json
        );
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
        assert_eq!(
            read_sync(json.as_bytes()),
            read_async(json.as_bytes()),
            "sync and async disagree on {}",
            json
        );
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
    let async_result = read_async(&json);

    assert!(
        sync_result.iter().any(|item| item.is_err()),
        "sync should report an error, got {:?}",
        sync_result
    );
    assert_eq!(
        sync_result, async_result,
        "sync and async disagree on a non-UTF-8 key"
    );
}

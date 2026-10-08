//! What the tests of the async readers share: a stream that hands over the chunks it is made of,
//! and an executor for it.

#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use rust_extensions::bytes::Bytes;
use rust_extensions::AsyncBytesStream;

/// A stream that hands over the chunks it was made of. It resolves immediately, so the futures
/// never pend and can be driven by `block_on` below without pulling in a runtime.
pub struct Chunks {
    chunks: Mutex<VecDeque<Result<Bytes, String>>>,
    /// How many times the stream was asked for a chunk
    pub requests: Arc<AtomicUsize>,
}

impl Chunks {
    pub fn new(chunks: Vec<Result<Bytes, String>>) -> Self {
        Self {
            chunks: Mutex::new(chunks.into()),
            requests: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn of(chunks: &[&str]) -> Self {
        Self::new(
            chunks
                .iter()
                .map(|chunk| Ok(Bytes::from(chunk.as_bytes().to_vec())))
                .collect(),
        )
    }

    /// `src` cut into chunks of `chunk_size` bytes.
    pub fn cut(src: &[u8], chunk_size: usize) -> Self {
        Self::new(
            src.chunks(chunk_size)
                .map(|chunk| Ok(Bytes::from(chunk.to_vec())))
                .collect(),
        )
    }
}

#[async_trait::async_trait]
impl AsyncBytesStream<String> for Chunks {
    async fn get_next(&self) -> Result<Option<Bytes>, String> {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.chunks.lock().unwrap().pop_front().transpose()
    }

    fn get_size(&self) -> Option<usize> {
        None
    }
}

/// The stream's futures are ready on the first poll, so a real runtime is unnecessary.
pub fn block_on<F: std::future::Future>(mut fut: F) -> F::Output {
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
            Poll::Pending => panic!("the stream of chunks never pends"),
        }
    }
}

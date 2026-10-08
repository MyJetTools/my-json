//! What the tests of the async readers share: the streams they read, and an executor to read them
//! with.

#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use rust_extensions::{AsyncBytesStream, DoubleBuffer, DoubleBufferReader};

/// A stream that hands over the chunks it was made of. It resolves immediately.
pub struct Chunks {
    chunks: Mutex<VecDeque<Result<Vec<u8>, String>>>,
    /// How many times the stream was asked for a chunk
    pub requests: Arc<AtomicUsize>,
}

impl Chunks {
    pub fn new(chunks: Vec<Result<Vec<u8>, String>>) -> Self {
        Self {
            chunks: Mutex::new(chunks.into()),
            requests: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn of(chunks: &[&str]) -> Self {
        Self::new(
            chunks
                .iter()
                .map(|chunk| Ok(chunk.as_bytes().to_vec()))
                .collect(),
        )
    }

    /// `src` cut into chunks of `chunk_size` bytes.
    pub fn cut(src: &[u8], chunk_size: usize) -> Self {
        Self::new(src.chunks(chunk_size).map(|chunk| Ok(chunk.to_vec())).collect())
    }
}

#[async_trait::async_trait]
impl AsyncBytesStream<String> for Chunks {
    type Chunk = Vec<u8>;

    async fn get_next(&self) -> Result<Option<Vec<u8>>, String> {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.chunks.lock().unwrap().pop_front().transpose()
    }

    fn get_size(&self) -> Option<usize> {
        None
    }
}

/// The stream a socket is read into: `src` is written into a `DoubleBuffer` with buffers of
/// `buffer_size` bytes, from a thread of its own.
///
/// A buffer is written into again once the chunk read out of it is dropped - so a reader that
/// held on to a chunk while it waited for the next one would wait forever.
pub fn double_buffer(src: &[u8], buffer_size: usize) -> DoubleBufferReader {
    let (writer, reader) = DoubleBuffer::new(buffer_size);
    let src = src.to_vec();

    std::thread::spawn(move || {
        block_on(async move {
            let mut src = src.as_slice();

            loop {
                // Err - the reader is dropped: nobody is going to process what is written
                let Ok(mut buffer) = writer.get_buffer_to_read().await else {
                    return;
                };

                let size = src.len().min(buffer.len());

                if size == 0 {
                    // `buffer` borrows the writer - it goes first
                    drop(buffer);
                    writer.finish();
                    return;
                }

                buffer[..size].copy_from_slice(&src[..size]);
                buffer.send(size);
                src = &src[size..];
            }
        })
    });

    reader
}

/// Runs the future on this thread, which sleeps while the future waits and is woken up by whoever
/// it waits for - no runtime is needed.
pub fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::task::{Context, Poll, Wake, Waker};

    struct Unpark(std::thread::Thread);

    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::park(),
        }
    }
}

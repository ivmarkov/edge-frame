use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct WasmFuture<T> {
    fut: Pin<Box<dyn Future<Output = T> + 'static>>,
}

impl<T: Send> WasmFuture<T> {
    pub fn new<F: Future<Output = T> + 'static>(fut: F) -> Pin<Box<Self>> {
        Box::pin(Self { fut: Box::pin(fut) })
    }
}

// This is safe because WASM doesn't have threads yet. Once WASM supports threads we should use a
// thread to park the blocking implementation until it's been completed.
unsafe impl<T: Send> Send for WasmFuture<T> {}

impl<T: Send> Future for WasmFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This is safe because we're only using this future as a pass-through for the inner
        // future, in order to implement `Send`. If it's safe to poll the inner future, it's safe
        // to proxy it too.
        unsafe { Pin::new_unchecked(&mut self.fut).poll(cx) }
    }
}
